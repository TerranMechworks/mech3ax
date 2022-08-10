use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Enum {
    pub name: &'static str,
    pub variants: &'static [&'static str],
}

impl Enum {
    pub fn new<E>() -> Self
    where
        E: mech3ax_metadata_types::Enum,
    {
        Self {
            name: E::NAME,
            variants: E::VARIANTS,
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

pub const ENUM_IMPL: &'static str = r###"using System;
using System.Text.Json;
using System.Text.Json.Serialization;
using Mech3DotNet.Json.Converters;

namespace Mech3DotNet.Json
{
    [JsonConverter(typeof({{ enum.name }}Converter))]
    public enum {{ enum.name }}
    {
{%- for variant in enum.variants %}
        {{ variant }},
{%- endfor %}
    }
}
"###;

pub const ENUM_CONV: &'static str = r###"using System;
using System.Text.Json;
using System.Text.Json.Serialization;
using Mech3DotNet.Json;

namespace Mech3DotNet.Json.Converters
{
    public class {{ enum.name }}Converter : EnumConverter<{{ enum.name }}>
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
            _ => throw new ArgumentOutOfRangeException("{{ enum.name }}"),
        };
    }
}
"###;
