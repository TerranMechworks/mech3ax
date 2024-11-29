use super::ScriptObject;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::Loop;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct LoopC {
    start: i32,
    loop_count: i32,
}
impl_as_bytes!(LoopC, 8);

impl ScriptObject for Loop {
    const INDEX: u8 = 30;
    const SIZE: u32 = LoopC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("loop size", size == LoopC::SIZE, read.offset)?;
        let loop_: LoopC = read.read_struct()?;
        assert_that!("loop start", loop_.start == 1, read.prev + 0)?;
        Ok(Loop {
            start: loop_.start,
            loop_count: loop_.loop_count,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write.write_struct(&LoopC {
            start: self.start,
            loop_count: self.loop_count,
        })?;
        Ok(())
    }
}
