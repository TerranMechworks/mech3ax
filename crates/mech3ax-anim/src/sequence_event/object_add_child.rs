use super::ScriptObject;
use crate::AnimDef;
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::size::ReprSize;
use mech3ax_common::{assert_that, static_assert_size, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct ObjectAddChildC {
    parent_index: u16,
    child_index: u16,
}
static_assert_size!(ObjectAddChildC, 4);

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectAddChild {
    // in the reader zbd, both values are fused into a list (PARENT_CHILD)
    pub parent: String,
    pub child: String,
}

impl ScriptObject for ObjectAddChild {
    const INDEX: u8 = 15;
    const SIZE: u32 = ObjectAddChildC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("object add child size", size == Self::SIZE, read.offset)?;
        let add_child: ObjectAddChildC = read.read_struct()?;
        let parent = anim_def.node_from_index(add_child.parent_index as usize, read.prev + 0)?;
        let child = anim_def.node_from_index(add_child.child_index as usize, read.prev + 2)?;
        Ok(Self { parent, child })
    }

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        let parent_index = anim_def.node_to_index(&self.parent)? as u16;
        let child_index = anim_def.node_to_index(&self.child)? as u16;
        write.write_struct(&ObjectAddChildC {
            parent_index,
            child_index,
        })?;
        Ok(())
    }
}
