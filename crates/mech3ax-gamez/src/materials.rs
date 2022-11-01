use log::{debug, trace};
use mech3ax_api_types::{static_assert_size, Color, ColoredMaterial, Material, ReprSize as _};
use mech3ax_common::assert::AssertionError;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct MaterialC {
    unk00: u8, // alpha?
    flags: u8,
    rgb: u16,
    color: Color,
    pointer: u32,
    zero20: f32, // always 0.0
    half24: f32, // always 0.5
    half28: f32, // always 0.5
    unk32: u32,  // f32, specular?
    cycle_ptr: u32,
}
static_assert_size!(MaterialC, 40);
pub const MATERIAL_C_SIZE: u32 = MaterialC::SIZE;

#[derive(Debug)]
pub struct RawTexturedMaterial {
    pub pointer: u32,
    pub cycle_ptr: Option<u32>,
    pub unk32: u32,
    pub flag: bool,
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

pub fn read_material(read: &mut CountingReader<impl Read>, index: u32) -> Result<RawMaterial> {
    debug!(
        "Reading material {} (mw, {}) at {}",
        index,
        MaterialC::SIZE,
        read.offset
    );
    let material: MaterialC = read.read_struct()?;
    trace!("{:#?}", material);

    let bitflags = MaterialFlags::from_bits(material.flags).ok_or_else(|| {
        AssertionError(format!(
            "Expected valid material flags, but was 0x{:02X} (at {})",
            material.flags,
            read.prev + 1
        ))
    })?;

    let flag_unknown = bitflags.contains(MaterialFlags::UNKNOWN);
    let flag_cycled = bitflags.contains(MaterialFlags::CYCLED);
    let flag_always = bitflags.contains(MaterialFlags::ALWAYS);
    let flag_free = bitflags.contains(MaterialFlags::FREE);

    assert_that!("flag always", flag_always == true, read.prev + 1)?;
    assert_that!("flag free", flag_free == false, read.prev + 1)?;

    assert_that!("field 20", material.zero20 == 0.0, read.prev + 20)?;
    assert_that!("field 24", material.half24 == 0.5, read.prev + 24)?;
    assert_that!("field 28", material.half28 == 0.5, read.prev + 28)?;

    let material = if bitflags.contains(MaterialFlags::TEXTURED) {
        assert_that!("field 00", material.unk00 == 0xFF, read.prev + 0)?;
        assert_that!("rgb", material.rgb == 0x7FFF, read.prev + 2)?;
        assert_that!("color", material.color == Color::WHITE_FULL, read.prev + 4)?;

        let cycle_ptr = if flag_cycled {
            Some(material.cycle_ptr)
        } else {
            assert_that!("cycle ptr", material.cycle_ptr == 0, read.prev + 36)?;
            None
        };

        RawMaterial::Textured(RawTexturedMaterial {
            pointer: material.pointer,
            cycle_ptr,
            unk32: material.unk32,
            flag: flag_unknown,
        })
    } else {
        assert_that!("flag unknown", flag_unknown == false, read.prev + 1)?;
        assert_that!("flag cycled", flag_cycled == false, read.prev + 1)?;
        assert_that!("rgb", material.rgb == 0x0000, read.prev + 2)?;
        assert_that!("pointer", material.pointer == 0, read.prev + 16)?;
        assert_that!("cycle ptr", material.cycle_ptr == 0, read.prev + 36)?;

        RawMaterial::Colored(ColoredMaterial {
            color: material.color,
            unk00: material.unk00,
            unk32: material.unk32,
        })
    };
    Ok(material)
}

pub fn write_material(
    write: &mut CountingWriter<impl Write>,
    material: &Material,
    pointer: Option<u32>,
    index: usize,
) -> Result<()> {
    let mat_c = match material {
        Material::Textured(material) => {
            let mut bitflags = MaterialFlags::ALWAYS | MaterialFlags::TEXTURED;
            if material.flag {
                bitflags |= MaterialFlags::UNKNOWN;
            }
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
                color: Color::WHITE_FULL,
                // this allows GameZ to override the pointer with the texture index
                // (without mutating the material)
                pointer: pointer.unwrap_or(material.pointer),
                zero20: 0.0,
                half24: 0.5,
                half28: 0.5,
                unk32: material.unk32,
                cycle_ptr,
            }
        }
        Material::Colored(material) => {
            let bitflags = MaterialFlags::ALWAYS;
            MaterialC {
                unk00: material.unk00,
                flags: bitflags.bits(),
                rgb: 0x0000,
                color: material.color,
                pointer: 0,
                zero20: 0.0,
                half24: 0.5,
                half28: 0.5,
                unk32: material.unk32,
                cycle_ptr: 0,
            }
        }
    };
    debug!(
        "Writing material {} ({}) at {}",
        index,
        MaterialC::SIZE,
        write.offset
    );
    trace!("{:#?}", mat_c);
    write.write_struct(&mat_c)?;
    Ok(())
}

pub fn read_materials_zero(
    read: &mut CountingReader<impl Read>,
    start: i16,
    end: i16,
) -> Result<()> {
    for index in start..end {
        debug!(
            "Reading zero material {} (mw, {}) at {}",
            index,
            MaterialC::SIZE,
            read.offset
        );
        let material: MaterialC = read.read_struct()?;

        assert_that!("field 00", material.unk00 == 0, read.prev + 0)?;
        assert_that!(
            "flag",
            material.flags == MaterialFlags::FREE.bits(),
            read.prev + 1
        )?;
        assert_that!("rgb", material.rgb == 0x0000, read.prev + 2)?;
        assert_that!("color", material.color == Color::BLACK, read.prev + 4)?;
        assert_that!("pointer", material.pointer == 0, read.prev + 16)?;
        assert_that!("field 20", material.zero20 == 0.0, read.prev + 20)?;
        assert_that!("field 24", material.half24 == 0.0, read.prev + 24)?;
        assert_that!("field 28", material.half28 == 0.0, read.prev + 28)?;
        assert_that!("field 32", material.unk32 == 0, read.prev + 32)?;
        assert_that!("cycle ptr", material.cycle_ptr == 0, read.prev + 36)?;

        let mut expected_index1 = index - 1;
        if expected_index1 < start {
            expected_index1 = -1;
        }
        let actual_index1 = read.read_i16()?;
        assert_that!("mat index 1", actual_index1 == expected_index1, read.prev)?;

        let mut expected_index2 = index + 1;
        if expected_index2 >= end {
            expected_index2 = -1;
        }
        let actual_index2 = read.read_i16()?;
        assert_that!("mat index 2", actual_index2 == expected_index2, read.prev)?;
    }
    Ok(())
}

pub fn write_materials_zero(
    write: &mut CountingWriter<impl Write>,
    start: i16,
    end: i16,
) -> Result<()> {
    let material = MaterialC {
        unk00: 0,
        flags: MaterialFlags::FREE.bits(),
        rgb: 0x0000,
        color: Color::BLACK,
        pointer: 0,
        zero20: 0.0,
        half24: 0.0,
        half28: 0.0,
        unk32: 0,
        cycle_ptr: 0,
    };

    for index in start..end {
        debug!(
            "Writing zero material {} (mw, {}) at {}",
            index,
            MaterialC::SIZE,
            write.offset
        );
        write.write_struct(&material)?;

        let mut index1 = index - 1;
        if index1 < start {
            index1 = -1;
        }
        write.write_i16(index1)?;

        let mut index2 = index + 1;
        if index2 >= end {
            index2 = -1;
        }
        write.write_i16(index2)?;
    }
    Ok(())
}
