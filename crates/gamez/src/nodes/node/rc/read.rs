use super::{ABORT_TEST_NAME, ABORT_TEST_NODE_NAME, NodeRcC, ZERO_NAME};
use crate::nodes::check::{ap, model_index, ptr};
use crate::nodes::types::{AreaPartitionC, NodeClass, NodeInfo, ZONE_ALWAYS};
use mech3ax_api_types::gamez::nodes::{ActiveBoundingBox, BoundingBox, NodeFlags, Partition};
use mech3ax_api_types::{Count, Vec3};
use mech3ax_common::{Result, chk};
use mech3ax_types::check::node_name;
use mech3ax_types::{Ascii, Ptr};

pub(crate) fn assert_node_zero(node: &NodeRcC, offset: usize) -> Result<()> {
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
    chk!(offset, node.area_partition == AreaPartitionC::ZERO)?;
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
    Ok(())
}

pub(crate) fn assert_node(node: &NodeRcC, offset: usize, model_count: Count) -> Result<NodeInfo> {
    let name = if node.name == ABORT_TEST_NODE_NAME {
        log::debug!("node name `abort_test` fixup");
        ABORT_TEST_NAME.to_string()
    } else {
        chk!(offset, node_name(&node.name))?
    };

    let flags = chk!(offset, ?node.flags)?;
    chk!(offset, node.field040 == 0)?;
    // TODO
    // update_flags 44
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

    // usually, parent count should be 0 or 1
    let parent_count = chk!(offset, ?node.parent_count)?;
    let parent_array_ptr = chk!(offset, ptr(node.parent_array_ptr, parent_count))?;

    let child_count = chk!(offset, ?node.child_count)?;
    let child_array_ptr = chk!(offset, ptr(node.child_array_ptr, child_count))?;

    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;

    // TODO
    // node_bbox (116) is variable
    // model_bbox (140) is variable
    // child_bbox (164) is variable

    if node_class == NodeClass::Lod {
        if flags.contains(NodeFlags::BBOX_NODE) {
            // ()
            // chk!(offset, node.node_bbox != BoundingBox::EMPTY)?;
        } else {
            // chk!(offset, node.node_bbox == BoundingBox::EMPTY)?;
            if node.node_bbox != BoundingBox::EMPTY {
                log::warn!("LOD BBOX_NODE FAIL");
            } else {
                log::warn!("LOD BBOX_NODE PASS");
            }
        }

        // model bbox is never set
        if flags.contains(NodeFlags::BBOX_MODEL) {
            panic!("LOD BBOX_MODEL");
        }

        if flags.contains(NodeFlags::BBOX_CHILD) {
            // ()
            // chk!(offset, node.child_bbox != BoundingBox::EMPTY)?;
        } else {
            // chk!(offset, node.child_bbox == BoundingBox::EMPTY)?;
            if node.child_bbox != BoundingBox::EMPTY {
                log::warn!("LOD BBOX_CHILD FAIL");
            } else {
                log::warn!("LOD BBOX_CHILD PASS");
            }
        }
    } else if node_class == NodeClass::Empty {
        if flags.contains(NodeFlags::BBOX_NODE) {
            // ()
            // chk!(offset, node.node_bbox != BoundingBox::EMPTY)?;
        } else {
            // chk!(offset, node.node_bbox == BoundingBox::EMPTY)?;
            if node.node_bbox != BoundingBox::EMPTY {
                log::warn!("EMPTY BBOX_NODE FAIL");
            } else {
                log::warn!("EMPTY BBOX_NODE PASS");
            }
        }

        if flags.contains(NodeFlags::BBOX_MODEL) {
            // ()
            // chk!(offset, node.model_bbox != BoundingBox::EMPTY)?;
        } else {
            // chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
            if node.model_bbox != BoundingBox::EMPTY {
                log::warn!("EMPTY BBOX_MODEL FAIL");
            } else {
                log::warn!("EMPTY BBOX_MODEL PASS");
            }
        }

        if flags.contains(NodeFlags::BBOX_CHILD) {
            // ()
            // chk!(offset, node.child_bbox != BoundingBox::EMPTY)?;
        } else {
            chk!(offset, node.child_bbox == BoundingBox::EMPTY)?;
        }
    } else {
        if flags.contains(NodeFlags::BBOX_NODE) {
            // ()
            // chk!(offset, node.node_bbox != BoundingBox::EMPTY)?;
        } else {
            chk!(offset, node.node_bbox == BoundingBox::EMPTY)?;
        }

        if flags.contains(NodeFlags::BBOX_MODEL) {
            // ()
            // chk!(offset, node.model_bbox != BoundingBox::EMPTY)?;
        } else {
            chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
        }

        if flags.contains(NodeFlags::BBOX_CHILD) {
            // ()
            // chk!(offset, node.child_bbox != BoundingBox::EMPTY)?;
        } else {
            chk!(offset, node.child_bbox == BoundingBox::EMPTY)?;
        }
    }

    chk!(offset, node.activation_ptr == Ptr::NULL)?;

    match node_class {
        NodeClass::Camera => assert_camera(node, offset)?,
        NodeClass::Display => assert_display(node, offset)?,
        NodeClass::Empty => assert_empty(node, offset)?,
        NodeClass::Light => assert_light(node, offset)?,
        NodeClass::Lod => assert_lod(node, offset)?,
        NodeClass::Object3d => assert_object3d(node, offset)?,
        NodeClass::Window => assert_window(node, offset)?,
        NodeClass::World => assert_world(node, offset)?,
    }

    Ok(NodeInfo {
        name,
        flags,
        update_flags: node.update_flags,
        zone_id,
        data_ptr: node.data_ptr,
        model_index,
        area_partition,
        virtual_partition: None,
        parent_count,
        parent_array_ptr,
        child_count,
        child_array_ptr,
        active_bbox: ActiveBoundingBox::Node,
        node_bbox: node.node_bbox,
        model_bbox: node.model_bbox,
        child_bbox: node.child_bbox,
        field192: 0,
        field196: 0,
        field200: 0,
        field204: 0,
        node_class,
        offset: 0, // to be filled in by read loop
    })
}

const CAMERA_NAME: Ascii<36> = Ascii::node_name("camera1");

fn assert_camera(node: &NodeRcC, offset: usize) -> Result<()> {
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
    Ok(())
}

const DISPLAY_NAME: Ascii<36> = Ascii::node_name("display");

fn assert_display(node: &NodeRcC, offset: usize) -> Result<()> {
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
    Ok(())
}

fn assert_empty(node: &NodeRcC, offset: usize) -> Result<()> {
    // chk!(offset, node.name == )?;
    // chk!(offset, node.flags == )?;
    chk!(offset, node.field040 == 0)?;
    // chk!(offset, node.update_flags == 0)?; // 5, 6
    // chk!(offset, node.zone_id == ZONE_ALWAYS)?;
    chk!(offset, node.node_class == NodeClass::Empty)?;
    chk!(offset, node.data_ptr == Ptr::NULL)?;
    chk!(offset, node.model_index == -1)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;
    chk!(offset, node.area_partition == AreaPartitionC::DEFAULT)?;
    // this is actually important for the later code
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

fn assert_light(node: &NodeRcC, offset: usize) -> Result<()> {
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
    Ok(())
}

fn assert_lod(node: &NodeRcC, offset: usize) -> Result<()> {
    // chk!(offset, node.name == )?;
    // chk!(offset, node.flags == )?;
    chk!(offset, node.field040 == 0)?;
    // chk!(offset, node.update_flags == 0)?; // 0, 4
    // chk!(offset, node.zone_id == ZONE_ALWAYS)?;
    chk!(offset, node.node_class == NodeClass::Lod)?;
    chk!(offset, node.data_ptr != Ptr::NULL)?;
    chk!(offset, node.model_index == -1)?;
    chk!(offset, node.environment_data == Ptr::NULL)?;
    chk!(offset, node.action_priority == 1)?;
    chk!(offset, node.action_callback == Ptr::NULL)?;
    chk!(offset, node.area_partition == AreaPartitionC::DEFAULT)?;
    // chk!(offset, node.parent_count == 0)?;
    // chk!(offset, node.parent_array_ptr == Ptr::NULL)?;
    // chk!(offset, node.child_count == 0)?;
    // chk!(offset, node.child_array_ptr == Ptr::NULL)?;
    chk!(offset, node.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, node.bbox_diag == 0.0)?;
    // chk!(offset, node.node_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.model_bbox == BoundingBox::EMPTY)?;
    // chk!(offset, node.child_bbox == BoundingBox::EMPTY)?;
    chk!(offset, node.activation_ptr == Ptr::NULL)?;
    Ok(())
}

fn assert_object3d(node: &NodeRcC, offset: usize) -> Result<()> {
    // chk!(offset, node.name == )?;
    // chk!(offset, node.flags == )?;
    chk!(offset, node.field040 == 0)?;
    // chk!(offset, node.update_flags == 0)?; // 4
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
    Ok(())
}

const WINDOW_NAME: Ascii<36> = Ascii::node_name("window1");

fn assert_window(node: &NodeRcC, offset: usize) -> Result<()> {
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
    Ok(())
}

const WORLD_NAME: Ascii<36> = Ascii::node_name("world1");

fn assert_world(node: &NodeRcC, offset: usize) -> Result<()> {
    chk!(offset, node.name == WORLD_NAME)?;
    // chk!(offset, node.flags == )?;
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
    Ok(())
}
