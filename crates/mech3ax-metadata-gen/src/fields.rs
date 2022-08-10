use crate::resolver::TypeResolver;
use heck::ToLowerCamelCase as _;
use mech3ax_metadata_types::{ComplexType, DefaultHandling, SimpleType, TypeInfo, TypeSemantic};
use serde::Serialize;
use std::collections::HashSet;

// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/integral-numeric-types
// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/floating-point-numeric-types
// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/bool
// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/char
// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/enum
// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/struct
// https://docs.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/value-tuples
#[derive(Debug, Clone, Copy, PartialEq)]
enum DataType {
    Val,
    ValNull,
    Ref,
    RefNull,
}

fn simple_type(ty: &SimpleType) -> (&'static str, DataType) {
    match ty {
        SimpleType::Bool => ("bool", DataType::Val),
        SimpleType::U8 => ("byte", DataType::Val),
        SimpleType::U16 => ("ushort", DataType::Val),
        SimpleType::U32 => ("uint", DataType::Val),
        // SimpleType::U64 => ("ulong", DataType::Val),
        SimpleType::I8 => ("sbyte", DataType::Val),
        SimpleType::I16 => ("short", DataType::Val),
        SimpleType::I32 => ("int", DataType::Val),
        // SimpleType::I64 => ("long", DataType::Val),
        SimpleType::F32 => ("float", DataType::Val),
        // SimpleType::F64 => ("double", DataType::Val),
        SimpleType::String => ("string", DataType::Ref),
        SimpleType::Bytes => ("byte[]", DataType::Ref),
        SimpleType::DateTime => ("DateTime", DataType::Val),
    }
}

pub type Generics = Option<HashSet<(&'static str, &'static str)>>;

fn lookup_semantic(
    name: &str,
    parent_generics: &HashSet<(&'static str, &'static str)>,
    resolver: &TypeResolver,
) -> (String, Generics, DataType) {
    if let Some(s) = resolver.lookup_struct(name) {
        // our "structs" can either be
        // * a C# `class` (reference type)
        // * a C# `struct` (value type)
        let data_type = match s.semantic {
            TypeSemantic::Ref => DataType::Ref,
            TypeSemantic::Val => DataType::Val,
        };
        let (name, generics) = match &s.generics {
            None => {
                // check if the parent makes this struct generic (in which case
                // it needs to be remapped)
                parent_generics
                    .iter()
                    .find_map(|(concrete, generic)| {
                        if &name == concrete {
                            Some((generic.to_string(), Some(parent_generics.clone())))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| (name.to_string(), None))
            }
            Some(gen) => {
                let mut generics_split = gen
                    .iter()
                    .map(|(_concrete, generic)| *generic)
                    .collect::<Vec<&str>>();
                generics_split.sort();
                let generics_joined = generics_split.join(", ");
                let name = format!("{}<{}>", name, generics_joined);
                (name, s.generics.clone())
            }
        };
        return (name, generics, data_type);
    }
    let n = name.to_string();
    if let Some(_) = resolver.lookup_enum(name) {
        // our "enums" are a C# enum, which are a C# `struct` (value type)
        return (n, None, DataType::Val);
    }
    if let Some(_) = resolver.lookup_union(name) {
        // our "unions" get transformed into C# classes (reference type)
        return (n, None, DataType::Ref);
    }
    eprintln!("WARNING: type `{}` not found", name);
    // Assume a C# class (reference type)
    (n, None, DataType::Ref)
}

fn complex_type(
    outer: &ComplexType,
    parent_generics: &HashSet<(&'static str, &'static str)>,
    resolver: &TypeResolver,
) -> (String, Generics, DataType) {
    match outer {
        ComplexType::Simple(inner) => {
            let (name, data_type) = simple_type(inner);
            (name.to_string(), None, data_type)
        }
        ComplexType::Struct(name) => lookup_semantic(name, parent_generics, resolver),
        ComplexType::Vec(inner) => {
            let (name, generics, _data_type) = complex_type(inner, parent_generics, resolver);
            // a vec/list is always a reference type, and has it's own
            // nullability
            (format!("List<{}>", name), generics, DataType::Ref)
        }
        ComplexType::Option(inner) => {
            let (name, generics, data_type) = complex_type(inner, parent_generics, resolver);
            match data_type {
                DataType::Val => (format!("{}?", name), generics, DataType::ValNull),
                DataType::ValNull => {
                    eprintln!(
                        "WARNING: doubly-nullable value type `{}` from `{:?}`",
                        name, outer
                    );
                    (name, generics, DataType::ValNull)
                }
                DataType::Ref => (format!("{}?", name), generics, DataType::RefNull),
                DataType::RefNull => {
                    eprintln!(
                        "WARNING: doubly-nullable ref type `{}` from `{:?}`",
                        name, outer
                    );
                    (name, generics, DataType::RefNull)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub json_name: String,
    pub name: String,
    pub ty: String,
    pub null_check: bool,
    pub default: Option<String>,
    pub generics: Generics,
}

impl Field {
    pub fn new(
        ti: &TypeInfo<'_>,
        parent_generics: &HashSet<(&'static str, &'static str)>,
        resolver: &TypeResolver,
    ) -> Self {
        let TypeInfo { name, ty, default } = ti;
        let json_name = name.to_string();
        let name = name.to_lower_camel_case();

        let (ty, generics, data_type) = complex_type(ty, parent_generics, resolver);
        let null_check = match data_type {
            DataType::Ref => true,
            _ => false,
        };
        let default = match default {
            DefaultHandling::Normal => None,
            DefaultHandling::OptionIsNone => Some("null".to_string()),
            DefaultHandling::BoolFalse => Some("false".to_string()),
            DefaultHandling::PointerZero => Some("0".to_string()),
        };

        Self {
            json_name,
            name,
            ty,
            null_check,
            default,
            generics,
        }
    }
}
