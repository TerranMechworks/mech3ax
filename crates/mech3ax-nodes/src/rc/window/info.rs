use crate::flags::NodeBitFlags;
use crate::rc::node::{NodeVariantRc, NodeVariantsRc};
use crate::types::ZONE_DEFAULT;
use mech3ax_api_types::nodes::rc::Window;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_common::{assert_that, Result};

pub const WINDOW_NAME: &str = "window1";

pub fn assert_variants(node: NodeVariantsRc, offset: u32) -> Result<NodeVariantRc> {
    assert_that!("window name", node.name eq WINDOW_NAME, offset + 0)?;
    assert_that!(
        "window flags",
        node.flags == NodeBitFlags::DEFAULT,
        offset + 36
    )?;
    // zero040 (40) already asserted
    assert_that!("window field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("window zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;
    // node_type (52) already asserted
    assert_that!("window data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("window mesh index", node.mesh_index == -1, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "window area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("window has parent", node.has_parent == false, offset + 84)?;
    // parent_array_ptr (88) already asserted
    assert_that!(
        "window children count",
        node.children_count == 0,
        offset + 92
    )?;
    // children_array_ptr (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    // zero112 (112) already asserted
    assert_that!(
        "window bbox 1",
        node.unk116 == BoundingBox::EMPTY,
        offset + 116
    )?;
    assert_that!(
        "window bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!(
        "window bbox 3",
        node.unk164 == BoundingBox::EMPTY,
        offset + 164
    )?;
    // zero188 (188) already asserted
    Ok(NodeVariantRc::Window {
        data_ptr: node.data_ptr,
    })
}

pub fn make_variants(window: &Window) -> NodeVariantsRc {
    NodeVariantsRc {
        name: WINDOW_NAME.to_owned(),
        flags: NodeBitFlags::DEFAULT,
        unk044: 0,
        zone_id: ZONE_DEFAULT,
        data_ptr: window.data_ptr,
        mesh_index: -1,
        area_partition: None,
        has_parent: false,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        unk116: BoundingBox::EMPTY,
        unk140: BoundingBox::EMPTY,
        unk164: BoundingBox::EMPTY,
    }
}
