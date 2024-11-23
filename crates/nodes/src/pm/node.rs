use super::wrappers::WrappedNodePm;
use super::{camera, display, light, lod, object3d, window, world};
use crate::flags::NodeBitFlags;
use crate::types::NodeType;
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::nodes::pm::{AreaPartitionPm, NodePm};
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, bool_c, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Ascii, Hex, Ptr};
use num_traits::FromPrimitive;
use std::io::{Read, Write};

#[derive(Debug)]
pub struct NodeVariantsPm {
    pub name: String,
    pub flags: NodeBitFlags,
    pub unk044: u32,
    pub zone_id: u32,
    pub data_ptr: u32,
    pub mesh_index: i32,
    pub area_partition: Option<AreaPartitionPm>,
    pub has_parent: bool,
    pub parent_array_ptr: u32,
    pub children_count: u16,
    pub children_array_ptr: u32,
    pub unk112: u32,
    pub unk116: BoundingBox,
    pub unk140: BoundingBox,
    pub unk164: BoundingBox,
    pub unk196: u32,
}

#[derive(Debug)]
pub struct NodeVariantLodPm {
    pub name: String,
    pub flags: NodeBitFlags,
    pub zone_id: u32,
    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_count: u16,
    pub children_array_ptr: u32,
    pub unk164: BoundingBox,
}

#[derive(Debug)]
pub enum NodeVariantPm {
    Camera {
        data_ptr: u32,
    },
    Display {
        data_ptr: u32,
    },
    Light {
        data_ptr: u32,
    },
    Lod(NodeVariantLodPm),
    Object3d(NodeVariantsPm),
    Window {
        data_ptr: u32,
    },
    World {
        data_ptr: u32,
        children_count: u16,
        children_array_ptr: u32,
    },
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct NodePmC {
    name: Ascii<36>,                 // 000
    flags: Hex<u32>,                 // 036
    zero040: u32,                    // 040
    unk044: u32,                     // 044
    zone_id: u32,                    // 048
    node_type: u32,                  // 052
    data_ptr: Ptr,                   // 056
    mesh_index: i32,                 // 060
    environment_data: u32,           // 064
    action_priority: u32,            // 068
    action_callback: u32,            // 072
    area_partition: AreaPartitionPm, // 076
    parent_count: u16,               // 084
    children_count: u16,             // 086
    parent_array_ptr: Ptr,           // 088
    children_array_ptr: Ptr,         // 092
    zero096: u32,                    // 096
    zero100: u32,                    // 100
    zero104: u32,                    // 104
    zero108: u32,                    // 108
    unk112: u32,                     // 112
    unk116: BoundingBox,             // 116
    unk140: BoundingBox,             // 140
    unk164: BoundingBox,             // 164
    zero188: u32,                    // 188
    zero192: u32,                    // 192
    unk196: u32,                     // 196
    zero200: u32,                    // 200
    zero204: u32,                    // 204
}
impl_as_bytes!(NodePmC, 208);

pub const NODE_PM_C_SIZE: u32 = NodePmC::SIZE;

fn assert_node(node: NodePmC, offset: usize) -> Result<(NodeType, NodeVariantsPm)> {
    // invariants for every node type

    let node_type = FromPrimitive::from_u32(node.node_type).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid node type, but was {} (at {})",
            node.node_type,
            offset + 52
        )
    })?;
    if node_type == NodeType::Empty {
        return Err(assert_with_msg!(
            "Expected valid node type, but was {} (at {})",
            node.node_type,
            offset + 52
        ));
    }

    let name = assert_utf8("node name", offset + 0, || node.name.to_str_node_name())?;
    let flags = NodeBitFlags::from_bits(node.flags.0).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid node flags, but was {:?} (at {})",
            node.flags,
            offset + 36
        )
    })?;
    assert_that!("field 040", node.zero040 == 0, offset + 40)?;
    // 45697 only in mechlib
    assert_that!("field 044", node.unk044 in [0, 1, 45697], offset + 44)?;
    // node_type (52) see above
    // data_ptr (56) is variable
    // mesh_index (60) is variable
    assert_that!("env data", node.environment_data == 0, offset + 64)?;
    assert_that!("action prio", node.action_priority == 1, offset + 68)?;
    assert_that!("action cb", node.action_callback == 0, offset + 72)?;
    // area partition (76) see below
    // parent_count (84) see below
    // children_count (86) is variable
    // parent_array_ptr (88) is variable
    // children_array_ptr (92) is variable
    assert_that!("field 096", node.zero096 == 0, offset + 96)?;
    assert_that!("field 100", node.zero100 == 0, offset + 100)?;
    assert_that!("field 104", node.zero104 == 0, offset + 104)?;
    assert_that!("field 108", node.zero108 == 0, offset + 108)?;
    assert_that!("field 112", node.unk112 in [0, 1, 2], offset + 112)?;
    // unk116 (116) is variable
    // unk140 (140) is variable
    // unk164 (164) is variable
    assert_that!("field 188", node.zero188 == 0, offset + 188)?;
    assert_that!("field 192", node.zero192 == 0, offset + 192)?;
    assert_that!("field 192", node.unk196 in [0, 160], offset + 196)?;
    assert_that!("field 200", node.zero200 == 0, offset + 200)?;
    assert_that!("field 204", node.zero204 == 0, offset + 204)?;

    // assert area partition properly once we have read the world data
    let area_partition = if node.area_partition == AreaPartitionPm::DEFAULT {
        None
    } else {
        // assert_that!("area partition x", 0 <= node.area_partition.x <= 64, offset + 76)?;
        // assert_that!("area partition y", 0 <= node.area_partition.y <= 64, offset + 78)?;
        // assert_that!("area partition vx", node.area_partition.virtual_x == node.area_partition.x, offset + 80)?;
        // assert_that!("area partition vy", node.area_partition.virtual_y == node.area_partition.y, offset + 82)?;
        assert_that!("area partition x", node.area_partition.x <= 64, offset + 76)?;
        assert_that!("area partition y", node.area_partition.y <= 64, offset + 78)?;
        assert_that!(
            "area partition vx",
            node.area_partition.virtual_x <= 64,
            offset + 80
        )?;
        assert_that!(
            "area partition vy",
            node.area_partition.virtual_y <= 64,
            offset + 82
        )?;
        Some(node.area_partition)
    };

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
        zone_id: node.zone_id,
        data_ptr: node.data_ptr.0,
        mesh_index: node.mesh_index,
        area_partition,
        has_parent,
        parent_array_ptr: node.parent_array_ptr.0,
        children_count: node.children_count,
        children_array_ptr: node.children_array_ptr.0,
        unk112: node.unk112,
        unk116: node.unk116,
        unk140: node.unk140,
        unk164: node.unk164,
        unk196: node.unk196,
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

pub fn read_node_info_gamez(
    read: &mut CountingReader<impl Read>,
    index: u32,
) -> Result<NodeVariantPm> {
    debug!(
        "Reading node info {} (pm, {}) at {}",
        index,
        NodePmC::SIZE,
        read.offset
    );
    let node: NodePmC = read.read_struct()?;
    trace!("{:#?}", node);

    let (node_type, node) = assert_node(node, read.prev)?;
    debug!("Node `{}` read", node.name);
    let variant = match node_type {
        NodeType::Empty => unreachable!("empty nodes should not occur in pm"),
        NodeType::World => world::assert_variants(node, read.prev)?,
        NodeType::Window => window::assert_variants(node, read.prev)?,
        NodeType::Camera => camera::assert_variants(node, read.prev)?,
        NodeType::Display => display::assert_variants(node, read.prev)?,
        NodeType::Light => light::assert_variants(node, read.prev)?,
        NodeType::LoD => lod::assert_variants(node, read.prev, false)?,
        NodeType::Object3d => object3d::assert_variants(node, read.prev, false)?,
    };
    Ok(variant)
}

pub fn read_node_data(
    read: &mut CountingReader<impl Read>,
    variant: NodeVariantPm,
    index: usize,
) -> Result<WrappedNodePm> {
    match variant {
        NodeVariantPm::World {
            data_ptr,
            children_count,
            children_array_ptr,
        } => {
            let world = world::read(read, data_ptr, children_count, children_array_ptr, index)?;
            Ok(WrappedNodePm::World(world))
        }
        NodeVariantPm::Window { data_ptr } => {
            let window = window::read(read, data_ptr, index)?;
            Ok(WrappedNodePm::Window(window))
        }
        NodeVariantPm::Camera { data_ptr } => {
            let camera = camera::read(read, data_ptr, index)?;
            Ok(WrappedNodePm::Camera(camera))
        }
        NodeVariantPm::Display { data_ptr } => {
            let display = display::read(read, data_ptr, index)?;
            Ok(WrappedNodePm::Display(display))
        }
        NodeVariantPm::Light { data_ptr } => {
            let light = light::read(read, data_ptr, index)?;
            Ok(WrappedNodePm::Light(light))
        }
        NodeVariantPm::Lod(node) => {
            let lod = lod::read(read, node, index)?;
            Ok(WrappedNodePm::Lod(lod))
        }
        NodeVariantPm::Object3d(node) => {
            let object3d = object3d::read(read, node, index)?;
            Ok(WrappedNodePm::Object3d(object3d))
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

    let name = Ascii::from_str_node_name(&variant.name);

    let area_partition = variant.area_partition.unwrap_or(AreaPartitionPm::DEFAULT);

    let node = NodePmC {
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
    node: &NodePm,
    mesh_index_is_ptr: bool,
    index: usize,
) -> Result<()> {
    match node {
        NodePm::World(world) => {
            let variant = world::make_variants(world)?;
            write_variant(write, NodeType::World, variant, index)
        }
        NodePm::Window(window) => {
            let variant = window::make_variants(window);
            write_variant(write, NodeType::Window, variant, index)
        }
        NodePm::Camera(camera) => {
            let variant = camera::make_variants(camera);
            write_variant(write, NodeType::Camera, variant, index)
        }
        NodePm::Display(display) => {
            let variant = display::make_variants(display);
            write_variant(write, NodeType::Display, variant, index)
        }
        NodePm::Light(light) => {
            let variant = light::make_variants(light);
            write_variant(write, NodeType::Light, variant, index)
        }
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
        NodePm::World(world) => world::write(write, world, index),
        NodePm::Window(window) => window::write(write, window, index),
        NodePm::Camera(camera) => camera::write(write, camera, index),
        NodePm::Display(display) => display::write(write, display, index),
        NodePm::Light(light) => light::write(write, light, index),
        NodePm::Lod(lod) => lod::write(write, lod, index),
        NodePm::Object3d(object3d) => object3d::write(write, object3d, index),
    }
}
