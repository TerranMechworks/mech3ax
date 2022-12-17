use crate::cs::node::{NodeVariantCs, NodeVariantsCs};
use crate::flags::NodeBitFlagsCs;
use crate::types::ZONE_DEFAULT;
use mech3ax_api_types::nodes::cs::World;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_common::{assert_that, Result};

const WORLD_NAME: &str = "world1";

const WORLD_FLAGS: NodeBitFlagsCs = NodeBitFlagsCs::from_bits_truncate(
    NodeBitFlagsCs::UNK02.bits()
        | NodeBitFlagsCs::UNK03.bits()
        | NodeBitFlagsCs::UNK04.bits()
        | NodeBitFlagsCs::UNK19.bits()
        | NodeBitFlagsCs::UNK24.bits(),
);

pub fn assert_variants(node: NodeVariantsCs, offset: u32) -> Result<NodeVariantCs> {
    assert_that!("world name", &node.name == WORLD_NAME, offset + 0)?;
    assert_that!("world flags", node.flags == WORLD_FLAGS, offset + 36)?;
    assert_that!("world field 040", node.unk040 == 0x00000000, offset + 40)?;
    assert_that!("world field 044", node.unk044 == 0x00000000, offset + 44)?;
    assert_that!("world zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;
    // node_type (52) already asserted
    // data_ptr (56) already asserted
    assert_that!("world mesh index", node.mesh_index == -1, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    assert_that!(
        "world area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("world has parent", node.has_parent == false, offset + 84)?;
    // parent_array_ptr (88) already asserted
    assert_that!("world children count", 1 <= node.children_count <= 128, offset + 92)?;
    // children_array_ptr (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    assert_that!("world field 112", node.unk112 == 0, offset + 112)?;
    assert_that!(
        "world bbox 1",
        node.unk116 == BoundingBox::EMPTY,
        offset + 116
    )?;
    assert_that!(
        "world bbox 2",
        node.unk140 == BoundingBox::EMPTY,
        offset + 140
    )?;
    assert_that!(
        "world bbox 3",
        node.unk164 == BoundingBox::EMPTY,
        offset + 164
    )?;
    // zero188 (188) already asserted
    // zero192 (192) already asserted
    assert_that!("world field 196", node.unk196 == 0, offset + 196)?;
    // zero200 (200) already asserted
    // zero204 (204) already asserted
    Ok(NodeVariantCs::World {
        data_ptr: node.data_ptr,
        children_count: node.children_count,
        children_array_ptr: node.children_array_ptr,
    })
}

pub fn make_variants(world: &World) -> Result<NodeVariantsCs> {
    // let children_count = assert_len!(u16, world.children.len(), "world children")?;
    let children_count = world.children_count;
    Ok(NodeVariantsCs {
        name: WORLD_NAME.to_string(),
        flags: WORLD_FLAGS,
        unk040: 0x00000000,
        unk044: 0x00000000,
        zone_id: ZONE_DEFAULT,
        data_ptr: world.data_ptr,
        mesh_index: -1,
        area_partition: None,
        has_parent: false,
        parent_array_ptr: 0,
        children_count,
        children_array_ptr: world.children_array_ptr,
        unk112: 0,
        unk116: BoundingBox::EMPTY,
        unk140: BoundingBox::EMPTY,
        unk164: BoundingBox::EMPTY,
        unk196: 0,
    })
}
