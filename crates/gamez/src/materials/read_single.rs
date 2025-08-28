use super::{CycleInfoC, MatType, MaterialC, MaterialFlags, RawMaterial, RawTexturedMaterial};
use log::trace;
use mech3ax_api_types::gamez::materials::{ColoredMaterial, CycleData, Soil, TexturedMaterial};
use mech3ax_api_types::Color;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Result};
use mech3ax_types::u32_to_usize;
use std::io::Read;

pub(crate) fn read_material(
    read: &mut CountingReader<impl Read>,
    ty: MatType,
) -> Result<RawMaterial> {
    let material: MaterialC = read.read_struct()?;

    let bitflags = assert_that!("material flags", flags material.flags, read.prev + 1)?;

    let flag_unknown = bitflags.contains(MaterialFlags::UNKNOWN);
    let flag_cycled = bitflags.contains(MaterialFlags::CYCLED);
    let flag_always = bitflags.contains(MaterialFlags::ALWAYS);
    let flag_free = bitflags.contains(MaterialFlags::FREE);

    if ty == MatType::Ng {
        assert_that!("flag always (ng)", flag_always == true, read.prev + 1)?;
    } else {
        assert_that!("flag always (rc)", flag_always == false, read.prev + 1)?;
    }
    assert_that!("flag free", flag_free == false, read.prev + 1)?;

    assert_that!("field 20", material.zero20 == 0.0, read.prev + 20)?;
    assert_that!("field 24", material.half24 == 0.5, read.prev + 24)?;
    assert_that!("field 28", material.half28 == 0.5, read.prev + 28)?;

    let soil = assert_that!("soil type", enum Soil => material.soil, read.prev + 32)?;

    let material = if bitflags.contains(MaterialFlags::TEXTURED) {
        // all color information comes from the texture
        assert_that!("alpha", material.alpha == 0xFF, read.prev + 0)?;
        assert_that!("rgb", material.rgb == 0x7FFF, read.prev + 2)?;
        assert_that!("color", material.color == Color::WHITE_FULL, read.prev + 4)?;

        if flag_cycled {
            assert_that!("cycle ptr", material.cycle_ptr != 0, read.prev + 36)?;
        } else {
            assert_that!("cycle ptr", material.cycle_ptr == 0, read.prev + 36)?;
        }

        RawMaterial::Textured(RawTexturedMaterial {
            pointer: material.index,
            cycle_ptr: material.cycle_ptr,
            soil,
            flag: flag_unknown,
        })
    } else {
        assert_that!("flag unknown", flag_unknown == false, read.prev + 1)?;
        assert_that!("flag cycled", flag_cycled == false, read.prev + 1)?;
        assert_that!("rgb", material.rgb == 0x0000, read.prev + 2)?;
        assert_that!("color r", 0.0 <= material.color.r <= 255.0, read.prev + 4)?;
        assert_that!("color g", 0.0 <= material.color.g <= 255.0, read.prev + 8)?;
        assert_that!("color b", 0.0 <= material.color.b <= 255.0, read.prev + 12)?;
        assert_that!("index", material.index == 0, read.prev + 16)?;

        assert_that!("cycle ptr", material.cycle_ptr == 0, read.prev + 36)?;

        RawMaterial::Colored(ColoredMaterial {
            color: material.color,
            alpha: material.alpha,
            soil,
        })
    };
    Ok(material)
}

pub(super) fn assert_material_zero(material: &MaterialC, ty: MatType, offset: usize) -> Result<()> {
    assert_that!("field 00", material.alpha == 0x00, offset + 0)?;
    match ty {
        MatType::Ng => assert_that!(
            "flag",
            material.flags == MaterialFlags::FREE.maybe(),
            offset + 1
        )?,
        MatType::Rc => assert_that!(
            "flag",
            material.flags == MaterialFlags::empty().maybe(),
            offset + 1
        )?,
    }
    assert_that!("rgb", material.rgb == 0x0000, offset + 2)?;
    assert_that!("color", material.color == Color::BLACK, offset + 4)?;
    assert_that!("index", material.index == 0, offset + 16)?;
    assert_that!("field 20", material.zero20 == 0.0, offset + 20)?;
    assert_that!("field 24", material.half24 == 0.0, offset + 24)?;
    assert_that!("field 28", material.half28 == 0.0, offset + 28)?;
    assert_that!("soil", material.soil == 0, offset + 32)?;
    assert_that!("cycle ptr", material.cycle_ptr == 0, offset + 36)?;

    Ok(())
}

pub(super) fn read_cycle(
    read: &mut CountingReader<impl Read>,
    mat: &mut TexturedMaterial,
    textures: &[String],
) -> Result<()> {
    let info: CycleInfoC = read.read_struct()?;

    let unk00 = assert_that!("field 00", bool info.unk00, read.prev + 0)?;
    assert_that!("field 08", info.zero08 == 0, read.prev + 8)?;
    // in MW: 2.0 <= info.unk12 <= 16.0
    // in CS: 0.0 <= info.unk12 <= 16.0
    assert_that!("field 12", 0.0 <= info.unk12 <= 16.0, read.prev + 12)?;
    assert_that!("cycle count", info.count1 == info.count2, read.prev + 20)?;
    assert_that!("cycle data ptr", info.data_ptr != 0, read.prev + 24)?;

    let textures = (0..info.count1)
        .map(|_| {
            let texture_index = u32_to_usize(read.read_u32()?);
            assert_that!("texture index", texture_index < textures.len(), read.prev)?;
            let texture = textures[texture_index].clone();
            trace!("{} -> `{}`", texture_index, texture);
            Ok(texture)
        })
        .collect::<Result<Vec<_>>>()?;

    mat.cycle = Some(CycleData {
        textures,
        info_ptr: mat.pointer,
        data_ptr: info.data_ptr,
        unk00,
        unk04: info.unk04,
        unk12: info.unk12,
    });
    // undo the horrible hack
    mat.pointer = 0;

    Ok(())
}
