use heck::ToSnakeCase as _;
use std::path::PathBuf;

pub(crate) fn path_mod_root() -> PathBuf {
    PathBuf::from("output/mech3py/autogen/")
}

pub(crate) fn path_mod_types() -> PathBuf {
    PathBuf::from("output/mech3py/autogen/types/")
}

pub(crate) fn rust_mod_path_to_py(module_path: &'static str, name: &str) -> String {
    let mut components = module_path.split("::");
    assert_eq!(components.next(), Some("mech3ax_api_types"));

    let mut path = "mech3py.types".to_string();
    for component in components {
        path.push('.');
        path.push_str(component);
    }
    path.push('.');
    let name = name.to_snake_case();
    path.push_str(&name);
    path
}

pub(crate) fn rust_mod_path_to_path(module_path: &'static str) -> PathBuf {
    let mut components = module_path.split("::");
    assert_eq!(components.next(), Some("mech3ax_api_types"));

    let mut path = path_mod_types();
    for component in components {
        path.push(component);
    }
    path
}
