use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_quote, parse_quote_spanned, Stmt};

#[proc_macro]
#[vacro_report::scope]
pub fn parse_stmt(input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let output: Stmt = parse_quote!(#input);
    quote! {#output}.into()
}

#[proc_macro]
#[vacro_report::scope]
pub fn parse_stmt_spanned(input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let span = Span::call_site();
    let output: Stmt = parse_quote_spanned! {span => #input};
    quote! {#output}.into()
}
