use super::ScriptObject;
use crate::AnimDef;
use mech3ax_api_types::{static_assert_size, ReprSize as _, Vec3};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::{assert_that, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const PI: f32 = std::f32::consts::PI;
const INPUT_NODE_INDEX: u16 = -200i16 as u16;

#[repr(C)]
struct ObjectRotateStateC {
    flags: u32,         // 00
    rotate: Vec3,       // 04
    node_index: u16,    // 16
    at_node_index: u16, // 18
}
static_assert_size!(ObjectRotateStateC, 20);

#[derive(Debug, Serialize, Deserialize)]
pub enum RotateState {
    Absolute(Vec3),
    AtNodeXYZ,
    AtNodeMatrix,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectRotateState {
    pub node: String,
    pub rotate: RotateState,
}

impl ScriptObject for ObjectRotateState {
    const INDEX: u8 = 9;
    const SIZE: u32 = ObjectRotateStateC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("object rotate state size", size == Self::SIZE, read.offset)?;
        let object_rotate_state: ObjectRotateStateC = read.read_struct()?;
        // FLAGS (mutually exclusive)
        // if this is a camera:
        // 0 -> absolute rotation
        // 1 -> relative rotation
        // if this is a 3d object:
        // 0 -> absolute rotation
        // 1 -> relative rotation
        // 2 -> AT_NODE_XYZ
        // 4 -> AT_NODE_MATRIX
        assert_that!("object rotate state flags", object_rotate_state.flags in [0, 2, 4], read.prev + 0)?;
        let node =
            anim_def.node_from_index(object_rotate_state.node_index as usize, read.prev + 16)?;

        let rotate = if object_rotate_state.flags == 0 {
            let rotate = object_rotate_state.rotate;
            assert_that!("object rotate state x", -PI <= rotate.0 <= PI, read.prev + 4)?;
            assert_that!("object rotate state y", -PI <= rotate.1 <= PI, read.prev + 8)?;
            assert_that!("object rotate state z", -PI <= rotate.2 <= PI, read.prev + 12)?;
            assert_that!(
                "object rotate state at node",
                object_rotate_state.at_node_index == 0,
                read.prev + 18
            )?;

            RotateState::Absolute(Vec3(
                rotate.0.to_degrees(),
                rotate.1.to_degrees(),
                rotate.2.to_degrees(),
            ))
        } else {
            assert_that!(
                "object rotate state rot",
                object_rotate_state.rotate == Vec3::EMPTY,
                read.prev + 4
            )?;
            assert_that!(
                "object rotate state at node",
                object_rotate_state.at_node_index == INPUT_NODE_INDEX,
                read.prev + 18
            )?;
            if object_rotate_state.flags == 2 {
                RotateState::AtNodeXYZ
            } else {
                RotateState::AtNodeMatrix
            }
        };

        Ok(Self { node, rotate })
    }

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        let (flags, rotate, at_node_index) = match &self.rotate {
            RotateState::Absolute(rotate) => {
                let rotate = Vec3(
                    rotate.0.to_radians(),
                    rotate.1.to_radians(),
                    rotate.2.to_radians(),
                );
                (0, rotate, 0)
            }
            RotateState::AtNodeXYZ => (2, Vec3::EMPTY, INPUT_NODE_INDEX),
            RotateState::AtNodeMatrix => (4, Vec3::EMPTY, INPUT_NODE_INDEX),
        };

        write.write_struct(&ObjectRotateStateC {
            flags,
            rotate,
            node_index: anim_def.node_to_index(&self.node)? as u16,
            at_node_index,
        })?;
        Ok(())
    }
}
