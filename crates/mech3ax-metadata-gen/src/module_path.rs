use heck::ToUpperCamelCase as _;

const RUST_MOD_ROOT: &str = "mech3ax_api_types::";
const CSHARP_MOD_ROOT: &str = "Mech3DotNet.Json";

pub fn convert_mod_path(module_path: &'static str) -> String {
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
