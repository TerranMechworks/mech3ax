use super::flags::NodeBitFlags;
use crate::types::{Matrix, Vec2, Vec3};
use serde::{Deserialize, Serialize};

pub const ZONE_DEFAULT: u32 = 255;

pub type NodeFlags = super::flags::NodeFlags;
pub type AreaPartition = Option<(i32, i32)>;
pub type Block = (f32, f32, f32, f32, f32, f32);
pub type Area = (i32, i32, i32, i32);

pub const BLOCK_EMPTY: Block = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

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
    pub unk116: Block,
    pub unk140: Block,
    pub unk164: Block,
    pub unk196: u32,
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
    pub area_partition: AreaPartition,
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
    pub area_partition: AreaPartition,
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
    pub nodes: Vec<u32>,
    pub unk: Vec3,
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

pub enum NodeVariant {
    Camera(u32),
    Display(u32),
    Empty(Empty),
    Light(u32),
    Lod(NodeVariants),
    Object3d(NodeVariants),
    Window(u32),
    World(u32, u32, u32),
}
