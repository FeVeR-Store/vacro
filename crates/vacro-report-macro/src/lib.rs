use proc_macro::TokenStream;

use crate::impls::{parse_quote_impl, report_scope_impl};

mod impls;

#[proc_macro_attribute]
pub fn scope(attr: TokenStream, item: TokenStream) -> TokenStream {
    report_scope_impl(attr.into(), item.into()).into()
}
#[proc_macro]
pub fn parse_quote(input: TokenStream) -> TokenStream {
    parse_quote_impl(input.into()).into()
}
