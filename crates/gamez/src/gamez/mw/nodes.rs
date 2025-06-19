use super::NODE_INDEX_INVALID;
use log::trace;
use mech3ax_api_types::nodes::mw::NodeMw;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, assert_with_msg, Result};
use mech3ax_nodes::common::{read_child_indices, write_child_indices};
use mech3ax_nodes::mw::{
    assert_node_info_zero, read_node_data, read_node_info_gamez, size_node, write_node_data,
    write_node_info, NodeMwC, NodeVariantMw, WrappedNodeMw,
};
use mech3ax_types::{i32_to_usize, u32_to_usize, AsBytes as _};
use std::io::{Read, Seek, Write};

pub(crate) fn read_nodes(
    read: &mut CountingReader<impl Read + Seek>,
    array_size: i32,
    model_count: i32,
) -> Result<Vec<NodeMw>> {
    let end_offset = read.offset + (u32_to_usize(NodeMwC::SIZE) + 4) * i32_to_usize(array_size);

    let mut variants = Vec::new();
    // the node_count is wildly inaccurate for some files, and there are more nodes to
    // read after the provided count. so, we basically have to check the entire array
    let mut actual_count = array_size;
    let mut display_node = 0;
    let mut light_node: Option<i32> = None;
    for index in 0..array_size {
        trace!("Processing node info {}/?", index);

        let node_info_pos = read.offset;
        let variant = read_node_info_gamez(read)?;

        match variant {
            None => {
                actual_count = index;
                break;
            }
            Some(variant) => {
                // this is an index for empty/zero nodes, and the offset for others
                let node_data_offset = read.read_u32()?;
                trace!("Node {} data offset: {}", index, node_data_offset);

                match &variant {
                    NodeVariantMw::World { .. } => {
                        assert_that!("node position (world)", index == 0, node_info_pos)?;
                    }
                    NodeVariantMw::Window { .. } => {
                        assert_that!("node position (window)", index == 1, node_info_pos)?;
                    }
                    NodeVariantMw::Camera { .. } => {
                        assert_that!("node position (camera)", index == 2, node_info_pos)?;
                    }
                    NodeVariantMw::Display { .. } => {
                        match display_node {
                            0 => {
                                assert_that!("node position (display)", index == 3, node_info_pos)?
                            }
                            1 => {
                                assert_that!("node position (display)", index == 4, node_info_pos)?
                            }
                            _ => {
                                return Err(assert_with_msg!(
                                    "Unexpected display node in position {} (at {})",
                                    index,
                                    node_info_pos
                                ));
                            }
                        }
                        display_node += 1;
                    }
                    NodeVariantMw::Empty(_) => {
                        // exclude world, window, camera, or display indices
                        assert_that!("node position (empty)", index > 3, node_info_pos)?;
                        let parent_index = node_data_offset as i32;
                        // cannot be parented to world, window, camera, display, or light
                        // check for light parent later
                        assert_that!("empty parent index", 4 <= parent_index <= array_size, read.prev)?;
                    }
                    NodeVariantMw::Light { .. } => {
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
                    NodeVariantMw::Lod(_) => {
                        // exclude world, window, camera, or display indices
                        assert_that!("node position (lod)", index > 3, node_info_pos)?;
                    }
                    NodeVariantMw::Object3d(object3d) => {
                        // exclude world, window, camera, or display indices
                        assert_that!("node position (object3d)", index > 3, node_info_pos)?;
                        if object3d.mesh_index >= 0 {
                            assert_that!(
                                "object3d mesh index",
                                object3d.mesh_index < model_count,
                                node_info_pos
                            )?;
                        }
                    }
                }
                variants.push((variant, node_data_offset));
            }
        }
    }

    trace!(
        "Processing {}..{} node info zeros at {}",
        actual_count,
        array_size,
        read.offset
    );
    for index in actual_count..array_size {
        let node: NodeMwC = read.read_struct_no_log()?;
        assert_node_info_zero(&node, read.prev)
            .inspect_err(|_| trace!("{:#?} (index: {}, at {})", node, index, read.prev))?;

        let actual_index = read.read_i32()?;

        let mut expected_index = index + 1;
        if expected_index == array_size {
            expected_index = NODE_INDEX_INVALID;
        }
        assert_that!("node zero index", actual_index == expected_index, read.prev)?;
    }
    trace!("Processed node info zeros at {}", read.offset);

    assert_that!("node info end", end_offset == read.offset, read.offset)?;
    assert_that!("has display node", display_node > 0, read.offset)?;
    let light_node_index = light_node
        .ok_or_else(|| assert_with_msg!("GameZ contains no light node (at {})", read.offset))?;

    let nodes = variants
        .into_iter()
        .enumerate()
        .map(|(index, (mut variant, node_data_offset))| {
            match &mut variant {
                NodeVariantMw::Empty(empty) => {
                    let parent_index = node_data_offset as i32;
                    assert_that!(
                        "empty parent index",
                        parent_index != light_node_index,
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

            if !matches!(variant, NodeVariantMw::Empty(_)) {
                trace!("Processing node data {}/{}", index, actual_count);
            }
            // node data is wrapped because of mechlib reading
            match read_node_data(read, variant)? {
                WrappedNodeMw::Camera(camera) => Ok(NodeMw::Camera(camera)),
                WrappedNodeMw::Display(display) => Ok(NodeMw::Display(display)),
                WrappedNodeMw::Empty(empty) => Ok(NodeMw::Empty(empty)),
                WrappedNodeMw::Light(light) => Ok(NodeMw::Light(light)),
                WrappedNodeMw::Window(window) => Ok(NodeMw::Window(window)),
                WrappedNodeMw::Lod(wrapped_lod) => {
                    let mut lod = wrapped_lod.wrapped;
                    lod.parent = read.read_u32()?;
                    lod.children = read_child_indices(read, wrapped_lod.children_count)?;
                    Ok(NodeMw::Lod(lod))
                }
                WrappedNodeMw::Object3d(wrapped_obj) => {
                    let mut object3d = wrapped_obj.wrapped;
                    object3d.parent = if wrapped_obj.has_parent {
                        Some(read.read_u32()?)
                    } else {
                        None
                    };
                    object3d.children = read_child_indices(read, wrapped_obj.children_count)?;
                    Ok(NodeMw::Object3d(object3d))
                }
                WrappedNodeMw::World(wrapped_world) => {
                    let mut world = wrapped_world.wrapped;
                    world.children = read_child_indices(read, wrapped_world.children_count)?;
                    Ok(NodeMw::World(world))
                }
            }
        })
        .collect::<Result<Vec<_>>>()?;

    assert_area_partitions(&nodes, read.offset)?;
    Ok(nodes)
}

fn assert_area_partitions(nodes: &[NodeMw], offset: usize) -> Result<()> {
    let (x_count, y_count) = match nodes.first() {
        Some(NodeMw::World(world)) => Ok((
            world.area_partition_x_count as i32,
            world.area_partition_y_count as i32,
        )),
        Some(_) => Err(assert_with_msg!("Expected the world node to be first")),
        None => Err(assert_with_msg!("Expected to have read some nodes")),
    }?;

    for node in nodes {
        let area_partition = match node {
            NodeMw::Lod(lod) => &lod.area_partition,
            NodeMw::Object3d(object3d) => &object3d.area_partition,
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
    nodes: &[NodeMw],
    array_size: i32,
    offset: u32,
) -> Result<()> {
    let mut offset = offset + (NodeMwC::SIZE + 4) * (array_size as u32);
    let node_count = assert_len!(i32, nodes.len(), "nodes")?;

    for (index, node) in nodes.iter().enumerate() {
        trace!("Processing node info {}/{}", index, node_count);
        write_node_info(write, node)?;

        let node_data_offset = match node {
            NodeMw::Empty(empty) => empty.parent,
            _ => offset,
        };

        trace!("Node {} data offset: {}", index, node_data_offset);
        write.write_u32(node_data_offset)?;
        offset += size_node(node);
    }
    // padding for spurious "Processing node info", since the count is not known
    // and we have to peek for zero node infos.
    trace!("");

    trace!(
        "Processing {}..{} node info zeros at {}",
        node_count,
        array_size,
        write.offset
    );
    let node_zero = NodeMwC::zero();
    for index in node_count..array_size {
        write.write_struct_no_log(&node_zero)?;
        let mut index = index + 1;
        if index == array_size {
            index = NODE_INDEX_INVALID;
        }
        write.write_i32(index)?;
    }
    trace!("Processed note info zeros at {}", write.offset);

    for (index, node) in nodes.iter().enumerate() {
        if !matches!(node, NodeMw::Empty(_)) {
            trace!("Processing node data {}/{}", index, node_count);
        }
        write_node_data(write, node)?;
        match node {
            NodeMw::Lod(lod) => {
                write.write_u32(lod.parent)?;
                write_child_indices(write, &lod.children)?;
            }
            NodeMw::Object3d(object3d) => {
                if let Some(parent) = object3d.parent {
                    write.write_u32(parent)?;
                }
                write_child_indices(write, &object3d.children)?;
            }
            NodeMw::World(world) => {
                write_child_indices(write, &world.children)?;
            }
            _ => {}
        }
    }

    Ok(())
}
