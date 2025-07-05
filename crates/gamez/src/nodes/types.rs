use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::nodes::{
    ActiveBoundingBox, AreaPartition, BoundingBox, NodeData, NodeFlags,
};
use mech3ax_types::{impl_as_bytes, primitive_enum, Offsets, Ptr};

pub(crate) struct NodeInfo {
    pub(crate) name: String,
    pub(crate) flags: NodeFlags,
    pub(crate) update_flags: u32,
    pub(crate) zone_id: i8,
    pub(crate) data_ptr: Ptr,
    pub(crate) model_index: Option<u16>,
    pub(crate) area_partition: Option<AreaPartition>,
    pub(crate) parent_count: u16,
    pub(crate) parent_array_ptr: Ptr,
    pub(crate) child_count: u16,
    pub(crate) child_array_ptr: Ptr,
    pub(crate) active_bbox: ActiveBoundingBox,
    pub(crate) node_bbox: BoundingBox,
    pub(crate) model_bbox: BoundingBox,
    pub(crate) child_bbox: BoundingBox,
    pub(crate) field192: u32,
    pub(crate) field196: u32,
    pub(crate) field200: u32,
    pub(crate) field204: u32,
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
pub(crate) struct AreaPartitionPg {
    pub(crate) x: i32, // 0
    pub(crate) y: i32, // 4
}
impl_as_bytes!(AreaPartitionPg, 8);

impl AreaPartitionPg {
    pub(crate) const DEFAULT: Self = Self { x: -1, y: -1 };
    pub(crate) const ZERO: Self = Self { x: 0, y: 0 };
}

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
pub(crate) struct AreaPartitionPm {
    pub(crate) x: i16,         // 0
    pub(crate) y: i16,         // 2
    pub(crate) virtual_x: i16, // 4
    pub(crate) virtual_y: i16, // 6
}
impl_as_bytes!(AreaPartitionPm, 8);

impl AreaPartitionPm {
    pub(crate) const DEFAULT: Self = Self {
        x: -1,
        y: -1,
        virtual_x: 0,
        virtual_y: 0,
    };
    pub(crate) const ZERO: Self = Self {
        x: 0,
        y: 0,
        virtual_x: 0,
        virtual_y: 0,
    };
}
