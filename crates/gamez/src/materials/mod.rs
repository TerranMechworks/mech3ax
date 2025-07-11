//! GameZ and mechlib material support.
mod read_multi;
mod read_single;
mod write_multi;
mod write_single;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::materials::{ColoredMaterial, Material, Soil};
use mech3ax_api_types::{Color, Count, Count32, IndexR, IndexR32};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::Result;
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Bool32, Maybe, Offsets, Ptr};
pub(super) use read_single::read_material_mechlib;
use std::io::{Read, Write};
pub(super) use write_single::write_material_mechlib;

#[derive(Debug, Clone, Copy)]
enum MatlType {
    /// Recoil
    Rc,
    /// MechWarrior3, Pirate's Moon
    Ng,
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct MaterialArrayC {
    array_size: Count32,
    count: Count32,
    index_free: i32,
    index_last: i32,
}
impl_as_bytes!(MaterialArrayC, 16);

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

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct MaterialC {
    alpha: u8,               // 00
    flags: Flags,            // 01
    rgb: u16,                // 02
    color: Color,            // 04
    texture_index: IndexR32, // 16
    field20: f32,            // 20
    field24: f32,            // 24
    field28: f32,            // 28
    soil: MSoil,             // 32
    cycle_ptr: Ptr,          // 36
}
impl_as_bytes!(MaterialC, 40);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct MaterialCycleC {
    looping: Bool32,        // 00
    current_frame: i32,     // 04
    current_index: f32,     // 08
    speed: f32,             // 12
    tex_map_count: Count32, // 16
    tex_map_index: Count32, // 20
    tex_map_ptr: Ptr,       // 24
}
impl_as_bytes!(MaterialCycleC, 28);

#[derive(Debug)]
struct RawTexturedMaterial {
    texture_index: IndexR,
    soil: Soil,
    flag: bool,
    cycle_ptr: Ptr,
}

#[derive(Debug)]
enum RawMaterial {
    Textured(RawTexturedMaterial),
    Colored(ColoredMaterial),
}

pub(crate) fn read_materials_ng(
    read: &mut CountingReader<impl Read>,
    texture_count: Count,
) -> Result<(Vec<Material>, Count, Count)> {
    read_multi::read_materials(read, texture_count, MatlType::Ng)
}

pub(crate) fn read_materials_rc(
    read: &mut CountingReader<impl Read>,
    texture_count: Count,
) -> Result<(Vec<Material>, Count, Count)> {
    read_multi::read_materials(read, texture_count, MatlType::Rc)
}

pub(crate) fn write_materials_ng(
    write: &mut CountingWriter<impl Write>,
    materials: &[Material],
    array_size: Count,
    texture_count: Count,
) -> Result<()> {
    write_multi::write_materials(write, materials, array_size, texture_count, MatlType::Ng)
}

pub(crate) fn write_materials_rc(
    write: &mut CountingWriter<impl Write>,
    materials: &[Material],
    array_size: Count,
    texture_count: Count,
) -> Result<()> {
    write_multi::write_materials(write, materials, array_size, texture_count, MatlType::Rc)
}

pub(crate) fn size_materials(materials: &[Material], array_size: Count) -> u32 {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let mut size = MaterialArrayC::SIZE + (MaterialC::SIZE + 2 + 2) * array_size.to_u32();
    for material in materials {
        if let Material::Textured(mat) = material {
            if let Some(cycle) = &mat.cycle {
                size += MaterialCycleC::SIZE + (cycle.texture_indices.len() as u32) * 4;
            }
        }
    }
    size
}
