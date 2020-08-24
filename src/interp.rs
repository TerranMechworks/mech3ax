use crate::assert::assert_utf8;
use crate::io_ext::{ReadHelper, WriteHelper};
use crate::size::ReprSize;
use crate::string::{str_from_c, str_to_c};
use crate::{assert_that, static_assert_size, Result};
use ::serde::{Deserialize, Serialize};
use chrono::{DateTime, TimeZone, Utc};
use std::io::{Read, Write};

const SIGNATURE: u32 = 0x08971119;
const VERSION: u32 = 7;

#[repr(C)]
struct EntryC {
    name: [u8; 120],
    last_modified: u32,
    start: u32,
}
static_assert_size!(EntryC, 128);

#[derive(Serialize, Deserialize, Debug)]
pub struct Script {
    pub name: String,
    pub last_modified: DateTime<Utc>,
    pub lines: Vec<String>,
}

fn read_script<R>(read: &mut R, offset: &mut u64) -> Result<Vec<String>>
where
    R: Read,
{
    let mut lines = Vec::new();
    loop {
        let size = read.read_u32()?;
        *offset += 4;
        if size == 0 {
            break;
        }
        let arg_count = read.read_u32()? as usize;
        *offset += 4;

        let mut buf = vec![0u8; size as usize];
        read.read_exact(&mut buf)?;
        *offset += size as u64;

        let mut zero_count = 0;
        // replace null characters with spaces and count them
        for v in &mut buf {
            if *v == 0 {
                zero_count += 1;
                *v = 32;
            }
        }
        assert_that!("arg count", zero_count == arg_count, *offset)?;
        let last = buf.pop().unwrap();
        assert_that!("command end", last == 32, *offset)?;

        let command = assert_utf8("command", *offset, || std::str::from_utf8(&buf))?;
        lines.push(command.to_owned());
    }
    Ok(lines)
}

pub fn read_interp<R>(read: &mut R) -> Result<Vec<Script>>
where
    R: Read,
{
    let signature = read.read_u32()?;
    assert_that!("signature", signature == SIGNATURE, 0)?;
    let version = read.read_u32()?;
    assert_that!("version", version == VERSION, 4)?;
    let count = read.read_u32()?;
    let mut offset = 12;

    let script_info = (0..count)
        .into_iter()
        .map(|_| {
            let entry = read.read_struct::<EntryC>()?;
            let name = assert_utf8("name", offset, || str_from_c(&entry.name))?;
            let last_modified = Utc.timestamp(entry.last_modified as i64, 0);
            offset += EntryC::SIZE as u64;
            Ok((name.into(), last_modified, entry.start as u64))
        })
        .collect::<Result<Vec<_>>>()?;

    script_info
        .into_iter()
        .map(|(name, last_modified, start)| {
            assert_that!("entry start", start == offset, offset)?;
            let lines = read_script(read, &mut offset)?;
            Ok(Script {
                name,
                last_modified,
                lines,
            })
        })
        .collect::<Result<Vec<_>>>()
}

fn write_script(lines: Vec<String>) -> (u32, Vec<(u32, Vec<u8>)>) {
    let mut size = 0;
    let transformed = lines
        .into_iter()
        .map(|line| {
            let mut buf = line.into_bytes();
            buf.push(32); // add "zero" terminator
            let mut zero_count = 0;
            for v in &mut buf {
                if *v == 32 {
                    zero_count += 1;
                    *v = 0;
                }
            }
            size += 8 + buf.len() as u32;
            (zero_count, buf)
        })
        .collect();
    // zero "size" u32 written to signify end of script
    size += 4;
    (size, transformed)
}

pub fn write_interp<W>(write: &mut W, scripts: Vec<Script>) -> Result<()>
where
    W: Write,
{
    let count = scripts.len() as u32;
    write.write_u32(SIGNATURE)?;
    write.write_u32(VERSION)?;
    write.write_u32(count)?;

    let mut offset = 12 + count * EntryC::SIZE;
    let transformed = scripts
        .into_iter()
        .map(|script| {
            let mut name = [0; 120];
            str_to_c(script.name, &mut name);
            let last_modified = script.last_modified.timestamp() as u32;
            let entry = EntryC {
                name,
                last_modified,
                start: offset,
            };
            write.write_struct::<EntryC>(&entry)?;

            let (size, lines) = write_script(script.lines);
            offset += size;
            Ok(lines)
        })
        .collect::<Result<Vec<_>>>()?;

    for lines in transformed.into_iter() {
        for (arg_count, command) in lines.into_iter() {
            write.write_u32(command.len() as u32)?;
            write.write_u32(arg_count)?;
            write.write_all(&command)?;
        }
        write.write_u32(0)?;
    }

    Ok(())
}
