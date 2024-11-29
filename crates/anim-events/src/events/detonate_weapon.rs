use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{AtNode, DetonateWeapon};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::Vec3;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Ascii};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct DetonateWeaponC {
    name: Ascii<10>,
    node_index: u16,
    translation: Vec3,
}
impl_as_bytes!(DetonateWeaponC, 24);

impl ScriptObject for DetonateWeapon {
    const INDEX: u8 = 41;
    const SIZE: u32 = DetonateWeaponC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("detonate weapon size", size == Self::SIZE, read.offset)?;
        let detonate_weapon: DetonateWeaponC = read.read_struct()?;
        let name = assert_utf8("detonate weapon name", read.prev + 0, || {
            detonate_weapon.name.to_str_padded()
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

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let name = Ascii::from_str_padded(&self.name);
        write.write_struct(&DetonateWeaponC {
            name,
            node_index: anim_def.node_to_index(&self.at_node.node)? as u16,
            translation: self.at_node.translation,
        })?;
        Ok(())
    }
}
