use crate::assert::AssertionError;
use crate::io_ext::{ReadHelper, WriteHelper};
use crate::size::ReprSize;
use crate::{assert_that, static_assert_size, Result};
use ::serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct MaterialC {
    unk00: u8,
    flags: u8,
    rgb: u16,
    red: f32,
    green: f32,
    blue: f32,
    pointer: u32,
    unk20: f32,
    unk24: f32,
    unk28: f32,
    unk32: u32,
    cycle_ptr: u32,
}
static_assert_size!(MaterialC, 40);

fn pointer_zero(pointer: &u32) -> bool {
    *pointer == 0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CycleData {
    info_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TexturedMaterial {
    pub texture: String,
    // the GameZ data doesn't use the pointer (it stores the texture name index)
    #[serde(skip_serializing_if = "pointer_zero", default)]
    pub pointer: u32,
    // the Mechlib data doesn't have cycled textures
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cycle: Option<CycleData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColoredMaterial {
    pub color: (f32, f32, f32),
    pub unk00: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Material {
    Textured(TexturedMaterial),
    Colored(ColoredMaterial),
}

#[derive(Debug)]
pub struct RawTexturedMaterial {
    pub pointer: u32,
    pub cycle_ptr: Option<u32>,
}

#[derive(Debug)]
pub enum RawMaterial {
    Textured(RawTexturedMaterial),
    Colored(ColoredMaterial),
}

bitflags::bitflags! {
    struct MaterialFlags: u8 {
        const TEXTURED = 1 << 0;
        const UNKNOWN = 1 << 1;
        const CYCLED = 1 << 2;
        const ALWAYS = 1 << 4;
        const FREE = 1 << 5;
    }
}

pub fn read_material<R>(read: &mut R, offset: &mut u32) -> Result<RawMaterial>
where
    R: Read,
{
    let material: MaterialC = read.read_struct()?;
    let bitflags = MaterialFlags::from_bits(material.flags).ok_or(AssertionError(format!(
        "Expected valid flag, but was {:X} (at {})",
        material.flags,
        *offset + 1
    )))?;

    let flag_unknown = bitflags.contains(MaterialFlags::UNKNOWN);
    let flag_cycled = bitflags.contains(MaterialFlags::CYCLED);
    let flag_always = bitflags.contains(MaterialFlags::ALWAYS);
    let flag_free = bitflags.contains(MaterialFlags::FREE);

    assert_that!("flag unknown", flag_unknown == false, *offset + 1)?;
    assert_that!("flag always", flag_always == true, *offset + 1)?;
    assert_that!("flag free", flag_free == false, *offset + 1)?;

    assert_that!("field 20", material.unk20 == 0.0, *offset + 20)?;
    assert_that!("field 24", material.unk24 == 0.5, *offset + 24)?;
    assert_that!("field 28", material.unk28 == 0.5, *offset + 28)?;
    assert_that!("field 32", material.unk32 == 0, *offset + 32)?;

    let material = if bitflags.contains(MaterialFlags::TEXTURED) {
        assert_that!("field 00", material.unk00 == 0xFF, *offset + 0)?;
        assert_that!("rgb", material.rgb == 0x7FFF, *offset + 2)?;
        assert_that!("color r", material.red == 255.0, *offset + 4)?;
        assert_that!("color g", material.green == 255.0, *offset + 8)?;
        assert_that!("color b", material.blue == 255.0, *offset + 12)?;

        let cycle_ptr = if flag_cycled {
            Some(material.cycle_ptr)
        } else {
            assert_that!("cycle ptr", material.cycle_ptr == 0, *offset + 36)?;
            None
        };

        *offset += MaterialC::SIZE;
        RawMaterial::Textured(RawTexturedMaterial {
            pointer: material.pointer,
            cycle_ptr,
        })
    } else {
        //assert_that!("field 00", material.unk00 == 0x00, *offset + 0)?;
        assert_that!("rgb", material.rgb == 0x0000, *offset + 2)?;
        assert_that!("pointer", material.pointer == 0, *offset + 16)?;
        assert_that!("flag cycled", flag_cycled == false, *offset + 1)?;
        assert_that!("cycle ptr", material.cycle_ptr == 0, *offset + 36)?;

        *offset += MaterialC::SIZE;
        RawMaterial::Colored(ColoredMaterial {
            color: (material.red, material.green, material.blue),
            unk00: material.unk00,
        })
    };
    Ok(material)
}

pub fn write_material<W>(write: &mut W, material: &Material) -> Result<()>
where
    W: Write,
{
    let mat_c = match material {
        Material::Textured(material) => {
            let mut bitflags = MaterialFlags::ALWAYS | MaterialFlags::TEXTURED;
            let cycle_ptr = if let Some(cycle) = &material.cycle {
                bitflags |= MaterialFlags::CYCLED;
                cycle.info_ptr
            } else {
                0
            };
            MaterialC {
                unk00: 0xFF,
                flags: bitflags.bits(),
                rgb: 0x7FFF,
                red: 255.0,
                green: 255.0,
                blue: 255.0,
                pointer: material.pointer,
                unk20: 0.0,
                unk24: 0.5,
                unk28: 0.5,
                unk32: 0,
                cycle_ptr,
            }
        }
        Material::Colored(material) => {
            let bitflags = MaterialFlags::ALWAYS;
            let (red, green, blue) = material.color;
            MaterialC {
                unk00: material.unk00,
                flags: bitflags.bits(),
                rgb: 0x0000,
                red,
                green,
                blue,
                pointer: 0,
                unk20: 0.0,
                unk24: 0.5,
                unk28: 0.5,
                unk32: 0,
                cycle_ptr: 0,
            }
        }
    };
    write.write_struct(&mat_c)?;
    Ok(())
}
