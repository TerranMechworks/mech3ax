use super::write_single::{write_cycle, write_material};
use super::{MaterialC, MaterialFlags, MaterialInfoC};
use log::{debug, trace};
use mech3ax_api_types::{Color, Material, ReprSize as _};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_with_msg, Result};
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
    let count = materials.len() as i32;
    let info = MaterialInfoC {
        array_size: array_size as i32,
        count,
        index_max: count,
        index_last: count - 1,
    };
    trace!("{:#?}", info);
    write.write_struct(&info)?;

    let count = materials.len() as i16;
    for (index, material) in materials.iter().enumerate() {
        let pointer = if let Material::Textured(textured) = material {
            // reconstruct the texture index
            let texture_index = textures
                .iter()
                .position(|tex| tex == &textured.texture)
                .ok_or_else(|| {
                    assert_with_msg!("Texture `{}` not found in textures list", textured.texture)
                })?;
            Some(texture_index as u32)
        } else {
            None
        };
        write_material(write, material, pointer, index)?;

        let index = index as i16;
        let mut index1 = index + 1;
        if index1 >= count {
            index1 = -1;
        }
        write.write_i16(index1)?;

        let mut index2 = index - 1;
        if index2 < 0 {
            index2 = -1;
        }
        write.write_i16(index2)?;
    }

    write_materials_zero(write, count, array_size)?;

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
