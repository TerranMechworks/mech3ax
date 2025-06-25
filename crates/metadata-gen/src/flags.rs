use crate::csharp_type::{CSharpType, SerializeType, TypeKind};
use crate::module_path::{dotnet_namespace_to_path, rust_mod_path_to_dotnet};
use crate::resolver::TypeResolver;
use heck::ToUpperCamelCase as _;
use mech3ax_metadata_types::{TypeInfoFlags, TypeInfoFlagsRepr};
use minijinja::{context, Environment};
use serde::Serialize;
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Variant {
    /// The flags variant's name.
    pub(crate) name: String,
    /// The flags variant's index.
    pub(crate) index: u32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Flags {
    /// The flags's C# type name.
    pub(crate) name: &'static str,
    /// The flags's C# namespace.
    pub(crate) namespace: String,
    /// The flags's full C# type, with namespace.
    pub(crate) full_name: String,
    /// The flag's C# (de)serialization type.
    pub(crate) serde_type: &'static str,
    /// The flag's C# base type.
    pub(crate) base_type: &'static str,
    /// The flags's C# variant names.
    pub(crate) variants: Vec<Variant>,
    /// The flags's path on the filesystem.
    pub(crate) path: PathBuf,
}

impl Flags {
    pub(crate) fn make_type(&self) -> CSharpType {
        // our "flags" are a C# enum, which are a C# `struct` (value type)
        CSharpType {
            name: Cow::Owned(self.full_name.clone()),
            kind: TypeKind::Val,
            generics: None,
            serde: SerializeType::Flags(self.full_name.clone()),
        }
    }

    pub(crate) fn new(resolver: &mut TypeResolver, fi: &TypeInfoFlags) -> Self {
        // luckily, Rust's casing for enum and variant names matches C#.
        let name = fi.name;
        let namespace = rust_mod_path_to_dotnet(fi.module_path);
        let full_name = format!("{}.{}", namespace, fi.name);

        let (serde_type, base_type) = match fi.repr {
            TypeInfoFlagsRepr::U8 => ("U8", "byte"),
            TypeInfoFlagsRepr::U16 => ("U16", "ushort"),
            TypeInfoFlagsRepr::U32 => ("U32", "uint"),
        };

        let variants = fi
            .variants
            .iter()
            .copied()
            .map(|(name, index)| Variant {
                name: name.to_upper_camel_case(),
                index,
            })
            .collect();

        let mut path = dotnet_namespace_to_path(&namespace);
        resolver.add_directory(&path);
        path.push(format!("{}.cs", name));

        Self {
            name,
            namespace,
            full_name,
            serde_type,
            base_type,
            variants,
            path,
        }
    }

    pub(crate) fn render_impl(&self, env: &Environment<'_>) -> Result<String, minijinja::Error> {
        let template = env.get_template("flags_impl.cs")?;
        template.render(context! { flags => self })
    }
}

pub(crate) const FLAGS_IMPL: &str = r#"using Mech3DotNet.Exchange;

namespace {{ flags.namespace }}
{
    [System.Flags]
    public enum {{ flags.name }} : {{ flags.base_type }}
    {
        None = 0u,
{%- for variant in flags.variants %}
        {{ variant.name }} = 1u << {{ variant.index }},
{%- endfor %}
    }

    public static class {{ flags.name }}Converter
    {
        public static readonly TypeConverter<{{ flags.name }}> Converter = new TypeConverter<{{ flags.name }}>(Deserialize, Serialize);

        private const {{ flags.name }} VALID = ({{ flags.name }})(0u{% for variant in flags.variants %} | 1u << {{ variant.index }}{% endfor %});

        private static void Serialize({{ flags.name }} v, Serializer s)
        {
            if ((v & ~VALID) != 0)
                throw new System.ArgumentOutOfRangeException();
            s.Serialize{{ flags.serde_type }}(({{ flags.base_type }})v);
        }

        private static {{ flags.name }} Deserialize(Deserializer d)
        {
            return ({{ flags.name }})d.Deserialize{{ flags.serde_type }}();
        }
    }
}
"#;
