use super::{NODE_INDEX_BOT_MASK, NODE_INDEX_TOP, NODE_INDEX_TOP_MASK};
use log::trace;
use mech3ax_api_types::gamez::mesh::MeshNg;
use mech3ax_api_types::nodes::cs::NodeCs;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_nodes::cs::{
    read_node_data, read_node_info, write_node_data, write_node_info, NodeVariantCs, NODE_CS_C_SIZE,
};
use mech3ax_types::u32_to_usize;
use std::io::{Read, Write};

pub(crate) fn read_nodes(
    read: &mut CountingReader<impl Read>,
    array_size: u32,
    light_index: u32,
    meshes: &[Option<MeshNg>],
    is_gamez: bool,
) -> Result<Vec<NodeCs>> {
    let end_offset = read.offset + (u32_to_usize(NODE_CS_C_SIZE) + 4) * u32_to_usize(array_size);

    let mut light_node = None;
    let variants = (0..array_size)
        .map(|index| {
            trace!("Reading node info {}/{}", index, array_size);

            let node_info_pos = read.offset;
            let variant = read_node_info(read)?;

            let node_index = read.read_u32()?;
            let top = node_index & NODE_INDEX_TOP_MASK;
            assert_that!("node index top", top == NODE_INDEX_TOP, read.prev)?;
            let node_index = node_index & NODE_INDEX_BOT_MASK;
            trace!("Node {} index: {}", index, node_index);

            match &variant {
                NodeVariantCs::World { .. } => {
                    assert_that!("node position (world)", index == 0, node_info_pos)?;
                    assert_that!("node index (world)", node_index == 1, read.prev)?;
                }
                NodeVariantCs::Display { .. } => {
                    assert_that!("node position (display)", index == 1, node_info_pos)?;
                    assert_that!("node index (display)", node_index == 2, read.prev)?;
                }
                NodeVariantCs::Window {
                    spyglass: false, ..
                } => {
                    assert_that!("node position (window normal)", index == 2, node_info_pos)?;
                    assert_that!("node index (window normal)", node_index == 3, read.prev)?;
                }
                NodeVariantCs::Camera {
                    spyglass: false, ..
                } => {
                    assert_that!("node position (camera normal)", index == 3, node_info_pos)?;
                    assert_that!("node index (camera normal)", node_index == 4, read.prev)?;
                }
                NodeVariantCs::Window { spyglass: true, .. } => {
                    assert_that!("node position (window spyglass)", index == 4, node_info_pos)?;
                    assert_that!("node index (window spyglass)", node_index == 5, read.prev)?;
                }
                NodeVariantCs::Camera { spyglass: true, .. } => {
                    assert_that!("node position (camera spyglass)", index == 5, node_info_pos)?;
                    assert_that!("node index (camera spyglass)", node_index == 6, read.prev)?;
                }
                NodeVariantCs::Light { .. } => {
                    // exclude world, window, camera, or display indices
                    assert_that!("node position (light)", index > 5, node_info_pos)?;
                    // we still have to assert that there was a light node, since
                    // the node index might not be unique or even present.
                    assert_that!("node index (light)", node_index == light_index, read.prev)?;
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
                NodeVariantCs::Lod(_) => {
                    // can't do this for planes.zbd
                    if is_gamez {
                        // exclude world, window, camera, or display indices
                        assert_that!("node position (lod)", index > 5, node_info_pos)?;
                    }
                }
                NodeVariantCs::Object3d(object3d) => {
                    // can't do this for planes.zbd
                    if is_gamez {
                        // exclude world, window, camera, or display indices
                        assert_that!("node position (object3d)", index > 5, node_info_pos)?;
                    }
                    if object3d.mesh_index >= 0 {
                        // Cast safety: >=0, usize::MAX > i32::MAX
                        let mesh_index = object3d.mesh_index as usize;
                        assert_that!(
                            "object3d mesh index",
                            mesh_index < meshes.len(),
                            node_info_pos
                        )?;
                        let mesh_exists = meshes[mesh_index].is_some();
                        assert_that!("object3d mesh index", mesh_exists == true, node_info_pos)?;
                    }
                }
            }
            Ok((variant, node_index))
        })
        .collect::<Result<Vec<_>>>()?;

    assert_that!("node info end", end_offset == read.offset, read.offset)?;

    // can't do this for planes.zbd
    if is_gamez {
        assert_that!("has light node", light_node != None, read.offset)?;
    }

    let nodes = variants
        .into_iter()
        .enumerate()
        .map(|(index, (variant, node_index))| {
            trace!("Reading node data {}/{}", index, array_size);
            read_node_data(read, variant, node_index)
        })
        .collect::<Result<Vec<_>>>()?;

    read.assert_end()?;
    if is_gamez {
        // can't do this for planes.zbd
        assert_gamez_area_partitions(&nodes, read.offset)?;
    } else {
        assert_planes_area_partitions(&nodes, read.offset)?;
    }

    Ok(nodes)
}

fn assert_planes_area_partitions(nodes: &[NodeCs], offset: usize) -> Result<()> {
    for node in nodes {
        if let NodeCs::Object3d(object3d) = node {
            assert_that!(
                "object3d area partition",
                object3d.area_partition == None,
                offset
            )?;
        }
    }
    Ok(())
}

fn assert_gamez_area_partitions(nodes: &[NodeCs], offset: usize) -> Result<()> {
    let (x_count, y_count) = match nodes.first() {
        Some(NodeCs::World(world)) => Ok((
            world.area.x_count(1024) as i16,
            world.area.y_count(1024) as i16,
        )),
        Some(_) => Err(assert_with_msg!("Expected the world node to be first")),
        None => Err(assert_with_msg!("Expected to have read some nodes")),
    }?;

    for node in nodes {
        let area_partition = match node {
            NodeCs::Object3d(object3d) => &object3d.area_partition,
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

pub(crate) fn write_nodes(write: &mut CountingWriter<impl Write>, nodes: &[NodeCs]) -> Result<()> {
    let node_count = nodes.len();

    for (index, node) in nodes.iter().enumerate() {
        trace!("Writing node info {}/{}", index, node_count);
        write_node_info(write, node)?;
        let node_index = match node {
            NodeCs::World(_) => 1,
            NodeCs::Display(_) => 2,
            NodeCs::Window(window) if window.name == "sgwin" => 5,
            NodeCs::Camera(camera) if camera.name == "spyglass" => 6,
            NodeCs::Camera(_) => 4,
            NodeCs::Window(_) => 3,
            NodeCs::Light(light) => light.node_index,
            NodeCs::Lod(lod) => lod.node_index,
            NodeCs::Object3d(object3d) => object3d.node_index,
        };
        trace!("Node {} index: {}", index, node_index);
        let node_index = node_index | NODE_INDEX_TOP;
        write.write_u32(node_index)?;
    }

    for (index, node) in nodes.iter().enumerate() {
        trace!("Writing node data {}/{}", index, node_count);
        write_node_data(write, node)?;
    }

    Ok(())
}
