use crate::fields::{Field, Generics};
use crate::resolver::TypeResolver;
use mech3ax_metadata_types::TypeSemantic;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize)]
pub struct Struct {
    pub name: &'static str,
    pub type_name: String,
    pub conv_name: String,
    pub sem_type: &'static str,
    pub fields: Vec<Field>,
    #[serde(skip)]
    pub semantic: TypeSemantic,
    #[serde(skip)]
    pub generics: Generics,
}

impl Struct {
    pub fn new<S>(resolver: &mut TypeResolver) -> Self
    where
        S: mech3ax_metadata_types::Struct,
    {
        let name = S::NAME;
        let semantic = S::SEMANTIC;
        let sem_type = match S::SEMANTIC {
            TypeSemantic::Ref => "class",
            TypeSemantic::Val => "struct",
        };

        // collect this struct's generics. this must be done before the fields,
        // so types can be substituted. even though we may inherit more
        // generics from the fields later, if they aren't declared on this
        // struct, they cannot appear as field types.
        let mut generics: HashSet<(&'static str, &'static str)> = match S::GENERICS {
            None => HashSet::new(),
            Some(gen) => gen
                .iter()
                .map(|(ty_old, generic)| (*ty_old, *generic))
                .collect(),
        };

        let fields: Vec<_> = S::FIELDS
            .iter()
            .map(|ti| Field::new(ti, &generics, resolver))
            .collect();

        // inherit generics from fields
        for field in fields.iter() {
            if let Some(field_generics) = &field.generics {
                generics.extend(field_generics.iter());
            }
        }

        let (type_name, conv_name, generics) = if generics.is_empty() {
            // type is not generic, so converter isn't either
            let type_name = S::NAME.to_string();
            let conv_name = format!("{}Converter", S::NAME);
            (type_name, conv_name, None)
        } else {
            let mut generics_split = generics
                .iter()
                .map(|(_concrete, generic)| *generic)
                .collect::<Vec<&str>>();
            // sort the generics by name for a nice, stable display order.
            generics_split.sort();
            let generics_joined = generics_split.join(", ");

            let type_name = format!("{}<{}>", name, generics_joined);
            let conv_name = format!("{}Converter<{}>", S::NAME, generics_joined);

            resolver.push_factory_converter(S::NAME.to_string(), generics.len());

            (type_name, conv_name, Some(generics))
        };

        Self {
            name,
            type_name,
            conv_name,
            sem_type,
            fields,
            semantic,
            generics,
        }
    }

    pub fn render_impl(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("struct", self);
        tera.render("struct_impl.cs", &context)
    }

    pub fn render_conv(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("struct", self);
        match self.generics {
            None => tera.render("struct_normal_conv.cs", &context),
            Some(_) => tera.render("struct_generic_conv.cs", &context),
        }
    }
}

pub const STRUCT_IMPL: &'static str = r###"using System;
using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;
using Mech3DotNet.Json.Converters;

namespace Mech3DotNet.Json
{
{%- if struct.name == struct.type_name %}
    [JsonConverter(typeof({{ struct.name }}Converter))]
{%- endif %}
    public {{ struct.sem_type }} {{ struct.type_name }}
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
}
"###;

pub const STRUCT_NORMAL_CONV: &'static str = r###"using System;
using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;
using Mech3DotNet.Json;

namespace Mech3DotNet.Json.Converters
{
    public class {{ struct.name }}Converter : StructConverter<{{ struct.name }}>
    {
        protected override {{ struct.name }} ReadStruct(ref Utf8JsonReader __reader, JsonSerializerOptions __options)
        {
{%- for field in struct.fields %}
            var {{ field.name }}Field = new Option<{{ field.ty }}>({% if field.default %}{{ field.default }}{% endif %});
{%- endfor %}
            string? __fieldName = null;
            while (ReadFieldName(ref __reader, out __fieldName))
            {
                switch (__fieldName)
                {
{%- for field in struct.fields %}
                    case "{{ field.json_name }}":
                        {
                            {{ field.ty }}{% if field.null_check %}?{% endif %} __value = ReadFieldValue<{{ field.ty }}{% if field.null_check %}?{% endif %}>(ref __reader, __options);
{%- if field.null_check %}
                            if (__value is null)
                            {
                                System.Diagnostics.Debug.WriteLine("Value of '{{ field.json_name }}' was null for '{{ struct.name }}'");
                                throw new JsonException();
                            }
{%- endif %}
                            {{ field.name }}Field.Set(__value);
                            break;
                        }
{%- endfor %}
                    default:
                        {
                            System.Diagnostics.Debug.WriteLine($"Invalid field '{__fieldName}' for '{{ struct.name }}'");
                            throw new JsonException();
                        }
                }
            }
            // pray there are no naming collisions
{%- for field in struct.fields %}
            var {{ field.name }} = {{ field.name }}Field.Unwrap("{{ field.json_name }}");
{%- endfor %}
            return new {{ struct.name }}({% for field in struct.fields %}{{ field.name }}{% if not loop.last %}, {% endif %}{% endfor %});
        }

        public override void Write(Utf8JsonWriter writer, {{ struct.name }} value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
{%- for field in struct.fields %}{% if field.default %}
            if (value.{{ field.name }} != {{ field.default}})
            {
                writer.WritePropertyName("{{ field.json_name }}");
                JsonSerializer.Serialize(writer, value.{{ field.name }}, options);
            }
{%- else %}
            writer.WritePropertyName("{{ field.json_name }}");
            JsonSerializer.Serialize(writer, value.{{ field.name }}, options);
{%- endif %}{% endfor %}
            writer.WriteEndObject();
        }
    }
}
"###;

pub const STRUCT_GENERIC_CONV: &'static str = r###"using System;
using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;
using Mech3DotNet.Json;

namespace Mech3DotNet.Json.Converters
{
    public class {{ struct.conv_name }} : StructConverter<{{ struct.type_name }}>
    {
{%- for field in struct.fields %}{% if field.generics %}
        private readonly Type __{{ field.name }}Type;
        private readonly JsonConverter<{{ field.ty }}{% if field.null_check %}?{% endif %}>? __{{ field.name }}Converter;
{%- endif %}{% endfor %}

        public {{ struct.name }}Converter(JsonSerializerOptions options)
        {
{%- for field in struct.fields %}{% if field.generics %}
            __{{ field.name }}Type = typeof({{ field.ty | trim_end_matches(pat="?") }});
            __{{ field.name }}Converter = (JsonConverter<{{ field.ty }}{% if field.null_check %}?{% endif %}>?)options.GetConverter(__{{ field.name }}Type);
{%- endif %}{% endfor %}
        }

        protected override {{ struct.type_name }} ReadStruct(ref Utf8JsonReader __reader, JsonSerializerOptions __options)
        {
{%- for field in struct.fields %}
            var {{ field.name }}Field = new Option<{{ field.ty }}>({% if field.default %}{{ field.default }}{% endif %});
{%- endfor %}
            string? __fieldName = null;
            while (ReadFieldName(ref __reader, out __fieldName))
            {
                switch (__fieldName)
                {
{%- for field in struct.fields %}
                    case "{{ field.json_name }}":
                        {
{%- if field.generics %}
                            {{ field.ty }}{% if field.null_check %}?{% endif %} __value = ReadFieldGeneric<{{ field.ty }}{% if field.null_check %}?{% endif %}>(
                                ref __reader,
                                __options,
                                __{{ field.name }}Converter,
                                __{{ field.name }}Type);
{%- else %}
                            {{ field.ty }}{% if field.null_check %}?{% endif %} __value = ReadFieldValue<{{ field.ty }}{% if field.null_check %}?{% endif %}>(
                                ref __reader, __options);
{%- endif %}
{%- if field.null_check %}
                            if (__value is null)
                            {
                                System.Diagnostics.Debug.WriteLine("Value of '{{ field.json_name }}' was null for '{{ struct.name }}'");
                                throw new JsonException();
                            }
{%- endif %}
                            {{ field.name }}Field.Set(__value);
                            break;
                        }
{%- endfor %}
                    default:
                        {
                            System.Diagnostics.Debug.WriteLine($"Invalid field '{__fieldName}' for '{{ struct.name }}'");
                            throw new JsonException();
                        }
                }
            }
            // pray there are no naming collisions
{%- for field in struct.fields %}
            var {{ field.name }} = {{ field.name }}Field.Unwrap("{{ field.json_name }}");
{%- endfor %}
            return new {{ struct.type_name }}({% for field in struct.fields %}{{ field.name }}{% if not loop.last %}, {% endif %}{% endfor %});
        }

        public override void Write(Utf8JsonWriter writer, {{ struct.type_name }} value, JsonSerializerOptions options)
        {
            writer.WriteStartObject();
{%- for field in struct.fields %}{% if field.default %}
            if (value.{{ field.name }} != {{ field.default}})
            {
                writer.WritePropertyName("{{ field.json_name }}");
{%- if field.generics %}
                if (__{{ field.name }}Converter != null)
                    __{{ field.name }}Converter.Write(writer, value.{{ field.name }}, options);
                else
                    JsonSerializer.Serialize(writer, value.{{ field.name }}, options);
{%- else %}
                JsonSerializer.Serialize(writer, value.{{ field.name }}, options);
{%- endif %}
            }
{%- else %}
            writer.WritePropertyName("{{ field.json_name }}");
{%- if field.generics %}
            if (__{{ field.name }}Converter != null)
                __{{ field.name }}Converter.Write(writer, value.{{ field.name }}, options);
            else
                JsonSerializer.Serialize(writer, value.{{ field.name }}, options);
{%- else %}
            JsonSerializer.Serialize(writer, value.{{ field.name }}, options);
{%- endif %}
{%- endif %}{% endfor %}
            writer.WriteEndObject();
        }
    }
}
"###;
