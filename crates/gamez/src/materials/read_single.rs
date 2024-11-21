use super::{CycleInfoC, MatType, MaterialC, MaterialFlags, RawMaterial, RawTexturedMaterial};
use log::{debug, trace};
use mech3ax_api_types::gamez::materials::{ColoredMaterial, CycleData, Material, TexturedMaterial};
use mech3ax_api_types::{u32_to_usize, Color};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_types::AsBytes as _;
use num_traits::FromPrimitive;
use std::io::Read;

pub(crate) fn read_material(
    read: &mut CountingReader<impl Read>,
    index: u32,
    ty: MatType,
) -> Result<RawMaterial> {
    debug!(
        "Reading material {} ({}) at {}",
        index,
        MaterialC::SIZE,
        read.offset
    );
    let material: MaterialC = read.read_struct()?;
    trace!("{:#?}", material);

    let bitflags = MaterialFlags::from_bits(material.flags).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid material flags, but was 0x{:02X} (at {})",
            material.flags,
            read.prev + 1
        )
    })?;

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

    let soil = FromPrimitive::from_u32(material.soil).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid soil type (0..13), but was {} (at {})",
            material.soil,
            read.prev + 32,
        )
    })?;

    let material = if bitflags.contains(MaterialFlags::TEXTURED) {
        // all color information comes from the texture
        assert_that!("alpha", material.alpha == 0xFF, read.prev + 0)?;
        assert_that!("rgb", material.rgb == 0x7FFF, read.prev + 2)?;
        assert_that!("color", material.color == Color::WHITE_FULL, read.prev + 4)?;

        let cycle_ptr = if flag_cycled {
            Some(material.cycle_ptr)
        } else {
            assert_that!("cycle ptr", material.cycle_ptr == 0, read.prev + 36)?;
            None
        };

        RawMaterial::Textured(RawTexturedMaterial {
            pointer: material.index,
            cycle_ptr,
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

pub(super) fn read_material_zero(
    read: &mut CountingReader<impl Read>,
    index: i16,
    ty: MatType,
) -> Result<()> {
    debug!(
        "Reading zero material {} ({}) at {}",
        index,
        MaterialC::SIZE,
        read.offset
    );
    let material: MaterialC = read.read_struct()?;

    assert_that!("field 00", material.alpha == 0x00, read.prev + 0)?;
    match ty {
        MatType::Ng => assert_that!(
            "flag",
            material.flags == MaterialFlags::FREE.bits(),
            read.prev + 1
        )?,
        MatType::Rc => assert_that!("flag", material.flags == 0, read.prev + 1)?,
    }
    assert_that!("rgb", material.rgb == 0x0000, read.prev + 2)?;
    assert_that!("color", material.color == Color::BLACK, read.prev + 4)?;
    assert_that!("index", material.index == 0, read.prev + 16)?;
    assert_that!("field 20", material.zero20 == 0.0, read.prev + 20)?;
    assert_that!("field 24", material.half24 == 0.0, read.prev + 24)?;
    assert_that!("field 28", material.half28 == 0.0, read.prev + 28)?;
    assert_that!("soil", material.soil == 0, read.prev + 32)?;
    assert_that!("cycle ptr", material.cycle_ptr == 0, read.prev + 36)?;

    Ok(())
}

pub(super) fn read_cycle(
    read: &mut CountingReader<impl Read>,
    material: RawMaterial,
    textures: &[String],
    index: usize,
) -> Result<Material> {
    Ok(match material {
        RawMaterial::Textured(mat) => {
            let texture_index = u32_to_usize(mat.pointer);
            assert_that!("texture index", texture_index < textures.len(), read.offset)?;
            let texture = textures[texture_index].clone();
            trace!(
                "Material {} texture `{}` index: {}",
                index,
                texture,
                texture_index
            );

            let cycle = if let Some(info_ptr) = mat.cycle_ptr {
                assert_that!("cycle info ptr", info_ptr != 0, read.prev + 0)?;

                debug!(
                    "Reading cycle info {} ({}) at {}",
                    index,
                    CycleInfoC::SIZE,
                    read.offset
                );
                let info: CycleInfoC = read.read_struct()?;
                trace!("{:#?}", info);

                let unk00 = assert_that!("field 00", bool info.unk00, read.prev + 0)?;
                assert_that!("field 08", info.zero08 == 0, read.prev + 8)?;
                // in MW: 2.0 <= info.unk12 <= 16.0
                // in CS: 0.0 <= info.unk12 <= 16.0
                assert_that!("field 12", 0.0 <= info.unk12 <= 16.0, read.prev + 12)?;
                assert_that!("cycle count", info.count1 == info.count2, read.prev + 20)?;
                assert_that!("cycle data ptr", info.data_ptr != 0, read.prev + 24)?;

                debug!(
                    "Reading {} x cycle textures {} at {}",
                    info.count1, index, read.offset
                );
                let textures = (0..info.count1)
                    .map(|_| {
                        let texture_index = u32_to_usize(read.read_u32()?);
                        assert_that!("texture index", texture_index < textures.len(), read.prev)?;
                        let texture = textures[texture_index].clone();
                        Ok(texture)
                    })
                    .collect::<Result<Vec<_>>>()?;

                Some(CycleData {
                    textures,
                    info_ptr,
                    data_ptr: info.data_ptr,
                    unk00,
                    unk04: info.unk04,
                    unk12: info.unk12,
                })
            } else {
                None
            };

            Material::Textured(TexturedMaterial {
                texture,
                // since this stores the index of the texture name, zero it out
                pointer: 0,
                cycle,
                soil: mat.soil,
                flag: mat.flag,
            })
        }
        RawMaterial::Colored(mat) => Material::Colored(mat),
    })
}
