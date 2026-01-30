use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use vacro_report::{help, scope};

#[proc_macro]
#[scope]
pub fn parse_stmt(input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let output: Stmt = parse_quote!(#input);
    quote! {#output}.into()
}

#[proc_macro]
#[scope]
pub fn parse_stmt_spanned(input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let span = Span::call_site();
    let output: Stmt = parse_quote_spanned! {span => #input};
    quote! {#output}.into()
}

use syn::{
    parse::{Parse, Parser},
    parse_quote, parse_quote_spanned, LitStr, Stmt,
};

help!(MyLitStr: LitStr {
    error: "这里需要一个String字面量",
    help: "应该在两侧添加双引号",
    example: "my-string"
});

#[proc_macro]
pub fn parse_help(input: TokenStream) -> TokenStream {
    match MyLitStr::parse.parse(input) {
        Ok(token) => quote! {#token}.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
