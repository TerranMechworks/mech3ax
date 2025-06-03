//! GameZ and mechlib material support.
mod read_multi;
mod read_single;
mod write_multi;
mod write_single;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::materials::{ColoredMaterial, Material, Soil};
use mech3ax_api_types::Color;
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Bool32, Maybe};
pub(crate) use read_multi::read_materials;
pub(crate) use read_single::read_material;
pub(crate) use write_multi::write_materials;
pub(crate) use write_single::write_material;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MatType {
    /// Recoil
    Rc,
    /// MechWarrior3, Pirate's Moon, Crimson Skies
    Ng,
}

impl MatType {
    pub(crate) fn size_i16(&self) -> i16 {
        match self {
            Self::Rc => 5000,
            Self::Ng => 1000,
        }
    }

    pub(crate) fn size_i32(&self) -> i32 {
        match self {
            Self::Rc => 5000,
            Self::Ng => 1000,
        }
    }

    pub(crate) fn size_u32(&self) -> u32 {
        match self {
            Self::Rc => 5000,
            Self::Ng => 1000,
        }
    }
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct MaterialInfoC {
    array_size: i32,
    count: i32,
    index_max: i32,
    index_last: i32,
}
impl_as_bytes!(MaterialInfoC, 16);

bitflags! {
    struct MaterialFlags: u8 {
        const TEXTURED = 1 << 0;    // 0x01
        const UNKNOWN = 1 << 1;     // 0x02
        const CYCLED = 1 << 2;      // 0x04
        const ALWAYS = 1 << 4;      // 0x08
        const FREE = 1 << 5;        // 0x10
    }
}

type Flags = Maybe<u8, MaterialFlags>;
type MSoil = Maybe<u32, Soil>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct MaterialC {
    alpha: u8,      // 00
    flags: Flags,   // 01
    rgb: u16,       // 02
    color: Color,   // 04
    index: u32,     // 16, ptr in mechlib, texture index in gamez
    zero20: f32,    // 20
    half24: f32,    // 24
    half28: f32,    // 28
    soil: MSoil,    // 32
    cycle_ptr: u32, // 36
}
impl_as_bytes!(MaterialC, 40);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct CycleInfoC {
    looping: Bool32,    // 00
    current_frame: i32, // 04
    current_index: f32, // 08
    speed: f32,         // 12
    tex_map_count: u32, // 16
    tex_map_index: u32, // 20
    tex_map_ptr: u32,   // 24
}
impl_as_bytes!(CycleInfoC, 28);

#[derive(Debug)]
pub(crate) struct RawTexturedMaterial {
    pub(crate) pointer: u32,
    pub(crate) cycle_ptr: u32,
    pub(crate) soil: Soil,
    pub(crate) flag: bool,
}

#[derive(Debug)]
pub(crate) enum RawMaterial {
    Textured(RawTexturedMaterial),
    Colored(ColoredMaterial),
}

pub(crate) fn size_materials(materials: &[Material], ty: MatType) -> u32 {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let mut size = MaterialInfoC::SIZE + (MaterialC::SIZE + 2 + 2) * ty.size_u32();
    for material in materials {
        if let Material::Textured(mat) = material {
            if let Some(cycle) = &mat.cycle {
                size += CycleInfoC::SIZE + (cycle.textures.len() as u32) * 4;
            }
        }
    }
    size
}
