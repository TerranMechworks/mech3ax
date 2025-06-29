//! Procedural macros for generating API type metadata.
mod attr_parsing;
mod structs;

#[proc_macro_derive(Struct, attributes(serde, dotnet))]
pub fn derive_struct_ref(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    structs::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
