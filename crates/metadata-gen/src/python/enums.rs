use super::module_path::{py_camel_case, rust_mod_path_to_path, rust_mod_path_to_py};
use super::python_type::{PythonType, SerializeType};
use super::resolver::TypeResolver;
use mech3ax_metadata_types::TypeInfoEnum;
use minijinja::{context, Environment};
use serde::Serialize;
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Variant {
    /// The enum variant's name.
    pub(crate) name: &'static str,
    /// The enum variant's index.
    pub(crate) index: u32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Enum {
    /// The enum's Python type name.
    pub(crate) name: &'static str,
    /// The enum's Python namespace.
    pub(crate) namespace: String,
    /// The enum's Python variant names.
    pub(crate) variants: Vec<Variant>,
    /// The enum's path on the filesystem.
    pub(crate) path: PathBuf,
}

impl Enum {
    pub(crate) fn make_type(&self) -> PythonType {
        PythonType {
            name: Cow::Borrowed(self.name),
            import: Some(format!("from {} import {}", self.namespace, self.name)),
            nullable: false,
            serde: SerializeType::Enum(self.name),
        }
    }

    pub(crate) fn new(resolver: &mut TypeResolver, ei: &TypeInfoEnum) -> Self {
        // luckily, Rust's casing for enum and variant names matches C#.
        let name = py_camel_case(ei.name);
        let (namespace, filename) = rust_mod_path_to_py(ei.module_path, ei.name);

        let variants = ei
            .variants
            .iter()
            .copied()
            .map(|(name, index)| Variant {
                name: py_camel_case(name),
                index,
            })
            .collect();

        let mut path = rust_mod_path_to_path(ei.module_path);
        resolver.add_directory(&path);
        path.push(filename);

        Self {
            name,
            namespace,
            variants,
            path,
        }
    }

    pub(crate) fn render_impl(&self, env: &Environment<'_>) -> Result<String, minijinja::Error> {
        let template = env.get_template("enum_impl.py")?;
        template.render(context! { enum => self })
    }
}

pub(crate) const ENUM_IMPL: &str = r#"from __future__ import annotations

from enum import IntEnum, auto, unique
from mech3py.exchange.deserializer import Deserializer
from mech3py.exchange.serializer import Serializer

_STRUCT_NAME = "{{ enum.name }}"

@unique
class {{ enum.name }}(IntEnum):
{%- for variant in enum.variants %}
    {{ variant.name }} = auto()
{%- endfor %}

    def serialize(self, s: Serializer) -> None:
        match self:
{%- for variant in enum.variants %}
            case {{ enum.name }}.{{ variant.name }}:
                variant_index = {{ variant.index }}
{%- endfor %}
            case _:
                raise ValueError()
        s.serialize_unit_variant(variant_index)

    @classmethod
    def deserialize(cls, d: Deserializer) -> {{ enum.name }}:
        variant_index = d.deserialize_unit_variant(_STRUCT_NAME)
        match variant_index:
{%- for variant in enum.variants %}
            case {{ variant.index }}:
                return {{ enum.name }}.{{ variant.name }}
{%- endfor %}
            case _:
                raise ValueError(f"unknown variant {variant_index} for {_STRUCT_NAME!r}")


"#;
