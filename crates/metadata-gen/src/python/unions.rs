use super::module_path::{rust_mod_path_to_path, rust_mod_path_to_py};
use super::python_type::{PythonType, SerializeType};
use super::resolver::TypeResolver;
use heck::AsSnakeCase;
use heck::ToSnakeCase as _;
use mech3ax_metadata_types::{TypeInfo, TypeInfoUnion};
use minijinja::{context, Environment};
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VarantSerde {
    pub(crate) serialize: String,
    pub(crate) deserialize: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Variant {
    /// The union variant's name.
    pub(crate) name: &'static str,
    /// The union variant's name for functions (snake_case).
    pub(crate) fn_name: String,
    /// The union variant's index.
    pub(crate) index: u32,
    /// The union variant's import definition, if it requires importing.
    pub(crate) import: Option<String>,
    /// The union variant's Python type.
    pub(crate) type_name: String,
    /// The struct field's serialization information, if it isn't a unit variant.
    pub(crate) serde: Option<VarantSerde>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Union {
    /// The union's Python type name.
    pub(crate) name: &'static str,
    /// The union's Python namespace.
    pub(crate) namespace: String,
    /// The struct's field types imports.
    pub(crate) imports: Vec<String>,
    /// The union variant types.
    pub(crate) variants: Vec<Variant>,
    /// The union's path on the filesystem.
    pub(crate) path: PathBuf,
}

fn resolve_variant(
    union_name: &'static str,
    resolver: &TypeResolver,
    variant_name: &'static str,
    variant_index: u32,
    variant_type: Option<&'static TypeInfo>,
) -> Variant {
    let fn_name = variant_name.to_snake_case();
    match variant_type {
        None => Variant {
            name: variant_name,
            fn_name,
            index: variant_index,
            import: None,
            type_name: variant_name.to_string(),
            serde: None,
        },
        Some(type_info) => {
            let ty = resolver.resolve(type_info, union_name);
            Variant {
                name: variant_name,
                fn_name,
                index: variant_index,
                import: ty.import.clone(),
                type_name: ty.name.to_string(),
                serde: Some(VarantSerde {
                    serialize: ty.serde.make_serialize(),
                    deserialize: ty.serde.make_deserialize(),
                }),
            }
        }
    }
}

impl Union {
    pub(crate) fn make_type(&self) -> PythonType {
        PythonType {
            name: Cow::Borrowed(self.name),
            import: Some(format!("from {} import {}", self.namespace, self.name)),
            nullable: false,
            serde: SerializeType::Union(self.name),
        }
    }

    pub(crate) fn new(resolver: &mut TypeResolver, ui: &TypeInfoUnion) -> Self {
        // luckily, Rust's casing for enum names matches Python.
        let name = ui.name;
        let namespace = rust_mod_path_to_py(ui.module_path, name);

        let variants = ui
            .variants
            .iter()
            .copied()
            .zip(0u32..)
            .map(|((variant_name, variant_type), variant_index)| {
                resolve_variant(name, resolver, variant_name, variant_index, variant_type)
            })
            .collect::<Vec<Variant>>();

        let mut imports = variants
            .iter()
            .filter_map(|variant| variant.import.as_ref())
            .collect::<HashSet<&String>>()
            .into_iter()
            .cloned()
            .collect::<Vec<String>>();
        imports.sort();

        let mut path = rust_mod_path_to_path(ui.module_path);
        resolver.add_directory(&path);
        path.push(format!("{}.py", AsSnakeCase(name)));

        Self {
            name,
            namespace,
            imports,
            variants,
            path,
        }
    }

    pub(crate) fn render_impl(&self, env: &Environment<'_>) -> Result<String, minijinja::Error> {
        let template = env.get_template("union_impl.py")?;
        template.render(context! { union => self })
    }
}

pub(crate) const UNION_IMPL: &str = r#"from __future__ import annotations

from enum import IntEnum, auto, unique
from mech3py.exchange.reader import EnumType
from mech3py.exchange.deserializer import Deserializer
from mech3py.exchange.serializer import Serializer

{% for import in union.imports %}
{{ import }}
{% endfor %}

_STRUCT_NAME = "{{ union.name }}"
type _{{ union.name }}Value = {% for variant in union.variants %}{{ variant.type_name }}{% if not loop.last %} | {% endif %}{% endfor %}

@unique
class {{ union.name }}Variant(IntEnum):
{%- for variant in union.variants %}
    {{ variant.name }} = auto()
{%- endfor %}

class {{ union.name }}:
    def __init__(self, value: _{{ union.name }}Value):
        self._value: _{{ union.name }}Value = value

{%- for variant in union.variants %}
    @classmethod
    def {{ variant.fn_name }}(cls, value: {{ variant.type_name }}) -> {{ union.name }}:
        return cls(value=value)
{%- endfor %}

    @property
    def value(self) -> _{{ union.name }}Value:
        return self._value

    @property
    def variant(self) -> {{ union.name }}Variant:
{%- for variant in union.variants %}
        if isinstance(self._value, {{ variant.type_name }}):
            return {{ union.name }}.{{ variant.name }}
{%- endfor %}
        raise AssertionError()

{%- for variant in union.variants %}
    @property
    def is_{{ variant.fn_name }}(self) -> bool:
        return isinstance(self._value, {{ variant.type_name }})
{%- endfor %}

{%- for variant in union.variants %}
    def as_{{ variant.fn_name }}(self) -> {{ variant.type_name }}:
        if not isinstance(self._value, {{ variant.type_name }}):
            raise ValueError()
        return self._value
{%- endfor %}

    def serialize(self, s: Serializer) -> None:
        match self._value:
{%- for variant in union.variants %}
            case {{ variant.type_name }}():
                s.serialize_new_type_variant({{ variant.index }})
                {{ variant.serde.serialize }}(self._value)
{%- endfor %}
            case _:
                raise AssertionError()

    @classmethod
    def deserialize(cls, d: Deserializer) -> Material:
        enum_type, variant_index = d.deserialize_enum()
        match variant_index:
{%- for variant in union.variants %}
            case {{ variant.index }}:
                if enum_type != EnumType.NewType:
                    raise ValueError(f"expected variant {EnumType.NewType}, but found {enum_type} for {_STRUCT_NAME!r}")
                value = {{ variant.serde.deserialize }}()
                return cls(value)
{%- endfor %}
            case _:
                raise ValueError(f"unknown variant {variant_index} for {_STRUCT_NAME!r}")


"#;
