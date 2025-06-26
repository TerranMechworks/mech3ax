use super::enums::ENUM_IMPL;
use super::flags::FLAGS_IMPL;
use super::structs::STRUCT_IMPL;
use super::unions::UNION_IMPL;
use minijinja::Environment;

pub(crate) fn make_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_template("enum_impl.py", ENUM_IMPL).unwrap();
    env.add_template("flags_impl.py", FLAGS_IMPL).unwrap();
    env.add_template("struct_impl.py", STRUCT_IMPL).unwrap();
    env.add_template("union_impl.py", UNION_IMPL).unwrap();
    env
}
