use mech3ax_metadata_types::{ComplexTypeOwned, SimpleType, TypeInfoOwned};
use syn::{parse_quote, ExprCall, ExprPath, ExprStruct};

fn generate_simple_type(ty: SimpleType) -> ExprPath {
    match ty {
        SimpleType::Bool => parse_quote! { ::mech3ax_metadata_types::SimpleType::Bool },
        SimpleType::U8 => parse_quote! { ::mech3ax_metadata_types::SimpleType::U8 },
        SimpleType::U16 => parse_quote! { ::mech3ax_metadata_types::SimpleType::U16 },
        SimpleType::U32 => parse_quote! { ::mech3ax_metadata_types::SimpleType::U32 },
        // SimpleType::U64 => parse_quote! { ::mech3ax_metadata_types::SimpleType::U64 },
        SimpleType::I8 => parse_quote! { ::mech3ax_metadata_types::SimpleType::I8 },
        SimpleType::I16 => parse_quote! { ::mech3ax_metadata_types::SimpleType::I16 },
        SimpleType::I32 => parse_quote! { ::mech3ax_metadata_types::SimpleType::I32 },
        // SimpleType::I64 => parse_quote! { ::mech3ax_metadata_types::SimpleType::I64 },
        SimpleType::F32 => parse_quote! { ::mech3ax_metadata_types::SimpleType::F32 },
        // SimpleType::F64 => parse_quote! { ::mech3ax_metadata_types::SimpleType::F64 },
        SimpleType::String => parse_quote! { ::mech3ax_metadata_types::SimpleType::String },
        SimpleType::Bytes => parse_quote! { ::mech3ax_metadata_types::SimpleType::Bytes },
        SimpleType::DateTime => parse_quote! { ::mech3ax_metadata_types::SimpleType::DateTime },
    }
}

fn generate_complex_type(ty: ComplexTypeOwned) -> ExprCall {
    match ty {
        ComplexTypeOwned::Simple(ty) => {
            let inner = generate_simple_type(ty);
            parse_quote! { ::mech3ax_metadata_types::ComplexType::Simple(#inner) }
        }
        ComplexTypeOwned::Struct(name) => {
            parse_quote! {
                ::mech3ax_metadata_types::ComplexType::Struct(#name)
            }
        }
        ComplexTypeOwned::Vec(ty) => {
            let inner = generate_complex_type(*ty);
            parse_quote! {
                ::mech3ax_metadata_types::ComplexType::Vec(& #inner)
            }
        }
        ComplexTypeOwned::Option(ty) => {
            let inner = generate_complex_type(*ty);
            parse_quote! {
                ::mech3ax_metadata_types::ComplexType::Option(& #inner)
            }
        }
    }
}

fn generate_type_info(field: TypeInfoOwned) -> ExprStruct {
    let TypeInfoOwned { name, ty } = field;
    let ty = generate_complex_type(ty);
    // let default: syn::Expr = match default {
    //     None => parse_quote! { None },
    //     Some(d) => parse_quote! { Some(#d) },
    // };
    parse_quote! {
        ::mech3ax_metadata_types::TypeInfo {
            name: #name,
            ty: #ty,
        }
    }
}

pub fn generate_type_infos(fields: Vec<TypeInfoOwned>) -> Vec<ExprStruct> {
    fields.into_iter().map(generate_type_info).collect()
}
