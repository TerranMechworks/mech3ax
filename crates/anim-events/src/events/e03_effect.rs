use super::EventAll;
use crate::types::{AnimDefLookup as _, INPUT_NODE_NAME, Idx16, index};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Vec3;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{AtNode, Effect};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, impl_as_bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct EffectC {
    effect_index: Idx16, // 00
    node_index: Idx16,   // 02
    translation: Vec3,   // 04
}
impl_as_bytes!(EffectC, 16);

impl EventAll for Effect {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(EffectC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("effect size", size == EffectC::SIZE, read.offset)?;
        let effect: EffectC = read.read_struct()?;

        let name = anim_def.effect_from_index(effect.effect_index, read.prev + 0)?;
        let node = if effect.node_index == index!(input) {
            INPUT_NODE_NAME.to_string()
        } else {
            anim_def.node_from_index(effect.node_index, read.prev + 2)?
        };

        Ok(Self {
            name,
            at_node: AtNode {
                name: node,
                pos: effect.translation,
            },
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let effect_index = anim_def.effect_to_index(&self.name)?;
        let node_index = if self.at_node.name == INPUT_NODE_NAME {
            index!(input)
        } else {
            anim_def.node_to_index(&self.at_node.name)?
        };

        let effect = EffectC {
            effect_index,
            node_index,
            translation: self.at_node.pos,
        };
        write.write_struct(&effect)?;
        Ok(())
    }
}
