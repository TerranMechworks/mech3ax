use crate::flags::NodeBitFlags;
use crate::rc::node::{NodeVariantRc, NodeVariantsRc};
use crate::types::ZONE_ALWAYS;
use mech3ax_api_types::nodes::rc::Empty;
use mech3ax_common::{assert_that, Result};

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    | NodeBitFlags::ACTIVE.bits()
    // | NodeBitFlags::ALTITUDE_SURFACE.bits()
    | NodeBitFlags::INTERSECT_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_BBOX.bits()
    // | NodeBitFlags::LANDMARK.bits()
    // | NodeBitFlags::UNK08.bits()
    // | NodeBitFlags::HAS_MESH.bits()
    // | NodeBitFlags::UNK10.bits()
    // | NodeBitFlags::TERRAIN.bits()
    // | NodeBitFlags::CAN_MODIFY.bits()
    // | NodeBitFlags::CLIP_TO.bits()
    | NodeBitFlags::TREE_VALID.bits()
    | NodeBitFlags::ID_ZONE_CHECK.bits()
    | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);
const VARIABLE_FLAGS: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    // | NodeBitFlags::ACTIVE.bits()
    | NodeBitFlags::ALTITUDE_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_SURFACE.bits()
    | NodeBitFlags::INTERSECT_BBOX.bits()
    // | NodeBitFlags::LANDMARK.bits()
    | NodeBitFlags::UNK08.bits()
    // | NodeBitFlags::HAS_MESH.bits()
    | NodeBitFlags::UNK10.bits()
    // | NodeBitFlags::TERRAIN.bits()
    | NodeBitFlags::CAN_MODIFY.bits()
    | NodeBitFlags::CLIP_TO.bits()
    // | NodeBitFlags::TREE_VALID.bits()
    // | NodeBitFlags::ID_ZONE_CHECK.bits()
    // | NodeBitFlags::UNK25.bits()
    | NodeBitFlags::UNK28.bits()
    | 0,
);

pub(crate) fn assert_variants(node: NodeVariantsRc, offset: usize) -> Result<NodeVariantRc> {
    // cannot assert name
    let const_flags = node.flags.mask_not(VARIABLE_FLAGS);
    assert_that!("empty flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    // zero040 (40) already asserted
    assert_that!("empty field 044", node.unk044 in [5, 6], offset + 44)?;
    assert_that!("empty zone id", node.zone_id >= ZONE_ALWAYS, offset + 48)?;
    // node_type (52) already asserted
    assert_that!("empty data ptr", node.data_ptr == 0, offset + 56)?;
    assert_that!("empty mesh index", node.mesh_index == -1, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "empty area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("empty parent count", node.parent_count == 0, offset + 84)?;
    // parent_array_ptr (88) already asserted
    assert_that!(
        "empty children count",
        node.children_count == 0,
        offset + 92
    )?;
    // children_array_ptr (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    // zero112 (112) already asserted
    // unk116 (116) is variable
    // unk140 (140) is variable
    // unk164 (164) is variable
    // zero188 (188) already asserted
    Ok(NodeVariantRc::Empty(Empty {
        name: node.name,
        flags: node.flags.into(),
        unk044: node.unk044,
        zone_id: node.zone_id as u32, // TODO
        unk116: node.unk116,
        unk140: node.unk140,
        unk164: node.unk164,
        parent: 0, // to be filled in via the index
    }))
}

pub(crate) fn make_variants(empty: &Empty) -> NodeVariantsRc {
    NodeVariantsRc {
        name: empty.name.clone(),
        flags: NodeBitFlags::from(&empty.flags),
        unk044: empty.unk044,
        zone_id: empty.zone_id as i8, // TODO
        data_ptr: 0,
        mesh_index: -1,
        area_partition: None,
        parent_count: 0,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        unk116: empty.unk116,
        unk140: empty.unk140,
        unk164: empty.unk164,
    }
}
