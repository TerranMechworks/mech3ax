use crate::flags::NodeBitFlags;
use crate::rc::node::{NodeVariantRc, NodeVariantsRc};
use crate::types::ZONE_ALWAYS;
use mech3ax_api_types::nodes::{BoundingBox, Window};
use mech3ax_common::{assert_that, Result};

const WINDOW_NAME: &str = "window1";

pub(crate) fn assert_variants(node: NodeVariantsRc, offset: usize) -> Result<NodeVariantRc> {
    assert_that!("window name", node.name eq WINDOW_NAME, offset + 0)?;
    assert_that!(
        "window flags",
        node.flags == NodeBitFlags::DEFAULT,
        offset + 36
    )?;
    // zero040 (40) already asserted
    assert_that!("window field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("window zone id", node.zone_id == ZONE_ALWAYS, offset + 48)?;
    // node_type (52) already asserted
    assert_that!("window data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("window model index", node.model_index == -1, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "window area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("window parent count", node.parent_count == 0, offset + 84)?;
    // parent_array_ptr (88) already asserted
    assert_that!(
        "window children count",
        node.children_count == 0,
        offset + 92
    )?;
    // children_array_ptr (96) already asserted
    // bbox_mid (100) already asserted
    // bbox_diag (112) already asserted
    assert_that!(
        "window node bbox",
        node.node_bbox == BoundingBox::EMPTY,
        offset + 116
    )?;
    assert_that!(
        "window model  bbox",
        node.model_bbox == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!(
        "window child bbox",
        node.child_bbox == BoundingBox::EMPTY,
        offset + 164
    )?;
    // zero188 (188) already asserted
    Ok(NodeVariantRc::Window {
        data_ptr: node.data_ptr,
    })
}

pub(crate) fn make_variants(window: &Window) -> NodeVariantsRc {
    NodeVariantsRc {
        name: WINDOW_NAME.to_owned(),
        flags: NodeBitFlags::DEFAULT,
        unk044: 0,
        zone_id: ZONE_ALWAYS,
        data_ptr: window.data_ptr,
        model_index: -1,
        area_partition: None,
        parent_count: 0,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        node_bbox: BoundingBox::EMPTY,
        model_bbox: BoundingBox::EMPTY,
        child_bbox: BoundingBox::EMPTY,
    }
}
