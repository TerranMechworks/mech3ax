use super::write_single::{write_cycle, write_material};
use super::{find_texture_index_by_name, MaterialC, MaterialFlags, MaterialInfoC};
use log::{debug, trace};
use mech3ax_api_types::gamez::materials::Material;
use mech3ax_api_types::{Color, ReprSize as _};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, Result};
use std::io::Write;

pub fn write_materials(
    write: &mut CountingWriter<impl Write>,
    textures: &[String],
    materials: &[Material],
    array_size: i16,
) -> Result<()> {
    debug!(
        "Writing material info header ({}) at {}",
        MaterialInfoC::SIZE,
        write.offset
    );
    let materials_len = assert_len!(i16, materials.len(), "materials")?;
    // Cast safety: i32 > i16
    let count = materials_len as i32;

    let info = MaterialInfoC {
        array_size: array_size as i32,
        count,
        index_max: count,
        index_last: count - 1,
    };
    trace!("{:#?}", info);
    write.write_struct(&info)?;

    for (index, material) in (0i16..).zip(materials.iter()) {
        let pointer = if let Material::Textured(textured) = material {
            // reconstruct the texture index
            let texture_index = find_texture_index_by_name(textures, &textured.texture)?;
            trace!(
                "Material {} texture `{}` index: {}",
                index,
                textured.texture,
                texture_index
            );
            Some(texture_index)
        } else {
            None
        };
        // Cast safety: index is purely for debug here, and also >= 0
        write_material(write, material, pointer, index as usize)?;

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

    write_materials_zero(write, materials_len, array_size)?;

    for (index, material) in materials.iter().enumerate() {
        write_cycle(write, textures, material, index)?;
    }
    Ok(())
}

pub fn write_materials_zero(
    write: &mut CountingWriter<impl Write>,
    start: i16,
    end: i16,
) -> Result<()> {
    let material = MaterialC {
        alpha: 0,
        flags: MaterialFlags::FREE.bits(),
        rgb: 0x0000,
        color: Color::BLACK,
        index: 0,
        zero20: 0.0,
        half24: 0.0,
        half28: 0.0,
        specular: 0.0,
        cycle_ptr: 0,
    };

    for index in start..end {
        debug!(
            "Writing zero material {} ({}) at {}",
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
