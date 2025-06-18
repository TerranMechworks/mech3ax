use crate::gamez::model::Model;
use crate::nodes::mw::NodeMw;
use crate::nodes::pm::NodePm;
use ::serde::{Deserialize, Serialize};
use mech3ax_metadata_proc_macro::Struct;

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct MechlibModelMw {
    pub nodes: Vec<NodeMw>,
    pub models: Vec<Model>,
    pub model_ptrs: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Struct)]
pub struct MechlibModelPm {
    pub nodes: Vec<NodePm>,
    pub models: Vec<Model>,
    pub model_ptrs: Vec<i32>,
}
