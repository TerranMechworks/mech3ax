pub mod materials;
pub mod model;
pub mod nodes;

use crate::{api, Count, Index};
use materials::Material;
use mech3ax_timestamp::DateTime;
use model::Model;
use nodes::Node;

api! {
    struct MechlibModel {
        nodes: Vec<Node>,
        models: Vec<Model>,
    }
}

api! {
    struct Texture {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        mip_index: Option<Index> = { None },
    }
}

api! {
    struct GameZMetadata {
        datetime: DateTime,
        model_array_size: Count,
        node_array_size: Count,
        node_data_count: Count,
    }
}

api! {
    struct GameZ {
        textures: Vec<Texture>,
        materials: Vec<Material>,
        models: Vec<Model>,
        nodes: Vec<Node>,
        metadata: GameZMetadata,
    }
}
