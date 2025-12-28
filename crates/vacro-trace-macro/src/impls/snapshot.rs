use crate::utils::crate_name;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_quote, Expr, LitStr, Token};

struct SnapshotInput {
    tag: String,
    ast: Expr,
}

impl Parse for SnapshotInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tag: LitStr = input.parse()?;
        let tag = tag.value();
        input.parse::<Token![,]>()?;
        let ast = input.parse()?;
        Ok(SnapshotInput { tag, ast })
    }
}

pub fn snapshot_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let input: SnapshotInput = parse_quote!(#input);
    let tag = input.tag;
    let ast = input.ast;
    let pkg = crate_name();
    let snapshot_impl = quote! {
        let ast = &#ast;
        let ast = #pkg::__private::quote!(##ast);
        #pkg::__private::snapshot(#tag, ast.to_string());
    };

    let code = parse_quote! {
        if ::std::env::var("VACRO_TRACE").is_ok() {
            #snapshot_impl
        }
    };

    Ok(code)
}
