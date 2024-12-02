use super::EventAll;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::Loop;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Bytes, Maybe};
use std::io::{Read, Write};

bitflags! {
    struct LoopFlags: u32 {
        const COUNT = 1 << 0;    // 0x1
        const RUN_TIME = 1 << 1; // 0x2
    }
}

type Flags = Maybe<u32, LoopFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct LoopC {
    flags: Flags,    // 00
    value: Bytes<4>, // 04
}
impl_as_bytes!(LoopC, 8);

impl EventAll for Loop {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(LoopC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, _anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("loop size", size == LoopC::SIZE, read.offset)?;
        let loop_: LoopC = read.read_struct()?;

        let flags = assert_that!("loop flags", flags loop_.flags, read.prev + 0)?;

        let has_run_time = flags.contains(LoopFlags::RUN_TIME);
        if flags.contains(LoopFlags::COUNT) {
            assert_that!(
                "loop flags is run time",
                has_run_time == false,
                read.prev + 0
            )?;

            let (hi, lo) = loop_.value.split();
            assert_that!("loop field 06", lo == [0; 2], read.prev + 6)?;

            let count = i16::from_le_bytes(hi);
            Ok(Self::Count(count))
        } else if has_run_time {
            let run_time = f32::from_le_bytes(loop_.value.into_inner());
            Ok(Self::RunTime(run_time))
        } else {
            Err(assert_with_msg!("empty loop flags"))
        }
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        let loop_ = match self {
            Self::Count(count) => {
                let flags = LoopFlags::COUNT.maybe();
                let value = Bytes::join(count.to_le_bytes(), [0; 2]);
                LoopC { flags, value }
            }
            Self::RunTime(run_time) => {
                let flags = LoopFlags::RUN_TIME.maybe();
                let value = Bytes::new(run_time.to_le_bytes());
                LoopC { flags, value }
            }
        };
        write.write_struct(&loop_)?;
        Ok(())
    }
}
