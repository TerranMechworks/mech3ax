use super::EventAll;
use crate::types::{AnimDefLookup as _, Idx32};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::ObjectScaleState;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::Vec3;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectScaleStateC {
    scale: Vec3,       // 00
    node_index: Idx32, // 12
}
impl_as_bytes!(ObjectScaleStateC, 16);

impl EventAll for ObjectScaleState {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectScaleStateC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object scale state size",
            size == ObjectScaleStateC::SIZE,
            read.offset
        )?;
        let state: ObjectScaleStateC = read.read_struct()?;

        let name = anim_def.node_from_index(state.node_index, read.prev + 12)?;
        Ok(Self {
            name,
            state: state.scale,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.name)?;

        let object_scale_state = ObjectScaleStateC {
            scale: self.state,
            node_index,
        };
        write.write_struct(&object_scale_state)?;
        Ok(())
    }
}
