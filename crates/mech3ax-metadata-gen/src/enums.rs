use crate::csharp_type::{CSharpType, SerializeType, TypeKind};
use crate::module_path::{dotnet_namespace_to_path, rust_mod_path_to_dotnet};
use crate::resolver::TypeResolver;
use mech3ax_metadata_types::TypeInfoEnum;
use serde::Serialize;
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct Variant {
    /// The enum variant's name.
    pub name: &'static str,
    /// The enum variant's index.
    pub index: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Enum {
    /// The enum's C# type name.
    pub name: &'static str,
    /// The enum's C# namespace.
    pub namespace: String,
    /// The enum's full C# type, with namespace.
    pub full_name: String,
    /// The enum's C# variant names.
    pub variants: Vec<Variant>,
    /// The enum's path on the filesystem.
    pub path: PathBuf,
}

impl Enum {
    pub fn make_type(&self) -> CSharpType {
        // our "enums" are a C# enum, which are a C# `struct` (value type)
        CSharpType {
            name: Cow::Owned(self.full_name.clone()),
            kind: TypeKind::Val,
            generics: None,
            serde: SerializeType::Enum(self.full_name.clone()),
        }
    }

    pub fn new(resolver: &mut TypeResolver, ei: &TypeInfoEnum) -> Self {
        // luckily, Rust's casing for enum and variant names matches C#.
        let name = ei.name;
        let namespace = rust_mod_path_to_dotnet(ei.module_path);
        let full_name = format!("{}.{}", namespace, ei.name);

        let variants = ei
            .variants
            .iter()
            .copied()
            .zip(0u32..)
            .map(|(name, index)| Variant { name, index })
            .collect();

        let mut path = dotnet_namespace_to_path(&namespace);
        resolver.add_directory(&path);
        path.push(format!("{}.cs", name));

        Self {
            name,
            namespace,
            full_name,
            variants,
            path,
        }
    }

    pub fn render_impl(&self, tera: &tera::Tera) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("enum", self);
        tera.render("enum_impl.cs", &context)
    }
}

pub const ENUM_IMPL: &str = r###"using Mech3DotNet.Exchange;

namespace {{ enum.namespace }}
{
    public enum {{ enum.name }}
    {
{%- for variant in enum.variants %}
        {{ variant.name }},
{%- endfor %}
    }

    public static class {{ enum.name }}Converter
    {
        public static readonly TypeConverter<{{ enum.name }}> Converter = new TypeConverter<{{ enum.name }}>(Deserialize, Serialize);

        private static void Serialize({{ enum.name }} v, Serializer s)
        {
            uint variantIndex = v switch
            {
{%- for variant in enum.variants %}
                {{ enum.name }}.{{ variant.name }} => {{ variant.index }},
{%- endfor %}
                _ => throw new System.ArgumentOutOfRangeException(),
            };
            s.SerializeUnitVariant(variantIndex);
        }

        private static {{ enum.name }} Deserialize(Deserializer d)
        {
            var variantIndex = d.DeserializeUnitVariant("{{ enum.name }}");
            return variantIndex switch
            {
{%- for variant in enum.variants %}
                {{ variant.index }} => {{ enum.name }}.{{ variant.name }},
{%- endfor %}
                _ => throw new UnknownVariantException("{{ enum.name }}", variantIndex),
            };
        }
    }
}
"###;
