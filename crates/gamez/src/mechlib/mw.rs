use crate::model::mw::{read_model_data, read_model_info, write_model_data, write_model_info};
use log::trace;
use mech3ax_api_types::gamez::mechlib::MechlibModelMw;
use mech3ax_api_types::gamez::model::Model;
use mech3ax_api_types::nodes::mw::{NodeMw, Object3d};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::Result;
use mech3ax_nodes::mw::{
    mechlib_only_err_mw, read_node_mechlib, write_node_data, write_node_info, WrappedNodeMw,
    WrapperMw,
};
use std::io::{Read, Write};

fn read_node_and_model(
    read: &mut CountingReader<impl Read>,
    nodes: &mut Vec<NodeMw>,
    models: &mut Vec<Model>,
    model_ptrs: &mut Vec<i32>,
) -> Result<u32> {
    trace!("Reading node {}", nodes.len());
    match read_node_mechlib(read)? {
        WrappedNodeMw::Object3d(wrapped) => {
            read_node_and_model_object3d(read, nodes, models, model_ptrs, wrapped)
        }
        _ => Err(mechlib_only_err_mw()),
    }
}

fn read_node_and_model_object3d(
    read: &mut CountingReader<impl Read>,
    nodes: &mut Vec<NodeMw>,
    models: &mut Vec<Model>,
    model_ptrs: &mut Vec<i32>,
    wrapped: WrapperMw<Object3d>,
) -> Result<u32> {
    let WrapperMw {
        wrapped: mut object3d,
        has_parent,
        children_count,
    } = wrapped;

    if object3d.mesh_index != 0 {
        let model_index: i32 = model_ptrs.len().try_into().unwrap();
        // preserve the pointer, store the new index
        model_ptrs.push(object3d.mesh_index);
        object3d.mesh_index = model_index;

        trace!("Reading model {}", model_index);
        let wrapped = read_model_info(read)?;
        // TODO: we ought to base this on the materials in mechlib, but...
        let material_count = 4096;
        let model = read_model_data(read, wrapped, material_count)?;
        models.push(model);
    } else {
        object3d.mesh_index = -1;
    }

    // we have to apply this, so data is written out correctly again, even if
    // the mechlib data doesn't read/write parents
    object3d.parent = if has_parent { Some(0) } else { None };

    let current_index = nodes.len();
    nodes.push(NodeMw::Object3d(object3d));

    let child_indices = (0..children_count)
        .map(|_| read_node_and_model(read, nodes, models, model_ptrs))
        .collect::<Result<Vec<_>>>()?;

    let object3d = match &mut nodes[current_index] {
        NodeMw::Object3d(o) => o,
        _ => panic!("node should be Object3d"),
    };
    object3d.children = child_indices;

    Ok(current_index.try_into().unwrap())
}

pub fn read_model(read: &mut CountingReader<impl Read>) -> Result<MechlibModelMw> {
    let mut nodes = Vec::new();
    let mut models = Vec::new();
    let mut model_ptrs = Vec::new();
    let _root_index = read_node_and_model(read, &mut nodes, &mut models, &mut model_ptrs)?;
    read.assert_end()?;
    Ok(MechlibModelMw {
        nodes,
        models,
        model_ptrs,
    })
}

fn write_node_and_model(
    write: &mut CountingWriter<impl Write>,
    node_index: u32,
    nodes: &mut [NodeMw],
    models: &[Model],
    model_ptrs: &[i32],
) -> Result<()> {
    let index = node_index as usize;
    let node = &mut nodes[index];

    let restore_index = match node {
        NodeMw::Object3d(object3d) => {
            // preserve mesh_index
            // if the mesh_index isn't -1, then we need to restore the correct pointer
            // before the node is written out
            if object3d.mesh_index > -1 {
                let index = object3d.mesh_index as usize;
                object3d.mesh_index = model_ptrs[index];
                Some(index)
            } else {
                object3d.mesh_index = 0;
                None
            }
        }
        _ => return Err(mechlib_only_err_mw()),
    };

    trace!("Writing node {}", index);
    write_node_info(write, node)?;
    write_node_data(write, node)?;

    // if mesh_index isn't -1, then we need to write out the model, too
    if let Some(model_index) = restore_index {
        let model = &models[model_index];
        trace!("Writing model {}", model_index);
        write_model_info(write, model)?;
        write_model_data(write, model)?;
    }

    let child_indices = match node {
        NodeMw::Object3d(object3d) => object3d.children.clone(),
        _ => unreachable!(),
    };

    for child_index in child_indices.into_iter() {
        write_node_and_model(write, child_index, nodes, models, model_ptrs)?;
    }
    Ok(())
}

pub fn write_model(
    write: &mut CountingWriter<impl Write>,
    mechlib_model: &mut MechlibModelMw,
) -> Result<()> {
    write_node_and_model(
        write,
        0,
        &mut mechlib_model.nodes,
        &mechlib_model.models,
        &mechlib_model.model_ptrs,
    )
}
