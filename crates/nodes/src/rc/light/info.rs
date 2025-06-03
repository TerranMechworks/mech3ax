use crate::flags::NodeBitFlags;
use crate::rc::node::{NodeVariantRc, NodeVariantsRc};
use crate::types::ZONE_ALWAYS;
use mech3ax_api_types::nodes::rc::Light;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_api_types::Vec3;
use mech3ax_common::{assert_that, Result};

const BBOX_LIGHT: BoundingBox = BoundingBox {
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
pub(crate) const LIGHT_NAME: &str = "sunlight";

pub(crate) fn assert_variants(node: NodeVariantsRc, offset: usize) -> Result<NodeVariantRc> {
    assert_that!("light name", node.name eq LIGHT_NAME, offset + 0)?;
    assert_that!(
        "light flags",
        node.flags == NodeBitFlags::DEFAULT | NodeBitFlags::BBOX_NODE,
        offset + 36
    )?;
    // zero040 (40) already asserted
    assert_that!("light field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("light zone id", node.zone_id == ZONE_ALWAYS, offset + 48)?;
    // node_type (52) already asserted
    assert_that!("light data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("light model index", node.model_index == -1, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "light area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("light parent count", node.parent_count == 0, offset + 84)?;
    // parent_array_ptr (88) already asserted
    assert_that!(
        "light children count",
        node.children_count == 0,
        offset + 92
    )?;
    // children_array_ptr (96) already asserted
    // bbox_mid (100) already asserted
    // bbox_diag (112) already asserted
    assert_that!(
        "light node bbox",
        node.node_bbox == BBOX_LIGHT,
        offset + 116
    )?;
    assert_that!(
        "light model bbox",
        node.model_bbox == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!(
        "light child bbox",
        node.child_bbox == BoundingBox::EMPTY,
        offset + 164
    )?;
    // zero188 (188) already asserted
    Ok(NodeVariantRc::Light {
        data_ptr: node.data_ptr,
    })
}

pub(crate) fn make_variants(light: &Light) -> NodeVariantsRc {
    NodeVariantsRc {
        name: LIGHT_NAME.to_owned(),
        flags: NodeBitFlags::DEFAULT | NodeBitFlags::BBOX_NODE,
        unk044: 0,
        zone_id: ZONE_ALWAYS,
        data_ptr: light.data_ptr,
        model_index: -1,
        area_partition: None,
        parent_count: 0,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        node_bbox: BBOX_LIGHT,
        model_bbox: BoundingBox::EMPTY,
        child_bbox: BoundingBox::EMPTY,
    }
}
