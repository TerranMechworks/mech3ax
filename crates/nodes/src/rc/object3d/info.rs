use super::has_borked_parents;
use crate::flags::NodeBitFlags;
use crate::rc::node::{NodeVariantRc, NodeVariantsRc};
use crate::types::ZONE_ALWAYS;
use mech3ax_api_types::nodes::rc::Object3d;
use mech3ax_common::{assert_len, assert_that, assert_with_msg, Result};

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    | NodeBitFlags::ACTIVE.bits()
    // | NodeBitFlags::ALTITUDE_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_BBOX.bits()
    // | NodeBitFlags::LANDMARK.bits()
    // | NodeBitFlags::BBOX_NODE.bits()
    // | NodeBitFlags::BBOX_MODEL.bits()
    // | NodeBitFlags::BBOX_CHILD.bits()
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
    | NodeBitFlags::BBOX_NODE.bits()
    | NodeBitFlags::BBOX_MODEL.bits()
    | NodeBitFlags::BBOX_CHILD.bits()
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
    assert_that!("object3d zone id", node.zone_id >= ZONE_ALWAYS, offset + 48)?;
    // node_type (52) already asserted
    assert_that!("object3d data ptr", node.data_ptr != 0, offset + 56)?;
    let model_bbox = node.flags.contains(NodeBitFlags::BBOX_MODEL);
    let has_model = node.model_index > -1;
    assert_that!("object3d bbox model", model_bbox == has_model, offset + 60)?;
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    // area_partition (76) is variable
    // parent_count (84) is variable
    if is_borked {
        assert_that!("object3d has parent", node.parent_count in [0, 8], offset + 84)?;
    } else {
        assert_that!("object3d has parent", node.parent_count in [0, 1], offset + 84)?;
    }
    // parent_array_ptr (88) already asserted
    // children_count (92) is variable
    // children_array_ptr (96) already asserted
    // bbox_mid (100) already asserted
    // bbox_diag (112) already asserted
    // node_bbox (116) is variable
    // model_bbox (140) is variable
    // child_bbox (164) is variable
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
        if object3d.parent.is_some() {
            1
        } else {
            0
        }
    };

    let children_count = assert_len!(u32, object3d.children.len(), "object 3d children")?;
    Ok(NodeVariantsRc {
        name: object3d.name.clone(),
        flags: NodeBitFlags::from(&object3d.flags),
        unk044: 4,
        zone_id: object3d.zone_id as i8, // TODO
        data_ptr: object3d.data_ptr,
        model_index: object3d.model_index,
        area_partition: object3d.area_partition,
        parent_count,
        parent_array_ptr: object3d.parent_array_ptr,
        children_count,
        children_array_ptr: object3d.children_array_ptr,
        node_bbox: object3d.node_bbox,
        model_bbox: object3d.model_bbox,
        child_bbox: object3d.child_bbox,
    })
}
