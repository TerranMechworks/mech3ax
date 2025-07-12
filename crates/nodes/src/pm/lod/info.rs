use crate::flags::NodeBitFlags;
use crate::pm::node::{NodeVariantLodPm, NodeVariantPm, NodeVariantsPm};
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_api_types::nodes::pm::Lod;
use mech3ax_common::{Result, assert_len, assert_that};

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    | NodeBitFlags::ACTIVE.bits()
    | NodeBitFlags::ALTITUDE_SURFACE.bits()
    | NodeBitFlags::INTERSECT_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_BBOX.bits()
    // | NodeBitFlags::LANDMARK.bits()
    | NodeBitFlags::BBOX_NODE.bits()
    // | NodeBitFlags::BBOX_MODEL.bits()
    | NodeBitFlags::BBOX_CHILD.bits()
    // | NodeBitFlags::TERRAIN.bits()
    // | NodeBitFlags::CAN_MODIFY.bits()
    // | NodeBitFlags::CLIP_TO.bits()
    | NodeBitFlags::TREE_VALID.bits()
    | NodeBitFlags::ID_ZONE_CHECK.bits()
    | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);

pub(crate) fn assert_variants(
    node: NodeVariantsPm,
    offset: usize,
    mesh_index_is_ptr: bool,
) -> Result<NodeVariantPm> {
    // cannot assert name
    assert_that!("lod flags", node.flags == ALWAYS_PRESENT, offset + 36)?;
    // zero040 (40) already asserted
    assert_that!("lod field 044", node.unk044 == 1, offset + 44)?;
    // zone_id (48) is variable
    // node_type (52) already asserted
    assert_that!("lod data ptr", node.data_ptr != 0, offset + 56)?;
    if mesh_index_is_ptr {
        assert_that!("lod mesh index", node.mesh_index == 0, offset + 60)?;
    } else {
        assert_that!("lod mesh index", node.mesh_index == -1, offset + 60)?;
    }
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "lod area partition",
        node.area_partition == None,
        offset + 76
    )?;
    // must have one parent
    assert_that!("lod has parent", node.has_parent == true, offset + 84)?;
    // parent_array_ptr (88) already asserted
    // always has at least one child
    assert_that!("lod children count", 1 <= node.children_count <= 32, offset + 86)?;
    // children_array_ptr (92) already asserted
    // zero096 (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    assert_that!("lod field 112", node.unk112 == 2, offset + 112)?;
    assert_that!(
        "lod bbox 1",
        node.unk116 == BoundingBox::EMPTY,
        offset + 116
    )?;
    assert_that!(
        "lod bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!(
        "lod bbox 3",
        node.unk164 != BoundingBox::EMPTY,
        offset + 164
    )?;
    // zero188 (188) already asserted
    // zero192 (192) already asserted
    assert_that!("lod field 196", node.unk196 == 160, offset + 196)?;
    // zero200 (200) already asserted
    // zero204 (204) already asserted
    Ok(NodeVariantPm::Lod(NodeVariantLodPm {
        name: node.name,
        flags: node.flags,
        zone_id: node.zone_id,
        data_ptr: node.data_ptr,
        parent_array_ptr: node.parent_array_ptr,
        children_count: node.children_count,
        children_array_ptr: node.children_array_ptr,
        unk164: node.unk164,
    }))
}

pub(crate) fn make_variants(lod: &Lod, mesh_index_is_ptr: bool) -> Result<NodeVariantsPm> {
    let children_count = assert_len!(u16, lod.children.len(), "lod children")?;
    Ok(NodeVariantsPm {
        name: lod.name.clone(),
        flags: ALWAYS_PRESENT,
        unk044: 1,
        zone_id: lod.zone_id,
        data_ptr: lod.data_ptr,
        mesh_index: if mesh_index_is_ptr { 0 } else { -1 },
        area_partition: None,
        has_parent: true,
        parent_array_ptr: lod.parent_array_ptr,
        children_count,
        children_array_ptr: lod.children_array_ptr,
        unk112: 2,
        unk116: BoundingBox::EMPTY,
        unk140: BoundingBox::EMPTY,
        unk164: lod.unk164,
        unk196: 160,
    })
}
