use crate::resolver::TypeResolver;
use crate::types::Field;
use mech3ax_metadata_types::TypeSemantic;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Struct {
    pub name: &'static str,
    #[serde(skip)]
    pub semantic: TypeSemantic,
    pub sem_type: &'static str,
    pub fields: Vec<Field>,
}

impl Struct {
    pub fn new<S>(resolver: &TypeResolver) -> Self
    where
        S: mech3ax_metadata_types::Struct,
    {
        let name = S::NAME;
        let semantic = S::SEMANTIC;
        let sem_type = match S::SEMANTIC {
            TypeSemantic::Ref => "class",
            TypeSemantic::Val => "struct",
        };
        let fields = S::FIELDS
            .iter()
            .map(|ti| Field::new(ti, resolver))
            .collect();
        Self {
            name,
            semantic,
            sem_type,
            fields,
        }
    }

    pub fn render(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("struct", self);
        tera.render("struct.cs", &context)
    }
}

macro_rules! struct_macro {
    () => (
r###"
{% macro make_struct(struct) %}
    [JsonConverter(typeof({{ struct.name }}Converter))]
    public {{ struct.sem_type }} {{ struct.name }}
    {
{%- for field in struct.fields %}
        public {{ field.ty }} {{ field.name }}{% if field.default %} = {{ field.default }}{% endif %};
{%- endfor %}

        public {{ struct.name }}({% for field in struct.fields %}{{ field.ty }} {{ field.name }}{% if not loop.last %}, {% endif %}{% endfor %})
        {
{%- for field in struct.fields %}
            this.{{ field.name }} = {{ field.name }};
{%- endfor %}
        }
    }

    internal class {{ struct.name }}Converter : StructConverter<{{ struct.name }}>
    {
        protected override {{ struct.name }} ReadStruct(JsonReader __reader, IEnumerable<object?> __fieldNames, JsonSerializer __serializer)
        {
{%- for field in struct.fields %}
            var {{ field.name }}Field = new Option<{{ field.ty }}>({% if field.default %}{{ field.default }}{% endif %});
{%- endfor %}
            foreach (var __fieldName in __fieldNames)
            {
                switch (__fieldName)
                {
{%- for field in struct.fields %}
                    case "{{ field.json_name }}":
                        {
{%- if field.null_check %}
                            {{ field.ty }}? __value = ReadFieldValue<{{ field.ty }}?>(__reader, __serializer);
                            if (__value is null)
                                throw new JsonSerializationException(
                                    AddPath(__reader, $"Required property '{{ field.json_name }}' expects a value but got null"));
{%- else %}
                            {{ field.ty }} __value = ReadFieldValue<{{ field.ty }}>(__reader, __serializer);
{%- endif %}
                            {{ field.name }}Field.Set(__value);
                            break;
                        }
{%- endfor %}
                    default:
                        throw new JsonSerializationException(
                            AddPath(__reader, $"Could not find member '{__fieldName}' on struct of type '{{ struct.name }}'"));
                }
            }
            // pray there are no naming collisions
{%- for field in struct.fields %}
            {{ field.ty }} {{ field.name }} = {{ field.name }}Field.Unwrap("{{ field.json_name }}", __reader);
{%- endfor %}
            return new {{ struct.name }}({% for field in struct.fields %}{{ field.name }}{% if not loop.last %}, {% endif %}{% endfor %});
        }

        public override void WriteJson(JsonWriter writer, {{ struct.name }} value, JsonSerializer serializer)
        {
            writer.WriteStartObject();
{%- for field in struct.fields %}
            writer.WritePropertyName("{{ field.json_name }}");
            serializer.Serialize(writer, value.{{ field.name }});
{%- endfor %}
            writer.WriteEndObject();
        }
    }
{% endmacro make_struct %}
"###
    )
}

pub(crate) use struct_macro;
