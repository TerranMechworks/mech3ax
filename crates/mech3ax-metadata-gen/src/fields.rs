use crate::resolver::TypeResolver;
use heck::ToLowerCamelCase as _;
use mech3ax_metadata_types::{DefaultHandling, TypeInfo, TypeInfoStructField};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize)]
pub struct Field {
    /// The struct field's JSON key name.
    pub json_name: String,
    /// The struct field's C# field name.
    pub name: String,
    /// The struct field's C# type, with generics.
    pub ty: String,
    /// Whether the type requires a null check on deserialization.
    pub null_check: bool,
    /// The struct field's default value, if any.
    pub default: Option<String>,
    /// The struct field type generic parameters.
    ///
    /// The templates mainly care if this is `Some(_)` or `None`, but the
    /// struct also needs to aggregate generic parameters from all fields.
    pub generics: Option<HashSet<&'static str>>,
}

fn rust_to_csharp_field_name(name: &str) -> String {
    match name {
        // TODO: add more reserved keywords
        "static_" => "static_".to_string(), // this breaks heck
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

pub fn join_generics(generics: &HashSet<&'static str>) -> String {
    // sort the generics by name for a nice, stable display order.
    let mut generics_sorted: Vec<&'static str> = generics.iter().copied().collect();
    generics_sorted.sort();
    generics_sorted.join(", ")
}

impl Field {
    pub fn new(
        struct_name: &'static str,
        struct_generics: Option<&'static [(&'static TypeInfo, &'static str)]>,
        field_info: &TypeInfoStructField,
        resolver: &TypeResolver,
    ) -> Self {
        // the JSON field name should match the type name, barring any serde
        // rename shenanigans.
        let json_name = field_info.name.to_string();
        // sadly, Rust's casing for field names doesn't match C#.
        let csharp_name = rust_to_csharp_field_name(field_info.name);

        let ty = resolver.resolve(field_info.type_info, struct_name);
        let null_check = ty.null_check();
        let mut type_name = ty.name.to_string();
        let mut field_generics = ty.generics;

        // this is kinda janky, and won't work for nested generics (e.g. Vec<T>)
        // however, the types use generics sparingly, and so this is better
        // than having to plumb the generics code through the resolver.
        if let Some(generics) = struct_generics {
            if let Some(rename) = find_generic_type(field_info, generics) {
                assert!(
                    field_generics.is_none(),
                    "field `{}` type cannot be made generic if it is already generic: {:#?}",
                    field_info.name,
                    field_generics
                );
                field_generics = hashset_generics(rename);
                type_name = rename.to_string();
            }
        }

        let default = match field_info.default {
            DefaultHandling::Normal => None,
            DefaultHandling::OptionIsNone => Some("null".to_string()),
            DefaultHandling::BoolFalse => Some("false".to_string()),
            DefaultHandling::PointerZero => Some("0".to_string()),
        };

        Self {
            json_name,
            name: csharp_name,
            ty: type_name,
            null_check,
            default,
            generics: field_generics,
        }
    }
}
