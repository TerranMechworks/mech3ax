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

#[derive(Debug, Clone, Serialize, Deserialize, Struct)]
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Struct)]
pub struct PolygonFlags {
    pub show_backface: bool, // RC, MW, PM
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk3: bool, // MW, PM
    #[serde(skip_serializing_if = "bool_false", default)]
    pub triangle_strip: bool, // MW, PM
    #[serde(skip_serializing_if = "bool_false", default)]
    pub in_out: bool, // PM
}

#[derive(Debug, Clone, Serialize, Deserialize, Struct)]
pub struct PolygonMaterial {
    pub material_index: u32,
    pub uv_coords: Option<Vec<UvCoord>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Struct)]
pub struct Polygon {
    pub flags: PolygonFlags,
    pub priority: i32,
    pub zone_set: Vec<i8>,
    pub vertex_indices: Vec<u32>,
    pub normal_indices: Option<Vec<u32>>,
    pub vertex_colors: Vec<Color>,
    pub materials: Vec<PolygonMaterial>,

    pub vertex_indices_ptr: u32, // RC, MW, PM
    pub normal_indices_ptr: u32, // RC, MW, PM
    pub uvs_ptr: u32,            // RC, MW, PM
    pub vertex_colors_ptr: u32,  // MW, PM
    pub matl_refs_ptr: u32,      // MW, PM
    pub materials_ptr: u32,      // PM
}

primitive_enum! {
    #[derive(Serialize, Deserialize, Enum)]
    pub enum ModelType: u32 {
        Default = 0,
        Facade = 1,
        Points = 2,
    }
}

primitive_enum! {
    #[derive(Serialize, Deserialize, Enum)]
    pub enum FacadeMode: u32 {
        CylindricalY = 0,
        SphericalY = 1,
        CylindricalX = 2,
        CylindricalZ = 3,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Struct)]
pub struct ModelFlags {
    /// Affected by lighting
    pub lighting: bool,
    /// Affected by fog
    pub fog: bool,
    /// Textures registered to world (?)
    #[serde(skip_serializing_if = "bool_false", default)]
    pub texture_registered: bool,
    /// Morph active (do not set)
    #[serde(skip_serializing_if = "bool_false", default)]
    pub morph: bool,
    /// Scroll active
    #[serde(skip_serializing_if = "bool_false", default)]
    pub texture_scroll: bool,
    /// Affected by clouds/cloud casting
    pub clouds: bool,
    /// Facade rotates around centroid
    pub facade_centroid: bool,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Model {
    pub model_type: ModelType,
    pub facade_mode: FacadeMode,
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
    pub material_refs_ptr: u32,
}
