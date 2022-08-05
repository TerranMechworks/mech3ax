use mech3ax_api_types::Vec3;
use serde::{Deserialize, Serialize};

pub const INPUT_NODE: &str = "INPUT_NODE";

#[derive(Debug, Serialize, Deserialize)]
pub struct AtNode {
    pub node: String,
    pub translation: Vec3,
}
