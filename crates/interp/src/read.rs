use super::{InterpEntryC, InterpHeaderC, SIGNATURE, VERSION};
use log::trace;
use mech3ax_api_types::interp::Script;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, assert_that};
use mech3ax_timestamp::unix::from_timestamp;
use mech3ax_types::{string_from_ascii, u32_to_usize};
use std::io::Read;

pub fn read_interp(read: &mut CountingReader<impl Read>) -> Result<Vec<Script>> {
    let header: InterpHeaderC = read.read_struct()?;
    assert_that!(
        "interp signature",
        header.signature == SIGNATURE,
        read.prev + 0
    )?;
    assert_that!("interp version", header.version == VERSION, read.prev + 4)?;

    let script_info = (0..header.count)
        .map(|index| {
            trace!("Reading interp entry {}", index);
            let entry: InterpEntryC = read.read_struct()?;
            let name = assert_utf8("name", read.prev, || entry.name.to_str_padded())?;
            let datetime = from_timestamp(entry.timestamp);
            let start = u32_to_usize(entry.start);
            Ok((index, name, datetime, start))
        })
        .collect::<Result<Vec<_>>>()?;

    let scripts = script_info
        .into_iter()
        .map(|(index, name, datetime, start)| {
            trace!("Reading interp script {}", index);
            assert_that!("entry start", start == read.offset, read.offset)?;
            let lines = read_script(read)?;
            Ok(Script {
                name,
                datetime,
                lines,
            })
        })
        .collect::<Result<Vec<_>>>();

    read.assert_end()?;
    scripts
}

fn read_script(read: &mut CountingReader<impl Read>) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    loop {
        let size = read.read_u32()?;
        // end of script
        if size == 0 {
            break;
        }
        let command = read_line(read, size, lines.len())?;
        lines.push(command);
    }
    Ok(lines)
}

fn read_line(read: &mut CountingReader<impl Read>, size: u32, index: usize) -> Result<String> {
    let len = u32_to_usize(size);
    let arg_count = read.read_u32()?;

    trace!(
        "Script line {}, size {}, args: {} at {}",
        index,
        size,
        arg_count,
        read.prev - 4,
    );

    let mut buf = vec![0u8; len];
    read.read_exact(&mut buf)?;
    trace!(
        "`{}` (len: {}, at {})",
        buf.escape_ascii(),
        buf.len(),
        read.prev
    );

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
