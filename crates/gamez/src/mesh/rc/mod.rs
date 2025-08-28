mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::mesh::MeshRc;
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Hex, Ptr};
pub(crate) use read::{assert_mesh_info_zero, read_mesh_data, read_mesh_info};
pub(crate) use write::{size_mesh, write_mesh_data, write_mesh_info};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
pub(crate) struct MeshRcC {
    file_ptr: u32,      // 00
    unk04: u32,         // 04
    parent_count: u32,  // 08
    polygon_count: u32, // 12
    vertex_count: u32,  // 16
    normal_count: u32,  // 20
    morph_count: u32,   // 24
    light_count: u32,   // 28
    zero32: u32,        // 32
    zero36: u32,        // 36
    zero40: u32,        // 40
    zero44: u32,        // 44
    polygons_ptr: Ptr,  // 48
    vertices_ptr: Ptr,  // 52
    normals_ptr: Ptr,   // 56
    lights_ptr: Ptr,    // 60
    morphs_ptr: Ptr,    // 64
    unk68: f32,         // 68
    unk72: f32,         // 72
    unk76: f32,         // 76
    unk80: f32,         // 80
}
impl_as_bytes!(MeshRcC, 84);
pub(crate) const MESH_C_SIZE: u32 = MeshRcC::SIZE;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PolygonRcC {
    vertex_info: Hex<u32>, // 00
    unk04: i32,            // 04
    vertices_ptr: Ptr,     // 08
    normals_ptr: Ptr,      // 12
    uvs_ptr: Ptr,          // 16
    material_index: u32,   // 28
    unk24: Hex<u32>,       // 24
}
impl_as_bytes!(PolygonRcC, 28);

bitflags! {
    struct PolygonBitFlags: u32 {
        const UNK0 = 1 << 0;
        const NORMALS = 1 << 1;
    }
}
pub(crate) struct WrappedMeshRc {
    pub(crate) mesh: MeshRc,
    pub(crate) polygon_count: u32,
    pub(crate) vertex_count: u32,
    pub(crate) normal_count: u32,
    pub(crate) morph_count: u32,
    pub(crate) light_count: u32,
}
