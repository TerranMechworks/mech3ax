use super::EventAll;
use crate::types::{index, AnimDefLookup as _, Idx16, INPUT_NODE_NAME};
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
    is_set: Bool16,       // 00
    active_state: Bool16, // 02
    opacity: f32,         // 04
    node_index: Idx16,    // 08
    pad10: u16,           // 10
}
impl_as_bytes!(ObjectOpacityStateC, 12);

impl EventAll for ObjectOpacityState {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectOpacityStateC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object opacity state size",
            size == ObjectOpacityStateC::SIZE,
            read.offset
        )?;
        let state: ObjectOpacityStateC = read.read_struct()?;

        let is_set = assert_that!("object opacity state is set", bool state.is_set, read.prev + 0)?;
        let active_state =
            assert_that!("object opacity state state", bool state.active_state, read.prev + 2)?;

        let opacity = if active_state {
            assert_that!("object opacity state opacity", 0.0 <= state.opacity <= 1.0, read.prev + 4)?;
            Some(state.opacity)
        } else {
            assert_that!(
                "object opacity state opacity",
                state.opacity == 0.0,
                read.prev + 4
            )?;
            None
        };

        let name = if state.node_index == index!(-100) {
            INPUT_NODE_NAME.to_string()
        } else {
            anim_def.node_from_index(state.node_index, read.prev + 8)?
        };

        assert_that!(
            "object opacity state field 10",
            state.pad10 == 0,
            read.prev + 10
        )?;

        Ok(Self {
            name,
            state: is_set,
            opacity,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let node_index = if self.name == INPUT_NODE_NAME {
            index!(-100)
        } else {
            anim_def.node_to_index(&self.name)?
        };

        let state = ObjectOpacityStateC {
            is_set: self.state.into(),
            active_state: self.opacity.is_some().into(),
            opacity: self.opacity.unwrap_or(0.0),
            node_index,
            pad10: 0,
        };
        write.write_struct(&state)?;
        Ok(())
    }
}
