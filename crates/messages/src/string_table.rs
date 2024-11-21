use crate::size::u16_to_usize;
use log::trace;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_with_msg, Result};
use std::collections::HashMap;
use std::io::Read;

fn utf16_decode(iter: Vec<u16>) -> Result<String> {
    char::decode_utf16(iter)
        .map(|r| {
            r.map_err(|e| {
                assert_with_msg!(
                    "Invalid UTF-16, unpaired surrogate {:#04X}",
                    e.unpaired_surrogate()
                )
            })
        })
        .collect::<Result<String>>()
}

pub fn read_string_block(
    block_id: u32,
    mut data: CountingReader<impl Read>,
    messages: &mut HashMap<u32, String>,
) -> Result<()> {
    let block_min = (block_id - 1) * 16;
    let block_max = block_id * 16;
    trace!("String block {}: {}..{}", block_id, block_min, block_max);
    for string_id in block_min..block_max {
        let len = u16_to_usize(data.read_u16()?);
        // blocks always have 16 strings; so missing strings are "empty"
        if len == 0 {
            continue;
        }
        let mut buf = [0; 2];
        let bytes = (0..len)
            .map(|_| {
                data.read_exact(&mut buf)?;
                Ok(u16::from_le_bytes(buf))
            })
            .collect::<Result<Vec<u16>>>()?;
        let chars = utf16_decode(bytes)?;
        trace!("Message {} ({}): {}", string_id, len, chars);
        if messages.insert(string_id, chars).is_some() {
            panic!("Duplicate string ID {}", string_id);
        }
    }
    Ok(())
}
