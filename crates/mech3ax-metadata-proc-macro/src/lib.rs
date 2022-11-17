//! Procedural macros for generating API type metadata.
mod attr_parsing;
mod enums;
mod structs;
mod unions;

use mech3ax_metadata_types::TypeSemantic;

#[proc_macro_derive(Enum)]
pub fn derive_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    enums::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(ValStruct)]
pub fn derive_struct_val(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    structs::derive(input, TypeSemantic::Val)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(RefStruct, attributes(generic, serde))]
pub fn derive_struct_ref(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    structs::derive(input, TypeSemantic::Ref)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Union)]
pub fn derive_sum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    unions::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
