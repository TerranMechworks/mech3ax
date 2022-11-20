use crate::csharp_type::{CSharpType, TypeKind};
use crate::module_path::convert_mod_path;
use crate::resolver::TypeResolver;
use mech3ax_metadata_types::TypeInfoEnum;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Clone, Serialize)]
pub struct Enum {
    /// The enum's C# type name.
    pub name: &'static str,
    /// The enum's C# namespace.
    pub namespace: String,
    /// The enum's full C# type, with namespace.
    pub full_name: String,
    /// The enum's C# variant names.
    pub variants: &'static [&'static str],
}

impl Enum {
    pub fn make_type(&self) -> CSharpType {
        // our "enums" are a C# enum, which are a C# `struct` (value type)
        CSharpType {
            name: Cow::Owned(self.full_name.clone()),
            kind: TypeKind::Val,
            generics: None,
        }
    }

    pub fn new(_resolver: &mut TypeResolver, ei: &TypeInfoEnum) -> Self {
        // luckily, Rust's casing for enum and variant names matches C#.
        let name = ei.name;
        let namespace = convert_mod_path(ei.module_path);
        let full_name = format!("{}.{}", namespace, ei.name);
        Self {
            name,
            namespace,
            full_name,
            variants: ei.variants,
        }
    }

    pub fn render_impl(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("enum", self);
        tera.render("enum_impl.cs", &context)
    }

    pub fn render_conv(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("enum", self);
        tera.render("enum_conv.cs", &context)
    }
}

pub const ENUM_IMPL: &str = r###"namespace {{ enum.namespace }}
{
    [System.Text.Json.Serialization.JsonConverter(typeof({{ enum.namespace }}.Converters.{{ enum.name }}Converter))]
    public enum {{ enum.name }}
    {
{%- for variant in enum.variants %}
        {{ variant }},
{%- endfor %}
    }
}
"###;

pub const ENUM_CONV: &str = r###"namespace {{ enum.namespace }}.Converters
{
    public class {{ enum.name }}Converter : Mech3DotNet.Json.Converters.EnumConverter<{{ enum.full_name }}>
    {
        public override {{ enum.full_name }} ReadVariant(string? name) => name switch
        {
{%- for variant in enum.variants %}
            "{{ variant }}" => {{ enum.full_name }}.{{ variant }},
{%- endfor %}
            null => DebugAndThrow("Variant cannot be null for '{{ enum.name }}'"),
            _ => DebugAndThrow($"Invalid variant '{name}' for '{{ enum.name }}'"),
        };

        public override string WriteVariant({{ enum.full_name }} value) => value switch
        {
{%- for variant in enum.variants %}
            {{ enum.full_name }}.{{ variant }} => "{{ variant }}",
{%- endfor %}
            _ => throw new System.ArgumentOutOfRangeException("{{ enum.name }}"),
        };
    }
}
"###;
