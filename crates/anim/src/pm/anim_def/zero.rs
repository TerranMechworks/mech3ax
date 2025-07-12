use super::AnimDefC;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use std::io::{Read, Write};

pub(crate) fn write_anim_def_zero(write: &mut CountingWriter<impl Write>) -> Result<()> {
    let anim_def = AnimDefC::default();
    write.write_struct(&anim_def)?;

    Ok(())
}

pub(crate) fn read_anim_def_zero(read: &mut CountingReader<impl Read>) -> Result<()> {
    let anim_def: AnimDefC = read.read_struct()?;
    assert_that!("anim def zero", default anim_def, read.prev)?;

    Ok(())
}
