use crate::mesh::{read_mesh_data_mw, read_mesh_info_mw, write_mesh_data_mw, write_mesh_info_mw};
use mech3ax_api_types::{MeshMw, ModelMw, Object3d};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::Result;
use mech3ax_nodes::{
    read_node_mechlib_mw, write_object_3d_data_mw, write_object_3d_info_mw, WrapperMw,
};
use std::io::{Read, Write};

fn read_node_and_mesh(
    read: &mut CountingReader<impl Read>,
    nodes: &mut Vec<Object3d>,
    meshes: &mut Vec<MeshMw>,
    mesh_ptrs: &mut Vec<i32>,
) -> Result<u32> {
    let WrapperMw {
        wrapped: mut object3d,
        has_parent,
        children_count,
    } = read_node_mechlib_mw(read)?;

    if object3d.mesh_index != 0 {
        let mesh_index: i32 = mesh_ptrs.len().try_into().unwrap();
        // preserve the pointer, store the new index
        mesh_ptrs.push(object3d.mesh_index);
        object3d.mesh_index = mesh_index;

        let wrapped_mesh = read_mesh_info_mw(read, mesh_index)?;
        let mesh = read_mesh_data_mw(read, wrapped_mesh, mesh_index)?;
        meshes.push(mesh);
    } else {
        object3d.mesh_index = -1;
    }

    // we have to apply this, so data is written out correctly again, even if
    // the mechlib data doesn't read/write parents
    object3d.parent = if has_parent { Some(0) } else { None };

    let current_index = nodes.len();
    nodes.push(object3d);

    let child_indices = (0..children_count)
        .map(|_| read_node_and_mesh(read, nodes, meshes, mesh_ptrs))
        .collect::<Result<Vec<_>>>()?;

    let object3d = &mut nodes[current_index];
    object3d.children = child_indices;

    Ok(current_index.try_into().unwrap())
}

pub fn read_model_mw(read: &mut CountingReader<impl Read>) -> Result<ModelMw> {
    let mut nodes = Vec::new();
    let mut meshes = Vec::new();
    let mut mesh_ptrs = Vec::new();
    let _root_index = read_node_and_mesh(read, &mut nodes, &mut meshes, &mut mesh_ptrs)?;
    read.assert_end()?;
    Ok(ModelMw {
        nodes,
        meshes,
        mesh_ptrs,
    })
}

fn write_node_and_mesh(
    write: &mut CountingWriter<impl Write>,
    node_index: u32,
    nodes: &mut [Object3d],
    meshes: &[MeshMw],
    mesh_ptrs: &[i32],
) -> Result<()> {
    let object3d = &mut nodes[node_index as usize];

    // preserve mesh_index
    // if the mesh_index isn't -1, then we need to restore the correct pointer
    // before the node is written out
    let restore_index = if object3d.mesh_index > -1 {
        let index = object3d.mesh_index as usize;
        object3d.mesh_index = mesh_ptrs[index];
        Some(index)
    } else {
        object3d.mesh_index = 0;
        None
    };

    write_object_3d_info_mw(write, &object3d)?;
    write_object_3d_data_mw(write, &object3d)?;

    // if mesh_index isn't -1, then we need to write out the mesh, too
    if let Some(mesh_index) = restore_index {
        let mesh = &meshes[mesh_index];
        write_mesh_info_mw(write, mesh, mesh_index)?;
        write_mesh_data_mw(write, mesh, mesh_index)?;
    }

    let child_indices = object3d.children.clone();
    for child_index in child_indices.into_iter() {
        write_node_and_mesh(write, child_index, nodes, meshes, mesh_ptrs)?;
    }
    Ok(())
}

pub fn write_model_mw(write: &mut CountingWriter<impl Write>, model: &mut ModelMw) -> Result<()> {
    write_node_and_mesh(write, 0, &mut model.nodes, &model.meshes, &model.mesh_ptrs)
}
