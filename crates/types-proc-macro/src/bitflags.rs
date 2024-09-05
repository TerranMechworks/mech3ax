use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parse_quote, Error, Ident, LitInt, Path, Result, Token};

#[derive(Debug)]
pub enum BitflagsRepr {
    U8(Path),
    U16(Path),
    U32(Path),
}

impl BitflagsRepr {
    pub fn max_bits(&self) -> usize {
        match self {
            Self::U8(_) => 8,
            Self::U16(_) => 16,
            Self::U32(_) => 32,
        }
    }
}

impl Parse for BitflagsRepr {
    fn parse(input: ParseStream) -> Result<Self> {
        let path: Path = input.parse()?;
        if path.is_ident("u8") {
            return Ok(Self::U8(path));
        }
        if path.is_ident("u16") {
            return Ok(Self::U16(path));
        }
        if path.is_ident("u32") {
            return Ok(Self::U32(path));
        }

        Err(Error::new_spanned(path, "expected `u8`, `u16`, or `u32`"))
    }
}

impl ToTokens for BitflagsRepr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::U8(path) => path.to_tokens(tokens),
            Self::U16(path) => path.to_tokens(tokens),
            Self::U32(path) => path.to_tokens(tokens),
        }
    }
}

#[derive(Debug)]
pub struct Value {
    value: usize,
    #[allow(dead_code)]
    token: LitInt,
}

impl Parse for Value {
    fn parse(input: ParseStream) -> Result<Self> {
        let token: LitInt = input.parse()?;
        let value: u8 = token.base10_parse()?;
        if token.suffix().is_empty() {
            Ok(Self {
                value: usize::from(value),
                token,
            })
        } else {
            Err(Error::new_spanned(token, "expected literal without suffix"))
        }
    }
}

#[derive(Debug)]
pub struct Bitflag {
    pub value: Value,
    pub const_case: String,
}

impl Parse for Bitflag {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let _fat_arrow_token: Token![=>] = input.parse()?;
        let value: Value = input.parse()?;

        let const_case = ident.to_string();

        Ok(Self { value, const_case })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct BitflagsInput {
    pub ident: Ident,
    pub repr: BitflagsRepr,
    pub flags: Punctuated<Bitflag, Token![,]>,
}

impl Parse for BitflagsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        let repr = input.parse()?;
        let content;
        let _brace_token = braced!(content in input);
        let flags = content.parse_terminated(Bitflag::parse, Token![,])?;
        Ok(Self { ident, repr, flags })
    }
}

pub fn derive(input: BitflagsInput) -> Result<TokenStream> {
    let BitflagsInput { ident, repr, flags } = input;
    let span = ident.span();

    // construct the flag names and validate the flag values in the process
    let bits = repr.max_bits();
    let mut flag_names = vec![None; bits];
    for flag in flags.iter() {
        let flag_name = flag_names
            .get_mut(flag.value.value)
            .ok_or_else(|| Error::new(span, "out of range"))?;
        if flag_name.is_some() {
            return Err(Error::new(span, "duplicate value"));
        }
        *flag_name = Some(flag.const_case.clone());
    }

    let mut valid = String::new();
    valid.push_str("0b");
    for set in flag_names.iter().map(|flag_name| flag_name.is_some()).rev() {
        valid.push(if set { '1' } else { '0' });
    }
    let valid = LitInt::new(&valid, ident.span());

    let flag_names: Vec<syn::Expr> = flag_names
        .into_iter()
        .map(|flag_name| match flag_name {
            Some(name) => parse_quote! { ::core::option::Option::Some(#name) },
            None => parse_quote! { ::core::option::Option::None },
        })
        .collect();

    let struct_impl: syn::ItemImpl = parse_quote! {
        impl #ident {
            const VALID: #repr = #valid;

            const FLAGS: &'static [::core::option::Option<&'static str>; #bits] = &[
                #(#flag_names,)*
            ];
        }
    };

    Ok(struct_impl.into_token_stream())
}
