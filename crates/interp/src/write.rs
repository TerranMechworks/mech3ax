use super::{InterpEntryC, InterpHeaderC, SIGNATURE, VERSION};
use log::trace;
use mech3ax_api_types::interp::Script;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, Result};
use mech3ax_timestamp::unix::to_timestamp;
use mech3ax_types::{AsBytes as _, Ascii};
use std::io::Write;

pub fn write_interp(write: &mut CountingWriter<impl Write>, scripts: &[Script]) -> Result<()> {
    let count = assert_len!(u32, scripts.len(), "scripts")?;
    let header = InterpHeaderC {
        signature: SIGNATURE,
        version: VERSION,
        count,
    };
    write.write_struct(&header)?;

    let mut offset = 12 + count * InterpEntryC::SIZE;
    for (index, script) in scripts.iter().enumerate() {
        trace!("Writing interp entry {}", index);
        let name = Ascii::from_str_padded(&script.name);
        // Cast safety: truncation simply leads to incorrect timestamp
        let last_modified = to_timestamp(&script.last_modified);
        let entry = InterpEntryC {
            name,
            last_modified,
            start: offset,
        };
        write.write_struct(&entry)?;
        offset += size_script(&script.lines);
    }

    for (index, script) in scripts.iter().enumerate() {
        trace!("Writing interp script {}", index);
        for (index, line) in script.lines.iter().enumerate() {
            write_line(write, line, index)?;
        }
        // end of script
        write.write_u32(0)?;
    }

    Ok(())
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

fn write_line(write: &mut CountingWriter<impl Write>, line: &str, index: usize) -> Result<()> {
    let mut buf = Vec::from(line.as_bytes());

    buf.push(32); // add terminating null (as a space for now)
    let size = assert_len!(u32, buf.len(), "script line length in bytes")?;

    // replace spaces with null characters and count them
    let mut arg_count = 0;
    for v in &mut buf {
        if *v == b' ' {
            arg_count += 1;
            *v = b'\0';
        }
    }

    trace!(
        "Script line {}, size {}, args: {} at {}",
        index,
        size,
        arg_count,
        write.offset
    );
    write.write_u32(size)?;
    write.write_u32(arg_count)?;
    trace!(
        "`{}` (len: {}, at {})",
        buf.escape_ascii(),
        buf.len(),
        write.offset
    );
    write.write_all(&buf)?;
    Ok(())
}
