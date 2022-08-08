use crate::gamez::mesh::Mesh;
use crate::gamez::nodes::Object3d;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::RefStruct;

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct Model {
    pub nodes: Vec<Object3d>,
    pub meshes: Vec<Mesh>,
    pub mesh_ptrs: Vec<i32>,
}
