use proc_macro::TokenStream;
pub(crate) mod impls;
pub(crate) mod utils;

#[proc_macro]
pub fn snapshot(input: TokenStream) -> TokenStream {
    impls::snapshot::snapshot_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
pub fn log(input: TokenStream) -> TokenStream {
    impls::log::log_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn instrument(attr: TokenStream, input: TokenStream) -> TokenStream {
    impls::instrument::instrument_impl(attr.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
