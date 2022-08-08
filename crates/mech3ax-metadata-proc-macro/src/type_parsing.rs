use crate::attr_parsing::ObjectJsonAttr;
use mech3ax_metadata_types::{ComplexTypeOwned, SimpleType, TypeInfoOwned};
use syn::{
    AngleBracketedGenericArguments, Attribute, Error, GenericArgument, Ident, Path, PathArguments,
    PathSegment, Result, Type, TypePath, TypeTuple,
};

fn parse_simple_type(ident: &Ident) -> ComplexTypeOwned {
    let name = ident.to_string();
    match name.as_str() {
        "bool" => ComplexTypeOwned::Simple(SimpleType::Bool),
        "u8" => ComplexTypeOwned::Simple(SimpleType::U8),
        "u16" => ComplexTypeOwned::Simple(SimpleType::U16),
        "u32" => ComplexTypeOwned::Simple(SimpleType::U32),
        // "u64" => Ok(SimpleKnownTypes::U64),
        "i8" => ComplexTypeOwned::Simple(SimpleType::I8),
        "i16" => ComplexTypeOwned::Simple(SimpleType::I16),
        "i32" => ComplexTypeOwned::Simple(SimpleType::I32),
        // "i64" => Ok(SimpleKnownTypes::I64),
        "f32" => ComplexTypeOwned::Simple(SimpleType::F32),
        // "f64" => Ok(SimpleKnownTypes::F64),
        "String" => ComplexTypeOwned::Simple(SimpleType::String),
        "OffsetDateTime" => ComplexTypeOwned::Simple(SimpleType::DateTime),
        _ => ComplexTypeOwned::Struct(name),
    }
}

fn unwrap_generic_arg(json: &ObjectJsonAttr, arg: &GenericArgument) -> Result<ComplexTypeOwned> {
    match arg {
        GenericArgument::Type(ty) => parse_complex_type(json, ty),
        GenericArgument::Binding(binding) => Err(Error::new_spanned(
            arg,
            format!("Can't derive metadata for binding `{:?}`", binding),
        )),
        GenericArgument::Const(expr) => Err(Error::new_spanned(
            arg,
            format!("Can't derive metadata for constant `{:?}`", expr),
        )),
        GenericArgument::Constraint(constraint) => Err(Error::new_spanned(
            arg,
            format!("Can't derive metadata for constraint `{:?}`", constraint),
        )),
        GenericArgument::Lifetime(lifetime) => Err(Error::new_spanned(
            arg,
            format!("Can't derive metadata for lifetime `{:?}`", lifetime),
        )),
    }
}

fn parse_option_type_inner(ty: ComplexTypeOwned) -> ComplexTypeOwned {
    ComplexTypeOwned::Option(Box::new(ty))
}

fn parse_vec_type_inner(ty: ComplexTypeOwned) -> ComplexTypeOwned {
    match ty {
        ComplexTypeOwned::Simple(inner) if inner == SimpleType::U8 => {
            ComplexTypeOwned::Simple(SimpleType::Bytes)
        }
        _ => ComplexTypeOwned::Vec(Box::new(ty)),
    }
}

fn parse_path_type(json: &ObjectJsonAttr, path: &TypePath) -> Result<ComplexTypeOwned> {
    let TypePath { qself: _, path } = path;
    let Path {
        leading_colon: _,
        segments,
    } = path;

    let segment = match segments.iter().collect::<Vec<&PathSegment>>()[..] {
        [segment] => Ok(segment),
        _ => Err(Error::new_spanned(
            segments,
            format!("Can't derive metadata for segmented path `{:?}`", segments),
        )),
    }?;

    let PathSegment { ident, arguments } = segment;
    match arguments {
        PathArguments::None => Ok(parse_simple_type(ident)),
        PathArguments::AngleBracketed(angled) => {
            let AngleBracketedGenericArguments {
                colon2_token: _,
                lt_token: _,
                args,
                gt_token: _,
            } = angled;
            let name = ident.to_string();
            match name.as_str() {
                "Vec" => match args.iter().collect::<Vec<&GenericArgument>>()[..] {
                    [arg] => unwrap_generic_arg(json, arg).map(parse_vec_type_inner),
                    _ => Err(Error::new_spanned(
                        segment,
                        format!("Expected single generic argument, but got `{:?}`", args),
                    )),
                },
                "Option" => match args.iter().collect::<Vec<&GenericArgument>>()[..] {
                    [arg] => unwrap_generic_arg(json, arg).map(parse_option_type_inner),
                    _ => Err(Error::new_spanned(
                        segment,
                        format!("Expected single generic argument, but got `{:?}`", args),
                    )),
                },
                _ => Err(Error::new_spanned(
                    segment,
                    format!(
                        "Can't derive metadata for unknown generic type `{:?}`",
                        ident
                    ),
                )),
            }
        }
        PathArguments::Parenthesized(parenthesized) => Err(Error::new_spanned(
            segment,
            format!(
                "Can't derive metadata for function pointer `{:?}`",
                parenthesized
            ),
        )),
    }
}

fn parse_tuple_type(json: &ObjectJsonAttr, tuple: &TypeTuple) -> Result<ComplexTypeOwned> {
    let name = json.tuple.clone().ok_or_else(|| {
        Error::new_spanned(tuple, "Can't derive metadata for tuple, missing annotation")
    })?;
    Ok(ComplexTypeOwned::Struct(name))
}

fn parse_complex_type(json: &ObjectJsonAttr, ty: &Type) -> Result<ComplexTypeOwned> {
    match ty {
        Type::Path(path) => parse_path_type(json, path),
        Type::Tuple(tuple) => parse_tuple_type(json, tuple),
        _ => Err(Error::new_spanned(
            ty,
            format!("Can't derive metadata for type `{:?}`", ty),
        )),
    }
}

pub fn parse_type_info(ident: Ident, attrs: &[Attribute], ty: &Type) -> Result<TypeInfoOwned> {
    let json = ObjectJsonAttr::parse_from_attrs(attrs)?;
    let ty = parse_complex_type(&json, ty)?;
    Ok(TypeInfoOwned {
        name: ident.to_string(),
        ty,
    })
}
