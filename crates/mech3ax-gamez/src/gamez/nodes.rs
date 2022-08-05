use crate::nodes::{
    read_node_data, read_node_info_gamez, read_node_info_zero, size_node, write_node_data,
    write_node_info, write_node_info_zero, NodeVariant, WrappedNode, NODE_C_SIZE,
};
use mech3ax_api_types::{IndexedNode, Node};
use mech3ax_common::assert::AssertionError;
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::{assert_that, Error, Result};
use std::io::{Read, Write};

pub fn read_nodes<R>(read: &mut CountingReader<R>, array_size: u32) -> Result<Vec<IndexedNode>>
where
    R: Read,
{
    let end_offset = read.offset + NODE_C_SIZE * array_size + 4 * array_size;

    let mut variants = Vec::new();
    // the node_count is wildly inaccurate for some files, and there are more nodes to
    // read after the provided count. so, we basically have to check the entire array
    let mut actual_count = array_size;
    let mut display_node = 0;
    let mut light_node = false;
    for i in 0..array_size {
        let node_info_pos = read.offset;
        let variant = read_node_info_gamez(read)?;
        // this is an index for empty/zero nodes, and the offset for others
        let actual_index = read.read_u32()?;
        match variant {
            None => {
                let mut expected_index = i + 1;
                if expected_index == array_size {
                    // we'll never know why???
                    expected_index = 0xFFFFFF
                }
                assert_that!("node zero index", actual_index == expected_index, read.prev)?;

                actual_count = i + 1;
                break;
            }
            Some(mut variant) => {
                match &mut variant {
                    NodeVariant::World(_, _, _) => {
                        assert_that!("node data position", i == 0, node_info_pos)?;
                    }
                    NodeVariant::Window(_) => {
                        assert_that!("node data position", i == 1, node_info_pos)?;
                    }
                    NodeVariant::Camera(_) => {
                        assert_that!("node data position", i == 2, node_info_pos)?;
                    }
                    NodeVariant::Display(_) => {
                        match display_node {
                            0 => assert_that!("node data position", i == 3, node_info_pos)?,
                            1 => assert_that!("node data position", i == 4, node_info_pos)?,
                            _ => {
                                return Err(Error::Assert(AssertionError(format!(
                                    "Unexpected display node in position {} (at {})",
                                    i, node_info_pos
                                ))))
                            }
                        }
                        display_node += 1;
                    }
                    NodeVariant::Empty(empty) => {
                        assert_that!("node data position", i > 3, node_info_pos)?;
                        assert_that!("empty ref index", 4 <= actual_index <= array_size, read.prev)?;
                        empty.parent = actual_index;
                    }
                    NodeVariant::Light(_) => {
                        assert_that!("node data position", i > 3, node_info_pos)?;
                        if light_node {
                            return Err(Error::Assert(AssertionError(format!(
                                "Unexpected light node in position {} (at {})",
                                i, node_info_pos
                            ))));
                        }
                        light_node = true;
                    }
                    _ => {
                        assert_that!("node data position", i > 3, node_info_pos)?;
                    }
                }
                variants.push((variant, actual_index));
            }
        }
    }

    for i in actual_count..array_size {
        read_node_info_zero(read)?;
        let actual_index = read.read_u32()?;

        let mut expected_index = i + 1;
        if expected_index == array_size {
            // we'll never know why???
            expected_index = 0xFFFFFF;
        }
        assert_that!("node zero index", actual_index == expected_index, read.prev)?;
    }

    assert_that!("node info end", end_offset == read.offset, read.offset)?;

    let nodes = variants
        .into_iter()
        .map(|(variant, offset)| {
            match &variant {
                NodeVariant::Empty(_) => {
                    // in the case of an empty node, the offset is used as the parent
                    // index, and not the offset (there is no node data)
                }
                _ => {
                    assert_that!("node data offset", offset == read.offset, read.offset)?;
                }
            }
            match read_node_data(read, variant)? {
                WrappedNode::Camera(camera) => Ok(Node::Camera(camera)),
                WrappedNode::Display(display) => Ok(Node::Display(display)),
                WrappedNode::Empty(empty) => Ok(Node::Empty(empty)),
                WrappedNode::Light(light) => Ok(Node::Light(light)),
                WrappedNode::Window(window) => Ok(Node::Window(window)),
                WrappedNode::Lod(wrapped_lod) => {
                    let mut lod = wrapped_lod.wrapped;
                    lod.parent = read.read_u32()?;
                    lod.children = (0..wrapped_lod.children_count)
                        .map(|_| read.read_u32())
                        .collect::<std::io::Result<Vec<_>>>()?;
                    Ok(Node::Lod(lod))
                }
                WrappedNode::Object3d(wrapped_obj) => {
                    let mut object3d = wrapped_obj.wrapped;

                    object3d.parent = if wrapped_obj.has_parent {
                        Some(read.read_u32()?)
                    } else {
                        None
                    };
                    object3d.children = (0..wrapped_obj.children_count)
                        .map(|_| read.read_u32())
                        .collect::<std::io::Result<Vec<_>>>()?;
                    Ok(Node::Object3d(object3d))
                }
                WrappedNode::World(wrapped_world) => {
                    let mut world = wrapped_world.wrapped;
                    world.children = (0..wrapped_world.children_count)
                        .map(|_| read.read_u32())
                        .collect::<std::io::Result<Vec<_>>>()?;
                    Ok(Node::World(world))
                }
            }
        })
        .collect::<Result<Vec<_>>>()?;

    read.assert_end()?;
    assert_area_partitions(&nodes, read.offset)?;
    Ok(nodes)
}

fn assert_area_partitions(nodes: &[Node<u32>], offset: u32) -> Result<()> {
    let (x_count, y_count) =
        if let Node::World(world) = nodes.first().expect("Expected to have read some nodes") {
            (
                world.area_partition_x_count as i32,
                world.area_partition_y_count as i32,
            )
        } else {
            return Err(AssertionError("Expected the world node to be first".to_owned()).into());
        };

    for node in nodes {
        let area_partition = match node {
            Node::Lod(lod) => &lod.area_partition,
            Node::Object3d(object3d) => &object3d.area_partition,
            _ => &None,
        };
        if let Some(ap) = area_partition {
            let x = ap.0;
            let y = ap.1;
            assert_that!("partition x", x < x_count, offset)?;
            assert_that!("partition y", y < y_count, offset)?;
        }
    }

    Ok(())
}

pub fn write_nodes<W>(
    write: &mut W,
    nodes: &[Node<u32>],
    array_size: u32,
    offset: u32,
) -> Result<()>
where
    W: Write,
{
    let mut offset = offset + NODE_C_SIZE * array_size + 4 * array_size;

    for node in nodes {
        write_node_info(write, node)?;
        let index = match node {
            Node::Empty(empty) => empty.parent,
            _ => offset,
        };
        write.write_u32(index)?;
        offset += size_node(node);
    }

    let node_count = nodes.len() as u32;

    for i in node_count..array_size {
        write_node_info_zero(write)?;
        let mut index = i + 1;
        if index == array_size {
            index = 0xFFFFFF;
        }
        write.write_u32(index)?;
    }

    for node in nodes {
        write_node_data(write, node)?;
        match node {
            Node::Lod(lod) => {
                write.write_u32(lod.parent)?;
                for child in &lod.children {
                    write.write_u32(*child)?;
                }
            }
            Node::Object3d(object3d) => {
                if let Some(parent) = object3d.parent {
                    write.write_u32(parent)?;
                }
                for child in &object3d.children {
                    write.write_u32(*child)?;
                }
            }
            Node::World(world) => {
                for child in &world.children {
                    write.write_u32(*child)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
