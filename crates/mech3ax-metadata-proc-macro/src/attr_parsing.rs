use mech3ax_metadata_types::DefaultHandling;
use syn::{AttrStyle, Attribute, Error, Lit, Meta, NestedMeta, Path, Result};

fn find_attr<'a, 'b>(attrs: &'a [Attribute], ident: &'b str) -> Option<&'a Attribute> {
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
        other => return Err(Error::new_spanned(other, "Expected #[json(...)]")),
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

pub fn parse_generic_attr(attrs: &[Attribute]) -> Result<Option<Vec<(Path, String)>>> {
    let attr = match find_attr(attrs, "generic") {
        Some(attr) => attr,
        None => return Ok(None),
    };

    match attr.parse_meta()? {
        Meta::List(meta) => meta
            .nested
            .into_iter()
            .map(parse_generic_param)
            .collect::<Result<_>>()
            .map(Some),
        other => Err(Error::new_spanned(other, "Expected #[generic(...)]")),
    }
}
