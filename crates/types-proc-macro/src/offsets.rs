use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::{Error, Ident, Result, Visibility};

macro_rules! cannot_derive {
    ($ident:ident, $for:literal) => {
        Err(Error::new_spanned(
            &$ident,
            concat!("Can't derive offsets for ", $for),
        ))
    };
}

pub fn derive(input: syn::DeriveInput) -> Result<TokenStream> {
    let syn::DeriveInput {
        attrs: _,
        vis,
        ident,
        generics: _,
        data,
    } = input;
    match data {
        syn::Data::Struct(syn::DataStruct {
            struct_token: _,
            fields,
            semi_token: _,
        }) => match fields {
            syn::Fields::Named(fields) => {
                let fields = parse_fields(fields)?;
                Ok(generate_offsets(vis, ident, fields).into_token_stream())
            }
            syn::Fields::Unnamed(_) => cannot_derive!(ident, "a tuple struct"),
            syn::Fields::Unit => cannot_derive!(ident, "a unit struct"),
        },
        syn::Data::Enum(_) => cannot_derive!(ident, "an enum"),
        syn::Data::Union(_) => cannot_derive!(ident, "a union"),
    }
}

fn parse_fields(fields: syn::FieldsNamed) -> Result<Vec<(Visibility, Ident)>> {
    fields
        .named
        .into_iter()
        .map(|field| {
            let syn::Field {
                attrs: _,
                vis,
                ident,
                colon_token: _,
                ty: _,
                mutability: _,
            } = field;
            match ident {
                Some(ident) => Ok((vis, ident)),
                None => Err(Error::new_spanned(
                    &ident,
                    "Expected field to have an identifier",
                )),
            }
        })
        .collect::<Result<Vec<_>>>()
}

fn generate_offsets(
    struct_vis: Visibility,
    struct_ident: Ident,
    fields: Vec<(Visibility, Ident)>,
) -> TokenStream {
    let offsets_ident = quote::format_ident!("__{}FieldOffsets", struct_ident);
    let (field_vis, field_idents): (Vec<_>, Vec<_>) = fields.into_iter().unzip();

    let offsets_struct: syn::ItemStruct = syn::parse_quote! {
        #[derive(Debug)]
        #struct_vis struct #offsets_ident {
            #(#field_vis #field_idents: usize,)*
        }
    };

    let offsets_const: syn::ItemImpl = syn::parse_quote! {
        impl #struct_ident {
            const __FO: #offsets_ident = #offsets_ident {
                #(#field_idents: ::std::mem::offset_of!(#struct_ident, #field_idents),)*
            };
        }
    };

    let offsets_trait: syn::ItemImpl = syn::parse_quote! {
        impl ::mech3ax_types::cstruct::CStruct for #struct_ident {
            type FieldOffsets = #offsets_ident;
            fn __field_offsets(&self) -> &'static Self::FieldOffsets {
                &Self::__FO
            }
        }
    };

    quote::quote! {
        #offsets_struct
        #offsets_const
        #offsets_trait
    }
}
