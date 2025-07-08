use super::{NODE_INDEX_BOT_MASK, NODE_INDEX_TOP, NODE_INDEX_TOP_MASK};
use crate::nodes::helpers::read_node_indices;
use crate::nodes::node::pm::{assert_node, NodePmC};
use crate::nodes::NodeClass;
use log::trace;
use mech3ax_api_types::gamez::nodes::{Node, NodeData};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{chk, Result};
use std::io::Read;

pub(crate) fn read_nodes(
    read: &mut CountingReader<impl Read>,
    array_size: i32,
    model_count: i32,
) -> Result<Vec<Node>> {
    // in PM, zeroed out nodes aren't written
    let count = array_size;

    let nodes = (0..count)
        .map(|index| {
            trace!("Processing node info {}/{}", index, count);
            let node: NodePmC = read.read_struct()?;
            let mut node_info = assert_node(&node, read.prev, model_count)?;

            let node_index = read.read_u32()?;
            trace!("Node index: 0x{node_index:08X}");

            // TODO
            // let top = node_index & NODE_INDEX_TOP_MASK;
            // chk!(read.prev => "node index top", top == NODE_INDEX_TOP)?;
            // let bot = node_index & NODE_INDEX_BOT_MASK;
            // chk!(read.prev => "node index bot", bot == (index as u32).wrapping_add(1))?;

            node_info.offset = node_index;

            Ok(node_info)
        })
        .collect::<Result<Vec<_>>>()?;

    let nodes = nodes
        .into_iter()
        .enumerate()
        .map(|(index, node_info)| {
            trace!("Processing node data {}/{}", index, count);

            let mut node = Node {
                name: node_info.name,
                flags: node_info.flags,
                update_flags: node_info.update_flags,
                zone_id: node_info.zone_id,
                model_index: node_info.model_index,
                area_partition: node_info.area_partition,
                virtual_partition: node_info.virtual_partition,
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
                index: node_info.offset,
            };

            match node_info.node_class {
                NodeClass::Camera => {
                    let camera = crate::nodes::camera::read(read)?;
                    node.data = NodeData::Camera(camera);
                }
                NodeClass::Display => {
                    let display = crate::nodes::display::read(read)?;
                    node.data = NodeData::Display(display);
                }
                NodeClass::Empty => {
                    unreachable!("pm empty")
                }
                NodeClass::Light => {
                    let light = crate::nodes::light::pm::read(read)?;
                    node.data = NodeData::Light(light);
                }
                NodeClass::Lod => {
                    let lod = crate::nodes::lod::pm::read(read)?;
                    node.data = NodeData::Lod(lod);
                }
                NodeClass::Object3d => {
                    let object3d = crate::nodes::object3d::read(read)?;
                    node.data = NodeData::Object3d(object3d);
                }
                NodeClass::Window => {
                    let window = crate::nodes::window::read(read)?;
                    node.data = NodeData::Window(window);
                }
                NodeClass::World => {
                    let world = crate::nodes::world::pm::read(read)?;
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

    trace!("Processed node data at {}", read.offset);
    Ok(nodes)
}
