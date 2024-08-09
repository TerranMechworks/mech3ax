use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::ObjectAddChild;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{impl_as_bytes, AsBytes as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectAddChildC {
    parent_index: u16,
    child_index: u16,
}
impl_as_bytes!(ObjectAddChildC, 4);

impl ScriptObject for ObjectAddChild {
    const INDEX: u8 = 15;
    const SIZE: u32 = ObjectAddChildC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("object add child size", size == Self::SIZE, read.offset)?;
        let add_child: ObjectAddChildC = read.read_struct()?;
        let parent = anim_def.node_from_index(add_child.parent_index as usize, read.prev + 0)?;
        let child = anim_def.node_from_index(add_child.child_index as usize, read.prev + 2)?;
        Ok(Self { parent, child })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let parent_index = anim_def.node_to_index(&self.parent)? as u16;
        let child_index = anim_def.node_to_index(&self.child)? as u16;
        write.write_struct(&ObjectAddChildC {
            parent_index,
            child_index,
        })?;
        Ok(())
    }
}
