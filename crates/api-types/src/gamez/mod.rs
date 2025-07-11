pub mod materials;
pub mod model;
pub mod nodes;

use crate::{api, sum, Color, Count, IndexO};
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
    struct MechlibTexturedMaterial {
        texture_name: String,
        // flag: bool,
        ptr: u32,
    }
}

api! {
    struct MechlibColoredMaterial {
        color: Color,
        alpha: u8,
    }
}

sum! {
    enum MechlibMaterial {
        Textured(MechlibTexturedMaterial),
        Colored(MechlibColoredMaterial),
    }
}
api! {
    struct Texture {
        name: String,
        #[serde(skip_serializing_if = "IndexO::is_none", default)]
        mip_index: IndexO = { -1i16 },
    }
}

api! {
    struct GameZMetadata {
        datetime: DateTime,
        material_array_size: Count,
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
