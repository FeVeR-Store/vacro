use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::utils::crate_name;
pub fn parse_quote_impl(input: TokenStream, spanned: bool) -> TokenStream {
    let pkg = crate_name(Span::call_site());
    let quote_token = if spanned {
        quote! {quote_spanned!}
    } else {
        quote! {quote!}
    };
    quote! {#pkg::__private::parse_quote_traced(#pkg::__private::#quote_token {#input}, #spanned)}
}
