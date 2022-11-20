use crate::mesh::ng::{read_mesh_data, read_mesh_info, write_mesh_data, write_mesh_info};
use mech3ax_api_types::gamez::mechlib::ModelPm;
use mech3ax_api_types::gamez::mesh::MeshNg;
use mech3ax_api_types::gamez::nodes::pm::{LodPm, NodePm, Object3dPm};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::Result;
use mech3ax_nodes::pm::{
    read_node_mechlib, write_node_data, write_node_info, WrappedNodePm, WrapperPm,
};
use std::io::{Read, Write};

fn read_node_and_mesh(
    read: &mut CountingReader<impl Read>,
    nodes: &mut Vec<NodePm>,
    meshes: &mut Vec<MeshNg>,
    mesh_ptrs: &mut Vec<i32>,
) -> Result<u32> {
    match read_node_mechlib(read, nodes.len())? {
        WrappedNodePm::Object3d(wrapped) => {
            read_node_and_mesh_object3d(read, nodes, meshes, mesh_ptrs, wrapped)
        }
        WrappedNodePm::Lod(wrapped) => {
            read_node_and_mesh_lod(read, nodes, meshes, mesh_ptrs, wrapped)
        } // _ => Err(mechlib_only_err_pm()),
    }
}

fn read_node_and_mesh_object3d(
    read: &mut CountingReader<impl Read>,
    nodes: &mut Vec<NodePm>,
    meshes: &mut Vec<MeshNg>,
    mesh_ptrs: &mut Vec<i32>,
    wrapped: WrapperPm<Object3dPm>,
) -> Result<u32> {
    let WrapperPm {
        wrapped: mut object3d,
        has_parent,
        children_count,
    } = wrapped;

    if object3d.mesh_index != 0 {
        let mesh_index: i32 = mesh_ptrs.len().try_into().unwrap();
        // preserve the pointer, store the new index
        mesh_ptrs.push(object3d.mesh_index);
        object3d.mesh_index = mesh_index;

        let wrapped_mesh = read_mesh_info(read, mesh_index)?;
        let mesh = read_mesh_data(read, wrapped_mesh, mesh_index)?;
        meshes.push(mesh);
    } else {
        object3d.mesh_index = -1;
    }

    // we have to apply this, so data is written out correctly again, even if
    // the mechlib data doesn't read/write parents
    object3d.parent = if has_parent { Some(0) } else { None };

    let current_index = nodes.len();
    nodes.push(NodePm::Object3d(object3d));

    let child_indices = (0..children_count)
        .map(|_| read_node_and_mesh(read, nodes, meshes, mesh_ptrs))
        .collect::<Result<Vec<_>>>()?;

    let object3d = match &mut nodes[current_index] {
        NodePm::Object3d(o) => o,
        _ => panic!("node should be Object3d"),
    };
    object3d.children = child_indices;

    Ok(current_index.try_into().unwrap())
}

fn read_node_and_mesh_lod(
    read: &mut CountingReader<impl Read>,
    nodes: &mut Vec<NodePm>,
    meshes: &mut Vec<MeshNg>,
    mesh_ptrs: &mut Vec<i32>,
    wrapped: WrapperPm<LodPm>,
) -> Result<u32> {
    let WrapperPm {
        wrapped: lod,
        has_parent: _,
        children_count,
    } = wrapped;

    let current_index = nodes.len();
    nodes.push(NodePm::Lod(lod));

    let child_indices = (0..children_count)
        .map(|_| read_node_and_mesh(read, nodes, meshes, mesh_ptrs))
        .collect::<Result<Vec<_>>>()?;

    let object3d = match &mut nodes[current_index] {
        NodePm::Lod(o) => o,
        _ => panic!("node should be Lod"),
    };
    object3d.children = child_indices;

    Ok(current_index.try_into().unwrap())
}

pub fn read_model(read: &mut CountingReader<impl Read>) -> Result<ModelPm> {
    let mut nodes = Vec::new();
    let mut meshes = Vec::new();
    let mut mesh_ptrs = Vec::new();
    let _root_index = read_node_and_mesh(read, &mut nodes, &mut meshes, &mut mesh_ptrs)?;
    read.assert_end()?;
    Ok(ModelPm {
        nodes,
        meshes,
        mesh_ptrs,
    })
}

fn write_node_and_mesh(
    write: &mut CountingWriter<impl Write>,
    node_index: u32,
    nodes: &mut [NodePm],
    meshes: &[MeshNg],
    mesh_ptrs: &[i32],
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
                object3d.mesh_index = mesh_ptrs[index];
                Some(index)
            } else {
                object3d.mesh_index = 0;
                None
            }
        }
        NodePm::Lod(_) => None,
        // _ => return Err(mechlib_only_err_pm()),
    };

    write_node_info(write, &node, true, index)?;
    write_node_data(write, &node, index)?;

    // if mesh_index isn't -1, then we need to write out the mesh, too
    if let Some(mesh_index) = restore_index {
        let mesh = &meshes[mesh_index];
        // Cast safety: mesh index is for debug only
        write_mesh_info(write, mesh, mesh_index as _)?;
        write_mesh_data(write, mesh, mesh_index)?;
    }

    let child_indices = match node {
        NodePm::Object3d(object3d) => object3d.children.clone(),
        NodePm::Lod(lod) => lod.children.clone(),
        // _ => unreachable!(),
    };

    for child_index in child_indices.into_iter() {
        write_node_and_mesh(write, child_index, nodes, meshes, mesh_ptrs)?;
    }
    Ok(())
}

pub fn write_model(write: &mut CountingWriter<impl Write>, model: &mut ModelPm) -> Result<()> {
    write_node_and_mesh(write, 0, &mut model.nodes, &model.meshes, &model.mesh_ptrs)
}
