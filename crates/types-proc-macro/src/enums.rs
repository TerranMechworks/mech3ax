use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::{
    parse_quote, AttrStyle, Attribute, Data, DataEnum, DeriveInput, Error, Expr, ExprLit, Ident,
    ImplItemConst, ImplItemFn, ImplItemType, ItemImpl, Lit, LitInt, Path, Result, Variant,
};

macro_rules! cannot_derive {
    ($ident:expr, $for:expr) => {
        Err(Error::new_spanned(
            $ident,
            concat!("Can't derive a primitive enum for ", $for),
        ))
    };
}

#[derive(Debug)]
struct EnumInfo {
    pub ident: Ident,
    pub repr: Path,
    pub variants: Vec<(Ident, LitInt)>,
    pub nums: Vec<usize>,
}

fn parse_repr_attr(ident: &Ident, attrs: &[Attribute]) -> Result<Path> {
    let attr = attrs
        .iter()
        .find(|attr| attr.style == AttrStyle::Outer && attr.path().is_ident("repr"))
        .ok_or_else(|| {
            Error::new_spanned(
                ident,
                "Expected enum to have `#[repr(<primitive>)]` attribute",
            )
        })?;

    let mut repr: Option<Path> = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("u8") || meta.path.is_ident("u32") || meta.path.is_ident("u16") {
            repr = Some(meta.path);
            return Ok(());
        }
        Err(meta.error("unrecognized repr"))
    })?;
    repr.ok_or_else(|| {
        Error::new_spanned(
            ident,
            "Expected enum to have `#[repr(<primitive>)]` attribute",
        )
    })
}

fn parse_variant(variant: Variant) -> Result<(Ident, LitInt)> {
    let Variant {
        attrs: _,
        ident,
        fields,
        discriminant,
    } = variant;
    if !fields.is_empty() {
        return cannot_derive!(&ident, "a variant with fields");
    }
    let discriminant = discriminant
        .map(|(_eq, expr)| expr)
        .and_then(|expr| match expr {
            Expr::Lit(ExprLit {
                lit: Lit::Int(int),
                attrs: _,
            }) => Some(int),
            _ => None,
        })
        .ok_or_else(|| {
            Error::new_spanned(&ident, "Expected all variants to have a discriminant")
        })?;
    Ok((ident, discriminant))
}

fn parse_enum(ident: Ident, data: DataEnum, attrs: &[Attribute]) -> Result<EnumInfo> {
    let DataEnum {
        enum_token: _,
        brace_token: _,
        variants,
    } = data;

    let repr = parse_repr_attr(&ident, attrs)?;
    let variants = variants
        .into_iter()
        .map(parse_variant)
        .collect::<Result<Vec<_>>>()?;

    let nums = variants
        .iter()
        .map(|(_ident, lit)| lit.base10_parse::<usize>())
        .collect::<Result<Vec<_>>>()?;

    Ok(EnumInfo {
        ident,
        repr,
        variants,
        nums,
    })
}

fn is_consecutive(first: usize, middle: &[usize], last: usize) -> bool {
    let mut curr = first + 1;
    for v in middle.iter().copied() {
        if curr != v {
            return false;
        }
        curr += 1;
    }
    curr == last
}

fn format_discriminants_more(first: usize, middle: &[usize], last: usize) -> String {
    if is_consecutive(first, middle, last) {
        format!("{}..{}", first, last)
    } else {
        use std::fmt::Write as _;
        let mut s = String::new();
        write!(s, "{}, ", first).unwrap();
        for v in middle {
            write!(s, "{}, ", v).unwrap();
        }
        write!(s, "or {}", last).unwrap();
        s
    }
}

fn format_discriminants(nums: &[usize]) -> String {
    match &nums[..] {
        [] => panic!("no discriminants"),
        [one] => format!("{}", one),
        [one, two] => format!("{} or {}", one, two),
        [first, middle @ .., last] => format_discriminants_more(*first, middle, *last),
    }
}

fn generate_enum(info: EnumInfo) -> TokenStream {
    let EnumInfo {
        ident,
        repr,
        variants,
        nums,
    } = info;

    let (variants, discriminants): (Vec<_>, Vec<_>) = variants.into_iter().unzip();

    let primitive: ImplItemType = parse_quote! {
        type Primitive = #repr;
    };

    let description = format_discriminants(&nums);
    let description: ImplItemConst = parse_quote! {
        const DISCRIMINANTS: &'static str = #description;
    };

    let from_primitive: ImplItemFn = parse_quote! {
        fn from_primitive(v: Self::Primitive) -> Option<Self> {
            match v {
                #(#discriminants => Some(Self::#variants),)*
                _ => None,
            }
        }
    };

    let trait_impl: ItemImpl = parse_quote! {
        impl ::mech3ax_types::PrimitiveEnum for #ident {
            #primitive
            #description

            #from_primitive
        }
    };

    let from_impl: ItemImpl = parse_quote! {
        impl From<#ident> for #repr {
            #[inline]
            fn from(value: #ident) -> Self {
                value as _
            }
        }
    };

    let const_impl: ItemImpl = parse_quote! {
        impl #ident {
            #[inline]
            pub const fn as_(self) -> #repr {
                self as _
            }
        }
    };

    let mut tokens = TokenStream::new();
    tokens.extend(trait_impl.into_token_stream());
    tokens.extend(from_impl.into_token_stream());
    tokens.extend(const_impl.into_token_stream());
    tokens
}

pub fn derive_primitive_enum(input: DeriveInput) -> Result<TokenStream> {
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
            Ok(generate_enum(info))
        }
        Data::Struct(_) => cannot_derive!(&ident, "a struct"),
        Data::Union(_) => cannot_derive!(&ident, "a union"),
    }
}
