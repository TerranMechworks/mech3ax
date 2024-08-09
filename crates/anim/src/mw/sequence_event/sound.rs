use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{AtNode, Sound};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{impl_as_bytes, AsBytes as _, Vec3};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct SoundC {
    sound_index: u16,
    node_index: u16,
    translation: Vec3,
}
impl_as_bytes!(SoundC, 16);

impl ScriptObject for Sound {
    const INDEX: u8 = 1;
    const SIZE: u32 = SoundC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("sound size", size == Self::SIZE, read.offset)?;
        let sound: SoundC = read.read_struct()?;
        let name = anim_def.sound_from_index(sound.sound_index as usize, read.prev + 0)?;
        let node = anim_def.node_from_index(sound.node_index as usize, read.prev + 2)?;
        Ok(Self {
            name,
            at_node: AtNode {
                node,
                translation: sound.translation,
            },
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        write.write_struct(&SoundC {
            sound_index: anim_def.sound_to_index(&self.name)? as u16,
            node_index: anim_def.node_to_index(&self.at_node.node)? as u16,
            translation: self.at_node.translation,
        })?;
        Ok(())
    }
}
