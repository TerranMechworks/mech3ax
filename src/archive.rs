use crate::assert::assert_utf8;
use crate::io_ext::{ReadHelper, WriteHelper};
use crate::serde::base64;
use crate::size::ReprSize;
use crate::string::{bytes_to_c, str_from_c, str_to_c};
use crate::{assert_that, static_assert_size, Result};
use ::serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom, Write};

const VERSION: u32 = 1;

#[repr(C)]
struct EntryC {
    start: u32,
    length: u32,
    name: [u8; 64],
    garbage: [u8; 76],
}
static_assert_size!(EntryC, 148);

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub name: String,
    #[serde(with = "base64")]
    pub garbage: Vec<u8>,
}

fn read_table<R>(read: &mut R) -> Result<Vec<(String, u64, u64, Vec<u8>)>>
where
    R: Read + Seek,
{
    let pos = read.seek(SeekFrom::End(-8))?;

    let version = read.read_u32()?;
    assert_that!("version", version == VERSION, pos + 4)?;
    let count = read.read_u32()?;

    let offset: i64 = 8 + (count * EntryC::SIZE) as i64;
    let table_start = read.seek(SeekFrom::End(-offset))?;
    let mut pos = table_start;

    (0..count)
        .map(|_| {
            let entry = read.read_struct::<EntryC>()?;

            let entry_start = entry.start as u64;
            let entry_len = entry.length as u64;
            let entry_end = entry_start + entry_len;

            assert_that!("entry start", entry_start < entry_end, pos + 0)?;
            assert_that!("entry end", entry_end <= table_start, pos + 4)?;
            let entry_name = assert_utf8("entry name", pos + 8, || str_from_c(&entry.name))?;

            pos += EntryC::SIZE as u64;
            Ok((
                entry_name.to_owned(),
                entry_start,
                entry_len,
                entry.garbage.to_vec(),
            ))
        })
        .collect::<Result<Vec<_>>>()
}

pub fn read_archive<R>(read: &mut R) -> Result<Vec<(Entry, Vec<u8>)>>
where
    R: Read + Seek,
{
    let entries = read_table(read)?;
    entries
        .into_iter()
        .map(|(name, start, length, garbage)| {
            read.seek(SeekFrom::Start(start))?;
            let mut buffer = vec![0; length as usize];
            read.read_exact(&mut buffer)?;
            let entry = Entry {
                name: name.to_owned(),
                garbage,
            };
            Ok((entry, buffer))
        })
        .collect::<Result<Vec<_>>>()
}

fn entry_to_c(entry: Entry, start: u32, length: u32) -> EntryC {
    let mut name = [0; 64];
    str_to_c(entry.name, &mut name);
    let mut garbage = [0; 76];
    bytes_to_c(entry.garbage, &mut garbage);

    EntryC {
        start,
        length,
        name,
        garbage,
    }
}

pub fn write_archive<W>(write: &mut W, entries: Vec<(Entry, Vec<u8>)>) -> Result<()>
where
    W: Write,
{
    let mut offset = 0;
    let count = entries.len() as u32;

    let transformed = entries
        .into_iter()
        .map(|(entry, data)| {
            let entry_c = entry_to_c(entry, offset, data.len() as u32);
            offset += entry_c.length;
            write.write_all(&data)?;
            Ok(entry_c)
        })
        .collect::<Result<Vec<_>>>()?;

    for entry in transformed.into_iter() {
        write.write_struct::<EntryC>(&entry)?
    }

    write.write_u32(VERSION)?;
    write.write_u32(count)?;
    Ok(())
}
