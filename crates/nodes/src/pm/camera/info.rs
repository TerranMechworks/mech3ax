use crate::flags::NodeBitFlags;
use crate::pm::node::{NodeVariantPm, NodeVariantsPm};
use crate::types::ZONE_DEFAULT;
use mech3ax_api_types::nodes::{BoundingBox, Camera};
use mech3ax_common::{assert_that, Result};

const CAMERA_NAME: &str = "camera1";

pub fn assert_variants(node: NodeVariantsPm, offset: usize) -> Result<NodeVariantPm> {
    assert_that!("camera name", node.name eq CAMERA_NAME, offset + 0)?;
    assert_that!(
        "camera flags",
        node.flags == NodeBitFlags::DEFAULT,
        offset + 36
    )?;
    // zero040 (40) already asserted
    assert_that!("camera field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("camera zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;
    // node_type (52) already asserted
    assert_that!("camera data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("camera mesh index", node.mesh_index == -1, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "camera area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("camera has parent", node.has_parent == false, offset + 84)?;
    // parent_array_ptr (88) already asserted
    assert_that!(
        "camera children count",
        node.children_count == 0,
        offset + 86
    )?;
    // children_array_ptr (92) already asserted
    // zero096 (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    assert_that!("camera field 112", node.unk112 == 0, offset + 112)?;
    assert_that!(
        "camera bbox 1",
        node.unk116 == BoundingBox::EMPTY,
        offset + 116
    )?;
    assert_that!(
        "camera bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!(
        "camera bbox 3",
        node.unk164 == BoundingBox::EMPTY,
        offset + 164
    )?;
    // zero188 (188) already asserted
    // zero192 (192) already asserted
    assert_that!("camera field 196", node.unk196 == 0, offset + 196)?;
    // zero200 (200) already asserted
    // zero204 (204) already asserted
    Ok(NodeVariantPm::Camera {
        data_ptr: node.data_ptr,
    })
}

pub fn make_variants(camera: &Camera) -> NodeVariantsPm {
    NodeVariantsPm {
        name: CAMERA_NAME.to_owned(),
        flags: NodeBitFlags::DEFAULT,
        unk044: 0,
        zone_id: ZONE_DEFAULT,
        data_ptr: camera.data_ptr,
        mesh_index: -1,
        area_partition: None,
        has_parent: false,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        unk112: 0,
        unk116: BoundingBox::EMPTY,
        unk140: BoundingBox::EMPTY,
        unk164: BoundingBox::EMPTY,
        unk196: 0,
    }
}
