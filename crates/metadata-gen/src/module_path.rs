use heck::ToUpperCamelCase as _;
use std::path::PathBuf;

pub fn path_mod_root() -> PathBuf {
    PathBuf::from("output/Mech3DotNet/AutoGen/")
}

pub fn rust_mod_path_to_dotnet(module_path: &'static str) -> String {
    let mut components = module_path.split("::");
    assert_eq!(components.next(), Some("mech3ax_api_types"));

    let mut path = "Mech3DotNet.Types".to_string();
    for component in components {
        path.push('.');
        path.push_str(&component.to_upper_camel_case());
    }
    path
}

pub fn dotnet_namespace_to_path(namespace: &str) -> PathBuf {
    let mut components = namespace.split('.');
    assert_eq!(components.next(), Some("Mech3DotNet"));

    let mut path = path_mod_root();
    for component in components {
        path.push(component.to_upper_camel_case());
    }
    path
}
