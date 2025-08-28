use super::has_borked_parents;
use crate::flags::NodeBitFlags;
use crate::rc::node::{NodeVariantRc, NodeVariantsRc};
use crate::types::ZONE_DEFAULT;
use mech3ax_api_types::nodes::rc::Object3d;
use mech3ax_common::{assert_len, assert_that, assert_with_msg, bool_c, Result};

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
    // | NodeBitFlags::TREE_VALID.bits()
    // | NodeBitFlags::ID_ZONE_CHECK.bits()
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
    // | NodeBitFlags::TERRAIN.bits()
    | NodeBitFlags::CAN_MODIFY.bits()
    | NodeBitFlags::CLIP_TO.bits()
    | NodeBitFlags::TREE_VALID.bits()
    | NodeBitFlags::ID_ZONE_CHECK.bits()
    // | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);

pub(crate) fn assert_variants(node: NodeVariantsRc, offset: usize) -> Result<NodeVariantRc> {
    let is_borked = has_borked_parents(node.data_ptr, node.parent_array_ptr);

    // cannot assert name
    let const_flags = node.flags.mask_not(VARIABLE_FLAGS);
    assert_that!("object3d flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    // zero040 (40) already asserted
    assert_that!("object3d field 044", node.unk044 == 4, offset + 44)?;
    if node.zone_id != ZONE_DEFAULT {
        assert_that!("object3d zone id", 0 <= node.zone_id <= 80, offset + 48)?;
    }
    // node_type (52) already asserted
    assert_that!("object3d data ptr", node.data_ptr != 0, offset + 56)?;
    if node.flags.contains(NodeBitFlags::HAS_MESH) {
        assert_that!("object3d mesh index", node.mesh_index >= 0, offset + 60)?;
    } else {
        assert_that!("object3d mesh index", node.mesh_index == -1, offset + 60)?;
    }
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    // area_partition (76) is variable
    // parent_count (84) is variable
    if is_borked {
        assert_that!("object3d has parent", node.parent_count in [0, 8], offset + 84)?;
    } else {
        assert_that!("object3d has parent", bool node.parent_count, offset + 84)?;
    }
    // parent_array_ptr (88) already asserted
    // children_count (92) is variable
    // children_array_ptr (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    // zero112 (112) already asserted
    // unk116 (116) is variable
    // unk140 (140) is variable
    // unk164 (164) is variable
    // zero188 (188) already asserted
    Ok(NodeVariantRc::Object3d(node))
}

pub(crate) fn make_variants(object3d: &Object3d) -> Result<NodeVariantsRc> {
    let is_borked = has_borked_parents(object3d.data_ptr, object3d.parent_array_ptr);

    let parent_count = if is_borked {
        match (object3d.parent, &object3d.parents) {
            (None, Some(parents)) => assert_len!(u32, parents.len(), "object 3d parents")?,
            _ => return Err(assert_with_msg!("Parents dirty hack error")),
        }
    } else {
        if object3d.parents.is_some() {
            return Err(assert_with_msg!(
                "Nodes must not have parents set (dirty hack)"
            ));
        }
        bool_c!(object3d.parent.is_some())
    };

    let children_count = assert_len!(u32, object3d.children.len(), "object 3d children")?;
    Ok(NodeVariantsRc {
        name: object3d.name.clone(),
        flags: NodeBitFlags::from(&object3d.flags),
        unk044: 4,
        zone_id: object3d.zone_id,
        data_ptr: object3d.data_ptr,
        mesh_index: object3d.mesh_index,
        area_partition: object3d.area_partition,
        parent_count,
        parent_array_ptr: object3d.parent_array_ptr,
        children_count,
        children_array_ptr: object3d.children_array_ptr,
        unk116: object3d.unk116,
        unk140: object3d.unk140,
        unk164: object3d.unk164,
    })
}
