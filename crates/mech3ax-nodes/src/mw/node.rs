use super::camera;
use super::display;
use super::empty;
use super::light;
use super::lod;
use super::object3d;
use super::window;
use super::world;
use super::wrappers::WrappedNodeMw;
use crate::flags::NodeBitFlags;
use crate::types::{NodeType, NodeVariantMw, NodeVariantsMw};
use log::{debug, trace};
use mech3ax_api_types::gamez::nodes::mw::NodeMw;
use mech3ax_api_types::gamez::nodes::{AreaPartition, BoundingBox};
use mech3ax_api_types::{static_assert_size, ReprSize as _};
use mech3ax_common::assert::{assert_all_zero, assert_utf8};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{str_from_c_node_name, str_to_c_node_name};
use mech3ax_common::{assert_that, assert_with_msg, bool_c, Result};
use mech3ax_debug::{Ascii, Hex, Ptr};
use num_traits::FromPrimitive;
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct NodeMwC {
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
    parent_count: u32,             // 084
    parent_array_ptr: Ptr,         // 088
    children_count: u32,           // 092
    children_array_ptr: Ptr,       // 096
    zero100: u32,                  // 100
    zero104: u32,                  // 104
    zero108: u32,                  // 108
    zero112: u32,                  // 112
    unk116: BoundingBox,           // 116
    unk140: BoundingBox,           // 140
    unk164: BoundingBox,           // 164
    zero188: u32,                  // 188
    zero192: u32,                  // 192
    unk196: u32,                   // 196
    zero200: u32,                  // 200
    zero204: u32,                  // 204
}
static_assert_size!(NodeMwC, 208);

pub const NODE_MW_C_SIZE: u32 = NodeMwC::SIZE;

fn assert_node(node: NodeMwC, offset: u32) -> Result<(NodeType, NodeVariantsMw)> {
    // invariants for every node type

    let name = assert_utf8("name", offset + 0, || str_from_c_node_name(&node.name.0))?;
    let flags = NodeBitFlags::from_bits(node.flags.0).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid node flags, but was {:?} (at {})",
            node.flags,
            offset + 36
        )
    })?;
    let node_type = FromPrimitive::from_u32(node.node_type).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid node type, but was {} (at {})",
            node.node_type,
            offset + 52
        )
    })?;

    assert_that!("field 040", node.zero040 == 0, offset + 40)?;

    assert_that!("env data", node.environment_data == 0, offset + 64)?;
    assert_that!("action prio", node.action_priority == 1, offset + 68)?;
    assert_that!("action cb", node.action_callback == 0, offset + 72)?;

    assert_that!("field 100", node.zero100 == 0, offset + 100)?;
    assert_that!("field 104", node.zero104 == 0, offset + 104)?;
    assert_that!("field 108", node.zero108 == 0, offset + 108)?;
    assert_that!("field 112", node.zero112 == 0, offset + 112)?;

    assert_that!("field 188", node.zero188 == 0, offset + 188)?;
    assert_that!("field 192", node.zero192 == 0, offset + 192)?;
    assert_that!("field 200", node.zero200 == 0, offset + 200)?;
    assert_that!("field 204", node.zero204 == 0, offset + 204)?;

    // assert area partition properly once we have read the world data
    let area_partition = if node.area_partition == AreaPartition::DEFAULT_MW {
        None
    } else {
        assert_that!("area partition x", 0 <= node.area_partition.x <= 64, offset + 76)?;
        assert_that!("area partition y", 0 <= node.area_partition.y <= 64, offset + 80)?;
        Some(node.area_partition)
    };

    // can only have one parent
    let has_parent = assert_that!("parent count", bool node.parent_count, offset + 84)?;
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
    assert_that!("children count", 0 <= node.children_count <= 64, offset + 92)?;
    if node.children_count == 0 {
        assert_that!(
            "children array ptr",
            node.children_array_ptr == Ptr::NULL,
            offset + 96
        )?;
    } else {
        assert_that!(
            "children array ptr",
            node.children_array_ptr != Ptr::NULL,
            offset + 96
        )?;
    };

    let variants = NodeVariantsMw {
        name,
        flags,
        unk044: node.unk044,
        zone_id: node.zone_id,
        data_ptr: node.data_ptr.0,
        mesh_index: node.mesh_index,
        area_partition,
        has_parent,
        parent_array_ptr: node.parent_array_ptr.0,
        children_count: node.children_count,
        children_array_ptr: node.children_array_ptr.0,
        unk116: node.unk116,
        unk140: node.unk140,
        unk164: node.unk164,
        unk196: node.unk196,
    };

    Ok((node_type, variants))
}

#[inline]
pub fn mechlib_only_err_mw() -> mech3ax_common::Error {
    assert_with_msg!("Expected only Object3d nodes in mechlib")
}

pub fn read_node_mechlib(
    read: &mut CountingReader<impl Read>,
    index: usize,
) -> Result<WrappedNodeMw> {
    debug!(
        "Reading mechlib node {} (mw, {}) at {}",
        index,
        NodeMwC::SIZE,
        read.offset
    );
    let node: NodeMwC = read.read_struct()?;
    trace!("{:#?}", node);

    let (node_type, node) = assert_node(node, read.prev)?;
    debug!("Node `{}` read", node.name);
    let variant = match node_type {
        NodeType::Object3d => object3d::assert_variants(node, read.prev, true),
        _ => Err(mechlib_only_err_mw()),
    }?;
    read_node_data(read, variant, index)
}

pub fn read_node_info_gamez(
    read: &mut CountingReader<impl Read>,
    index: u32,
) -> Result<Option<NodeVariantMw>> {
    debug!(
        "Reading node info {} (mw, {}) at {}",
        index,
        NodeMwC::SIZE,
        read.offset
    );
    let node: NodeMwC = read.read_struct()?;
    trace!("{:#?}", node);

    if node.name.0[0] == 0 {
        assert_node_info_zero(node, read.prev)?;
        Ok(None)
    } else {
        let (node_type, node) = assert_node(node, read.prev)?;
        let variant = match node_type {
            NodeType::Camera => camera::assert_variants(node, read.prev)?,
            NodeType::Display => display::assert_variants(node, read.prev)?,
            NodeType::Empty => empty::assert_variants(node, read.prev)?,
            NodeType::Light => light::assert_variants(node, read.prev)?,
            NodeType::LoD => lod::assert_variants(node, read.prev)?,
            NodeType::Object3d => object3d::assert_variants(node, read.prev, false)?,
            NodeType::Window => window::assert_variants(node, read.prev)?,
            NodeType::World => world::assert_variants(node, read.prev)?,
        };
        Ok(Some(variant))
    }
}

pub fn read_node_data(
    read: &mut CountingReader<impl Read>,
    variant: NodeVariantMw,
    index: usize,
) -> Result<WrappedNodeMw> {
    match variant {
        NodeVariantMw::Camera(data_ptr) => {
            Ok(WrappedNodeMw::Camera(camera::read(read, data_ptr, index)?))
        }
        NodeVariantMw::Display(data_ptr) => Ok(WrappedNodeMw::Display(display::read(
            read, data_ptr, index,
        )?)),
        NodeVariantMw::Empty(empty) => Ok(WrappedNodeMw::Empty(empty)),
        NodeVariantMw::Light(data_ptr) => {
            Ok(WrappedNodeMw::Light(light::read(read, data_ptr, index)?))
        }
        NodeVariantMw::Lod(node) => Ok(WrappedNodeMw::Lod(lod::read(read, node, index)?)),
        NodeVariantMw::Object3d(node) => {
            Ok(WrappedNodeMw::Object3d(object3d::read(read, node, index)?))
        }
        NodeVariantMw::Window(data_ptr) => {
            Ok(WrappedNodeMw::Window(window::read(read, data_ptr, index)?))
        }
        NodeVariantMw::World(data_ptr, children_count, children_array_ptr) => {
            Ok(WrappedNodeMw::World(world::read(
                read,
                data_ptr,
                children_count,
                children_array_ptr,
                index,
            )?))
        }
    }
}

fn write_variant(
    write: &mut CountingWriter<impl Write>,
    node_type: NodeType,
    variant: NodeVariantsMw,
    index: usize,
) -> Result<()> {
    debug!(
        "Writing node info {} (mw, {}) at {}",
        index,
        NodeMwC::SIZE,
        write.offset
    );

    let mut name = Ascii::new();
    str_to_c_node_name(variant.name, &mut name.0);

    let area_partition = variant.area_partition.unwrap_or(AreaPartition::DEFAULT_MW);

    let node = NodeMwC {
        name,
        flags: Hex(variant.flags.bits()),
        zero040: 0,
        unk044: variant.unk044,
        zone_id: variant.zone_id,
        node_type: node_type as u32,
        data_ptr: Ptr(variant.data_ptr),
        mesh_index: variant.mesh_index,
        environment_data: 0,
        action_priority: 1,
        action_callback: 0,
        area_partition,
        parent_count: bool_c!(variant.has_parent),
        parent_array_ptr: Ptr(variant.parent_array_ptr),
        children_count: variant.children_count,
        children_array_ptr: Ptr(variant.children_array_ptr),
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
    };
    trace!("{:#?}", node);
    write.write_struct(&node)?;
    Ok(())
}

pub fn write_node_info(
    write: &mut CountingWriter<impl Write>,
    node: &NodeMw,
    index: usize,
) -> Result<()> {
    match node {
        NodeMw::Camera(camera) => {
            let variant = camera::make_variants(camera);
            write_variant(write, NodeType::Camera, variant, index)
        }
        NodeMw::Display(display) => {
            let variant = display::make_variants(display);
            write_variant(write, NodeType::Display, variant, index)
        }
        NodeMw::Empty(empty) => {
            let variant = empty::make_variants(empty);
            write_variant(write, NodeType::Empty, variant, index)
        }
        NodeMw::Light(light) => {
            let variant = light::make_variants(light);
            write_variant(write, NodeType::Light, variant, index)
        }
        NodeMw::Lod(lod) => {
            let variant = lod::make_variants(lod)?;
            write_variant(write, NodeType::LoD, variant, index)
        }
        NodeMw::Object3d(object3d) => {
            let variant = object3d::make_variants(object3d)?;
            write_variant(write, NodeType::Object3d, variant, index)
        }
        NodeMw::Window(window) => {
            let variant = window::make_variants(window);
            write_variant(write, NodeType::Window, variant, index)
        }
        NodeMw::World(world) => {
            let variant = world::make_variants(world)?;
            write_variant(write, NodeType::World, variant, index)
        }
    }
}

pub fn write_node_data(
    write: &mut CountingWriter<impl Write>,
    node: &NodeMw,
    index: usize,
) -> Result<()> {
    match node {
        NodeMw::Camera(camera) => camera::write(write, camera, index),
        NodeMw::Display(display) => display::write(write, display, index),
        NodeMw::Empty(_) => Ok(()),
        NodeMw::Light(light) => light::write(write, light, index),
        NodeMw::Lod(lod) => lod::write(write, lod, index),
        NodeMw::Object3d(object3d) => object3d::write(write, object3d, index),
        NodeMw::Window(window) => window::write(write, window, index),
        NodeMw::World(world) => world::write(write, world, index),
    }
}

pub fn size_node(node: &NodeMw) -> u32 {
    match node {
        NodeMw::Camera(_) => camera::size(),
        NodeMw::Empty(_) => empty::size(),
        NodeMw::Display(_) => display::size(),
        NodeMw::Light(_) => light::size(),
        NodeMw::Lod(lod) => lod::size(lod),
        NodeMw::Object3d(object3d) => object3d::size(object3d),
        NodeMw::Window(_) => window::size(),
        NodeMw::World(world) => world::size(world),
    }
}

fn assert_node_info_zero(node: NodeMwC, offset: u32) -> Result<()> {
    assert_all_zero("name", offset + 0, &node.name.0)?;
    assert_that!("flags", node.flags.0 == 0, offset + 36)?;
    assert_that!("node type", node.node_type == 0, offset + 52)?;

    assert_that!("field 040", node.zero040 == 0, offset + 40)?;
    assert_that!("field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("zone id", node.zone_id == 0, offset + 48)?;
    // node type
    assert_that!("data ptr", node.data_ptr == Ptr::NULL, offset + 56)?;
    assert_that!("mesh index", node.mesh_index == -1, offset + 60)?;
    assert_that!("env data", node.environment_data == 0, offset + 64)?;
    assert_that!("action prio", node.action_priority == 0, offset + 68)?;
    assert_that!("action cb", node.action_callback == 0, offset + 72)?;
    assert_that!(
        "area partition",
        node.area_partition == AreaPartition::ZERO,
        offset + 76
    )?;
    assert_that!("parent count", node.parent_count == 0, offset + 84)?;
    assert_that!(
        "parent array ptr",
        node.parent_array_ptr == Ptr::NULL,
        offset + 88
    )?;
    assert_that!("children count", node.children_count == 0, offset + 92)?;
    assert_that!(
        "children array ptr",
        node.children_array_ptr == Ptr::NULL,
        offset + 96
    )?;
    assert_that!("field 100", node.zero100 == 0, offset + 100)?;
    assert_that!("field 104", node.zero104 == 0, offset + 104)?;
    assert_that!("field 108", node.zero108 == 0, offset + 108)?;
    assert_that!("field 112", node.zero112 == 0, offset + 112)?;
    assert_that!("bbox 1", node.unk116 == BoundingBox::EMPTY, offset + 116)?;
    assert_that!("bbox 2", node.unk140 == BoundingBox::EMPTY, offset + 140)?;
    assert_that!("bbox 3", node.unk164 == BoundingBox::EMPTY, offset + 164)?;
    assert_that!("field 188", node.zero188 == 0, offset + 188)?;
    assert_that!("field 192", node.zero192 == 0, offset + 192)?;
    assert_that!("field 196", node.unk196 == 0, offset + 196)?;
    assert_that!("field 200", node.zero200 == 0, offset + 200)?;
    assert_that!("field 204", node.zero204 == 0, offset + 204)?;
    Ok(())
}

pub fn read_node_info_zero(read: &mut CountingReader<impl Read>, index: u32) -> Result<()> {
    debug!(
        "Reading zero node info {} (mw, {}) at {}",
        index,
        NodeMwC::SIZE,
        read.offset
    );
    let node: NodeMwC = read.read_struct()?;
    assert_node_info_zero(node, read.prev)
}

pub fn write_node_info_zero(write: &mut CountingWriter<impl Write>, index: u32) -> Result<()> {
    debug!(
        "Writing zero node info {} (mw, {}) at {}",
        index,
        NodeMwC::SIZE,
        write.offset
    );
    let node = NodeMwC {
        name: Ascii::new(),
        flags: Hex(0),
        zero040: 0,
        unk044: 0,
        zone_id: 0,
        node_type: 0,
        data_ptr: Ptr::NULL,
        mesh_index: -1,
        environment_data: 0,
        action_priority: 0,
        action_callback: 0,
        area_partition: AreaPartition::ZERO,
        parent_count: 0,
        parent_array_ptr: Ptr::NULL,
        children_count: 0,
        children_array_ptr: Ptr::NULL,
        zero100: 0,
        zero104: 0,
        zero108: 0,
        zero112: 0,
        unk116: BoundingBox::EMPTY,
        unk140: BoundingBox::EMPTY,
        unk164: BoundingBox::EMPTY,
        zero188: 0,
        zero192: 0,
        unk196: 0,
        zero200: 0,
        zero204: 0,
    };
    write.write_struct(&node)?;
    Ok(())
}
