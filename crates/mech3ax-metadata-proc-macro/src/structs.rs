use crate::attr_parsing::parse_generic_attr;
use crate::type_generation::generate_type_infos;
use crate::type_parsing::parse_type_info;
use mech3ax_metadata_types::{TypeInfoOwned, TypeSemantic};
use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::{
    parse_quote, Attribute, Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed,
    Generics, Ident, ImplItemConst, ItemImpl, Result,
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
struct StructInfo {
    pub ident: Ident,
    pub name: String,
    pub semantic: TypeSemantic,
    pub generics: Option<Vec<(String, String)>>,
    pub fields: Vec<TypeInfoOwned>,
}

fn parse_struct_generics(attrs: &[Attribute]) -> Result<Option<Vec<(String, String)>>> {
    parse_generic_attr(attrs)
}

fn parse_struct_fields(fields: FieldsNamed) -> Result<Vec<TypeInfoOwned>> {
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

fn generate_struct_name(name: String) -> ImplItemConst {
    parse_quote! {
        const NAME: &'static str = #name;
    }
}

fn generate_struct_semantic(semantic: TypeSemantic) -> ImplItemConst {
    match semantic {
        TypeSemantic::Ref => {
            parse_quote! {
                const SEMANTIC: ::mech3ax_metadata_types::TypeSemantic = ::mech3ax_metadata_types::TypeSemantic::Ref;
            }
        }
        TypeSemantic::Val => {
            parse_quote! {
                const SEMANTIC: ::mech3ax_metadata_types::TypeSemantic = ::mech3ax_metadata_types::TypeSemantic::Val;
            }
        }
    }
}

fn generate_struct_generics(generics: Option<Vec<(String, String)>>) -> ImplItemConst {
    let res = match generics {
        None => parse_quote! {
            const GENERICS: Option<&'static [(&'static str, &'static str)]> = None;
        },
        Some(generics) => {
            let (name, value): (Vec<_>, Vec<_>) = generics.into_iter().unzip();
            parse_quote! {
                const GENERICS: Option<&'static [(&'static str, &'static str)]> = Some(&[
                    #( (#name, #value), )*
                ]);
            }
        }
    };
    res
}

fn generate_struct_fields(fields: Vec<TypeInfoOwned>) -> ImplItemConst {
    let fields = generate_type_infos(fields);
    parse_quote! {
        const FIELDS: &'static [::mech3ax_metadata_types::TypeInfo<'static>] = &[
            #( #fields, )*
        ];
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

    let name = generate_struct_name(name);
    let semantic = generate_struct_semantic(semantic);
    let generics = generate_struct_generics(generics);
    let fields = generate_struct_fields(fields);

    parse_quote! {
        impl #struct_generics ::mech3ax_metadata_types::Struct for #ident #struct_generics {
            #name
            #semantic
            #generics
            #fields
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
