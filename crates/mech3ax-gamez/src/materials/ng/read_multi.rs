use super::read_single::{read_cycle, read_material, read_material_zero};
use super::{assert_material_info, MaterialInfoC};
use log::{debug, trace};
use mech3ax_api_types::gamez::Material;
use mech3ax_api_types::ReprSize as _;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Result};
use std::io::Read;

pub fn read_materials(
    read: &mut CountingReader<impl Read>,
    textures: &[String],
) -> Result<(Vec<Material>, i16)> {
    debug!(
        "Reading material info header ({}) at {}",
        MaterialInfoC::SIZE,
        read.offset
    );
    let info: MaterialInfoC = read.read_struct()?;
    trace!("{:#?}", info);

    let (array_size, count) = assert_material_info(info, read.prev)?;

    // read materials without cycle data
    let materials = (0..count)
        .map(|index| {
            // Cast safety: index is >= 0, and count is i16
            let material = read_material(read, index as u32)?;

            let mut expected_index1 = index + 1;
            if expected_index1 >= count {
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

    read_materials_zero(read, count, array_size)?;

    // now read cycle data
    let materials = materials
        .into_iter()
        .enumerate()
        .map(|(index, material)| read_cycle(read, material, textures, index))
        .collect::<Result<Vec<_>>>()?;

    Ok((materials, array_size))
}

pub fn read_materials_zero(
    read: &mut CountingReader<impl Read>,
    start: i16,
    end: i16,
) -> Result<()> {
    for index in start..end {
        read_material_zero(read, index)?;

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
