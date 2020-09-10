use super::types::INPUT_NODE;
use super::ScriptObject;
use crate::anim::AnimDef;
use crate::io_ext::{CountingReader, WriteHelper};
use crate::size::ReprSize;
use crate::types::Vec3;
use crate::{assert_that, static_assert_size, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct ObjectTranslateStateC {
    at_node_matrix: u32,
    translate: Vec3,
    node_index: u32,
}
static_assert_size!(ObjectTranslateStateC, 20);

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectTranslateState {
    pub node: String,
    pub translate: Vec3,
    pub node_index: i32,
}

impl ScriptObject for ObjectTranslateState {
    const INDEX: u8 = 7;
    const SIZE: u32 = ObjectTranslateStateC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object translate state size",
            size == Self::SIZE,
            read.offset
        )?;
        let object_translate_state: ObjectTranslateStateC = read.read_struct()?;
        assert_that!(
            "object translate state field 00",
            object_translate_state.at_node_matrix == 0,
            read.prev + 0
        )?;
        // TODO: figure this out
        let node_index = object_translate_state.node_index as i32;
        let node = if node_index < 1 {
            INPUT_NODE.to_owned()
        } else {
            anim_def.node_from_index(object_translate_state.node_index as usize, read.prev + 16)?
        };

        Ok(Self {
            node,
            translate: object_translate_state.translate,
            node_index,
        })
    }

    fn write<W: Write>(&self, write: &mut W, _anim_def: &AnimDef) -> Result<()> {
        write.write_struct(&ObjectTranslateStateC {
            at_node_matrix: 0,
            translate: self.translate,
            node_index: self.node_index as u32,
        })?;
        Ok(())
    }
}
