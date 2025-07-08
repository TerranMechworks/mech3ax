use super::NODE_INDEX_TOP;
use crate::nodes::helpers::write_node_indices;
use crate::nodes::node::pm::make_node;
use log::trace;
use mech3ax_api_types::gamez::nodes::{Node, NodeData};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::Result;
use std::io::Write;

pub(crate) fn write_nodes(write: &mut CountingWriter<impl Write>, nodes: &[Node]) -> Result<()> {
    let node_count = nodes.len();

    for (index, node) in nodes.iter().enumerate() {
        trace!("Processing node info {}/{}", index, node_count);
        let node_info = make_node(node)?;
        write.write_struct(&node_info)?;

        // TODO
        // let node_index = node.index | NODE_INDEX_TOP;
        let node_index = node.index;
        trace!("Node index: 0x{node_index:08X}");
        write.write_u32(node_index)?;
    }

    for (index, node) in nodes.iter().enumerate() {
        trace!("Processing node data {}/{}", index, node_count);
        match &node.data {
            NodeData::Camera(camera) => crate::nodes::camera::write(write, camera)?,
            NodeData::Display(display) => crate::nodes::display::write(write, display)?,
            NodeData::Empty => {}
            NodeData::Light(light) => crate::nodes::light::pm::write(write, light)?,
            NodeData::Lod(lod) => crate::nodes::lod::pm::write(write, lod)?,
            NodeData::Object3d(object3d) => crate::nodes::object3d::write(write, object3d)?,
            NodeData::Window(window) => crate::nodes::window::write(write, window)?,
            NodeData::World(world) => crate::nodes::world::pm::write(write, world)?,
        }

        write_node_indices(write, &node.parent_indices)?;
        write_node_indices(write, &node.child_indices)?;
    }

    trace!("Processed node data at {}", write.offset);
    Ok(())
}
