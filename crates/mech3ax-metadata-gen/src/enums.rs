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

    pub fn render(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("enum", self);
        tera.render("enum.cs", &context)
    }
}

macro_rules! enum_macro {
    () => (
r###"
{% macro make_enum(enum) %}
    [JsonConverter(typeof({{ enum.name }}Converter))]
    public enum {{ enum.name }}
    {
{%- for variant in enum.variants %}
        {{ variant }},
{%- endfor %}
    }

    internal class {{ enum.name }}Converter : EnumConverter<{{ enum.name }}>
    {
        public override {{ enum.name }} ReadVariant(string? name) => name switch {
{%- for variant in enum.variants %}
            "{{ variant }}" => {{ enum.name }}.{{ variant }},
{%- endfor %}
            null => throw new JsonSerializationException($"Variant cannot be null for '{{ enum.name }}'"),
            _ => throw new JsonSerializationException($"Invalid variant '{name}' for '{{ enum.name }}'"),
        };

        public override string WriteVariant({{ enum.name }} value) => value switch {
{%- for variant in enum.variants %}
            {{ enum.name }}.{{ variant }} => "{{ variant }}",
{%- endfor %}
            _ => throw new ArgumentOutOfRangeException("{{ enum.name }}"),
        };
    }
{% endmacro make_enum %}
"###
    )
}

pub(crate) use enum_macro;
