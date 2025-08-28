mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::mesh::MeshMw;
use mech3ax_types::{impl_as_bytes, AsBytes as _, Hex, Ptr};
pub(crate) use read::{assert_mesh_info_zero, read_mesh_data, read_mesh_info};
pub(crate) use write::{size_mesh, write_mesh_data, write_mesh_info};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
pub(crate) struct MeshMwC {
    file_ptr: u32,      // 00
    unk04: u32,         // 04
    unk08: u32,         // 08
    parent_count: u32,  // 12
    polygon_count: u32, // 16
    vertex_count: u32,  // 20
    normal_count: u32,  // 24
    morph_count: u32,   // 28
    light_count: u32,   // 32
    zero36: u32,        // 36
    unk40: f32,         // 40
    unk44: f32,         // 44
    zero48: u32,        // 48
    polygons_ptr: Ptr,  // 52
    vertices_ptr: Ptr,  // 56
    normals_ptr: Ptr,   // 60
    lights_ptr: Ptr,    // 64
    morphs_ptr: Ptr,    // 68
    unk72: f32,         // 72
    unk76: f32,         // 76
    unk80: f32,         // 80
    unk84: f32,         // 84
    zero88: u32,        // 88
}
impl_as_bytes!(MeshMwC, 92);
pub(crate) const MESH_C_SIZE: u32 = MeshMwC::SIZE;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PolygonMwC {
    vertex_info: Hex<u32>, // 00
    unk04: i32,            // 04
    vertices_ptr: Ptr,     // 08
    normals_ptr: Ptr,      // 12
    uvs_ptr: Ptr,          // 16
    colors_ptr: Ptr,       // 20
    unk_ptr: Ptr,          // 24
    material_index: u32,   // 28
    texture_info: u32,     // 32
}
impl_as_bytes!(PolygonMwC, 36);

pub(crate) struct WrappedMeshMw {
    pub(crate) mesh: MeshMw,
    pub(crate) polygon_count: u32,
    pub(crate) vertex_count: u32,
    pub(crate) normal_count: u32,
    pub(crate) morph_count: u32,
    pub(crate) light_count: u32,
}
