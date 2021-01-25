use crate::assert::{assert_utf8, AssertionError};
use crate::io_ext::ReadHelper;
use crate::string::str_from_c_sized;
use crate::{assert_that, Result};
use ::serde::{Deserialize, Serialize};
use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, Encoding};
use log::trace;
use pelite::pe32::{Pe, PeFile};
use pelite::resources::{DataEntry, FindError, Name, Resources};
use std::collections::HashMap;
use std::io::{Cursor, Read};

const DLL_BASE_ADDRESS: u32 = 0x10000000;

#[derive(Debug, Serialize, Deserialize)]
pub struct Messages {
    pub language_id: u32,
    pub entries: Vec<(String, u32, String)>,
}

pub fn read_messages<R>(read: &mut R, skip_data: Option<u64>) -> Result<Messages>
where
    R: Read,
{
    let mut buf = Vec::new();
    read.read_to_end(&mut buf)?;

    let pe = PeFile::from_bytes(&buf)?;

    let resources = pe.resources()?;
    trace!("Resources: {}", resources.to_string());
    let (language_id, entry) = find_message_table(resources)
        .map_err(|_| AssertionError("Expected DLL to contain a message table".to_owned()))?;

    let mut messages = read_message_table(entry.bytes()?)?;

    let data_section = pe
        .section_headers()
        .by_name(".data")
        .ok_or_else(|| AssertionError("Expected DLL to contain a data directory".to_owned()))?;

    let data = pe.get_section_bytes(data_section)?;

    let offset = data_section.VirtualAddress + DLL_BASE_ADDRESS;
    let message_ids = read_zlocids(data, skip_data, offset, offset + data_section.VirtualSize)?;

    let mut entries = message_ids
        .into_iter()
        .map(|(entry_id, name)| {
            let message = messages.remove(&entry_id).ok_or_else(|| {
                AssertionError(format!("Message \"{}\" ({}) not found", &name, entry_id))
            })?;
            Ok((name, entry_id, message))
        })
        .collect::<Result<Vec<_>>>()?;

    assert_that!("all message table strings used", 0 == messages.len(), 0)?;

    entries.reverse();
    Ok(Messages {
        language_id,
        entries,
    })
}

fn find_message_table(resources: Resources) -> std::result::Result<(u32, DataEntry), FindError> {
    let dir = resources
        .root()?
        .get_dir(Name::Id(11))? // RT_MESSAGETABLE
        .first_dir()?;
    let entry = dir.entries().next().ok_or(FindError::NotFound)?;
    // for a message table, the name/ID is the language id
    let language_id = if let Name::Id(language_id) = entry.name()? {
        language_id
    } else {
        return Err(FindError::NotFound);
    };
    let data = entry.entry()?.data().ok_or(FindError::UnDirectory)?;
    Ok((language_id, data))
}

fn remove_trailing(buf: &mut Vec<u8>) -> std::result::Result<(), AssertionError> {
    // remove from back: \0 (0, multiple), \n (10, single), and \r (13, single)
    loop {
        match buf.last() {
            Some(0) => buf.pop(),
            Some(_) => break,
            None => return Err(AssertionError(format!("Message table: ran out of chars"))),
        };
    }
    match buf.pop() {
        Some(10) => {}
        Some(actual) => {
            return Err(AssertionError(format!(
                "Message table: expected trailing \n, was {}",
                actual
            )))
        }
        None => return Err(AssertionError(format!("Message table: ran out of chars"))),
    };
    match buf.pop() {
        Some(13) => {}
        Some(actual) => {
            return Err(AssertionError(format!(
                "Message table: expected trailing \r, was {}",
                actual
            )))
        }
        None => return Err(AssertionError(format!("Message table: ran out of chars"))),
    };
    Ok(())
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
            remove_trailing(&mut buf)?;

            // all the English, German, and French locale IDs map to the same codepage (1251)
            let message_contents = WINDOWS_1252
                .decode(&buf, DecoderTrap::Strict)
                .map_err(|err| AssertionError(err.into()))?;

            entries.insert(entry_id, message_contents);
        }
    }

    Ok(entries)
}

fn read_zlocids(
    data: &[u8],
    skip: Option<u64>,
    mem_start: u32,
    mem_end: u32,
) -> Result<Vec<(u32, String)>> {
    let mut read = Cursor::new(data);

    // skip the CRT initialization section
    if let Some(pos) = skip {
        read.set_position(pos);
    } else {
        for i in 0..4 {
            let initterm = read.read_u32()?;
            assert_that!("initterm", initterm == 0, i * 4)?;
        }
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

            while data[relative_end] != 0 {
                relative_end += 1;
            }

            let message_name =
                assert_utf8(format!("message {}", entry_id), relative_start, || {
                    str_from_c_sized(&data[relative_start..relative_end])
                })?;

            Ok((entry_id, message_name))
        })
        .collect::<Result<Vec<_>>>()
}
