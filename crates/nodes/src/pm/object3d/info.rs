use crate::flags::NodeBitFlags;
use crate::pm::node::{NodeVariantPm, NodeVariantsPm};
use mech3ax_api_types::nodes::pm::Object3d;
use mech3ax_common::{assert_len, assert_that, Result};

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    | NodeBitFlags::ACTIVE.bits()
    // | NodeBitFlags::ALTITUDE_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_SURFACE.bits()
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
    | NodeBitFlags::INTERSECT_SURFACE.bits()
    | NodeBitFlags::INTERSECT_BBOX.bits()
    | NodeBitFlags::LANDMARK.bits()
    | NodeBitFlags::UNK08.bits()
    | NodeBitFlags::HAS_MESH.bits()
    | NodeBitFlags::UNK10.bits()
    | NodeBitFlags::TERRAIN.bits()
    | NodeBitFlags::CAN_MODIFY.bits()
    | NodeBitFlags::CLIP_TO.bits()
    // | NodeBitFlags::TREE_VALID.bits()
    // | NodeBitFlags::ID_ZONE_CHECK.bits()
    // | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);

#[allow(clippy::collapsible_else_if)]
pub(crate) fn assert_variants(
    node: NodeVariantsPm,
    offset: usize,
    mesh_index_is_ptr: bool,
) -> Result<NodeVariantPm> {
    // cannot assert name
    let const_flags = node.flags.mask_not(VARIABLE_FLAGS);
    assert_that!("object3d flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    // zero040 (40) already asserted
    // 45697 only in mechlib
    assert_that!("object3d field 044", node.unk044 in [1, 45697], offset + 44)?;
    // zone_id (48) is variable
    // node_type (52) already asserted
    assert_that!("object3d data ptr", node.data_ptr != 0, offset + 56)?;
    if mesh_index_is_ptr {
        if node.flags.contains(NodeBitFlags::HAS_MESH) {
            // non-zero, but the memory on 32-bit is limited
            assert_that!("object3d mesh index", node.mesh_index > 0, offset + 60)?;
        } else {
            assert_that!("object3d mesh index", node.mesh_index == 0, offset + 60)?;
        }
    } else {
        if node.flags.contains(NodeBitFlags::HAS_MESH) {
            assert_that!("object3d mesh index", node.mesh_index >= 0, offset + 60)?;
        } else {
            assert_that!("object3d mesh index", node.mesh_index == -1, offset + 60)?;
        }
    }
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    // area_partition (76) is variable
    // has_parent (84) is variable
    // parent_array_ptr (88) already asserted
    // children_count (86) is variable
    // children_array_ptr (92) already asserted
    // zero096 (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    assert_that!("object3d field 112", node.unk112 in [0, 1, 2], offset + 112)?;
    // unk116 (116) is variable
    // unk140 (140) is variable
    // unk164 (164) is variable
    // zero188 (188) already asserted
    // zero192 (192) already asserted
    assert_that!("object3d field 196", node.unk196 == 160, offset + 196)?;
    // zero200 (200) already asserted
    // zero204 (204) already asserted
    Ok(NodeVariantPm::Object3d(node))
}

pub(crate) fn make_variants(object3d: &Object3d) -> Result<NodeVariantsPm> {
    let children_count = assert_len!(u16, object3d.children.len(), "object 3d children")?;
    Ok(NodeVariantsPm {
        name: object3d.name.clone(),
        flags: NodeBitFlags::from(&object3d.flags),
        unk044: object3d.unk044,
        zone_id: object3d.zone_id,
        data_ptr: object3d.data_ptr,
        mesh_index: object3d.mesh_index,
        area_partition: object3d.area_partition,
        has_parent: object3d.parent.is_some(),
        parent_array_ptr: object3d.parent_array_ptr,
        children_count,
        children_array_ptr: object3d.children_array_ptr,
        unk112: object3d.unk112,
        unk116: object3d.unk116,
        unk140: object3d.unk140,
        unk164: object3d.unk164,
        unk196: 160,
    })
}
