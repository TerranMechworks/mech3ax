use super::read_single::{assert_material_zero, read_cycle, read_material};
use super::{MatType, MaterialC, MaterialInfoC};
use crate::materials::RawMaterial;
use log::trace;
use mech3ax_api_types::gamez::materials::{Material, TexturedMaterial};
use mech3ax_api_types::gamez::Texture;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Result};
use mech3ax_types::u32_to_usize;
use std::io::Read;

pub(crate) fn read_materials(
    read: &mut CountingReader<impl Read>,
    textures: &[Texture],
    ty: MatType,
) -> Result<(Vec<Material>, u32)> {
    let info: MaterialInfoC = read.read_struct()?;

    let (valid, material_count) = assert_material_info(info, ty, read.prev)?;

    // read materials without cycle data
    let mut materials = (0..valid)
        .map(|index| {
            trace!("Reading material {}/{}", index, valid);
            let material = read_material(read, ty)?;

            let material = match material {
                RawMaterial::Textured(mat) => {
                    let texture_index = u32_to_usize(mat.pointer);
                    assert_that!(
                        "matl texture index",
                        texture_index < textures.len(),
                        read.offset
                    )?;
                    let texture = textures[texture_index].name.clone();
                    trace!("{} -> `{}`", texture, texture_index);

                    Material::Textured(TexturedMaterial {
                        texture,
                        // since this stores the index of the texture name, zero
                        // it out... later. in the meantime, use it for the
                        // cycle ptr
                        pointer: mat.cycle_ptr,
                        // will be filled in later
                        cycle: None,
                        soil: mat.soil,
                        flag: mat.flag,
                    })
                }
                RawMaterial::Colored(mat) => Material::Colored(mat),
            };

            let mut expected_index1 = index + 1;
            if expected_index1 >= valid {
                expected_index1 = -1;
            }
            let actual_index1 = read.read_i16()?;
            assert_that!("matl index 1", actual_index1 == expected_index1, read.prev)?;

            let mut expected_index2 = index - 1;
            if expected_index2 < 0 {
                expected_index2 = -1;
            }
            let actual_index2 = read.read_i16()?;
            assert_that!("matl index 2", actual_index2 == expected_index2, read.prev)?;

            Ok(material)
        })
        .collect::<Result<Vec<_>>>()?;

    read_materials_zero(read, valid, ty)?;

    // now read cycle data
    for (index, material) in materials.iter_mut().enumerate() {
        match material {
            Material::Colored(_) => {}
            Material::Textured(mat) if mat.pointer == 0 => {}
            Material::Textured(mat) => {
                trace!("Reading cycle info {}", index);
                read_cycle(read, mat, textures)?;
            }
        }
    }

    Ok((materials, material_count))
}

fn assert_material_info(info: MaterialInfoC, ty: MatType, offset: usize) -> Result<(i16, u32)> {
    assert_that!("matl array size", 0 <= info.array_size <= ty.size_i32(), offset + 0)?;
    assert_that!("matl count", 0 <= info.count <= info.array_size, offset + 4)?;
    assert_that!("matl index max", info.index_max == info.count, offset + 8)?;
    assert_that!(
        "matl index last",
        info.index_last == info.count - 1,
        offset + 12
    )?;

    // Cast safety: see asserts above
    let valid = info.count as i16;
    let material_count = info.count as u32;
    Ok((valid, material_count))
}

fn read_materials_zero(
    read: &mut CountingReader<impl Read>,
    start: i16,
    ty: MatType,
) -> Result<()> {
    let end = ty.size_i16();
    trace!(
        "Reading {}..{} material zeros at {}",
        start,
        end,
        read.offset
    );
    for index in start..end {
        let material: MaterialC = read.read_struct_no_log()?;
        assert_material_zero(&material, ty, read.prev)
            .inspect_err(|_e| trace!("{:#?} (index: {}, at {})", material, index, read.prev))?;

        let mut expected_index1 = index - 1;
        if expected_index1 < start {
            expected_index1 = -1;
        }
        let actual_index1 = read.read_i16()?;
        assert_that!("matl index 1", actual_index1 == expected_index1, read.prev)?;

        let mut expected_index2 = index + 1;
        if expected_index2 >= end {
            expected_index2 = -1;
        }
        let actual_index2 = read.read_i16()?;
        assert_that!("matl index 2", actual_index2 == expected_index2, read.prev)?;
    }
    trace!("Read material zeros at {}", read.offset);
    Ok(())
}
