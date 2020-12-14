use crate::assert::{assert_utf8, AssertionError};
use crate::io_ext::ReadHelper;
use crate::string::str_from_c_sized;
use crate::{assert_that, Result};
use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, Encoding};
use log::trace;
use pelite::pe32::{Pe, PeFile};
use pelite::resources::{DataEntry, FindError, Name, Resources};
use std::collections::HashMap;
use std::io::{Cursor, Read};

const DLL_BASE_ADDRESS: u32 = 0x10000000;

pub fn read_messages<R>(read: &mut R) -> Result<Vec<(String, u32, String)>>
where
    R: Read,
{
    let mut buf = Vec::new();
    read.read_to_end(&mut buf)?;

    let pe = PeFile::from_bytes(&buf)?;

    let resources = pe.resources()?;
    trace!("Resources: {}", resources.to_string());
    let entry = find_message_table(resources)
        .map_err(|_| AssertionError("Expected DLL to contain a message table".to_owned()))?;

    let mut messages = read_message_table(entry.bytes()?)?;

    let data_section = pe
        .section_headers()
        .by_name(".data")
        .ok_or_else(|| AssertionError("Expected DLL to contain a data directory".to_owned()))?;

    let data = pe.get_section_bytes(data_section)?;

    let offset = data_section.VirtualAddress + DLL_BASE_ADDRESS;
    let message_ids = read_zlocids(data, offset, offset + data_section.VirtualSize)?;

    message_ids
        .into_iter()
        .map(|(entry_id, name)| {
            let message = messages.remove(&entry_id).ok_or_else(|| {
                AssertionError(format!("Message \"{}\" ({}) not found", &name, entry_id))
            })?;
            Ok((name, entry_id, message))
        })
        .collect()
}

fn find_message_table(resources: Resources) -> std::result::Result<DataEntry, FindError> {
    resources
        .root()?
        .get_dir(Name::Id(11))?
        .first_dir()?
        .first_data()
}

fn read_message_table(data: &[u8]) -> Result<HashMap<u32, String>> {
    let mut read = Cursor::new(data);
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
        read.set_position(offset_to_entries as u64);
        for entry_id in low_id..high_id + 1 {
            let length = read.read_u16()? - 4;
            let flags = read.read_u16()?;

            assert_that!("unicode flags", flags == 0x0000, offset_to_entries)?;
            let mut buf = vec![0; length as usize];
            read.read_exact(&mut buf)?;

            // remove trailing \0 (0), \n (10), and \r (13)
            loop {
                match buf.pop() {
                    Some(0) => {}
                    Some(10) => {}
                    Some(13) => {}
                    Some(char) => {
                        buf.push(char);
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }

            // all the English, German, and French locale IDs map to the same CP
            let message_contents = WINDOWS_1252
                .decode(&buf, DecoderTrap::Strict)
                .map_err(|err| AssertionError(err.into()))?;

            entries.insert(entry_id, message_contents);
        }
    }

    Ok(entries)
}

fn read_zlocids(data: &[u8], mem_start: u32, mem_end: u32) -> Result<Vec<(u32, String)>> {
    let mut read = Cursor::new(data);

    // skip the CRT initialization section
    for i in 0..4 {
        let initterm = read.read_u32()?;
        assert_that!("initterm", initterm == 0, i * 4)?;
    }

    let mut entry_table = Vec::new();
    loop {
        let mem_offset = read.read_u32()?;

        // the data isn't meant to be read like this; but this condition triggers
        // if we've read 4 bytes into the string data
        if mem_offset > mem_end {
            break;
        }

        let entry_id = read.read_u32()?;
        let relative_offset = (mem_offset - mem_start) as usize;
        entry_table.push((entry_id, relative_offset));
    }

    // the table of message offsets and message table IDs is written backwards, highest
    // address first.
    entry_table
        .into_iter()
        .rev()
        .map(|(entry_id, relative_start)| {
            let mut relative_end = relative_start;

            loop {
                if data[relative_end] != 0 {
                    relative_end += 1;
                } else {
                    break;
                }
            }

            let message_name =
                assert_utf8(format!("message {}", entry_id), relative_start, || {
                    str_from_c_sized(&data[relative_start..relative_end])
                })?;

            Ok((entry_id, message_name))
        })
        .collect::<Result<Vec<_>>>()
}
