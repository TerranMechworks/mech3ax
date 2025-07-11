use super::read_single::{assert_material, assert_material_zero, read_cycle};
use super::{MaterialArrayC, MaterialC, MatlType};
use crate::materials::RawMaterial;
use log::trace;
use mech3ax_api_types::gamez::materials::{Material, TexturedMaterial};
use mech3ax_api_types::Count;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{chk, Result};
use mech3ax_types::Ptr;
use std::io::Read;

pub(super) fn read_materials(
    read: &mut CountingReader<impl Read>,
    texture_count: Count,
    ty: MatlType,
) -> Result<(Vec<Material>, Count, Count)> {
    let material_array: MaterialArrayC = read.read_struct()?;
    let (count, array_size) = assert_material_array(material_array, read.prev)?;

    let materials = (0..count.to_i16())
        .map(|index| {
            trace!("Processing material {}/{}", index, count);
            let material: MaterialC = read.read_struct()?;
            let matl = assert_material(&material, read.prev, texture_count, ty)?;

            let mut expected_index_next = index + 1;
            if expected_index_next >= count.to_i16() {
                expected_index_next = -1;
            }
            let material_index_next = read.read_i16()?;
            chk!(read.prev, material_index_next == expected_index_next)?;

            let mut expected_index_prev = index - 1;
            if expected_index_prev < 0 {
                expected_index_prev = -1;
            }
            let material_index_prev = read.read_i16()?;
            chk!(read.prev, material_index_prev == expected_index_prev)?;

            Ok(matl)
        })
        .collect::<Result<Vec<_>>>()?;

    let zero_start = count.to_i16();
    let zero_end = array_size.to_i16();

    trace!(
        "Processing {}..{} material zeros at {}",
        zero_start,
        zero_end,
        read.offset
    );
    for index in zero_start..zero_end {
        let material: MaterialC = read.read_struct_no_log()?;
        assert_material_zero(&material, read.prev, ty)
            .inspect_err(|_e| trace!("{:#?} (index: {}, at {})", material, index, read.prev))?;

        let mut expected_index_prev = index - 1;
        if expected_index_prev < zero_start {
            expected_index_prev = -1;
        }
        let material_index_prev = read.read_i16()?;
        chk!(read.prev, material_index_prev == expected_index_prev)?;

        let mut expected_index_next = index + 1;
        if expected_index_next >= zero_end {
            expected_index_next = -1;
        }
        let material_index_next = read.read_i16()?;
        chk!(read.prev, material_index_next == expected_index_next)?;
    }

    trace!("Processed material zeros at {}", read.offset);

    let materials = materials
        .into_iter()
        .enumerate()
        .map(|(index, matl)| match matl {
            RawMaterial::Colored(colored) => Ok(Material::Colored(colored)),
            RawMaterial::Textured(textured) => {
                let cycle = if textured.cycle_ptr != Ptr::NULL {
                    trace!("Processing material cycle {}/{}", index, count);
                    Some(read_cycle(read, textured.cycle_ptr, texture_count)?)
                } else {
                    None
                };
                Ok(Material::Textured(TexturedMaterial {
                    texture_index: textured.texture_index,
                    soil: textured.soil,
                    cycle,
                    flag: textured.flag,
                }))
            }
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((materials, count, array_size))
}

fn assert_material_array(material_array: MaterialArrayC, offset: usize) -> Result<(Count, Count)> {
    let material_array_size = chk!(offset, ?material_array.array_size)?;
    let material_count = chk!(offset, ?material_array.count)?;
    chk!(offset, material_count <= material_array_size)?;
    let index_max = material_count.to_i32();
    chk!(offset, material_array.index_max == index_max)?;
    chk!(offset, material_array.index_last == index_max - 1)?;
    Ok((material_count, material_array_size))
}
