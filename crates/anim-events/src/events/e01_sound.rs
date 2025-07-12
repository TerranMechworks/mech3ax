use super::EventAll;
use crate::types::{AnimDefLookup as _, Idx16, index};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Vec3;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{AtNode, Sound};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, impl_as_bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct SoundC {
    sound_index: Idx16, // 00
    node_index: Idx16,  // 02
    pos: Vec3,          // 04
}
impl_as_bytes!(SoundC, 16);

impl EventAll for Sound {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(SoundC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("sound size", size == SoundC::SIZE, read.offset)?;
        let sound: SoundC = read.read_struct()?;

        let name = anim_def.stc_sound_from_index(sound.sound_index, read.prev + 0)?;

        let at_node = if sound.node_index == index!(0) {
            assert_that!("sound pos", sound.pos == Vec3::DEFAULT, read.prev + 4)?;
            None
        } else {
            let node = anim_def.node_from_index(sound.node_index, read.prev + 2)?;
            Some(AtNode {
                name: node,
                pos: sound.pos,
            })
        };
        Ok(Self { name, at_node })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let sound_index = anim_def.stc_sound_to_index(&self.name)?;

        let (node_index, pos) = match &self.at_node {
            Some(at_node) => {
                let node_index = anim_def.node_to_index(&at_node.name)?;
                (node_index, at_node.pos)
            }
            None => (index!(0), Vec3::DEFAULT),
        };

        let sound = SoundC {
            sound_index,
            node_index,
            pos,
        };
        write.write_struct(&sound)?;
        Ok(())
    }
}
