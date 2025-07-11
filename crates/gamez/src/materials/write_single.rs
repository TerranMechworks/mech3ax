use super::{MaterialC, MaterialCycleC, MaterialFlags, MatlType};
use mech3ax_api_types::gamez::materials::{CycleData, Material, Soil};
use mech3ax_api_types::gamez::MechlibMaterial;
use mech3ax_api_types::{Color, Count, IndexR, IndexR32};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{err, len, Result};
use mech3ax_types::Ptr;
use std::io::Write;

fn tex_index(index: IndexR, count: Count) -> Result<IndexR32, String> {
    if index.to_i16() < count.to_i16() {
        Ok(index.maybe())
    } else {
        Err(format!(
            "Invalid texture index: expected {} < {}",
            index, count
        ))
    }
}

pub(super) fn make_material(
    material: &Material,
    texture_count: Count,
    ty: MatlType,
) -> Result<MaterialC> {
    match material {
        Material::Textured(textured) => {
            let texture_index =
                tex_index(textured.texture_index, texture_count).map_err(|msg| err!(msg))?;

            let mut flags = match ty {
                MatlType::Ng => MaterialFlags::ALWAYS | MaterialFlags::TEXTURED,
                MatlType::Rc => MaterialFlags::TEXTURED,
            };
            if textured.flag {
                flags |= MaterialFlags::UNKNOWN;
            }
            let cycle_ptr = if let Some(cycle) = &textured.cycle {
                flags |= MaterialFlags::CYCLED;
                Ptr(cycle.cycle_ptr)
            } else {
                Ptr::NULL
            };
            Ok(MaterialC {
                alpha: 0xFF,
                flags: flags.maybe(),
                rgb: 0x7FFF,
                color: Color::WHITE_FULL,
                texture_index,
                field20: 0.0,
                field24: 0.5,
                field28: 0.5,
                soil: textured.soil.maybe(),
                cycle_ptr,
            })
        }
        Material::Colored(colored) => {
            let flags = match ty {
                MatlType::Ng => MaterialFlags::ALWAYS,
                MatlType::Rc => MaterialFlags::empty(),
            };
            Ok(MaterialC {
                alpha: colored.alpha,
                flags: flags.maybe(),
                rgb: 0x0000,
                color: colored.color,
                texture_index: IndexR::ZERO.maybe(),
                field20: 0.0,
                field24: 0.5,
                field28: 0.5,
                soil: colored.soil.maybe(),
                cycle_ptr: Ptr::NULL,
            })
        }
    }
}

pub(super) fn make_material_zero(ty: MatlType) -> MaterialC {
    let flags = match ty {
        MatlType::Ng => MaterialFlags::FREE,
        MatlType::Rc => MaterialFlags::empty(),
    };
    MaterialC {
        alpha: 0,
        flags: flags.maybe(),
        rgb: 0x0000,
        color: Color::BLACK,
        texture_index: IndexR::ZERO.maybe(),
        field20: 0.0,
        field24: 0.0,
        field28: 0.0,
        soil: Soil::Default.maybe(),
        cycle_ptr: Ptr::NULL,
    }
}

pub(super) fn write_cycle(
    write: &mut CountingWriter<impl Write>,
    cycle: &CycleData,
    texture_count: Count,
) -> Result<()> {
    let count = len!(cycle.texture_indices.len(), "material cycle texture map")?;

    let cyc = MaterialCycleC {
        looping: cycle.looping.into(),
        current_frame: cycle.current_frame,
        current_index: 0.0,
        speed: cycle.speed,
        tex_map_count: count.maybe(),
        tex_map_index: count.maybe(),
        tex_map_ptr: Ptr(cycle.tex_map_ptr),
    };
    write.write_struct(&cyc)?;

    for texture_index in cycle.texture_indices.iter().copied() {
        let index = tex_index(texture_index, texture_count).map_err(|msg| err!(msg))?;
        write.write_i32(index.value)?;
    }
    Ok(())
}

pub(crate) fn write_material_mechlib(
    write: &mut CountingWriter<impl Write>,
    material: &MechlibMaterial,
) -> Result<()> {
    let matl = match material {
        MechlibMaterial::Textured(textured) => {
            let flags = MaterialFlags::ALWAYS | MaterialFlags::TEXTURED;
            MaterialC {
                alpha: 0xFF,
                flags: flags.maybe(),
                rgb: 0x7FFF,
                color: Color::WHITE_FULL,
                texture_index: IndexR32::new(textured.ptr as i32),
                field20: 0.0,
                field24: 0.5,
                field28: 0.5,
                soil: Soil::Default.maybe(),
                cycle_ptr: Ptr::NULL,
            }
        }
        MechlibMaterial::Colored(colored) => {
            let flags = MaterialFlags::ALWAYS;
            MaterialC {
                alpha: colored.alpha,
                flags: flags.maybe(),
                rgb: 0x000,
                color: colored.color,
                texture_index: IndexR32::empty(),
                field20: 0.0,
                field24: 0.5,
                field28: 0.5,
                soil: Soil::Default.maybe(),
                cycle_ptr: Ptr::NULL,
            }
        }
    };

    write.write_struct(&matl)?;

    if let MechlibMaterial::Textured(textured) = material {
        write.write_string(&textured.texture_name)?;
    }
    Ok(())
}
