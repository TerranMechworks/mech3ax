use crate::attr_parsing::{parse_repr_attr, ReprType};
use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::{
    parse_quote, Attribute, BinOp, Data, DataEnum, DeriveInput, Error, Expr, ExprBinary, ExprLit,
    Ident, ImplItemConst, ItemImpl, Lit, Path, Result, Variant,
};

macro_rules! cannot_derive {
    ($ident:expr, $for:expr) => {
        Err(Error::new_spanned(
            $ident,
            concat!("Can't derive flags metadata for ", $for),
        ))
    };
}

#[derive(Debug)]
struct FlagsInfo {
    pub(crate) ident: Ident,
    pub(crate) name: String,
    pub(crate) repr: ReprType,
    pub(crate) variants: Vec<(String, u32)>,
}

fn parse_discriminant(discriminant: Expr) -> Result<u32> {
    match discriminant {
        Expr::Binary(ExprBinary {
            attrs: _,
            left,
            op,
            right,
        }) => {
            match *left {
                Expr::Lit(ExprLit { attrs: _, lit }) => match lit {
                    Lit::Int(literal) if literal.base10_digits() == "1" => {}
                    other => return Err(Error::new_spanned(other, "invalid discriminant")),
                },
                other => return Err(Error::new_spanned(other, "invalid discriminant")),
            }
            if !matches!(op, BinOp::Shl(_)) {
                return Err(Error::new_spanned(op, "invalid discriminant"));
            }
            match *right {
                Expr::Lit(ExprLit { attrs: _, lit }) => match lit {
                    Lit::Int(literal) => literal.base10_parse(),
                    other => Err(Error::new_spanned(other, "invalid discriminant")),
                },
                other => Err(Error::new_spanned(other, "invalid discriminant")),
            }
        }
        other => Err(Error::new_spanned(other, "invalid discriminant")),
    }
}

fn parse_variant(variant: Variant) -> Result<(String, u32)> {
    let Variant {
        attrs: _,
        ident,
        fields,
        discriminant,
    } = variant;
    if !fields.is_empty() {
        return cannot_derive!(&ident, "a variant with fields");
    }
    let Some((_eq, discriminant)) = discriminant else {
        return cannot_derive!(&ident, "a variant without a discriminant");
    };
    let index = parse_discriminant(discriminant)?;
    Ok((ident.to_string(), index))
}

fn parse_enum(ident: Ident, data: DataEnum, attrs: &[Attribute]) -> Result<FlagsInfo> {
    let DataEnum {
        enum_token,
        brace_token: _,
        variants,
    } = data;
    let name = ident.to_string();
    let repr = parse_repr_attr(&enum_token, attrs)?;
    let variants = variants
        .into_iter()
        .map(parse_variant)
        .collect::<Result<Vec<_>>>()?;
    Ok(FlagsInfo {
        ident,
        name,
        repr,
        variants,
    })
}

fn generate_repr_type(repr: ReprType) -> Path {
    match repr {
        ReprType::U8 => parse_quote! { ::mech3ax_metadata_types::TypeInfoFlagsRepr::U8 },
        ReprType::U16 => parse_quote! { ::mech3ax_metadata_types::TypeInfoFlagsRepr::U16 },
        ReprType::U32 => parse_quote! { ::mech3ax_metadata_types::TypeInfoFlagsRepr::U32 },
    }
}

fn generate_flags_type(
    name: String,
    repr: ReprType,
    variants: Vec<(String, u32)>,
) -> ImplItemConst {
    let (variant_names, variant_indices): (Vec<_>, Vec<_>) = variants.into_iter().unzip();
    let repr = generate_repr_type(repr);
    parse_quote! {
        const TYPE_INFO: &'static ::mech3ax_metadata_types::TypeInfo =
            &::mech3ax_metadata_types::TypeInfo::Flags(::mech3ax_metadata_types::TypeInfoFlags {
                name: #name,
                repr: #repr,
                variants: &[
                    #((#variant_names, #variant_indices),)*
                ],
                module_path: ::std::module_path!(),
            });
    }
}

fn generate_flags(info: FlagsInfo) -> ItemImpl {
    let FlagsInfo {
        ident,
        name,
        repr,
        variants,
    } = info;
    let type_type = generate_flags_type(name, repr, variants);
    parse_quote! {
        impl ::mech3ax_metadata_types::DerivedMetadata for #ident {
            #type_type
        }
    }
}

pub(crate) fn derive(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        vis: _,
        ident,
        generics: _,
        data,
    } = input;
    match data {
        Data::Enum(data) => {
            let info = parse_enum(ident, data, &attrs)?;
            Ok(generate_flags(info).into_token_stream())
        }
        Data::Struct(_) => cannot_derive!(&ident, "a struct"),
        Data::Union(_) => cannot_derive!(&ident, "a union"),
    }
}
