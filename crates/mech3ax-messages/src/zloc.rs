use log::trace;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::string::str_from_c_sized;
use mech3ax_common::{assert_that, assert_with_msg, Result};
use std::io::Cursor;

pub fn read_zlocids(
    data: &[u8],
    skip: Option<usize>,
    mem_start: u32,
    mem_end: u32,
    base_offset: u32,
) -> Result<Vec<(u32, String)>> {
    let mut read = CountingReader::new(Cursor::new(data));
    read.offset = base_offset;

    // skip the CRT initialization section
    if let Some(pos) = skip {
        trace!("skipping {} bytes of CRT initialization section", pos);
        let mut buf = vec![0; pos];
        read.read_exact(&mut buf)?;
    } else {
        for _ in 0..4 {
            let initterm = read.read_u32()?;
            assert_that!("initterm", initterm == 0, read.prev)?;
        }
    }

    let mut entry_table = Vec::new();
    loop {
        let mem_offset = read.read_u32()?;

        // the data isn't meant to be read like this; but this condition triggers
        // if we've read 4 bytes into the string data
        if mem_offset > mem_end {
            trace!("finished reading entry table at offset {}", read.prev);
            break;
        }

        let relative_offset = mem_offset.checked_sub(mem_start).ok_or_else(|| {
            assert_with_msg!(
                "Entry memory offset {} underflowed (start: {}, end: {}, at: {})",
                mem_offset,
                mem_start,
                mem_end,
                read.prev
            )
        })?;

        let entry_id = read.read_u32()?;
        entry_table.push((entry_id, relative_offset));
    }

    // the table of message offsets and message table IDs is written backwards, highest
    // address first.
    entry_table
        .into_iter()
        .rev()
        .map(|(entry_id, start)| {
            let pos = base_offset + start;
            let relative_start = start as usize;
            let mut relative_end = relative_start;

            while data[relative_end] != 0 {
                relative_end += 1;
            }

            let message_name = assert_utf8(format!("message {}", entry_id), pos, || {
                str_from_c_sized(&data[relative_start..relative_end])
            })?;

            Ok((entry_id, message_name))
        })
        .collect::<Result<Vec<_>>>()
}
