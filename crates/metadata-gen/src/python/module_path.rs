use heck::{ToSnakeCase as _, ToUpperCamelCase as _};
use std::path::PathBuf;

pub(crate) fn path_mod_root() -> PathBuf {
    PathBuf::from("output/mech3py/autogen/")
}

pub(crate) fn path_mod_types() -> PathBuf {
    PathBuf::from("output/mech3py/autogen/types/")
}

fn is_reserved_camel_case(name: &str) -> Option<&str> {
    match name {
        "None" => Some("None_"),
        _ => None,
    }
}

pub(crate) fn py_camel_case(name: &str) -> &str {
    is_reserved_camel_case(name).unwrap_or(name)
}

pub(crate) fn py_to_camel_case(name: &str) -> String {
    let name = name.to_upper_camel_case();
    match is_reserved_camel_case(&name) {
        Some(remap) => remap.to_string(),
        None => name,
    }
}

fn is_reserved_snake_case(name: &str) -> Option<&str> {
    match name {
        "if" => Some("if_"),
        "else" => Some("else_"),
        "global" => Some("global_"),
        "from" => Some("from_"),
        _ => None,
    }
}

pub(crate) fn py_snake_case(name: &str) -> &str {
    is_reserved_snake_case(name).unwrap_or(name)
}

pub(crate) fn py_to_snake_case(name: &str) -> String {
    let name = name.to_snake_case();
    match is_reserved_snake_case(&name) {
        Some(remap) => remap.to_string(),
        None => name,
    }
}

pub(crate) fn rust_mod_path_to_py(module_path: &'static str, name: &str) -> (String, String) {
    let mut components = module_path.split("::");
    assert_eq!(components.next(), Some("mech3ax_api_types"));

    let mut path = "mech3py.types".to_string();
    for component in components {
        path.push('.');
        path.push_str(py_snake_case(component));
    }
    path.push('.');
    let name = py_to_snake_case(name);
    path.push_str(&name);
    (path, format!("{}.py", name))
}

pub(crate) fn rust_mod_path_to_path(module_path: &'static str) -> PathBuf {
    let mut components = module_path.split("::");
    assert_eq!(components.next(), Some("mech3ax_api_types"));

    let mut path = path_mod_types();
    for component in components {
        path.push(py_snake_case(component));
    }
    path
}
