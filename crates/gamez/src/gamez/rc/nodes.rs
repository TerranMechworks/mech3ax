use super::{NODE_ARRAY_SIZE, NODE_INDEX_INVALID};
use log::trace;
use mech3ax_api_types::nodes::rc::NodeRc;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, assert_with_msg, Result};
use mech3ax_nodes::rc::{
    assert_node_info_zero, read_node_data, read_node_info, size_node, write_node_data,
    write_node_info, NodeRcC, NodeVariantRc,
};
use mech3ax_types::{u32_to_usize, AsBytes as _};
use std::io::{Read, Write};

pub(crate) fn read_nodes(
    read: &mut CountingReader<impl Read>,
    count: u32,
    meshes_count: i32,
) -> Result<Vec<NodeRc>> {
    let node_write_size = u32_to_usize(NodeRcC::SIZE) + 4;
    let valid_offset = read.offset + node_write_size * u32_to_usize(count);
    let end_offset = read.offset + node_write_size * u32_to_usize(NODE_ARRAY_SIZE);

    let mut variants = Vec::new();
    let mut light_node: Option<u32> = None;
    for index in 0..count {
        trace!("Reading node info {}/{}", index, count);

        let node_info_pos = read.offset;
        let variant = read_node_info(read)?;

        // this is an index for empty/zero nodes, and the offset for others
        let node_data_offset = read.read_u32()?;
        trace!("Node {} data offset: {}", index, node_data_offset);

        match &variant {
            NodeVariantRc::World { .. } => {
                assert_that!("node position (world)", index == 0, node_info_pos)?;
            }
            NodeVariantRc::Window { .. } => {
                assert_that!("node position (window)", index == 1, node_info_pos)?;
            }
            NodeVariantRc::Camera { .. } => {
                assert_that!("node position (camera)", index == 2, node_info_pos)?;
            }
            NodeVariantRc::Display { .. } => {
                assert_that!("node position (display)", index == 3, node_info_pos)?;
            }
            NodeVariantRc::Empty(_) => {
                // exclude world, window, camera, or display indices
                assert_that!("node position (empty)", index > 3, node_info_pos)?;
                // cannot be parented to world, window, camera, display, or light
                // check for light parent later
                assert_that!("empty parent index", node_data_offset > 3, read.prev)?;
            }
            NodeVariantRc::Light { .. } => {
                // exclude world, window, camera, or display indices
                assert_that!("node position (light)", index > 3, node_info_pos)?;
                if let Some(i) = light_node {
                    return Err(assert_with_msg!(
                        "Unexpected light node in position {}, already found in {} (at {})",
                        index,
                        i,
                        node_info_pos,
                    ));
                }
                light_node = Some(index);
            }
            NodeVariantRc::Lod(_) => {
                // exclude world, window, camera, or display indices
                assert_that!("node position (lod)", index > 3, node_info_pos)?;
            }
            NodeVariantRc::Object3d(object3d) => {
                // exclude world, window, camera, or display indices
                assert_that!("node position (object3d)", index > 3, node_info_pos)?;
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
    let light_node_index = light_node
        .ok_or_else(|| assert_with_msg!("GameZ contains no light node (at {})", read.offset))?;

    trace!(
        "Reading {}..{} node info zeros at {}",
        count,
        NODE_ARRAY_SIZE,
        read.offset
    );
    for index in count..NODE_ARRAY_SIZE {
        let node: NodeRcC = read.read_struct_no_log()?;
        assert_node_info_zero(&node, read.prev)
            .inspect_err(|_| trace!("{:#?} (index: {}, at {})", node, index, read.prev))?;

        let actual_index = read.read_u32()?;

        let mut expected_index = index + 1;
        if expected_index == NODE_ARRAY_SIZE {
            expected_index = NODE_INDEX_INVALID;
        }
        assert_that!("node zero index", actual_index == expected_index, read.prev)?;
    }
    trace!("Read node info zeros at {}", read.offset);

    assert_that!("node info end", end_offset == read.offset, read.offset)?;

    let nodes = variants
        .into_iter()
        .enumerate()
        .map(|(index, (mut variant, node_data_offset))| {
            match &mut variant {
                NodeVariantRc::Empty(empty) => {
                    assert_that!(
                        "empty parent index",
                        node_data_offset != light_node_index,
                        read.prev
                    )?;
                    // in the case of an empty node, the offset is used as the parent
                    // index, and not the offset (there is no node data)
                    empty.parent = node_data_offset;
                }
                _ => {
                    let offset = u32_to_usize(node_data_offset);
                    assert_that!("node data offset", read.offset == offset, read.offset)?;
                }
            }

            if !matches!(variant, NodeVariantRc::Empty(_)) {
                trace!("Reading node data {}/{}", index, count);
            }
            read_node_data(read, variant)
        })
        .collect::<Result<Vec<_>>>()?;

    read.assert_end()?;
    assert_area_partitions(&nodes, read.offset)?;

    Ok(nodes)
}

fn assert_area_partitions(nodes: &[NodeRc], offset: usize) -> Result<()> {
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

pub(crate) fn write_nodes(
    write: &mut CountingWriter<impl Write>,
    nodes: &[NodeRc],
    offset: u32,
) -> Result<()> {
    let mut offset = offset + (NodeRcC::SIZE + 4) * NODE_ARRAY_SIZE;
    let node_count = assert_len!(u32, nodes.len(), "nodes")?;

    for (index, node) in nodes.iter().enumerate() {
        trace!("Writing node info {}/{}", index, node_count);
        write_node_info(write, node)?;
        let node_data_offset = match node {
            NodeRc::Empty(empty) => empty.parent,
            _ => offset,
        };
        trace!("Node {} data offset: {}", index, node_data_offset);
        write.write_u32(node_data_offset)?;
        offset += size_node(node);
    }

    trace!(
        "Writing {}..{} node info zeros at {}",
        node_count,
        NODE_ARRAY_SIZE,
        write.offset
    );
    let node_zero = NodeRcC::zero();
    for index in node_count..NODE_ARRAY_SIZE {
        write.write_struct_no_log(&node_zero)?;
        let mut index = index + 1;
        if index == NODE_ARRAY_SIZE {
            index = NODE_INDEX_INVALID;
        }
        write.write_u32(index)?;
    }
    trace!("Wrote note info zeros at {}", write.offset);

    for (index, node) in nodes.iter().enumerate() {
        if !matches!(node, NodeRc::Empty(_)) {
            trace!("Writing node data {}/{}", index, node_count);
        }
        write_node_data(write, node)?;
    }

    Ok(())
}
