mod materials;
mod mechlib;
mod mesh;
mod nodes;

use ::serde::{Deserialize, Serialize};

pub use materials::*;
pub use mechlib::*;
pub use mesh::*;
pub use nodes::*;

pub type IndexedNode = Node<u32>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub material_array_size: i16,
    pub meshes_array_size: i32,
    pub node_array_size: u32,
    pub node_data_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameZ {
    pub metadata: Metadata,
    pub textures: Vec<String>,
    pub materials: Vec<Material>,
    pub meshes: Vec<Mesh>,
    pub nodes: Vec<IndexedNode>,
}
