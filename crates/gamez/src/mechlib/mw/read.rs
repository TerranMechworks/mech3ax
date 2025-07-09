use crate::model::mw::{read_model_data, read_model_info};
use crate::nodes::node::mw::{assert_node_mechlib, NodeMwC};
use log::trace;
use mech3ax_api_types::gamez::mechlib::MechlibModel;
use mech3ax_api_types::gamez::model::Model;
use mech3ax_api_types::gamez::nodes::{Node, NodeData};
use mech3ax_api_types::Index;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{err, Error, Result};
use std::io::Read;

pub fn read_model(read: &mut CountingReader<impl Read>) -> Result<MechlibModel> {
    let mut nodes = Vec::new();
    let mut models = Vec::new();
    let _root_index = read_tree(read, &mut nodes, &mut models, None)?;
    read.assert_end()?;
    Ok(MechlibModel { nodes, models })
}

fn usize_to_index(len: usize) -> Option<Index> {
    len.try_into().ok().and_then(Index::from_i16)
}

fn read_tree(
    read: &mut CountingReader<impl Read>,
    nodes: &mut Vec<Node>,
    models: &mut Vec<Model>,
    parent_index: Option<Index>,
) -> Result<Index> {
    let node_index = nodes.len();
    let index = usize_to_index(node_index).ok_or_else(|| -> Error { err!("too many nodes") })?;

    trace!("Processing node {}", node_index);

    let node: NodeMwC = read.read_struct()?;
    let node_info = assert_node_mechlib(&node, read.prev)?;
    let object3d = crate::nodes::object3d::read(read)?;

    // this holds the model ptr for mechlib
    let model_index = if node_info.offset != 0 {
        // the model index is the current length, as we'll push a new model
        let model_index =
            usize_to_index(models.len()).ok_or_else(|| -> Error { err!("too many models") })?;

        let wrapped = read_model_info(read)?;
        // TODO
        let material_count = 4096;
        let model = read_model_data(read, wrapped, material_count)?;

        models.push(model);

        Some(model_index)
    } else {
        None
    };

    let parent_indices = match (parent_index, node_info.parent_count.to_i16()) {
        (Some(pi), 1) => vec![pi],
        (None, 0) => vec![],
        (Some(_), pc) => return Err(err!("expected parent count to be 1, but was {}", pc)),
        (None, pc) => return Err(err!("expected parent count to be 0, but was {}", pc)),
    };

    let node = Node {
        name: node_info.name,
        flags: node_info.flags,
        update_flags: node_info.update_flags,
        zone_id: node_info.zone_id,
        model_index,
        area_partition: node_info.area_partition,
        virtual_partition: node_info.virtual_partition,
        parent_indices,
        child_indices: Vec::new(),
        active_bbox: node_info.active_bbox,
        node_bbox: node_info.node_bbox,
        model_bbox: node_info.model_bbox,
        child_bbox: node_info.child_bbox,
        field192: node_info.field192,
        field196: node_info.field196,
        field200: node_info.field200,
        field204: node_info.field204,
        data: NodeData::Object3d(object3d),
        data_ptr: node_info.data_ptr.0,
        parent_array_ptr: node_info.parent_array_ptr.0,
        child_array_ptr: node_info.child_array_ptr.0,
        index: node_info.offset,
    };

    // we must push the node, but need to set the child indices after
    nodes.push(node);

    let child_indices = (0..node_info.child_count.to_i16())
        .map(|_index| read_tree(read, nodes, models, Some(index)))
        .collect::<Result<Vec<_>>>()?;

    let node = &mut nodes[node_index];
    node.child_indices = child_indices;

    Ok(index)
}
