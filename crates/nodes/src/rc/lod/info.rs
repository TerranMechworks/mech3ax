use crate::flags::NodeBitFlags;
use crate::rc::node::{NodeVariantLodRc, NodeVariantRc, NodeVariantsRc};
use crate::types::ZONE_DEFAULT;
use mech3ax_api_types::nodes::rc::Lod;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_common::{assert_len, assert_that, bool_c, Result};

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    | NodeBitFlags::ACTIVE.bits()
    | NodeBitFlags::ALTITUDE_SURFACE.bits()
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
    | NodeBitFlags::UNK08.bits()
    // | NodeBitFlags::HAS_MESH.bits()
    | NodeBitFlags::UNK10.bits()
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

pub fn assert_variants(node: NodeVariantsRc, offset: usize) -> Result<NodeVariantRc> {
    // cannot assert name
    let const_flags = node.flags & !VARIABLE_FLAGS;
    assert_that!("lod flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    if node.flags.contains(NodeBitFlags::UNK08) {
        let has_10 = node.flags.contains(NodeBitFlags::UNK10);
        assert_that!("lod flags 10", has_10 == true, offset + 36)?;
    }
    // zero040 (40) already asserted
    // there's six cases where unk044 is 0, so we'll use a dirty hack
    if BORKED_UNK044.contains(&node.data_ptr) {
        assert_that!("lod field 044", node.unk044 == 0, offset + 44)?;
    } else {
        assert_that!("lod field 044", node.unk044 == 4, offset + 44)?;
    }
    if node.zone_id != ZONE_DEFAULT {
        assert_that!("lod zone id", 0 <= node.zone_id <= 80, offset + 48)?;
    }
    // node_type (52) already asserted
    assert_that!("lod data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("lod mesh index", node.mesh_index == -1, offset + 60)?;
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
    let has_parent = assert_that!("parent count", bool node.parent_count, offset + 84)?;
    // parent_array_ptr (88) already asserted
    // children_count (92) is variable
    // children_array_ptr (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    // zero112 (112) already asserted
    assert_that!(
        "lod bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!("lod bbox 3", node.unk164 == node.unk116, offset + 164)?;
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
        unk116: node.unk116,
    }))
}

pub fn make_variants(lod: &Lod) -> Result<NodeVariantsRc> {
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
        zone_id: lod.zone_id,
        data_ptr: lod.data_ptr,
        mesh_index: -1,
        area_partition: None,
        parent_count: bool_c!(lod.parent.is_some()),
        parent_array_ptr: lod.parent_array_ptr,
        children_count,
        children_array_ptr: lod.children_array_ptr,
        unk116: lod.unk116,
        unk140: BoundingBox::EMPTY,
        unk164: lod.unk116,
    })
}
