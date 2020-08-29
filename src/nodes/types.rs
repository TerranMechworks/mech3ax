use super::flags::NodeBitFlags;
use crate::types::{Matrix, Vec3};
use serde::{Deserialize, Serialize};

pub const ZONE_DEFAULT: u32 = 255;

pub type NodeFlags = super::flags::NodeFlags;
pub type AreaPartition = Option<(i32, i32)>;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[repr(u32)]
pub enum NodeType {
    EMPTY = 0,
    CAMERA = 1,
    WORLD = 2,
    WINDOW = 3,
    DISPLAY = 4,
    OBJECT3D = 5,
    LOD = 6,
    LIGHT = 9,
}

pub struct NodeVariants {
    pub name: String,
    pub flags: NodeBitFlags,
    pub unk044: u32,
    pub zone_id: u32,
    pub data_ptr: u32,
    pub mesh_index: i32,
    pub area_partition: AreaPartition,
    pub has_parent: bool,
    pub parent_array_ptr: u32,
    pub children_count: u32,
    pub children_array_ptr: u32,
    pub unk116: (f32, f32, f32, f32, f32, f32),
    pub unk140: (f32, f32, f32, f32, f32, f32),
    pub unk164: (f32, f32, f32, f32, f32, f32),
    pub unk196: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transformation {
    pub rotation: Vec3,
    pub translation: Vec3,
    pub matrix: Option<Matrix>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Object3d {
    pub name: String,
    pub flags: NodeFlags,
    pub zone_id: u32,
    pub area_partition: AreaPartition,
    pub transformation: Option<Transformation>,
    pub matrix_signs: u32,
    pub mesh_index: i32,
    pub parent: Option<u32>,
    pub children: Option<Vec<Node>>,
    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
    pub unk116: (f32, f32, f32, f32, f32, f32),
    pub unk140: (f32, f32, f32, f32, f32, f32),
    pub unk164: (f32, f32, f32, f32, f32, f32),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Node {
    Object3d(Object3d),
}
