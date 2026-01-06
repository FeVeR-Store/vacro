use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, parse_quote_spanned,
    visit_mut::{self, VisitMut},
    ItemFn, Macro,
};

use crate::utils::crate_name;

pub fn report_scope_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // 原函数
    let original_fn: ItemFn = parse_quote!(#item);

    // 要增强的函数
    let mut enhanced_fn = original_fn.clone();

    // 改写函数体里的宏调用
    let mut rewriter = TraceRewriter::new();
    rewriter.visit_item_fn_mut(&mut enhanced_fn);

    // 补全替换parse_quote可能导致的unused警告
    if rewriter.unused_parse_quote {
        enhanced_fn.block.stmts.insert(
            0,
            parse_quote! {
                if false {
                    let _: ::proc_macro2::TokenStream = parse_quote!{};
                }
            },
        );
    }

    quote! {
        #[cfg(debug_assertions)]
        #enhanced_fn

        #[cfg(not(debug_assertions))]
        #original_fn
    }
}

#[allow(dead_code)]
struct TraceRewriter {
    rewrite_quote: bool,
    rewrite_parse_quote: bool,
    unused_parse_quote: bool,
}

impl TraceRewriter {
    fn new() -> Self {
        Self {
            rewrite_quote: true,
            rewrite_parse_quote: true,
            unused_parse_quote: false,
        }
    }

    fn rewrite_macro_path(&mut self, mac: &mut Macro) {
        let last_segment = mac.path.segments.last();
        let Some(last) = last_segment else { return };
        let last_ident_str = last.ident.to_string();

        if self.rewrite_parse_quote && last_ident_str == "parse_quote" {
            self.unused_parse_quote = true;

            let origin_span = last.ident.span();

            let pkg = crate_name(origin_span);

            mac.path = parse_quote_spanned! {origin_span=>
                #pkg::__private::parse_quote
            };
        }
    }
}

impl VisitMut for TraceRewriter {
    fn visit_macro_mut(&mut self, mac: &mut Macro) {
        visit_mut::visit_macro_mut(self, mac);

        self.rewrite_macro_path(mac);
    }
}

pub fn parse_quote_impl(input: TokenStream) -> TokenStream {
    let pkg = crate_name(Span::call_site());
    quote! {#pkg::__private::parse_quote_traced(#pkg::__private::quote! {#input})}
}
