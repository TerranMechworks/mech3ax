use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::ObjectActiveState;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{impl_as_bytes, AsBytes as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, bool_c, Result};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectActiveStateC {
    state: u32,
    node_index: u32,
}
impl_as_bytes!(ObjectActiveStateC, 8);

impl ScriptObject for ObjectActiveState {
    const INDEX: u8 = 6;
    const SIZE: u32 = ObjectActiveStateC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("object active state size", size == Self::SIZE, read.offset)?;
        let object_active_state: ObjectActiveStateC = read.read_struct()?;
        let state =
            assert_that!("object active state", bool object_active_state.state, read.prev + 0)?;

        let node =
            anim_def.node_from_index(object_active_state.node_index as usize, read.prev + 4)?;
        Ok(Self { node, state })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write.write_struct(&ObjectActiveStateC {
            state: bool_c!(self.state),
            node_index: anim_def.node_to_index(&self.node)? as u32,
        })?;
        Ok(())
    }
}
