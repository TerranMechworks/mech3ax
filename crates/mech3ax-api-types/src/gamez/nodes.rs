use crate::serde::bool_false;
use crate::types::{Matrix, Vec2, Vec3};
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AreaPartition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Area {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Block(pub f32, pub f32, pub f32, pub f32, pub f32, pub f32);

impl Block {
    pub const EMPTY: Block = Block(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeFlags {
    #[serde(skip_serializing_if = "bool_false", default)]
    pub altitude_surface: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub intersect_surface: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub intersect_bbox: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub landmark: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk08: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub has_mesh: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk10: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub terrain: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub can_modify: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub clip_to: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk25: bool,
    #[serde(skip_serializing_if = "bool_false", default)]
    pub unk28: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Camera {
    pub name: String,
    pub clip: Vec2,
    pub fov: Vec2,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Display {
    pub name: String,
    pub resolution: (u32, u32),
    pub clear_color: Vec3,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Empty {
    pub name: String,
    pub flags: NodeFlags,
    pub unk044: u32,
    pub zone_id: u32,
    pub unk116: Block,
    pub unk140: Block,
    pub unk164: Block,
    pub parent: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Light {
    pub name: String,
    pub direction: Vec3,
    pub diffuse: f32,
    pub ambient: f32,
    pub color: Vec3,
    pub range: Vec2,
    pub parent_ptr: u32,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lod {
    pub name: String,

    pub level: bool,
    pub range: Vec2,
    pub unk60: f32,
    pub unk76: Option<u32>,

    pub flags: NodeFlags,
    pub zone_id: u32,
    pub area_partition: Option<AreaPartition>,
    pub parent: u32,
    pub children: Vec<u32>,
    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
    pub unk116: Block,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transformation {
    pub rotation: Vec3,
    pub translation: Vec3,
    pub matrix: Option<Matrix>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Object3d<T> {
    pub name: String,
    pub transformation: Option<Transformation>,
    pub matrix_signs: u32,

    pub flags: NodeFlags,
    pub zone_id: u32,
    pub area_partition: Option<AreaPartition>,
    pub mesh_index: i32,
    pub parent: Option<u32>,
    pub children: Vec<T>,

    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
    pub unk116: Block,
    pub unk140: Block,
    pub unk164: Block,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Window {
    pub name: String,
    pub resolution: (u32, u32),
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Partition {
    pub x: i32,
    pub y: i32,
    pub z_min: f32,
    pub z_max: f32,
    pub z_mid: f32,
    pub nodes: Vec<u32>,
    pub ptr: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    pub name: String,
    pub area: Area,
    pub partitions: Vec<Vec<Partition>>,
    pub area_partition_x_count: u32,
    pub area_partition_y_count: u32,
    pub fudge_count: bool,
    pub area_partition_ptr: u32,
    pub virt_partition_ptr: u32,
    pub world_children_ptr: u32,
    pub world_child_value: u32,
    pub world_lights_ptr: u32,

    pub children: Vec<u32>,
    pub data_ptr: u32,
    pub children_array_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Node<T> {
    Camera(Camera),
    Display(Display),
    Empty(Empty),
    Light(Light),
    Lod(Lod),
    Object3d(Object3d<T>),
    Window(Window),
    World(World),
}
