mod offsets;

#[proc_macro_derive(Offsets, attributes(serde, dotnet))]
pub fn derive_offsets(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    offsets::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
