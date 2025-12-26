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

#[proc_macro]
pub fn error(input: TokenStream) -> TokenStream {
    impls::log::shortcut_impl("Error", input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
pub fn warn(input: TokenStream) -> TokenStream {
    impls::log::shortcut_impl("Warn", input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
pub fn info(input: TokenStream) -> TokenStream {
    impls::log::shortcut_impl("Info", input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
pub fn debug(input: TokenStream) -> TokenStream {
    impls::log::shortcut_impl("Debug", input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
pub fn trace(input: TokenStream) -> TokenStream {
    impls::log::shortcut_impl("Trace", input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn instrument(attr: TokenStream, input: TokenStream) -> TokenStream {
    impls::instrument::instrument_impl(attr.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
