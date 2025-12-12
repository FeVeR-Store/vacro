use std::sync::{Arc, Mutex};

use proc_macro2::{Delimiter, Punct, Spacing};
use syn::{Ident, Token, braced, bracketed, ext::IdentExt, parenthesized, token};

use crate::{
    ast::{
        capture::CaptureSpec,
        keyword::Keyword,
        pattern::{Pattern, PatternList},
    },
    syntax::{context::ParseContext, keyword::parse_keyword},
};

impl PatternList {
    pub fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut ctx = ParseContext::default();
        let mut pattern_list = vec![];

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![#]) {
                input.parse::<Token![#]>()?;
                if !input.peek(token::Paren) {
                    pattern_list.push(Pattern::Literal(Keyword::Rust(String::from("#"))));
                    continue;
                }
                let content;
                let _paren = parenthesized!(content in input);
                let spec = CaptureSpec::parse(&content, &mut ctx)?;
                pattern_list.push(Pattern::Capture(spec, None));
            } else if lookahead.peek(Ident::peek_any) {
                let id = Ident::parse_any(input)?;
                let keyword = parse_keyword(id, &mut ctx);
                pattern_list.push(Pattern::Literal(keyword));
            } else if lookahead.peek(token::Brace)
                || lookahead.peek(token::Bracket)
                || lookahead.peek(token::Paren)
            {
                let content;
                let delimiter;
                if lookahead.peek(token::Brace) {
                    let _brace = braced!(content in input);
                    delimiter = Delimiter::Brace;
                } else if lookahead.peek(token::Bracket) {
                    let _bracket = bracketed!(content in input);
                    delimiter = Delimiter::Bracket;
                } else if lookahead.peek(token::Paren) {
                    let _paren = parenthesized!(content in input);
                    delimiter = Delimiter::Parenthesis;
                } else {
                    return Err(syn::Error::new(input.span(), "Unexpected token"));
                }
                let inner: PatternList = PatternList::parse(&content)?;
                pattern_list.push(Pattern::Group(delimiter, inner));
            } else {
                let mut collect = String::new();
                let mut punct: Punct = input.parse()?;
                while punct.spacing() == Spacing::Joint {
                    if input.peek(Token![#]) {
                        break;
                    }
                    collect.push(punct.as_char());
                    punct = input.parse()?;
                }
                collect.push(punct.as_char());
                pattern_list.push(Pattern::Literal(parse_keyword(collect, &mut ctx)));
            }
        }
        Ok(PatternList {
            list: pattern_list,
            capture_list: Arc::new(Mutex::new(vec![])),
            parse_context: ctx,
        })
    }
}
