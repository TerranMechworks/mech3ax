use super::ScriptObject;
use crate::AnimDef;
use mech3ax_api_types::{static_assert_size, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::{assert_that, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct ObjectCycleTextureC {
    increment: u16,   // guess
    start_index: u16, // guess
    node_index: u16,
    reset: u16,
}
static_assert_size!(ObjectCycleTextureC, 8);

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectCycleTexture {
    pub node: String,
    pub reset: u16,
}

impl ScriptObject for ObjectCycleTexture {
    const INDEX: u8 = 17;
    const SIZE: u32 = ObjectCycleTextureC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("object cycle texture size", size == Self::SIZE, read.offset)?;
        let object_cycle_texture: ObjectCycleTextureC = read.read_struct()?;
        assert_that!(
            "object cycle texture increment",
            object_cycle_texture.increment == 1,
            read.prev + 0
        )?;
        assert_that!(
            "object cycle texture start",
            object_cycle_texture.start_index == 0,
            read.prev + 2
        )?;
        let node =
            anim_def.node_from_index(object_cycle_texture.node_index as usize, read.prev + 4)?;
        assert_that!("object cycle texture reset", 0 <= object_cycle_texture.reset <= 5, read.prev + 6)?;
        Ok(Self {
            node,
            reset: object_cycle_texture.reset,
        })
    }

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.node)? as u16;
        write.write_struct(&ObjectCycleTextureC {
            increment: 1,
            start_index: 0,
            node_index,
            reset: self.reset,
        })?;
        Ok(())
    }
}
