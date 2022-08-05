use super::ScriptObject;
use crate::AnimDef;
use mech3ax_api_types::{static_assert_size, ReprSize as _, Vec3};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::{assert_that, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct ObjectScaleStateC {
    scale: Vec3,
    node_index: u32,
}
static_assert_size!(ObjectScaleStateC, 16);

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectScaleState {
    pub node: String,
    pub scale: Vec3,
}

impl ScriptObject for ObjectScaleState {
    const INDEX: u8 = 8;
    const SIZE: u32 = ObjectScaleStateC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("object scale state size", size == Self::SIZE, read.offset)?;
        let object_scale_state: ObjectScaleStateC = read.read_struct()?;
        let node =
            anim_def.node_from_index(object_scale_state.node_index as usize, read.prev + 12)?;
        Ok(Self {
            node,
            scale: object_scale_state.scale,
        })
    }

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        write.write_struct(&ObjectScaleStateC {
            scale: self.scale,
            node_index: anim_def.node_to_index(&self.node)? as u32,
        })?;
        Ok(())
    }
}
