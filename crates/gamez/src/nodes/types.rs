use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::nodes::{
    ActiveBoundingBox, BoundingBox, NodeData, NodeFlags, Partition,
};
use mech3ax_api_types::{Count, IndexO};
use mech3ax_types::{Offsets, Ptr, impl_as_bytes, primitive_enum};

pub(crate) struct NodeInfo {
    pub(crate) name: String,
    pub(crate) flags: NodeFlags,
    pub(crate) update_flags: u32,
    pub(crate) zone_id: i8,
    pub(crate) data_ptr: Ptr,
    pub(crate) model_index: IndexO,
    pub(crate) area_partition: Option<Partition>,
    pub(crate) virtual_partition: Option<Partition>,
    pub(crate) parent_count: Count,
    pub(crate) parent_array_ptr: Ptr,
    pub(crate) child_count: Count,
    pub(crate) child_array_ptr: Ptr,
    pub(crate) active_bbox: ActiveBoundingBox,
    pub(crate) node_bbox: BoundingBox,
    pub(crate) model_bbox: BoundingBox,
    pub(crate) child_bbox: BoundingBox,
    pub(crate) field192: i32,
    pub(crate) field196: i32,
    pub(crate) field200: i32,
    pub(crate) field204: i32,
    pub(crate) node_class: NodeClass,
    pub(crate) offset: u32,
}

pub(crate) const ZONE_ALWAYS: i8 = -1;

primitive_enum! {
    pub(crate) enum NodeClass: u32 {
        Empty = 0,
        Camera = 1,
        World = 2,
        Window = 3,
        Display = 4,
        Object3d = 5,
        Lod = 6,
        Light = 9,
    }
}

impl NodeClass {
    pub(crate) fn from_data(data: &NodeData) -> Self {
        match data {
            NodeData::Camera(_) => Self::Camera,
            NodeData::Display(_) => Self::Display,
            NodeData::Empty => Self::Empty,
            NodeData::Light(_) => Self::Light,
            NodeData::Lod(_) => Self::Lod,
            NodeData::Object3d(_) => Self::Object3d,
            NodeData::Window(_) => Self::Window,
            NodeData::World(_) => Self::World,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern, Offsets, Default)]
#[repr(C)]
pub(crate) struct AreaPartitionC {
    pub(crate) x: i32, // 0
    pub(crate) z: i32, // 4
}
impl_as_bytes!(AreaPartitionC, 8);

impl AreaPartitionC {
    pub(crate) const DEFAULT: Self = Self { x: -1, z: -1 };
    pub(crate) const ZERO: Self = Self { x: 0, z: 0 };
}
