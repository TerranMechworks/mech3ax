use super::types::AtNode;
use super::ScriptObject;
use crate::anim::AnimDef;
use crate::io_ext::{CountingReader, WriteHelper};
use crate::size::ReprSize;
use crate::types::Vec3;
use crate::{assert_that, static_assert_size, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct SoundC {
    sound_index: u16,
    node_index: u16,
    translation: Vec3,
}
static_assert_size!(SoundC, 16);

#[derive(Debug, Serialize, Deserialize)]
pub struct Sound {
    pub name: String,
    pub at_node: AtNode,
}

impl ScriptObject for Sound {
    const INDEX: u8 = 1;
    const SIZE: u32 = SoundC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
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

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        write.write_struct(&SoundC {
            sound_index: anim_def.sound_to_index(&self.name)? as u16,
            node_index: anim_def.node_to_index(&self.at_node.node)? as u16,
            translation: self.at_node.translation,
        })?;
        Ok(())
    }
}
