use super::flags::NodeBitFlags;
use super::object3d::{node_object3d, read_object3d, write_object3d};
use super::types::{Node, NodeType, NodeVariants};
use super::wrappers::WrappedNode;
use crate::assert::{assert_utf8, AssertionError};
use crate::io_ext::{ReadHelper, WriteHelper};
use crate::string::{str_from_c_node_name, str_to_c_node_name};
use crate::{assert_that, static_assert_size, Result};
use std::io::{Read, Write};

#[repr(C)]
pub struct NodeC {
    name: [u8; 36],
    flags: u32, // 036
    zero040: u32,
    unk044: u32,
    zone_id: u32,            // 048
    node_type: u32,          // 052
    data_ptr: u32,           // 056
    mesh_index: i32,         // 060
    environment_data: u32,   // 064
    action_priority: u32,    // 068
    action_callback: u32,    // 072
    area_partition_x: i32,   // 076
    area_partition_y: i32,   // 080
    parent_count: u32,       // 084
    parent_array_ptr: u32,   // 088
    children_count: u32,     // 092
    children_array_ptr: u32, // 096
    zero100: u32,
    zero104: u32,
    zero108: u32,
    zero112: u32,
    unk116: (f32, f32, f32, f32, f32, f32),
    unk140: (f32, f32, f32, f32, f32, f32),
    unk164: (f32, f32, f32, f32, f32, f32),
    zero188: u32,
    zero192: u32,
    unk196: u32,
    zero200: u32,
    zero204: u32,
}
static_assert_size!(NodeC, 208);

fn assert_node(node: NodeC, offset: u32) -> Result<(NodeType, NodeVariants)> {
    // invariants for every node type

    let name = assert_utf8("name", offset + 0, || str_from_c_node_name(&node.name))?;
    let flags = NodeBitFlags::from_bits(node.flags).ok_or(AssertionError(format!(
        "Expected valid flag, but was {:08X} (at {})",
        node.flags,
        offset + 36
    )))?;
    let node_type = match node.node_type {
        value if value == NodeType::EMPTY as u32 => NodeType::EMPTY,
        value if value == NodeType::CAMERA as u32 => NodeType::CAMERA,
        value if value == NodeType::WORLD as u32 => NodeType::WORLD,
        value if value == NodeType::WINDOW as u32 => NodeType::WINDOW,
        value if value == NodeType::DISPLAY as u32 => NodeType::DISPLAY,
        value if value == NodeType::OBJECT3D as u32 => NodeType::OBJECT3D,
        value if value == NodeType::LOD as u32 => NodeType::LOD,
        value if value == NodeType::LIGHT as u32 => NodeType::LIGHT,
        value => {
            let msg = format!(
                "Expected valid node type, but was {} (at {})",
                value,
                offset + 52
            );
            return Err(AssertionError(msg))?;
        }
    };

    assert_that!("field 040", node.zero040 == 0, offset + 40)?;

    assert_that!("env data", node.environment_data == 0, offset + 64)?;
    assert_that!("action prio", node.action_priority == 1, offset + 68)?;
    assert_that!("faction cb", node.action_callback == 0, offset + 72)?;

    assert_that!("field 100", node.zero100 == 0, offset + 100)?;
    assert_that!("field 104", node.zero104 == 0, offset + 104)?;
    assert_that!("field 108", node.zero108 == 0, offset + 108)?;
    assert_that!("field 112", node.zero112 == 0, offset + 112)?;

    assert_that!("field 188", node.zero188 == 0, offset + 188)?;
    assert_that!("field 192", node.zero192 == 0, offset + 192)?;
    assert_that!("field 200", node.zero200 == 0, offset + 200)?;
    assert_that!("field 204", node.zero204 == 0, offset + 204)?;

    // assert area partition properly once we have read the world data
    let area_partition = if node.area_partition_x == -1 || node.area_partition_y == -1 {
        None
    } else {
        assert_that!("area partition x", 0 <= node.area_partition_x <= 64, offset + 76)?;
        assert_that!("area partition y", 0 <= node.area_partition_y <= 64, offset + 80)?;
        Some((node.area_partition_x, node.area_partition_y))
    };

    // can only have one parent
    let has_parent = assert_that!("parent count", bool node.parent_count, offset + 84)?;
    if has_parent {
        assert_that!("parent array ptr", node.parent_array_ptr != 0, offset + 88)?;
    } else {
        assert_that!("parent array ptr", node.parent_array_ptr == 0, offset + 88)?;
    };

    // upper bound is arbitrary
    assert_that!("children count", 0 <= node.children_count <= 64, offset + 92)?;
    if node.children_count == 0 {
        assert_that!(
            "children array ptr",
            node.children_array_ptr == 0,
            offset + 96
        )?;
    } else {
        assert_that!(
            "children array ptr",
            node.children_array_ptr != 0,
            offset + 96
        )?;
    };

    let variants = NodeVariants {
        name,
        flags,
        unk044: node.unk044,
        zone_id: node.zone_id,
        data_ptr: node.data_ptr,
        mesh_index: node.mesh_index,
        area_partition,
        has_parent,
        parent_array_ptr: node.parent_array_ptr,
        children_count: node.children_count,
        children_array_ptr: node.children_array_ptr,
        unk116: node.unk116,
        unk140: node.unk140,
        unk164: node.unk164,
        unk196: node.unk196,
    };

    Ok((node_type, variants))
}

pub fn read_node<R>(read: &mut R, offset: &mut u32) -> Result<WrappedNode>
where
    R: Read,
{
    let node: NodeC = read.read_struct()?;
    let (node_type, node) = assert_node(node, *offset)?;
    // the node types should increment offset after doing further asserts on it
    match node_type {
        NodeType::OBJECT3D => Ok(WrappedNode::Object3d(read_object3d(read, node, offset)?)),
        _ => panic!("other node types unimplemented"),
    }
}

fn write_variant<W>(write: &mut W, node_type: NodeType, variant: NodeVariants) -> Result<()>
where
    W: Write,
{
    let mut name = [0; 36];
    str_to_c_node_name(variant.name, &mut name);

    let (area_partition_x, area_partition_y) = variant.area_partition.unwrap_or((-1, -1));
    let parent_count = if variant.has_parent { 1 } else { 0 };

    write.write_struct(&NodeC {
        name,
        flags: variant.flags.bits(),
        zero040: 0,
        unk044: variant.unk044,
        zone_id: variant.zone_id,
        node_type: node_type as u32,
        data_ptr: variant.data_ptr,
        mesh_index: variant.mesh_index,
        environment_data: 0,
        action_priority: 1,
        action_callback: 0,
        area_partition_x,
        area_partition_y,
        parent_count,
        parent_array_ptr: variant.parent_array_ptr,
        children_count: variant.children_count,
        children_array_ptr: variant.children_array_ptr,
        zero100: 0,
        zero104: 0,
        zero108: 0,
        zero112: 0,
        unk116: variant.unk116,
        unk140: variant.unk140,
        unk164: variant.unk164,
        zero188: 0,
        zero192: 0,
        unk196: variant.unk196,
        zero200: 0,
        zero204: 0,
    })?;
    Ok(())
}

pub fn write_node<W>(write: &mut W, node: &Node) -> Result<()>
where
    W: Write,
{
    match node {
        Node::Object3d(object3d) => {
            let variant = node_object3d(object3d);
            write_variant(write, NodeType::OBJECT3D, variant)?;
            write_object3d(write, object3d)
        }
    }
}
