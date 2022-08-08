use syn::{AttrStyle, Attribute, Error, Lit, Meta, NestedMeta, Result};

pub fn find_attr<'a, 'b>(attrs: &'a [Attribute], ident: &'b str) -> Option<&'a Attribute> {
    attrs
        .iter()
        .find(|attr| attr.style == AttrStyle::Outer && attr.path.is_ident(ident))
}

#[derive(Debug)]
pub struct ObjectJsonAttr {
    pub tuple: Option<String>,
    pub default: Option<String>,
    pub omit_if_default: bool,
}

impl ObjectJsonAttr {
    pub fn parse_from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let attr = match find_attr(attrs, "json") {
            Some(attr) => attr,
            None => {
                return Ok(Self {
                    tuple: None,
                    default: None,
                    omit_if_default: false,
                })
            }
        };

        let mut this = Self {
            tuple: None,
            default: None,
            omit_if_default: false,
        };

        match attr.parse_meta()? {
            Meta::List(meta) => {
                for nested in meta.nested.into_iter() {
                    match nested {
                        // Parse `#[json(omit_if_default)]`
                        NestedMeta::Meta(Meta::Path(p)) if p.is_ident("omit_if_default") => {
                            this.omit_if_default = true;
                        }
                        // Parse `#[json(tuple = "foo")]`
                        NestedMeta::Meta(Meta::NameValue(m)) if m.path.is_ident("tuple") => match m
                            .lit
                        {
                            Lit::Str(lit) => this.tuple = Some(lit.value()),
                            other => {
                                return Err(Error::new_spanned(other, "Expected string literal"))
                            }
                        },
                        // Parse `#[json(default = "foo")]`
                        NestedMeta::Meta(Meta::NameValue(m)) if m.path.is_ident("default") => {
                            match m.lit {
                                Lit::Str(lit) => this.default = Some(lit.value()),
                                other => {
                                    return Err(Error::new_spanned(
                                        other,
                                        "Expected string literal",
                                    ))
                                }
                            }
                        }
                        other => return Err(Error::new_spanned(other, "Unknown JSON attribute")),
                    }
                }
            }
            other => return Err(Error::new_spanned(other, "Expected #[json(...)]")),
        }

        Ok(this)
    }
}
