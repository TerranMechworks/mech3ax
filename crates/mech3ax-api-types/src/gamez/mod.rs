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
pub struct GameZMwMetadata {
    pub material_array_size: i16,
    pub meshes_array_size: i32,
    pub node_array_size: u32,
    pub node_data_count: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZMwData {
    pub textures: Vec<String>,
    pub materials: Vec<Material>,
    pub meshes: Vec<MeshMw>,
    pub nodes: Vec<NodeMw>,
    pub metadata: GameZMwMetadata,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZPmMetadata {
    pub gamez_header_unk08: u32,
    pub material_array_size: i16,
    pub meshes_array_size: i32,
    pub node_array_size: u32,
    pub node_data_count: u32,
    pub texture_ptrs: Vec<Option<u32>>,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZPmData {
    pub textures: Vec<String>,
    pub materials: Vec<Material>,
    pub meshes: Vec<MeshPm>,
    pub nodes: Vec<u8>,
    // pub nodes: Vec<NodePm>,
    pub metadata: GameZPmMetadata,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZCsMetadata {
    pub gamez_header_unk08: u32,
    pub material_array_size: i16,
    // pub meshes_array_size: i32,
    pub node_array_size: u32,
    pub node_data_count: u32,
    pub texture_ptrs: Vec<Option<u32>>,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZCsData {
    pub textures: Vec<String>,
    pub materials: Vec<Material>,
    pub meshes: Vec<u8>,
    // pub meshes: Vec<MeshCs>,
    pub nodes: Vec<u8>,
    // pub nodes: Vec<NodeMw>,
    pub metadata: GameZCsMetadata,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZRcMetadata {
    pub material_array_size: i16,
    // pub meshes_array_size: i32,
    pub node_array_size: u32,
    pub node_data_count: u32,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZRcData {
    pub textures: Vec<String>,
    pub materials: Vec<Material>,
    pub meshes: Vec<u8>,
    // pub meshes: Vec<MeshRc>,
    pub nodes: Vec<u8>,
    // pub nodes: Vec<NodeRc>,
    pub metadata: GameZRcMetadata,
}
