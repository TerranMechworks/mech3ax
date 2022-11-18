use crate::attr_parsing::{parse_generic_attr, parse_serde_attr};
use mech3ax_metadata_types::{DefaultHandling, TypeSemantic};
use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::{
    parse_quote, Attribute, Data, DataStruct, DeriveInput, Error, Expr, ExprStruct, Field, Fields,
    FieldsNamed, Generics, Ident, ImplItemConst, ItemImpl, Path, Result, Type,
};

macro_rules! cannot_derive {
    ($ident:expr, $for:expr) => {
        Err(Error::new_spanned(
            $ident,
            concat!("Can't derive struct metadata for ", $for),
        ))
    };
}

#[derive(Debug)]
pub struct FieldInfoOwned {
    pub name: String,
    pub ty: Type,
    pub default: DefaultHandling,
}

#[derive(Debug)]
struct StructInfo {
    pub ident: Ident,
    pub name: String,
    pub semantic: TypeSemantic,
    pub generics: Option<Vec<(Path, String)>>,
    pub fields: Vec<FieldInfoOwned>,
}

fn parse_struct_generics(attrs: &[Attribute]) -> Result<Option<Vec<(Path, String)>>> {
    parse_generic_attr(attrs)
}

fn parse_struct_field(ident: Ident, attrs: &[Attribute], ty: &Type) -> Result<FieldInfoOwned> {
    let default = parse_serde_attr(attrs)?;
    Ok(FieldInfoOwned {
        name: ident.to_string(),
        ty: ty.clone(),
        default,
    })
}

fn parse_struct_fields(fields: FieldsNamed) -> Result<Vec<FieldInfoOwned>> {
    let FieldsNamed {
        brace_token: _,
        named,
    } = fields;

    named
        .into_iter()
        .map(|field| {
            let Field {
                attrs,
                vis: _,
                ident,
                colon_token: _,
                ty,
            } = field;
            let ident = ident
                .ok_or_else(|| Error::new_spanned(&ty, "Expected field to have an identifier"))?;
            parse_struct_field(ident, &attrs, &ty)
        })
        .collect::<Result<Vec<_>>>()
}

fn parse_struct_named(
    ident: Ident,
    fields: FieldsNamed,
    attrs: &[Attribute],
    semantic: TypeSemantic,
) -> Result<StructInfo> {
    let name = ident.to_string();
    let generics = parse_struct_generics(attrs)?;
    let fields = parse_struct_fields(fields)?;

    Ok(StructInfo {
        ident,
        name,
        semantic,
        generics,
        fields,
    })
}

fn parse_struct(
    ident: Ident,
    data: DataStruct,
    attrs: &[Attribute],
    semantic: TypeSemantic,
) -> Result<StructInfo> {
    let DataStruct {
        struct_token: _,
        fields,
        semi_token: _,
    } = data;
    match fields {
        Fields::Named(fields) => parse_struct_named(ident, fields, attrs, semantic),
        Fields::Unnamed(_) => cannot_derive!(&ident, "a tuple struct"),
        Fields::Unit => cannot_derive!(&ident, "a unit struct"),
    }
}

fn generate_struct_semantic(semantic: TypeSemantic) -> Path {
    match semantic {
        TypeSemantic::Ref => parse_quote! { ::mech3ax_metadata_types::TypeSemantic::Ref },
        TypeSemantic::Val => parse_quote! { ::mech3ax_metadata_types::TypeSemantic::Val },
    }
}

fn generate_struct_generics(generics: Option<Vec<(Path, String)>>) -> Expr {
    match generics {
        None => parse_quote! { None },
        Some(generics) => {
            let (path, value): (Vec<_>, Vec<_>) = generics.into_iter().unzip();
            parse_quote! {
                Some(&[
                    #( (<#path as ::mech3ax_metadata_types::DerivedMetadata>::TYPE_INFO, #value), )*
                ])
            }
        }
    }
}

fn generate_struct_field_default(default: DefaultHandling) -> Path {
    match default {
        DefaultHandling::Normal => parse_quote! {
            ::mech3ax_metadata_types::DefaultHandling::Normal
        },
        DefaultHandling::OptionIsNone => parse_quote! {
            ::mech3ax_metadata_types::DefaultHandling::OptionIsNone
        },
        DefaultHandling::BoolFalse => parse_quote! {
            ::mech3ax_metadata_types::DefaultHandling::BoolFalse
        },
        DefaultHandling::PointerZero => parse_quote! {
            ::mech3ax_metadata_types::DefaultHandling::PointerZero
        },
    }
}

fn generate_struct_field(field: FieldInfoOwned) -> ExprStruct {
    let FieldInfoOwned { name, ty, default } = field;
    let default = generate_struct_field_default(default);
    parse_quote! {
        ::mech3ax_metadata_types::TypeInfoStructField {
            name: #name,
            type_info: <#ty as ::mech3ax_metadata_types::DerivedMetadata>::TYPE_INFO,
            default: #default,
        }
    }
}

fn generate_struct_type(
    name: String,
    semantic: TypeSemantic,
    generics: Option<Vec<(Path, String)>>,
    fields: Vec<FieldInfoOwned>,
) -> ImplItemConst {
    let semantic = generate_struct_semantic(semantic);
    let generics = generate_struct_generics(generics);
    let fields: Vec<_> = fields.into_iter().map(generate_struct_field).collect();

    parse_quote! {
        const TYPE_INFO: &'static ::mech3ax_metadata_types::TypeInfo =
            &::mech3ax_metadata_types::TypeInfo::Struct(::mech3ax_metadata_types::TypeInfoStruct {
                name: #name,
                semantic: #semantic,
                generics: #generics,
                fields: &[
                    #( #fields, )*
                ],
                module_path: ::std::module_path!(),
            });
    }
}

fn generate_struct(info: StructInfo, struct_generics: Generics) -> ItemImpl {
    let StructInfo {
        ident,
        name,
        semantic,
        generics,
        fields,
    } = info;
    let type_type = generate_struct_type(name, semantic, generics, fields);
    parse_quote! {
        impl #struct_generics ::mech3ax_metadata_types::DerivedMetadata for #ident #struct_generics {
            #type_type
        }
    }
}

pub fn derive(input: DeriveInput, semantic: TypeSemantic) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        vis: _,
        ident,
        generics,
        data,
    } = input;
    match data {
        Data::Struct(data) => {
            let info = parse_struct(ident, data, &attrs, semantic)?;
            Ok(generate_struct(info, generics).into_token_stream())
        }
        Data::Enum(_) => cannot_derive!(&ident, "an enum"),
        Data::Union(_) => cannot_derive!(&ident, "a union"),
    }
}
