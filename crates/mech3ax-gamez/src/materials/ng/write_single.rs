use super::{find_texture_index_by_name, CycleInfoC, MaterialC, MaterialFlags};
use log::{debug, trace};
use mech3ax_api_types::{Color, Material, ReprSize as _};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, bool_c, Result};
use std::io::Write;

pub fn write_material(
    write: &mut CountingWriter<impl Write>,
    material: &Material,
    pointer: Option<u32>,
    index: usize,
) -> Result<()> {
    debug!(
        "Writing material {} ({}) at {}",
        index,
        MaterialC::SIZE,
        write.offset
    );
    let mat_c = match material {
        Material::Textured(material) => {
            let mut bitflags = MaterialFlags::ALWAYS | MaterialFlags::TEXTURED;
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
                specular: material.specular,
                cycle_ptr,
            }
        }
        Material::Colored(material) => {
            let bitflags = MaterialFlags::ALWAYS;
            MaterialC {
                alpha: material.alpha,
                flags: bitflags.bits(),
                rgb: 0x0000,
                color: material.color,
                index: 0,
                zero20: 0.0,
                half24: 0.5,
                half28: 0.5,
                specular: material.specular,
                cycle_ptr: 0,
            }
        }
    };
    trace!("{:#?}", mat_c);
    write.write_struct(&mat_c)?;
    Ok(())
}

pub fn write_cycle(
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
