use super::csharp_type::SerializeType;
use super::resolver::TypeResolver;
use heck::ToLowerCamelCase as _;
use mech3ax_metadata_types::{DefaultHandling, TypeInfo, TypeInfoStructField};
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FieldSerde {
    pub(crate) serialize: String,
    pub(crate) deserialize: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Field {
    /// The struct field's JSON key name.
    pub(crate) key: String,
    /// The struct field's C# field name.
    pub(crate) name: String,
    /// The struct field's C# type, with generics.
    pub(crate) ty: String,
    /// The struct field's serialization information.
    pub(crate) serde: FieldSerde,
    /// The struct field's default value, if any.
    pub(crate) default: Option<String>,
    /// The struct field type generic parameters.
    ///
    /// The templates mainly care if this is `Some(_)` or `None`, but the
    /// struct also needs to aggregate generic parameters from all fields.
    pub(crate) generics: Option<HashSet<&'static str>>,
}

fn rust_to_csharp_field_name(name: &str) -> String {
    match name {
        // TODO: add more reserved keywords
        "static_" => "static_".to_string(),  // this breaks heck
        "base" => "base_".to_string(),       // this breaks C#
        "default" => "default_".to_string(), // this breaks C#
        other => other.to_lower_camel_case(),
    }
}

fn find_generic_type(
    field_info: &TypeInfoStructField,
    generics: &'static [(&'static TypeInfo, &'static str)],
) -> Option<&'static str> {
    generics.iter().copied().find_map(|(type_info, rename)| {
        if type_info == field_info.type_info {
            Some(rename)
        } else {
            None
        }
    })
}

fn hashset_generics(rename: &'static str) -> Option<HashSet<&'static str>> {
    let mut h = HashSet::with_capacity(1);
    h.insert(rename);
    Some(h)
}

pub(crate) fn sort_generics(generics: &HashSet<&'static str>) -> Vec<&'static str> {
    // sort the generics by name for a nice, stable display order.
    let mut generics_sorted: Vec<&'static str> = generics.iter().copied().collect();
    generics_sorted.sort();
    generics_sorted
}

impl Field {
    pub(crate) fn new(
        struct_name: &'static str,
        struct_generics: Option<&'static [(&'static TypeInfo, &'static str)]>,
        field_info: &TypeInfoStructField,
        resolver: &TypeResolver,
    ) -> Self {
        // the JSON field name should match the type name, barring any serde
        // rename shenanigans.
        let key = field_info.name.to_string();
        // sadly, Rust's casing for field names doesn't match C#.
        let csharp_name = rust_to_csharp_field_name(field_info.name);

        let mut ty = resolver.resolve(field_info.type_info, struct_name);

        // this is kinda janky, and won't work for nested generics (e.g. Vec<T>)
        // however, the types use generics sparingly, and so this is better
        // than having to plumb the generics code through the resolver.
        if let Some(generics) = struct_generics {
            if let Some(rename) = find_generic_type(field_info, generics) {
                assert!(
                    ty.generics.is_none(),
                    "field `{}` type cannot be made generic if it is already generic: {:#?}",
                    field_info.name,
                    ty.generics
                );
                ty.generics = hashset_generics(rename);
                ty.name = Cow::Borrowed(rename);
                ty.serde = SerializeType::Generic(rename);
            }
        }
        let serde = FieldSerde {
            serialize: ty.serde.make_serialize(),
            deserialize: ty.serde.make_deserialize(),
        };

        let default = match field_info.default {
            DefaultHandling::Normal => None,
            DefaultHandling::OptionIsNone => Some("null".to_string()),
            DefaultHandling::BoolFalse => Some("false".to_string()),
            DefaultHandling::BoolTrue => Some("true".to_string()),
            DefaultHandling::PointerZero => Some("0".to_string()),
            DefaultHandling::SoilIsDefault => {
                Some("Mech3DotNet.Types.Gamez.Materials.Soil.Default".to_string())
            }
            DefaultHandling::I32IsNegOne => Some("-1".to_string()),
        };

        Self {
            key,
            name: csharp_name,
            ty: ty.name.to_string(),
            serde,
            default,
            generics: ty.generics,
        }
    }
}
