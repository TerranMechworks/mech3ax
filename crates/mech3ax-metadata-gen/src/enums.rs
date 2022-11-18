use crate::csharp_type::{CSharpType, TypeKind};
use crate::resolver::TypeResolver;
use mech3ax_metadata_types::TypeInfoEnum;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Clone, Serialize)]
pub struct Enum {
    /// The enum's C# enum name.
    pub name: &'static str,
    /// The enum's C# variant names.
    pub variants: &'static [&'static str],
}

impl Enum {
    pub fn make_type(&self) -> CSharpType {
        // our "enums" are a C# enum, which are a C# `struct` (value type)
        CSharpType {
            name: Cow::Borrowed(self.name),
            kind: TypeKind::Val,
            generics: None,
        }
    }

    pub fn new(_resolver: &mut TypeResolver, ei: &TypeInfoEnum) -> Self {
        // luckily, Rust's casing for enum and variant names matches C#.
        Self {
            name: ei.name,
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

pub const ENUM_IMPL: &'static str = r###"namespace Mech3DotNet.Json
{
    [System.Text.Json.Serialization.JsonConverter(typeof(Mech3DotNet.Json.Converters.{{ enum.name }}Converter))]
    public enum {{ enum.name }}
    {
{%- for variant in enum.variants %}
        {{ variant }},
{%- endfor %}
    }
}
"###;

pub const ENUM_CONV: &'static str = r###"namespace Mech3DotNet.Json.Converters
{
    public class {{ enum.name }}Converter : Mech3DotNet.Json.Converters.EnumConverter<{{ enum.name }}>
    {
        public override {{ enum.name }} ReadVariant(string? name) => name switch
        {
{%- for variant in enum.variants %}
            "{{ variant }}" => {{ enum.name }}.{{ variant }},
{%- endfor %}
            null => DebugAndThrow("Variant cannot be null for '{{ enum.name }}'"),
            _ => DebugAndThrow($"Invalid variant '{name}' for '{{ enum.name }}'"),
        };

        public override string WriteVariant({{ enum.name }} value) => value switch
        {
{%- for variant in enum.variants %}
            {{ enum.name }}.{{ variant }} => "{{ variant }}",
{%- endfor %}
            _ => throw new System.ArgumentOutOfRangeException("{{ enum.name }}"),
        };
    }
}
"###;
