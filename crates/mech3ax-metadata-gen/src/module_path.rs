use heck::ToUpperCamelCase as _;
use std::path::PathBuf;

const RUST_MOD_ROOT: &str = "mech3ax_api_types::";
const CSHARP_MOD_ROOT: &str = "Mech3DotNet.Types";
const PATH_MOD_ROOT: &str = "output/Mech3DotNet/Types/";

pub fn path_mod_root() -> PathBuf {
    PathBuf::from(PATH_MOD_ROOT)
}
pub fn rust_mod_path_to_dotnet(module_path: &'static str) -> String {
    let components = module_path
        .strip_prefix(RUST_MOD_ROOT)
        .expect("wrong crate")
        .split("::");

    let mut path = CSHARP_MOD_ROOT.to_string();
    for component in components {
        path.push('.');
        path.push_str(&component.to_upper_camel_case());
    }
    path
}

pub fn rust_mod_path_to_path(module_path: &'static str) -> PathBuf {
    let components = module_path
        .strip_prefix(RUST_MOD_ROOT)
        .expect("wrong crate")
        .split("::");

    let mut path = path_mod_root();
    for component in components {
        path.push(component.to_upper_camel_case());
    }
    path
}
