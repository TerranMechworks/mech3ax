use super::NODE_ARRAY_SIZE;
use mech3ax_api_types::nodes::rc::*;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_nodes::rc::{
    read_node_info, read_node_info_zero, size_node, write_node_info, write_node_info_zero,
    NodeVariantRc, NODE_RC_C_SIZE,
};
use std::io::{Read, Write};

pub fn read_nodes(
    read: &mut CountingReader<impl Read>,
    count: u32,
) -> Result<(Vec<NodeRc>, Vec<u8>)> {
    let valid_offset = read.offset + NODE_RC_C_SIZE * count + 4 * count;
    let end_offset = read.offset + NODE_RC_C_SIZE * NODE_ARRAY_SIZE + 4 * NODE_ARRAY_SIZE;

    let mut variants = Vec::new();
    let mut light_node = false;
    for index in 0..count {
        let node_info_pos = read.offset;
        let mut variant = read_node_info(read, index)?;
        // this is an index for empty/zero nodes, and the offset for others
        let node_data_offset = read.read_u32()?;
        log::debug!("Node {} data offset: {}", index, node_data_offset);
        match &mut variant {
            NodeVariantRc::World {
                data_ptr: _,
                children_count: _,
                children_array_ptr: _,
            } => {
                assert_that!("node data position", index == 0, node_info_pos)?;
            }
            NodeVariantRc::Window { data_ptr: _ } => {
                assert_that!("node data position", index == 1, node_info_pos)?;
            }
            NodeVariantRc::Camera { data_ptr: _ } => {
                assert_that!("node data position", index == 2, node_info_pos)?;
            }
            NodeVariantRc::Display { data_ptr: _ } => {
                assert_that!("node data position", index == 3, node_info_pos)?;
            }
            NodeVariantRc::Empty(empty) => {
                assert_that!("node data position", index > 3, node_info_pos)?;
                assert_that!("empty ref index", 4 <= node_data_offset <= NODE_ARRAY_SIZE, read.prev)?;
                empty.parent = node_data_offset;
            }
            NodeVariantRc::Light { data_ptr: _ } => {
                assert_that!("node data position", index > 3, node_info_pos)?;
                if light_node {
                    return Err(assert_with_msg!(
                        "Unexpected light node in position {} (at {})",
                        index,
                        node_info_pos
                    ));
                }
                light_node = true;
            }
            _ => {
                assert_that!("node data position", index > 3, node_info_pos)?;
            }
        }
        variants.push((variant, node_data_offset));
    }

    assert_that!("node info valid", valid_offset == read.offset, read.offset)?;

    for index in count..NODE_ARRAY_SIZE {
        read_node_info_zero(read, index)?;
        let actual_index = read.read_u32()?;

        let mut expected_index = index + 1;
        if expected_index == NODE_ARRAY_SIZE {
            // we'll never know why???
            expected_index = 0xFFFFFF;
        }
        assert_that!("node zero index", actual_index == expected_index, read.prev)?;
    }

    assert_that!("node info end", end_offset == read.offset, read.offset)?;

    let nodes = variants
        .into_iter()
        .enumerate()
        .map(|(index, (variant, _node_data_offset))| {
            match variant {
                NodeVariantRc::Camera { data_ptr } => Ok(NodeRc::Camera(Camera { data_ptr })),
                NodeVariantRc::Display { data_ptr } => Ok(NodeRc::Display(Display { data_ptr })),
                NodeVariantRc::Empty(empty) => Ok(NodeRc::Empty(empty)),
                NodeVariantRc::Light { data_ptr } => Ok(NodeRc::Light(Light { data_ptr })),
                NodeVariantRc::Window { data_ptr } => Ok(NodeRc::Window(Window { data_ptr })),
                NodeVariantRc::Lod(lod) => {
                    let lod = Lod {
                        name: lod.name,
                        flags: lod.flags.into(),
                        zone_id: lod.zone_id,
                        // area_partition: lod.area_partition,
                        // parent: lod.parent,
                        children: (0..lod.children_count).collect(),
                        data_ptr: lod.data_ptr,
                        parent_array_ptr: lod.parent_array_ptr,
                        children_array_ptr: lod.children_array_ptr,
                        unk116: lod.unk116,
                    };
                    Ok(NodeRc::Lod(lod))
                }
                NodeVariantRc::Object3d(object3d) => {
                    let object3d = Object3d {
                        name: object3d.name,
                        flags: object3d.flags.into(),
                        zone_id: object3d.zone_id,
                        area_partition: object3d.area_partition,
                        mesh_index: object3d.mesh_index,
                        parent: if object3d.has_parent { Some(0) } else { None },
                        children: (0..object3d.children_count).collect(),
                        data_ptr: object3d.data_ptr,
                        parent_array_ptr: object3d.parent_array_ptr,
                        children_array_ptr: object3d.children_array_ptr,
                        unk116: object3d.unk116,
                        unk140: object3d.unk140,
                        unk164: object3d.unk164,
                    };
                    Ok(NodeRc::Object3d(object3d))
                }
                NodeVariantRc::World {
                    data_ptr,
                    children_count,
                    children_array_ptr,
                } => {
                    log::debug!("Reading world data at {}", read.offset);
                    let wrapped_world = mech3ax_nodes::rc::world::read(
                        read,
                        data_ptr,
                        children_count,
                        children_array_ptr,
                        index,
                    )?;
                    let mut world = wrapped_world.wrapped;
                    log::debug!("Reading world children at {}", read.offset);
                    world.children = (0..wrapped_world.children_count)
                        .map(|_| read.read_u32())
                        .collect::<std::io::Result<Vec<_>>>()?;
                    Ok(NodeRc::World(world))
                }
            }
        })
        .collect::<Result<Vec<_>>>()?;

    log::debug!("would read node data at {}", read.offset);

    let node_data = read.read_to_end()?;
    read.assert_end()?;
    // assert_area_partitions(&nodes, read.offset)?;

    Ok((nodes, node_data))
}

pub fn write_nodes(
    write: &mut CountingWriter<impl Write>,
    nodes: &[NodeRc],
    offset: u32,
) -> Result<()> {
    let mut offset = offset + NODE_RC_C_SIZE * NODE_ARRAY_SIZE + 4 * NODE_ARRAY_SIZE;

    for (index, node) in nodes.iter().enumerate() {
        write_node_info(write, node, index)?;
        let node_data_offset = match node {
            NodeRc::Empty(empty) => empty.parent,
            _ => offset,
        };
        log::debug!("Node {} data offset: {}", index, node_data_offset);
        write.write_u32(node_data_offset)?;
        offset += size_node(node);
    }

    let node_count = nodes.len() as u32;

    for index in node_count..NODE_ARRAY_SIZE {
        write_node_info_zero(write, index)?;
        let mut index = index + 1;
        if index == NODE_ARRAY_SIZE {
            index = 0xFFFFFF;
        }
        write.write_u32(index)?;
    }

    for (index, node) in nodes.iter().enumerate() {
        match node {
            NodeRc::World(world) => {
                mech3ax_nodes::rc::world::write(write, world, index)?;
                for child in &world.children {
                    write.write_u32(*child)?;
                }
            }
            _ => {}
        }
        //     write_node_data(write, node, index)?;
        //     match node {
        //         NodeMw::Lod(lod) => {
        //             write.write_u32(lod.parent)?;
        //             for child in &lod.children {
        //                 write.write_u32(*child)?;
        //             }
        //         }
        //         NodeMw::Object3d(object3d) => {
        //             if let Some(parent) = object3d.parent {
        //                 write.write_u32(parent)?;
        //             }
        //             for child in &object3d.children {
        //                 write.write_u32(*child)?;
        //             }
        //         }
        //         NodeMw::World(world) => {
        //             for child in &world.children {
        //                 write.write_u32(*child)?;
        //             }
        //         }
        //         _ => {}
        //     }
    }

    Ok(())
}
