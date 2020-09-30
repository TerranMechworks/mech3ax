use super::types::AtNode;
use super::ScriptObject;
use crate::anim::AnimDef;
use crate::assert::assert_utf8;
use crate::io_ext::{CountingReader, WriteHelper};
use crate::size::ReprSize;
use crate::string::{str_from_c_padded, str_to_c_padded};
use crate::types::Vec3;
use crate::{assert_that, static_assert_size, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct SoundNodeC {
    name: [u8; 32],
    one32: u32,
    inherit_translation: u32, // 36
    active_state: u32,        // 40
    node_index: u32,          // 44
    translation: Vec3,        // 48
}
static_assert_size!(SoundNodeC, 60);

#[derive(Debug, Serialize, Deserialize)]
pub struct SoundNode {
    pub name: String,
    pub active_state: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub at_node: Option<AtNode>,
}

impl ScriptObject for SoundNode {
    const INDEX: u8 = 2;
    const SIZE: u32 = SoundNodeC::SIZE;

    fn read<R: Read>(read: &mut CountingReader<R>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("sound node size", size == Self::SIZE, read.offset)?;
        let sound_node: SoundNodeC = read.read_struct()?;
        let name = assert_utf8("sound node name", read.prev + 0, || {
            str_from_c_padded(&sound_node.name)
        })?;
        assert_that!("sound node field 32", sound_node.one32 == 1, read.prev + 32)?;
        assert_that!("sound node field 36", sound_node.inherit_translation in [0, 2], read.prev + 36)?;
        let active_state =
            assert_that!("sound node active state", bool sound_node.active_state, read.prev + 40)?;

        let at_node = if sound_node.inherit_translation == 0 {
            assert_that!(
                "sound node at node index",
                sound_node.node_index == 0,
                read.prev + 44
            )?;
            assert_that!(
                "sound node translation",
                sound_node.translation == Vec3::EMPTY,
                read.prev + 48
            )?;
            None
        } else {
            let node = anim_def.node_from_index(sound_node.node_index as usize, read.prev + 44)?;
            Some(AtNode {
                node,
                translation: sound_node.translation,
            })
        };
        Ok(Self {
            name,
            active_state,
            at_node,
        })
    }

    fn write<W: Write>(&self, write: &mut W, anim_def: &AnimDef) -> Result<()> {
        let mut name = [0; 32];
        str_to_c_padded(&self.name, &mut name);
        let active_state = if self.active_state { 1 } else { 0 };

        let (inherit_translation, node_index, translation) = if let Some(at_node) = &self.at_node {
            let node_index = anim_def.node_to_index(&at_node.node)? as u32;
            (2, node_index, at_node.translation)
        } else {
            (0, 0, Vec3::EMPTY)
        };
        write.write_struct(&SoundNodeC {
            name,
            one32: 1,
            inherit_translation,
            active_state,
            node_index,
            translation,
        })?;
        Ok(())
    }
}
