use crate::io_ext::{ReadHelper, WriteHelper};
use crate::{assert_that, Result};
use std::io::{Read, Write};

const VERSION: u32 = 27;
const FORMAT: u32 = 1;

pub fn read_version<R>(read: &mut R) -> Result<()>
where
    R: Read,
{
    let version = read.read_u32()?;
    assert_that!("version", version == VERSION, 0)?;
    read.assert_end()
}

pub fn read_format<R>(read: &mut R) -> Result<()>
where
    R: Read,
{
    let format = read.read_u32()?;
    assert_that!("format", format == FORMAT, 0)?;
    read.assert_end()
}

pub fn write_version<W>(write: &mut W) -> Result<()>
where
    W: Write,
{
    write.write_u32(VERSION)?;
    Ok(())
}

pub fn write_format<W>(write: &mut W) -> Result<()>
where
    W: Write,
{
    write.write_u32(FORMAT)?;
    Ok(())
}
