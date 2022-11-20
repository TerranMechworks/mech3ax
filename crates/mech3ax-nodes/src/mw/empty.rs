use crate::flags::NodeBitFlags;
use crate::types::{NodeVariantMw, NodeVariantsMw, ZONE_DEFAULT};
use mech3ax_api_types::nodes::mw::Empty;
use mech3ax_common::{assert_that, Result};

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::BASE;
const NEVER_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    NodeBitFlags::LANDMARK.bits()
        | NodeBitFlags::HAS_MESH.bits()
        | NodeBitFlags::TERRAIN.bits()
        | NodeBitFlags::CAN_MODIFY.bits()
        | NodeBitFlags::CLIP_TO.bits(),
);

pub fn assert_variants(node: NodeVariantsMw, offset: u32) -> Result<NodeVariantMw> {
    // cannot assert name
    let const_flags = node.flags & (ALWAYS_PRESENT | NEVER_PRESENT);
    assert_that!("empty flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    // variable
    /*
    const ALTITUDE_SURFACE = 1 << 3;
    const INTERSECT_SURFACE = 1 << 4;
    const INTERSECT_BBOX = 1 << 5;
    const UNK08 = 1 << 8;
    const UNK10 = 1 << 10;
    const UNK25 = 1 << 25;
    const UNK28 = 1 << 28;
    */

    assert_that!("empty field 044", node.unk044 in [1, 3, 5, 7], offset + 56)?;
    assert_that!("empty zone id", node.zone_id in [1, ZONE_DEFAULT], offset + 56)?;
    assert_that!("empty data ptr", node.data_ptr == 0, offset + 56)?;
    assert_that!("empty mesh index", node.mesh_index == -1, offset + 60)?;
    assert_that!(
        "empty area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("empty has parent", node.has_parent == false, offset + 84)?;
    // parent array ptr is already asserted
    assert_that!(
        "empty children count",
        node.children_count == 0,
        offset + 92
    )?;
    // children array ptr is already asserted
    assert_that!("empty field 196", node.unk196 == 160, offset + 196)?;

    Ok(NodeVariantMw::Empty(Empty {
        name: node.name,
        flags: node.flags.into(),
        unk044: node.unk044,
        zone_id: node.zone_id,
        unk116: node.unk116,
        unk140: node.unk140,
        unk164: node.unk164,
        parent: 0,
    }))
}

pub fn make_variants(empty: &Empty) -> NodeVariantsMw {
    NodeVariantsMw {
        name: empty.name.clone(),
        flags: NodeBitFlags::from(&empty.flags),
        unk044: empty.unk044,
        zone_id: empty.zone_id,
        data_ptr: 0,
        mesh_index: -1,
        area_partition: None,
        has_parent: false,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        unk116: empty.unk116,
        unk140: empty.unk140,
        unk164: empty.unk164,
        unk196: 160,
    }
}

pub fn size() -> u32 {
    0
}
