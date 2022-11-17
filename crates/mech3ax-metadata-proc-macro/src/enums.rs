use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::{
    parse_quote, Data, DataEnum, DeriveInput, Error, Ident, ImplItemConst, ItemImpl, Result,
    Variant,
};

macro_rules! cannot_derive {
    ($ident:expr, $for:expr) => {
        Err(Error::new_spanned(
            $ident,
            concat!("Can't derive enum metadata for ", $for),
        ))
    };
}

#[derive(Debug)]
struct EnumInfo {
    pub ident: Ident,
    pub name: String,
    pub variants: Vec<String>,
}

fn parse_variant(variant: Variant) -> Result<String> {
    let Variant {
        attrs: _,
        ident,
        fields,
        discriminant: _,
    } = variant;
    if !fields.is_empty() {
        return cannot_derive!(&ident, "a variant with fields");
    }
    Ok(ident.to_string())
}

fn parse_enum(ident: Ident, data: DataEnum) -> Result<EnumInfo> {
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
    Ok(EnumInfo {
        ident,
        name,
        variants,
    })
}

fn generate_enum_type(name: String, variants: Vec<String>) -> ImplItemConst {
    parse_quote! {
        const TYPE_INFO: &'static ::mech3ax_metadata_types::TypeInfo =
            &::mech3ax_metadata_types::TypeInfo::Enum(::mech3ax_metadata_types::TypeInfoEnum {
                name: #name,
                variants: &[
                    #(#variants,)*
                ],
            });
    }
}

fn generate_enum(info: EnumInfo) -> ItemImpl {
    let EnumInfo {
        ident,
        name,
        variants,
    } = info;
    let type_type = generate_enum_type(name, variants);
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
            let info = parse_enum(ident, data)?;
            Ok(generate_enum(info).into_token_stream())
        }
        Data::Struct(_) => cannot_derive!(&ident, "a struct"),
        Data::Union(_) => cannot_derive!(&ident, "a union"),
    }
}
