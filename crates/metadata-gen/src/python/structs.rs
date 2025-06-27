use super::fields::Field;
use super::module_path::{rust_mod_path_to_path, rust_mod_path_to_py};
use super::resolver::TypeResolver;
use crate::python::python_type::{PythonType, SerializeType};
use mech3ax_metadata_types::{TypeInfoStruct, TypeSemantic};
use minijinja::{context, Environment};
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Struct {
    /// The struct's Python name.
    pub(crate) name: &'static str,
    /// The struct's Python namespace.
    pub(crate) namespace: String,
    /// Whether the struct should use slots.
    pub(crate) slots: bool,
    /// The struct's field types imports.
    pub(crate) imports: Vec<String>,
    /// The struct's fields.
    pub(crate) fields: Vec<Field>,
    /// The structs's path on the filesystem.
    pub(crate) path: PathBuf,
}

impl Struct {
    pub(crate) fn make_type(&self) -> PythonType {
        PythonType {
            name: Cow::Borrowed(self.name),
            import: Some(format!("from {} import {}", self.namespace, self.name)),
            nullable: false,
            serde: SerializeType::Class(self.name),
        }
    }

    pub(crate) fn new(resolver: &mut TypeResolver, si: &TypeInfoStruct) -> Self {
        // luckily, Rust's casing for structs matches Python.
        let name = si.name;
        let (namespace, filename) = rust_mod_path_to_py(si.module_path, name);
        let slots = matches!(si.dotnet.semantic, TypeSemantic::Val);

        let fields: Vec<_> = si
            .fields
            .iter()
            .map(|field_info| Field::new(si.name, field_info, resolver))
            .collect();

        let mut imports = fields
            .iter()
            .filter_map(|field| field.import.as_ref())
            .collect::<HashSet<&String>>()
            .into_iter()
            .cloned()
            .collect::<Vec<String>>();
        imports.sort();

        let mut path = rust_mod_path_to_path(si.module_path);
        resolver.add_directory(&path);
        path.push(filename);

        Self {
            name,
            namespace,
            slots,
            imports,
            fields,
            path,
        }
    }

    pub(crate) fn render_impl(&self, env: &Environment<'_>) -> Result<String, minijinja::Error> {
        let template = env.get_template("struct_impl.py")?;
        template.render(context! { struct => self })
    }
}

pub(crate) const STRUCT_IMPL: &str = r#"from __future__ import annotations

from dataclasses import dataclass
from mech3py.exchange.deserializer import Deserializer
{% if struct.fields %}from mech3py.exchange.field import Field{% endif %}
from mech3py.exchange.serializer import Serializer

{% for import in struct.imports %}
{{ import }}
{% endfor %}

_STRUCT_NAME: str = "{{ struct.name }}"

@dataclass{% if struct.slots %}(slots=True){% endif %}
class {{ struct.name }}:
{%- if struct.fields %}{% for field in struct.fields %}
    {{ field.name }}: {{ field.ty }}
{%- endfor %}{% else %}
    pass
{%- endif %}

    def serialize(self, s: Serializer) -> None:
        s.serialize_struct({{ struct.fields | length }})
{%- for field in struct.fields %}
        s.serialize_field_name("{{ field.key }}")
        {{ field.serde.serialize }}(self.{{ field.name }})
{%- endfor %}

    @classmethod
    def deserialize(cls, d: Deserializer) -> {{ struct.name }}:
{%- if struct.fields %}
        @dataclass
        class Fields:
{%- for field in struct.fields %}
            {{ field.name }}: Field[{{ field.ty }}]
{%- endfor %}

        fields = Fields(
{%- for field in struct.fields %}
            {{ field.name }}=Field[{{ field.ty }}]{% if field.default %}.some({{ field.default }}){% else %}.none(){% endif %},
{%- endfor %}
        )
{%- endif %}
        for field_name in d.deserialize_struct():
            match field_name:
{%- for field in struct.fields %}
                case "{{ field.key }}":
                    fields.{{ field.name }}.set_value({{ field.serde.deserialize }}())
{%- endfor %}
                case _:
                    raise ValueError(f"unknown field {field_name!r} for {_STRUCT_NAME!r}")

        return cls(
{%- for field in struct.fields %}
            {{ field.name }}=fields.{{ field.name }}.unwrap(_STRUCT_NAME, "{{ field.name }}"),
{%- endfor %}
        )

"#;
