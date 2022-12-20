use super::pm::AreaPartitionPm;
use super::{Area, BoundingBox, Display, Transformation};
use crate::static_assert_size;
use crate::types::Range;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{RefStruct, Union};

#[derive(Debug, Serialize, Deserialize, RefStruct, Clone, PartialEq)]
#[repr(C)]
pub struct PartitionValueCs {
    pub index: u32,
    pub z_min: f32,
    pub z_max: f32,
}
static_assert_size!(PartitionValueCs, 12);

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct PartitionCs {
    pub x: i32,
    pub y: i32,
    pub z_min: f32,
    pub z_max: f32,
    pub nodes: Vec<PartitionValueCs>,
    pub ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Camera {
    pub name: String,
    pub focus_node_xy: i32,
    pub data_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Light {
    pub name: String,
    pub range: Range,
    pub parent_ptr: u32,
    pub data_ptr: u32,
    pub node_index: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Lod {
    pub name: String,
    pub level: bool,
    pub range: Range,
    pub unk64: f32,
    pub unk72: f32,
    pub parent: u32,
    pub children: Vec<u32>,
    pub flags_unk03: bool,
    pub flags_unk04: bool,
    pub flags_unk07: bool,
    pub unk040: u32,
    pub zone_id: u32,
    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
    pub unk164: BoundingBox,
    pub node_index: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Object3d {
    pub name: String,
    pub transformation: Option<Transformation>,
    pub matrix_signs: u32,
    pub rotation_signs: u32,
    // pub flags: NodeFlags,
    pub flags: u32,
    pub unk040: u32,
    pub unk044: u32,
    pub zone_id: u32,
    pub data_ptr: u32,
    pub mesh_index: i32,
    pub area_partition: Option<AreaPartitionPm>,
    pub parent: Option<u32>,
    pub parent_array_ptr: u32,
    pub children: Vec<u32>,
    pub children_array_ptr: u32,
    pub unk112: u32,
    pub unk116: BoundingBox,
    pub unk140: BoundingBox,
    pub unk164: BoundingBox,
    pub node_index: u32,
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
    pub partitions: Vec<Vec<PartitionCs>>,
    pub flags: bool,
    pub area_partition_count: u32,
    pub area_partition_ptr: u32,
    pub virt_partition_x_min: u32,
    pub virt_partition_y_min: u32,
    pub virt_partition_ptr: u32,
    pub world_children_ptr: u32,
    pub world_child_value: u32,
    pub world_lights_ptr: u32,
    pub children: Vec<u32>,
    pub data_ptr: u32,
    pub children_array_ptr: u32,
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum NodeCs {
    Camera(Camera),
    Display(Display),
    Light(Light),
    Lod(Lod),
    Object3d(Object3d),
    Window(Window),
    World(World),
}
