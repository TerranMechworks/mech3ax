use crate::csharp_type::{CSharpType, SerializeType, TypeKind};
use crate::module_path::{rust_mod_path_to_dotnet, rust_mod_path_to_path};
use crate::resolver::TypeResolver;
use mech3ax_metadata_types::{TypeInfo, TypeInfoUnion};
use serde::Serialize;
use std::borrow::Cow;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct VarantSerde {
    pub serialize: String,
    pub deserialize: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Variant {
    /// The union variant's name.
    pub name: &'static str,
    /// The union variant's index.
    pub index: u32,
    /// The union variant's full C# type, with namespace; or the unit variant
    /// class name, without namespace.
    pub type_name: String,
    pub serde: Option<VarantSerde>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Union {
    /// The union's C# type name.
    pub name: &'static str,
    /// The union's C# namespace.
    pub namespace: String,
    /// The union's full C# type, with namespace.
    pub full_name: String,
    /// The union variant types.
    pub variants: Vec<Variant>,
    /// The union's path on the filesystem.
    pub path: PathBuf,
}

fn resolve_variant(
    union_name: &'static str,
    resolver: &TypeResolver,
    variant_name: &'static str,
    variant_index: u32,
    variant_type: Option<&'static TypeInfo>,
) -> Variant {
    // Luckily, Rust's casing for variant names matches C#.
    match variant_type {
        None => Variant {
            name: variant_name,
            index: variant_index,
            type_name: variant_name.to_string(),
            serde: None,
        },
        Some(type_info) => {
            let ty = resolver.resolve(type_info, union_name);
            Variant {
                name: variant_name,
                index: variant_index,
                type_name: ty.name.to_string(),
                serde: Some(VarantSerde {
                    serialize: ty.serde.make_serialize(),
                    deserialize: ty.serde.make_deserialize(),
                }),
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
            serde: SerializeType::Union(self.full_name.clone()),
        }
    }

    pub fn new(resolver: &mut TypeResolver, ui: &TypeInfoUnion) -> Self {
        // luckily, Rust's casing for enum names matches C# classes.
        let name = ui.name;
        let namespace = rust_mod_path_to_dotnet(ui.module_path);
        let full_name = format!("{}.{}", namespace, ui.name);

        let variants = ui
            .variants
            .iter()
            .copied()
            .zip(0u32..)
            .map(|((variant_name, variant_type), variant_index)| {
                resolve_variant(name, resolver, variant_name, variant_index, variant_type)
            })
            .collect();

        let mut path = rust_mod_path_to_path(ui.module_path);
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
        context.insert("union", self);
        tera.render("union_impl.cs", &context)
    }
}

pub const UNION_IMPL: &str = r###"using System;
using Mech3DotNet.Exchange;

namespace {{ union.namespace }}
{
    public sealed class {{ union.name }}
    {
        public static readonly TypeConverter<{{ union.name }}> Converter = new TypeConverter<{{ union.name }}>(Deserialize, Serialize);

        public enum Variants
        {
{%- for variant in union.variants %}
            {{ variant.name }},
{%- endfor %}
        }

        private {{ union.name }}(Variants variant, object value)
        {
            Variant = variant;
            Value = value;
        }

{%- for variant in union.variants %}{% if variant.serde %}
        public static {{ union.name }} {{ variant.name }}({{ variant.type_name }} value) => new {{ union.name }}(Variants.{{ variant.name }}, value);
{% else %}
        public static readonly {{ union.name }} {{ variant.name }} = new {{ union.name }}(Variants.{{ variant.name }}, new object());
{% endif %}{% endfor %}
        public object Value { get; private set; }
        public Variants Variant { get; private set; }
{%- for variant in union.variants %}
        public bool Is{{ variant.name }}() => Variant == Variants.{{ variant.name }};
{%- if variant.serde %}
        public {{ variant.type_name }} As{{ variant.name }}() => ({{ variant.type_name }})Value;
{%- endif %}
{%- endfor %}

        private static void Serialize({{ union.name }} v, Serializer s)
        {
            switch (v.Variant)
            {
{%- for variant in union.variants %}
                case Variants.{{ variant.name }}: // {{ variant.index }}
                    {
{%- if variant.serde %}
                        var inner = v.As{{ variant.name }}();
                        s.SerializeNewTypeVariant({{ variant.index }});
                        {{ variant.serde.serialize }}(inner);
{%- else %}
                        s.SerializeUnitVariant({{ variant.index }});
{%- endif %}
                        break;
                    }
{% endfor %}
                default:
                    throw new System.ArgumentOutOfRangeException();
            }
        }

        private static {{ union.name }} Deserialize(Deserializer d)
        {
            var (enumType, variantIndex) = d.DeserializeEnum();
            switch (variantIndex)
            {
{%- for variant in union.variants %}
                case {{ variant.index }}: // {{ variant.name }}
                    {
{%- if variant.serde %}
                        if (enumType != EnumType.NewType)
                            throw new InvalidVariantException("{{ union.name }}", {{ variant.index }}, EnumType.NewType, enumType);
                        var inner = {{ variant.serde.deserialize }}();
                        return {{ union.name }}.{{ variant.name }}(inner);
{%- else %}
                        if (enumType != EnumType.Unit)
                            throw new InvalidVariantException("{{ union.name }}", {{ variant.index }}, EnumType.Unit, enumType);
                        return {{ union.name }}.{{ variant.name }};
{%- endif %}
                    }
{% endfor %}
                default:
                    throw new UnknownVariantException("{{ union.name }}", variantIndex);
            }
        }
    }
}
"###;
