use crate::gamez::mesh::Mesh;
use crate::gamez::nodes::Node;
use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolvedNode(pub Node<ResolvedNode>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub root: ResolvedNode,
    pub meshes: Vec<Mesh>,
    pub mesh_ptrs: Vec<i32>,
}
