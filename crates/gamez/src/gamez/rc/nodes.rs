use super::NODE_INDEX_INVALID;
use crate::nodes::helpers::read_node_indices;
use crate::nodes::node::rc::{assert_node, assert_node_zero, NodeRcC};
use crate::nodes::NodeClass;
use log::trace;
use mech3ax_api_types::gamez::nodes::{Node, NodeData};
use mech3ax_common::check::amend_err;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, err, Result};
use mech3ax_types::{i32_to_usize, u32_to_usize, AsBytes as _};
use std::io::{Read, Write};

pub(crate) fn read_nodes(
    read: &mut CountingReader<impl Read>,
    array_size: i32,
    count: i32,
    model_count: i32,
) -> Result<Vec<Node>> {
    let node_write_size = u32_to_usize(NodeRcC::SIZE) + 4;
    let valid_offset = read.offset + node_write_size * i32_to_usize(count);
    let end_offset = read.offset + node_write_size * i32_to_usize(array_size);

    let nodes = (0..count)
        .map(|index| {
            trace!("Processing node info {}/{}", index, count);
            let node: NodeRcC = read.read_struct()?;
            let mut node_info = assert_node(&node, read.prev, model_count)?;
            // note: the offset is read separately because it's a pain for zero
            // nodes, empty nodes, and for the mechlib in MW/PM.
            //
            // usually, this is the offset of the node data, or for empty nodes
            // it's the parent index since they do not have data.
            node_info.offset = read.read_u32()?;
            Ok(node_info)
        })
        .collect::<Result<Vec<_>>>()?;

    // TODO: kinda dumb?
    if read.offset != valid_offset {
        return Err(err!(
            "read offset {} != {} (node info)",
            read.offset,
            valid_offset
        ));
    }

    trace!(
        "Processing {}..{} node info zeros at {}",
        count,
        array_size,
        read.offset
    );
    for index in count..array_size {
        let node: NodeRcC = read.read_struct_no_log()?;
        assert_node_zero(&node, read.prev)
            .inspect_err(|_| trace!("{:#?} (index: {}, at {})", node, index, read.prev))?;

        let actual_index = read.read_i32()?;

        let mut expected_index = index + 1;
        if expected_index == array_size {
            expected_index = NODE_INDEX_INVALID;
        }

        if actual_index != expected_index {
            return Err(err!(
                "node off {} != {} (index: {}, at {})",
                actual_index,
                expected_index,
                index,
                read.prev,
            ));
        }
    }
    trace!("Processed node info zeros at {}", read.offset);

    // TODO: kinda dumb?
    if read.offset != end_offset {
        return Err(err!(
            "read offset {} != {} (node zero)",
            read.offset,
            end_offset
        ));
    }

    let nodes = nodes
        .into_iter()
        .enumerate()
        .map(|(index, node_info)| {
            trace!("Processing node data {}/{}", index, count);

            if node_info.node_class != NodeClass::Empty {
                let node_offset = u32_to_usize(node_info.offset);
                if read.offset != node_offset {
                    return Err(err!(
                        "read offset {} != {} (node {})",
                        read.offset,
                        end_offset,
                        index,
                    ));
                }
            }

            let mut node = Node {
                name: node_info.name,
                flags: node_info.flags,
                update_flags: node_info.update_flags,
                zone_id: node_info.zone_id,
                model_index: node_info.model_index,
                area_partition: node_info.area_partition,
                parent_indices: Vec::new(),
                child_indices: Vec::new(),
                active_bbox: node_info.active_bbox,
                node_bbox: node_info.node_bbox,
                model_bbox: node_info.model_bbox,
                child_bbox: node_info.child_bbox,
                field192: node_info.field192,
                field196: node_info.field196,
                field200: node_info.field200,
                field204: node_info.field204,
                data: NodeData::Empty,
                data_ptr: node_info.data_ptr.0,
                parent_array_ptr: node_info.parent_array_ptr.0,
                child_array_ptr: node_info.child_array_ptr.0,
            };

            match node_info.node_class {
                NodeClass::Camera => {
                    let camera = crate::nodes::camera::rc::read(read)?;
                    node.data = NodeData::Camera(camera);
                }
                NodeClass::Display => {
                    let display = crate::nodes::display::rc::read(read)?;
                    node.data = NodeData::Display(display);
                }
                NodeClass::Empty => {
                    let parent_index = node_info.offset as i32;
                    let parent_index =
                        crate::nodes::check::node_index2(parent_index).map_err(|msg| {
                            let name = format!("empty {}/{} parent index", index, count);
                            amend_err(msg, &name, read.offset, file!(), line!())
                        })?;
                    node.parent_indices = vec![parent_index];
                }
                NodeClass::Light => {
                    let light = crate::nodes::light::rc::read(read)?;
                    node.data = NodeData::Light(light);
                }
                NodeClass::Lod => {
                    let lod = crate::nodes::lod::rc::read(read)?;
                    node.data = NodeData::Lod(lod);
                }
                NodeClass::Object3d => {
                    let object3d = crate::nodes::object3d::rc::read(read)?;
                    node.data = NodeData::Object3d(object3d);
                }
                NodeClass::Window => {
                    let window = crate::nodes::window::rc::read(read)?;
                    node.data = NodeData::Window(window);
                }
                NodeClass::World => {
                    let world = crate::nodes::world::rc::read(read)?;
                    node.data = NodeData::World(world);
                }
            };

            if node_info.parent_count > 0 {
                node.parent_indices =
                    read_node_indices!(read, node_info.parent_count, |idx, cnt| {
                        format!("node {}/{} parent index {}/{}", index, count, idx, cnt)
                    })?;
            }

            if node_info.child_count > 0 {
                node.child_indices =
                    read_node_indices!(read, node_info.child_count, |idx, cnt| {
                        format!("node {}/{} child index {}/{}", index, count, idx, cnt)
                    })?;
            }

            Ok(node)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(nodes)
}

pub(crate) fn write_nodes(
    write: &mut CountingWriter<impl Write>,
    nodes: &[Node],
    array_size: i32,
    offset: u32,
) -> Result<()> {
    // let mut offset = offset + (NodeRcC::SIZE + 4) * (array_size as u32);
    // let node_count = assert_len!(i32, nodes.len(), "GameZ nodes")?;

    // for (index, node) in nodes.iter().enumerate() {
    //     trace!("Processing node info {}/{}", index, node_count);
    //     write_node_info(write, node)?;

    //     let node_data_offset = match node {
    //         NodeRc::Empty(empty) => empty.parent,
    //         _ => offset,
    //     };

    //     trace!("Node {} data offset: {}", index, node_data_offset);
    //     write.write_u32(node_data_offset)?;
    //     offset += size_node(node);
    // }

    // trace!(
    //     "Processing {}..{} node info zeros at {}",
    //     node_count,
    //     array_size,
    //     write.offset
    // );
    // let node_zero = NodeRcC::zero();
    // for index in node_count..array_size {
    //     write.write_struct_no_log(&node_zero)?;
    //     let mut index = index + 1;
    //     if index == array_size {
    //         index = NODE_INDEX_INVALID;
    //     }
    //     write.write_i32(index)?;
    // }
    // trace!("Processed note info zeros at {}", write.offset);

    // for (index, node) in nodes.iter().enumerate() {
    //     if !matches!(node, NodeRc::Empty(_)) {
    //         trace!("Processing node data {}/{}", index, node_count);
    //     }
    //     write_node_data(write, node)?;
    // }

    Ok(())
}
