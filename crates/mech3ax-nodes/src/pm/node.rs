use super::lod;
use super::object3d;
use super::wrappers::WrappedNodePm;
use crate::flags::NodeBitFlags;
use crate::types::{NodeType, ZONE_DEFAULT};
use log::{debug, trace};
use mech3ax_api_types::nodes::pm::NodePm;
use mech3ax_api_types::nodes::{AreaPartition, BoundingBox};
use mech3ax_api_types::{static_assert_size, ReprSize as _};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{str_from_c_node_name, str_to_c_node_name};
use mech3ax_common::{assert_that, assert_with_msg, bool_c, Result};
use mech3ax_debug::{Ascii, Hex, Ptr};
use num_traits::FromPrimitive;
use std::io::{Read, Write};

pub struct NodeVariantsPm {
    pub name: String,
    pub flags: NodeBitFlags,
    pub unk044: u32,
    // pub zone_id: u32,
    pub data_ptr: u32,
    pub mesh_index: i32,
    // pub area_partition: Option<AreaPartition>,
    pub has_parent: bool,
    pub parent_array_ptr: u32,
    pub children_count: u16,
    pub children_array_ptr: u32,
    pub unk112: u32,
    pub unk116: BoundingBox,
    pub unk140: BoundingBox,
    pub unk164: BoundingBox,
}

pub enum NodeVariantPm {
    Lod(NodeVariantsPm),
    Object3d(NodeVariantsPm),
}

#[derive(Debug)]
#[repr(C)]
struct NodePmC {
    name: Ascii<36>,               // 000
    flags: Hex<u32>,               // 036
    zero040: u32,                  // 040
    unk044: u32,                   // 044
    zone_id: u32,                  // 048
    node_type: u32,                // 052
    data_ptr: Ptr,                 // 056
    mesh_index: i32,               // 060
    environment_data: u32,         // 064
    action_priority: u32,          // 068
    action_callback: u32,          // 072
    area_partition: AreaPartition, // 076
    parent_count: u16,             // 084
    children_count: u16,           // 086
    parent_array_ptr: Ptr,         // 088
    children_array_ptr: Ptr,       // 092
    zero096: u32,                  // 096
    zero100: u32,                  // 100
    zero104: u32,                  // 104
    zero108: u32,                  // 108
    unk112: u32,                   // 112
    unk116: BoundingBox,           // 116
    unk140: BoundingBox,           // 140
    unk164: BoundingBox,           // 164
    zero188: u32,                  // 188
    zero192: u32,                  // 192
    unk196: u32,                   // 196
    zero200: u32,                  // 200
    zero204: u32,                  // 204
}
static_assert_size!(NodePmC, 208);

pub const NODE_PM_C_SIZE: u32 = NodePmC::SIZE;

fn assert_node(node: NodePmC, offset: u32) -> Result<(NodeType, NodeVariantsPm)> {
    // invariants for every node type

    let node_type = FromPrimitive::from_u32(node.node_type).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid node type, but was {} (at {})",
            node.node_type,
            offset + 52
        )
    })?;

    assert_that!("field 040", node.zero040 == 0, offset + 40)?;
    // mechlib only?
    assert_that!("field 044", node.unk044 in [1, 45697], offset + 44)?;
    // mechlib only?
    assert_that!("zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;

    assert_that!("env data", node.environment_data == 0, offset + 64)?;
    assert_that!("action prio", node.action_priority == 1, offset + 68)?;
    assert_that!("action cb", node.action_callback == 0, offset + 72)?;

    assert_that!("field 096", node.zero096 == 0, offset + 96)?;
    assert_that!("field 100", node.zero100 == 0, offset + 100)?;
    assert_that!("field 104", node.zero104 == 0, offset + 104)?;
    assert_that!("field 108", node.zero108 == 0, offset + 108)?;
    assert_that!("field 112", node.unk112 in [0, 1, 2], offset + 112)?;

    assert_that!("field 188", node.zero188 == 0, offset + 188)?;
    assert_that!("field 192", node.zero192 == 0, offset + 192)?;
    assert_that!("field 192", node.unk196 == 0x000000A0, offset + 196)?;
    assert_that!("field 200", node.zero200 == 0, offset + 200)?;
    assert_that!("field 204", node.zero204 == 0, offset + 204)?;

    // mechlib only?
    assert_that!(
        "area partition",
        node.area_partition == AreaPartition::DEFAULT_PM,
        offset + 76
    )?;
    /*
    // assert area partition properly once we have read the world data
    let area_partition = if node.area_partition == AreaPartition::DEFAULT_PM {
        None
    } else {
        assert_that!("area partition x", 0 <= node.area_partition.x <= 64, offset + 76)?;
        assert_that!("area partition y", 0 <= node.area_partition.y <= 64, offset + 80)?;
        Some(node.area_partition)
    };
    */

    // can only have one parent
    let has_parent = assert_that!("parent count", bool node.parent_count as _, offset + 84)?;
    if has_parent {
        assert_that!(
            "parent array ptr",
            node.parent_array_ptr != Ptr::NULL,
            offset + 88
        )?;
    } else {
        assert_that!(
            "parent array ptr",
            node.parent_array_ptr == Ptr::NULL,
            offset + 88
        )?;
    };

    // upper bound is arbitrary
    assert_that!("children count", 0 <= node.children_count <= 64, offset + 86)?;
    if node.children_count == 0 {
        assert_that!(
            "children array ptr",
            node.children_array_ptr == Ptr::NULL,
            offset + 92
        )?;
    } else {
        assert_that!(
            "children array ptr",
            node.children_array_ptr != Ptr::NULL,
            offset + 92
        )?;
    };

    let variants = NodeVariantsPm {
        name,
        flags,
        unk044: node.unk044,
        // zone_id: node.zone_id,
        data_ptr: node.data_ptr.0,
        mesh_index: node.mesh_index,
        // area_partition,
        has_parent,
        parent_array_ptr: node.parent_array_ptr.0,
        children_count: node.children_count,
        children_array_ptr: node.children_array_ptr.0,
        unk112: node.unk112,
        unk116: node.unk116,
        unk140: node.unk140,
        unk164: node.unk164,
    };

    Ok((node_type, variants))
}

#[inline]
pub fn mechlib_only_err_pm() -> mech3ax_common::Error {
    assert_with_msg!("Expected only Object3d or Lod nodes in mechlib")
}

pub fn read_node_mechlib(
    read: &mut CountingReader<impl Read>,
    index: usize,
) -> Result<WrappedNodePm> {
    debug!(
        "Reading mechlib node {} (pm, {}) at {}",
        index,
        NodePmC::SIZE,
        read.offset
    );
    let node: NodePmC = read.read_struct()?;
    trace!("{:#?}", node);

    let (node_type, node) = assert_node(node, read.prev)?;
    debug!("Node `{}` read", node.name);
    let variant = match node_type {
        NodeType::Object3d => object3d::assert_variants(node, read.prev, true),
        NodeType::LoD => lod::assert_variants(node, read.prev, true),
        _ => Err(mechlib_only_err_pm()),
    }?;
    read_node_data(read, variant, index)
}

pub fn read_node_data(
    read: &mut CountingReader<impl Read>,
    variant: NodeVariantPm,
    index: usize,
) -> Result<WrappedNodePm> {
    match variant {
        NodeVariantPm::Lod(node) => Ok(WrappedNodePm::Lod(lod::read(read, node, index)?)),
        NodeVariantPm::Object3d(node) => {
            Ok(WrappedNodePm::Object3d(object3d::read(read, node, index)?))
        }
    }
}

fn write_variant(
    write: &mut CountingWriter<impl Write>,
    node_type: NodeType,
    variant: NodeVariantsPm,
    index: usize,
) -> Result<()> {
    debug!(
        "Writing node info {} (pm, {}) at {}",
        index,
        NodePmC::SIZE,
        write.offset
    );

    let mut name = Ascii::new();
    str_to_c_node_name(variant.name, &mut name.0);

    let node = NodePmC {
        name,
        flags: Hex(variant.flags.bits()),
        zero040: 0,
        unk044: variant.unk044,
        // zone_id: variant.zone_id,
        zone_id: ZONE_DEFAULT,
        node_type: node_type as u32,
        data_ptr: Ptr(variant.data_ptr),
        mesh_index: variant.mesh_index,
        environment_data: 0,
        action_priority: 1,
        action_callback: 0,
        area_partition: AreaPartition::DEFAULT_PM,
        parent_count: bool_c!(variant.has_parent),
        children_count: variant.children_count,
        parent_array_ptr: Ptr(variant.parent_array_ptr),
        children_array_ptr: Ptr(variant.children_array_ptr),
        zero096: 0,
        zero100: 0,
        zero104: 0,
        zero108: 0,
        unk112: variant.unk112,
        unk116: variant.unk116,
        unk140: variant.unk140,
        unk164: variant.unk164,
        zero188: 0,
        zero192: 0,
        unk196: 0x000000A0,
        zero200: 0,
        zero204: 0,
    };
    trace!("{:#?}", node);
    write.write_struct(&node)?;
    Ok(())
}

pub fn write_node_info(
    write: &mut CountingWriter<impl Write>,
    node: &NodePm,
    mesh_index_is_ptr: bool,
    index: usize,
) -> Result<()> {
    match node {
        NodePm::Lod(lod) => {
            let variant = lod::make_variants(lod, mesh_index_is_ptr)?;
            write_variant(write, NodeType::LoD, variant, index)
        }
        NodePm::Object3d(object3d) => {
            let variant = object3d::make_variants(object3d)?;
            write_variant(write, NodeType::Object3d, variant, index)
        }
    }
}

pub fn write_node_data(
    write: &mut CountingWriter<impl Write>,
    node: &NodePm,
    index: usize,
) -> Result<()> {
    match node {
        NodePm::Lod(lod) => lod::write(write, lod, index),
        NodePm::Object3d(object3d) => object3d::write(write, object3d, index),
    }
}

pub fn size_node(node: &NodePm) -> u32 {
    match node {
        NodePm::Lod(lod) => lod::size(lod),
        NodePm::Object3d(object3d) => object3d::size(object3d),
    }
}
