use super::{camera, display, light, lod, object3d, window, world};
use crate::flags::NodeBitFlagsCs;
use crate::types::{NodeType, ZONE_DEFAULT};
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::nodes::cs::NodeCs;
use mech3ax_api_types::nodes::pm::AreaPartitionPm;
use mech3ax_api_types::nodes::BoundingBox;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, bool_c, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Ascii, Maybe, Ptr};
use std::io::{Read, Write};

#[derive(Debug)]
pub struct NodeVariantsCs {
    pub name: String,
    pub flags: NodeBitFlagsCs,
    pub unk040: u32,
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
pub struct NodeVariantLodCs {
    pub name: String,
    pub flags_unk03: bool,
    pub flags_unk04: bool,
    pub flags_unk07: bool,
    pub unk040: u32,
    pub zone_id: u32,
    pub data_ptr: u32,
    pub parent_array_ptr: u32,
    pub children_count: u16,
    pub children_array_ptr: u32,
    pub unk164: BoundingBox,
}

#[derive(Debug)]
pub enum NodeVariantCs {
    World {
        data_ptr: u32,
        children_count: u16,
        children_array_ptr: u32,
    },
    Display {
        data_ptr: u32,
    },
    Window {
        data_ptr: u32,
        spyglass: bool,
    },
    Camera {
        data_ptr: u32,
        spyglass: bool,
    },
    Light {
        data_ptr: u32,
    },
    Lod(NodeVariantLodCs),
    Object3d(NodeVariantsCs),
}

type Flags = Maybe<u32, NodeBitFlagsCs>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct NodeCsC {
    name: Ascii<36>,                 // 000
    flags: Flags,                    // 036
    unk040: u32,                     // 040
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
impl_as_bytes!(NodeCsC, 208);

pub const NODE_CS_C_SIZE: u32 = NodeCsC::SIZE;

const GEOMETRY_NODE_NAME: &[u8; 36] = b"geometry\08\0e_name\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
const GEOMETRY_NAME: &str = "geometry";

const COCKPIT_NODE_NAME: &[u8; 36] = b"cockpit1\x000\0e_name\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
const COCKPIT_NAME: &str = "cockpit1";

const ALWAYS_PRESENT: NodeBitFlagsCs =
    NodeBitFlagsCs::from_bits_truncate(NodeBitFlagsCs::UNK19.bits() | NodeBitFlagsCs::UNK24.bits());

const UNK040: &[u32] = &[
    0x00000000, // gamez (36636), planes (3317)
    0x00400000, // gamez (2)
    0x082AAAAA, // gamez (6980)
    0x10155555, // gamez (3220)
    0x80000000, // gamez (118)
    0x80400000, // gamez (193)
    0x882AAAAA, // gamez (533)
    0x886AAAAA, // gamez (2)
    0x90155555, // gamez (204)
    0x90555555, // gamez (7)
];
const UNK044: &[u32] = &[
    0x00000000, // gamez (49)
    0x00000001, // gamez (47837), planes (3310)
    0x0000E481, // planes (2)
    0x00013281, // planes (3)
    0x00014A81, // planes (2)
    0x0001FE81, // gamez (6)
    0x04000001, // gamez (3)
];

fn assert_node(node: NodeCsC, offset: usize) -> Result<(NodeType, NodeVariantsCs)> {
    // invariants for every node type

    let node_type = assert_that!("node type", enum NodeType => node.node_type, offset + 52)?;
    if node_type == NodeType::Empty {
        return Err(assert_with_msg!(
            "Expected valid node type, but was {} (at {})",
            node.node_type,
            offset + 52
        ));
    }

    let name = if node.name == GEOMETRY_NODE_NAME {
        GEOMETRY_NAME.to_string()
    } else if node.name == COCKPIT_NODE_NAME {
        COCKPIT_NAME.to_string()
    } else {
        assert_utf8("node name", offset + 0, || node.name.to_str_node_name())?
    };

    let flags = assert_that!("node flags (cs)", flags node.flags, offset + 36)?;
    let const_flags = flags.mask(ALWAYS_PRESENT);
    assert_that!("const flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    assert_that!("field 040", node.unk040 in UNK040, offset + 40)?;
    assert_that!("field 044", node.unk044 in UNK044, offset + 44)?;
    assert_that!("zone id", node.zone_id in [ZONE_DEFAULT, 1, 2, 3], offset + 48)?;
    // node_type (52) see above
    assert_that!("data ptr", node.data_ptr != Ptr::NULL, offset + 56)?;
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
        assert_that!(
            "area partition x",
            -1 <= node.area_partition.x <= 15,
            offset + 76
        )?;
        assert_that!(
            "area partition y",
            -1 <= node.area_partition.y <= 15,
            offset + 78
        )?;
        assert_that!(
            "area partition vx",
            0 <= node.area_partition.virtual_x <= 16,
            offset + 80
        )?;
        assert_that!(
            "area partition vy",
            0 <= node.area_partition.virtual_y <= 16,
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
    assert_that!("children count", 0 <= node.children_count <= 128, offset + 86)?;
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

    let variants = NodeVariantsCs {
        name,
        flags,
        unk040: node.unk040,
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

pub fn read_node_info(read: &mut CountingReader<impl Read>, index: u32) -> Result<NodeVariantCs> {
    debug!(
        "Reading node info {} (cs, {}) at {}",
        index,
        NodeCsC::SIZE,
        read.offset
    );
    let node: NodeCsC = read.read_struct()?;
    trace!("{:#?}", node);

    let (node_type, node) = assert_node(node, read.prev)?;
    debug!("Node `{}` read", node.name);
    let variant = match node_type {
        NodeType::World => world::assert_variants(node, read.prev)?,
        NodeType::Display => display::assert_variants(node, read.prev)?,
        NodeType::Window => window::assert_variants(node, read.prev)?,
        NodeType::Camera => camera::assert_variants(node, read.prev)?,
        NodeType::Light => light::assert_variants(node, read.prev)?,
        NodeType::LoD => lod::assert_variants(node, read.prev)?,
        NodeType::Object3d => object3d::assert_variants(node, read.prev)?,
        NodeType::Empty => unreachable!("empty nodes should not occur in cs"),
    };
    Ok(variant)
}

fn write_variant(
    write: &mut CountingWriter<impl Write>,
    node_type: NodeType,
    variant: NodeVariantsCs,
    index: usize,
) -> Result<()> {
    debug!(
        "Writing node info {} (cs, {}) at {}",
        index,
        NodeCsC::SIZE,
        write.offset
    );

    let name = if variant.name == GEOMETRY_NAME {
        Ascii::new(GEOMETRY_NODE_NAME)
    } else if variant.name == COCKPIT_NAME {
        Ascii::new(COCKPIT_NODE_NAME)
    } else {
        Ascii::from_str_node_name(&variant.name)
    };

    let area_partition = variant.area_partition.unwrap_or(AreaPartitionPm::DEFAULT);

    let node = NodeCsC {
        name,
        flags: variant.flags.maybe(),
        unk040: variant.unk040,
        unk044: variant.unk044,
        zone_id: variant.zone_id,
        node_type: node_type.into(),
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
    node: &NodeCs,
    index: usize,
) -> Result<()> {
    match node {
        NodeCs::World(world) => {
            let variant = world::make_variants(world)?;
            write_variant(write, NodeType::World, variant, index)
        }
        NodeCs::Display(display) => {
            let variant = display::make_variants(display);
            write_variant(write, NodeType::Display, variant, index)
        }
        NodeCs::Window(window) => {
            let variant = window::make_variants(window);
            write_variant(write, NodeType::Window, variant, index)
        }
        NodeCs::Camera(camera) => {
            let variant = camera::make_variants(camera);
            write_variant(write, NodeType::Camera, variant, index)
        }
        NodeCs::Light(light) => {
            let variant = light::make_variants(light);
            write_variant(write, NodeType::Light, variant, index)
        }
        NodeCs::Lod(lod) => {
            let variant = lod::make_variants(lod)?;
            write_variant(write, NodeType::LoD, variant, index)
        }
        NodeCs::Object3d(object3d) => {
            let variant = object3d::make_variants(object3d)?;
            write_variant(write, NodeType::Object3d, variant, index)
        }
    }
}

pub fn read_node_data(
    read: &mut CountingReader<impl Read>,
    variant: NodeVariantCs,
    node_index: u32,
    index: usize,
) -> Result<NodeCs> {
    match variant {
        NodeVariantCs::World {
            data_ptr,
            children_count,
            children_array_ptr,
        } => {
            let world = world::read(read, data_ptr, children_count, children_array_ptr, index)?;
            Ok(NodeCs::World(world))
        }
        NodeVariantCs::Display { data_ptr } => {
            let display = display::read(read, data_ptr, index)?;
            Ok(NodeCs::Display(display))
        }
        NodeVariantCs::Window { data_ptr, spyglass } => {
            let window = window::read(read, data_ptr, spyglass, index)?;
            Ok(NodeCs::Window(window))
        }
        NodeVariantCs::Camera { data_ptr, spyglass } => {
            let camera = camera::read(read, data_ptr, spyglass, index)?;
            Ok(NodeCs::Camera(camera))
        }
        NodeVariantCs::Light { data_ptr } => {
            let light = light::read(read, data_ptr, node_index, index)?;
            Ok(NodeCs::Light(light))
        }
        NodeVariantCs::Lod(lod) => {
            let lod = lod::read(read, lod, node_index, index)?;
            Ok(NodeCs::Lod(lod))
        }
        NodeVariantCs::Object3d(node) => {
            let object3d = object3d::read(read, node, node_index, index)?;
            Ok(NodeCs::Object3d(object3d))
        }
    }
}

pub fn write_node_data(
    write: &mut CountingWriter<impl Write>,
    node: &NodeCs,
    index: usize,
) -> Result<()> {
    match node {
        NodeCs::World(world) => world::write(write, world, index),
        NodeCs::Display(display) => display::write(write, display, index),
        NodeCs::Window(window) => window::write(write, window, index),
        NodeCs::Camera(camera) => camera::write(write, camera, index),
        NodeCs::Light(light) => light::write(write, light, index),
        NodeCs::Lod(lod) => lod::write(write, lod, index),
        NodeCs::Object3d(object3d) => object3d::write(write, object3d, index),
    }
}
