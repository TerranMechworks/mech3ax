use log::{debug, trace};
use mech3ax_api_types::nodes::cs::*;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_nodes::cs::{
    read_node_data, read_node_info, write_node_data, write_node_info, NodeVariantCs, NODE_CS_C_SIZE,
};
use std::io::{Read, Write};

pub fn read_nodes(
    read: &mut CountingReader<impl Read>,
    array_size: u32,
    light_index: u32,
) -> Result<Vec<NodeCs>> {
    let end_offset = read.offset + NODE_CS_C_SIZE * array_size + 4 * array_size;

    let mut variants = Vec::new();
    let mut light_node = false;
    for index in 0..array_size {
        let node_info_pos = read.offset;
        let variant = read_node_info(read, index)?;

        let node_index = read.read_u32()?;
        let top = node_index & 0xFF000000;
        assert_that!("node index top", top == 0x02000000, read.prev)?;
        let node_index = node_index & 0x00FFFFFF;
        trace!("Node {} index: {}", index, node_index);

        match &variant {
            NodeVariantCs::World {
                data_ptr: _,
                children_count: _,
                children_array_ptr: _,
            } => {
                assert_that!("node data position", index == 0, node_info_pos)?;
                assert_that!("node index", node_index == 1, read.prev)?;
            }
            NodeVariantCs::Display { data_ptr: _ } => {
                assert_that!("node data position", index == 1, node_info_pos)?;
                assert_that!("node index", node_index == 2, read.prev)?;
            }
            NodeVariantCs::Window {
                data_ptr: _,
                spyglass: false,
            } => {
                assert_that!("node data position", index == 2, node_info_pos)?;
                assert_that!("node index", node_index == 3, read.prev)?;
            }
            NodeVariantCs::Camera {
                data_ptr: _,
                spyglass: false,
            } => {
                assert_that!("node data position", index == 3, node_info_pos)?;
                assert_that!("node index", node_index == 4, read.prev)?;
            }
            NodeVariantCs::Window {
                data_ptr: _,
                spyglass: true,
            } => {
                assert_that!("node data position", index == 4, node_info_pos)?;
                assert_that!("node index", node_index == 5, read.prev)?;
            }
            NodeVariantCs::Camera {
                data_ptr: _,
                spyglass: true,
            } => {
                assert_that!("node data position", index == 5, node_info_pos)?;
                assert_that!("node index", node_index == 6, read.prev)?;
            }
            NodeVariantCs::Light { data_ptr: _ } => {
                assert_that!("node index", node_index == light_index, read.prev)?;
                assert_that!("node data position", index > 5, node_info_pos)?;
                if light_node {
                    return Err(assert_with_msg!(
                        "Unexpected light node in position {} (at {})",
                        index,
                        node_info_pos
                    ));
                }
                light_node = true;
            }
            NodeVariantCs::Lod(_) => {
                // can't do this for planes.zbd
                // assert_that!("node data position", index > 5, node_info_pos)?;
            }
            NodeVariantCs::Object3d(_) => {
                // can't do this for planes.zbd
                // assert_that!("node data position", index > 5, node_info_pos)?;
            }
        }
        variants.push((variant, node_index));
    }

    assert_that!("node info end", end_offset == read.offset, read.offset)?;
    trace!("Node data pos {}", read.offset);

    let nodes = variants
        .into_iter()
        .enumerate()
        .map(|(index, (variant, node_index))| read_node_data(read, variant, node_index, index))
        .collect::<Result<Vec<_>>>()?;

    read.assert_end()?;
    // assert_area_partitions(&nodes, read.offset)?;

    Ok(nodes)
}

// fn assert_area_partitions(nodes: &[NodePm], offset: u32) -> Result<()> {
//     let (x_count, y_count) = match nodes.first() {
//         Some(NodePm::World(world)) => {
//             Ok((world.area.x_count() as i16, world.area.y_count() as i16))
//         }
//         Some(_) => Err(assert_with_msg!("Expected the world node to be first")),
//         None => Err(assert_with_msg!("Expected to have read some nodes")),
//     }?;

//     for node in nodes {
//         let area_partition = match node {
//             NodePm::Object3d(object3d) => &object3d.area_partition,
//             _ => &None,
//         };
//         if let Some(ap) = area_partition {
//             assert_that!("area partition x", ap.x < x_count, offset)?;
//             assert_that!("area partition y", ap.y < y_count, offset)?;
//             assert_that!("virt partition x", ap.virtual_x <= x_count, offset)?;
//             assert_that!("virt partition y", ap.virtual_y <= y_count, offset)?;
//         }
//     }

//     Ok(())
// }

pub fn write_nodes(write: &mut CountingWriter<impl Write>, nodes: &[NodeCs]) -> Result<()> {
    for (index, node) in nodes.iter().enumerate() {
        write_node_info(write, node, index)?;
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
        let node_index = node_index | 0x02000000;
        write.write_u32(node_index)?;
    }

    for (index, node) in nodes.iter().enumerate() {
        write_node_data(write, node, index)?;
    }

    Ok(())
}
