use mech3ax_metadata_types::{DefaultHandling, TypeSemantic};
use syn::{AttrStyle, Attribute, Error, Lit, Meta, NestedMeta, Path, Result};

fn find_attr<'a>(attrs: &'a [Attribute], ident: &str) -> Option<&'a Attribute> {
    attrs
        .iter()
        .find(|attr| attr.style == AttrStyle::Outer && attr.path.is_ident(ident))
}

fn parse_serde_skip_serializing_if(lit: Lit) -> Result<DefaultHandling> {
    match lit {
        Lit::Str(lit_str) => {
            let value = lit_str.value();
            match value.as_str() {
                "Option::is_none" => Ok(DefaultHandling::OptionIsNone),
                "bool_false" => Ok(DefaultHandling::BoolFalse),
                "bool_true" => Ok(DefaultHandling::BoolTrue),
                "pointer_zero" => Ok(DefaultHandling::PointerZero),
                _ => Err(Error::new_spanned(
                    lit_str,
                    format!("Unknown skip `{}`", value),
                )),
            }
        }
        other => Err(Error::new_spanned(other, "Expected string literal")),
    }
}

pub fn parse_serde_attr(attrs: &[Attribute]) -> Result<DefaultHandling> {
    let attr = match find_attr(attrs, "serde") {
        Some(attr) => attr,
        None => return Ok(DefaultHandling::Normal),
    };

    match attr.parse_meta()? {
        Meta::List(meta) => {
            for nested in meta.nested.into_iter() {
                match nested {
                    // Parse `#[serde(skip_serializing_if = "foo")]`
                    NestedMeta::Meta(Meta::NameValue(m))
                        if m.path.is_ident("skip_serializing_if") =>
                    {
                        return parse_serde_skip_serializing_if(m.lit);
                    }
                    _ => {}
                }
            }
        }
        other => return Err(Error::new_spanned(other, "Expected #[serde(...)]")),
    }

    Ok(DefaultHandling::Normal)
}

fn parse_generic_param(nested: NestedMeta) -> Result<(Path, String)> {
    match nested {
        // Parse `#[generic(foo = "foo"...)]`
        NestedMeta::Meta(Meta::NameValue(nv)) => {
            let value = match nv.lit {
                Lit::Str(litstr) => Ok(litstr.value()),
                other => Err(Error::new_spanned(
                    other,
                    "Expected #[generic(foo = \"foo\"...)]",
                )),
            }?;
            let path = nv.path;
            Ok((path, value))
        }
        other => Err(Error::new_spanned(
            other,
            "Expected #[generic(foo = \"foo\"...)]",
        )),
    }
}

fn parse_namespace_param(lit: Lit) -> Result<String> {
    match lit {
        Lit::Str(lit_str) => Ok(lit_str.value()),
        other => Err(Error::new_spanned(other, "Expected string literal")),
    }
}

#[derive(Debug, Default)]
pub struct DotNetInfoOwned {
    pub semantic: TypeSemantic,
    pub generics: Option<Vec<(Path, String)>>,
    pub partial: bool,
    pub namespace: Option<String>,
}

fn parse_dotnet_inner(inner: impl Iterator<Item = NestedMeta>) -> Result<DotNetInfoOwned> {
    let mut dotnet = DotNetInfoOwned::default();
    for nested in inner {
        let NestedMeta::Meta(attr) = nested else {
            return Err(Error::new_spanned(
                nested,
                "Expected #[dotnet(...)], but found literal",
            ));
        };
        match attr {
            Meta::Path(path) if path.is_ident("partial") => dotnet.partial = true,
            Meta::Path(path) if path.is_ident("val_struct") => dotnet.semantic = TypeSemantic::Val,
            Meta::Path(path) => {
                return Err(Error::new_spanned(
                    path,
                    "Expected #[dotnet(partial)] or #[dotnet(val_struct)]",
                ));
            }
            Meta::List(list) if list.path.is_ident("generic") => {
                let generic = list
                    .nested
                    .into_iter()
                    .map(parse_generic_param)
                    .collect::<Result<_>>()?;
                dotnet.generics = Some(generic);
            }
            Meta::List(list) => {
                return Err(Error::new_spanned(list, "Expected #[dotnet(generic(...))]"));
            }
            Meta::NameValue(nv) if nv.path.is_ident("namespace") => {
                let namespace = parse_namespace_param(nv.lit)?;
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

pub fn parse_dotnet_attr(attrs: &[Attribute]) -> Result<DotNetInfoOwned> {
    let Some(dotnet) = find_attr(attrs, "dotnet") else {
        return Ok(DotNetInfoOwned::default());
    };
    match dotnet.parse_meta()? {
        Meta::List(list) => parse_dotnet_inner(list.nested.into_iter()),
        other => Err(Error::new_spanned(other, "Expected #[dotnet(...)]")),
    }
}
