mod read_multi;
mod read_single;
mod write_multi;
mod write_single;

use mech3ax_api_types::{static_assert_size, Color, ReprSize as _};

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
    index: u32,     // 16
    zero20: f32,    // 20
    half24: f32,    // 24
    half28: f32,    // 28
    specular: f32,  // 32
    cycle_ptr: u32, // 36
}
static_assert_size!(MaterialC, 40);

bitflags::bitflags! {
    struct MaterialFlags: u8 {
        const TEXTURED = 1 << 0;
    }
}

pub fn size_materials(array_size: i16) -> u32 {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    MaterialInfoC::SIZE + (MaterialC::SIZE + 2 + 2) * array_size as u32
}

pub use read_multi::{read_materials, read_materials_zero};
pub use read_single::read_material;
pub use write_multi::{write_materials, write_materials_zero};
pub use write_single::write_material;
