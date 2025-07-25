use crate::flags::NodeBitFlags;
use crate::rc::node::{NodeVariantLodRc, NodeVariantRc, NodeVariantsRc};
use crate::types::ZONE_ALWAYS;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_api_types::nodes::rc::Lod;
use mech3ax_common::{Result, assert_len, assert_that};

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    | NodeBitFlags::ACTIVE.bits()
    | NodeBitFlags::ALTITUDE_SURFACE.bits()
    | NodeBitFlags::INTERSECT_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_BBOX.bits()
    // | NodeBitFlags::LANDMARK.bits()
    // | NodeBitFlags::BBOX_NODE.bits()
    // | NodeBitFlags::BBOX_MODEL.bits()
    // | NodeBitFlags::BBOX_CHILD.bits()
    // | NodeBitFlags::TERRAIN.bits()
    // | NodeBitFlags::CAN_MODIFY.bits()
    // | NodeBitFlags::CLIP_TO.bits()
    | NodeBitFlags::TREE_VALID.bits()
    | NodeBitFlags::ID_ZONE_CHECK.bits()
    // | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);
const VARIABLE_FLAGS: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    // | NodeBitFlags::ACTIVE.bits()
    // | NodeBitFlags::ALTITUDE_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_BBOX.bits()
    // | NodeBitFlags::LANDMARK.bits()
    | NodeBitFlags::BBOX_NODE.bits()
    // | NodeBitFlags::BBOX_MODEL.bits()
    | NodeBitFlags::BBOX_CHILD.bits()
    // | NodeBitFlags::TERRAIN.bits()
    // | NodeBitFlags::CAN_MODIFY.bits()
    // | NodeBitFlags::CLIP_TO.bits()
    // | NodeBitFlags::TREE_VALID.bits()
    // | NodeBitFlags::ID_ZONE_CHECK.bits()
    // | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);

const BORKED_UNK044: &[u32; 6] = &[
    0x0189AF70, 0x018B16A0, 0x018C2C30, 0x018C54C0, 0x018CC320, 0x018F4930,
];

pub(crate) fn assert_variants(node: NodeVariantsRc, offset: usize) -> Result<NodeVariantRc> {
    // cannot assert name
    let const_flags = node.flags.mask_not(VARIABLE_FLAGS);
    assert_that!("lod flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    if node.flags.contains(NodeBitFlags::BBOX_NODE) {
        let has_10 = node.flags.contains(NodeBitFlags::BBOX_CHILD);
        assert_that!("lod flags 10", has_10 == true, offset + 36)?;
    }
    // zero040 (40) already asserted
    // there's six cases where unk044 is 0, so we'll use a dirty hack
    if BORKED_UNK044.contains(&node.data_ptr) {
        assert_that!("lod field 044", node.unk044 == 0, offset + 44)?;
    } else {
        assert_that!("lod field 044", node.unk044 == 4, offset + 44)?;
    }
    assert_that!("lod zone id", node.zone_id >= ZONE_ALWAYS, offset + 48)?;
    // node_type (52) already asserted
    assert_that!("lod data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("lod model index", node.model_index == -1, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "display area partition",
        node.area_partition == None,
        offset + 76
    )?;
    // parent_count (84) is variable
    // can only have one parent
    assert_that!("parent count", node.parent_count in [0, 1], offset + 84)?;
    let has_parent = node.parent_count > 0;
    // parent_array_ptr (88) already asserted
    // children_count (92) is variable
    // children_array_ptr (96) already asserted
    // bbox_mid (100) already asserted
    // bbox_diag (112) already asserted
    // node_bbox (116) is variable
    assert_that!(
        "lod model bbox",
        node.model_bbox == BoundingBox::EMPTY,
        offset + 140
    )?;
    // child_bbox (164) is variable... technically, it's always the same as the
    // node_bbox, but I've decided to expose both
    // zero188 (188) already asserted
    Ok(NodeVariantRc::Lod(NodeVariantLodRc {
        name: node.name,
        flags: node.flags,
        zone_id: node.zone_id,
        data_ptr: node.data_ptr,
        has_parent,
        parent_array_ptr: node.parent_array_ptr,
        children_count: node.children_count,
        children_array_ptr: node.children_array_ptr,
        node_bbox: node.node_bbox,
        child_bbox: node.child_bbox,
    }))
}

pub(crate) fn make_variants(lod: &Lod) -> Result<NodeVariantsRc> {
    let children_count = assert_len!(u32, lod.children.len(), "lod children")?;
    let unk044 = if BORKED_UNK044.contains(&lod.data_ptr) {
        0
    } else {
        4
    };
    Ok(NodeVariantsRc {
        name: lod.name.clone(),
        flags: NodeBitFlags::from(&lod.flags),
        unk044,
        zone_id: lod.zone_id as i8, // TODO
        data_ptr: lod.data_ptr,
        model_index: -1,
        area_partition: None,
        parent_count: if lod.parent.is_some() { 1 } else { 0 },
        parent_array_ptr: lod.parent_array_ptr,
        children_count,
        children_array_ptr: lod.children_array_ptr,
        node_bbox: lod.node_bbox,
        model_bbox: BoundingBox::EMPTY,
        child_bbox: lod.child_bbox,
    })
}
