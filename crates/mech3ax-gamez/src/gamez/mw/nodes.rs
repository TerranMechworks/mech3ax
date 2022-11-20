use mech3ax_api_types::gamez::NodeMw;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_nodes::mw::{
    read_node_data, read_node_info_gamez, read_node_info_zero, size_node, write_node_data,
    write_node_info, write_node_info_zero, WrappedNodeMw, NODE_MW_C_SIZE,
};
use mech3ax_nodes::NodeVariantMw;
use std::io::{Read, Write};

pub fn read_nodes(read: &mut CountingReader<impl Read>, array_size: u32) -> Result<Vec<NodeMw>> {
    let end_offset = read.offset + NODE_MW_C_SIZE * array_size + 4 * array_size;

    let mut variants = Vec::new();
    // the node_count is wildly inaccurate for some files, and there are more nodes to
    // read after the provided count. so, we basically have to check the entire array
    let mut actual_count = array_size;
    let mut display_node = 0;
    let mut light_node = false;
    for index in 0..array_size {
        let node_info_pos = read.offset;
        let variant = read_node_info_gamez(read, index)?;
        // this is an index for empty/zero nodes, and the offset for others
        let actual_index = read.read_u32()?;
        match variant {
            None => {
                let mut expected_index = index + 1;
                if expected_index == array_size {
                    // we'll never know why???
                    expected_index = 0xFFFFFF
                }
                assert_that!("node zero index", actual_index == expected_index, read.prev)?;

                actual_count = index + 1;
                break;
            }
            Some(mut variant) => {
                match &mut variant {
                    NodeVariantMw::World(_, _, _) => {
                        assert_that!("node data position", index == 0, node_info_pos)?;
                    }
                    NodeVariantMw::Window(_) => {
                        assert_that!("node data position", index == 1, node_info_pos)?;
                    }
                    NodeVariantMw::Camera(_) => {
                        assert_that!("node data position", index == 2, node_info_pos)?;
                    }
                    NodeVariantMw::Display(_) => {
                        match display_node {
                            0 => assert_that!("node data position", index == 3, node_info_pos)?,
                            1 => assert_that!("node data position", index == 4, node_info_pos)?,
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
                    NodeVariantMw::Empty(empty) => {
                        assert_that!("node data position", index > 3, node_info_pos)?;
                        assert_that!("empty ref index", 4 <= actual_index <= array_size, read.prev)?;
                        empty.parent = actual_index;
                    }
                    NodeVariantMw::Light(_) => {
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
                variants.push((variant, actual_index));
            }
        }
    }

    for index in actual_count..array_size {
        read_node_info_zero(read, index)?;
        let actual_index = read.read_u32()?;

        let mut expected_index = index + 1;
        if expected_index == array_size {
            // we'll never know why???
            expected_index = 0xFFFFFF;
        }
        assert_that!("node zero index", actual_index == expected_index, read.prev)?;
    }

    assert_that!("node info end", end_offset == read.offset, read.offset)?;

    let nodes = variants
        .into_iter()
        .enumerate()
        .map(|(index, (variant, offset))| {
            match &variant {
                NodeVariantMw::Empty(_) => {
                    // in the case of an empty node, the offset is used as the parent
                    // index, and not the offset (there is no node data)
                }
                _ => {
                    assert_that!("node data offset", offset == read.offset, read.offset)?;
                }
            }
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
            let x = ap.x;
            let y = ap.y;
            assert_that!("partition x", x < x_count, offset)?;
            assert_that!("partition y", y < y_count, offset)?;
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

    for (index, node) in nodes.iter().enumerate() {
        write_node_info(write, node, index)?;
        let index = match node {
            NodeMw::Empty(empty) => empty.parent,
            _ => offset,
        };
        write.write_u32(index)?;
        offset += size_node(node);
    }

    let node_count = nodes.len() as u32;

    for index in node_count..array_size {
        write_node_info_zero(write, index)?;
        let mut index = index + 1;
        if index == array_size {
            index = 0xFFFFFF;
        }
        write.write_u32(index)?;
    }

    for (index, node) in nodes.iter().enumerate() {
        write_node_data(write, node, index)?;
        match node {
            NodeMw::Lod(lod) => {
                write.write_u32(lod.parent)?;
                for child in &lod.children {
                    write.write_u32(*child)?;
                }
            }
            NodeMw::Object3d(object3d) => {
                if let Some(parent) = object3d.parent {
                    write.write_u32(parent)?;
                }
                for child in &object3d.children {
                    write.write_u32(*child)?;
                }
            }
            NodeMw::World(world) => {
                for child in &world.children {
                    write.write_u32(*child)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
