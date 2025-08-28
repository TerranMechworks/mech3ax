//! GameZ and mechlib mesh support for PM, CS
mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::mesh::{MeshNg, PolygonFlags};
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Hex, Ptr};
pub(crate) use read::{assert_mesh_info, assert_mesh_info_zero, read_mesh_data, read_mesh_info};
pub(crate) use write::{size_mesh, write_mesh_data, write_mesh_info};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
pub(crate) struct MeshNgC {
    file_ptr: u32,                // 00
    unk04: u32,                   // 04
    unk08: u32,                   // 08
    pub(crate) parent_count: u32, // 12
    polygon_count: u32,           // 16
    vertex_count: u32,            // 20
    normal_count: u32,            // 24
    morph_count: u32,             // 28
    light_count: u32,             // 32
    zero36: u32,                  // 36
    unk40: f32,                   // 40
    unk44: f32,                   // 44
    zero48: u32,                  // 48
    polygons_ptr: Ptr,            // 52
    vertices_ptr: Ptr,            // 56
    normals_ptr: Ptr,             // 60
    lights_ptr: Ptr,              // 64
    morphs_ptr: Ptr,              // 68
    unk72: f32,                   // 72
    unk76: f32,                   // 76
    unk80: f32,                   // 80
    unk84: f32,                   // 84
    zero88: u32,                  // 88
    material_count: u32,          // 92
    materials_ptr: Ptr,           // 96
}
impl_as_bytes!(MeshNgC, 100);
pub(crate) const MESH_C_SIZE: u32 = MeshNgC::SIZE;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PolygonNgC {
    vertex_info: Hex<u32>, // 00
    unk04: i32,            // 04
    vertices_ptr: Ptr,     // 08
    normals_ptr: Ptr,      // 12
    mat_count: u32,        // 16
    uvs_ptr: Ptr,          // 20
    colors_ptr: Ptr,       // 24
    unk28: Ptr,            // 28
    unk32: Ptr,            // 32
    unk36: Hex<u32>,       // 36
}
impl_as_bytes!(PolygonNgC, 40);

bitflags! {
    struct PolygonBitFlags: u32 {
        const UNK2 = 1 << 2;
        const UNK3 = 1 << 3; // not in mechlib
        const NORMALS = 1 << 4;
        const TRI_STRIP = 1 << 5;
        const UNK6 = 1 << 6; // not in mechlib
    }
}

impl From<PolygonBitFlags> for PolygonFlags {
    fn from(flags: PolygonBitFlags) -> Self {
        Self {
            unk2: flags.contains(PolygonBitFlags::UNK2),
            unk3: flags.contains(PolygonBitFlags::UNK3),
            triangle_strip: flags.contains(PolygonBitFlags::TRI_STRIP),
            unk6: flags.contains(PolygonBitFlags::UNK6),
        }
    }
}

impl From<&PolygonFlags> for PolygonBitFlags {
    fn from(flags: &PolygonFlags) -> Self {
        let mut bitflags = Self::empty();
        if flags.unk2 {
            bitflags |= PolygonBitFlags::UNK2;
        }
        if flags.unk3 {
            bitflags |= PolygonBitFlags::UNK3;
        }
        if flags.triangle_strip {
            bitflags |= PolygonBitFlags::TRI_STRIP;
        }
        if flags.unk6 {
            bitflags |= PolygonBitFlags::UNK6;
        }
        bitflags
    }
}

pub(crate) struct WrappedMeshNg {
    pub(crate) mesh: MeshNg,
    pub(crate) polygon_count: u32,
    pub(crate) vertex_count: u32,
    pub(crate) normal_count: u32,
    pub(crate) morph_count: u32,
    pub(crate) light_count: u32,
    pub(crate) material_count: u32,
}
