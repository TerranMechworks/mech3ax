mod read;
mod write;

use crate::nodes::types::{AreaPartitionPg, NodeClass};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::nodes::{BoundingBox, NodeFlags};
use mech3ax_api_types::Vec3;
use mech3ax_types::{impl_as_bytes, Ascii, Maybe, Offsets, PaddedI8, Ptr};
pub(crate) use read::{assert_node, assert_node_zero};
pub(crate) use write::{make_node, make_node_zero};

type Flags = Maybe<u32, NodeFlags>;
type Class = Maybe<u32, NodeClass>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
pub(crate) struct NodeMwC {
    name: Ascii<36>,                 // 000
    flags: Flags,                    // 036
    field040: u32,                   // 040
    update_flags: u32,               // 044
    zone_id: PaddedI8,               // 048
    node_class: Class,               // 052
    data_ptr: Ptr,                   // 056
    model_index: i32,                // 060
    environment_data: Ptr,           // 064
    action_priority: u32,            // 068
    action_callback: Ptr,            // 072
    area_partition: AreaPartitionPg, // 076
    parent_count: i32,               // 084
    parent_array_ptr: Ptr,           // 088
    child_count: i32,                // 092
    child_array_ptr: Ptr,            // 096
    bbox_mid: Vec3,                  // 100
    bbox_diag: f32,                  // 112
    node_bbox: BoundingBox,          // 116
    model_bbox: BoundingBox,         // 140
    child_bbox: BoundingBox,         // 164
    activation_ptr: Ptr,             // 188
    field192: i32,                   // 192
    field196: i32,                   // 196
    field200: i32,                   // 200
    field204: i32,                   // 204
}
impl_as_bytes!(NodeMwC, 208);

impl NodeMwC {
    pub(crate) fn is_zero(&self) -> bool {
        self.name == ZERO_NAME
    }
}

const ZERO_NAME: Ascii<36> = Ascii::zero();
