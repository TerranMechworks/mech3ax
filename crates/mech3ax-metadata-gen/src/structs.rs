use crate::csharp_type::{CSharpType, SerializeType, TypeKind};
use crate::fields::{sort_generics, Field};
use crate::module_path::convert_mod_path;
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
    /// The struct's C# namespace.
    pub namespace: String,
    /// The struct's full C# type, with namespace and generics.
    pub full_name: String,
    /// The struct's C# converter type name, with generics.
    pub conv_name: String,
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
    pub generics_sorted: Vec<&'static str>,
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
        let serde = match self.semantic {
            TypeSemantic::Ref => SerializeType::Class(self.full_name.clone()),
            TypeSemantic::Val => SerializeType::Struct(self.full_name.clone()),
        };
        CSharpType {
            name: Cow::Owned(self.full_name.clone()),
            kind,
            generics: self.generics.clone(),
            serde,
        }
    }

    pub fn new(resolver: &mut TypeResolver, si: &TypeInfoStruct) -> Self {
        // luckily, Rust's casing for structs matches C#.
        let name = si.name;
        let namespace = convert_mod_path(si.module_path);

        let semantic = si.semantic;
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

        let (type_name, conv_name, generics, generics_sorted) = if generics.is_empty() {
            // type is not generic, so converter isn't either
            let type_name = Cow::Borrowed(si.name);
            let conv_name = format!("{}Converter", si.name);
            (type_name, conv_name, None, Vec::new())
        } else {
            if semantic == TypeSemantic::Val {
                panic!("Value type `{}` cannot have generics", si.name);
            }
            // the is generic, so the converter is also generic
            let generics_sorted = sort_generics(&generics);
            let generics_joined = generics_sorted.join(", ");
            let type_name = Cow::Owned(format!("{}<{}>", name, generics_joined));
            let conv_name = format!("{}Converter<{}>", si.name, generics_joined);
            (type_name, conv_name, Some(generics), generics_sorted)
        };

        let full_name = format!("{}.{}", namespace, type_name);

        Self {
            name,
            type_name,
            namespace,
            full_name,
            conv_name,
            fields,
            semantic,
            generics,
            generics_sorted,
        }
    }

    pub fn render_impl(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("struct", self);
        let template_name = match self.semantic {
            TypeSemantic::Ref => "class_impl.cs",
            TypeSemantic::Val => "struct_impl.cs",
        };
        tera.render(template_name, &context)
    }
}

pub const CLASS_IMPL: &str = r###"using System;
using Mech3DotNet.Exchange;

namespace {{ struct.namespace }}
{
    public sealed class {{ struct.type_name }}
{%- for generic in struct.generics_sorted %}
        where {{ generic }} : notnull
{%- endfor %}
    {
        public static readonly TypeConverter<{{ struct.type_name }}> Converter = new TypeConverter<{{ struct.type_name }}>(Deserialize, Serialize);

{%- for field in struct.fields %}
        public {{ field.ty }} {{ field.name }}{% if field.default %} = {{ field.default }}{% endif %};
{%- endfor %}

        public {{ struct.name }}({% for field in struct.fields %}{{ field.ty }} {{ field.name }}{% if not loop.last %}, {% endif %}{% endfor %})
        {
{%- for field in struct.fields %}
            this.{{ field.name }} = {{ field.name }};
{%- endfor %}
        }

        private struct Fields
        {
{%- for field in struct.fields %}
            public Field<{{ field.ty }}> {{ field.name }};
{%- endfor %}
        }

        public static void Serialize({{ struct.type_name }} v, Serializer s)
        {
            s.SerializeStruct("{{ struct.name }}", {{ struct.fields | length }});
{%- for field in struct.fields %}
            s.SerializeFieldName("{{ field.key }}");
            {{ field.serde.serialize }}(v.{{ field.name }});
{%- endfor %}
        }

        public static {{ struct.type_name }} Deserialize(Deserializer d)
        {
            var fields = new Fields()
            {
{%- for field in struct.fields %}
                {{ field.name }} = new Field<{{ field.ty }}>({% if field.default %}{{ field.default }}{% endif %}),
{%- endfor %}
            };
            foreach (var fieldName in d.DeserializeStruct("{{ struct.name }}"))
            {
                switch (fieldName)
                {
{%- for field in struct.fields %}
                    case "{{ field.key }}":
                        fields.{{ field.name }}.Value = {{ field.serde.deserialize }}();
                        break;
{%- endfor %}
                    default:
                        throw new UnknownFieldException("{{ struct.name }}", fieldName);
                }
            }
            return new {{ struct.type_name }}(
{% for field in struct.fields %}
                fields.{{ field.name }}.Unwrap("{{ struct.name }}", "{{ field.name }}"){% if not loop.last %},{% endif %}
{% endfor %}
            );
        }
    }
}
"###;

pub const STRUCT_IMPL: &str = r###"using System;
using Mech3DotNet.Exchange;

namespace {{ struct.namespace }}
{
    public struct {{ struct.type_name }}
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

    public static class {{ struct.type_name }}Converter
    {
        public static readonly TypeConverter<{{ struct.type_name }}> Converter = new TypeConverter<{{ struct.type_name }}>(Deserialize, Serialize);

        private struct Fields
        {
{%- for field in struct.fields %}
            public Field<{{ field.ty }}> {{ field.name }};
{%- endfor %}
        }

        public static void Serialize({{ struct.type_name }} v, Serializer s)
        {
            s.SerializeStruct("{{ struct.name }}", {{ struct.fields | length }});
{%- for field in struct.fields %}
            s.SerializeFieldName("{{ field.key }}");
            {{ field.serde.serialize }}(v.{{ field.name }});
{%- endfor %}
        }

        public static {{ struct.type_name }} Deserialize(Deserializer d)
        {
            var fields = new Fields()
            {
{%- for field in struct.fields %}
                {{ field.name }} = new Field<{{ field.ty }}>({% if field.default %}{{ field.default }}{% endif %}),
{%- endfor %}
            };
            foreach (var fieldName in d.DeserializeStruct("{{ struct.name }}"))
            {
                switch (fieldName)
                {
{%- for field in struct.fields %}
                    case "{{ field.key }}":
                        fields.{{ field.name }}.Value = {{ field.serde.deserialize }}();
                        break;
{%- endfor %}
                    default:
                        throw new UnknownFieldException("{{ struct.name }}", fieldName);
                }
            }
            return new {{ struct.type_name }}(
{% for field in struct.fields %}
                fields.{{ field.name }}.Unwrap("{{ struct.name }}", "{{ field.name }}"){% if not loop.last %},{% endif %}
{% endfor %}
            );
        }
    }
}
"###;
