use crate::size::u16_to_usize;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, assert_that, assert_with_msg};
use mech3ax_encoding::windows1252_decode;
use mech3ax_types::u32_to_usize;
use std::collections::HashMap;
use std::io::Cursor;

fn remove_trailing(buf: &mut Vec<u8>) -> Result<()> {
    // remove from back: \0 (0, multiple), \n (10, single), and \r (13, single)
    loop {
        match buf.last() {
            Some(0) => buf.pop(),
            Some(_) => break,
            None => return Err(assert_with_msg!("Message table: ran out of chars")),
        };
    }
    match buf.pop() {
        Some(10) => {}
        Some(actual) => {
            return Err(assert_with_msg!(
                "Message table: expected trailing \n, was {}",
                actual
            ));
        }
        None => return Err(assert_with_msg!("Message table: ran out of chars")),
    };
    match buf.pop() {
        Some(13) => {}
        Some(actual) => {
            return Err(assert_with_msg!(
                "Message table: expected trailing \r, was {}",
                actual
            ));
        }
        None => return Err(assert_with_msg!("Message table: ran out of chars")),
    };
    Ok(())
}

pub fn read_message_table(data: &[u8]) -> Result<HashMap<u32, String>> {
    let mut read = CountingReader::new(Cursor::new(data));
    let count = read.read_u32()?;

    let table = (0..count)
        .map(|_| {
            let low_id = read.read_u32()?;
            let high_id = read.read_u32()?;
            let offset_to_entries = read.read_u32()?;
            Ok((low_id, high_id, offset_to_entries))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut entries = HashMap::new();
    for (low_id, high_id, offset_to_entries) in table {
        read.get_mut().set_position(u64::from(offset_to_entries));
        for entry_id in low_id..=high_id {
            let length = read.read_u16()? - 4;
            let flags = read.read_u16()?;

            assert_that!(
                "unicode flags",
                flags == 0x0000,
                u32_to_usize(offset_to_entries)
            )?;
            let mut buf = vec![0; u16_to_usize(length)];
            read.read_exact(&mut buf)?;
            remove_trailing(&mut buf)?;

            // all the English, German, and French locale IDs map to the same codepage (1251)
            let message_contents = windows1252_decode(&buf).into_owned();
            entries.insert(entry_id, message_contents);
        }
    }

    Ok(entries)
}
