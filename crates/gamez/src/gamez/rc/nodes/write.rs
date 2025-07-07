use super::NODE_INDEX_INVALID;
use crate::nodes::helpers::write_node_indices;
use crate::nodes::node::rc::{make_node, make_node_zero, NodeRcC};
use log::trace;
use mech3ax_api_types::gamez::nodes::{Node, NodeData};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, err, Result};
use mech3ax_types::AsBytes as _;
use std::io::Write;

pub(crate) fn write_nodes(
    write: &mut CountingWriter<impl Write>,
    nodes: &[Node],
    array_size: i32,
    offset: u32,
) -> Result<()> {
    let mut offset = offset + (NodeRcC::SIZE + 4) * (array_size as u32);
    let node_count = assert_len!(i32, nodes.len(), "GameZ nodes")?;

    for (index, node) in nodes.iter().enumerate() {
        trace!("Processing node info {}/{}", index, node_count);
        let node_info = make_node(node)?;
        write.write_struct(&node_info)?;

        let node_data_offset = match node.data {
            NodeData::Empty => match &node.parent_indices[..] {
                &[parent] => parent.to_i32() as u32,
                _ => return Err(err!("empty {}/{} has multiple parents", index, node_count)),
            },
            _ => offset,
        };

        trace!("Node data offset: {}", node_data_offset);
        write.write_u32(node_data_offset)?;
        offset += size_node(node);
    }

    trace!(
        "Processing {}..{} node info zeros at {}",
        node_count,
        array_size,
        write.offset
    );
    let node_zero = make_node_zero();
    for index in node_count..array_size {
        write.write_struct_no_log(&node_zero)?;
        let mut index = index + 1;
        if index == array_size {
            index = NODE_INDEX_INVALID;
        }
        write.write_i32(index)?;
    }

    trace!("Processed note info zeros at {}", write.offset);

    for (index, node) in nodes.iter().enumerate() {
        trace!("Processing node data {}/{}", index, node_count);
        match &node.data {
            NodeData::Camera(camera) => crate::nodes::camera::rc::write(write, camera)?,
            NodeData::Display(display) => crate::nodes::display::rc::write(write, display)?,
            NodeData::Empty => {}
            NodeData::Light(light) => crate::nodes::light::rc::write(write, light)?,
            NodeData::Lod(lod) => crate::nodes::lod::rc::write(write, lod)?,
            NodeData::Object3d(object3d) => crate::nodes::object3d::rc::write(write, object3d)?,
            NodeData::Window(window) => crate::nodes::window::rc::write(write, window)?,
            NodeData::World(world) => crate::nodes::world::rc::write(write, world)?,
        }

        if !matches!(node.data, NodeData::Empty) {
            write_node_indices(write, &node.parent_indices)?;
        }
        write_node_indices(write, &node.child_indices)?;
    }

    trace!("Processed node data at {}", write.offset);
    Ok(())
}

fn size_node(node: &Node) -> u32 {
    let node_size = match &node.data {
        NodeData::Camera(_camera) => crate::nodes::camera::rc::size(),
        NodeData::Display(_display) => crate::nodes::display::rc::size(),
        NodeData::Empty => return 0,
        NodeData::Light(light) => crate::nodes::light::rc::size(light),
        NodeData::Lod(_lod) => crate::nodes::lod::rc::size(),
        NodeData::Object3d(_object3d) => crate::nodes::object3d::rc::size(),
        NodeData::Window(_window) => crate::nodes::window::rc::size(),
        NodeData::World(world) => crate::nodes::world::rc::size(world),
    };

    let parent_size = (node.parent_indices.len() as u32).wrapping_mul(4);
    let child_size = (node.child_indices.len() as u32).wrapping_mul(4);

    node_size.wrapping_add(parent_size).wrapping_add(child_size)
}
