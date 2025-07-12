use heck::ToSnakeCase as _;
use syn::{Ident, Result, Token, parse_quote};

pub(crate) struct JsonFlagsVariant {
    pub(crate) attrs: Vec<syn::Attribute>,
    pub(crate) ident: Ident,
}

impl syn::parse::Parse for JsonFlagsVariant {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let ident = input.parse()?;
        Ok(Self { attrs, ident })
    }
}

pub(crate) struct JsonFlagsInput {
    pub(crate) struct_ident: Ident,
    pub(crate) variants: syn::punctuated::Punctuated<JsonFlagsVariant, Token![,]>,
}

impl syn::parse::Parse for JsonFlagsInput {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let _struct_token: Token![struct] = input.parse()?;
        let struct_ident = input.parse()?;
        let content;
        let _brace_token = syn::braced!(content in input);
        let variants = content.parse_terminated(JsonFlagsVariant::parse, Token![,])?;
        Ok(Self {
            struct_ident,
            variants,
        })
    }
}

pub(crate) fn make(input: JsonFlagsInput) -> Result<proc_macro2::TokenStream> {
    let JsonFlagsInput {
        struct_ident,
        variants,
    } = input;
    let struct_name = struct_ident.to_string();
    let json_ident = quote::format_ident!("{}Exhaustive", struct_ident);

    let (variants, mut variant_attrs): (Vec<Ident>, Vec<Vec<syn::Attribute>>) = variants
        .into_iter()
        .map(|JsonFlagsVariant { attrs, ident }| (ident, attrs))
        .unzip();
    let field_idents: Vec<Ident> = variants
        .iter()
        .zip(variant_attrs.iter_mut())
        .map(|(ident, attrs)| {
            let mut name = ident.to_string().to_snake_case();
            match name.as_str() {
                "override" => {
                    name = "override_".to_string();
                    attrs.push(parse_quote! {
                        #[serde(rename = "override")]
                    });
                }
                "static" => {
                    name = "static_".to_string();
                    attrs.push(parse_quote! {
                        #[serde(rename = "static")]
                    });
                }
                _ => {}
            }
            Ident::new(&name, ident.span())
        })
        .collect();

    let json_struct: syn::ItemStruct = parse_quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        #[serde(rename = #struct_name)]
        pub struct #json_ident {
        #(
            #(#variant_attrs)*
            pub #field_idents: bool,
        )*
        }
    };

    let flags_to_json: syn::ItemFn = parse_quote! {
        pub fn exhaustive(self) -> #json_ident {
            #json_ident {
                #(#field_idents: self.contains(Self::#variants),)*
            }
        }
    };

    let json_to_flags: syn::ItemFn = parse_quote! {
        pub fn from_exhaustive(json: &#json_ident) -> Self {
            let mut v = Self::empty();
            #(
            if json.#field_idents {
                v |= Self::#variants;
            }
            )*
            v
        }
    };

    let struct_impl: syn::ItemImpl = parse_quote! {
        impl #struct_ident {
            #flags_to_json
            #json_to_flags
        }
    };

    Ok(quote::quote! {
        #json_struct
        #struct_impl
    })
}
