use super::csharp_type::{CSharpType, SerializeType, TypeKind};
use super::fields::{sort_generics, Field};
use super::module_path::{dotnet_namespace_to_path, rust_mod_path_to_dotnet};
use super::resolver::TypeResolver;
use mech3ax_metadata_types::{TypeInfoStruct, TypeSemantic};
use minijinja::{context, Environment};
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;

const MOTION_FRAME_GENERICS: &[(&mech3ax_metadata_types::TypeInfo, &str)] = &[
    (
        <mech3ax_api_types::Vec3 as mech3ax_metadata_types::DerivedMetadata>::TYPE_INFO,
        "TVec3",
    ),
    (
        <mech3ax_api_types::Quaternion as mech3ax_metadata_types::DerivedMetadata>::TYPE_INFO,
        "TQuaternion",
    ),
];

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Struct {
    /// The struct's C# struct name, without generics.
    pub(crate) name: &'static str,
    /// The struct's C# type name, with generics.
    pub(crate) type_name: Cow<'static, str>,
    /// The struct's C# namespace.
    pub(crate) namespace: String,
    /// The struct's full C# type, with namespace and generics.
    pub(crate) full_name: String,
    /// The struct's C# converter type name, with generics.
    pub(crate) conv_name: String,
    /// The struct's fields.
    pub(crate) fields: Vec<Field>,
    /// The struct's C# semantic type.
    ///
    /// Used for `make_type()`, but not inside any templates.
    #[serde(skip)]
    pub(crate) semantic: TypeSemantic,
    /// The struct's generic arguments.
    ///
    /// Used for selecting the converter template, but not inside any templates.
    #[serde(skip)]
    pub(crate) generics: Option<HashSet<&'static str>>,
    pub(crate) generics_sorted: Vec<&'static str>,
    /// Whether the struct is a partial struct.
    pub(crate) partial: bool,
    /// The structs's path on the filesystem.
    pub(crate) path: PathBuf,
}

impl Struct {
    pub(crate) fn make_type(&self) -> CSharpType {
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

    pub(crate) fn new(resolver: &mut TypeResolver, si: &TypeInfoStruct) -> Self {
        // luckily, Rust's casing for structs matches C#.
        let name = si.name;
        let (namespace, partial) = match si.name {
            "GameZDataMw" | "GameZDataPm" | "GameZDataRc" | "Messages" | "Zmap" => {
                ("Mech3DotNet.Zbd".to_string(), true)
            }
            _ => (rust_mod_path_to_dotnet(si.module_path), false),
        };
        let semantic = si.semantic;

        let generics = if si.name == "MotionFrame" {
            Some(MOTION_FRAME_GENERICS)
        } else {
            None
        };

        // field generics must be declared on this struct specifically.
        let fields: Vec<_> = si
            .fields
            .iter()
            .map(|field_info| Field::new(si.name, generics, field_info, resolver))
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

        let mut path = dotnet_namespace_to_path(&namespace);
        resolver.add_directory(&path);
        path.push(format!("{}.cs", name));

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
            partial,
            path,
        }
    }

    pub(crate) fn render_impl(&self, env: &Environment<'_>) -> Result<String, minijinja::Error> {
        let template_name = match self.semantic {
            TypeSemantic::Ref => "class_impl.cs",
            TypeSemantic::Val => "struct_impl.cs",
        };
        let template = env.get_template(template_name)?;
        template.render(context! { struct => self })
    }
}

pub(crate) const CLASS_IMPL: &str = r#"using System;
using Mech3DotNet.Exchange;

namespace {{ struct.namespace }}
{
    public {% if struct.partial %}partial{% else %}sealed{% endif %} class {{ struct.type_name }}
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
            s.SerializeStruct({{ struct.fields | length }});
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
            foreach (var fieldName in d.DeserializeStruct())
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

"#;

pub(crate) const STRUCT_IMPL: &str = r#"using System;
using Mech3DotNet.Exchange;

namespace {{ struct.namespace }}
{
    public struct {{ struct.type_name }}
    {
{%- for field in struct.fields %}
        public {{ field.ty }} {{ field.name }};
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
            s.SerializeStruct({{ struct.fields | length }});
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
            foreach (var fieldName in d.DeserializeStruct())
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

"#;
