use super::EventAll;
use crate::types::{AnimDefLookup as _, INPUT_NODE_NAME, Idx16, index};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::ObjectActiveState;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Bool32, impl_as_bytes};
use std::io::{Read, Write};

const PM_FIXUP_NODE_INDEX: Idx16 = index!(-1);
const PM_FIXUP_NODE_NAME: &str = "pltes";

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectActiveStateC {
    state: Bool32,     // 00
    node_index: Idx16, // 04
    pad06: u16,        // 06
}
impl_as_bytes!(ObjectActiveStateC, 8);

impl EventAll for ObjectActiveState {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectActiveStateC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object active state size",
            size == ObjectActiveStateC::SIZE,
            read.offset
        )?;
        let state: ObjectActiveStateC = read.read_struct()?;

        let active_state = assert_that!("object active state", bool state.state, read.prev + 0)?;
        assert_that!(
            "object active state field 06",
            state.pad06 == 0,
            read.prev + 6
        )?;

        let node = if state.node_index == PM_FIXUP_NODE_INDEX {
            log::debug!(
                "object active state fixup: {} -> `{}`",
                PM_FIXUP_NODE_INDEX,
                PM_FIXUP_NODE_NAME,
            );
            PM_FIXUP_NODE_NAME.to_string()
        } else if state.node_index == index!(-100) {
            INPUT_NODE_NAME.to_string()
        } else {
            anim_def.node_from_index(state.node_index, read.prev + 4)?
        };

        Ok(Self {
            node,
            state: active_state,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let node_index = if self.node == PM_FIXUP_NODE_NAME {
            log::debug!(
                "object active state fixup: `{}` -> {}",
                PM_FIXUP_NODE_NAME,
                PM_FIXUP_NODE_INDEX,
            );
            PM_FIXUP_NODE_INDEX
        } else if self.node == INPUT_NODE_NAME {
            index!(-100)
        } else {
            anim_def.node_to_index(&self.node)?
        };

        let state = ObjectActiveStateC {
            state: self.state.into(),
            node_index,
            pad06: 0,
        };
        write.write_struct(&state)?;
        Ok(())
    }
}
