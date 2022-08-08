use crate::resolver::TypeResolver;
use heck::ToLowerCamelCase as _;
use mech3ax_metadata_types::{ComplexType, SimpleType, TypeInfo, TypeSemantic};
use serde::Serialize;

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

fn lookup_semantic(resolver: &TypeResolver, name: &str) -> (String, DataType) {
    let n = name.to_string();
    if let Some(s) = resolver.lookup_struct(name) {
        // our "structs" can either be
        // * a C# `class` (reference type)
        // * a C# `struct` (value type)
        let data_type = match s.semantic {
            TypeSemantic::Ref => DataType::Ref,
            TypeSemantic::Val => DataType::Val,
        };
        return (n, data_type);
    }
    if let Some(_) = resolver.lookup_enum(name) {
        // our "enums" are a C# enum, which are a C# `struct` (value type)
        return (n, DataType::Val);
    }
    if let Some(_) = resolver.lookup_union(name) {
        // our "unions" get transformed into C# classes (reference type)
        return (n, DataType::Ref);
    }
    eprintln!("WARNING: type `{}` not found", name);
    // Assume a C# class (reference type)
    (n, DataType::Ref)
}

fn complex_type(outer: &ComplexType, resolver: &TypeResolver) -> (String, DataType) {
    match outer {
        ComplexType::Simple(inner) => {
            let (name, data_type) = simple_type(inner);
            (name.to_string(), data_type)
        }
        ComplexType::Struct(name) => lookup_semantic(resolver, name),
        ComplexType::Vec(inner) => {
            let (name, _data_type) = complex_type(inner, resolver);
            // a vec/list is always a reference type, and has it's own
            // nullability
            (format!("List<{}>", name), DataType::Ref)
        }
        ComplexType::Option(inner) => {
            let (name, data_type) = complex_type(inner, resolver);
            match data_type {
                DataType::Val => (format!("{}?", name), DataType::ValNull),
                DataType::ValNull => {
                    eprintln!(
                        "WARNING: doubly-nullable value type `{}` from `{:?}`",
                        name, outer
                    );
                    (name, DataType::ValNull)
                }
                DataType::Ref => (format!("{}?", name), DataType::RefNull),
                DataType::RefNull => {
                    eprintln!(
                        "WARNING: doubly-nullable ref type `{}` from `{:?}`",
                        name, outer
                    );
                    (name, DataType::RefNull)
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
}

impl Field {
    pub fn new<'a>(ti: &'a TypeInfo<'a>, resolver: &TypeResolver) -> Self {
        let TypeInfo { name, ty } = ti;
        let json_name = name.to_string();
        let name = name.to_lower_camel_case();

        let (ty, data_type) = complex_type(ty, resolver);
        let null_check = match data_type {
            DataType::Ref => true,
            _ => false,
        };

        Self {
            json_name,
            name,
            ty,
            null_check,
        }
    }
}
