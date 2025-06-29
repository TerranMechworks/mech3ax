use super::{Area, AreaPartition, BoundingBox, Camera, Display, NodeFlags, PartitionPg, Window};
use crate::{sum, Color, Matrix, Range, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct RotationTranslation {
    pub rotation: Vec3,
    pub translation: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct TranslationOnly {
    pub translation: Vec3,
    pub matrix: Option<Matrix>,
}

sum! {
    enum Transformation {
        None,
        ScaleOnly(Vec3),
        RotationTranslation(RotationTranslation),
        TranslationOnly(TranslationOnly),
    }
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Empty {
    pub name: String,
    pub flags: NodeFlags,
    pub unk044: u32,
    pub zone_id: u32, // TODO: i8
    pub node_bbox: BoundingBox,
    pub model_bbox: BoundingBox,
    pub child_bbox: BoundingBox,
    pub parent: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
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

#[derive(Debug, Serialize, Deserialize, Struct)]
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

    pub node_bbox: BoundingBox,
    pub child_bbox: BoundingBox,

    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Object3d {
    pub name: String,
    pub transformation: Transformation,
    pub matrix_signs: u32,
    pub flags: NodeFlags,
    pub zone_id: u32,
    pub area_partition: Option<AreaPartition>,
    pub model_index: i32,
    pub parent: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub parents: Option<Vec<u32>>,
    pub children: Vec<u32>,

    pub node_bbox: BoundingBox,
    pub model_bbox: BoundingBox,
    pub child_bbox: BoundingBox,

    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct World {
    pub name: String,
    pub area: Area,
    pub fog_color: Color,
    pub fog_range: Range,
    pub fog_altitude: Range,
    pub partitions: Vec<Vec<PartitionPg>>,
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

sum! {
    enum NodeRc {
        Camera(Camera),
        Display(Display),
        Empty(Empty),
        Light(Light),
        Lod(Lod),
        Object3d(Object3d),
        Window(Window),
        World(World),
    }
}
