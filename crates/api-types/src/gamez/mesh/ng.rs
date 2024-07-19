use super::{MeshLight, UvCoord};
use crate::impl_as_bytes;
use crate::serde::bool_false;
use crate::{Color, Vec3};
use ::serde::{Deserialize, Serialize};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_metadata_proc_macro::Struct;

#[derive(Debug, Serialize, Deserialize, Struct)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, NoUninit, AnyBitPattern, Struct)]
#[repr(C)]
pub struct MeshMaterialInfo {
    pub material_index: u32,      // 00
    pub polygon_usage_count: u32, // 04
    pub unk_ptr: u32,             // 08
}
impl_as_bytes!(MeshMaterialInfo, 12);

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct PolygonMaterialNg {
    pub material_index: u32,
    pub uv_coords: Vec<UvCoord>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct PolygonNg {
    pub flags: PolygonFlags,
    pub vertex_indices: Vec<u32>,
    pub vertex_colors: Vec<Color>,
    pub normal_indices: Option<Vec<u32>>,
    pub materials: Vec<PolygonMaterialNg>,

    pub unk04: i32,
    pub vertices_ptr: u32,
    pub normals_ptr: u32,
    pub uvs_ptr: u32,
    pub colors_ptr: u32,
    pub unk28: u32,
    pub unk32: u32,
    pub unk36: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct MeshNg {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub morphs: Vec<Vec3>,
    pub lights: Vec<MeshLight>,
    pub polygons: Vec<PolygonNg>,
    pub material_infos: Vec<MeshMaterialInfo>,
    pub polygons_ptr: u32,
    pub vertices_ptr: u32,
    pub normals_ptr: u32,
    pub lights_ptr: u32,
    pub morphs_ptr: u32,
    pub materials_ptr: u32,
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
