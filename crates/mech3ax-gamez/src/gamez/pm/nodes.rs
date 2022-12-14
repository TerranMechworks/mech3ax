use log::debug;
use mech3ax_api_types::nodes::pm::*;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, assert_with_msg, Result};
use mech3ax_nodes::pm::{
    read_node_data, read_node_info_gamez, write_node_data, write_node_info, NodeVariantPm,
    WrappedNodePm, NODE_PM_C_SIZE,
};
use std::io::{Read, Write};

pub fn read_nodes(read: &mut CountingReader<impl Read>, array_size: u32) -> Result<Vec<NodePm>> {
    let end_offset = read.offset + NODE_PM_C_SIZE * array_size + 4 * array_size;

    let mut variants = Vec::new();
    let mut light_node = false;
    for index in 0..array_size {
        let node_info_pos = read.offset;
        let variant = read_node_info_gamez(read, index)?;

        let node_index = read.read_u32()?;
        let top = node_index & 0xFF000000;
        assert_that!("node index top", top == 0x02000000, read.prev)?;
        let node_index = node_index & 0x00FFFFFF;

        // debug!("Node {} data offset: {}", index, node_data_offset);
        match &variant {
            NodeVariantPm::World {
                data_ptr: _,
                children_count: _,
                children_array_ptr: _,
            } => {
                assert_that!("node data position", index == 0, node_info_pos)?;
                assert_that!("node index", node_index == 1, read.prev)?;
            }
            NodeVariantPm::Window { data_ptr: _ } => {
                assert_that!("node data position", index == 1, node_info_pos)?;
                assert_that!("node index", node_index == 2, read.prev)?;
            }
            NodeVariantPm::Camera { data_ptr: _ } => {
                assert_that!("node data position", index == 2, node_info_pos)?;
                assert_that!("node index", node_index == 3, read.prev)?;
            }
            NodeVariantPm::Display { data_ptr: _ } => {
                assert_that!("node data position", index == 3, node_info_pos)?;
                assert_that!("node index", node_index == 4, read.prev)?;
            }
            NodeVariantPm::Light { data_ptr: _ } => {
                assert_that!("node data position", index > 3, node_info_pos)?;
                if light_node {
                    return Err(assert_with_msg!(
                        "Unexpected light node in position {} (at {})",
                        index,
                        node_info_pos
                    ));
                }
                light_node = true;
                debug!("LIGHT NODE {} index {}", index, node_index);
            }
            NodeVariantPm::Lod(_) => {
                assert_that!("node data position", index > 3, node_info_pos)?;
                debug!("LOD NODE {} index {}", index, node_index);
            }
            NodeVariantPm::Object3d(_) => {
                assert_that!("node data position", index > 3, node_info_pos)?;
                debug!("OBJECT3D NODE {} index {}", index, node_index);
            }
        }
        variants.push((variant, node_index));
    }

    assert_that!("node info end", end_offset == read.offset, read.offset)?;

    let nodes = variants
        .into_iter()
        .enumerate()
        .map(
            |(index, (variant, node_data_offset))| match read_node_data(read, variant, index)? {
                WrappedNodePm::World(wrapped_world) => {
                    let mut world = wrapped_world.wrapped;
                    debug!(
                        "Reading node {} children x{} (pm) at {}",
                        index, wrapped_world.children_count, read.offset
                    );
                    world.children = (0..wrapped_world.children_count)
                        .map(|_| read.read_u32())
                        .collect::<std::io::Result<Vec<_>>>()?;
                    Ok(NodePm::World(world))
                }
                WrappedNodePm::Window(window) => Ok(NodePm::Window(window)),
                WrappedNodePm::Camera(camera) => Ok(NodePm::Camera(camera)),
                WrappedNodePm::Display(display) => Ok(NodePm::Display(display)),
                WrappedNodePm::Light(light) => Ok(NodePm::Light(light)),
                WrappedNodePm::Lod(wrapped_lod) => {
                    let mut lod = wrapped_lod.wrapped;

                    lod.parent = read.read_u32()?;
                    debug!(
                        "Reading node {} children x{} (pm) at {}",
                        index, wrapped_lod.children_count, read.offset
                    );
                    lod.children = (0..wrapped_lod.children_count)
                        .map(|_| read.read_u32())
                        .collect::<std::io::Result<Vec<_>>>()?;
                    Ok(NodePm::Lod(lod))
                }
                WrappedNodePm::Object3d(wrapped_obj) => {
                    let mut object3d = wrapped_obj.wrapped;

                    object3d.parent = if wrapped_obj.has_parent {
                        Some(read.read_u32()?)
                    } else {
                        None
                    };
                    debug!(
                        "Reading node {} children x{} (pm) at {}",
                        index, wrapped_obj.children_count, read.offset
                    );
                    object3d.children = (0..wrapped_obj.children_count)
                        .map(|_| read.read_u32())
                        .collect::<std::io::Result<Vec<_>>>()?;
                    Ok(NodePm::Object3d(object3d))
                }
            },
        )
        .collect::<Result<Vec<_>>>()?;

    read.assert_end()?;
    // assert_area_partitions(&nodes, read.offset)?;

    Ok(nodes)
}

pub fn write_nodes(write: &mut CountingWriter<impl Write>, nodes: &[NodePm]) -> Result<()> {
    for (index, node) in nodes.iter().enumerate() {
        write_node_info(write, node, false, index)?;
        let node_data_offset = match node {
            NodePm::World(_) => 1,
            NodePm::Window(_) => 2,
            NodePm::Camera(_) => 3,
            NodePm::Display(_) => 4,
            NodePm::Light(o) => 0,
            NodePm::Lod(o) => 0,
            NodePm::Object3d(o) => 0,
        };
        debug!("Node {} data offset: {}", index, node_data_offset);
        let node_data_offset = node_data_offset | 0x02000000;
        write.write_u32(node_data_offset)?;
    }

    for (index, node) in nodes.iter().enumerate() {
        write_node_data(write, node, index)?;
        match node {
            NodePm::World(world) => {
                debug!(
                    "Writing node {} children x{} (pm) at {}",
                    index,
                    world.children.len(),
                    write.offset
                );
                for child in &world.children {
                    write.write_u32(*child)?;
                }
            }
            NodePm::Lod(lod) => {
                write.write_u32(lod.parent)?;
                debug!(
                    "Writing node {} children x{} (pm) at {}",
                    index,
                    lod.children.len(),
                    write.offset
                );
                for child in &lod.children {
                    write.write_u32(*child)?;
                }
            }
            NodePm::Object3d(object3d) => {
                if let Some(parent) = object3d.parent {
                    write.write_u32(parent)?;
                }
                debug!(
                    "Writing node {} children x{} (pm) at {}",
                    index,
                    object3d.children.len(),
                    write.offset
                );
                for child in &object3d.children {
                    write.write_u32(*child)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
