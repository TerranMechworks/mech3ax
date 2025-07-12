use super::EventAll;
use crate::types::{AnimDefLookup as _, Idx16};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::ObjectCycleTexture;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Bool32, impl_as_bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectCycleTextureC {
    flags: Bool32,     // 00
    node_index: Idx16, // 04
    reset: u16,        // 06
}
impl_as_bytes!(ObjectCycleTextureC, 8);

impl EventAll for ObjectCycleTexture {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectCycleTextureC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object cycle texture size",
            size == ObjectCycleTextureC::SIZE,
            read.offset
        )?;
        let state: ObjectCycleTextureC = read.read_struct()?;

        // if flags was 0, nothing would happen?
        assert_that!(
            "object cycle texture flags",
            state.flags == Bool32::TRUE,
            read.prev + 0
        )?;
        let node = anim_def.node_from_index(state.node_index, read.prev + 4)?;
        // the cycle texture limit is 6?
        assert_that!("object cycle texture reset", 0 <= state.reset <= 5, read.prev + 6)?;
        Ok(Self {
            name: node,
            reset: state.reset,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.name)?;

        let state = ObjectCycleTextureC {
            flags: Bool32::TRUE,
            node_index,
            reset: self.reset,
        };
        write.write_struct(&state)?;
        Ok(())
    }
}
