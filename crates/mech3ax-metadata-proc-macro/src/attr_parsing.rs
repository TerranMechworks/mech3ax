use mech3ax_metadata_types::{DefaultHandling, TypeSemantic};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{AttrStyle, Attribute, Error, Expr, Lit, LitStr, Meta, MetaList, Path, Result, Token};

fn find_attr<'a>(attrs: &'a [Attribute], ident: &str) -> Option<&'a Attribute> {
    attrs
        .iter()
        .find(|attr| attr.style == AttrStyle::Outer && attr.path().is_ident(ident))
}

fn parse_serde_skip_serializing_if(lit: LitStr) -> Result<DefaultHandling> {
    let value = lit.value();
    match value.as_str() {
        "Option::is_none" => Ok(DefaultHandling::OptionIsNone),
        "bool_false" => Ok(DefaultHandling::BoolFalse),
        "bool_true" => Ok(DefaultHandling::BoolTrue),
        "pointer_zero" => Ok(DefaultHandling::PointerZero),
        _ => Err(Error::new_spanned(lit, format!("unknown skip `{}`", value))),
    }
}

fn parse_expr_to_str_lit(expr: Expr) -> Result<LitStr> {
    match expr {
        Expr::Lit(expr_lit) => {
            if !expr_lit.attrs.is_empty() {
                return Err(Error::new_spanned(
                    expr_lit,
                    "expected string literal with no attributes",
                ));
            }
            match expr_lit.lit {
                Lit::Str(lit) => Ok(lit),
                other => Err(Error::new_spanned(other, "expected string literal")),
            }
        }
        other => Err(Error::new_spanned(other, "expected literal")),
    }
}

pub fn parse_serde_attr(attrs: &[Attribute]) -> Result<DefaultHandling> {
    let attr = match find_attr(attrs, "serde") {
        Some(attr) => attr,
        None => return Ok(DefaultHandling::Normal),
    };

    let meta_list = attr.meta.require_list()?;
    let punctuated: Punctuated<Meta, Token![,]> =
        meta_list.parse_args_with(Punctuated::parse_separated_nonempty)?;

    let mut handling = DefaultHandling::Normal;
    for meta in punctuated {
        match meta {
            Meta::NameValue(nv) if nv.path.is_ident("skip_serializing_if") => {
                let lit = parse_expr_to_str_lit(nv.value)?;
                handling = parse_serde_skip_serializing_if(lit)?
            }
            _ => {}
        }
    }
    Ok(handling)
}

#[derive(Debug)]
pub struct DotNetInfoOwned {
    pub semantic: TypeSemantic,
    pub generics: Option<Vec<(Path, String)>>,
    pub partial: bool,
    pub namespace: Option<String>,
}

impl Default for DotNetInfoOwned {
    fn default() -> Self {
        Self {
            semantic: TypeSemantic::Ref,
            generics: None,
            partial: false,
            namespace: None,
        }
    }
}

#[derive(Debug)]
struct GenericParam {
    path: Path,
    value: String,
}

impl GenericParam {
    fn into_pair(self) -> (Path, String) {
        let Self { path, value } = self;
        (path, value)
    }
}

impl Parse for GenericParam {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let path: Path = input.parse()?;
        let _eq_token: Token![=] = input.parse()?;
        let lit: LitStr = input.parse()?;
        let value = lit.value();
        Ok(Self { path, value })
    }
}

fn parse_meta_list_to_generics(meta_list: MetaList) -> Result<Vec<(Path, String)>> {
    let punctuated: Punctuated<GenericParam, Token![,]> =
        meta_list.parse_args_with(Punctuated::parse_separated_nonempty)?;
    Ok(punctuated
        .into_iter()
        .map(GenericParam::into_pair)
        .collect())
}

pub fn parse_dotnet_attr(attrs: &[Attribute]) -> Result<DotNetInfoOwned> {
    let Some(attr) = find_attr(attrs, "dotnet") else {
        return Ok(DotNetInfoOwned::default());
    };

    let meta_list = attr.meta.require_list()?;
    let punctuated: Punctuated<Meta, Token![,]> =
        meta_list.parse_args_with(Punctuated::parse_separated_nonempty)?;

    let mut dotnet = DotNetInfoOwned {
        semantic: TypeSemantic::Ref,
        generics: None,
        partial: false,
        namespace: None,
    };
    for meta in punctuated {
        match meta {
            Meta::Path(path) if path.is_ident("partial") => dotnet.partial = true,
            Meta::Path(path) if path.is_ident("val_struct") => dotnet.semantic = TypeSemantic::Val,
            Meta::Path(path) => {
                return Err(Error::new_spanned(
                    path,
                    "Expected #[dotnet(partial)] or #[dotnet(val_struct)]",
                ));
            }
            Meta::List(list) if list.path.is_ident("generic") => {
                let generics = parse_meta_list_to_generics(list)?;
                dotnet.generics = Some(generics);
            }
            Meta::List(list) => {
                return Err(Error::new_spanned(list, "Expected #[dotnet(generic(...))]"));
            }
            Meta::NameValue(nv) if nv.path.is_ident("namespace") => {
                let lit = parse_expr_to_str_lit(nv.value)?;
                let namespace = lit.value();
                dotnet.namespace = Some(namespace);
            }
            Meta::NameValue(nv) => {
                return Err(Error::new_spanned(
                    nv,
                    "Expected #[dotnet(namespace = \"...\")]",
                ));
            }
        }
    }
    Ok(dotnet)
}
