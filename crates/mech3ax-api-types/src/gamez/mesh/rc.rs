use super::{MeshLight, UvCoord};
use crate::Vec3;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::RefStruct;

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct PolygonRc {
    pub vertex_indices: Vec<u32>,
    pub normal_indices: Option<Vec<u32>>,
    pub uv_coords: Option<Vec<UvCoord>>,
    pub material_index: u32,

    pub unk0_flag: bool,
    pub unk04: i32,
    pub unk24: u32,

    pub vertices_ptr: u32,
    pub normals_ptr: u32,
    pub uvs_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct MeshRc {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub morphs: Vec<Vec3>,
    pub lights: Vec<MeshLight>,
    pub polygons: Vec<PolygonRc>,
    pub polygons_ptr: u32,
    pub vertices_ptr: u32,
    pub normals_ptr: u32,
    pub lights_ptr: u32,
    pub morphs_ptr: u32,
    pub file_ptr: bool,
    pub unk04: u32,
    pub parent_count: u32,
    pub unk68: f32,
    pub unk72: f32,
    pub unk76: f32,
    pub unk80: f32,
}
