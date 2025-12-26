use proc_macro::TokenStream;
pub(crate) mod impls;
pub(crate) mod utils;

#[proc_macro]
pub fn snapshot(input: TokenStream) -> TokenStream {
    impls::snapshot::snapshot_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn instrument(input: TokenStream, attr: TokenStream) -> TokenStream {
    impls::instrument::instrument_impl(input.into(), attr.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
