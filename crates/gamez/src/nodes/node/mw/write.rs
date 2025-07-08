use super::{Class, Flags, NodeMwC, ZERO_NAME};
use crate::nodes::types::AreaPartitionC;
use crate::nodes::NodeClass;
use mech3ax_api_types::gamez::nodes::{BoundingBox, Node};
use mech3ax_api_types::{Index, Vec3};
use mech3ax_common::{assert_len, Result};
use mech3ax_types::maybe::SupportsMaybe as _;
use mech3ax_types::{Ascii, PaddedI8, Ptr};

pub(crate) fn make_node_zero() -> NodeMwC {
    NodeMwC {
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
        area_partition: AreaPartitionC::ZERO,
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
        field192: 0,
        field196: 0,
        field200: 0,
        field204: 0,
    }
}

pub(crate) fn make_node(node: &Node) -> Result<NodeMwC> {
    let name = Ascii::from_str_node_name(&node.name);
    let node_class = NodeClass::from_data(&node.data);

    let area_partition = match &node.area_partition {
        Some(ap) => AreaPartitionC {
            x: ap.x.into(),
            z: ap.z.into(),
        },
        None => AreaPartitionC::DEFAULT,
    };

    if node.virtual_partition.is_some() {
        log::warn!("WARN: node virtual partition ignored in MW");
    }

    // TODO
    let mut parent_count = assert_len!(i32, node.parent_indices.len(), "node parent indices")?;
    let child_count = assert_len!(i32, node.child_indices.len(), "node child indices")?;
    let parent_array_ptr = Ptr(node.parent_array_ptr);
    let child_array_ptr = Ptr(node.child_array_ptr);

    if node_class == NodeClass::Empty {
        parent_count = 0;
    }

    Ok(NodeMwC {
        name,
        flags: node.flags.maybe(),
        field040: 0,
        update_flags: node.update_flags,
        zone_id: node.zone_id.maybe(),
        node_class: node_class.maybe(),
        data_ptr: Ptr(node.data_ptr),
        model_index: node.model_index.map(Index::to_i32).unwrap_or(-1),
        environment_data: Ptr::NULL,
        action_priority: 1,
        action_callback: Ptr::NULL,
        area_partition,
        parent_count,
        parent_array_ptr,
        child_count,
        child_array_ptr,
        bbox_mid: Vec3::DEFAULT,
        bbox_diag: 0.0,
        node_bbox: node.node_bbox,
        model_bbox: node.model_bbox,
        child_bbox: node.child_bbox,
        activation_ptr: Ptr::NULL,
        field192: node.field192,
        field196: node.field196,
        field200: node.field200,
        field204: node.field204,
    })
}
