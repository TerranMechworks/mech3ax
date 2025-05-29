pub mod materials;
pub mod mechlib;
pub mod mesh;

use crate::nodes::cs::NodeCs;
use crate::nodes::mw::NodeMw;
use crate::nodes::pm::NodePm;
use crate::nodes::rc::NodeRc;
use crate::serde::{i32_is_neg_one, i32_neg_one};
use ::serde::{Deserialize, Serialize};
use materials::Material;
use mech3ax_metadata_proc_macro::Struct;
use mech3ax_timestamp::DateTime;
use mesh::{MeshMw, MeshNg, ModelRc};

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Texture {
    pub name: String,
    #[serde(skip_serializing_if = "i32_is_neg_one", default = "i32_neg_one")]
    pub mip: i32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct GameZMetadataMw {
    pub meshes_array_size: i32,
    pub node_array_size: u32,
    pub node_data_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(partial, namespace = "Mech3DotNet.Zbd")]
pub struct GameZDataMw {
    pub textures: Vec<String>,
    pub materials: Vec<Material>,
    pub meshes: Vec<MeshMw>,
    pub nodes: Vec<NodeMw>,
    pub metadata: GameZMetadataMw,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct GameZMetadataPm {
    pub datetime: DateTime,
    pub meshes_array_size: i32,
    pub node_data_count: u32,
    pub texture_ptrs: Vec<Option<u32>>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(partial, namespace = "Mech3DotNet.Zbd")]
pub struct GameZDataPm {
    pub textures: Vec<String>,
    pub materials: Vec<Material>,
    pub meshes: Vec<MeshNg>,
    pub nodes: Vec<NodePm>,
    pub metadata: GameZMetadataPm,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct GameZMetadataCs {
    pub datetime: DateTime,
    pub texture_ptrs: Vec<Option<u32>>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct TextureName {
    pub original: String,
    pub renamed: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(partial, namespace = "Mech3DotNet.Zbd")]
pub struct GameZDataCs {
    pub textures: Vec<TextureName>,
    pub materials: Vec<Material>,
    pub meshes: Vec<Option<MeshNg>>,
    pub nodes: Vec<NodeCs>,
    pub metadata: GameZMetadataCs,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct GameZMetadataRc {
    pub model_array_size: i32,
    pub node_array_size: u32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(partial, namespace = "Mech3DotNet.Zbd")]
pub struct GameZDataRc {
    pub textures: Vec<Texture>,
    pub materials: Vec<Material>,
    pub models: Vec<ModelRc>,
    pub nodes: Vec<NodeRc>,
    pub metadata: GameZMetadataRc,
}
