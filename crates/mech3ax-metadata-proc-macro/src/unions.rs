use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::{
    parse_quote, Data, DataEnum, DeriveInput, Error, ExprTuple, Field, FieldMutability, Fields,
    FieldsUnnamed, Ident, ImplItemConst, ItemImpl, Path, Result, Type, TypePath, Variant,
};

macro_rules! cannot_derive {
    ($ident:expr, $for:expr) => {
        Err(Error::new_spanned(
            $ident,
            concat!("Can't derive union metadata for ", $for),
        ))
    };
}

#[derive(Debug)]
enum VariantInfo {
    ForeignStruct { name: String, path: Path },
    Unit { name: String },
}

#[derive(Debug)]
struct UnionInfo {
    pub ident: Ident,
    pub name: String,
    pub variants: Vec<VariantInfo>,
}

fn parse_variant(variant: Variant) -> Result<VariantInfo> {
    let Variant {
        attrs: _,
        ident,
        fields,
        discriminant: _,
    } = variant;
    match fields {
        Fields::Unit => Ok(VariantInfo::Unit {
            name: ident.to_string(),
        }),
        Fields::Named(_) => cannot_derive!(&ident, "a variant with named fields"),
        Fields::Unnamed(fields_unnamed) => {
            let FieldsUnnamed {
                paren_token: _,
                unnamed,
            } = fields_unnamed;
            let unnamed: Vec<Field> = unnamed.into_iter().collect();
            match &unnamed[..] {
                [field] => {
                    let Field {
                        attrs: _,
                        vis: _,
                        ident: field_ident,
                        colon_token: _,
                        ty,
                        mutability,
                    } = field;
                    if !matches!(mutability, FieldMutability::None) {
                        return Err(Error::new_spanned(
                            ty,
                            "Expected field to have no mutability",
                        ));
                    }
                    if let Some(field_ident) = field_ident {
                        return Err(Error::new_spanned(
                            field_ident,
                            "Expected field to have no identifier",
                        ));
                    }
                    match ty {
                        Type::Path(path) => {
                            let TypePath { qself: _, path } = path;
                            Ok(VariantInfo::ForeignStruct {
                                name: ident.to_string(),
                                path: path.clone(),
                            })
                        }
                        other => cannot_derive!(&other, "a field with a non-path type"),
                    }
                }
                _ => cannot_derive!(&ident, "a variant with multiple unnamed fields"),
            }
        }
    }
}

fn parse_union(ident: Ident, data: DataEnum) -> Result<UnionInfo> {
    let DataEnum {
        enum_token: _,
        brace_token: _,
        variants,
    } = data;
    let name = ident.to_string();
    let variants = variants
        .into_iter()
        .map(parse_variant)
        .collect::<Result<Vec<_>>>()?;
    Ok(UnionInfo {
        ident,
        name,
        variants,
    })
}

fn generate_union_variant(variant: VariantInfo) -> ExprTuple {
    match variant {
        VariantInfo::Unit { name } => {
            parse_quote! {
                ( #name, None )
            }
        }
        VariantInfo::ForeignStruct { name, path } => {
            parse_quote! {
                ( #name, Some(<#path as ::mech3ax_metadata_types::DerivedMetadata>::TYPE_INFO) )
            }
        }
    }
}

fn generate_union_type(name: String, variants: Vec<VariantInfo>) -> ImplItemConst {
    let variants: Vec<_> = variants.into_iter().map(generate_union_variant).collect();
    parse_quote! {
        const TYPE_INFO: &'static ::mech3ax_metadata_types::TypeInfo =
            &::mech3ax_metadata_types::TypeInfo::Union(::mech3ax_metadata_types::TypeInfoUnion {
                name: #name,
                variants: &[
                    #( #variants, )*
                ],
                module_path: ::std::module_path!(),
            });
    }
}

fn generate_union(info: UnionInfo) -> ItemImpl {
    let UnionInfo {
        ident,
        name,
        variants,
    } = info;
    let type_type = generate_union_type(name, variants);
    parse_quote! {
        impl ::mech3ax_metadata_types::DerivedMetadata for #ident {
            #type_type
        }
    }
}

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident,
        generics: _,
        data,
    } = input;
    match data {
        Data::Enum(data) => {
            let info = parse_union(ident, data)?;
            Ok(generate_union(info).into_token_stream())
        }
        Data::Struct(_) => cannot_derive!(&ident, "a struct"),
        Data::Union(_) => cannot_derive!(&ident, "a union"),
    }
}
