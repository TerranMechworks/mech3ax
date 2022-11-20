use crate::csharp_type::{CSharpType, TypeKind};
use crate::module_path::convert_mod_path;
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
    /// The union's C# type name.
    pub name: &'static str,
    /// The union's C# namespace.
    pub namespace: String,
    /// The union's full C# type, with namespace.
    pub full_name: String,
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
            name: Cow::Owned(self.full_name.clone()),
            kind: TypeKind::Ref,
            generics: None,
        }
    }

    pub fn new(resolver: &TypeResolver, ui: &TypeInfoUnion) -> Self {
        // luckily, Rust's casing for enum names matches C# classes.
        let name = ui.name;
        let namespace = convert_mod_path(ui.module_path);
        let full_name = format!("{}.{}", namespace, ui.name);
        let choice = format!("{}Variant", name);

        let variants = ui
            .variants
            .iter()
            .copied()
            .map(|(variant_name, variant_type)| {
                resolve_variant(name, resolver, variant_name, variant_type)
            })
            .collect();

        Self {
            name,
            namespace,
            full_name,
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

pub const UNION_IMPL: &str = r###"namespace {{ union.namespace }}
{
    public enum {{ union.choice }}
    {
{%- for variant in union.variants %}
        {{ variant.name }},
{%- endfor %}
    }

    [System.Text.Json.Serialization.JsonConverter(typeof({{ union.namespace }}.Converters.{{ union.name }}Converter))]
    public class {{ union.name }} : Mech3DotNet.Json.Converters.IDiscriminatedUnion<{{ union.choice }}>
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

pub const UNION_CONV: &str = r###"using System.Text.Json;

namespace {{ union.namespace }}.Converters
{
    public class {{ union.name }}Converter : Mech3DotNet.Json.Converters.UnionConverter<{{ union.full_name }}>
    {
        public override {{ union.full_name }} ReadUnitVariant(string? name)
        {
            switch (name)
            {
{%- for variant in union.variants %}{%- if not variant.ty %}
                case "{{ variant.name }}":
                    {
                        var inner = new {{ union.full_name }}.{{ variant.name }}();
                        return new {{ union.full_name }}(inner);
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

        public override {{ union.full_name }} ReadStructVariant(ref Utf8JsonReader reader, string? name, JsonSerializerOptions options)
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
                        return new {{ union.full_name }}(inner);
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

        public override void Write(Utf8JsonWriter writer, {{ union.full_name }} value, JsonSerializerOptions options)
        {
            switch (value.Variant)
            {
{%- for variant in union.variants %}
                case {{ union.namespace }}.{{ union.choice }}.{{ variant.name }}:
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
                    throw new System.ArgumentOutOfRangeException("Variant", $"Invalid variant '{value.Variant}' for '{{ union.name }}'");
            }
        }
    }
}
"###;
