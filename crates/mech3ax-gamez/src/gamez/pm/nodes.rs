use log::debug;
use mech3ax_api_types::nodes::pm::*;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, assert_with_msg, Result};
use mech3ax_nodes::pm::{
    read_node_data, read_node_info_gamez, size_node, write_node_data, write_node_info,
    NodeVariantPm, NODE_PM_C_SIZE,
};
use std::io::{Read, Write};

pub fn read_nodes(
    read: &mut CountingReader<impl Read>,
    array_size: u32,
) -> Result<(Vec<NodePm>, Vec<u8>)> {
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
        .map(|(index, (mut variant, node_data_offset))| {
            //         match &mut variant {
            //             NodeVariantPm::Empty(empty) => {
            //                 // in the case of an empty node, the offset is used as the parent
            //                 // index, and not the offset (there is no node data)
            //                 empty.parent = node_data_offset;
            //             }
            //             _ => {
            //                 assert_that!(
            //                     "node data offset",
            //                     read.offset == node_data_offset,
            //                     read.offset
            //                 )?;
            //             }
            //         }
            //         read_node_data(read, variant, index)
            match variant {
                NodeVariantPm::World {
                    data_ptr,
                    children_count,
                    children_array_ptr,
                } => Ok(NodePm::World(mech3ax_nodes::pm::world::read(
                    read,
                    data_ptr,
                    children_count,
                    children_array_ptr,
                    index,
                )?)),
                NodeVariantPm::Window { data_ptr } => Ok(NodePm::Window(Window { data_ptr })),
                NodeVariantPm::Camera { data_ptr } => Ok(NodePm::Camera(Camera { data_ptr })),
                NodeVariantPm::Display { data_ptr } => Ok(NodePm::Display(Display { data_ptr })),
                NodeVariantPm::Light { data_ptr } => Ok(NodePm::Light(Light {
                    data_ptr,
                    parent_ptr: 0,
                    node_data_offset,
                })),
                NodeVariantPm::Lod(lod) => Ok(NodePm::Lod(Lod {
                    name: lod.name,
                    flags: lod.flags.into(),
                    zone_id: lod.zone_id,
                    data_ptr: lod.data_ptr,
                    parent_array_ptr: lod.parent_array_ptr,
                    children: (0..(lod.children_count as u32)).collect(),
                    children_array_ptr: lod.children_array_ptr,
                    unk164: lod.unk164,
                    node_data_offset,
                })),
                NodeVariantPm::Object3d(object3d) => {
                    Ok(NodePm::Object3d(Object3d {
                        name: object3d.name,
                        flags: object3d.flags.into(),
                        zone_id: object3d.zone_id,
                        area_partition: object3d.area_partition,
                        mesh_index: object3d.mesh_index,
                        // parent: object3d.parent,
                        parent: if object3d.parent_array_ptr == 0 {
                            None
                        } else {
                            Some(0)
                        },
                        children: (0..(object3d.children_count as u32)).collect(),
                        data_ptr: object3d.data_ptr,
                        parent_array_ptr: object3d.parent_array_ptr,
                        children_array_ptr: object3d.children_array_ptr,
                        unk112: object3d.unk112,
                        unk116: object3d.unk116,
                        unk140: object3d.unk140,
                        unk164: object3d.unk164,
                        node_data_offset,
                    }))
                }
            }
        })
        .collect::<Result<Vec<_>>>()?;

    let node_data = read.read_to_end()?;

    // read.assert_end()?;
    // assert_area_partitions(&nodes, read.offset)?;

    Ok((nodes, node_data))
}

pub fn write_nodes(write: &mut CountingWriter<impl Write>, nodes: &[NodePm]) -> Result<()> {
    for (index, node) in nodes.iter().enumerate() {
        write_node_info(write, node, false, index)?;
        let node_data_offset = match node {
            NodePm::World(_) => 1,
            NodePm::Window(_) => 2,
            NodePm::Camera(_) => 3,
            NodePm::Display(_) => 4,
            NodePm::Light(o) => o.node_data_offset,
            NodePm::Lod(o) => o.node_data_offset,
            NodePm::Object3d(o) => o.node_data_offset,
        };
        debug!("Node {} data offset: {}", index, node_data_offset);
        let node_data_offset = node_data_offset | 0x02000000;
        write.write_u32(node_data_offset)?;
    }

    for (index, node) in nodes.iter().enumerate() {
        match node {
            NodePm::World(world) => mech3ax_nodes::pm::world::write(write, world, index)?,
            _ => {}
        }
        //     write_node_data(write, node, index)?;
    }

    Ok(())
}
