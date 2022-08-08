use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::{
    parse_quote, Data, DataEnum, DeriveInput, Error, ExprTuple, Field, Fields, FieldsUnnamed,
    Generics, Ident, ImplItemConst, ItemImpl, Result, Type, TypePath, Variant,
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
    ForeignStruct {
        variant_name: String,
        struct_name: String,
    },
    Unit {
        variant_name: String,
    },
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
            variant_name: ident.to_string(),
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
                    } = field;
                    if let Some(field_ident) = field_ident {
                        return Err(Error::new_spanned(
                            &field_ident,
                            "Expected field to have no identifier",
                        ));
                    }
                    match ty {
                        Type::Path(path) => {
                            let TypePath { qself: _, path } = path;
                            Ok(VariantInfo::ForeignStruct {
                                variant_name: ident.to_string(),
                                struct_name: path.get_ident().unwrap().to_string(),
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

fn generate_union_name(name: String) -> ImplItemConst {
    parse_quote! {
        const NAME: &'static str = #name;
    }
}

fn generate_union_variant(variant: VariantInfo) -> ExprTuple {
    match variant {
        VariantInfo::Unit { variant_name } => {
            parse_quote! {
                ( #variant_name, None )
            }
        }
        VariantInfo::ForeignStruct {
            variant_name,
            struct_name,
        } => {
            parse_quote! {
                ( #variant_name, Some(#struct_name) )
            }
        }
    }
}

fn generate_union_variants(variants: Vec<VariantInfo>) -> ImplItemConst {
    let variants: Vec<ExprTuple> = variants.into_iter().map(generate_union_variant).collect();
    parse_quote! {
        const VARIANTS: &'static [(&'static str, Option<&'static str>)] = &[
            #( #variants, )*
        ];
    }
}

fn generate_union(info: UnionInfo, union_generics: Generics) -> ItemImpl {
    let UnionInfo {
        ident,
        name,
        variants,
    } = info;
    let name = generate_union_name(name);
    let variants = generate_union_variants(variants);
    parse_quote! {
        impl #union_generics ::mech3ax_metadata_types::Union for #ident #union_generics {
            #name
            #variants
        }
    }
}

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident,
        generics,
        data,
    } = input;
    match data {
        Data::Enum(data) => {
            let info = parse_union(ident, data)?;
            Ok(generate_union(info, generics).into_token_stream())
        }
        Data::Struct(_) => cannot_derive!(&ident, "a struct"),
        Data::Union(_) => cannot_derive!(&ident, "a union"),
    }
}
