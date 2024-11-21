use super::ScriptObject;
use crate::types::AnimDefLookup as _;
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
    scale: Vec3,
    node_index: u32,
}
impl_as_bytes!(ObjectScaleStateC, 16);

impl ScriptObject for ObjectScaleState {
    const INDEX: u8 = 8;
    const SIZE: u32 = ObjectScaleStateC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("object scale state size", size == Self::SIZE, read.offset)?;
        let object_scale_state: ObjectScaleStateC = read.read_struct()?;
        let node =
            anim_def.node_from_index(object_scale_state.node_index as usize, read.prev + 12)?;
        Ok(Self {
            node,
            scale: object_scale_state.scale,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write.write_struct(&ObjectScaleStateC {
            scale: self.scale,
            node_index: anim_def.node_to_index(&self.node)? as u32,
        })?;
        Ok(())
    }
}
