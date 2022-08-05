use super::types::INPUT_NODE;
use super::ScriptObject;
use crate::AnimDef;
use mech3ax_api_types::{static_assert_size, ReprSize as _, Vec3};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::{assert_that, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const INPUT_NODE_INDEX: u16 = -200i16 as u16;

#[repr(C)]
struct ObjectTranslateStateC {
    zero00: u32,        // 00
    translate: Vec3,    // 04
    node_index: u16,    // 16
    at_node_index: u16, // 18
}
static_assert_size!(ObjectTranslateStateC, 20);

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectTranslateState {
    pub node: String,
    pub translate: Vec3,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub at_node: Option<String>,
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
            object_translate_state.zero00 == 0,
            read.prev + 0
        )?;

        let node =
            anim_def.node_from_index(object_translate_state.node_index as usize, read.prev + 16)?;
        let at_node = if object_translate_state.at_node_index == INPUT_NODE_INDEX {
            Some(INPUT_NODE.to_owned())
        } else {
            assert_that!(
                "object translate state at node",
                object_translate_state.at_node_index == 0,
                read.prev + 18
            )?;
            None
        };

        Ok(Self {
            node,
            translate: object_translate_state.translate,
            at_node,
        })
    }

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.node)? as u16;
        let at_node_index = if let Some(at_node) = &self.at_node {
            assert_that!("object translate state at node", at_node == INPUT_NODE, 0)?;
            INPUT_NODE_INDEX
        } else {
            0
        };
        write.write_struct(&ObjectTranslateStateC {
            zero00: 0,
            translate: self.translate,
            node_index,
            at_node_index,
        })?;
        Ok(())
    }
}
