use crate::gamez::mesh::Mesh;
use crate::gamez::nodes::Object3d;
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub nodes: Vec<Object3d>,
    pub meshes: Vec<Mesh>,
    pub mesh_ptrs: Vec<i32>,
}
