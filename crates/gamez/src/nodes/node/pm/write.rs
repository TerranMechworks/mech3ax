use super::{AreaPartitionC, NodePmC, VirtualPartitionC};
use crate::nodes::NodeClass;
use mech3ax_api_types::gamez::nodes::Node;
use mech3ax_api_types::{IndexO32, Vec3};
use mech3ax_common::{Result, len};
use mech3ax_types::{Ascii, Ptr, SupportsMaybe as _};

pub(crate) fn make_node(node: &Node) -> Result<NodePmC> {
    let name = Ascii::from_str_node_name(&node.name);
    let node_class = NodeClass::from_data(&node.data);

    let area_partition = match &node.area_partition {
        Some(ap) => AreaPartitionC {
            x: ap.x.into(),
            z: ap.z.into(),
        },
        None => AreaPartitionC::DEFAULT,
    };

    let virtual_partition = match &node.virtual_partition {
        Some(vp) => VirtualPartitionC {
            x: vp.x.into(),
            z: vp.z.into(),
        },
        None => VirtualPartitionC::DEFAULT,
    };

    let parent_count = len!(node.parent_indices.len(), "node parent indices")?;
    let child_count = len!(node.child_indices.len(), "node child indices")?;
    let parent_array_ptr = Ptr(node.parent_array_ptr);
    let child_array_ptr = Ptr(node.child_array_ptr);

    Ok(NodePmC {
        name,
        flags: node.flags.maybe(),
        field040: 0,
        update_flags: node.update_flags,
        zone_id: node.zone_id.maybe(),
        node_class: node_class.maybe(),
        data_ptr: Ptr(node.data_ptr),
        model_index: node.model_index.maybe(),
        environment_data: Ptr::NULL,
        action_priority: 1,
        action_callback: Ptr::NULL,
        area_partition,
        virtual_partition,
        parent_count: parent_count.maybe(),
        child_count: child_count.maybe(),
        parent_array_ptr,
        child_array_ptr,
        bbox_mid: Vec3::DEFAULT,
        bbox_diag: 0.0,
        active_bbox: node.active_bbox.maybe(),
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

pub(crate) fn make_node_mechlib(node: &Node) -> Result<NodePmC> {
    let name = Ascii::from_str_node_name(&node.name);
    let node_class = NodeClass::from_data(&node.data);

    // this holds the model ptr for mechlib
    let model_index = IndexO32::new(node.index as i32);

    let area_partition = match &node.area_partition {
        Some(ap) => AreaPartitionC {
            x: ap.x.into(),
            z: ap.z.into(),
        },
        None => AreaPartitionC::DEFAULT,
    };

    let virtual_partition = match &node.virtual_partition {
        Some(vp) => VirtualPartitionC {
            x: vp.x.into(),
            z: vp.z.into(),
        },
        None => VirtualPartitionC::DEFAULT,
    };

    let parent_count = len!(node.parent_indices.len(), "node parent indices")?;
    let child_count = len!(node.child_indices.len(), "node child indices")?;
    let parent_array_ptr = Ptr(node.parent_array_ptr);
    let child_array_ptr = Ptr(node.child_array_ptr);

    Ok(NodePmC {
        name,
        flags: node.flags.maybe(),
        field040: 0,
        update_flags: node.update_flags,
        zone_id: node.zone_id.maybe(),
        node_class: node_class.maybe(),
        data_ptr: Ptr(node.data_ptr),
        model_index,
        environment_data: Ptr::NULL,
        action_priority: 1,
        action_callback: Ptr::NULL,
        area_partition,
        virtual_partition,
        parent_count: parent_count.maybe(),
        child_count: child_count.maybe(),
        parent_array_ptr,
        child_array_ptr,
        bbox_mid: Vec3::DEFAULT,
        bbox_diag: 0.0,
        active_bbox: node.active_bbox.maybe(),
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
