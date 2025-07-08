use super::NODE_INDEX_INVALID;
use log::trace;
use mech3ax_api_types::nodes::mw::NodeMw;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, Result};
use mech3ax_nodes::common::write_child_indices;
use mech3ax_nodes::mw::{size_node, write_node_data, write_node_info, NodeMwC};
use mech3ax_types::AsBytes as _;
use std::io::Write;

pub(crate) fn write_nodes(
    write: &mut CountingWriter<impl Write>,
    nodes: &[NodeMw],
    array_size: i32,
    offset: u32,
) -> Result<()> {
    let mut offset = offset + (NodeMwC::SIZE + 4) * (array_size as u32);
    let node_count = assert_len!(i32, nodes.len(), "GameZ nodes")?;

    for (index, node) in nodes.iter().enumerate() {
        trace!("Processing node info {}/{}", index, node_count);
        write_node_info(write, node)?;

        let node_data_offset = match node {
            NodeMw::Empty(empty) => empty.parent,
            _ => offset,
        };

        trace!("Node {} data offset: {}", index, node_data_offset);
        write.write_u32(node_data_offset)?;
        offset += size_node(node);
    }
    // padding for spurious "Processing node info", since the count is not known
    // and we have to peek for zero node infos.
    trace!("");

    trace!(
        "Processing {}..{} node info zeros at {}",
        node_count,
        array_size,
        write.offset
    );
    let node_zero = NodeMwC::zero();
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
        if !matches!(node, NodeMw::Empty(_)) {
            trace!("Processing node data {}/{}", index, node_count);
        }
        write_node_data(write, node)?;
        match node {
            NodeMw::Lod(lod) => {
                write.write_u32(lod.parent)?;
                write_child_indices(write, &lod.children)?;
            }
            NodeMw::Object3d(object3d) => {
                if let Some(parent) = object3d.parent {
                    write.write_u32(parent)?;
                }
                write_child_indices(write, &object3d.children)?;
            }
            NodeMw::World(world) => {
                write_child_indices(write, &world.children)?;
            }
            _ => {}
        }
    }

    Ok(())
}
