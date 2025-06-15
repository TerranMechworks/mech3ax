use super::{ModelFlags, ModelType, PointLight, UvCoord};
use crate::Vec3;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct PolygonRc {
    pub vertex_indices: Vec<u32>,
    pub normal_indices: Option<Vec<u32>>,
    pub uv_coords: Option<Vec<UvCoord>>,
    pub material_index: u32,
    pub show_backface: bool,
    pub priority: i32,
    pub zone_set: Vec<i8>,

    pub vertices_ptr: u32,
    pub normals_ptr: u32,
    pub uvs_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct ModelRc {
    pub model_type: ModelType,
    pub flags: ModelFlags,
    pub parent_count: u32,

    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub morphs: Vec<Vec3>,
    pub lights: Vec<PointLight>,
    pub polygons: Vec<PolygonRc>,

    pub texture_scroll: UvCoord,
    pub bbox_mid: Vec3,
    pub bbox_diag: f32,

    pub polygons_ptr: u32,
    pub vertices_ptr: u32,
    pub normals_ptr: u32,
    pub lights_ptr: u32,
    pub morphs_ptr: u32,
}
