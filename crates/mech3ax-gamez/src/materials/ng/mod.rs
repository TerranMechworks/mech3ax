//! GameZ and mechlib material support for MW, PM, CS
mod read_multi;
mod read_single;
mod write_multi;
mod write_single;

use super::{assert_material_info, find_texture_index_by_name, MaterialC, MaterialInfoC};
use mech3ax_api_types::{static_assert_size, ColoredMaterial, Material, ReprSize as _};

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
