use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use mech3ax_api_types::{static_assert_size, AnimDef, AtNode, DetonateWeapon, ReprSize as _, Vec3};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[repr(C)]
struct DetonateWeaponC {
    name: [u8; 10],
    node_index: u16,
    translation: Vec3,
}
static_assert_size!(DetonateWeaponC, 24);

impl ScriptObject for DetonateWeapon {
    const INDEX: u8 = 41;
    const SIZE: u32 = DetonateWeaponC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("detonate weapon size", size == Self::SIZE, read.offset)?;
        let detonate_weapon: DetonateWeaponC = read.read_struct()?;
        let name = assert_utf8("detonate weapon name", read.prev + 0, || {
            str_from_c_padded(&detonate_weapon.name)
        })?;
        let node = anim_def.node_from_index(detonate_weapon.node_index as usize, read.prev + 10)?;
        Ok(Self {
            name,
            at_node: AtNode {
                node,
                translation: detonate_weapon.translation,
            },
        })
    }

    fn write(&self, write: &mut impl Write, anim_def: &AnimDef) -> Result<()> {
        let mut name = [0; 10];
        str_to_c_padded(&self.name, &mut name);
        write.write_struct(&DetonateWeaponC {
            name,
            node_index: anim_def.node_to_index(&self.at_node.node)? as u16,
            translation: self.at_node.translation,
        })?;
        Ok(())
    }
}
