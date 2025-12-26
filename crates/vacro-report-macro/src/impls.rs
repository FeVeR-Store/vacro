use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote, parse_quote_spanned,
    visit_mut::{self, VisitMut},
    ItemFn, Macro,
};

pub fn report_scope_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f: ItemFn = parse_quote!(#item);

    // 改写函数体里的宏调用
    let mut rewriter = TraceRewriter::new();
    rewriter.visit_item_fn_mut(&mut f);

    quote!(#f)
}

#[allow(dead_code)]
struct TraceRewriter {
    rewrite_quote: bool,
    rewrite_parse_quote: bool,
}

impl TraceRewriter {
    fn new() -> Self {
        Self {
            rewrite_quote: true,
            rewrite_parse_quote: true,
        }
    }

    fn rewrite_macro_path(&self, mac: &mut Macro) {
        let last_segment = mac.path.segments.last();
        let Some(last) = last_segment else { return };
        let last_ident_str = last.ident.to_string();

        if self.rewrite_parse_quote && last_ident_str == "parse_quote" {
            let origin_span = last.ident.span();

            mac.path = if cfg!(feature = "standalone") {
                parse_quote_spanned! {origin_span=>
                    ::vacro_report::__private::parse_quote
                }
            } else {
                parse_quote_spanned! {origin_span=>
                    ::vacro::report::__private::parse_quote
                }
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
    if cfg!(feature = "standalone") {
        quote! {::vacro_report::__private::parse_quote(::quote::quote! {#input})}
    } else {
        quote! {::vacro::report::__private::parse_quote_traced(::quote::quote! {#input})}
    }
}
