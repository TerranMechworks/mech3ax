use super::EventAll;
use crate::types::{AnimDefLookup as _, INPUT_NODE_NAME, Idx16, index};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Vec3;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{AtNode, DetonateWeapon};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Ascii, impl_as_bytes};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct DetonateWeaponC {
    weapon: Ascii<10>, // 00
    node_index: Idx16, // 10
    translation: Vec3, // 12
}
impl_as_bytes!(DetonateWeaponC, 24);

impl EventAll for DetonateWeapon {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(DetonateWeaponC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "detonate weapon size",
            size == DetonateWeaponC::SIZE,
            read.offset
        )?;
        let state: DetonateWeaponC = read.read_struct()?;

        let weapon = assert_utf8("detonate weapon name", read.prev + 0, || {
            state.weapon.to_str_padded()
        })?;
        let name = if state.node_index == index!(input) {
            INPUT_NODE_NAME.to_string()
        } else {
            anim_def.node_from_index(state.node_index, read.prev + 10)?
        };
        Ok(Self {
            weapon,
            at_node: AtNode {
                name,
                pos: state.translation,
            },
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let weapon = Ascii::from_str_padded(&self.weapon);
        let node_index = if self.at_node.name == INPUT_NODE_NAME {
            index!(input)
        } else {
            anim_def.node_to_index(&self.at_node.name)?
        };

        let detonate_weapon = DetonateWeaponC {
            weapon,
            node_index,
            translation: self.at_node.pos,
        };
        write.write_struct(&detonate_weapon)?;
        Ok(())
    }
}
