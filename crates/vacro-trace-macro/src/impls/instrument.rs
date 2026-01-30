use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, ItemFn, Stmt};

use crate::utils::resolve_crate_root;

pub fn instrument_impl(_attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let mut fn_impl: ItemFn = parse_quote!(#input);
    // 后续针对入口与辅助函数进行特异性判断
    let _macro_entry = fn_impl.attrs.iter().any(|a| {
        let path = a.meta.path();
        path.is_ident("proc_macro")
            || path.is_ident("proc_macro_attribute")
            || path.is_ident("proc_macro_derive")
    });
    let pkg = resolve_crate_root();

    let macro_name = &fn_impl.sig.ident;
    let current_crate = std::env::var("CARGO_CRATE_NAME").unwrap_or_else(|_| "unknown".to_string());
    let mut enter_session: Vec<Stmt> = parse_quote! {
        #[doc(hidden)]
        let __guard = if cfg!(debug_assertions) || ::std::env::var("VACRO_TRACE").is_ok() {
            ::std::option::Option::Some(#pkg::__private::TraceSession::enter(stringify!(#macro_name), #current_crate))
        } else {
            ::std::option::Option::None
        };
    };
    let mut stmts = fn_impl.block.stmts;
    enter_session.append(&mut stmts);
    fn_impl.block.stmts = enter_session;

    Ok(quote! {
        #fn_impl
    })
}
