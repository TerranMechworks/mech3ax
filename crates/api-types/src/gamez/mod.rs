pub mod materials;
pub mod mechlib;
pub mod model;

use crate::nodes::mw::NodeMw;
use crate::nodes::pm::NodePm;
use crate::nodes::rc::NodeRc;
use crate::serde::{i32_is_neg_one, i32_neg_one};
use ::serde::{Deserialize, Serialize};
use materials::Material;
use mech3ax_metadata_proc_macro::Struct;
use mech3ax_timestamp::DateTime;
use model::Model;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct Texture {
    pub name: String,
    #[serde(skip_serializing_if = "i32_is_neg_one", default = "i32_neg_one")]
    pub mip: i32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct GameZMetadataMw {
    pub model_array_size: i32,
    pub node_array_size: i32,
    pub node_data_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(partial, namespace = "Mech3DotNet.Zbd")]
pub struct GameZDataMw {
    pub textures: Vec<Texture>,
    pub materials: Vec<Material>,
    pub models: Vec<Model>,
    pub nodes: Vec<NodeMw>,
    pub metadata: GameZMetadataMw,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct GameZMetadataPm {
    pub datetime: DateTime,
    pub model_array_size: i32,
    pub node_data_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(partial, namespace = "Mech3DotNet.Zbd")]
pub struct GameZDataPm {
    pub textures: Vec<Texture>,
    pub materials: Vec<Material>,
    pub models: Vec<Model>,
    pub nodes: Vec<NodePm>,
    pub metadata: GameZMetadataPm,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct GameZMetadataRc {
    pub model_array_size: i32,
    pub node_array_size: i32,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
#[dotnet(partial, namespace = "Mech3DotNet.Zbd")]
pub struct GameZDataRc {
    pub textures: Vec<Texture>,
    pub materials: Vec<Material>,
    pub models: Vec<Model>,
    pub nodes: Vec<NodeRc>,
    pub metadata: GameZMetadataRc,
}
