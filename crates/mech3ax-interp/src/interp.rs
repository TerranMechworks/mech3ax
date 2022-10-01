use mech3ax_api_types::{static_assert_size, ReprSize as _, Script};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::string::{str_from_c_padded, str_from_c_sized, str_to_c_padded};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};
use time::OffsetDateTime;

const SIGNATURE: u32 = 0x08971119;
const VERSION: u32 = 7;

#[repr(C)]
struct EntryC {
    name: [u8; 120],
    last_modified: u32,
    start: u32,
}
static_assert_size!(EntryC, 128);

fn read_script(read: &mut CountingReader<impl Read>, offset: &mut u64) -> Result<Vec<String>> {
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

        let command = assert_utf8("command", *offset, || str_from_c_sized(&buf))?;
        lines.push(command);
    }
    Ok(lines)
}

pub fn read_interp(read: &mut CountingReader<impl Read>) -> Result<Vec<Script>> {
    let signature = read.read_u32()?;
    assert_that!("signature", signature == SIGNATURE, 0)?;
    let version = read.read_u32()?;
    assert_that!("version", version == VERSION, 4)?;
    let count = read.read_u32()?;
    let mut offset = 12;

    let script_info = (0..count)
        .map(|_| {
            let entry: EntryC = read.read_struct()?;
            let name = assert_utf8("name", offset, || str_from_c_padded(&entry.name))?;
            let last_modified =
                OffsetDateTime::from_unix_timestamp(entry.last_modified as i64).unwrap();
            offset += EntryC::SIZE as u64;
            Ok((name, last_modified, entry.start as u64))
        })
        .collect::<Result<Vec<_>>>()?;

    let scripts = script_info
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
        .collect::<Result<Vec<_>>>();

    read.assert_end()?;
    scripts
}

fn write_script(lines: &[String]) -> (u32, Vec<(u32, Vec<u8>)>) {
    let mut size = 0;
    let transformed = lines
        .iter()
        .map(|line| {
            let mut buf = Vec::from(line.as_bytes());
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

pub fn write_interp(write: &mut impl Write, scripts: &[Script]) -> Result<()> {
    let count = scripts.len() as u32;
    write.write_u32(SIGNATURE)?;
    write.write_u32(VERSION)?;
    write.write_u32(count)?;

    let mut offset = 12 + count * EntryC::SIZE;
    let transformed = scripts
        .iter()
        .map(|script| {
            let mut name = [0; 120];
            str_to_c_padded(&script.name, &mut name);
            let last_modified = script.last_modified.unix_timestamp() as u32;
            let entry = EntryC {
                name,
                last_modified,
                start: offset,
            };
            write.write_struct(&entry)?;

            let (size, lines) = write_script(&script.lines);
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
