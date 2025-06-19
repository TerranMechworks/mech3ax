use crate::model::pm::{
    make_material_refs, read_model_data, read_model_info, write_model_data, write_model_info,
};
use log::trace;
use mech3ax_api_types::gamez::mechlib::MechlibModelPm;
use mech3ax_api_types::gamez::model::Model;
use mech3ax_api_types::nodes::pm::{Lod, NodePm, Object3d};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::Result;
use mech3ax_nodes::pm::{
    mechlib_only_err_pm, read_node_mechlib, write_node_data, write_node_info, WrappedNodePm,
    WrapperPm,
};
use std::io::{Read, Write};

fn read_node(
    read: &mut CountingReader<impl Read>,
    nodes: &mut Vec<NodePm>,
    models: &mut Vec<Model>,
    model_ptrs: &mut Vec<i32>,
) -> Result<u32> {
    trace!("Processing node {}", nodes.len());
    match read_node_mechlib(read)? {
        WrappedNodePm::Object3d(wrapped) => {
            read_node_object3d(read, nodes, models, model_ptrs, wrapped)
        }
        WrappedNodePm::Lod(wrapped) => read_node_lod(read, nodes, models, model_ptrs, wrapped),
        _ => Err(mechlib_only_err_pm()),
    }
}

fn read_node_object3d(
    read: &mut CountingReader<impl Read>,
    nodes: &mut Vec<NodePm>,
    models: &mut Vec<Model>,
    model_ptrs: &mut Vec<i32>,
    wrapped: WrapperPm<Object3d>,
) -> Result<u32> {
    let WrapperPm {
        wrapped: mut object3d,
        has_parent,
        children_count,
    } = wrapped;

    if object3d.mesh_index != 0 {
        let model_index: i32 = model_ptrs.len().try_into().unwrap();
        // preserve the pointer, store the new index
        model_ptrs.push(object3d.mesh_index);
        object3d.mesh_index = model_index;

        trace!("Processing model {}", model_index);
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
    nodes.push(NodePm::Object3d(object3d));

    let child_indices = (0..children_count)
        .map(|_| read_node(read, nodes, models, model_ptrs))
        .collect::<Result<Vec<_>>>()?;

    let object3d = match &mut nodes[current_index] {
        NodePm::Object3d(o) => o,
        _ => panic!("node should be Object3d"),
    };
    object3d.children = child_indices;

    Ok(current_index.try_into().unwrap())
}

fn read_node_lod(
    read: &mut CountingReader<impl Read>,
    nodes: &mut Vec<NodePm>,
    models: &mut Vec<Model>,
    model_ptrs: &mut Vec<i32>,
    wrapped: WrapperPm<Lod>,
) -> Result<u32> {
    let WrapperPm {
        wrapped: lod,
        has_parent: _,
        children_count,
    } = wrapped;

    let current_index = nodes.len();
    nodes.push(NodePm::Lod(lod));

    let child_indices = (0..children_count)
        .map(|_| read_node(read, nodes, models, model_ptrs))
        .collect::<Result<Vec<_>>>()?;

    let object3d = match &mut nodes[current_index] {
        NodePm::Lod(o) => o,
        _ => panic!("node should be Lod"),
    };
    object3d.children = child_indices;

    Ok(current_index.try_into().unwrap())
}

pub fn read_model(read: &mut CountingReader<impl Read>) -> Result<MechlibModelPm> {
    let mut nodes = Vec::new();
    let mut models = Vec::new();
    let mut model_ptrs = Vec::new();
    let _root_index = read_node(read, &mut nodes, &mut models, &mut model_ptrs)?;
    read.assert_end()?;
    Ok(MechlibModelPm {
        nodes,
        models,
        model_ptrs,
    })
}

fn write_node(
    write: &mut CountingWriter<impl Write>,
    node_index: u32,
    nodes: &mut [NodePm],
    models: &[Model],
    model_ptrs: &[i32],
) -> Result<()> {
    let index = node_index as usize;
    let node = &mut nodes[index];

    let restore_index = match node {
        NodePm::Object3d(object3d) => {
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
        NodePm::Lod(_) => None,
        _ => return Err(mechlib_only_err_pm()),
    };

    trace!("Processing node {}", index);
    write_node_info(write, node, true)?;
    write_node_data(write, node)?;

    // if mesh_index isn't -1, then we need to write out the model, too
    if let Some(model_index) = restore_index {
        let model = &models[model_index];
        trace!("Processing model {}", model_index);
        // TODO: we could get the materials here, but it would be a pain/API
        // change. they are only used to determine if the material is cycled,
        // and in the default mechlib, no materials are cycled.
        let material_refs = make_material_refs(&[], model, true);
        write_model_info(write, model, &material_refs, model_index)?;
        write_model_data(write, model, &material_refs, model_index)?;
    }

    let child_indices = match node {
        NodePm::Object3d(object3d) => object3d.children.clone(),
        NodePm::Lod(lod) => lod.children.clone(),
        _ => unreachable!(),
    };

    for child_index in child_indices.into_iter() {
        write_node(write, child_index, nodes, models, model_ptrs)?;
    }
    Ok(())
}

pub fn write_model(
    write: &mut CountingWriter<impl Write>,
    mechlib_model: &mut MechlibModelPm,
) -> Result<()> {
    write_node(
        write,
        0,
        &mut mechlib_model.nodes,
        &mechlib_model.models,
        &mechlib_model.model_ptrs,
    )
}
