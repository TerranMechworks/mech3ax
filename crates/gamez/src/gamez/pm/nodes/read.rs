use super::{NODE_INDEX_BOT_MASK, NODE_INDEX_TOP, NODE_INDEX_TOP_MASK};
use log::trace;
use mech3ax_api_types::nodes::pm::NodePm;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_nodes::common::read_child_indices;
use mech3ax_nodes::mw::NodeMwC;
use mech3ax_nodes::pm::{read_node_data, read_node_info_gamez, NodeVariantPm, WrappedNodePm};
use mech3ax_types::{i32_to_usize, u32_to_usize, AsBytes as _};
use std::io::Read;

pub(crate) fn read_nodes(
    read: &mut CountingReader<impl Read>,
    array_size: i32,
    model_count: i32,
) -> Result<Vec<NodePm>> {
    let node_write_size = u32_to_usize(NodeMwC::SIZE) + 4;
    let end_offset = read.offset + node_write_size * i32_to_usize(array_size);

    let mut light_node: Option<i32> = None;
    let variants = (0..array_size)
        .map(|index| {
            trace!("Processing node info {}/{}", index, array_size);

            let node_info_pos = read.offset;
            let variant = read_node_info_gamez(read)?;

            let node_index = read.read_u32()?;
            let top = node_index & NODE_INDEX_TOP_MASK;
            assert_that!("node index top", top == NODE_INDEX_TOP, read.prev)?;
            let node_index = node_index & NODE_INDEX_BOT_MASK;
            trace!("Node {} index: {}", index, node_index);

            match &variant {
                NodeVariantPm::World { .. } => {
                    assert_that!("node position (world)", index == 0, node_info_pos)?;
                    assert_that!("node index (world)", node_index == 1, read.prev)?;
                }
                NodeVariantPm::Window { .. } => {
                    assert_that!("node position (window)", index == 1, node_info_pos)?;
                    assert_that!("node index (window)", node_index == 2, read.prev)?;
                }
                NodeVariantPm::Camera { .. } => {
                    assert_that!("node position (camera)", index == 2, node_info_pos)?;
                    assert_that!("node index (camera)", node_index == 3, read.prev)?;
                }
                NodeVariantPm::Display { .. } => {
                    assert_that!("node position (display)", index == 3, node_info_pos)?;
                    assert_that!("node index (display)", node_index == 4, read.prev)?;
                }
                NodeVariantPm::Light { .. } => {
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
                NodeVariantPm::Lod(_) => {
                    // exclude world, window, camera, or display indices
                    assert_that!("node position (lod)", index > 3, node_info_pos)?;
                }
                NodeVariantPm::Object3d(object3d) => {
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
            Ok((variant, node_index))
        })
        .collect::<Result<Vec<_>>>()?;

    assert_that!("node info end", end_offset == read.offset, read.offset)?;
    let _light_node_index = light_node
        .ok_or_else(|| assert_with_msg!("GameZ contains no light node (at {})", read.offset))?;

    let nodes = variants
        .into_iter()
        .enumerate()
        .map(|(index, (variant, node_index))| {
            trace!("Processing node data {}/{}", index, array_size);
            match read_node_data(read, variant)? {
                WrappedNodePm::Camera(camera) => Ok(NodePm::Camera(camera)),
                WrappedNodePm::Display(display) => Ok(NodePm::Display(display)),
                WrappedNodePm::Light(mut light) => {
                    light.node_index = node_index;
                    Ok(NodePm::Light(light))
                }
                WrappedNodePm::Window(window) => Ok(NodePm::Window(window)),
                WrappedNodePm::Lod(wrapped_lod) => {
                    let mut lod = wrapped_lod.wrapped;
                    lod.node_index = node_index;
                    lod.parent = read.read_u32()?;
                    lod.children = read_child_indices(read, u32::from(wrapped_lod.children_count))?;
                    Ok(NodePm::Lod(lod))
                }
                WrappedNodePm::Object3d(wrapped_obj) => {
                    let mut object3d = wrapped_obj.wrapped;
                    object3d.node_index = node_index;
                    object3d.parent = if wrapped_obj.has_parent {
                        Some(read.read_u32()?)
                    } else {
                        None
                    };
                    object3d.children =
                        read_child_indices(read, u32::from(wrapped_obj.children_count))?;
                    Ok(NodePm::Object3d(object3d))
                }
                WrappedNodePm::World(wrapped_world) => {
                    let mut world = wrapped_world.wrapped;
                    world.children =
                        read_child_indices(read, u32::from(wrapped_world.children_count))?;
                    Ok(NodePm::World(world))
                }
            }
        })
        .collect::<Result<Vec<_>>>()?;

    assert_area_partitions(&nodes, read.offset)?;

    Ok(nodes)
}

fn assert_area_partitions(nodes: &[NodePm], offset: usize) -> Result<()> {
    let (x_count, y_count) = match nodes.first() {
        Some(NodePm::World(world)) => Ok((
            world.area.x_count(256) as i16,
            world.area.y_count(256) as i16,
        )),
        Some(_) => Err(assert_with_msg!("Expected the world node to be first")),
        None => Err(assert_with_msg!("Expected to have read some nodes")),
    }?;

    for node in nodes {
        let area_partition = match node {
            NodePm::Object3d(object3d) => &object3d.area_partition,
            _ => &None,
        };
        if let Some(ap) = area_partition {
            // this isn't really a great validation; the values can still be
            // negative... this is because some AP values seem bogus, e.g.
            // when either x or y are -1, but the other component isn't.
            assert_that!("area partition x", ap.x < x_count, offset)?;
            assert_that!("area partition y", ap.y < y_count, offset)?;
            assert_that!("virt partition x", ap.virtual_x <= x_count, offset)?;
            assert_that!("virt partition y", ap.virtual_y <= y_count, offset)?;
        }
    }

    Ok(())
}
