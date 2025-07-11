use crate::model::pm::{make_material_refs, write_model_data, write_model_info};
use crate::nodes::node::pm::make_node_mechlib;
use crate::nodes::NodeClass;
use log::trace;
use mech3ax_api_types::gamez::model::Model;
use mech3ax_api_types::gamez::nodes::{Node, NodeData};
use mech3ax_api_types::gamez::MechlibModel;
use mech3ax_api_types::IndexR;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{err, Error, Result};
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
        NodeData::Lod(lod) => {
            if node.model_index.is_some() {
                return Err(err!("lod node has model index"));
            }
            crate::nodes::lod::pm::write(write, lod)?;
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

        // TODO: we could get the materials here, but it would be a pain/API
        // change. they are only used to determine if the material is cycled,
        // and in the default mechlib, no materials are cycled.
        let material_refs = make_material_refs(&[], model, true);

        write_model_info(write, model, &material_refs, index)?;
        write_model_data(write, model, &material_refs, index)?;
    }

    for child_index in node.child_indices.iter().copied() {
        write_tree(write, child_index, nodes, models)?;
    }
    Ok(())
}
