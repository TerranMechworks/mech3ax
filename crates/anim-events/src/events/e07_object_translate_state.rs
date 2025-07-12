use super::EventAll;
use crate::types::{AnimDefLookup as _, INPUT_NODE_NAME, Idx16, index};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Vec3;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::ObjectTranslateState;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Bool32, impl_as_bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectTranslateStateC {
    relative: Bool32,     // 00
    translate: Vec3,      // 04
    node_index: Idx16,    // 16
    at_node_index: Idx16, // 18
}
impl_as_bytes!(ObjectTranslateStateC, 20);

impl EventAll for ObjectTranslateState {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectTranslateStateC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object translate state size",
            size == ObjectTranslateStateC::SIZE,
            read.offset
        )?;
        let state: ObjectTranslateStateC = read.read_struct()?;

        // TODO: don't love this. also, probably interacts with at_node_index?
        let relative = assert_that!(
            "object translate state relative",
            bool state.relative,
            read.prev + 0
        )?;

        let node = anim_def.node_from_index(state.node_index, read.prev + 16)?;

        let at_node = if state.at_node_index == index!(input) {
            Some(INPUT_NODE_NAME.to_string())
        } else if state.at_node_index > 0 {
            Some(anim_def.node_from_index(state.at_node_index, read.prev + 18)?)
        } else {
            assert_that!(
                "object translate state at node",
                state.at_node_index == index!(0),
                read.prev + 18
            )?;
            None
        };

        Ok(Self {
            node,
            relative,
            state: state.translate,
            at_node,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.node)?;
        let at_node_index = match self.at_node.as_deref() {
            Some(name) if name == INPUT_NODE_NAME => index!(input),
            Some(name) => anim_def.node_to_index(name)?,
            None => index!(0),
        };

        let state = ObjectTranslateStateC {
            relative: self.relative.into(),
            translate: self.state,
            node_index,
            at_node_index,
        };
        write.write_struct(&state)?;
        Ok(())
    }
}
