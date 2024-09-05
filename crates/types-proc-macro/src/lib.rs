//! Procedural macros for mech3ax (outside of metadata generation).
mod bitflags;

#[proc_macro]
pub fn bitflags_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as bitflags::BitflagsInput);
    bitflags::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
