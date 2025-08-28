use crate::flags::NodeBitFlags;
use crate::mw::node::{NodeVariantMw, NodeVariantsMw};
use crate::types::ZONE_DEFAULT;
use mech3ax_api_types::nodes::mw::World;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_common::{assert_len, assert_that, Result};

pub(crate) const WORLD_NAME: &str = "world1";

pub(crate) fn assert_variants(node: NodeVariantsMw, offset: usize) -> Result<NodeVariantMw> {
    assert_that!("world name", node.name eq WORLD_NAME, offset + 0)?;
    assert_that!(
        "world flags",
        node.flags == NodeBitFlags::DEFAULT,
        offset + 36
    )?;
    // zero040 (40) already asserted
    assert_that!("world field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("world zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;
    // node_type (52) already asserted
    assert_that!("world data ptr", node.data_ptr != 0, offset + 56)?;
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
    assert_that!("world children count", 1 <= node.children_count <= 64, offset + 92)?;
    // children_array_ptr (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    // zero112 (112) already asserted
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
    Ok(NodeVariantMw::World {
        data_ptr: node.data_ptr,
        children_count: node.children_count,
        children_array_ptr: node.children_array_ptr,
    })
}

pub(crate) fn make_variants(world: &World) -> Result<NodeVariantsMw> {
    let children_count = assert_len!(u32, world.children.len(), "world children")?;
    Ok(NodeVariantsMw {
        name: WORLD_NAME.to_owned(),
        flags: NodeBitFlags::DEFAULT,
        unk044: 0,
        zone_id: ZONE_DEFAULT,
        data_ptr: world.data_ptr,
        mesh_index: -1,
        area_partition: None,
        has_parent: false,
        parent_array_ptr: 0,
        children_count,
        children_array_ptr: world.children_array_ptr,
        unk116: BoundingBox::EMPTY,
        unk140: BoundingBox::EMPTY,
        unk164: BoundingBox::EMPTY,
        unk196: 0,
    })
}
