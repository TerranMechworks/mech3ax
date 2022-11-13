use super::{MeshLight, UvCoord};
use crate::serde::bool_false;
use crate::static_assert_size;
use crate::types::{Color, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::RefStruct;

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct PolygonFlags {
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk2: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk3: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub triangle_strip: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk6: bool,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
#[repr(C)]
pub struct MeshTexture {
    pub texture_index: u32,       // 00
    pub polygon_usage_count: u32, // 04
    pub unk_ptr: u32,             // 08
}
static_assert_size!(MeshTexture, 12);

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct PolygonTextureNg {
    pub texture_index: u32,
    pub uv_coords: Vec<UvCoord>,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct PolygonNg {
    pub flags: PolygonFlags,
    pub vertex_indices: Vec<u32>,
    pub vertex_colors: Vec<Color>,
    pub normal_indices: Option<Vec<u32>>,
    pub textures: Vec<PolygonTextureNg>,

    pub unk04: i32,
    pub vertices_ptr: u32,
    pub normals_ptr: u32,
    pub uvs_ptr: u32,
    pub colors_ptr: u32,
    pub unk28: u32,
    pub unk32: u32,
    pub unk36: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct MeshNg {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub morphs: Vec<Vec3>,
    pub lights: Vec<MeshLight>,
    pub polygons: Vec<PolygonNg>,
    pub textures: Vec<MeshTexture>,
    pub polygons_ptr: u32,
    pub vertices_ptr: u32,
    pub normals_ptr: u32,
    pub lights_ptr: u32,
    pub morphs_ptr: u32,
    pub texture_ptr: u32,
    pub file_ptr: bool,
    pub unk04: u32,
    pub unk08: u32,
    pub parent_count: u32,
    pub unk40: f32,
    pub unk44: f32,
    pub unk72: f32,
    pub unk76: f32,
    pub unk80: f32,
    pub unk84: f32,
}
