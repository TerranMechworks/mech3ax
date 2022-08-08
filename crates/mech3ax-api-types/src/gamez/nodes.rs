use crate::serde::bool_false;
use crate::static_assert_size;
use crate::types::{Color, Matrix, Range, Vec3};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{RefStruct, Union, ValStruct};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ValStruct)]
#[repr(C)]
pub struct AreaPartition {
    pub x: i32,
    pub y: i32,
}
static_assert_size!(AreaPartition, 8);

impl AreaPartition {
    pub const DEFAULT: Self = Self { x: -1, y: -1 };
    pub const ZERO: Self = Self { x: 0, y: 0 };
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ValStruct)]
pub struct Area {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, RefStruct)]
#[repr(C)]
pub struct BoundingBox {
    pub a: Vec3,
    pub b: Vec3,
}

impl BoundingBox {
    pub const EMPTY: Self = Self {
        a: Vec3::DEFAULT,
        b: Vec3::DEFAULT,
    };
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Transformation {
    pub rotation: Vec3,
    pub translation: Vec3,
    pub matrix: Option<Matrix>,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Partition {
    pub x: i32,
    pub y: i32,
    pub z_min: f32,
    pub z_max: f32,
    pub z_mid: f32,
    pub nodes: Vec<u32>,
    pub ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
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

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Camera {
    pub name: String,
    pub clip: Range,
    pub fov: Range,
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
    pub direction: Vec3,
    pub diffuse: f32,
    pub ambient: f32,
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
    pub area_partition: Option<AreaPartition>,
    pub parent: u32,
    pub children: Vec<u32>,
    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
    pub unk116: BoundingBox,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Object3d {
    pub name: String,
    pub transformation: Option<Transformation>,
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

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum Node {
    Camera(Camera),
    Display(Display),
    Empty(Empty),
    Light(Light),
    Lod(Lod),
    Object3d(Object3d),
    Window(Window),
    World(World),
}
