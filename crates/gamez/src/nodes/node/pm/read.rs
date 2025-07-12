use super::{AreaPartitionC, Class, NodePmC, VirtualPartitionC};
use crate::nodes::check::{model_index, ptr};
use crate::nodes::types::{NodeClass, NodeInfo, ZONE_ALWAYS};
use mech3ax_api_types::gamez::nodes::{ActiveBoundingBox, BoundingBox, NodeFlags, Partition};
use mech3ax_api_types::{Count, IndexO, Vec3};
use mech3ax_common::{Result, chk};
use mech3ax_types::check::node_name;
use mech3ax_types::{Ascii, Ptr};

fn ap(value: i16) -> Result<u8, String> {
    u8::try_from(value).map_err(|_e| format!("expected {} in 0..={}", value, u8::MAX))
}

pub(crate) fn assert_node(node: &NodePmC, offset: usize, model_count: Count) -> Result<NodeInfo> {
    let name = chk!(offset, node_name(&node.name))?;
    let flags = chk!(offset, ?node.flags)?;
    chk!(offset, node.field040 == 0)?;
    // TODO
    // let update_flags 44
    let zone_id = chk!(offset, ?node.zone_id)?;
    let node_class = chk!(offset, ?node.node_class)?;
    // data_ptr (056) is variable
    let model_index = chk!(offset, model_index(node.model_index, model_count))?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;

    let area_partition = if node.area_partition == AreaPartitionC::DEFAULT {
        None
    } else {
        let x = chk!(offset, ap(node.area_partition.x))?;
        let z = chk!(offset, ap(node.area_partition.z))?;
        Some(Partition { x, z })
    };

    let virtual_partition = if node.virtual_partition == VirtualPartitionC::DEFAULT {
        None
    } else {
        let x = chk!(offset, ap(node.virtual_partition.x))?;
        let z = chk!(offset, ap(node.virtual_partition.z))?;
        Some(Partition { x, z })
    };

    // usually, parent count should be 0 or 1
    let parent_count = chk!(offset, ?node.parent_count)?;
    let parent_array_ptr = chk!(offset, ptr(node.parent_array_ptr, parent_count))?;

    let child_count = chk!(offset, ?node.child_count)?;
    let child_array_ptr = chk!(offset, ptr(node.child_array_ptr, child_count))?;

    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    let active_bbox = chk!(offset, ?node.active_bbox)?;
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
        NodeClass::Empty => {}
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
        virtual_partition,
        parent_count,
        parent_array_ptr,
        child_count,
        child_array_ptr,
        active_bbox,
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

fn node_class_mechlib(_: Class, node_class: NodeClass) -> Result<(), String> {
    match node_class {
        NodeClass::Object3d => Ok(()),
        NodeClass::Lod => Ok(()),
        _ => Err(format!(
            "expected {:?} to be {:?}  or {:?} in mechlib",
            node_class,
            NodeClass::Object3d,
            NodeClass::Lod,
        )),
    }
}

pub(crate) fn assert_node_mechlib(node: &NodePmC, offset: usize) -> Result<NodeInfo> {
    let name = chk!(offset, node_name(&node.name))?;
    let flags = chk!(offset, ?node.flags)?;
    chk!(offset, node.field040 == 0)?;
    // TODO
    // let update_flags 44
    let zone_id = chk!(offset, ?node.zone_id)?;
    let node_class = chk!(offset, ?node.node_class)?;
    chk!(offset, node_class_mechlib(node.node_class, node_class))?;
    chk!(offset, node.data_ptr != Ptr::NULL)?;
    // in mechlib, the model index is a pointer
    match node_class {
        NodeClass::Object3d => {}
        NodeClass::Lod => {
            chk!(offset, node.model_index == 0)?;
        }
        _ => unreachable!("invalid mechlib node class {node_class:?}"),
    }
    let model_ptr = node.model_index.value as u32;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;

    let area_partition = if node.area_partition == AreaPartitionC::DEFAULT {
        None
    } else {
        let x = chk!(offset, ap(node.area_partition.x))?;
        let z = chk!(offset, ap(node.area_partition.z))?;
        Some(Partition { x, z })
    };

    let virtual_partition = if node.virtual_partition == VirtualPartitionC::DEFAULT {
        None
    } else {
        let x = chk!(offset, ap(node.virtual_partition.x))?;
        let z = chk!(offset, ap(node.virtual_partition.z))?;
        Some(Partition { x, z })
    };

    // usually, parent count should be 0 or 1
    let parent_count = chk!(offset, ?node.parent_count)?;
    chk!(offset, node.parent_count < 2)?;
    let parent_array_ptr = chk!(offset, ptr(node.parent_array_ptr, parent_count))?;

    let child_count = chk!(offset, ?node.child_count)?;
    let child_array_ptr = chk!(offset, ptr(node.child_array_ptr, child_count))?;

    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    let active_bbox = chk!(offset, ?node.active_bbox)?;
    // TODO: assert based on flags
    // node_bbox (116) is variable
    // model_bbox (140) is variable
    // child_bbox (164) is variable
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    chk!(offset, node.field192 == 0)?;
    chk!(offset, node.field196 == 160)?;
    chk!(offset, node.field200 == 0)?;
    chk!(offset, node.field204 == 0)?;

    Ok(NodeInfo {
        name,
        flags,
        update_flags: node.update_flags,
        zone_id,
        data_ptr: node.data_ptr,
        model_index: IndexO::NONE,
        area_partition,
        virtual_partition,
        parent_count,
        parent_array_ptr,
        child_count,
        child_array_ptr,
        active_bbox,
        node_bbox: node.node_bbox,
        model_bbox: node.model_bbox,
        child_bbox: node.child_bbox,
        field192: node.field192,
        field196: node.field196,
        field200: node.field200,
        field204: node.field204,
        node_class,
        offset: model_ptr,
    })
}

const CAMERA_NAME: Ascii<36> = Ascii::node_name("camera1");

fn assert_camera(node: &NodePmC, offset: usize) -> Result<()> {
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
    chk!(offset, node.area_partition == AreaPartitionC::DEFAULT)?;
    chk!(offset, node.virtual_partition == VirtualPartitionC::DEFAULT)?;
    chk!(offset, node.parent_count == 0)?;
    chk!(offset, node.child_count == 0)?;
    chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.active_bbox == ActiveBoundingBox::Node)?;
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

fn assert_display(node: &NodePmC, offset: usize) -> Result<()> {
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
    chk!(offset, node.area_partition == AreaPartitionC::DEFAULT)?;
    chk!(offset, node.virtual_partition == VirtualPartitionC::DEFAULT)?;
    chk!(offset, node.parent_count == 0)?;
    chk!(offset, node.child_count == 0)?;
    chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.active_bbox == ActiveBoundingBox::Node)?;
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

fn assert_light(node: &NodePmC, offset: usize) -> Result<()> {
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
    chk!(offset, node.area_partition == AreaPartitionC::DEFAULT)?;
    chk!(offset, node.virtual_partition == VirtualPartitionC::DEFAULT)?;
    chk!(offset, node.parent_count == 0)?;
    chk!(offset, node.child_count == 0)?;
    chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.active_bbox == ActiveBoundingBox::Node)?;
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

fn assert_lod(node: &NodePmC, offset: usize) -> Result<()> {
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
    chk!(offset, node.area_partition == AreaPartitionC::DEFAULT)?;
    chk!(offset, node.virtual_partition == VirtualPartitionC::DEFAULT)?;
    chk!(offset, node.parent_count == 1)?;
    chk!(offset, node.child_count > 0)?;
    chk!(offset, node.parent_array_ptr != Ptr::NULL)?;
    chk!(offset, node.child_array_ptr != Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.active_bbox == ActiveBoundingBox::Child)?;
    chk!(offset, node.node_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.child_bbox != BoundingBox::EMPTY)?;
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    chk!(offset, node.field192 == 0)?;
    chk!(offset, node.field196 == 160)?;
    chk!(offset, node.field200 == 0)?;
    chk!(offset, node.field204 == 0)?;
    Ok(())
}

fn assert_object3d(node: &NodePmC, offset: usize) -> Result<()> {
    // chk!(offset, node.name == )?;
    // chk!(offset, node.flags == )?;
    chk!(offset, node.field040 == 0)?;
    // chk!(offset, node.update_flags == 0)?; // 1, 45697
    // chk!(offset, node.zone_id == ZONE_ALWAYS)?;
    chk!(offset, node.node_class == NodeClass::Object3d)?;
    chk!(offset, node.data_ptr != Ptr::NULL)?;
    // chk!(offset, node.model_index == -1)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;
    // chk!(offset, node.area_partition == AreaPartitionPm::DEFAULT)?;
    // chk!(offset, node.parent_count == 0)?;
    // chk!(offset, node.child_count == 0)?;
    // chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    // chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    // chk!(offset, node.active_bbox == ActiveBoundingBox::Node)?;
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

fn assert_window(node: &NodePmC, offset: usize) -> Result<()> {
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
    chk!(offset, node.area_partition == AreaPartitionC::DEFAULT)?;
    chk!(offset, node.virtual_partition == VirtualPartitionC::DEFAULT)?;
    chk!(offset, node.parent_count == 0)?;
    chk!(offset, node.child_count == 0)?;
    chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.active_bbox == ActiveBoundingBox::Node)?;
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

fn assert_world(node: &NodePmC, offset: usize) -> Result<()> {
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
    chk!(offset, node.area_partition == AreaPartitionC::DEFAULT)?;
    chk!(offset, node.virtual_partition == VirtualPartitionC::DEFAULT)?;
    chk!(offset, node.parent_count == 0)?;
    chk!(offset, node.child_count > 0)?;
    chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    chk!(offset, node.child_array_ptr != Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    chk!(offset, node.active_bbox == ActiveBoundingBox::Node)?;
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
