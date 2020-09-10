use super::camera;
use super::display;
use super::empty;
use super::flags::NodeBitFlags;
use super::light;
use super::lod;
use super::object3d;
use super::types::{Node, NodeType, NodeVariant, NodeVariants, BLOCK_EMPTY};
use super::window;
use super::world;
use super::wrappers::WrappedNode;
use crate::assert::{assert_all_zero, assert_utf8, AssertionError};
use crate::io_ext::{CountingReader, WriteHelper};
use crate::size::ReprSize;
use crate::string::{str_from_c_node_name, str_to_c_node_name};
use crate::{assert_that, bool_c, static_assert_size, Result};
use std::io::{Read, Write};

#[repr(C)]
struct NodeC {
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
pub const NODE_C_SIZE: u32 = NodeC::SIZE;

fn assert_node(node: NodeC, offset: u32) -> Result<(NodeType, NodeVariants)> {
    // invariants for every node type

    let name = assert_utf8("name", offset + 0, || str_from_c_node_name(&node.name))?;
    let flags = NodeBitFlags::from_bits(node.flags).ok_or_else(|| {
        AssertionError(format!(
            "Expected valid node flags, but was 0x{:08X} (at {})",
            node.flags,
            offset + 36
        ))
    })?;
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
            return Err(AssertionError(msg).into());
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

pub fn read_node_info_mechlib<R>(read: &mut CountingReader<R>) -> Result<NodeVariant>
where
    R: Read,
{
    let node: NodeC = read.read_struct()?;
    let (node_type, node) = assert_node(node, read.prev)?;
    match node_type {
        NodeType::CAMERA => camera::assert_variants(node, read.prev),
        NodeType::DISPLAY => display::assert_variants(node, read.prev),
        NodeType::EMPTY => empty::assert_variants(node, read.prev),
        NodeType::LIGHT => light::assert_variants(node, read.prev),
        NodeType::LOD => lod::assert_variants(node, read.prev),
        NodeType::OBJECT3D => object3d::assert_variants(node, read.prev, true),
        NodeType::WINDOW => window::assert_variants(node, read.prev),
        NodeType::WORLD => world::assert_variants(node, read.prev),
    }
}

pub fn read_node_info_gamez<R>(read: &mut CountingReader<R>) -> Result<Option<NodeVariant>>
where
    R: Read,
{
    let node: NodeC = read.read_struct()?;
    if node.name[0] == 0 {
        assert_node_info_zero(node, read.prev)?;
        Ok(None)
    } else {
        let (node_type, node) = assert_node(node, read.prev)?;
        let variant = match node_type {
            NodeType::CAMERA => camera::assert_variants(node, read.prev)?,
            NodeType::DISPLAY => display::assert_variants(node, read.prev)?,
            NodeType::EMPTY => empty::assert_variants(node, read.prev)?,
            NodeType::LIGHT => light::assert_variants(node, read.prev)?,
            NodeType::LOD => lod::assert_variants(node, read.prev)?,
            NodeType::OBJECT3D => object3d::assert_variants(node, read.prev, false)?,
            NodeType::WINDOW => window::assert_variants(node, read.prev)?,
            NodeType::WORLD => world::assert_variants(node, read.prev)?,
        };
        Ok(Some(variant))
    }
}

pub fn read_node_data<R, T>(
    read: &mut CountingReader<R>,
    variant: NodeVariant,
) -> Result<WrappedNode<T>>
where
    R: Read,
{
    match variant {
        NodeVariant::Camera(data_ptr) => Ok(WrappedNode::Camera(camera::read(read, data_ptr)?)),
        NodeVariant::Display(data_ptr) => Ok(WrappedNode::Display(display::read(read, data_ptr)?)),
        NodeVariant::Empty(empty) => Ok(WrappedNode::Empty(empty)),
        NodeVariant::Light(data_ptr) => Ok(WrappedNode::Light(light::read(read, data_ptr)?)),
        NodeVariant::Lod(node) => Ok(WrappedNode::Lod(lod::read(read, node)?)),
        NodeVariant::Object3d(node) => Ok(WrappedNode::Object3d(object3d::read(read, node)?)),
        NodeVariant::Window(data_ptr) => Ok(WrappedNode::Window(window::read(read, data_ptr)?)),
        NodeVariant::World(data_ptr, children_count, children_array_ptr) => Ok(WrappedNode::World(
            world::read(read, data_ptr, children_count, children_array_ptr)?,
        )),
    }
}

fn write_variant<W>(write: &mut W, node_type: NodeType, variant: NodeVariants) -> Result<()>
where
    W: Write,
{
    let mut name = [0; 36];
    str_to_c_node_name(variant.name, &mut name);

    let (area_partition_x, area_partition_y) = variant.area_partition.unwrap_or((-1, -1));

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
        parent_count: bool_c!(variant.has_parent),
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

pub fn write_node_info<W, T>(write: &mut W, node: &Node<T>) -> Result<()>
where
    W: Write,
{
    match node {
        Node::Camera(camera) => {
            let variant = camera::make_variants(camera);
            write_variant(write, NodeType::CAMERA, variant)
        }
        Node::Display(display) => {
            let variant = display::make_variants(display);
            write_variant(write, NodeType::DISPLAY, variant)
        }
        Node::Empty(empty) => {
            let variant = empty::make_variants(empty);
            write_variant(write, NodeType::EMPTY, variant)
        }
        Node::Light(light) => {
            let variant = light::make_variants(light);
            write_variant(write, NodeType::LIGHT, variant)
        }
        Node::Lod(lod) => {
            let variant = lod::make_variants(lod);
            write_variant(write, NodeType::LOD, variant)
        }
        Node::Object3d(object3d) => {
            let variant = object3d::make_variants(object3d);
            write_variant(write, NodeType::OBJECT3D, variant)
        }
        Node::Window(window) => {
            let variant = window::make_variants(window);
            write_variant(write, NodeType::WINDOW, variant)
        }
        Node::World(world) => {
            let variant = world::make_variants(world);
            write_variant(write, NodeType::WORLD, variant)
        }
    }
}

pub fn write_node_data<W, T>(write: &mut W, node: &Node<T>) -> Result<()>
where
    W: Write,
{
    match node {
        Node::Camera(camera) => camera::write(write, camera),
        Node::Display(display) => display::write(write, display),
        Node::Empty(_) => Ok(()),
        Node::Light(light) => light::write(write, light),
        Node::Lod(lod) => lod::write(write, lod),
        Node::Object3d(object3d) => object3d::write(write, object3d),
        Node::Window(window) => window::write(write, window),
        Node::World(world) => world::write(write, world),
    }
}

pub fn size_node<T>(node: &Node<T>) -> u32 {
    match node {
        Node::Camera(_) => camera::size(),
        Node::Empty(_) => empty::size(),
        Node::Display(_) => display::size(),
        Node::Light(_) => light::size(),
        Node::Lod(lod) => lod::size(lod),
        Node::Object3d(object3d) => object3d::size(object3d),
        Node::Window(_) => window::size(),
        Node::World(world) => world::size(world),
    }
}

fn assert_node_info_zero(node: NodeC, offset: u32) -> Result<()> {
    assert_all_zero("name", offset + 0, &node.name)?;
    assert_that!("flags", node.flags == 0, offset + 36)?;
    assert_that!("flags", node.node_type == 0, offset + 40)?;
    assert_that!("field 040", node.zero040 == 0, offset + 40)?;
    assert_that!("field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("zone id", node.zone_id == 0, offset + 48)?;
    assert_that!("data ptr", node.data_ptr == 0, offset + 56)?;
    assert_that!("mesh index", node.mesh_index == -1, offset + 60)?;
    assert_that!("env data", node.environment_data == 0, offset + 64)?;
    assert_that!("action prio", node.action_priority == 0, offset + 68)?;
    assert_that!("faction cb", node.action_callback == 0, offset + 72)?;
    assert_that!("area partition x", node.area_partition_x == 0, offset + 76)?;
    assert_that!("area partition y", node.area_partition_y == 0, offset + 80)?;
    assert_that!("parent count", node.parent_count == 0, offset + 84)?;
    assert_that!("parent array ptr", node.parent_array_ptr == 0, offset + 88)?;
    assert_that!("children count", node.children_count == 0, offset + 92)?;
    assert_that!(
        "children array ptr",
        node.children_array_ptr == 0,
        offset + 96
    )?;
    assert_that!("field 100", node.zero100 == 0, offset + 100)?;
    assert_that!("field 104", node.zero104 == 0, offset + 104)?;
    assert_that!("field 108", node.zero108 == 0, offset + 108)?;
    assert_that!("field 112", node.zero112 == 0, offset + 112)?;
    assert_that!("block 1", node.unk116 == BLOCK_EMPTY, offset + 116)?;
    assert_that!("block 2", node.unk140 == BLOCK_EMPTY, offset + 140)?;
    assert_that!("block 3", node.unk164 == BLOCK_EMPTY, offset + 164)?;
    assert_that!("field 188", node.zero188 == 0, offset + 188)?;
    assert_that!("field 192", node.zero192 == 0, offset + 192)?;
    assert_that!("field 196", node.unk196 == 0, offset + 196)?;
    assert_that!("field 200", node.zero200 == 0, offset + 200)?;
    assert_that!("field 204", node.zero204 == 0, offset + 204)?;
    Ok(())
}

pub fn read_node_info_zero<R>(read: &mut CountingReader<R>) -> Result<()>
where
    R: Read,
{
    let node: NodeC = read.read_struct()?;
    assert_node_info_zero(node, read.prev)
}

pub fn write_node_info_zero<W>(write: &mut W) -> Result<()>
where
    W: Write,
{
    write.write_struct(&NodeC {
        name: [0; 36],
        flags: 0,
        zero040: 0,
        unk044: 0,
        zone_id: 0,
        node_type: 0,
        data_ptr: 0,
        mesh_index: -1,
        environment_data: 0,
        action_priority: 0,
        action_callback: 0,
        area_partition_x: 0,
        area_partition_y: 0,
        parent_count: 0,
        parent_array_ptr: 0,
        children_count: 0,
        children_array_ptr: 0,
        zero100: 0,
        zero104: 0,
        zero108: 0,
        zero112: 0,
        unk116: BLOCK_EMPTY,
        unk140: BLOCK_EMPTY,
        unk164: BLOCK_EMPTY,
        zero188: 0,
        zero192: 0,
        unk196: 0,
        zero200: 0,
        zero204: 0,
    })?;
    Ok(())
}
