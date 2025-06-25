mod json_flags;
mod offsets;

/// This exists because macro_rules cannot (yes) generate unique idents
#[proc_macro_derive(Offsets, attributes(serde, dotnet))]
pub fn derive_offsets(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    offsets::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// This exists because `#[serde(rename_all = "snake_case")]` is  horribly broken:
/// * https://github.com/serde-rs/serde/issues/2323
/// * https://github.com/serde-rs/serde/issues/2635
#[proc_macro]
pub fn json_flags(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as json_flags::JsonFlagsInput);
    json_flags::make(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
