use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;

#[inline]
pub fn crate_name(span: Span) -> TokenStream {
    if cfg!(feature = "standalone") {
        quote_spanned! { span => ::vacro_report }
    } else {
        quote_spanned! { span => ::vacro::report }
    }
}
