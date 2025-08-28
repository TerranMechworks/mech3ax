use crate::cs::node::{NodeVariantCs, NodeVariantLodCs, NodeVariantsCs};
use crate::flags::NodeBitFlagsCs;
use mech3ax_api_types::nodes::cs::Lod;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_common::{assert_len, assert_that, Result};

const ALWAYS_PRESENT: NodeBitFlagsCs = NodeBitFlagsCs::from_bits_truncate(
    NodeBitFlagsCs::UNK02.bits()
        | NodeBitFlagsCs::UNK08.bits()
        | NodeBitFlagsCs::UNK10.bits()
        | NodeBitFlagsCs::UNK19.bits()
        | NodeBitFlagsCs::UNK24.bits()
        | NodeBitFlagsCs::UNK25.bits(),
);
const VARIABLE_FLAGS: NodeBitFlagsCs = NodeBitFlagsCs::from_bits_truncate(
    0 | NodeBitFlagsCs::UNK03.bits()
        | NodeBitFlagsCs::UNK04.bits()
        | NodeBitFlagsCs::UNK07.bits()
        | 0,
);

pub(crate) fn assert_variants(node: NodeVariantsCs, offset: usize) -> Result<NodeVariantCs> {
    // can't assert name
    let const_flags = node.flags.mask_not(VARIABLE_FLAGS);
    assert_that!("lod flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    // unk040 (40) is variable
    assert_that!("lod field 044", node.unk044 == 0x00000001, offset + 44)?;
    // zone_id (48) is variable
    // node_type (52) already asserted
    // data_ptr (56) already asserted
    assert_that!("lod mesh index", node.mesh_index == -1, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "lod area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("lod has parent", node.has_parent == true, offset + 84)?;
    // parent_array_ptr (88) already asserted
    assert_that!(
        "lod children count",
        1 <= node.children_count <= 64,
        offset + 92
    )?;
    // children_array_ptr (96) already asserted
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
    // unk164 (164) is variable
    // zero188 (188) already asserted
    // zero192 (192) already asserted
    assert_that!("lod field 196", node.unk196 == 160, offset + 196)?;
    // zero200 (200) already asserted
    // zero204 (204) already asserted
    Ok(NodeVariantCs::Lod(NodeVariantLodCs {
        name: node.name,
        flags_unk03: node.flags.contains(NodeBitFlagsCs::UNK03),
        flags_unk04: node.flags.contains(NodeBitFlagsCs::UNK04),
        flags_unk07: node.flags.contains(NodeBitFlagsCs::UNK07),
        unk040: node.unk040,
        zone_id: node.zone_id,
        data_ptr: node.data_ptr,
        parent_array_ptr: node.parent_array_ptr,
        children_count: node.children_count,
        children_array_ptr: node.children_array_ptr,
        unk164: node.unk164,
    }))
}

pub(crate) fn make_variants(lod: &Lod) -> Result<NodeVariantsCs> {
    let children_count = assert_len!(u16, lod.children.len(), "lod children")?;
    let mut flags = ALWAYS_PRESENT;
    if lod.flags_unk03 {
        flags |= NodeBitFlagsCs::UNK03;
    }
    if lod.flags_unk04 {
        flags |= NodeBitFlagsCs::UNK04;
    }
    if lod.flags_unk07 {
        flags |= NodeBitFlagsCs::UNK07;
    }
    Ok(NodeVariantsCs {
        name: lod.name.to_string(),
        flags,
        unk040: lod.unk040,
        unk044: 0x00000001,
        zone_id: lod.zone_id,
        data_ptr: lod.data_ptr,
        mesh_index: -1,
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
