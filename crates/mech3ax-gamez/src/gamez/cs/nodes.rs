use log::{debug, trace};
use mech3ax_api_types::nodes::cs::*;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_nodes::cs::{read_node_info, write_node_info, NodeVariantCs, NODE_CS_C_SIZE};
use std::io::{Read, Write};

pub fn read_nodes(
    read: &mut CountingReader<impl Read>,
    array_size: u32,
    light_index: u32,
) -> Result<(Vec<NodeCs>, Vec<u8>)> {
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
        .map(|(index, (variant, node_index))| match variant {
            NodeVariantCs::World {
                data_ptr,
                children_count,
                children_array_ptr,
            } => {
                let world = mech3ax_nodes::cs::world::read(
                    read,
                    data_ptr,
                    children_count,
                    children_array_ptr,
                    index,
                )?;
                Ok(NodeCs::World(world))
            }
            NodeVariantCs::Display { data_ptr } => {
                let display = mech3ax_nodes::cs::display::read(read, data_ptr, index)?;
                Ok(NodeCs::Display(display))
            }
            NodeVariantCs::Window { data_ptr, spyglass } => {
                let window = mech3ax_nodes::cs::window::read(read, data_ptr, spyglass, index)?;
                Ok(NodeCs::Window(window))
            }
            NodeVariantCs::Camera { data_ptr, spyglass } => {
                let camera = mech3ax_nodes::cs::camera::read(read, data_ptr, spyglass, index)?;
                Ok(NodeCs::Camera(camera))
            }
            NodeVariantCs::Light { data_ptr } => Ok(NodeCs::Light(Light {
                name: "sunlight".to_string(),
                data_ptr,
                node_index,
            })),
            NodeVariantCs::Lod(lod) => Ok(NodeCs::Lod(Lod {
                name: lod.name,
                flags_unk03: lod.flags_unk03,
                flags_unk04: lod.flags_unk04,
                flags_unk07: lod.flags_unk07,
                unk040: lod.unk040,
                zone_id: lod.zone_id,
                data_ptr: lod.data_ptr,
                parent_array_ptr: lod.parent_array_ptr,
                children_count: lod.children_count,
                children_array_ptr: lod.children_array_ptr,
                unk164: lod.unk164,
                node_index,
            })),
            NodeVariantCs::Object3d(node) => Ok(NodeCs::Object3d(Object3d {
                name: node.name,
                flags: node.flags.bits(),
                unk040: node.unk040,
                unk044: node.unk044,
                zone_id: node.zone_id,
                data_ptr: node.data_ptr,
                mesh_index: node.mesh_index,
                area_partition: node.area_partition,
                has_parent: node.has_parent,
                parent_array_ptr: node.parent_array_ptr,
                children_count: node.children_count,
                children_array_ptr: node.children_array_ptr,
                unk112: node.unk112,
                unk116: node.unk116,
                unk140: node.unk140,
                unk164: node.unk164,
                node_index,
            })),
        })
        .collect::<Result<Vec<_>>>()?;

    // let nodes = variants
    //     .into_iter()
    //     .enumerate()
    //     .map(
    //         |(index, (variant, node_index))| match read_node_data(read, variant, index)? {
    //             WrappedNodePm::World(wrapped_world) => {
    //                 let mut world = wrapped_world.wrapped;
    //                 debug!(
    //                     "Reading node {} children x{} (pm) at {}",
    //                     index, wrapped_world.children_count, read.offset
    //                 );
    //                 world.children = (0..wrapped_world.children_count)
    //                     .map(|_| read.read_u32())
    //                     .collect::<std::io::Result<Vec<_>>>()?;
    //                 Ok(NodePm::World(world))
    //             }
    //             WrappedNodePm::Window(window) => Ok(NodePm::Window(window)),
    //             WrappedNodePm::Camera(camera) => Ok(NodePm::Camera(camera)),
    //             WrappedNodePm::Display(display) => Ok(NodePm::Display(display)),
    //             WrappedNodePm::Light(mut light) => {
    //                 light.node_index = node_index;
    //                 Ok(NodePm::Light(light))
    //             }
    //             WrappedNodePm::Lod(wrapped_lod) => {
    //                 let mut lod = wrapped_lod.wrapped;

    //                 lod.node_index = node_index;
    //                 lod.parent = read.read_u32()?;
    //                 debug!(
    //                     "Reading node {} children x{} (pm) at {}",
    //                     index, wrapped_lod.children_count, read.offset
    //                 );
    //                 lod.children = (0..wrapped_lod.children_count)
    //                     .map(|_| read.read_u32())
    //                     .collect::<std::io::Result<Vec<_>>>()?;
    //                 Ok(NodePm::Lod(lod))
    //             }
    //             WrappedNodePm::Object3d(wrapped_obj) => {
    //                 let mut object3d = wrapped_obj.wrapped;

    //                 object3d.node_index = node_index;
    //                 object3d.parent = if wrapped_obj.has_parent {
    //                     Some(read.read_u32()?)
    //                 } else {
    //                     None
    //                 };
    //                 debug!(
    //                     "Reading node {} children x{} (pm) at {}",
    //                     index, wrapped_obj.children_count, read.offset
    //                 );
    //                 object3d.children = (0..wrapped_obj.children_count)
    //                     .map(|_| read.read_u32())
    //                     .collect::<std::io::Result<Vec<_>>>()?;
    //                 Ok(NodePm::Object3d(object3d))
    //             }
    //         },
    //     )
    //     .collect::<Result<Vec<_>>>()?;

    // read.assert_end()?;
    // assert_area_partitions(&nodes, read.offset)?;
    trace!("Unparsed data pos {}", read.offset);
    let node_data = read.read_to_end()?;

    Ok((nodes, node_data))
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
        match node {
            NodeCs::World(world) => mech3ax_nodes::cs::world::write(write, world, index)?,
            NodeCs::Display(display) => mech3ax_nodes::cs::display::write(write, display, index)?,
            NodeCs::Window(window) => mech3ax_nodes::cs::window::write(write, window, index)?,
            NodeCs::Camera(camera) => mech3ax_nodes::cs::camera::write(write, camera, index)?,
            _ => {}
        }
        //     write_node_data(write, node, index)?;
        //     match node {
        //         NodePm::World(world) => {
        //             debug!(
        //                 "Writing node {} children x{} (pm) at {}",
        //                 index,
        //                 world.children.len(),
        //                 write.offset
        //             );
        //             for child in &world.children {
        //                 write.write_u32(*child)?;
        //             }
        //         }
        //         NodePm::Lod(lod) => {
        //             write.write_u32(lod.parent)?;
        //             debug!(
        //                 "Writing node {} children x{} (pm) at {}",
        //                 index,
        //                 lod.children.len(),
        //                 write.offset
        //             );
        //             for child in &lod.children {
        //                 write.write_u32(*child)?;
        //             }
        //         }
        //         NodePm::Object3d(object3d) => {
        //             if let Some(parent) = object3d.parent {
        //                 write.write_u32(parent)?;
        //             }
        //             debug!(
        //                 "Writing node {} children x{} (pm) at {}",
        //                 index,
        //                 object3d.children.len(),
        //                 write.offset
        //             );
        //             for child in &object3d.children {
        //                 write.write_u32(*child)?;
        //             }
        //         }
        //         _ => {}
        //     }
    }

    Ok(())
}
