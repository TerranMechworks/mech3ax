use super::{ActiveBBox, Class, Flags, NodePmC, ZERO_NAME};
use crate::nodes::types::{AreaPartitionPg, AreaPartitionPm};
use crate::nodes::NodeClass;
use mech3ax_api_types::gamez::nodes::Node;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_api_types::{Index, Vec3};
use mech3ax_common::{assert_len, Result};
use mech3ax_types::maybe::SupportsMaybe as _;
use mech3ax_types::{Ascii, PaddedI8, Ptr};

pub(crate) fn make_node_zero() -> NodePmC {
    NodePmC {
        name: ZERO_NAME,
        flags: Flags::empty(),
        field040: 0,
        update_flags: 0,
        zone_id: PaddedI8::empty(),
        node_class: Class::empty(),
        data_ptr: Ptr::NULL,
        model_index: -1,
        environment_data: Ptr::NULL,
        action_priority: 0,
        action_callback: Ptr::NULL,
        area_partition: AreaPartitionPm::ZERO,
        parent_count: 0,
        child_count: 0,
        parent_array_ptr: Ptr::NULL,
        child_array_ptr: Ptr::NULL,
        bbox_mid: Vec3::DEFAULT,
        bbox_diag: 0.0,
        active_bbox: ActiveBBox::empty(),
        node_bbox: BoundingBox::EMPTY,
        model_bbox: BoundingBox::EMPTY,
        child_bbox: BoundingBox::EMPTY,
        activation_ptr: Ptr::NULL,
        field192: 0,
        field196: 0,
        field200: 0,
        field204: 0,
    }
}
