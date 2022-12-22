use super::{Area, BoundingBox, Camera, Display, NodeFlags, PartitionNg, Transformation, Window};
use crate::static_assert_size;
use crate::types::Range;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{RefStruct, Union};

#[derive(Debug, Serialize, Deserialize, RefStruct, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct AreaPartitionPm {
    pub x: i16,
    pub y: i16,
    pub virtual_x: i16,
    pub virtual_y: i16,
}
static_assert_size!(AreaPartitionPm, 8);

impl AreaPartitionPm {
    pub const ZERO: AreaPartitionPm = AreaPartitionPm {
        x: 0,
        y: 0,
        virtual_x: 0,
        virtual_y: 0,
    };
    pub const DEFAULT: AreaPartitionPm = AreaPartitionPm {
        x: -1,
        y: -1,
        virtual_x: 0,
        virtual_y: 0,
    };
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Light {
    pub name: String,
    pub unk004: f32,
    pub unk156: f32,
    pub unk160: f32,
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
    pub zone_id: u32,
    pub parent: u32,
    pub children: Vec<u32>,
    pub unk164: BoundingBox,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
    pub data_ptr: u32,
    pub node_index: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Object3d {
    pub name: String,
    pub transformation: Option<Transformation>,
    pub matrix_signs: u32,
    pub flags: NodeFlags,
    pub zone_id: u32,
    pub area_partition: Option<AreaPartitionPm>,
    pub mesh_index: i32,
    pub parent: Option<u32>,
    pub children: Vec<u32>,

    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_array_ptr: u32,
    pub unk044: u32,
    pub unk112: u32,
    pub unk116: BoundingBox,
    pub unk140: BoundingBox,
    pub unk164: BoundingBox,
    pub node_index: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct World {
    pub name: String,
    pub area: Area,
    pub partitions: Vec<Vec<PartitionNg>>,
    pub virtual_partition: bool,
    pub area_partition_count: u32,
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
pub enum NodePm {
    Camera(Camera),
    Display(Display),
    Light(Light),
    Lod(Lod),
    Object3d(Object3d),
    Window(Window),
    World(World),
}
