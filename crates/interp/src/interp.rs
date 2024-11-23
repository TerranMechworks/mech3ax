use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::interp::Script;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use mech3ax_types::{impl_as_bytes, string_from_ascii, u32_to_usize, AsBytes as _, Ascii};
use std::io::{Read, Write};
use time::OffsetDateTime;

const SIGNATURE: u32 = 0x08971119;
const VERSION: u32 = 7;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct InterpHeaderC {
    signature: u32,
    version: u32,
    count: u32,
}
impl_as_bytes!(InterpHeaderC, 12);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct InterpEntryC {
    name: Ascii<120>,
    last_modified: u32,
    start: u32,
}
impl_as_bytes!(InterpEntryC, 128);

fn read_line(read: &mut CountingReader<impl Read>, size: u32) -> Result<String> {
    let size = u32_to_usize(size);
    let arg_count = read.read_u32()?;

    let mut buf = vec![0u8; size];
    read.read_exact(&mut buf)?;

    // replace null characters with spaces and count them
    let mut zero_count = 0u32;
    for v in &mut buf {
        if *v == b'\0' {
            zero_count += 1;
            *v = b' ';
        }
    }
    assert_that!("arg count", zero_count == arg_count, read.prev)?;

    // remove terminating null (which is now a space)
    // strictly speaking this always succeeds, as size > 0 here
    let last = buf.pop();
    assert_that!("command end", last == Some(b' '), read.prev)?;

    let command = assert_utf8("command", read.prev, || string_from_ascii(buf))?;
    Ok(command)
}

fn read_script(read: &mut CountingReader<impl Read>, script_index: usize) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    let mut line_index = 0;
    loop {
        trace!(
            "Reading interp script {} line {} at {}",
            script_index,
            line_index,
            read.offset
        );
        let size = read.read_u32()?;
        // end of script
        if size == 0 {
            break;
        }
        let command = read_line(read, size)?;
        lines.push(command);
        line_index += 1;
    }
    Ok(lines)
}

pub fn read_interp(read: &mut CountingReader<impl Read>) -> Result<Vec<Script>> {
    debug!(
        "Reading interp header ({}) at {}",
        InterpHeaderC::SIZE,
        read.offset
    );
    let header: InterpHeaderC = read.read_struct()?;
    trace!("{:#?}", header);
    assert_that!(
        "interp signature",
        header.signature == SIGNATURE,
        read.prev + 0
    )?;
    assert_that!("interp version", header.version == VERSION, read.prev + 4)?;

    let script_info = (0..header.count)
        .map(|index| {
            debug!(
                "Reading interp entry {} ({}) at {}",
                index,
                InterpEntryC::SIZE,
                read.offset
            );
            let entry: InterpEntryC = read.read_struct()?;
            trace!("{:#?}", entry);
            let name = assert_utf8("name", read.prev, || entry.name.to_str_padded())?;
            // Cast safety: i64 > u32
            let last_modified =
                OffsetDateTime::from_unix_timestamp(entry.last_modified as i64).unwrap();
            let start = u32_to_usize(entry.start);
            Ok((name, last_modified, start))
        })
        .collect::<Result<Vec<_>>>()?;

    let scripts = script_info
        .into_iter()
        .enumerate()
        .map(|(script_index, (name, last_modified, start))| {
            debug!("Reading interp script {} at {}", script_index, read.offset);
            assert_that!("entry start", start == read.offset, read.offset)?;
            let lines = read_script(read, script_index)?;
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

fn size_script(lines: &[String]) -> u32 {
    let mut size = 0;
    for line in lines {
        // add size + arg_count
        size += 8;
        // add terminating null
        let line_size = line.as_bytes().len() + 1;
        // Cast safety: truncation simply leads to incorrect size, and is
        // validated properly later.
        size += line_size as u32;
    }
    // zero "size" u32 written to signify end of script
    size += 4;
    size
}

fn write_line(write: &mut CountingWriter<impl Write>, line: &str) -> Result<()> {
    let mut buf = Vec::from(line.as_bytes());

    buf.push(32); // add terminating null (as a space for now)
    let line_size = assert_len!(u32, buf.len(), "script line length in bytes")?;

    // replace spaces with null characters and count them
    let mut zero_count = 0;
    for v in &mut buf {
        if *v == b' ' {
            zero_count += 1;
            *v = b'\0';
        }
    }

    write.write_u32(line_size)?;
    write.write_u32(zero_count)?;
    write.write_all(&buf)?;
    Ok(())
}

pub fn write_interp(write: &mut CountingWriter<impl Write>, scripts: &[Script]) -> Result<()> {
    debug!(
        "Writing interp header ({}) at {}",
        InterpHeaderC::SIZE,
        write.offset
    );
    let count = assert_len!(u32, scripts.len(), "scripts")?;
    let header = InterpHeaderC {
        signature: SIGNATURE,
        version: VERSION,
        count,
    };
    trace!("{:#?}", header);
    write.write_struct(&header)?;

    let mut offset = 12 + count * InterpEntryC::SIZE;
    for (index, script) in scripts.iter().enumerate() {
        debug!(
            "Writing interp entry {} ({}) at {}",
            index,
            InterpEntryC::SIZE,
            write.offset
        );
        let name = Ascii::from_str_padded(&script.name);
        // Cast safety: truncation simply leads to incorrect timestamp
        let last_modified = script.last_modified.unix_timestamp() as u32;
        let entry = InterpEntryC {
            name,
            last_modified,
            start: offset,
        };
        trace!("{:#?}", entry);
        write.write_struct(&entry)?;
        offset += size_script(&script.lines);
    }

    for (script_index, script) in scripts.iter().enumerate() {
        debug!("Writing interp script {} at {}", script_index, write.offset);
        for (line_index, line) in script.lines.iter().enumerate() {
            trace!(
                "Writing interp script {} line {} at {}",
                script_index,
                line_index,
                write.offset
            );
            write_line(write, line)?;
        }
        trace!(
            "Writing interp script {} line {} at {}",
            script_index,
            script.lines.len(),
            write.offset
        );
        // end of script
        write.write_u32(0)?;
    }

    Ok(())
}
