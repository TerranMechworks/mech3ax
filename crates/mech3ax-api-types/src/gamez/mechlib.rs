use crate::gamez::mesh::{MeshMw, MeshPm};
use crate::gamez::nodes::{LodPm, NodePm, Object3d, Object3dPm};
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::{RefStruct, Union};

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct ModelMw {
    pub nodes: Vec<Object3d>,
    pub meshes: Vec<MeshMw>,
    pub mesh_ptrs: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Union)]
pub enum ModelNodePm {
    Object3d(Object3dPm),
    Lod(LodPm),
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct ModelPm {
    pub nodes: Vec<NodePm>,
    pub meshes: Vec<MeshPm>,
    pub mesh_ptrs: Vec<i32>,
}
