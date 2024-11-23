use crate::cs::node::{NodeVariantCs, NodeVariantsCs};
use crate::flags::NodeBitFlagsCs;
use mech3ax_api_types::nodes::cs::Object3d;
use mech3ax_common::{assert_len, assert_that, Result};

const ALWAYS_PRESENT: NodeBitFlagsCs = NodeBitFlagsCs::from_bits_truncate(
    NodeBitFlagsCs::UNK19.bits() | NodeBitFlagsCs::UNK24.bits() | NodeBitFlagsCs::UNK25.bits(),
);
const VARIABLE_FLAGS: NodeBitFlagsCs = NodeBitFlagsCs::from_bits_truncate(
    0
    | NodeBitFlagsCs::UNK02.bits()
    | NodeBitFlagsCs::UNK03.bits()
    | NodeBitFlagsCs::UNK04.bits()
    | NodeBitFlagsCs::UNK05.bits()
    | NodeBitFlagsCs::UNK07.bits()
    | NodeBitFlagsCs::UNK08.bits()
    | NodeBitFlagsCs::UNK09.bits()
    | NodeBitFlagsCs::UNK10.bits()
    | NodeBitFlagsCs::UNK12.bits()
    | NodeBitFlagsCs::UNK15.bits()
    // | NodeBitFlagsCs::UNK19.bits() // always present
    | NodeBitFlagsCs::UNK23.bits()
    // | NodeBitFlagsCs::UNK23.bits() // always present
    // | NodeBitFlagsCs::UNK25.bits() // always present
    | 0,
);

pub fn assert_variants(node: NodeVariantsCs, offset: usize) -> Result<NodeVariantCs> {
    // can't assert name
    // flags (36) is variable
    let const_flags = node.flags & !VARIABLE_FLAGS;
    assert_that!("object3d flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    // unk040 (40) is variable
    // unk044 (44) is variable
    // zone_id (48) is variable
    // node_type (52) already asserted
    // data_ptr (56) already asserted
    // mesh_index (60) is variable
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    // area_partition (76) is variable
    // has_parent (84) is variable
    // parent_array_ptr (88) already asserted
    // children_count (92) is variable
    // children_array_ptr (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    // unk112 (112) is variable
    // unk116 (116) is variable
    // unk140 (140) is variable
    // unk164 (164) is variable
    // zero188 (188) already asserted
    // zero192 (192) already asserted
    assert_that!("object3d field 196", node.unk196 == 160, offset + 196)?;
    // zero200 (200) already asserted
    // zero204 (204) already asserted
    Ok(NodeVariantCs::Object3d(node))
}

pub fn make_variants(object3d: &Object3d) -> Result<NodeVariantsCs> {
    let children_count = assert_len!(u16, object3d.children.len(), "object3d children")?;
    //let mut flags = ALWAYS_PRESENT;
    let flags = NodeBitFlagsCs::from_bits_truncate(object3d.flags);
    Ok(NodeVariantsCs {
        name: object3d.name.clone(),
        flags,
        unk040: object3d.unk040,
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
