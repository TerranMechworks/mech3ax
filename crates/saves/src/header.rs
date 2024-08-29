use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

const VERSION_MW: u32 = 8;
const FORMAT: u32 = 2;

pub fn read_save_header(read: &mut CountingReader<impl Read>) -> Result<()> {
    let version = read.read_u32()?;
    assert_that!("save header version", version == VERSION_MW, read.prev)?;
    let format = read.read_u32()?;
    assert_that!("save header format", format == FORMAT, read.prev)?;
    Ok(())
}

pub fn write_save_header(write: &mut CountingWriter<impl Write>) -> Result<()> {
    write.write_u32(VERSION_MW)?;
    write.write_u32(FORMAT)?;
    Ok(())
}
