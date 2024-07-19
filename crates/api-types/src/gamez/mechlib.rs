use crate::gamez::mesh::{MeshMw, MeshNg};
use crate::nodes::mw::NodeMw;
use crate::nodes::pm::NodePm;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct ModelMw {
    pub nodes: Vec<NodeMw>,
    pub meshes: Vec<MeshMw>,
    pub mesh_ptrs: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct ModelPm {
    pub nodes: Vec<NodePm>,
    pub meshes: Vec<MeshNg>,
    pub mesh_ptrs: Vec<i32>,
}
