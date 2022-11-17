use crate::csharp_type::{CSharpType, TypeKind};
use crate::resolver::TypeResolver;
use mech3ax_metadata_types::{TypeInfo, TypeInfoUnion};
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Clone, Serialize)]
pub struct Variant {
    /// The union variant's C# class and enum name.
    pub name: &'static str,
    /// The union variant's C# type, or `None` for unit variants.
    pub ty: Option<String>,
    /// Whether the type requires a null check on deserialization.
    pub null_check: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Union {
    /// The union's C# class name.
    pub name: &'static str,
    /// The union variant's C# enum name.
    pub choice: String,
    /// The union variant types.
    pub variants: Vec<Variant>,
}

fn resolve_variant(
    union_name: &'static str,
    resolver: &TypeResolver,
    variant_name: &'static str,
    variant_type: Option<&'static TypeInfo>,
) -> Variant {
    // Luckily, Rust's casing for variant names matches C#.
    match variant_type {
        None => Variant {
            name: variant_name,
            ty: None,
            null_check: false,
        },
        Some(type_info) => {
            let ty = resolver.resolve(type_info, union_name);
            Variant {
                name: variant_name,
                ty: Some(ty.name.to_string()),
                null_check: ty.null_check(),
            }
        }
    }
}

impl Union {
    pub fn make_type(&self) -> CSharpType {
        // our "unions" get transformed into C# classes (reference type)
        // disallow generics for now
        CSharpType {
            name: Cow::Borrowed(self.name),
            kind: TypeKind::Ref,
            generics: None,
        }
    }

    pub fn new(resolver: &TypeResolver, ui: &TypeInfoUnion) -> Self {
        // luckily, Rust's casing for enum names matches C# classes.
        let name = ui.name;
        let choice = format!("{}Variant", name);

        let variants = ui
            .variants
            .into_iter()
            .copied()
            .map(|(variant_name, variant_type)| {
                resolve_variant(name, resolver, variant_name, variant_type)
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
        {{ variant.name }},
{%- endfor %}
    }

    [JsonConverter(typeof({{ union.name }}Converter))]
    public class {{ union.name }} : IDiscriminatedUnion<{{ union.choice }}>
    {
{%- for variant in union.variants %}{% if not variant.ty %}
        public sealed class {{ variant.name }}
        {
            public {{ variant.name }}() { }
        }
{% endif %}{% endfor %}
{%- for variant in union.variants %}
        public {{ union.name }}({% if variant.ty %}{{ variant.ty }}{% else %}{{ variant.name }}{% endif %} value)
        {
            this.value = value;
            Variant = {{ union.choice }}.{{ variant.name }};
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
{%- for variant in union.variants %}{%- if not variant.ty %}
                case "{{ variant.name }}":
                    {
                        var inner = new {{ union.name }}.{{ variant.name }}();
                        return new {{ union.name }}(inner);
                    }
{%- endif %}{% endfor %}
{%- for variant in union.variants %}{%- if variant.ty %}
                case "{{ variant.name }}":
                    {
                        System.Diagnostics.Debug.WriteLine("Invalid unit variant '{{ variant.name }}' for '{{ union.name }}'");
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
{%- for variant in union.variants %}{%- if variant.ty %}
                case "{{ variant.name }}":
                    {
                        var inner = JsonSerializer.Deserialize<{{ variant.ty }}>(ref reader, options);
{%- if variant.null_check %}
                        if (inner is null)
                        {
                            System.Diagnostics.Debug.WriteLine("Value of '{{ variant.name }}' was null for '{{ union.name }}'");
                            throw new JsonException();
                        }
{%- endif %}
                        return new {{ union.name }}(inner);
                    }
{%- endif %}{% endfor %}
{%- for variant in union.variants %}{%- if not variant.ty %}
                case "{{ variant.name }}":
                    {
                        System.Diagnostics.Debug.WriteLine("Invalid struct variant '{{ variant.name }}' for '{{ union.name }}'");
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
                case {{ union.choice }}.{{ variant.name }}:
{%- if variant.ty %}
                    {
                        var inner = value.As<{{ variant.ty }}>();
                        writer.WriteStartObject();
                        writer.WritePropertyName("{{ variant.name }}");
                        JsonSerializer.Serialize(writer, inner, options);
                        writer.WriteEndObject();
                        break;
                    }
{%- else %}
                    {
                        writer.WriteStringValue("{{ variant.name }}");
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
