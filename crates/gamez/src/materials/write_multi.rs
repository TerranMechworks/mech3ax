use super::write_single::{make_material, make_material_zero, write_cycle};
use super::{MaterialArrayC, MatlType};
use log::trace;
use mech3ax_api_types::gamez::materials::Material;
use mech3ax_api_types::Count;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{err, len, Result};
use std::io::Write;

pub(super) fn write_materials(
    write: &mut CountingWriter<impl Write>,
    materials: &[Material],
    array_size: Count,
    texture_count: Count,
    ty: MatlType,
) -> Result<()> {
    let count = len!(materials.len(), "GameZ materials")?;
    let matl_array = make_material_array(count, array_size)?;
    write.write_struct(&matl_array)?;

    for (index, material) in (0i16..).zip(materials.iter()) {
        trace!("Processing material {}/{}", index, count);

        let matl = make_material(material, texture_count, ty)?;
        write.write_struct(&matl)?;

        // since materials_len <= i16::MAX, this is also true for index, so no
        // overflow is possible
        let mut material_index_next = index + 1;
        if material_index_next >= count.to_i16() {
            material_index_next = -1;
        }
        write.write_i16(material_index_next)?;

        // since index >= 0, no underflow possible
        let mut material_index_prev = index - 1;
        if material_index_prev < 0 {
            material_index_prev = -1;
        }
        write.write_i16(material_index_prev)?;
    }

    let zero_start = count.to_i16();
    let zero_end = array_size.to_i16();

    trace!(
        "Processing {}..{} material zeros at {}",
        zero_start,
        zero_end,
        write.offset
    );
    let matl_zero = make_material_zero(ty);
    for index in zero_start..zero_end {
        write.write_struct_no_log(&matl_zero)?;

        let mut material_index_prev = index - 1;
        if material_index_prev < zero_start {
            material_index_prev = -1;
        }
        write.write_i16(material_index_prev)?;

        let mut material_index_next = index + 1;
        if material_index_next >= zero_end {
            material_index_next = -1;
        }
        write.write_i16(material_index_next)?;
    }

    trace!("Processed material zeros at {}", write.offset);

    for (index, material) in materials.iter().enumerate() {
        // TODO: edition 2024 combine these into one if statement
        if let Material::Textured(textured) = material {
            if let Some(cycle) = &textured.cycle {
                trace!("Processing material cycle {}/{}", index, count);
                write_cycle(write, cycle, texture_count)?;
            }
        }
    }
    Ok(())
}

fn make_material_array(count: Count, array_size: Count) -> Result<MaterialArrayC> {
    if count > array_size {
        return Err(err!(
            "Too many GameZ materials: expected {} <= {}",
            count,
            array_size
        ));
    }
    let index_max = count.to_i32();

    Ok(MaterialArrayC {
        array_size: array_size.maybe(),
        count: count.maybe(),
        index_max,
        index_last: index_max - 1,
    })
}
