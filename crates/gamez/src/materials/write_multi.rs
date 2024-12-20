use super::write_single::{find_texture_index_by_name, write_cycle, write_material};
use super::{MatType, MaterialC, MaterialFlags, MaterialInfoC};
use log::trace;
use mech3ax_api_types::gamez::materials::{Material, Soil};
use mech3ax_api_types::Color;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, Result};
use std::io::Write;

pub(crate) fn write_materials(
    write: &mut CountingWriter<impl Write>,
    textures: &[String],
    materials: &[Material],
    ty: MatType,
) -> Result<()> {
    let materials_len = assert_len!(i16, materials.len(), "materials")?;
    // Cast safety: i32 > i16
    let count = materials_len as i32;

    let info = MaterialInfoC {
        array_size: ty.size_i32(),
        count,
        index_max: count,
        index_last: count - 1,
    };
    write.write_struct(&info)?;

    for (index, material) in (0i16..).zip(materials.iter()) {
        trace!("Writing material {}/{}", index, materials_len);

        let pointer = if let Material::Textured(textured) = material {
            // reconstruct the texture index
            let texture_index = find_texture_index_by_name(textures, &textured.texture)?;
            trace!("`{}` -> {}", textured.texture, texture_index);
            Some(texture_index)
        } else {
            None
        };

        write_material(write, material, pointer, ty)?;

        // since materials_len <= i16::MAX, this is also true for index, so no
        // overflow is possible
        let mut index1 = index + 1;
        if index1 >= materials_len {
            index1 = -1;
        }
        write.write_i16(index1)?;

        // since index >= 0, no underflow possible
        let mut index2 = index - 1;
        if index2 < 0 {
            index2 = -1;
        }
        write.write_i16(index2)?;
    }

    write_materials_zero(write, materials_len, ty)?;

    for (index, material) in materials.iter().enumerate() {
        write_cycle(write, textures, material, index)?;
    }
    Ok(())
}

fn write_materials_zero(
    write: &mut CountingWriter<impl Write>,
    start: i16,
    ty: MatType,
) -> Result<()> {
    let flags = match ty {
        MatType::Ng => MaterialFlags::FREE,
        MatType::Rc => MaterialFlags::empty(),
    };
    let material = MaterialC {
        alpha: 0,
        flags: flags.maybe(),
        rgb: 0x0000,
        color: Color::BLACK,
        index: 0,
        zero20: 0.0,
        half24: 0.0,
        half28: 0.0,
        soil: Soil::Default.maybe(),
        cycle_ptr: 0,
    };

    let end = ty.size_i16();
    trace!(
        "Writing {}..{} material zeros at {}",
        start,
        end,
        write.offset
    );
    for index in start..end {
        write.write_struct_no_log(&material)?;

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
    trace!("Wrote material zeros at {}", write.offset);
    Ok(())
}
