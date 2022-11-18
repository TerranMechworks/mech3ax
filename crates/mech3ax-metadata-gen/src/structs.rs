use crate::csharp_type::{CSharpType, TypeKind};
use crate::fields::{join_generics, Field};
use crate::resolver::TypeResolver;
use mech3ax_metadata_types::{TypeInfoStruct, TypeSemantic};
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize)]
pub struct Struct {
    /// The struct's C# struct name, without generics.
    pub name: &'static str,
    /// The struct's C# type name, with generics.
    pub type_name: Cow<'static, str>,
    /// The struct's C# converter type name, with generics.
    pub conv_name: String,
    /// The struct's C# semantic type as a string, `class` or `struct`.
    pub sem_type: &'static str,
    /// The struct's fields.
    pub fields: Vec<Field>,
    /// The struct's C# semantic type.
    ///
    /// Used for `make_type()`, but not inside any templates.
    #[serde(skip)]
    pub semantic: TypeSemantic,
    /// The struct's generic arguments.
    ///
    /// Used for selecting the converter template, but not inside any templates.
    #[serde(skip)]
    pub generics: Option<HashSet<&'static str>>,
}

impl Struct {
    pub fn make_type(&self) -> CSharpType {
        // our "structs" can either be
        // * a C# `class` (reference type)
        // * a C# `struct` (value type)
        let kind = match self.semantic {
            TypeSemantic::Ref => TypeKind::Ref,
            TypeSemantic::Val => TypeKind::Val,
        };
        CSharpType {
            name: self.type_name.clone(),
            kind,
            generics: self.generics.clone(),
        }
    }

    pub fn new(resolver: &mut TypeResolver, si: &TypeInfoStruct) -> Self {
        // luckily, Rust's casing for structs matches C#.
        let name = si.name;
        let semantic = si.semantic;
        let sem_type = match semantic {
            TypeSemantic::Ref => "class",
            TypeSemantic::Val => "struct",
        };
        // field generics must be declared on this struct specifically.
        let fields: Vec<_> = si
            .fields
            .iter()
            .map(|field_info| Field::new(si.name, si.generics, field_info, resolver))
            .collect();

        // since the struct's generics should've be used in fields, we can
        // simply inherit all the generics from the fields. note that this also
        // includes fields that might not be directly generic (e.g. fields that
        // are in themselves generic structures).
        let mut generics: HashSet<&'static str> = HashSet::new();
        for field in fields.iter() {
            if let Some(field_generics) = &field.generics {
                generics.extend(field_generics.iter());
            }
        }

        let (type_name, conv_name, generics) = if generics.is_empty() {
            // type is not generic, so converter isn't either
            let type_name = Cow::Borrowed(si.name);
            let conv_name = format!("{}Converter", si.name);
            (type_name, conv_name, None)
        } else {
            // the is generic, so the converter is also generic
            let generics_joined = join_generics(&generics);
            let type_name = Cow::Owned(format!("{}<{}>", name, generics_joined));
            let conv_name = format!("{}Converter<{}>", si.name, generics_joined);
            resolver.push_factory_converter(si.name.to_string(), generics.len());
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
        match self.generics.is_some() {
            false => tera.render("struct_normal_conv.cs", &context),
            true => tera.render("struct_generic_conv.cs", &context),
        }
    }
}

pub const STRUCT_IMPL: &'static str = r###"namespace Mech3DotNet.Json
{
{%- if struct.name == struct.type_name %}
    [System.Text.Json.Serialization.JsonConverter(typeof(Mech3DotNet.Json.Converters.{{ struct.name }}Converter))]
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

pub const STRUCT_NORMAL_CONV: &'static str = r###"using System.Text.Json;

namespace Mech3DotNet.Json.Converters
{
    public class {{ struct.name }}Converter : Mech3DotNet.Json.Converters.StructConverter<{{ struct.name }}>
    {
        protected override {{ struct.name }} ReadStruct(ref Utf8JsonReader __reader, JsonSerializerOptions __options)
        {
{%- for field in struct.fields %}
            var {{ field.name }}Field = new Mech3DotNet.Json.Converters.Option<{{ field.ty }}>({% if field.default %}{{ field.default }}{% endif %});
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

pub const STRUCT_GENERIC_CONV: &'static str = r###"using System.Text.Json;

namespace Mech3DotNet.Json.Converters
{
    public class {{ struct.conv_name }} : Mech3DotNet.Json.Converters.StructConverter<{{ struct.type_name }}>
    {
{%- for field in struct.fields %}{% if field.generics %}
        private readonly System.Type __{{ field.name }}Type;
        private readonly System.Text.Json.Serialization.JsonConverter<{{ field.ty }}{% if field.null_check %}?{% endif %}>? __{{ field.name }}Converter;
{%- endif %}{% endfor %}

        public {{ struct.name }}Converter(JsonSerializerOptions options)
        {
{%- for field in struct.fields %}{% if field.generics %}
            __{{ field.name }}Type = typeof({{ field.ty | trim_end_matches(pat="?") }});
            __{{ field.name }}Converter = (System.Text.Json.Serialization.JsonConverter<{{ field.ty }}{% if field.null_check %}?{% endif %}>?)options.GetConverter(__{{ field.name }}Type);
{%- endif %}{% endfor %}
        }

        protected override {{ struct.type_name }} ReadStruct(ref Utf8JsonReader __reader, JsonSerializerOptions __options)
        {
{%- for field in struct.fields %}
            var {{ field.name }}Field = new Mech3DotNet.Json.Converters.Option<{{ field.ty }}>({% if field.default %}{{ field.default }}{% endif %});
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
