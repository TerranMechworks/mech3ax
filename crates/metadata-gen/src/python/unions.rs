use super::module_path::{
    py_camel_case, py_to_snake_case, rust_mod_path_to_path, rust_mod_path_to_py,
};
use super::python_type::{PythonType, SerializeType};
use super::resolver::TypeResolver;
use mech3ax_metadata_types::{TypeInfo, TypeInfoUnion};
use minijinja::{context, Environment};
use serde::Serialize;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VariantSerde {
    pub(crate) serialize: String,
    pub(crate) deserialize: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VariantWrap {
    pub(crate) internal: String,
    pub(crate) external: String,
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
    /// The union variant's Python type exposed externally.
    pub(crate) out_type: String,
    /// The union variant's Python type used internally.
    pub(crate) in_type: String,
    /// The struct field's serialization information, if it isn't a unit variant.
    pub(crate) serde: Option<VariantSerde>,
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
    /// Horrible hack to de-duplicate types if needed.
    pub(crate) wraps: Vec<VariantWrap>,
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
    let fn_name = py_to_snake_case(variant_name);
    let name = py_camel_case(variant_name);
    let mut import = None;
    let mut serde = None;
    let out_type = match variant_type {
        None => {
            // unit variant
            format!("{}.{}", union_name, name)
        }
        Some(type_info) => {
            // new type variant
            let ty = resolver.resolve(type_info, union_name);
            import = ty.import.clone();
            serde = Some(VariantSerde {
                serialize: ty.serde.make_serialize(),
                deserialize: ty.serde.make_deserialize(),
            });
            ty.name.to_string()
        }
    };
    let in_type = out_type.clone();
    Variant {
        name,
        fn_name,
        index: variant_index,
        import,
        out_type,
        in_type,
        serde,
    }
}

fn variant_types_unique(variants: &[Variant]) -> bool {
    let mut variant_types = HashSet::new();
    for variant in variants {
        if !variant_types.insert(variant.out_type.as_str()) {
            return false;
        }
    }
    true
}

fn variant_types_wrap(variants: &mut [Variant]) -> Vec<VariantWrap> {
    let mut dupes: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, variant) in variants.iter().enumerate() {
        dupes
            .entry(variant.out_type.clone())
            .or_default()
            .push(index);
    }
    dupes.retain(|_k, v| v.len() > 1);

    let mut wraps = Vec::new();
    for (k, indices) in dupes {
        for i in indices {
            let variant = &mut variants[i];
            let in_type = format!("_{}", variant.name);
            wraps.push(VariantWrap {
                internal: in_type.clone(),
                external: k.clone(),
            });
            variant.in_type = in_type;
        }
    }
    wraps
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
        let name = py_camel_case(ui.name);
        let (namespace, filename) = rust_mod_path_to_py(ui.module_path, ui.name);

        let mut variants = ui
            .variants
            .iter()
            .copied()
            .zip(0u32..)
            .map(|((variant_name, variant_type), variant_index)| {
                resolve_variant(name, resolver, variant_name, variant_index, variant_type)
            })
            .collect::<Vec<Variant>>();

        let wraps = if !variant_types_unique(&variants) {
            // println!("non-unique variant types for {}", name);
            variant_types_wrap(&mut variants)
        } else {
            Vec::new()
        };

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
        path.push(filename);

        Self {
            name,
            namespace,
            imports,
            variants,
            wraps,
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

{% for import in union.imports -%}
{{ import }}
{% endfor %}
{% for wrap in union.wraps -%}
class {{ wrap.internal }}:
    __slots__ = ["inner"]
    def __init__(self, value: {{ wrap.external }}):
        self.inner: {{ wrap.external }} = value
{% endfor %}

type _{{ union.name }}Value = {% for variant in union.variants %}{{ variant.in_type }}{% if not loop.last %} | {% endif %}{% endfor %}

_STRUCT_NAME = "{{ union.name }}"

class {{ union.name }}:
{%- for variant in union.variants %}{% if not variant.serde %}
    class {{ variant.name }}:
        pass
{%- endif %}{% endfor %}

    @unique
    class Variant(IntEnum):
{%- for variant in union.variants %}
        {{ variant.name }} = auto()
{%- endfor %}

    def __init__(self, value: _{{ union.name }}Value):
        self._value: _{{ union.name }}Value = value

    def __repr__(self) -> str:
        match self._value:
{%- for variant in union.variants %}
{%- if variant.serde %}
            case {{ variant.in_type }}():
                return f"{{ union.name }}.{{ variant.name }}({self._value!r})"
{%- else %}
            case {{ variant.in_type }}():
                return "{{ union.name }}.{{ variant.name }}()"
{%- endif %}
{%- endfor %}

{% for variant in union.variants %}
{%- if variant.serde %}
    @classmethod
    def {{ variant.fn_name }}(cls, value: {{ variant.out_type }}) -> {{ union.name }}:
        return cls(value={% if variant.out_type != variant.in_type %}{{ variant.in_type }}(value){% else %}value{% endif %})
{%- else %}
    @classmethod
    def {{ variant.fn_name }}(cls) -> {{ union.name }}:
        return cls(value={{ variant.in_type }}())
{%- endif %}
{% endfor %}

    @property
    def value(self) -> _{{ union.name }}Value:
        return self._value

    @property
    def variant(self) -> {{ union.name }}.Variant:
        match self._value:
{%- for variant in union.variants %}
            case {{ variant.in_type }}():
                return {{ union.name }}.Variant.{{ variant.name }}
{%- endfor %}

{% for variant in union.variants %}
{%- if variant.serde %}
    @property
    def is_{{ variant.fn_name }}(self) -> bool:
        return isinstance(self._value, {{ variant.in_type }})
{%- else %}
    @property
    def is_{{ variant.fn_name }}(self) -> bool:
        return isinstance(self._value, {{ variant.in_type }})
{%- endif %}
{% endfor %}

{% for variant in union.variants %}
{%- if variant.serde %}
    def as_{{ variant.fn_name }}(self) -> {{ variant.out_type }}:
        if not isinstance(self._value, {{ variant.in_type }}):
            raise ValueError()
        return self._value{% if variant.out_type != variant.in_type %}.inner{% endif %}
{%- endif %}
{% endfor %}

    def serialize(self, s: Serializer) -> None:
        match self._value:
{%- for variant in union.variants %}
{%- if variant.serde %}
            case {{ variant.in_type }}():
                s.serialize_new_type_variant({{ variant.index }})
                {{ variant.serde.serialize }}(self._value{% if variant.out_type != variant.in_type %}.inner{% endif %})
{%- else %}
            case {{ variant.in_type }}():
                s.serialize_unit_variant({{ variant.index }})
{%- endif %}
{%- endfor %}

    @classmethod
    def deserialize(cls, d: Deserializer) -> {{ union.name }}:
        enum_type, variant_index = d.deserialize_enum()
        match variant_index:
{%- for variant in union.variants %}
{%- if variant.serde %}
            case {{ variant.index }}:
                if enum_type != EnumType.NewType:
                    raise ValueError(f"expected variant {EnumType.NewType!r}, but found {enum_type!r} for {_STRUCT_NAME!r}")
                value = {{ variant.serde.deserialize }}()
                return cls.{{ variant.fn_name }}(value)
{%- else %}
            case {{ variant.index }}:
                if enum_type != EnumType.Unit:
                    raise ValueError(f"expected variant {EnumType.Unit!r}, but found {enum_type!r} for {_STRUCT_NAME!r}")
                return cls.{{ variant.fn_name }}()
{%- endif %}
{%- endfor %}
            case _:
                raise ValueError(f"unknown variant {variant_index} for {_STRUCT_NAME!r}")


"#;
