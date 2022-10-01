use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use mech3ax_api_types::{static_assert_size, AnimDef, ObjectOpacityState, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, bool_c, Result};
use std::io::{Read, Write};

#[repr(C)]
struct ObjectOpacityStateC {
    is_set: u16,
    state: u16,
    opacity: f32,
    node_index: u32,
}
static_assert_size!(ObjectOpacityStateC, 12);

impl ScriptObject for ObjectOpacityState {
    const INDEX: u8 = 13;
    const SIZE: u32 = ObjectOpacityStateC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("object opacity state size", size == Self::SIZE, read.offset)?;
        let object_opacity_state: ObjectOpacityStateC = read.read_struct()?;
        let is_set = assert_that!("object opacity state is set", bool object_opacity_state.is_set as u32, read.prev + 0)?;
        let state = assert_that!("object opacity state state", bool object_opacity_state.state as u32, read.prev + 2)?;
        assert_that!("object opacity state opacity", 0.0 <= object_opacity_state.opacity <= 1.0, read.prev + 4)?;
        let node =
            anim_def.node_from_index(object_opacity_state.node_index as usize, read.prev + 8)?;
        Ok(Self {
            node,
            is_set,
            state,
            opacity: object_opacity_state.opacity,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write.write_struct(&ObjectOpacityStateC {
            is_set: bool_c!(self.is_set),
            state: bool_c!(self.state),
            opacity: self.opacity,
            node_index: anim_def.node_to_index(&self.node)? as u32,
        })?;
        Ok(())
    }
}
