use super::{MaterialC, MaterialCycleC, MaterialFlags, MatlType, RawMaterial, RawTexturedMaterial};
use mech3ax_api_types::gamez::materials::{ColoredMaterial, CycleData, Soil};
use mech3ax_api_types::gamez::{MechlibColoredMaterial, MechlibMaterial, MechlibTexturedMaterial};
use mech3ax_api_types::{Color, Count, IndexR, IndexR32};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, chk};
use mech3ax_types::Ptr;
use std::io::Read;

fn color(value: f32) -> Result<f32, String> {
    if value < 0.0 || value > 255.0 {
        Err(format!("expected {value} in 0.0..=255.0"))
    } else {
        Ok(value)
    }
}

fn tex_index(index: IndexR32, count: Count) -> Result<IndexR, String> {
    count.index_req_i32(index)
}

pub(super) fn assert_material(
    material: &MaterialC,
    offset: usize,
    texture_count: Count,
    ty: MatlType,
) -> Result<RawMaterial> {
    let bitflags = chk!(offset, ?material.flags)?;

    let material_flag_unknown = bitflags.contains(MaterialFlags::UNKNOWN);
    let material_flag_cycled = bitflags.contains(MaterialFlags::CYCLED);
    let material_flag_always = bitflags.contains(MaterialFlags::ALWAYS);
    let material_flag_free = bitflags.contains(MaterialFlags::FREE);

    let always = match ty {
        MatlType::Ng => true,
        MatlType::Rc => false,
    };
    chk!(offset + 1, material_flag_always == always)?;
    chk!(offset + 1, material_flag_free == false)?;

    chk!(offset, material.field20 == 0.0)?;
    chk!(offset, material.field24 == 0.5)?;
    chk!(offset, material.field28 == 0.5)?;

    let soil = chk!(offset, ?material.soil)?;

    let material = if bitflags.contains(MaterialFlags::TEXTURED) {
        // all color information comes from the texture
        chk!(offset, material.alpha == 0xFF)?;
        chk!(offset, material.rgb == 0x7FFF)?;
        chk!(offset, material.color == Color::WHITE_FULL)?;
        let texture_index = chk!(offset, tex_index(material.texture_index, texture_count))?;

        if material_flag_cycled {
            chk!(offset, material.cycle_ptr != Ptr::NULL)?;
        } else {
            chk!(offset, material.cycle_ptr == Ptr::NULL)?;
        }

        RawMaterial::Textured(RawTexturedMaterial {
            texture_index,
            soil,
            flag: material_flag_unknown,
            cycle_ptr: material.cycle_ptr,
        })
    } else {
        // alpha variable
        chk!(offset + 1, material_flag_unknown == false)?;
        chk!(offset + 1, material_flag_cycled == false)?;
        chk!(offset, material.rgb == 0x0000)?;
        chk!(offset, color(material.color.r))?;
        chk!(offset, color(material.color.g))?;
        chk!(offset, color(material.color.b))?;
        chk!(offset, material.texture_index == 0)?;
        chk!(offset, material.cycle_ptr == Ptr::NULL)?;

        RawMaterial::Colored(ColoredMaterial {
            color: material.color,
            alpha: material.alpha,
            soil,
        })
    };
    Ok(material)
}

pub(super) fn assert_material_zero(
    material: &MaterialC,
    offset: usize,
    ty: MatlType,
) -> Result<()> {
    chk!(offset, material.alpha == 0x00)?;
    match ty {
        MatlType::Ng => chk!(offset, material.flags == MaterialFlags::FREE)?,
        MatlType::Rc => chk!(offset, material.flags == MaterialFlags::empty())?,
    }
    chk!(offset, material.rgb == 0x0000)?;
    chk!(offset, material.color == Color::BLACK)?;
    chk!(offset, material.texture_index == 0)?;
    chk!(offset, material.field20 == 0.0)?;
    chk!(offset, material.field24 == 0.0)?;
    chk!(offset, material.field28 == 0.0)?;
    chk!(offset, material.soil == Soil::Default)?;
    chk!(offset, material.cycle_ptr == 0)?;

    Ok(())
}

pub(super) fn read_cycle(
    read: &mut CountingReader<impl Read>,
    ptr: Ptr,
    texture_count: Count,
) -> Result<CycleData> {
    let cycle: MaterialCycleC = read.read_struct()?;
    let offset = read.prev;

    let looping = chk!(offset, ?cycle.looping)?;
    chk!(offset, cycle.current_index == 0.0)?;
    let count = chk!(offset, ?cycle.tex_map_count)?;
    chk!(offset, cycle.tex_map_index == cycle.tex_map_count)?;
    chk!(offset, cycle.tex_map_ptr != Ptr::NULL)?;

    let texture_indices = count
        .iter()
        .map(|_| {
            let texture_index = IndexR32::new(read.read_i32()?);
            Ok(chk!(read.prev, tex_index(texture_index, texture_count))?)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(CycleData {
        texture_indices,
        looping,
        speed: cycle.speed,
        current_frame: cycle.current_frame,
        cycle_ptr: ptr.0,
        tex_map_ptr: cycle.tex_map_ptr.0,
    })
}

pub(crate) fn read_material_mechlib(
    read: &mut CountingReader<impl Read>,
) -> Result<MechlibMaterial> {
    let material: MaterialC = read.read_struct()?;
    let offset = read.prev;

    let bitflags = chk!(offset, ?material.flags)?;

    let material_flag_unknown = bitflags.contains(MaterialFlags::UNKNOWN);
    let material_flag_cycled = bitflags.contains(MaterialFlags::CYCLED);
    let material_flag_always = bitflags.contains(MaterialFlags::ALWAYS);
    let material_flag_free = bitflags.contains(MaterialFlags::FREE);

    chk!(offset + 1, material_flag_unknown == false)?;
    // mechlib cannot have cycled textures
    chk!(offset + 1, material_flag_cycled == false)?;
    chk!(offset + 1, material_flag_always == true)?;
    chk!(offset + 1, material_flag_free == false)?;

    chk!(offset, material.field20 == 0.0)?;
    chk!(offset, material.field24 == 0.5)?;
    chk!(offset, material.field28 == 0.5)?;

    chk!(offset, material.soil == Soil::Default)?;
    chk!(offset, material.cycle_ptr == Ptr::NULL)?;

    if bitflags.contains(MaterialFlags::TEXTURED) {
        // all color information comes from the texture
        chk!(offset, material.alpha == 0xFF)?;
        chk!(offset, material.rgb == 0x7FFF)?;
        chk!(offset, material.color == Color::WHITE_FULL)?;
        let ptr = material.texture_index.value as u32;

        let texture = read.read_string()?;

        Ok(MechlibMaterial::Textured(MechlibTexturedMaterial {
            texture_name: texture,
            ptr,
        }))
    } else {
        chk!(offset, material.rgb == 0x0000)?;
        chk!(offset, color(material.color.r))?;
        chk!(offset, color(material.color.g))?;
        chk!(offset, color(material.color.b))?;
        chk!(offset, material.texture_index == 0)?;

        Ok(MechlibMaterial::Colored(MechlibColoredMaterial {
            color: material.color,
            alpha: material.alpha,
        }))
    }
}
