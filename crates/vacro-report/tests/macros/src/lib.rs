use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_quote, Stmt};

#[proc_macro]
#[vacro_report::scope]
pub fn parse_stmt(input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let output: Stmt = parse_quote!(#input);
    quote! {#output}.into()
}
