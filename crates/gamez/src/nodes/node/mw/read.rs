use super::{NodeMwC, ZERO_NAME};
use crate::nodes::check::{ap, model_index, node_count, ptr};
use crate::nodes::types::{AreaPartitionPg, NodeClass, NodeInfo, ZONE_ALWAYS};
use mech3ax_api_types::gamez::nodes::{ActiveBoundingBox, AreaPartition, NodeFlags};
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_api_types::Vec3;
use mech3ax_common::{chk, Result};
use mech3ax_types::check::node_name;
use mech3ax_types::{Ascii, Ptr};

pub(crate) fn assert_node_zero(node: &NodeMwC, offset: usize) -> Result<()> {
    chk!(offset, node.name == ZERO_NAME)?;
    chk!(offset, node.flags == NodeFlags::empty())?;
    chk!(offset, node.field040 == 0)?;
    chk!(offset, node.update_flags == 0)?;
    chk!(offset, node.zone_id == 0u32)?;
    chk!(offset, node.node_class == 0)?;
    chk!(offset, node.data_ptr == Ptr::NULL)?;
    chk!(offset, node.model_index == -1)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 0)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;
    chk!(offset, node.area_partition == AreaPartitionPg::ZERO)?;
    chk!(offset, node.parent_count == 0)?;
    chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    chk!(offset, node.child_count == 0)?;
    chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.node_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.child_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    chk!(offset, node.field192 == 0)?;
    chk!(offset, node.field196 == 0)?;
    chk!(offset, node.field200 == 0)?;
    chk!(offset, node.field204 == 0)?;
    Ok(())
}

pub(crate) fn assert_node(node: &NodeMwC, offset: usize, model_count: i32) -> Result<NodeInfo> {
    let name = chk!(offset, node_name(&node.name))?;
    let flags = chk!(offset, ?node.flags)?;
    chk!(offset, node.field040 == 0)?;
    // TODO
    // let update_flags 44
    let zone_id = chk!(offset, ?node.zone_id)?;
    let node_class = chk!(offset, ?node.node_class)?;
    // data_ptr (056) is variable
    let model_index = chk!(offset, model_index(node.model_index))?;
    chk!(offset, node.model_index < model_count)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;

    let area_partition = if node.area_partition == AreaPartitionPg::DEFAULT {
        None
    } else {
        let x = chk!(offset, ap(node.area_partition.x))?;
        let y = chk!(offset, ap(node.area_partition.y))?;
        Some(AreaPartition {
            x,
            y,
            virtual_x: 0,
            virtual_y: 0,
        })
    };

    // usually, parent count should be 0 or 1
    let parent_count = chk!(offset, node_count(node.parent_count))?;
    let parent_array_ptr = chk!(offset, ptr(node.parent_array_ptr, parent_count))?;

    let child_count = chk!(offset, node_count(node.child_count))?;
    let child_array_ptr = chk!(offset, ptr(node.child_array_ptr, child_count))?;

    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    // TODO: assert based on flags
    // node_bbox (116) is variable
    // model_bbox (140) is variable
    // child_bbox (164) is variable
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    chk!(offset, node.field192 == 0)?;
    // chk!(offset, node.field196 == 0)?;
    chk!(offset, node.field200 == 0)?;
    chk!(offset, node.field204 == 0)?;

    match node_class {
        NodeClass::Camera => assert_camera(&node, offset)?,
        NodeClass::Display => assert_display(&node, offset)?,
        NodeClass::Empty => assert_empty(&node, offset)?,
        NodeClass::Light => assert_light(&node, offset)?,
        NodeClass::Lod => assert_lod(&node, offset)?,
        NodeClass::Object3d => assert_object3d(&node, offset)?,
        NodeClass::Window => assert_window(&node, offset)?,
        NodeClass::World => assert_world(&node, offset)?,
    }

    Ok(NodeInfo {
        name,
        flags,
        update_flags: node.update_flags,
        zone_id,
        data_ptr: node.data_ptr,
        model_index,
        area_partition,
        parent_count,
        parent_array_ptr,
        child_count,
        child_array_ptr,
        active_bbox: ActiveBoundingBox::Node,
        node_bbox: node.node_bbox,
        model_bbox: node.model_bbox,
        child_bbox: node.child_bbox,
        field192: node.field192,
        field196: node.field196,
        field200: node.field200,
        field204: node.field204,
        node_class,
        offset: 0, // to be filled in by read loop
    })
}

const CAMERA_NAME: Ascii<36> = Ascii::node_name("camera1");

fn assert_camera(node: &NodeMwC, offset: usize) -> Result<()> {
    let camera_flags = NodeFlags::ACTIVE
        | NodeFlags::TREE_VALID
        | NodeFlags::ID_ZONE_CHECK
        | NodeFlags::ALTITUDE_SURFACE
        | NodeFlags::INTERSECT_SURFACE;
    chk!(offset, node.name == CAMERA_NAME)?;
    chk!(offset, node.flags == camera_flags)?;
    chk!(offset, node.field040 == 0)?;
    chk!(offset, node.update_flags == 0)?;
    chk!(offset, node.zone_id == ZONE_ALWAYS)?;
    chk!(offset, node.node_class == NodeClass::Camera)?;
    chk!(offset, node.data_ptr != Ptr::NULL)?;
    chk!(offset, node.model_index == -1)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;
    chk!(offset, node.area_partition == AreaPartitionPg::DEFAULT)?;
    chk!(offset, node.parent_count == 0)?;
    chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    chk!(offset, node.child_count == 0)?;
    chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.node_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.child_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    chk!(offset, node.field192 == 0)?;
    chk!(offset, node.field196 == 0)?;
    chk!(offset, node.field200 == 0)?;
    chk!(offset, node.field204 == 0)?;
    Ok(())
}

const DISPLAY_NAME: Ascii<36> = Ascii::node_name("display");

fn assert_display(node: &NodeMwC, offset: usize) -> Result<()> {
    let display_flags = NodeFlags::ACTIVE
        | NodeFlags::TREE_VALID
        | NodeFlags::ID_ZONE_CHECK
        | NodeFlags::ALTITUDE_SURFACE
        | NodeFlags::INTERSECT_SURFACE;
    chk!(offset, node.name == DISPLAY_NAME)?;
    chk!(offset, node.flags == display_flags)?;
    chk!(offset, node.field040 == 0)?;
    chk!(offset, node.update_flags == 0)?;
    chk!(offset, node.zone_id == ZONE_ALWAYS)?;
    chk!(offset, node.node_class == NodeClass::Display)?;
    chk!(offset, node.data_ptr != Ptr::NULL)?;
    chk!(offset, node.model_index == -1)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;
    chk!(offset, node.area_partition == AreaPartitionPg::DEFAULT)?;
    chk!(offset, node.parent_count == 0)?;
    chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    chk!(offset, node.child_count == 0)?;
    chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.node_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.child_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    chk!(offset, node.field192 == 0)?;
    chk!(offset, node.field196 == 0)?;
    chk!(offset, node.field200 == 0)?;
    chk!(offset, node.field204 == 0)?;
    Ok(())
}

fn assert_empty(node: &NodeMwC, offset: usize) -> Result<()> {
    // chk!(offset, node.name == )?;
    // chk!(offset, node.flags == )?;
    chk!(offset, node.field040 == 0)?;
    // chk!(offset, node.update_flags == 0)?; // [1, 3, 5, 7]
    // chk!(offset, node.zone_id == ZONE_ALWAYS)?;
    chk!(offset, node.node_class == NodeClass::Empty)?;
    chk!(offset, node.data_ptr == Ptr::NULL)?;
    chk!(offset, node.model_index == -1)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;
    chk!(offset, node.area_partition == AreaPartitionPg::DEFAULT)?;
    chk!(offset, node.parent_count == 0)?;
    chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    chk!(offset, node.child_count == 0)?;
    chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    // chk!(offset, node.node_bbox == BoundingBox::EMPTY)?;
    // chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
    // chk!(offset, node.child_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    chk!(offset, node.field192 == 0)?;
    chk!(offset, node.field196 == 160)?;
    chk!(offset, node.field200 == 0)?;
    chk!(offset, node.field204 == 0)?;
    Ok(())
}

const LIGHT_NAME: Ascii<36> = Ascii::node_name("sunlight");
const LIGHT_BBOX: BoundingBox = BoundingBox {
    a: Vec3 {
        x: 1.0,
        y: 1.0,
        z: -2.0,
    },
    b: Vec3 {
        x: 2.0,
        y: 2.0,
        z: -1.0,
    },
};

fn assert_light(node: &NodeMwC, offset: usize) -> Result<()> {
    let light_flags = NodeFlags::ACTIVE
        | NodeFlags::TREE_VALID
        | NodeFlags::ID_ZONE_CHECK
        | NodeFlags::ALTITUDE_SURFACE
        | NodeFlags::INTERSECT_SURFACE
        | NodeFlags::BBOX_NODE;
    chk!(offset, node.name == LIGHT_NAME)?;
    chk!(offset, node.flags == light_flags)?;
    chk!(offset, node.field040 == 0)?;
    chk!(offset, node.update_flags == 0)?;
    chk!(offset, node.zone_id == ZONE_ALWAYS)?;
    chk!(offset, node.node_class == NodeClass::Light)?;
    chk!(offset, node.data_ptr != Ptr::NULL)?;
    chk!(offset, node.model_index == -1)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;
    chk!(offset, node.area_partition == AreaPartitionPg::DEFAULT)?;
    chk!(offset, node.parent_count == 0)?;
    chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    chk!(offset, node.child_count == 0)?;
    chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.node_bbox == LIGHT_BBOX)?;
    chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.child_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    chk!(offset, node.field192 == 0)?;
    chk!(offset, node.field196 == 0)?;
    chk!(offset, node.field200 == 0)?;
    chk!(offset, node.field204 == 0)?;
    Ok(())
}

fn assert_lod(node: &NodeMwC, offset: usize) -> Result<()> {
    // chk!(offset, node.name == )?;
    // chk!(offset, node.flags == )?;
    chk!(offset, node.field040 == 0)?;
    // chk!(offset, node.update_flags == 0)?; // 1
    // chk!(offset, node.zone_id == ZONE_ALWAYS)?;
    chk!(offset, node.node_class == NodeClass::Lod)?;
    chk!(offset, node.data_ptr != Ptr::NULL)?;
    chk!(offset, node.model_index == -1)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;
    // chk!(offset, node.area_partition == AreaPartitionPg::DEFAULT)?;
    chk!(offset, node.parent_count == 1)?;
    chk!(offset, node.parent_array_ptr != Ptr::NULL)?;
    chk!(offset, node.child_count > 0)?;
    chk!(offset, node.child_array_ptr != Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.node_bbox != BoundingBox::EMPTY)?;
    chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.child_bbox == node.node_bbox)?;
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    chk!(offset, node.field192 == 0)?;
    chk!(offset, node.field196 == 160)?;
    chk!(offset, node.field200 == 0)?;
    chk!(offset, node.field204 == 0)?;
    Ok(())
}

fn assert_object3d(node: &NodeMwC, offset: usize) -> Result<()> {
    // chk!(offset, node.name == )?;
    // chk!(offset, node.flags == )?;
    chk!(offset, node.field040 == 0)?;
    // chk!(offset, node.update_flags == 0)?; // 1
    // chk!(offset, node.zone_id == ZONE_ALWAYS)?;
    chk!(offset, node.node_class == NodeClass::Object3d)?;
    chk!(offset, node.data_ptr != Ptr::NULL)?;
    // chk!(offset, node.model_index == -1)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;
    // chk!(offset, node.area_partition == AreaPartitionPg::DEFAULT)?;
    // chk!(offset, node.parent_count == 0)?;
    // chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    // chk!(offset, node.child_count == 0)?;
    // chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    // chk!(offset, node.node_bbox == BoundingBox::EMPTY)?;
    // chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
    // chk!(offset, node.child_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    chk!(offset, node.field192 == 0)?;
    chk!(offset, node.field196 == 160)?;
    chk!(offset, node.field200 == 0)?;
    chk!(offset, node.field204 == 0)?;
    Ok(())
}

const WINDOW_NAME: Ascii<36> = Ascii::node_name("window1");

fn assert_window(node: &NodeMwC, offset: usize) -> Result<()> {
    let window_flags = NodeFlags::ACTIVE
        | NodeFlags::TREE_VALID
        | NodeFlags::ID_ZONE_CHECK
        | NodeFlags::ALTITUDE_SURFACE
        | NodeFlags::INTERSECT_SURFACE;
    chk!(offset, node.name == WINDOW_NAME)?;
    chk!(offset, node.flags == window_flags)?;
    chk!(offset, node.field040 == 0)?;
    chk!(offset, node.update_flags == 0)?;
    chk!(offset, node.zone_id == ZONE_ALWAYS)?;
    chk!(offset, node.node_class == NodeClass::Window)?;
    chk!(offset, node.data_ptr != Ptr::NULL)?;
    chk!(offset, node.model_index == -1)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;
    chk!(offset, node.area_partition == AreaPartitionPg::DEFAULT)?;
    chk!(offset, node.parent_count == 0)?;
    chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    chk!(offset, node.child_count == 0)?;
    chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.node_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.child_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    chk!(offset, node.field192 == 0)?;
    chk!(offset, node.field196 == 0)?;
    chk!(offset, node.field200 == 0)?;
    chk!(offset, node.field204 == 0)?;
    Ok(())
}

const WORLD_NAME: Ascii<36> = Ascii::node_name("world1");

fn assert_world(node: &NodeMwC, offset: usize) -> Result<()> {
    let world_flags = NodeFlags::ACTIVE
        | NodeFlags::TREE_VALID
        | NodeFlags::ID_ZONE_CHECK
        | NodeFlags::ALTITUDE_SURFACE
        | NodeFlags::INTERSECT_SURFACE;
    chk!(offset, node.name == WORLD_NAME)?;
    chk!(offset, node.flags == world_flags)?;
    chk!(offset, node.field040 == 0)?;
    chk!(offset, node.update_flags == 0)?;
    chk!(offset, node.zone_id == ZONE_ALWAYS)?;
    chk!(offset, node.node_class == NodeClass::World)?;
    chk!(offset, node.data_ptr != Ptr::NULL)?;
    chk!(offset, node.model_index == -1)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;
    chk!(offset, node.area_partition == AreaPartitionPg::DEFAULT)?;
    chk!(offset, node.parent_count == 0)?;
    chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    chk!(offset, node.child_count > 0)?;
    chk!(offset, node.child_array_ptr != Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.node_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.child_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    chk!(offset, node.field192 == 0)?;
    chk!(offset, node.field196 == 0)?;
    chk!(offset, node.field200 == 0)?;
    chk!(offset, node.field204 == 0)?;
    Ok(())
}
