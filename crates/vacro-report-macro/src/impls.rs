use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse_quote, parse_quote_spanned,
    visit_mut::{self, VisitMut},
    Block, ItemFn, Macro, Stmt,
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

    snapshot!("fn", enhanced_fn);
    let mut stmts: Vec<Stmt> = vec![];

    if rewriter.unused_parse_quote {
        stmts.push(parse_quote! {let _: ::proc_macro2::TokenStream = parse_quote!();});
    }
    if rewriter.unused_parse_quote_spanned {
        stmts.push(parse_quote! { let span = ::proc_macro2::Span::call_site(); });
        stmts.push(
            parse_quote! { let _: ::proc_macro2::TokenStream = parse_quote_spanned!(span=>()); },
        );
    }

    // 补全替换parse_quote可能导致的unused警告
    if !stmts.is_empty() {
        let mut block: Block = parse_quote!({});
        block.stmts.append(&mut stmts);
        snapshot!("block", block);
        enhanced_fn.block.stmts.insert(
            0,
            parse_quote! {
                if false { #block }
            },
        );
    }
    snapshot!("fn", enhanced_fn);

    quote! {
        #[cfg(debug_assertions)]
        #enhanced_fn

        #[cfg(not(debug_assertions))]
        #original_fn
    }
}

#[allow(dead_code)]
struct TraceRewriter {
    unused_parse_quote: bool,
    unused_parse_quote_spanned: bool,
}

impl TraceRewriter {
    fn new() -> Self {
        Self {
            unused_parse_quote: false,
            unused_parse_quote_spanned: false,
        }
    }

    fn rewrite_macro_path(&mut self, mac: &mut Macro) {
        let len = mac.path.segments.len();
        let last_segment = mac.path.segments.last();
        let Some(last) = last_segment else { return };
        let last_ident_str = last.ident.to_string();

        if last_ident_str == "parse_quote" || last_ident_str == "parse_quote_spanned" {
            let origin_span = last.ident.span();
            let pkg = crate_name(origin_span);
            let parse_quote_token = if last_ident_str == "parse_quote" {
                self.unused_parse_quote = len == 1;
                quote_spanned! { origin_span => parse_quote }
            } else {
                self.unused_parse_quote_spanned = len == 1;
                quote_spanned! { origin_span => parse_quote_spanned }
            };

            mac.path = parse_quote_spanned! {origin_span=>
                #pkg::__private::#parse_quote_token
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

pub fn parse_quote_impl(input: TokenStream, spanned: bool) -> TokenStream {
    let pkg = crate_name(Span::call_site());
    let quote_token = if spanned {
        quote! {quote_spanned!}
    } else {
        quote! {quote!}
    };
    quote! {#pkg::__private::parse_quote_traced(#pkg::__private::#quote_token {#input}, #spanned)}
}
