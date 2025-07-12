use super::EventAll;
use crate::types::{AnimDefLookup as _, INPUT_NODE_NAME, Idx16, index};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Vec3;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{ObjectRotateState, RotateBasis};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Maybe, bitflags, impl_as_bytes};
use std::io::{Read, Write};

const PI: f32 = std::f32::consts::PI;

// FLAGS (mutually exclusive?)
// if this is a camera:
// 0 -> absolute rotation
// 1 -> relative rotation
// if this is a 3d object:
// 0 -> absolute rotation
// 1 -> relative rotation
// 2 -> AT_NODE_XYZ
// 4 -> AT_NODE_MATRIX
bitflags! {
    struct ObjectRotateStateFlags: u32 {
        const RELATIVE = 1 << 0; // 0x1
        const AT_NODE_XYZ = 1 << 1; // 0x2
        const AT_NODE_MATRIX = 1 << 2; // 0x4
    }
}

type Flags = Maybe<u32, ObjectRotateStateFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectRotateStateC {
    flags: Flags,         // 00
    rotate: Vec3,         // 04
    node_index: Idx16,    // 16
    at_node_index: Idx16, // 18
}
impl_as_bytes!(ObjectRotateStateC, 20);

impl EventAll for ObjectRotateState {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectRotateStateC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object rotate state size",
            size == ObjectRotateStateC::SIZE,
            read.offset
        )?;
        let state: ObjectRotateStateC = read.read_struct()?;

        let flags = assert_that!("object rotate state flags", flags state.flags, read.prev + 0)?;
        let rotate = {
            let rotate = state.rotate;
            assert_that!("object rotate state x", -PI <= rotate.x <= PI, read.prev + 4)?;
            assert_that!("object rotate state y", -PI <= rotate.y <= PI, read.prev + 8)?;
            assert_that!("object rotate state z", -PI <= rotate.z <= PI, read.prev + 12)?;
            Vec3 {
                x: rotate.x,
                y: rotate.y,
                z: rotate.z,
            }
        };
        let name = anim_def.node_from_index(state.node_index, read.prev + 16)?;

        let basis = if flags.contains(ObjectRotateStateFlags::AT_NODE_XYZ) {
            assert_that!(
                "object rotate state flags",
                flags == ObjectRotateStateFlags::AT_NODE_XYZ,
                read.prev + 0
            )?;
            let at_node = if state.at_node_index == index!(input) {
                INPUT_NODE_NAME.to_string()
            } else {
                anim_def.node_from_index(state.at_node_index, read.prev + 18)?
            };
            RotateBasis::AtNodeXYZ(at_node)
        } else if flags.contains(ObjectRotateStateFlags::AT_NODE_MATRIX) {
            assert_that!(
                "object rotate state flags",
                flags == ObjectRotateStateFlags::AT_NODE_MATRIX,
                read.prev + 0
            )?;
            let at_node = if state.at_node_index == index!(input) {
                INPUT_NODE_NAME.to_string()
            } else {
                anim_def.node_from_index(state.at_node_index, read.prev + 18)?
            };
            RotateBasis::AtNodeMatrix(at_node)
        } else if flags.contains(ObjectRotateStateFlags::RELATIVE) {
            assert_that!(
                "object rotate state flags",
                flags == ObjectRotateStateFlags::RELATIVE,
                read.prev + 0
            )?;
            assert_that!(
                "object rotate state at node",
                state.at_node_index == index!(0),
                read.prev + 18
            )?;
            RotateBasis::Relative
        } else {
            assert_that!(
                "object rotate state flags",
                flags == ObjectRotateStateFlags::empty(),
                read.prev + 0
            )?;
            assert_that!(
                "object rotate state at node",
                state.at_node_index == index!(0),
                read.prev + 18
            )?;
            RotateBasis::Absolute
        };

        Ok(Self {
            name,
            state: rotate,
            basis,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.name)?;
        let rotate = Vec3 {
            x: self.state.x,
            y: self.state.y,
            z: self.state.z,
        };

        let mut flags = ObjectRotateStateFlags::empty();
        let at_node_index = match &self.basis {
            RotateBasis::AtNodeXYZ(at_node) => {
                flags |= ObjectRotateStateFlags::AT_NODE_XYZ;
                if at_node == INPUT_NODE_NAME {
                    index!(input)
                } else {
                    anim_def.node_to_index(at_node)?
                }
            }
            RotateBasis::AtNodeMatrix(at_node) => {
                flags |= ObjectRotateStateFlags::AT_NODE_MATRIX;
                if at_node == INPUT_NODE_NAME {
                    index!(input)
                } else {
                    anim_def.node_to_index(at_node)?
                }
            }
            RotateBasis::Relative => {
                flags |= ObjectRotateStateFlags::RELATIVE;
                index!(0)
            }
            RotateBasis::Absolute => {
                index!(0)
            }
        };

        let state = ObjectRotateStateC {
            flags: flags.maybe(),
            rotate,
            node_index,
            at_node_index,
        };
        write.write_struct(&state)?;
        Ok(())
    }
}
