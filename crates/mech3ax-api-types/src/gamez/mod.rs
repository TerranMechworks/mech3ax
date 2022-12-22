pub mod materials;
pub mod mechlib;
pub mod mesh;

use crate::nodes::cs::NodeCs;
use crate::nodes::mw::NodeMw;
use crate::nodes::pm::NodePm;
use crate::nodes::rc::NodeRc;
use ::serde::{Deserialize, Serialize};
use materials::Material;
use mech3ax_metadata_proc_macro::RefStruct;
use mesh::{MeshMw, MeshNg, MeshRc};

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZMwMetadata {
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
    pub meshes_array_size: i32,
    pub node_data_count: u32,
    pub texture_ptrs: Vec<Option<u32>>,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZPmData {
    pub textures: Vec<String>,
    pub materials: Vec<Material>,
    pub meshes: Vec<MeshNg>,
    pub nodes: Vec<NodePm>,
    pub metadata: GameZPmMetadata,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZCsMetadata {
    pub gamez_header_unk08: u32,
    pub texture_ptrs: Vec<Option<u32>>,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZCsData {
    pub textures: Vec<String>,
    pub materials: Vec<Material>,
    pub meshes: Vec<Option<MeshNg>>,
    pub nodes: Vec<NodeCs>,
    pub metadata: GameZCsMetadata,
}

#[derive(Debug, Serialize, Deserialize, RefStruct)]
pub struct GameZRcData {
    pub textures: Vec<String>,
    pub materials: Vec<Material>,
    pub meshes: Vec<MeshRc>,
    pub nodes: Vec<NodeRc>,
}
