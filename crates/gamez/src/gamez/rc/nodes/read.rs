use super::NODE_INDEX_INVALID;
use crate::nodes::NodeClass;
use crate::nodes::helpers::read_node_indices;
use crate::nodes::node::rc::{NodeRcC, assert_node, assert_node_zero};
use log::trace;
use mech3ax_api_types::gamez::nodes::{Node, NodeData};
use mech3ax_api_types::{Count, IndexR32};
use mech3ax_common::check::amend_err;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, err};
use mech3ax_types::u32_to_usize;
use std::io::{Read, Seek};

pub(crate) fn read_nodes(
    read: &mut CountingReader<impl Read + Seek>,
    array_size: Count,
    model_count: Count,
) -> Result<Vec<Node>> {
    // this code assumes the nodes are contiguous, but strictly speaking the
    // engine likely doesn't require this
    let mut count = -1;
    let prev_offset = read.offset;
    for index in array_size.iter() {
        let node: NodeRcC = read.read_struct_no_log()?;
        if node.is_zero() {
            count = index.to_i32();
            break;
        }
        let _ = read.read_i32()?;
    }
    read.seek(std::io::SeekFrom::Start(prev_offset as _))?;

    // need at least world, window, camera, display, and light
    if count < 6 {
        return Err(err!("Too few nodes in GameZ ({})", count));
    }

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
            trace!("Node data offset: {}", node_info.offset);
            Ok(node_info)
        })
        .collect::<Result<Vec<_>>>()?;

    trace!(
        "Processing {}..{} node info zeros at {}",
        count, array_size, read.offset
    );
    for index in count..array_size.to_i32() {
        let node: NodeRcC = read.read_struct_no_log()?;
        assert_node_zero(&node, read.prev)
            .inspect_err(|_| trace!("{:#?} (index: {}, at {})", node, index, read.prev))?;

        let actual_index = read.read_i32()?;

        let mut expected_index = index + 1;
        if expected_index == array_size.to_i32() {
            expected_index = NODE_INDEX_INVALID;
        }

        if actual_index != expected_index {
            return Err(err!(
                "node offset {} != {} (index: {}, at {})",
                actual_index,
                expected_index,
                index,
                read.prev,
            ));
        }
    }
    trace!("Processed node info zeros at {}", read.offset);

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
                        node_offset,
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
                index: 0,
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
                    let parent_index = IndexR32::new(node_info.offset as i32);
                    let parent_index = parent_index.check().map_err(|msg| {
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
                    let object3d = crate::nodes::object3d::read(read)?;
                    node.data = NodeData::Object3d(object3d);
                }
                NodeClass::Window => {
                    let window = crate::nodes::window::read(read)?;
                    node.data = NodeData::Window(window);
                }
                NodeClass::World => {
                    let world = crate::nodes::world::rc::read(read)?;
                    node.data = NodeData::World(world);
                }
            };

            if !node_info.parent_count.is_empty() {
                node.parent_indices =
                    read_node_indices!(read, node_info.parent_count, |idx, cnt| {
                        format!("node {}/{} parent index {}/{}", index, count, idx, cnt)
                    })?;
            }

            if !node_info.child_count.is_empty() {
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
