use crate::enums::ENUM_IMPL;
use crate::structs::{CLASS_IMPL, STRUCT_IMPL};
use crate::unions::UNION_IMPL;
use minijinja::Environment;

pub fn make_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_template("enum_impl.cs", ENUM_IMPL).unwrap();
    env.add_template("class_impl.cs", CLASS_IMPL).unwrap();
    env.add_template("struct_impl.cs", STRUCT_IMPL).unwrap();
    env.add_template("union_impl.cs", UNION_IMPL).unwrap();
    env
}
