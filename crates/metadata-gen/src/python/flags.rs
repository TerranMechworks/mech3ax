use super::module_path::{rust_mod_path_to_path, rust_mod_path_to_py};
use super::python_type::{PythonType, SerializeType};
use super::resolver::TypeResolver;
use heck::AsSnakeCase;
use heck::ToUpperCamelCase as _;
use mech3ax_metadata_types::{TypeInfoFlags, TypeInfoFlagsRepr};
use minijinja::{context, Environment};
use serde::Serialize;
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Variant {
    /// The flags variant's name.
    pub(crate) name: String,
    /// The flags variant's index.
    pub(crate) index: u32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Flags {
    /// The flags' Python type name.
    pub(crate) name: &'static str,
    /// The flags' Python namespace.
    pub(crate) namespace: String,
    /// The flags' Python (de)serialization type.
    pub(crate) serde_type: &'static str,
    /// The flags' Python variant names.
    pub(crate) variants: Vec<Variant>,
    /// The flags' path on the filesystem.
    pub(crate) path: PathBuf,
}

impl Flags {
    pub(crate) fn make_type(&self) -> PythonType {
        PythonType {
            name: Cow::Borrowed(self.name),
            import: Some(format!("from {} import {}", self.namespace, self.name)),
            nullable: false,
            serde: SerializeType::Flags(self.name),
        }
    }

    pub(crate) fn new(resolver: &mut TypeResolver, fi: &TypeInfoFlags) -> Self {
        // luckily, Rust's casing for structs matches Python.
        let name = fi.name;
        let namespace = rust_mod_path_to_py(fi.module_path, name);

        let serde_type = match fi.repr {
            TypeInfoFlagsRepr::U8 => "u8",
            TypeInfoFlagsRepr::U16 => "u16",
            TypeInfoFlagsRepr::U32 => "u32",
        };

        let variants = fi
            .variants
            .iter()
            .copied()
            .map(|(name, index)| Variant {
                name: name.to_upper_camel_case(),
                index,
            })
            .collect();

        let mut path = rust_mod_path_to_path(fi.module_path);
        resolver.add_directory(&path);
        path.push(format!("{}.py", AsSnakeCase(name)));

        Self {
            name,
            namespace,
            serde_type,
            variants,
            path,
        }
    }

    pub(crate) fn render_impl(&self, env: &Environment<'_>) -> Result<String, minijinja::Error> {
        let template = env.get_template("flags_impl.py")?;
        template.render(context! { flags => self })
    }
}

pub(crate) const FLAGS_IMPL: &str = r#"from __future__ import annotations

from enum import IntFlag, unique
from mech3py.exchange.deserializer import Deserializer
from mech3py.exchange.serializer import Serializer

_VALID: int = 0{% for variant in flags.variants %} | 1 << {{ variant.index }}{% endfor %}

@unique
class {{ flags.name }}(IntFlag):
{%- for variant in flags.variants %}
    {{ variant.name }} = 1 << {{ variant.index }}
{%- endfor %}

    def serialize(self, s: Serializer) -> None:
        v = int(self)
        if (v & ~_VALID) != 0:
            raise ValueError()
        s.serialize_{{ flags.serde_type }}(v)

    @classmethod
    def deserialize(cls, d: Deserializer) -> {{ flags.name }}:
        v = d.deserialize_{{ flags.serde_type }}()
        return cls(v)


"#;
