use super::read_single::{read_cycle, read_material, read_material_zero};
use super::{MatType, MaterialInfoC};
use log::debug;
use mech3ax_api_types::gamez::materials::Material;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Result};
use mech3ax_types::AsBytes as _;
use std::io::Read;

pub(crate) fn read_materials(
    read: &mut CountingReader<impl Read>,
    textures: &[String],
    ty: MatType,
) -> Result<(Vec<Material>, u32)> {
    debug!(
        "Reading material info header ({}) at {}",
        MaterialInfoC::SIZE,
        read.offset
    );
    let info: MaterialInfoC = read.read_struct()?;

    let (valid, material_count) = assert_material_info(info, ty, read.prev)?;

    // read materials without cycle data
    let materials = (0..valid)
        .map(|index| {
            // Cast safety: index is >= 0, and valid is i16
            let material = read_material(read, index as u32, ty)?;

            let mut expected_index1 = index + 1;
            if expected_index1 >= valid {
                expected_index1 = -1;
            }
            let actual_index1 = read.read_i16()?;
            assert_that!("mat index 1", actual_index1 == expected_index1, read.prev)?;

            let mut expected_index2 = index - 1;
            if expected_index2 < 0 {
                expected_index2 = -1;
            }
            let actual_index2 = read.read_i16()?;
            assert_that!("mat index 2", actual_index2 == expected_index2, read.prev)?;

            Ok(material)
        })
        .collect::<Result<Vec<_>>>()?;

    read_materials_zero(read, valid, ty)?;

    // now read cycle data
    let materials = materials
        .into_iter()
        .enumerate()
        .map(|(index, material)| read_cycle(read, material, textures, index))
        .collect::<Result<Vec<_>>>()?;

    Ok((materials, material_count))
}

fn assert_material_info(info: MaterialInfoC, ty: MatType, offset: usize) -> Result<(i16, u32)> {
    assert_that!("mat array size", 0 <= info.array_size <= ty.size_i32(), offset + 0)?;
    assert_that!("mat count", 0 <= info.count <= info.array_size, offset + 4)?;
    assert_that!("mat index max", info.index_max == info.count, offset + 8)?;
    assert_that!(
        "mat index last",
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
    for index in start..end {
        read_material_zero(read, index, ty)?;

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
