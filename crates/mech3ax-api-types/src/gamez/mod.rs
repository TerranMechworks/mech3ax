mod materials;
mod mechlib;
mod mesh;
mod nodes;

pub use materials::*;
pub use mechlib::*;
pub use mesh::*;
pub use nodes::*;

use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::RefStruct;

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZMetadata {
    pub material_array_size: i16,
    pub meshes_array_size: i32,
    pub node_array_size: u32,
    pub node_data_count: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZData {
    pub metadata: GameZMetadata,
    pub textures: Vec<String>,
    pub materials: Vec<Material>,
    pub meshes: Vec<Mesh>,
    pub nodes: Vec<Node>,
}
