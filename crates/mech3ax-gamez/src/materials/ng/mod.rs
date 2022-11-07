//! GameZ and mechlib material support for MW, PM, CS
mod read_multi;
mod read_single;
mod write_multi;
mod write_single;

use mech3ax_api_types::{static_assert_size, Color, ColoredMaterial, Material, ReprSize as _};

#[derive(Debug)]
#[repr(C)]
struct MaterialInfoC {
    array_size: i32,
    count: i32,
    index_max: i32,
    index_last: i32,
}
static_assert_size!(MaterialInfoC, 16);

#[derive(Debug)]
#[repr(C)]
struct MaterialC {
    alpha: u8,      // 00
    flags: u8,      // 01
    rgb: u16,       // 02
    color: Color,   // 04
    index: u32,     // 16, ptr in mechlib, texture index in gamez
    zero20: f32,    // 20
    half24: f32,    // 24
    half28: f32,    // 28
    specular: f32,  // 32
    cycle_ptr: u32, // 36
}
static_assert_size!(MaterialC, 40);

#[derive(Debug)]
#[repr(C)]
struct CycleInfoC {
    unk00: u32,
    unk04: u32,
    zero08: u32,
    unk12: f32,
    count1: u32,
    count2: u32,
    data_ptr: u32,
}
static_assert_size!(CycleInfoC, 28);

#[derive(Debug)]
pub struct RawTexturedMaterial {
    pub pointer: u32,
    pub cycle_ptr: Option<u32>,
    pub specular: f32,
    pub flag: bool,
}

#[derive(Debug)]
pub enum RawMaterial {
    Textured(RawTexturedMaterial),
    Colored(ColoredMaterial),
}

bitflags::bitflags! {
    struct MaterialFlags: u8 {
        const TEXTURED = 1 << 0;
        const UNKNOWN = 1 << 1;
        const CYCLED = 1 << 2;
        const ALWAYS = 1 << 4;
        const FREE = 1 << 5;
    }
}

pub fn size_materials(array_size: i16, materials: &[Material]) -> u32 {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let mut size = MaterialInfoC::SIZE + (MaterialC::SIZE + 2 + 2) * array_size as u32;
    for material in materials {
        if let Material::Textured(mat) = material {
            if let Some(cycle) = &mat.cycle {
                size += CycleInfoC::SIZE + (cycle.textures.len() as u32) * 4;
            }
        }
    }
    size
}

pub use read_multi::{read_materials, read_materials_zero};
pub use read_single::read_material;
pub use write_multi::{write_materials, write_materials_zero};
pub use write_single::write_material;
