use super::EventAll;
use crate::types::{index, AnimDefLookup as _, Idx32};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{AtNode, SoundNode, Translate};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::Vec3;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Ascii, Bool32, Maybe};
use std::io::{Read, Write};

bitflags! {
    struct SoundNodeFlags: u32 {
        const TRANSLATE_ABS = 1 << 0; // 0x1
        const AT_NODE = 1 << 1; // 0x2
    }
}

type Flags = Maybe<u32, SoundNodeFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct SoundNodeC {
    sound_name: Ascii<32>, // 00
    sound_index: Idx32,    // 32
    flags: Flags,          // 36
    active_state: Bool32,  // 40
    node_index: Idx32,     // 44
    translate: Vec3,       // 48
}
impl_as_bytes!(SoundNodeC, 60);

impl EventAll for SoundNode {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(SoundNodeC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("sound node size", size == SoundNodeC::SIZE, read.offset)?;
        let sound_node: SoundNodeC = read.read_struct()?;

        // not sure why this information is duplicated?
        let name = assert_utf8("sound node name", read.prev + 0, || {
            sound_node.sound_name.to_str_padded()
        })?;
        let expected_name =
            anim_def.dyn_sound_from_index(sound_node.sound_index, read.prev + 32)?;
        assert_that!("sound node name", name == expected_name, read.prev + 32)?;

        let flags = assert_that!("sound node flags", flags sound_node.flags, read.prev + 36)?;
        let active_state =
            assert_that!("sound node active state", bool sound_node.active_state, read.prev + 40)?;

        let translate = if flags.contains(SoundNodeFlags::AT_NODE) {
            let name = anim_def.node_from_index(sound_node.node_index, read.prev + 44)?;
            Some(Translate::AtNode(AtNode {
                name,
                pos: sound_node.translate,
            }))
        } else if flags.contains(SoundNodeFlags::TRANSLATE_ABS) {
            assert_that!(
                "sound node at node index",
                sound_node.node_index == index!(0),
                read.prev + 44
            )?;
            Some(Translate::Absolute(sound_node.translate))
        } else {
            assert_that!(
                "sound node at node index",
                sound_node.node_index == index!(0),
                read.prev + 44
            )?;
            assert_that!(
                "sound node translation",
                sound_node.translate == Vec3::DEFAULT,
                read.prev + 48
            )?;
            None
        };

        Ok(Self {
            name,
            active_state,
            translate,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let sound_name = Ascii::from_str_padded(&self.name);
        let sound_index = anim_def.dyn_sound_to_index(&self.name)?;

        let active_state = self.active_state.into();

        let mut flags = SoundNodeFlags::empty();
        let mut node_index = index!(0);
        let translate = match &self.translate {
            Some(Translate::AtNode(AtNode { name, pos })) => {
                flags |= SoundNodeFlags::AT_NODE;
                node_index = anim_def.node_to_index(name)?;
                *pos
            }
            Some(Translate::Absolute(pos)) => {
                flags |= SoundNodeFlags::TRANSLATE_ABS;
                *pos
            }
            None => Vec3::DEFAULT,
        };

        let sound_node = SoundNodeC {
            sound_name,
            sound_index,
            flags: flags.maybe(),
            active_state,
            node_index,
            translate,
        };
        write.write_struct(&sound_node)?;
        Ok(())
    }
}
