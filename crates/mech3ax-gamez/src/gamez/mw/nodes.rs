use super::NODE_INDEX_INVALID;
use mech3ax_api_types::nodes::mw::NodeMw;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, assert_with_msg, Result};
use mech3ax_nodes::mw::{
    read_node_data, read_node_info_gamez, read_node_info_zero, size_node, write_node_data,
    write_node_info, write_node_info_zero, NodeVariantMw, WrappedNodeMw, NODE_MW_C_SIZE,
};
use std::io::{Read, Write};

pub fn read_nodes(
    read: &mut CountingReader<impl Read>,
    array_size: u32,
    meshes_count: i32,
) -> Result<Vec<NodeMw>> {
    let end_offset = read.offset + NODE_MW_C_SIZE * array_size + 4 * array_size;

    let mut variants = Vec::new();
    // the node_count is wildly inaccurate for some files, and there are more nodes to
    // read after the provided count. so, we basically have to check the entire array
    let mut actual_count = array_size;
    let mut display_node = 0;
    let mut light_node: Option<u32> = None;
    for index in 0..array_size {
        let node_info_pos = read.offset;
        let variant = read_node_info_gamez(read, index)?;
        // this is an index for empty/zero nodes, and the offset for others
        let node_data_offset = read.read_u32()?;
        match variant {
            None => {
                let mut expected_index = index + 1;
                if expected_index == array_size {
                    expected_index = NODE_INDEX_INVALID;
                }
                assert_that!(
                    "node zero index",
                    node_data_offset == expected_index,
                    read.prev
                )?;

                actual_count = index + 1;
                break;
            }
            Some(variant) => {
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
                        // cannot be parented to world, window, camera, display, or light
                        // check for light parent later
                        assert_that!("empty parent index", 4 <= node_data_offset <= array_size, read.prev)?;
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
                                object3d.mesh_index < meshes_count,
                                node_info_pos
                            )?;
                        }
                    }
                }
                variants.push((variant, node_data_offset));
            }
        }
    }

    for index in actual_count..array_size {
        read_node_info_zero(read, index)?;
        let actual_index = read.read_u32()?;

        let mut expected_index = index + 1;
        if expected_index == array_size {
            expected_index = NODE_INDEX_INVALID;
        }
        assert_that!("node zero index", actual_index == expected_index, read.prev)?;
    }

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
                    assert_that!(
                        "node data offset",
                        read.offset == node_data_offset,
                        read.offset
                    )?;
                }
            }
            // node data is wrapped because of mechlib reading
            match read_node_data(read, variant, index)? {
                WrappedNodeMw::Camera(camera) => Ok(NodeMw::Camera(camera)),
                WrappedNodeMw::Display(display) => Ok(NodeMw::Display(display)),
                WrappedNodeMw::Empty(empty) => Ok(NodeMw::Empty(empty)),
                WrappedNodeMw::Light(light) => Ok(NodeMw::Light(light)),
                WrappedNodeMw::Window(window) => Ok(NodeMw::Window(window)),
                WrappedNodeMw::Lod(wrapped_lod) => {
                    let mut lod = wrapped_lod.wrapped;
                    lod.parent = read.read_u32()?;
                    lod.children = (0..wrapped_lod.children_count)
                        .map(|_| read.read_u32())
                        .collect::<std::io::Result<Vec<_>>>()?;
                    Ok(NodeMw::Lod(lod))
                }
                WrappedNodeMw::Object3d(wrapped_obj) => {
                    let mut object3d = wrapped_obj.wrapped;

                    object3d.parent = if wrapped_obj.has_parent {
                        Some(read.read_u32()?)
                    } else {
                        None
                    };
                    object3d.children = (0..wrapped_obj.children_count)
                        .map(|_| read.read_u32())
                        .collect::<std::io::Result<Vec<_>>>()?;
                    Ok(NodeMw::Object3d(object3d))
                }
                WrappedNodeMw::World(wrapped_world) => {
                    let mut world = wrapped_world.wrapped;
                    world.children = (0..wrapped_world.children_count)
                        .map(|_| read.read_u32())
                        .collect::<std::io::Result<Vec<_>>>()?;
                    Ok(NodeMw::World(world))
                }
            }
        })
        .collect::<Result<Vec<_>>>()?;

    read.assert_end()?;
    assert_area_partitions(&nodes, read.offset)?;
    Ok(nodes)
}

fn assert_area_partitions(nodes: &[NodeMw], offset: u32) -> Result<()> {
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

pub fn write_nodes(
    write: &mut CountingWriter<impl Write>,
    nodes: &[NodeMw],
    array_size: u32,
    offset: u32,
) -> Result<()> {
    let mut offset = offset + NODE_MW_C_SIZE * array_size + 4 * array_size;
    let node_count = assert_len!(u32, nodes.len(), "nodes")?;

    for (index, node) in nodes.iter().enumerate() {
        write_node_info(write, node, index)?;
        let index = match node {
            NodeMw::Empty(empty) => empty.parent,
            _ => offset,
        };
        write.write_u32(index)?;
        offset += size_node(node);
    }

    for index in node_count..array_size {
        write_node_info_zero(write, index)?;
        let mut index = index + 1;
        if index == array_size {
            index = NODE_INDEX_INVALID;
        }
        write.write_u32(index)?;
    }

    for (index, node) in nodes.iter().enumerate() {
        write_node_data(write, node, index)?;
        match node {
            NodeMw::Lod(lod) => {
                write.write_u32(lod.parent)?;
                for child in lod.children.iter().copied() {
                    write.write_u32(child)?;
                }
            }
            NodeMw::Object3d(object3d) => {
                if let Some(parent) = object3d.parent {
                    write.write_u32(parent)?;
                }
                for child in object3d.children.iter().copied() {
                    write.write_u32(child)?;
                }
            }
            NodeMw::World(world) => {
                for child in world.children.iter().copied() {
                    write.write_u32(child)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
