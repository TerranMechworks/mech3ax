use crate::assert::AssertionError;
use crate::io_ext::{ReadHelper, WriteHelper};
use crate::nodes::{
    read_node_data, read_node_info, read_node_info_zero, size_node, write_node_data,
    write_node_info, write_node_info_zero, Node, NodeVariant, WrappedNode, NODE_C_SIZE,
};
use crate::{assert_that, Result};
use std::io::{Read, Write};

pub fn read_nodes<R>(
    read: &mut R,
    offset: &mut u32,
    count: u32,
    array_size: u32,
) -> Result<Vec<Node>>
where
    R: Read,
{
    let end_offset = *offset + NODE_C_SIZE * array_size + 4 * array_size;

    // the node_count is wildly inaccurate for some files, and there are more nodes to
    // read after the provided count. so, we basically have to check the entire array
    let mut variants = Vec::new();
    for i in 0..count {
        let variant = read_node_info(read, offset)?;
        let index = read.read_u32()?;
        *offset += 4;
        match &variant {
            NodeVariant::World(_, _, _) => {
                assert_that!("node data position", i == 0, *offset)?;
            }
            NodeVariant::Window(_) => {
                assert_that!("node data position", i == 1, *offset)?;
            }
            NodeVariant::Camera(_) => {
                assert_that!("node data position", i == 2, *offset)?;
            }
            _ => {}
        }
        variants.push((variant, index));
        // TODO: break
    }

    for i in count..array_size {
        read_node_info_zero(read, offset)?;
        let actual_index = read.read_u32()?;
        *offset += 4;

        let mut expected_index = i + 1;
        if expected_index == array_size {
            // we'll never know why???
            expected_index = 0xFFFFFF
        }
        assert_that!("node zero index", actual_index == expected_index, *offset)?;
    }

    assert_that!("node info end", end_offset == *offset, *offset)?;

    let nodes = variants
        .into_iter()
        .map(|(mut variant, index)| {
            match &mut variant {
                NodeVariant::Empty(empty) => {
                    assert_that!("empty ref index", 4 <= index <= array_size, *offset)?;
                    empty.parent = index;
                }
                _ => {
                    // it's more likely our read logic is wrong than this data being wrong
                    let actual = *offset;
                    assert_that!("node data offset", actual == index, *offset)?;
                }
            }
            match read_node_data(read, offset, variant)? {
                WrappedNode::Camera(camera) => Ok(Node::Camera(camera)),
                WrappedNode::Display(display) => Ok(Node::Display(display)),
                WrappedNode::Empty(empty) => Ok(Node::Empty(empty)),
                WrappedNode::Light(light) => Ok(Node::Light(light)),
                WrappedNode::Window(window) => Ok(Node::Window(window)),
                WrappedNode::Lod(wrapped_lod) => {
                    let mut lod = wrapped_lod.wrapped;
                    lod.parent = read.read_u32()?;
                    *offset += 4;
                    lod.children = (0..wrapped_lod.children_count)
                        .map(|_| {
                            let child = read.read_u32()?;
                            *offset += 4;
                            Ok(child)
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Node::Lod(lod))
                }
                WrappedNode::Object3d(wrapped_obj) => {
                    let mut object3d = wrapped_obj.wrapped;

                    object3d.parent = if wrapped_obj.has_parent {
                        let parent = read.read_u32()?;
                        *offset += 4;
                        Some(parent)
                    } else {
                        None
                    };
                    /*object3d.children = */
                    (0..wrapped_obj.children_count)
                        .map(|_| {
                            let child = read.read_u32()?;
                            *offset += 4;
                            Ok(child)
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Node::Object3d(object3d))
                }
                WrappedNode::World(wrapped_world) => {
                    let mut world = wrapped_world.wrapped;
                    world.children = (0..wrapped_world.children_count)
                        .map(|_| {
                            let child = read.read_u32()?;
                            *offset += 4;
                            Ok(child)
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Node::World(world))
                }
            }
        })
        .collect::<Result<Vec<_>>>()?;

    read.assert_end()?;
    assert_area_partitions(&nodes, *offset)?;
    Ok(nodes)
}

fn assert_area_partitions(nodes: &[Node], offset: u32) -> Result<()> {
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

pub fn write_nodes<W>(write: &mut W, nodes: &[Node], array_size: u32, offset: u32) -> Result<()>
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
                for _ in &object3d.children {
                    // TODO: fixme
                    write.write_u32(0xEFBEADDE)?;
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
