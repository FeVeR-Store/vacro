use proc_macro::TokenStream;

use crate::trace::{parse_quote_impl, trace_impl};

mod trace;

#[proc_macro_attribute]
pub fn trace(attr: TokenStream, item: TokenStream) -> TokenStream {
    trace_impl(attr.into(), item.into()).into()
}
#[proc_macro]
pub fn parse_quote(input: TokenStream) -> TokenStream {
    parse_quote_impl(input.into()).into()
}
