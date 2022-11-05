use super::read_single::{read_material, read_material_zero};
use super::MaterialInfoC;
use log::{debug, trace};
use mech3ax_api_types::{Material, ReprSize as _};
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

    assert_that!("mat array size", 0 <= info.array_size <= i16::MAX as i32, read.prev + 0)?;
    assert_that!("mat count", 0 <= info.count <= info.array_size, read.prev + 0)?;
    assert_that!("mat index max", info.index_max == info.count, read.prev + 8)?;
    assert_that!(
        "mat index last",
        info.index_last == info.count - 1,
        read.prev + 12
    )?;

    let count = info.count as i16;
    let array_size = info.array_size as i16;

    let materials = (0..count)
        .map(|index| {
            // Cast safety: index is >= 0, and count is i16
            let material = read_material(read, textures, index as _)?;

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
