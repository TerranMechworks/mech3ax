use crate::cs::node::{NodeVariantCs, NodeVariantsCs};
use crate::flags::NodeBitFlagsCs;
use crate::types::ZONE_DEFAULT;
use mech3ax_api_types::nodes::cs::Window;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_common::{assert_that, assert_with_msg, Result};

pub(crate) const WINDOW_NAME: &str = "window1";
pub(crate) const SPYGLASS_NAME: &str = "sgwin";

const WINDOW_FLAGS: NodeBitFlagsCs = NodeBitFlagsCs::from_bits_truncate(
    NodeBitFlagsCs::UNK02.bits()
        | NodeBitFlagsCs::UNK03.bits()
        | NodeBitFlagsCs::UNK04.bits()
        | NodeBitFlagsCs::UNK19.bits()
        | NodeBitFlagsCs::UNK24.bits(),
);

pub(crate) fn assert_variants(node: NodeVariantsCs, offset: usize) -> Result<NodeVariantCs> {
    let spyglass = match node.name.as_str() {
        WINDOW_NAME => Ok(false),
        SPYGLASS_NAME => Ok(true),
        other => Err(assert_with_msg!(
            "Expected `camera1` or `spyglass`, but was `{}` (at {})",
            other,
            offset + 0
        )),
    }?;
    assert_that!("window flags", node.flags == WINDOW_FLAGS, offset + 36)?;
    assert_that!("window field 040", node.unk040 == 0x00000000, offset + 40)?;
    assert_that!("window field 044", node.unk044 == 0x00000000, offset + 44)?;
    assert_that!("window zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;
    // node_type (52) already asserted
    // data_ptr (56) already asserted
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
    assert_that!("window field 112", node.unk112 == 0, offset + 112)?;
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
    // zero192 (192) already asserted
    assert_that!("window field 196", node.unk196 == 0, offset + 196)?;
    // zero200 (200) already asserted
    // zero204 (204) already asserted
    Ok(NodeVariantCs::Window {
        data_ptr: node.data_ptr,
        spyglass,
    })
}

pub(crate) fn make_variants(window: &Window) -> NodeVariantsCs {
    NodeVariantsCs {
        name: window.name.clone(),
        flags: WINDOW_FLAGS,
        unk040: 0x00000000,
        unk044: 0x00000000,
        zone_id: ZONE_DEFAULT,
        data_ptr: window.data_ptr,
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
