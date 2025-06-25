use super::enums::ENUM_IMPL;
use super::flags::FLAGS_IMPL;
use super::structs::{CLASS_IMPL, STRUCT_IMPL};
use super::unions::UNION_IMPL;
use minijinja::Environment;

pub(crate) fn make_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_template("enum_impl.cs", ENUM_IMPL).unwrap();
    env.add_template("class_impl.cs", CLASS_IMPL).unwrap();
    env.add_template("struct_impl.cs", STRUCT_IMPL).unwrap();
    env.add_template("union_impl.cs", UNION_IMPL).unwrap();
    env.add_template("flags_impl.cs", FLAGS_IMPL).unwrap();
    env
}
