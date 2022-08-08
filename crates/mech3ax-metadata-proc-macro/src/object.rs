use crate::attr_parsing::find_attr;
use crate::type_generation::generate_type_infos;
use crate::type_parsing::parse_type_info;
use mech3ax_metadata_types::TypeInfoOwned;
use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::punctuated::Punctuated;
use syn::{
    parse_quote, Attribute, Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed,
    Generics, Ident, ImplItemConst, ItemImpl, LitStr, Result, Token,
};

macro_rules! cannot_derive {
    ($ident:expr, $for:expr) => {
        Err(Error::new_spanned(
            $ident,
            concat!("Can't derive object metadata for ", $for),
        ))
    };
}

#[derive(Debug)]
struct ObjectInfo {
    pub ident: Ident,
    pub name: String,
    pub generics: Option<Vec<String>>,
    pub fields: Vec<TypeInfoOwned>,
}

fn parse_name(ident: &Ident, attrs: &[Attribute]) -> Result<String> {
    match find_attr(attrs, "rename") {
        Some(attr) => attr.parse_args::<LitStr>().map(|lit| lit.value()),
        None => Ok(ident.to_string()),
    }
}

fn parse_generics(attrs: &[Attribute]) -> Result<Option<Vec<String>>> {
    find_attr(attrs, "generic")
        .map(|attr| {
            attr.parse_args_with(<Punctuated<LitStr, Token![,]>>::parse_terminated)
                .map(|punctuated| punctuated.into_iter().map(|lit| lit.value()).collect())
        })
        .transpose()
}

fn parse_fields(fields: FieldsNamed) -> Result<Vec<TypeInfoOwned>> {
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
            parse_type_info(ident, &attrs, &ty)
        })
        .collect::<Result<Vec<_>>>()
}

fn parse_object_named(
    ident: Ident,
    fields: FieldsNamed,
    attrs: &[Attribute],
) -> Result<ObjectInfo> {
    let name = parse_name(&ident, attrs)?;
    let generics = parse_generics(attrs)?;
    let fields = parse_fields(fields)?;

    Ok(ObjectInfo {
        ident,
        name,
        generics,
        fields,
    })
}

fn parse_object(ident: Ident, data: DataStruct, attrs: &[Attribute]) -> Result<ObjectInfo> {
    let DataStruct {
        struct_token: _,
        fields,
        semi_token: _,
    } = data;
    match fields {
        Fields::Named(fields) => parse_object_named(ident, fields, attrs),
        Fields::Unnamed(_) => cannot_derive!(&ident, "a tuple object"),
        Fields::Unit => cannot_derive!(&ident, "a unit object"),
    }
}

fn generate_object_name(name: String) -> ImplItemConst {
    parse_quote! {
        const NAME: &'static str = #name;
    }
}

fn generate_object_generics(generics: Option<Vec<String>>) -> ImplItemConst {
    match generics {
        Some(generics) => parse_quote! {
            const GENERICS: Option<&'static [&'static str]> = Some(&[
                #(#generics,)*
            ]);
        },
        None => parse_quote! {
            const GENERICS: Option<&'static [&'static str]> = None;
        },
    }
}

fn generate_object_fields(fields: Vec<TypeInfoOwned>) -> ImplItemConst {
    let fields = generate_type_infos(fields);
    parse_quote! {
        const FIELDS: &'static [::mech3ax_metadata_types::TypeInfo<'static>] = &[
            #( #fields, )*
        ];
    }
}

fn generate_object(info: ObjectInfo, struct_generics: Generics) -> ItemImpl {
    let ObjectInfo {
        ident,
        name,
        generics,
        fields,
    } = info;

    let name = generate_object_name(name);
    let generics = generate_object_generics(generics);
    let fields = generate_object_fields(fields);

    parse_quote! {
        impl #struct_generics ::mech3ax_metadata_types::Object for #ident #struct_generics {
            #name
            #generics
            #fields
        }
    }
}

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        vis: _,
        ident,
        generics,
        data,
    } = input;
    match data {
        Data::Struct(data) => {
            let info = parse_object(ident, data, &attrs)?;
            Ok(generate_object(info, generics).into_token_stream())
        }
        Data::Enum(_) => cannot_derive!(&ident, "an enum"),
        Data::Union(_) => cannot_derive!(&ident, "a union"),
    }
}
