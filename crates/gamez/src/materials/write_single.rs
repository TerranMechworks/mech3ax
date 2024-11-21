use super::{CycleInfoC, MatType, MaterialC, MaterialFlags};
use log::{debug, trace};
use mech3ax_api_types::gamez::materials::Material;
use mech3ax_api_types::Color;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, assert_with_msg, bool_c, Result};
use mech3ax_types::AsBytes as _;
use std::io::Write;

pub(super) fn find_texture_index_by_name(textures: &[String], texture_name: &str) -> Result<u32> {
    let texture_index = textures
        .iter()
        .position(|name| name == texture_name)
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
    index: usize,
    ty: MatType,
) -> Result<()> {
    debug!(
        "Writing material {} ({}) at {}",
        index,
        MaterialC::SIZE,
        write.offset
    );
    let mat_c = match material {
        Material::Textured(material) => {
            let mut bitflags = match ty {
                MatType::Ng => MaterialFlags::ALWAYS | MaterialFlags::TEXTURED,
                MatType::Rc => MaterialFlags::TEXTURED,
            };
            if material.flag {
                bitflags |= MaterialFlags::UNKNOWN;
            }
            let cycle_ptr = if let Some(cycle) = &material.cycle {
                bitflags |= MaterialFlags::CYCLED;
                cycle.info_ptr
            } else {
                0
            };
            MaterialC {
                alpha: 0xFF,
                flags: bitflags.bits(),
                rgb: 0x7FFF,
                color: Color::WHITE_FULL,
                // this allows GameZ to override the pointer with the texture index
                // (without mutating the material)
                index: pointer.unwrap_or(material.pointer),
                zero20: 0.0,
                half24: 0.5,
                half28: 0.5,
                soil: material.soil as u32,
                cycle_ptr,
            }
        }
        Material::Colored(material) => {
            let bitflags = match ty {
                MatType::Ng => MaterialFlags::ALWAYS,
                MatType::Rc => MaterialFlags::empty(),
            };
            MaterialC {
                alpha: material.alpha,
                flags: bitflags.bits(),
                rgb: 0x0000,
                color: material.color,
                index: 0,
                zero20: 0.0,
                half24: 0.5,
                half28: 0.5,
                soil: material.soil as u32,
                cycle_ptr: 0,
            }
        }
    };
    trace!("{:#?}", mat_c);
    write.write_struct(&mat_c)?;
    Ok(())
}

pub(super) fn write_cycle(
    write: &mut CountingWriter<impl Write>,
    textures: &[String],
    material: &Material,
    index: usize,
) -> Result<()> {
    if let Material::Textured(mat) = material {
        if let Some(cycle) = &mat.cycle {
            debug!(
                "Writing cycle info {} ({}) at {}",
                index,
                CycleInfoC::SIZE,
                write.offset
            );

            let unk00 = bool_c!(cycle.unk00);
            let count = assert_len!(u32, cycle.textures.len(), "cycle textures")?;
            let info = CycleInfoC {
                unk00,
                unk04: cycle.unk04,
                zero08: 0,
                unk12: cycle.unk12,
                count1: count,
                count2: count,
                data_ptr: cycle.data_ptr,
            };
            trace!("{:#?}", info);
            write.write_struct(&info)?;

            debug!(
                "Writing {} x cycle textures {} at {}",
                count, index, write.offset
            );
            for texture_name in &cycle.textures {
                let index = find_texture_index_by_name(textures, texture_name)?;
                write.write_u32(index)?;
            }
        }
    }
    Ok(())
}
