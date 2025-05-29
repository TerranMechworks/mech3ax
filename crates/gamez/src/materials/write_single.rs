use super::{CycleInfoC, MatType, MaterialC, MaterialFlags};
use log::trace;
use mech3ax_api_types::gamez::materials::Material;
use mech3ax_api_types::Color;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, assert_with_msg, Result};
use std::io::Write;

pub(super) fn find_texture_index_by_name(
    texture_names: &[&String],
    texture_name: &str,
) -> Result<u32> {
    let texture_index = texture_names
        .iter()
        .position(|name| *name == texture_name)
        .ok_or_else(|| assert_with_msg!("Texture `{}` not found in textures list", texture_name))?;
    // Cast safety: truncation only results in the wrong texture
    // index being written. Additionally writing the textures
    // should've already failed.
    Ok(texture_index as u32)
}

pub(crate) fn write_material(
    write: &mut CountingWriter<impl Write>,
    material: &Material,
    pointer: Option<u32>,
    ty: MatType,
) -> Result<()> {
    let mat_c = match material {
        Material::Textured(material) => {
            let mut flags = match ty {
                MatType::Ng => MaterialFlags::ALWAYS | MaterialFlags::TEXTURED,
                MatType::Rc => MaterialFlags::TEXTURED,
            };
            if material.flag {
                flags |= MaterialFlags::UNKNOWN;
            }
            let cycle_ptr = if let Some(cycle) = &material.cycle {
                flags |= MaterialFlags::CYCLED;
                cycle.info_ptr
            } else {
                0
            };
            MaterialC {
                alpha: 0xFF,
                flags: flags.maybe(),
                rgb: 0x7FFF,
                color: Color::WHITE_FULL,
                // this allows GameZ to override the pointer with the texture index
                // (without mutating the material)
                index: pointer.unwrap_or(material.pointer),
                zero20: 0.0,
                half24: 0.5,
                half28: 0.5,
                soil: material.soil.maybe(),
                cycle_ptr,
            }
        }
        Material::Colored(material) => {
            let flags = match ty {
                MatType::Ng => MaterialFlags::ALWAYS,
                MatType::Rc => MaterialFlags::empty(),
            };
            MaterialC {
                alpha: material.alpha,
                flags: flags.maybe(),
                rgb: 0x0000,
                color: material.color,
                index: 0,
                zero20: 0.0,
                half24: 0.5,
                half28: 0.5,
                soil: material.soil.maybe(),
                cycle_ptr: 0,
            }
        }
    };
    write.write_struct(&mat_c)?;
    Ok(())
}

pub(super) fn write_cycle(
    write: &mut CountingWriter<impl Write>,
    texture_names: &[&String],
    material: &Material,
    index: usize,
) -> Result<()> {
    if let Material::Textured(mat) = material {
        if let Some(cycle) = &mat.cycle {
            trace!("Writing cycle info {}", index);

            let count = assert_len!(u32, cycle.textures.len(), "cycle textures")?;
            let info = CycleInfoC {
                unk00: cycle.unk00.into(),
                unk04: cycle.unk04,
                zero08: 0,
                unk12: cycle.unk12,
                count1: count,
                count2: count,
                data_ptr: cycle.data_ptr,
            };
            write.write_struct(&info)?;

            for texture_name in &cycle.textures {
                let index = find_texture_index_by_name(texture_names, texture_name)?;
                trace!("`{}` -> {}", texture_name, index);
                write.write_u32(index)?;
            }
        }
    }
    Ok(())
}
