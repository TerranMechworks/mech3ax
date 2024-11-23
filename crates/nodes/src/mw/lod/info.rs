use crate::flags::NodeBitFlags;
use crate::mw::node::{NodeVariantLodMw, NodeVariantMw, NodeVariantsMw};
use crate::types::ZONE_DEFAULT;
use mech3ax_api_types::nodes::mw::Lod;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_common::{assert_len, assert_that, Result};

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    | NodeBitFlags::ACTIVE.bits()
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
    | NodeBitFlags::TREE_VALID.bits()
    | NodeBitFlags::ID_ZONE_CHECK.bits()
    // | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);
const VARIABLE_FLAGS: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    // | NodeBitFlags::ACTIVE.bits()
    | NodeBitFlags::ALTITUDE_SURFACE.bits()
    | NodeBitFlags::INTERSECT_SURFACE.bits()
    | NodeBitFlags::INTERSECT_BBOX.bits()
    // | NodeBitFlags::LANDMARK.bits()
    // | NodeBitFlags::UNK08.bits()
    // | NodeBitFlags::HAS_MESH.bits()
    // | NodeBitFlags::UNK10.bits()
    | NodeBitFlags::TERRAIN.bits()
    // | NodeBitFlags::CAN_MODIFY.bits()
    // | NodeBitFlags::CLIP_TO.bits()
    // | NodeBitFlags::TREE_VALID.bits()
    // | NodeBitFlags::ID_ZONE_CHECK.bits()
    | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);

pub fn assert_variants(node: NodeVariantsMw, offset: usize) -> Result<NodeVariantMw> {
    // cannot assert name
    let const_flags = node.flags & !VARIABLE_FLAGS;
    assert_that!("lod flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    // zero040 (40) already asserted
    assert_that!("lod field 044", node.unk044 == 1, offset + 44)?;
    if node.zone_id != ZONE_DEFAULT {
        assert_that!("lod zone id", 1 <= node.zone_id <= 80, offset + 48)?;
    }
    // node_type (52) already asserted
    assert_that!("lod data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("lod mesh index", node.mesh_index == -1, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    // area_partition (76) is variable
    // must have one parent
    assert_that!("lod has parent", node.has_parent == true, offset + 84)?;
    // parent_array_ptr (88) already asserted
    // always has at least one child
    assert_that!("lod children count", 1 <= node.children_count <= 32, offset + 92)?;
    // children_array_ptr (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    // zero112 (112) already asserted
    assert_that!(
        "lod bbox 1",
        node.unk116 != BoundingBox::EMPTY,
        offset + 116
    )?;
    assert_that!(
        "lod bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!("lod bbox 3", node.unk164 == node.unk116, offset + 164)?;
    // zero188 (188) already asserted
    // zero192 (192) already asserted
    assert_that!("lod field 196", node.unk196 == 160, offset + 196)?;
    // zero200 (200) already asserted
    // zero204 (204) already asserted
    Ok(NodeVariantMw::Lod(NodeVariantLodMw {
        name: node.name,
        flags: node.flags,
        zone_id: node.zone_id,
        data_ptr: node.data_ptr,
        area_partition: node.area_partition,
        parent_array_ptr: node.parent_array_ptr,
        children_count: node.children_count,
        children_array_ptr: node.children_array_ptr,
        unk116: node.unk116,
    }))
}

pub fn make_variants(lod: &Lod) -> Result<NodeVariantsMw> {
    let children_count = assert_len!(u32, lod.children.len(), "lod children")?;
    Ok(NodeVariantsMw {
        name: lod.name.clone(),
        flags: NodeBitFlags::from(&lod.flags),
        unk044: 1,
        zone_id: lod.zone_id,
        data_ptr: lod.data_ptr,
        mesh_index: -1,
        area_partition: lod.area_partition,
        has_parent: true,
        parent_array_ptr: lod.parent_array_ptr,
        children_count,
        children_array_ptr: lod.children_array_ptr,
        unk116: lod.unk116,
        unk140: BoundingBox::EMPTY,
        unk164: lod.unk116,
        unk196: 160,
    })
}
