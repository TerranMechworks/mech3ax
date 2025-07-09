use crate::api;
use crate::gamez::model::Model;
use crate::gamez::nodes::Node;

api! {
    struct MechlibModel {
        nodes: Vec<Node>,
        models: Vec<Model>,
    }
}
