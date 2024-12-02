use super::EventAll;
use crate::types::{AnimDefLookup as _, Idx16};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::ObjectDeleteChild;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectDeleteChildC {
    parent_index: Idx16,
    child_index: Idx16,
}
impl_as_bytes!(ObjectDeleteChildC, 4);

impl EventAll for ObjectDeleteChild {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectDeleteChildC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object delete child size",
            size == ObjectDeleteChildC::SIZE,
            read.offset
        )?;
        let del_child: ObjectDeleteChildC = read.read_struct()?;

        let parent = anim_def.node_from_index(del_child.parent_index, read.prev + 0)?;
        let child = anim_def.node_from_index(del_child.child_index, read.prev + 2)?;
        Ok(Self { parent, child })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let parent_index = anim_def.node_to_index(&self.parent)?;
        let child_index = anim_def.node_to_index(&self.child)?;

        let del_child = ObjectDeleteChildC {
            parent_index,
            child_index,
        };
        write.write_struct(&del_child)?;
        Ok(())
    }
}
