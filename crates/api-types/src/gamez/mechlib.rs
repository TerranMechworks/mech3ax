use crate::api;
use crate::gamez::model::Model;
use crate::nodes::mw::NodeMw;
use crate::nodes::pm::NodePm;

api! {
    struct MechlibModelMw {
        nodes: Vec<NodeMw>,
        models: Vec<Model>,
        model_ptrs: Vec<i32>,
    }
}

api! {
    struct MechlibModelPm {
        nodes: Vec<NodePm>,
        models: Vec<Model>,
        model_ptrs: Vec<i32>,
    }
}
