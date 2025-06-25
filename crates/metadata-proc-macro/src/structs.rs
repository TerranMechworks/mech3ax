use crate::attr_parsing::{parse_dotnet_attr, parse_serde_attr, DotNetInfoOwned};
use mech3ax_metadata_types::{DefaultHandling, TypeSemantic};
use proc_macro2::TokenStream;
use quote::ToTokens as _;
use syn::{
    parse_quote, Attribute, Data, DataStruct, DeriveInput, Error, Expr, ExprStruct, Field,
    FieldMutability, Fields, FieldsNamed, Generics, Ident, ImplItemConst, ItemImpl, LitBool, Path,
    Result, Type, Visibility,
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
pub(crate) struct FieldInfoOwned {
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) default: DefaultHandling,
}

#[derive(Debug)]
struct StructInfo {
    pub(crate) ident: Ident,
    pub(crate) name: String,
    pub(crate) fields: Vec<FieldInfoOwned>,
    pub(crate) dotnet: DotNetInfoOwned,
}

fn parse_struct_field(ident: Ident, attrs: &[Attribute], ty: &Type) -> Result<FieldInfoOwned> {
    let default = parse_serde_attr(attrs)?;
    Ok(FieldInfoOwned {
        name: ident.to_string(),
        ty: ty.clone(),
        default,
    })
}

fn parse_struct_fields(fields: FieldsNamed) -> Result<Vec<FieldInfoOwned>> {
    let FieldsNamed {
        brace_token: _,
        named,
    } = fields;

    named
        .into_iter()
        .map(|field| {
            let Field {
                attrs,
                vis,
                ident,
                colon_token: _,
                ty,
                mutability,
            } = field;
            if !matches!(vis, Visibility::Public(_)) {
                return Err(Error::new_spanned(&ty, "Expected field to be public"));
            }
            if !matches!(mutability, FieldMutability::None) {
                return Err(Error::new_spanned(
                    &ty,
                    "Expected field to have no mutability",
                ));
            }
            let ident = ident
                .ok_or_else(|| Error::new_spanned(&ty, "Expected field to have an identifier"))?;
            parse_struct_field(ident, &attrs, &ty)
        })
        .collect::<Result<Vec<_>>>()
}

fn parse_struct_named(
    ident: Ident,
    fields: FieldsNamed,
    attrs: &[Attribute],
) -> Result<StructInfo> {
    let name = ident.to_string();
    let dotnet = parse_dotnet_attr(attrs)?;
    let fields = parse_struct_fields(fields)?;

    Ok(StructInfo {
        ident,
        name,
        fields,
        dotnet,
    })
}

fn parse_struct(ident: Ident, data: DataStruct, attrs: &[Attribute]) -> Result<StructInfo> {
    let DataStruct {
        struct_token: _,
        fields,
        semi_token: _,
    } = data;
    match fields {
        Fields::Named(fields) => parse_struct_named(ident, fields, attrs),
        Fields::Unnamed(_) => cannot_derive!(&ident, "a tuple struct"),
        Fields::Unit => cannot_derive!(&ident, "a unit struct"),
    }
}

fn generate_struct_generics(generics: Option<Vec<(Path, String)>>) -> Expr {
    match generics {
        None => parse_quote! { None },
        Some(generics) => {
            let (path, value): (Vec<_>, Vec<_>) = generics.into_iter().unzip();
            parse_quote! {
                Some(&[
                    #( (<#path as ::mech3ax_metadata_types::DerivedMetadata>::TYPE_INFO, #value), )*
                ])
            }
        }
    }
}

fn generate_struct_field_default(default: DefaultHandling) -> Path {
    match default {
        DefaultHandling::Normal => parse_quote! {
            ::mech3ax_metadata_types::DefaultHandling::Normal
        },
        DefaultHandling::OptionIsNone => parse_quote! {
            ::mech3ax_metadata_types::DefaultHandling::OptionIsNone
        },
        DefaultHandling::BoolFalse => parse_quote! {
            ::mech3ax_metadata_types::DefaultHandling::BoolFalse
        },
        DefaultHandling::BoolTrue => parse_quote! {
            ::mech3ax_metadata_types::DefaultHandling::BoolTrue
        },
        DefaultHandling::PointerZero => parse_quote! {
            ::mech3ax_metadata_types::DefaultHandling::PointerZero
        },
        DefaultHandling::SoilIsDefault => parse_quote! {
            ::mech3ax_metadata_types::DefaultHandling::SoilIsDefault
        },
        DefaultHandling::I32IsNegOne => parse_quote! {
            ::mech3ax_metadata_types::DefaultHandling::I32IsNegOne
        },
    }
}

fn generate_struct_field(field: FieldInfoOwned) -> ExprStruct {
    let FieldInfoOwned { name, ty, default } = field;
    let default = generate_struct_field_default(default);
    parse_quote! {
        ::mech3ax_metadata_types::TypeInfoStructField {
            name: #name,
            type_info: <#ty as ::mech3ax_metadata_types::DerivedMetadata>::TYPE_INFO,
            default: #default,
        }
    }
}

fn generate_struct_dotnet(dotnet: DotNetInfoOwned) -> ExprStruct {
    let DotNetInfoOwned {
        semantic,
        generics,
        partial,
        namespace,
    } = dotnet;

    let semantic: Path = match semantic {
        TypeSemantic::Ref => parse_quote! { ::mech3ax_metadata_types::TypeSemantic::Ref },
        TypeSemantic::Val => parse_quote! { ::mech3ax_metadata_types::TypeSemantic::Val },
    };
    let generics = generate_struct_generics(generics);
    let partial: LitBool = if partial {
        parse_quote!(true)
    } else {
        parse_quote!(false)
    };

    let namespace: Expr = match namespace {
        Some(namespace) => parse_quote!(Some(#namespace)),
        None => parse_quote!(None),
    };

    parse_quote! {
        ::mech3ax_metadata_types::TypeInfoStructDotNet {
            semantic: #semantic,
            generics: #generics,
            partial: #partial,
            namespace: #namespace,
        }
    }
}

fn generate_struct(info: StructInfo, struct_generics: Generics) -> ItemImpl {
    let StructInfo {
        ident,
        name,
        fields,
        dotnet,
    } = info;

    let fields: Vec<_> = fields.into_iter().map(generate_struct_field).collect();
    let dotnet = generate_struct_dotnet(dotnet);

    let type_type: ImplItemConst = parse_quote! {
        const TYPE_INFO: &'static ::mech3ax_metadata_types::TypeInfo =
            &::mech3ax_metadata_types::TypeInfo::Struct(::mech3ax_metadata_types::TypeInfoStruct {
                name: #name,
                fields: &[
                    #( #fields, )*
                ],
                module_path: ::std::module_path!(),
                dotnet: #dotnet,
            });
    };

    parse_quote! {
        impl #struct_generics ::mech3ax_metadata_types::DerivedMetadata for #ident #struct_generics {
            #type_type
        }
    }
}

pub(crate) fn derive(input: DeriveInput) -> Result<TokenStream> {
    let DeriveInput {
        attrs,
        vis: _,
        ident,
        generics,
        data,
    } = input;
    match data {
        Data::Struct(data) => {
            let info = parse_struct(ident, data, &attrs)?;
            Ok(generate_struct(info, generics).into_token_stream())
        }
        Data::Enum(_) => cannot_derive!(&ident, "an enum"),
        Data::Union(_) => cannot_derive!(&ident, "a union"),
    }
}
