use crate::resolver::TypeResolver;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Union {
    pub name: &'static str,
    pub choice: String,
    pub variants: &'static [(&'static str, Option<&'static str>)],
}

impl Union {
    pub fn new<S>(resolver: &TypeResolver) -> Self
    where
        S: mech3ax_metadata_types::Union,
    {
        let name = S::NAME;
        let choice = format!("{}Variant", name);
        let variants = S::VARIANTS;

        for (_variant_name, struct_name) in variants {
            if let Some(name) = struct_name {
                if resolver.lookup_struct(name).is_none() {
                    eprintln!("WARNING: structure {} not found", name);
                }
            }
        }

        Self {
            name,
            choice,
            variants,
        }
    }

    pub fn render(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("union", self);
        tera.render("union.cs", &context)
    }
}

macro_rules! union_macro {
    () => (
r###"
{% macro make_union(union) %}
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
        public bool Is<T>() where T : class { return typeof(T).IsInstanceOfType(value); }
        public T As<T>() where T : class { return (T)value; }
        public object GetValue() { return value; }
    }

    public class {{ union.name }}Converter : UnionConverter<{{ union.name }}>
    {
        public override {{ union.name }} ReadUnitVariant(JsonReader reader, string? name, JsonSerializer serializer)
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
                    throw new JsonException("Invalid unit variant '{{ variant.0 }}' for '{{ union.name }}'");
{%- endif %}{% endfor %}
                case null:
                    throw new JsonException("Variant for '{{ union.name }}' cannot be null");
                default:
                    throw new JsonException($"Invalid variant '{name}' for '{{ union.name }}'");
            }
        }

        public override {{ union.name }} ReadStructVariant(JsonReader reader, string? name, JsonSerializer serializer)
        {
            switch (name)
            {
{%- for variant in union.variants %}{%- if variant.1 %}
                case "{{ variant.0 }}":
                    {
                        var inner = serializer.Deserialize<{{ variant.1 }}>(reader);
                        if (inner is null)
                            throw new JsonException("Data of variant '{{ variant.0 }}' for '{{ union.name }}' cannot be null");
                        return new {{ union.name }}(inner);
                    }
{%- endif %}{% endfor %}
{%- for variant in union.variants %}{%- if not variant.1 %}
                case "{{ variant.0 }}":
                    throw new JsonException("Invalid struct variant '{{ variant.0 }}' for '{{ union.name }}'");
{%- endif %}{% endfor %}
                case null:
                    throw new JsonException("Variant for '{{ union.name }}' cannot be null");
                default:
                    throw new JsonException($"Invalid variant '{name}' for '{{ union.name }}'");
            }
        }

        public override void WriteJson(JsonWriter writer, {{ union.name }} value, JsonSerializer serializer)
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
                        serializer.Serialize(writer, inner);
                        writer.WriteEndObject();
                        break;
                    }
{%- else %}
                    {
                        writer.WriteValue("{{ variant.0 }}");
                        break;
                    }
{%- endif %}
{%- endfor %}
                default:
                    throw new ArgumentOutOfRangeException("Variant", $"Invalid variant '{value.Variant}' for '{{ union.name }}'");
            }
        }
    }
{% endmacro make_union %}
"###
    )
}

pub(crate) use union_macro;
