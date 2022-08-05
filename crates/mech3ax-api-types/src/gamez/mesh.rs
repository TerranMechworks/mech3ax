use crate::types::{Vec2, Vec3};
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Polygon {
    pub vertex_indices: Vec<u32>,
    pub vertex_colors: Vec<Vec3>,
    pub normal_indices: Option<Vec<u32>>,
    pub uv_coords: Option<Vec<Vec2>>,
    pub texture_index: u32,
    pub texture_info: u32,
    pub unk04: u32,
    pub unk_bit: bool,
    pub vtx_bit: bool,
    pub vertices_ptr: u32,
    pub normals_ptr: u32,
    pub uvs_ptr: u32,
    pub colors_ptr: u32,
    pub unk_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshLight {
    pub unk00: u32,
    pub unk04: u32,
    pub unk08: u32,
    pub extra: Vec<Vec3>,
    pub unk16: u32,
    pub unk20: u32,
    pub unk24: u32,
    pub unk28: f32,
    pub unk32: f32,
    pub unk36: f32,
    pub unk40: f32,
    pub ptr: u32,
    pub unk48: f32,
    pub unk52: f32,
    pub unk56: f32,
    pub unk60: f32,
    pub unk64: f32,
    pub unk68: f32,
    pub unk72: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub morphs: Vec<Vec3>,
    pub lights: Vec<MeshLight>,
    pub polygons: Vec<Polygon>,
    pub polygons_ptr: u32,
    pub vertices_ptr: u32,
    pub normals_ptr: u32,
    pub lights_ptr: u32,
    pub morphs_ptr: u32,
    pub file_ptr: bool,
    pub unk04: bool,
    pub unk08: u32,
    pub parent_count: u32,
    pub unk40: f32,
    pub unk44: f32,
    pub unk72: f32,
    pub unk76: f32,
    pub unk80: f32,
    pub unk84: f32,
}