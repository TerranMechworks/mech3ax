use super::types::INPUT_NODE;
use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use mech3ax_api_types::anim::events::{
    CallAnimation, CallAnimationAtNode, CallAnimationParameters, CallAnimationTargetNode,
    CallAnimationWithNode,
};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{static_assert_size, ReprSize as _, Vec3};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_debug::Ascii;
use std::io::{Read, Write};

const INPUT_NODE_INDEX: u32 = 65336;

#[repr(C)]
struct CallAnimationC {
    name: Ascii<32>,          // 00
    operand_index: u16,       // 32
    flags: u16,               // 34
    anim_index: u16,          // 36
    wait_for_completion: u16, // 38
    node_index: u32,          // 40
    translation: Vec3,        // 44
    rotation: Vec3,           // 56
}
static_assert_size!(CallAnimationC, 68);

bitflags::bitflags! {
    struct CallAnimationFlags: u16 {
        // Call with AT_NODE (OPERAND_NODE can't be used)
        const AT_NODE = 1 << 0;
        // AT_NODE/WITH_NODE has translation coordinates
        const TRANSLATION = 1 << 1;
        // AT_NODE has rotation coordinates
        const ROTATION = 1 << 2;
        // Call with WITH_NODE (OPERAND_NODE can't be used)
        const WITH_NODE = 1 << 3;
        // WAIT_FOR_COMPLETION is set
        const WAIT_FOR = 1 << 4;
    }
}

impl ScriptObject for CallAnimation {
    const INDEX: u8 = 24;
    const SIZE: u32 = CallAnimationC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("call animation size", size == Self::SIZE, read.offset)?;
        let call_animation: CallAnimationC = read.read_struct()?;

        let name = assert_utf8("call animation name", read.prev + 0, || {
            str_from_c_padded(&call_animation.name)
        })?;
        let flags = CallAnimationFlags::from_bits(call_animation.flags).ok_or_else(|| {
            assert_with_msg!(
                "Expected valid call animation flags, but was 0x{:04X} (at {})",
                call_animation.flags,
                read.prev + 34
            )
        })?;
        // this is used to store the index of the animation to call once loaded
        assert_that!(
            "call animation anim index",
            call_animation.anim_index == 0,
            read.prev + 36
        )?;

        let wait_for_completion = if flags.contains(CallAnimationFlags::WAIT_FOR) {
            let max_prev_ref = anim_def.anim_refs.as_ref().map(|v| v.len()).unwrap_or(0) as u16 - 1;
            assert_that!("call animation wait for", 0 <= call_animation.wait_for_completion <= max_prev_ref, read.prev + 38)?;
            Some(call_animation.wait_for_completion)
        } else {
            assert_that!(
                "call animation wait for",
                call_animation.wait_for_completion == u16::MAX,
                read.prev + 38
            )?;
            None
        };

        let translation = if flags.contains(CallAnimationFlags::TRANSLATION) {
            Some(call_animation.translation)
        } else {
            assert_that!(
                "call animation translation",
                call_animation.translation == Vec3::DEFAULT,
                read.prev + 44
            )?;
            None
        };

        let rotation = if flags.contains(CallAnimationFlags::ROTATION) {
            Some(call_animation.rotation)
        } else {
            assert_that!(
                "call animation rotation",
                call_animation.rotation == Vec3::DEFAULT,
                read.prev + 56
            )?;
            None
        };

        let with_node = flags.contains(CallAnimationFlags::WITH_NODE);
        let parameters = if flags.contains(CallAnimationFlags::AT_NODE) {
            assert_that!(
                "call animation with node",
                with_node == false,
                read.prev + 34
            )?;
            // when using AT_NODE, OPERAND_NODE can't be used
            assert_that!(
                "call animation operand index",
                call_animation.operand_index == 0,
                read.prev + 32
            )?;
            let node = if call_animation.node_index == INPUT_NODE_INDEX {
                INPUT_NODE.to_owned()
            } else {
                anim_def.node_from_index(call_animation.node_index as usize, read.prev + 40)?
            };
            CallAnimationParameters::AtNode(CallAnimationAtNode {
                node,
                translation,
                rotation,
            })
        } else if with_node {
            let has_rotation = rotation.is_some();
            assert_that!(
                "call animation has rotation",
                has_rotation == false,
                read.prev + 34
            )?;
            // when using WITH_NODE, OPERAND_NODE can't be used
            assert_that!(
                "call animation operand index",
                call_animation.operand_index == 0,
                read.prev + 32
            )?;
            // WITH_NODE doesn't seem to use INPUT_NODE
            let node =
                anim_def.node_from_index(call_animation.node_index as usize, read.prev + 40)?;
            CallAnimationParameters::WithNode(CallAnimationWithNode { node, translation })
        } else {
            let has_translation = translation.is_some();
            let has_rotation = rotation.is_some();
            assert_that!(
                "call animation has translation",
                has_translation == false,
                read.prev + 34
            )?;
            assert_that!(
                "call animation has rotation",
                has_rotation == false,
                read.prev + 34
            )?;
            assert_that!(
                "call animation node index",
                call_animation.node_index == 0,
                read.prev + 40
            )?;
            // OPERAND_NODE may be used but doesn't need to be
            if call_animation.operand_index == 0 {
                CallAnimationParameters::None
            } else {
                let operand_node = anim_def
                    .node_from_index(call_animation.operand_index as usize, read.prev + 32)?;
                CallAnimationParameters::TargetNode(CallAnimationTargetNode { operand_node })
            }
        };

        Ok(Self {
            name,
            wait_for_completion,
            parameters,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let mut name = Ascii::zero();
        str_to_c_padded(&self.name, &mut name);
        let mut flags = CallAnimationFlags::empty();
        if self.wait_for_completion.is_some() {
            flags |= CallAnimationFlags::WAIT_FOR;
        }
        let (operand_index, node_index, translation, rotation) = match &self.parameters {
            CallAnimationParameters::None => (0, 0, Vec3::DEFAULT, Vec3::DEFAULT),
            CallAnimationParameters::TargetNode(CallAnimationTargetNode { operand_node }) => {
                let operand_index = anim_def.node_to_index(operand_node)? as u16;
                (operand_index, 0, Vec3::DEFAULT, Vec3::DEFAULT)
            }
            CallAnimationParameters::WithNode(CallAnimationWithNode { node, translation }) => {
                flags |= CallAnimationFlags::WITH_NODE;
                let node_index = anim_def.node_to_index(node)? as u32;
                let translation = if let Some(translation) = translation {
                    flags |= CallAnimationFlags::TRANSLATION;
                    *translation
                } else {
                    Vec3::DEFAULT
                };
                (0, node_index, translation, Vec3::DEFAULT)
            }
            CallAnimationParameters::AtNode(CallAnimationAtNode {
                node,
                translation,
                rotation,
            }) => {
                flags |= CallAnimationFlags::AT_NODE;
                let node_index = if node == INPUT_NODE {
                    INPUT_NODE_INDEX
                } else {
                    anim_def.node_to_index(node)? as u32
                };
                let translation = if let Some(translation) = translation {
                    flags |= CallAnimationFlags::TRANSLATION;
                    *translation
                } else {
                    Vec3::DEFAULT
                };
                let rotation = if let Some(rotation) = rotation {
                    flags |= CallAnimationFlags::ROTATION;
                    *rotation
                } else {
                    Vec3::DEFAULT
                };
                (0, node_index, translation, rotation)
            }
        };

        write.write_struct(&CallAnimationC {
            name,
            operand_index,
            flags: flags.bits(),
            anim_index: 0,
            wait_for_completion: self.wait_for_completion.unwrap_or(u16::MAX),
            node_index,
            translation,
            rotation,
        })?;
        Ok(())
    }
}
