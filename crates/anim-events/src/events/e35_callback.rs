use super::ScriptObject;
use mech3ax_api_types::anim::events::Callback;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

impl ScriptObject for Callback {
    const INDEX: u8 = 35;
    const SIZE: u32 = 4;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("callback size", size == Self::SIZE, read.offset)?;
        assert_that!(
            "anim def has callbacks",
            anim_def.has_callbacks == true,
            read.offset
        )?;
        let value = read.read_u32()?;
        Ok(Self { value })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write.write_u32(self.value)?;
        Ok(())
    }
}
