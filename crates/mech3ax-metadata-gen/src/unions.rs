use crate::resolver::TypeResolver;
use mech3ax_metadata_types::TypeSemantic;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Union {
    pub name: &'static str,
    pub choice: String,
    pub variants: Vec<(&'static str, Option<&'static str>, bool)>,
}

impl Union {
    pub fn new<S>(resolver: &TypeResolver) -> Self
    where
        S: mech3ax_metadata_types::Union,
    {
        let name = S::NAME;
        let choice = format!("{}Variant", name);
        let variants = S::VARIANTS
            .into_iter()
            .map(|(variant_name, struct_name)| {
                let mut varant_null_check = true;
                if let Some(name) = struct_name {
                    match resolver.lookup_struct(name) {
                        None => eprintln!("WARNING: structure {} not found", name),
                        Some(s) => match s.semantic {
                            TypeSemantic::Ref => varant_null_check = true,
                            TypeSemantic::Val => varant_null_check = false,
                        },
                    }
                }
                (*variant_name, struct_name.clone(), varant_null_check)
            })
            .collect();

        Self {
            name,
            choice,
            variants,
        }
    }

    pub fn render_impl(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("union", self);
        tera.render("union_impl.cs", &context)
    }

    pub fn render_conv(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("union", self);
        tera.render("union_conv.cs", &context)
    }
}

pub const UNION_IMPL: &'static str = r###"using System;
using System.Text.Json;
using System.Text.Json.Serialization;
using Mech3DotNet.Json.Converters;

namespace Mech3DotNet.Json
{
    public enum {{ union.choice }}
    {
{%- for variant in union.variants %}
        {{ variant.0 }},
{%- endfor %}
    }

    [JsonConverter(typeof({{ union.name }}Converter))]
    public class {{ union.name }} : IDiscriminatedUnion<{{ union.choice }}>
    {
{%- for variant in union.variants %}{% if not variant.1 %}
        public sealed class {{ variant.0 }}
        {
            public {{ variant.0 }}() { }
        }
{% endif %}{% endfor %}
{%- for variant in union.variants %}
        public {{ union.name }}({% if variant.1 %}{{ variant.1 }}{% else %}{{ variant.0 }}{% endif %} value)
        {
            this.value = value;
            Variant = {{ union.choice }}.{{ variant.0 }};
        }
{% endfor %}
        protected object value;
        public {{ union.choice }} Variant { get; protected set; }
        public bool Is<T>() { return typeof(T).IsInstanceOfType(value); }
        public T As<T>() { return (T)value; }
        public object GetValue() { return value; }
    }
}
"###;

pub const UNION_CONV: &'static str = r###"using System;
using System.Text.Json;
using System.Text.Json.Serialization;
using Mech3DotNet.Json;

namespace Mech3DotNet.Json.Converters
{
    public class {{ union.name }}Converter : UnionConverter<{{ union.name }}>
    {
        public override {{ union.name }} ReadUnitVariant(string? name)
        {
            switch (name)
            {
{%- for variant in union.variants %}{%- if not variant.1 %}
                case "{{ variant.0 }}":
                    {
                        var inner = new {{ union.name }}.{{ variant.0 }}();
                        return new {{ union.name }}(inner);
                    }
{%- endif %}{% endfor %}
{%- for variant in union.variants %}{%- if variant.1 %}
                case "{{ variant.0 }}":
                    {
                        System.Diagnostics.Debug.WriteLine("Invalid unit variant '{{ variant.0 }}' for '{{ union.name }}'");
                        throw new JsonException();
                    }
{%- endif %}{% endfor %}
                case null:
                    {
                        System.Diagnostics.Debug.WriteLine("Variant cannot be null for '{{ union.name }}'");
                        throw new JsonException();
                    }
                default:
                    {
                        System.Diagnostics.Debug.WriteLine($"Invalid variant '{name}' for '{{ union.name }}'");
                        throw new JsonException();
                    }
            }
        }

        public override {{ union.name }} ReadStructVariant(ref Utf8JsonReader reader, string? name, JsonSerializerOptions options)
        {
            switch (name)
            {
{%- for variant in union.variants %}{%- if variant.1 %}
                case "{{ variant.0 }}":
                    {
                        var inner = JsonSerializer.Deserialize<{{ variant.1 }}>(ref reader, options);
{%- if variant.2 %}
                        if (inner is null)
                        {
                            System.Diagnostics.Debug.WriteLine("Value of '{{ variant.0 }}' was null for '{{ union.name }}'");
                            throw new JsonException();
                        }
{%- endif %}
                        return new {{ union.name }}(inner);
                    }
{%- endif %}{% endfor %}
{%- for variant in union.variants %}{%- if not variant.1 %}
                case "{{ variant.0 }}":
                    {
                        System.Diagnostics.Debug.WriteLine("Invalid struct variant '{{ variant.0 }}' for '{{ union.name }}'");
                        throw new JsonException();
                    }
{%- endif %}{% endfor %}
                case null:
                    {
                        System.Diagnostics.Debug.WriteLine("Variant cannot be null for '{{ union.name }}'");
                        throw new JsonException();
                    }
                default:
                    {
                        System.Diagnostics.Debug.WriteLine($"Invalid variant '{name}' for '{{ union.name }}'");
                        throw new JsonException();
                    }
            }
        }

        public override void Write(Utf8JsonWriter writer, {{ union.name }} value, JsonSerializerOptions options)
        {
            switch (value.Variant)
            {
{%- for variant in union.variants %}
                case {{ union.choice }}.{{ variant.0 }}:
{%- if variant.1 %}
                    {
                        var inner = value.As<{{ variant.1 }}>();
                        writer.WriteStartObject();
                        writer.WritePropertyName("{{ variant.0 }}");
                        JsonSerializer.Serialize(writer, inner, options);
                        writer.WriteEndObject();
                        break;
                    }
{%- else %}
                    {
                        writer.WriteStringValue("{{ variant.0 }}");
                        break;
                    }
{%- endif %}
{%- endfor %}
                default:
                    throw new ArgumentOutOfRangeException("Variant", $"Invalid variant '{value.Variant}' for '{{ union.name }}'");
            }
        }
    }
}
"###;
