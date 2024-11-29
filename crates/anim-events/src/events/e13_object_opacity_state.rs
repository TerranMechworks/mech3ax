use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::ObjectOpacityState;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Bool16};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectOpacityStateC {
    is_set: Bool16,
    state: Bool16,
    opacity: f32,
    node_index: u32,
}
impl_as_bytes!(ObjectOpacityStateC, 12);

impl ScriptObject for ObjectOpacityState {
    const INDEX: u8 = 13;
    const SIZE: u32 = ObjectOpacityStateC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("object opacity state size", size == Self::SIZE, read.offset)?;
        let object_opacity_state: ObjectOpacityStateC = read.read_struct()?;
        let is_set = assert_that!("object opacity state is set", bool object_opacity_state.is_set, read.prev + 0)?;
        let state = assert_that!("object opacity state state", bool object_opacity_state.state, read.prev + 2)?;
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
            is_set: self.is_set.into(),
            state: self.state.into(),
            opacity: self.opacity,
            node_index: anim_def.node_to_index(&self.node)? as u32,
        })?;
        Ok(())
    }
}
