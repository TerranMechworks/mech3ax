use crate::model::mw::{write_model_data, write_model_info};
use crate::nodes::NodeClass;
use crate::nodes::node::mw::make_node_mechlib;
use log::trace;
use mech3ax_api_types::IndexR;
use mech3ax_api_types::gamez::MechlibModel;
use mech3ax_api_types::gamez::model::Model;
use mech3ax_api_types::gamez::nodes::{Node, NodeData};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{Error, Result, err};
use std::io::Write;

pub fn write_model(
    write: &mut CountingWriter<impl Write>,
    mechlib_model: &MechlibModel,
) -> Result<()> {
    write_tree(
        write,
        IndexR::ZERO,
        &mechlib_model.nodes,
        &mechlib_model.models,
    )
}

fn write_tree(
    write: &mut CountingWriter<impl Write>,
    node_index: IndexR,
    nodes: &[Node],
    models: &[Model],
) -> Result<()> {
    let index = node_index.to_usize();
    trace!("Processing node {}", index);

    let node = nodes
        .get(index)
        .ok_or_else(|| -> Error { err!("invalid node index {}", index) })?;

    let node_info = make_node_mechlib(node)?;
    write.write_struct(&node_info)?;

    match &node.data {
        NodeData::Object3d(object3d) => {
            crate::nodes::object3d::write(write, object3d)?;
        }
        _ => {
            let node_class = NodeClass::from_data(&node.data);
            return Err(err!(
                "expected {:?} to be {:?} in mechlib",
                node_class,
                NodeClass::Object3d
            ));
        }
    }

    if let Some(model_index) = node.model_index.to_req() {
        let index = model_index.to_usize();
        trace!("Processing model {}", model_index);

        let model = models
            .get(index)
            .ok_or_else(|| -> Error { err!("invalid model index {}", index) })?;

        write_model_info(write, model, index)?;
        write_model_data(write, model, index)?;
    }

    for child_index in node.child_indices.iter().copied() {
        write_tree(write, child_index, nodes, models)?;
    }
    Ok(())
}
