use super::EventAll;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::AnimVerbose;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Bool32};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct AnimVerboseC {
    on: Bool32, // 00
}
impl_as_bytes!(AnimVerboseC, 4);

impl EventAll for AnimVerbose {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(AnimVerboseC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("anim verbose size", size == AnimVerboseC::SIZE, read.offset)?;
        let anim_verbose: AnimVerboseC = read.read_struct()?;
        let on = assert_that!("anim verbose on", bool anim_verbose.on, read.prev + 0)?;
        Ok(Self { on })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        let anim_verbose = AnimVerboseC { on: self.on.into() };
        write.write_struct(&anim_verbose)?;
        Ok(())
    }
}
