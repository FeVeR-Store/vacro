use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, ItemFn, Stmt};

use crate::utils::crate_name;

pub fn instrument_impl(_attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let mut fn_impl: ItemFn = parse_quote!(#input);
    let macro_entry = fn_impl.attrs.iter().any(|a| {
        let path = a.meta.path();
        path.is_ident("proc_macro")
            || path.is_ident("proc_macro_attribute")
            || path.is_ident("proc_macro_derive")
    });
    let pkg = crate_name();
    if macro_entry {
        let macro_name = &fn_impl.sig.ident;
        let mut enter_session: Vec<Stmt> = parse_quote! {
            #[doc(hidden)]
            let __guard = #pkg::__private::TraceSession::enter(stringify!(#macro_name));
        };
        let mut stmts = fn_impl.block.stmts;
        enter_session.append(&mut stmts);
        fn_impl.block.stmts = enter_session;
    };

    Ok(quote! {
        #fn_impl
    })
}
