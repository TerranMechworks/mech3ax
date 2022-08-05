use super::flags::NodeBitFlags;
use mech3ax_api_types::{AreaPartition, BoundingBox, Empty};
use num_derive::FromPrimitive;

pub const ZONE_DEFAULT: u32 = 255;

#[derive(Debug, PartialEq, FromPrimitive, Copy, Clone)]
#[repr(u32)]
pub enum NodeType {
    Empty = 0,
    Camera = 1,
    World = 2,
    Window = 3,
    Display = 4,
    Object3d = 5,
    LoD = 6,
    Light = 9,
}

pub struct NodeVariants {
    pub name: String,
    pub flags: NodeBitFlags,
    pub unk044: u32,
    pub zone_id: u32,
    pub data_ptr: u32,
    pub mesh_index: i32,
    pub area_partition: Option<AreaPartition>,
    pub has_parent: bool,
    pub parent_array_ptr: u32,
    pub children_count: u32,
    pub children_array_ptr: u32,
    pub unk116: BoundingBox,
    pub unk140: BoundingBox,
    pub unk164: BoundingBox,
    pub unk196: u32,
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
