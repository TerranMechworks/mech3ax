use super::camera;
use super::display;
use super::empty;
use super::light;
use super::lod;
use super::object3d;
use super::window;
use super::world;
use super::wrappers::WrappedNodeRc;
use crate::flags::NodeBitFlags;
use crate::types::{NodeType, ZONE_DEFAULT};
use log::{debug, trace};
use mech3ax_api_types::nodes::rc::{Empty, NodeRc};
use mech3ax_api_types::nodes::{AreaPartition, BoundingBox};
use mech3ax_api_types::{static_assert_size, ReprSize as _};
use mech3ax_common::assert::{assert_all_zero, assert_utf8};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{str_from_c_node_name, str_to_c_node_name};
use mech3ax_common::{assert_that, assert_with_msg, bool_c, Result};
use mech3ax_debug::{Ascii, Hex, Ptr};
use num_traits::FromPrimitive;
use std::io::{Read, Write};

#[derive(Debug)]
pub struct NodeVariantsRc {
    pub name: String,
    pub flags: NodeBitFlags,
    pub unk044: u32,
    pub zone_id: u32,
    pub data_ptr: u32,
    pub mesh_index: i32,
    pub area_partition: Option<AreaPartition>,
    pub has_parent: bool,
    pub parent_array_ptr: u32,
    pub children_count: u32,
    pub children_array_ptr: u32,
    pub unk116: BoundingBox,
    pub unk140: BoundingBox,
    pub unk164: BoundingBox,
}

#[derive(Debug)]
pub struct NodeVariantLodRc {
    pub name: String,
    pub flags: NodeBitFlags,
    pub zone_id: u32,
    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_count: u32,
    pub children_array_ptr: u32,
    pub unk116: BoundingBox,
}

#[derive(Debug)]
pub enum NodeVariantRc {
    Camera {
        data_ptr: u32,
    },
    Display {
        data_ptr: u32,
    },
    Empty(Empty),
    Light {
        data_ptr: u32,
    },
    Lod(NodeVariantLodRc),
    Object3d(NodeVariantsRc),
    Window {
        data_ptr: u32,
    },
    World {
        data_ptr: u32,
        children_count: u32,
        children_array_ptr: u32,
    },
}

#[derive(Debug)]
#[repr(C)]
struct NodeRcC {
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
}
static_assert_size!(NodeRcC, 192);

pub const NODE_RC_C_SIZE: u32 = NodeRcC::SIZE;

const ABORT_TEST_NODE_NAME: &[u8; 36] =
    b"abort_test\0ng\0ame\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
const ABORT_TEST_NAME: &str = "abort_test";

fn assert_node(node: NodeRcC, offset: u32) -> Result<(NodeType, NodeVariantsRc)> {
    // invariants for every node type

    let node_type = FromPrimitive::from_u32(node.node_type).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid node type, but was {} (at {})",
            node.node_type,
            offset + 52
        )
    })?;

    let name = if &node.name.0 == ABORT_TEST_NODE_NAME {
        ABORT_TEST_NAME.to_string()
    } else {
        assert_utf8("name", offset + 0, || str_from_c_node_name(&node.name.0))?
    };
    let flags = NodeBitFlags::from_bits(node.flags.0).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid node flags, but was {:?} (at {})",
            node.flags,
            offset + 36
        )
    })?;
    assert_that!("field 040", node.zero040 == 0, offset + 40)?;
    // unk044 (044) is variable
    // zone_id (048) is variable
    // node_type (052) see above
    // data_ptr (056) is variable
    // mesh_index (060) is variable
    assert_that!("env data", node.environment_data == 0, offset + 64)?;
    assert_that!("action prio", node.action_priority == 1, offset + 68)?;
    assert_that!("action cb", node.action_callback == 0, offset + 72)?;
    // area_partition (076) see below
    // parent_count (084) see below
    // parent_array_ptr (088) is variable
    // children_count (092) is variable
    // children_array_ptr (096) is variable
    assert_that!("field 100", node.zero100 == 0, offset + 100)?;
    assert_that!("field 104", node.zero104 == 0, offset + 104)?;
    assert_that!("field 108", node.zero108 == 0, offset + 108)?;
    assert_that!("field 112", node.zero112 == 0, offset + 112)?;
    // unk116 (116) is variable
    // unk140 (140) is variable
    // unk164 (164) is variable
    assert_that!("field 188", node.zero188 == 0, offset + 188)?;

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
    assert_that!("children count", 0 <= node.children_count <= 80, offset + 92)?;
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

    let variants = NodeVariantsRc {
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
    };

    Ok((node_type, variants))
}

pub fn read_node_info(read: &mut CountingReader<impl Read>, index: u32) -> Result<NodeVariantRc> {
    debug!(
        "Reading node info {} (rc, {}) at {}",
        index,
        NodeRcC::SIZE,
        read.offset
    );
    let node: NodeRcC = read.read_struct()?;
    trace!("{:#?}", node);

    let (node_type, node) = assert_node(node, read.prev)?;
    let variant = match node_type {
        // Empty nodes only occur in m6?
        NodeType::Empty => empty::assert_variants(node, read.prev)?,
        // NodeType::Empty => return Err(assert_with_msg!("No Empty")),
        NodeType::Camera => camera::assert_variants(node, read.prev)?,
        NodeType::World => world::assert_variants(node, read.prev)?,
        NodeType::Window => window::assert_variants(node, read.prev)?,
        NodeType::Display => display::assert_variants(node, read.prev)?,
        NodeType::Object3d => object3d::assert_variants(node, read.prev)?,
        NodeType::LoD => lod::assert_variants(node, read.prev)?,
        NodeType::Light => light::assert_variants(node, read.prev)?,
    };
    Ok(variant)
}

fn assert_node_info_zero(node: NodeRcC, offset: u32) -> Result<()> {
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
    Ok(())
}

pub fn read_node_info_zero(read: &mut CountingReader<impl Read>, index: u32) -> Result<()> {
    debug!(
        "Reading zero node info {} (rc, {}) at {}",
        index,
        NodeRcC::SIZE,
        read.offset
    );
    let node: NodeRcC = read.read_struct()?;
    assert_node_info_zero(node, read.prev)
}

fn write_variant(
    write: &mut CountingWriter<impl Write>,
    node_type: NodeType,
    variant: NodeVariantsRc,
    index: usize,
) -> Result<()> {
    debug!(
        "Writing node info {} (rc, {}) at {}",
        index,
        NodeRcC::SIZE,
        write.offset
    );

    let mut name = Ascii::new();
    if variant.name == ABORT_TEST_NAME {
        name.0.copy_from_slice(ABORT_TEST_NODE_NAME);
    } else {
        str_to_c_node_name(variant.name, &mut name.0);
    }

    let area_partition = variant.area_partition.unwrap_or(AreaPartition::DEFAULT_MW);

    let node = NodeRcC {
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
    };
    trace!("{:#?}", node);
    write.write_struct(&node)?;
    Ok(())
}

pub fn write_node_info(
    write: &mut CountingWriter<impl Write>,
    node: &NodeRc,
    index: usize,
) -> Result<()> {
    match node {
        NodeRc::Camera(camera) => {
            let variant = camera::make_variants(camera);
            write_variant(write, NodeType::Camera, variant, index)
        }
        NodeRc::Display(display) => {
            let variant = display::make_variants(display);
            write_variant(write, NodeType::Display, variant, index)
        }
        NodeRc::Empty(empty) => {
            let variant = empty::make_variants(empty);
            write_variant(write, NodeType::Empty, variant, index)
        }
        NodeRc::Light(light) => {
            let variant = light::make_variants(light);
            write_variant(write, NodeType::Light, variant, index)
        }
        NodeRc::Lod(lod) => {
            let variant = lod::make_variants(lod)?;
            write_variant(write, NodeType::LoD, variant, index)
        }
        NodeRc::Object3d(object3d) => {
            let variant = object3d::make_variants(object3d)?;
            write_variant(write, NodeType::Object3d, variant, index)
        }
        NodeRc::Window(window) => {
            let variant = window::make_variants(window);
            write_variant(write, NodeType::Window, variant, index)
        }
        NodeRc::World(world) => {
            let variant = world::make_variants(world)?;
            write_variant(write, NodeType::World, variant, index)
        }
    }
}

// pub fn write_node_data(
//     write: &mut CountingWriter<impl Write>,
//     node: &NodeRc,
//     index: usize,
// ) -> Result<()> {
//     match node {
//         NodeRc::Camera(camera) => camera::write(write, camera, index),
//         NodeRc::Display(display) => display::write(write, display, index),
//         NodeRc::Empty(_) => Ok(()),
//         NodeRc::Light(light) => light::write(write, light, index),
//         NodeRc::Lod(lod) => lod::write(write, lod, index),
//         NodeRc::Object3d(object3d) => object3d::write(write, object3d, index),
//         NodeRc::Window(window) => window::write(write, window, index),
//         NodeRc::World(world) => world::write(write, world, index),
//     }
// }

pub fn size_node(node: &NodeRc) -> u32 {
    match node {
        NodeRc::Camera(_) => camera::size(),
        NodeRc::Empty(_) => empty::size(),
        NodeRc::Display(_) => display::size(),
        NodeRc::Light(_) => light::size(),
        NodeRc::Lod(lod) => lod::size(lod),
        NodeRc::Object3d(object3d) => object3d::size(object3d),
        NodeRc::Window(_) => window::size(),
        NodeRc::World(world) => world::size(world),
    }
}

pub fn write_node_info_zero(write: &mut CountingWriter<impl Write>, index: u32) -> Result<()> {
    debug!(
        "Writing zero node info {} (rc, {}) at {}",
        index,
        NodeRcC::SIZE,
        write.offset
    );
    let node = NodeRcC {
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
    };
    write.write_struct(&node)?;
    Ok(())
}
