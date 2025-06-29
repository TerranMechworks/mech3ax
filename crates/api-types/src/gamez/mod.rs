pub mod materials;
pub mod mechlib;
pub mod model;

use crate::fld;
use crate::nodes::mw::NodeMw;
use crate::nodes::pm::NodePm;
use crate::nodes::rc::NodeRc;
use materials::Material;
use mech3ax_timestamp::DateTime;
use model::Model;

fld! {
    struct GameZMetadata {
        datetime: DateTime,
        model_array_size: i32,
        node_array_size: i32,
        node_data_count: i32,
    }
}

fld! {
    struct Texture {
        name: String,
        mip: i32 = { -1i32 },
    }
}

fld! {
    struct GameZDataMw {
        textures: Vec<Texture>,
        materials: Vec<Material>,
        models: Vec<Model>,
        nodes: Vec<NodeMw>,
        metadata: GameZMetadata,
    }
}

fld! {
    struct GameZDataPm {
        textures: Vec<Texture>,
        materials: Vec<Material>,
        models: Vec<Model>,
        nodes: Vec<NodePm>,
        metadata: GameZMetadata,
    }
}

fld! {
    struct GameZDataRc {
        textures: Vec<Texture>,
        materials: Vec<Material>,
        models: Vec<Model>,
        nodes: Vec<NodeRc>,
        metadata: GameZMetadata,
    }
}
