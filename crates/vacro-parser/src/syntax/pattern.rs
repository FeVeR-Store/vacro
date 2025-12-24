use proc_macro2::{Delimiter, Group, Punct, Spacing, TokenStream, TokenTree};
use quote::TokenStreamExt;
use syn::{
    braced, bracketed,
    ext::IdentExt,
    parenthesized,
    parse::{ParseStream, Parser},
    spanned::Spanned,
    token, Ident, Result, Token,
};

use crate::{
    ast::{
        capture::Capture,
        keyword::Keyword,
        node::{Pattern, PatternKind},
    },
    syntax::{context::ParseContext, keyword::parse_keyword},
};

impl Pattern {
    pub fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut ctx = ParseContext::default();
        let mut pattern_list = vec![];
        let start_span = input.span();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![#]) {
                if !input.peek2(token::Paren) {
                    let _hash_tag: Keyword = Keyword::parse(input, &mut ctx)?;
                    let start_span = _hash_tag.span();
                    let pattern = Pattern {
                        kind: PatternKind::Literal(_hash_tag),
                        span: start_span,
                        meta: None,
                    };
                    pattern_list.push(pattern);
                    continue;
                }
                let _hash_tag = input.parse::<Token![#]>()?;
                let content;
                let _paren = parenthesized!(content in input);
                let inner: TokenStream = content.parse()?;

                let mut content = TokenStream::new();
                let mut hash_punct = Punct::new('#', Spacing::Alone);
                hash_punct.set_span(_hash_tag.span);

                let mut group = Group::new(Delimiter::Parenthesis, inner);
                group.set_span(_paren.span.span());

                content.extend([TokenTree::Punct(hash_punct), TokenTree::Group(group)]);

                let parser = |content: ParseStream| -> Result<Capture> {
                    Capture::parse(&content, &mut ctx)
                };
                let capture = parser.parse2(content)?;
                let span = capture.span;
                let pattern = Pattern {
                    kind: PatternKind::Capture(capture),
                    span,
                    meta: None,
                };
                pattern_list.push(pattern);
            } else if lookahead.peek(Ident::peek_any) {
                let id = Ident::parse_any(input)?;
                let keyword = parse_keyword(id, &mut ctx);
                let span = keyword.span();
                pattern_list.push(Pattern {
                    kind: PatternKind::Literal(keyword),
                    span,
                    meta: None,
                });
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
                let inner: Pattern = Pattern::parse(&content)?;
                let span = inner.span;
                let pattern = Pattern {
                    kind: PatternKind::Group {
                        delimiter,
                        children: vec![inner],
                    },
                    span,
                    meta: None,
                };
                pattern_list.push(pattern);
            } else {
                let mut collect = TokenStream::new();
                let mut punct: Punct = input.parse()?;
                let start_span = punct.span();
                while punct.spacing() == Spacing::Joint {
                    if input.peek(Token![#]) {
                        break;
                    }
                    collect.append(punct);
                    punct = input.parse()?;
                }
                let end_span = punct.span();
                collect.append(punct);
                let keyword = parse_keyword(collect, &mut ctx);
                let pattern = Pattern {
                    kind: PatternKind::Literal(keyword),
                    span: start_span.join(end_span).unwrap_or(end_span),
                    meta: None,
                };
                pattern_list.push(pattern);
            }
        }
        let end_span = if let Some(pattern) = pattern_list.last() {
            pattern.span
        } else {
            start_span
        };
        Ok(Pattern {
            kind: PatternKind::Group {
                delimiter: Delimiter::None,
                children: pattern_list,
            },
            span: start_span.join(end_span).unwrap_or(start_span),
            meta: None,
        })
    }
}
