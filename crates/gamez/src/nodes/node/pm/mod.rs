mod read;
mod write;

use crate::nodes::types::NodeClass;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::nodes::{ActiveBoundingBox, BoundingBox, NodeFlags};
use mech3ax_api_types::{Count16, IndexO32, Vec3};
use mech3ax_types::{Ascii, Maybe, Offsets, PaddedI8, Ptr, impl_as_bytes};
pub(crate) use read::{assert_node, assert_node_mechlib};
pub(crate) use write::{make_node, make_node_mechlib};

type Flags = Maybe<u32, NodeFlags>;
type Class = Maybe<u32, NodeClass>;
type ActiveBBox = Maybe<u32, ActiveBoundingBox>;

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct AreaPartitionC {
    x: i16, // 0
    z: i16, // 2
}
impl_as_bytes!(AreaPartitionC, 4);

impl AreaPartitionC {
    const DEFAULT: Self = Self { x: -1, z: -1 };
}

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct VirtualPartitionC {
    x: i16, // 0
    z: i16, // 2
}
impl_as_bytes!(VirtualPartitionC, 4);

impl VirtualPartitionC {
    const DEFAULT: Self = Self { x: 0, z: 0 };
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
pub(crate) struct NodePmC {
    name: Ascii<36>,                      // 000
    flags: Flags,                         // 036
    field040: u32,                        // 040
    update_flags: u32,                    // 044
    zone_id: PaddedI8,                    // 048
    node_class: Class,                    // 052
    data_ptr: Ptr,                        // 056
    model_index: IndexO32,                // 060
    environment_data: Ptr,                // 064
    action_priority: u32,                 // 068
    action_callback: Ptr,                 // 072
    area_partition: AreaPartitionC,       // 076
    virtual_partition: VirtualPartitionC, // 080
    parent_count: Count16,                // 084
    child_count: Count16,                 // 086
    parent_array_ptr: Ptr,                // 088
    child_array_ptr: Ptr,                 // 092
    bbox_mid: Vec3,                       // 096
    bbox_diag: f32,                       // 108
    active_bbox: ActiveBBox,              // 112
    node_bbox: BoundingBox,               // 116
    model_bbox: BoundingBox,              // 140
    child_bbox: BoundingBox,              // 164
    activation_ptr: Ptr,                  // 188
    field192: i32,                        // 192
    field196: i32,                        // 196
    field200: i32,                        // 200
    field204: i32,                        // 204
}
impl_as_bytes!(NodePmC, 208);
