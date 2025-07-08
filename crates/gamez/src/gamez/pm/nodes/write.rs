use super::NODE_INDEX_TOP;
use log::trace;
use mech3ax_api_types::nodes::pm::NodePm;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::Result;
use mech3ax_nodes::common::write_child_indices;
use mech3ax_nodes::pm::{write_node_data, write_node_info};
use std::io::Write;

pub(crate) fn write_nodes(write: &mut CountingWriter<impl Write>, nodes: &[NodePm]) -> Result<()> {
    let node_count = nodes.len();

    for (index, node) in nodes.iter().enumerate() {
        trace!("Processing node info {}/{}", index, node_count);
        write_node_info(write, node, false)?;
        let node_index = match node {
            NodePm::World(_) => 1,
            NodePm::Window(_) => 2,
            NodePm::Camera(_) => 3,
            NodePm::Display(_) => 4,
            NodePm::Light(light) => light.node_index,
            NodePm::Lod(lod) => lod.node_index,
            NodePm::Object3d(object3d) => object3d.node_index,
        };
        trace!("Node {} index: {}", index, node_index);
        let node_index = node_index | NODE_INDEX_TOP;
        write.write_u32(node_index)?;
    }

    for (index, node) in nodes.iter().enumerate() {
        trace!("Processing node data {}/{}", index, node_count);
        write_node_data(write, node)?;
        match node {
            NodePm::Lod(lod) => {
                write.write_u32(lod.parent)?;
                write_child_indices(write, &lod.children)?;
            }
            NodePm::Object3d(object3d) => {
                if let Some(parent) = object3d.parent {
                    write.write_u32(parent)?;
                }
                write_child_indices(write, &object3d.children)?;
            }
            NodePm::World(world) => {
                write_child_indices(write, &world.children)?;
            }
            _ => {}
        }
    }

    Ok(())
}
