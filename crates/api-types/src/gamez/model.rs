use crate::serde::bool_false;
use crate::{Color, Vec3};
use ::serde::{Deserialize, Serialize};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_metadata_proc_macro::{Enum, Struct};
use mech3ax_types::{impl_as_bytes, primitive_enum};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    NoUninit,
    AnyBitPattern,
    Struct,
)]
#[dotnet(val_struct)]
#[repr(C)]
pub struct UvCoord {
    pub u: f32,
    pub v: f32,
}
impl_as_bytes!(UvCoord, 8);

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct PointLight {
    pub unk00: u32,
    pub unk04: u32,
    pub unk08: f32,
    pub extra: Vec<Vec3>,
    pub unk24: u32,
    pub color: Color,
    pub flags: u16,
    pub ptr: u32,
    pub unk48: f32,
    pub unk52: f32,
    pub unk56: f32,
    pub unk60: u32,
    pub unk64: f32,
    pub unk68: f32,
    pub unk72: f32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct PolygonFlags {
    pub show_backface: bool,  // RC, MW, PM
    pub unk3: bool,           // MW, PM
    pub triangle_strip: bool, // PM
    pub unk6: bool,           // PM
}

// TODO
#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct PolygonMaterialNg {
    pub material_index: u32,
    pub uv_coords: Vec<UvCoord>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Polygon {
    pub vertex_indices: Vec<u32>,
    pub normal_indices: Option<Vec<u32>>,
    pub uv_coords: Option<Vec<UvCoord>>,
    pub vertex_colors: Vec<Color>,
    pub material_index: u32,
    pub flags: PolygonFlags,
    pub priority: i32,
    pub zone_set: Vec<i8>,

    pub vertex_indices_ptr: u32, // RC, MW, PM
    pub normal_indices_ptr: u32, // RC, MW, PM
    pub uvs_ptr: u32,            // RC, MW, PM
    pub vertex_colors_ptr: u32,  // MW, PM
    pub unk_ptr: u32,            // MW, PM (matl info)
    pub materials_ptr: u32,      // PM

    // TODO
    pub materials: Vec<PolygonMaterialNg>,
}

primitive_enum! {
    #[derive(Serialize, Deserialize, Enum)]
    pub enum ModelType: u32 {
        Default = 0,
        Facade = 1,
        Points = 2,
    }
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct ModelFlags {
    pub lighting: bool,
    pub fog: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub texture_registered: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub morph: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub texture_scroll: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub facade_tilt: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub clouds: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk7: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk8: bool,
}

// TODO
#[derive(Debug, Clone, Copy, Serialize, Deserialize, NoUninit, AnyBitPattern, Struct)]
#[repr(C)]
pub struct MeshMaterialInfo {
    pub material_index: u32, // 00
    // polygon offset?
    pub polygon_usage_count: u32, // 04
    // polygons ptr
    pub unk_ptr: u32, // 08
}
impl_as_bytes!(MeshMaterialInfo, 12);

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Model {
    pub model_type: ModelType,
    pub flags: ModelFlags,
    pub parent_count: u32,
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub morphs: Vec<Vec3>,
    pub lights: Vec<PointLight>,
    pub polygons: Vec<Polygon>,
    pub texture_scroll: UvCoord,
    pub bbox_mid: Vec3,
    pub bbox_diag: f32,

    pub polygons_ptr: u32,
    pub vertices_ptr: u32,
    pub normals_ptr: u32,
    pub lights_ptr: u32,
    pub morphs_ptr: u32,
    pub materials_ptr: u32,

    // TODO
    pub material_infos: Vec<MeshMaterialInfo>,
}
