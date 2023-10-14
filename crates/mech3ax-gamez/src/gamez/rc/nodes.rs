use super::{NODE_ARRAY_SIZE, NODE_INDEX_INVALID};
use log::trace;
use mech3ax_api_types::nodes::rc::*;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, assert_with_msg, Result};
use mech3ax_nodes::rc::{
    read_node_data, read_node_info, read_node_info_zero, size_node, write_node_data,
    write_node_info, write_node_info_zero, NodeVariantRc, NODE_RC_C_SIZE,
};
use std::io::{Read, Write};

pub fn read_nodes(
    read: &mut CountingReader<impl Read>,
    count: u32,
    meshes_count: i32,
) -> Result<Vec<NodeRc>> {
    let valid_offset = read.offset + NODE_RC_C_SIZE * count + 4 * count;
    let end_offset = read.offset + NODE_RC_C_SIZE * NODE_ARRAY_SIZE + 4 * NODE_ARRAY_SIZE;
    // last general node index, excluding the light index (count - 1)
    let last_index = count - 2;

    let mut variants = Vec::new();
    for index in 0..count {
        let node_info_pos = read.offset;
        let variant = read_node_info(read, index)?;
        // this is an index for empty/zero nodes, and the offset for others
        let node_data_offset = read.read_u32()?;
        trace!("Node {} data offset: {}", index, node_data_offset);
        match &variant {
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
            NodeVariantRc::Empty(_) => {
                // exclude world, window, camera, display, or light indices
                assert_that!("node data position", 4 <= index <= last_index, node_info_pos)?;
                // cannot be parented to world, window, camera, display, or light
                assert_that!("empty ref index", 4 <= node_data_offset <= last_index, read.prev)?;
            }
            NodeVariantRc::Light { data_ptr: _ } => {
                assert_that!("node data position", index == count - 1, node_info_pos)?;
            }
            NodeVariantRc::Lod(_) => {
                // exclude world, window, camera, display, or light indices
                assert_that!("node data position", 4 <= index <= last_index, node_info_pos)?;
            }
            NodeVariantRc::Object3d(object3d) => {
                // exclude world, window, camera, display, or light indices
                assert_that!("node data position", 4 <= index <= last_index, node_info_pos)?;
                if object3d.mesh_index >= 0 {
                    assert_that!(
                        "object3d mesh index",
                        object3d.mesh_index < meshes_count,
                        node_info_pos
                    )?;
                }
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
            expected_index = NODE_INDEX_INVALID;
        }
        assert_that!("node zero index", actual_index == expected_index, read.prev)?;
    }

    assert_that!("node info end", end_offset == read.offset, read.offset)?;

    let nodes = variants
        .into_iter()
        .enumerate()
        .map(|(index, (mut variant, node_data_offset))| {
            match &mut variant {
                NodeVariantRc::Empty(empty) => {
                    // in the case of an empty node, the offset is used as the parent
                    // index, and not the offset (there is no node data)
                    empty.parent = node_data_offset;
                }
                _ => {
                    assert_that!(
                        "node data offset",
                        read.offset == node_data_offset,
                        read.offset
                    )?;
                }
            }
            read_node_data(read, variant, index)
        })
        .collect::<Result<Vec<_>>>()?;

    read.assert_end()?;
    assert_area_partitions(&nodes, read.offset)?;

    Ok(nodes)
}

fn assert_area_partitions(nodes: &[NodeRc], offset: u32) -> Result<()> {
    let (x_count, y_count) = match nodes.first() {
        Some(NodeRc::World(world)) => Ok((
            world.virt_partition_x_count as i32,
            world.virt_partition_y_count as i32,
        )),
        Some(_) => Err(assert_with_msg!("Expected the world node to be first")),
        None => Err(assert_with_msg!("Expected to have read some nodes")),
    }?;

    for node in nodes {
        let area_partition = match node {
            NodeRc::Object3d(object3d) => &object3d.area_partition,
            _ => &None,
        };
        if let Some(ap) = area_partition {
            assert_that!("partition x", ap.x < x_count, offset)?;
            assert_that!("partition y", ap.y < y_count, offset)?;
        }
    }

    Ok(())
}

pub fn write_nodes(
    write: &mut CountingWriter<impl Write>,
    nodes: &[NodeRc],
    offset: u32,
) -> Result<()> {
    let mut offset = offset + NODE_RC_C_SIZE * NODE_ARRAY_SIZE + 4 * NODE_ARRAY_SIZE;
    let node_count = assert_len!(u32, nodes.len(), "nodes")?;

    for (index, node) in nodes.iter().enumerate() {
        write_node_info(write, node, index)?;
        let node_data_offset = match node {
            NodeRc::Empty(empty) => empty.parent,
            _ => offset,
        };
        trace!("Node {} data offset: {}", index, node_data_offset);
        write.write_u32(node_data_offset)?;
        offset += size_node(node);
    }

    for index in node_count..NODE_ARRAY_SIZE {
        write_node_info_zero(write, index)?;
        let mut index = index + 1;
        if index == NODE_ARRAY_SIZE {
            index = NODE_INDEX_INVALID;
        }
        write.write_u32(index)?;
    }

    for (index, node) in nodes.iter().enumerate() {
        write_node_data(write, node, index)?;
    }

    Ok(())
}
