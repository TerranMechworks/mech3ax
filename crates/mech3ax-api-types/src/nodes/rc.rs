use super::{Area, AreaPartition, BoundingBox, NodeFlags, Partition};
use crate::types::{Color, Matrix, Range, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{RefStruct, Union};

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct RotationTranslation {
    pub rotation: Vec3,
    pub translation: Vec3,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct TranslationOnly {
    pub translation: Vec3,
    pub matrix: Option<Matrix>,
}

#[derive(Debug, Serialize, Deserialize, Union)]
#[repr(u16)]
pub enum Transformation {
    None,
    ScaleOnly(Vec3),
    RotationTranslation(RotationTranslation),
    TranslationOnly(TranslationOnly),
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Camera {
    pub name: String,
    pub clip: Range,
    pub fov: Range,
    pub focus_node_xy: i32,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Display {
    pub name: String,
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub clear_color: Color,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Empty {
    pub name: String,
    pub flags: NodeFlags,
    pub unk044: u32,
    pub zone_id: u32,
    pub unk116: BoundingBox,
    pub unk140: BoundingBox,
    pub unk164: BoundingBox,
    pub parent: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Light {
    pub name: String,
    pub unk008: f32,
    pub unk012: f32,
    // pub direction: Vec3,
    // pub diffuse: f32,
    // pub ambient: f32,
    pub color: Color,
    pub range: Range,
    pub parent_ptr: u32,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Lod {
    pub name: String,
    pub level: bool,
    pub range: Range,
    pub unk60: f32,
    pub unk76: Option<u32>,
    pub flags: NodeFlags,
    pub zone_id: u32,
    pub parent: Option<u32>,
    pub children: Vec<u32>,
    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
    pub unk116: BoundingBox,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Object3d {
    pub name: String,
    pub transformation: Transformation,
    pub matrix_signs: u32,
    pub flags: NodeFlags,
    pub zone_id: u32,
    pub area_partition: Option<AreaPartition>,
    pub mesh_index: i32,
    pub parent: Option<u32>,
    pub children: Vec<u32>,

    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
    pub unk116: BoundingBox,
    pub unk140: BoundingBox,
    pub unk164: BoundingBox,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Window {
    pub name: String,
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct World {
    pub name: String,
    pub area: Area,
    pub fog_color: Color,
    pub fog_range: Range,
    pub fog_altitude: Range,
    pub partitions: Vec<Vec<Partition>>,
    pub area_partition_unk: u32,
    pub virt_partition_x_count: u32,
    pub virt_partition_y_count: u32,
    pub area_partition_ptr: u32,
    pub virt_partition_ptr: u32,
    pub world_children_ptr: u32,
    pub world_child_value: u32,
    pub world_lights_ptr: u32,
    pub children: Vec<u32>,
    pub data_ptr: u32,
    pub children_array_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum NodeRc {
    Camera(Camera),
    Display(Display),
    Empty(Empty),
    Light(Light),
    Lod(Lod),
    Object3d(Object3d),
    Window(Window),
    World(World),
}
