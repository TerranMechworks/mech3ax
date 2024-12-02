use super::EventAll;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::Callback;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct CallbackC {
    value: u32,
}
impl_as_bytes!(CallbackC, 4);

impl EventAll for Callback {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(CallbackC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("callback size", size == CallbackC::SIZE, read.offset)?;
        assert_that!(
            "anim def has callbacks",
            anim_def.has_callbacks == true,
            read.offset
        )?;

        let callback: CallbackC = read.read_struct()?;
        Ok(Self {
            value: callback.value,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, _anim_def: &AnimDef) -> Result<()> {
        write.write_struct(&CallbackC { value: self.value })?;
        Ok(())
    }
}
