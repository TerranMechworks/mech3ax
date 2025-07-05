use super::{Class, Flags, NodeRcC, ABORT_TEST_NAME, ABORT_TEST_NODE_NAME, ZERO_NAME};
use crate::nodes::types::AreaPartitionPg;
use crate::nodes::NodeClass;
use mech3ax_api_types::gamez::nodes::{Node, NodeData};
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_api_types::Vec3;
use mech3ax_types::maybe::SupportsMaybe as _;
use mech3ax_types::{Ascii, PaddedI8, Ptr};

pub(crate) fn make_node_zero() -> NodeRcC {
    NodeRcC {
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
        area_partition: AreaPartitionPg::ZERO,
        parent_count: 0,
        parent_array_ptr: Ptr::NULL,
        child_count: 0,
        child_array_ptr: Ptr::NULL,
        bbox_mid: Vec3::DEFAULT,
        bbox_diag: 0.0,
        node_bbox: BoundingBox::EMPTY,
        model_bbox: BoundingBox::EMPTY,
        child_bbox: BoundingBox::EMPTY,
        activation_ptr: Ptr::NULL,
    }
}

pub(crate) fn make_node(node: &Node) -> NodeRcC {
    let name = if node.name == ABORT_TEST_NAME {
        log::debug!("node name `abort_test` fixup");
        ABORT_TEST_NODE_NAME
    } else {
        Ascii::from_str_node_name(&node.name)
    };

    let node_class = NodeClass::from_data(&node.data);

    NodeRcC {
        name,
        flags: node.flags.maybe(),
        field040: 0,
        update_flags: node.update_flags,
        zone_id: node.zone_id.maybe(),
        node_class: node_class.maybe(),
        data_ptr: Ptr(node.data_ptr),
        model_index: node.model_index.map(Into::into).unwrap_or(-1),
        environment_data: Ptr::NULL,
        action_priority: 0,
        action_callback: Ptr::NULL,
        area_partition: AreaPartitionPg::ZERO,
        parent_count: 0,
        parent_array_ptr: Ptr::NULL,
        child_count: 0,
        child_array_ptr: Ptr::NULL,
        bbox_mid: Vec3::DEFAULT,
        bbox_diag: 0.0,
        node_bbox: BoundingBox::EMPTY,
        model_bbox: BoundingBox::EMPTY,
        child_bbox: BoundingBox::EMPTY,
        activation_ptr: Ptr::NULL,
    }
}
