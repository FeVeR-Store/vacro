use syn::{Ident, Token, Type, bracketed, parenthesized, spanned::Spanned, token};

use crate::{
    ast::{
        capture::{Binder, Capture, Matcher, MatcherKind, Quantity},
        keyword::Keyword,
        node::{Pattern, PatternKind},
    },
    syntax::context::ParseContext,
};

impl Capture {
    pub fn parse(input: syn::parse::ParseStream, ctx: &mut ParseContext) -> syn::Result<Self> {
        let _hash_tag: Token![#] = input.parse()?;
        let start_span = _hash_tag.span;
        let content;
        let _paren = parenthesized!(content in input);

        let lookahead = content.lookahead1();
        let fork = content.fork();
        if fork.parse::<Type>().is_ok() && fork.is_empty() {
            // 匿名捕获 <Capture> 类型
            let ty: Type = input.parse()?;
            let end_span = ty.span();
            let matcher = Matcher {
                kind: MatcherKind::SynType(ty),
                span: start_span.join(end_span).unwrap_or(start_span),
            };
            let quantity = Quantity::One;
            let binder = Binder::Anonymous;
            Ok(Capture {
                _hash_tag,
                _paren,
                matcher,
                quantity,
                binder,
                edge: None,

                span: start_span.join(end_span).unwrap_or(start_span),
            })
        } else if lookahead.peek(Ident) || lookahead.peek(Token![@]) {
            let i = ctx.inline_counter;
            let inline_mode = ctx.inline_mode;
            let binder = if lookahead.peek(Ident) {
                let ident: Ident = content.parse()?;
                if i != 0 {
                    return Err(syn::Error::new(
                        ident.span(),
                        "unexpected named capture; previous captures were inline",
                    ));
                }
                if !inline_mode {
                    ctx.inline_mode = false;
                }
                Binder::Named(ident)
            } else {
                let _at = content.parse::<Token![@]>()?;
                if inline_mode {
                    return Err(syn::Error::new(
                        _at.span(),
                        "unexpected inline capture; previous captures were named",
                    ));
                }
                ctx.inline_counter += 1;
                Binder::Inline(i)
            };
            let mut quantity = Quantity::One;
            if content.peek(Token![?]) {
                quantity = Quantity::Optional;
                content.parse::<Token![?]>()?;
            } else if content.peek(Token![*]) {
                content.parse::<Token![*]>()?;
                if content.peek(token::Bracket) {
                    let separator_tokens;
                    let _br = bracketed!(separator_tokens in content);
                    if separator_tokens.is_empty() {
                        return Err(separator_tokens.error("expected '[<separator>]' like '[,]'"));
                    }
                    let separater = Keyword::parse(&separator_tokens, ctx)?;
                    quantity = Quantity::Many(Some(separater));
                } else {
                    return Err(content.error("expected '[<separator>]' like '[,]'"));
                };
            }
            if content.peek(Token![:]) {
                let _colon = content.parse::<Token![:]>()?;
                let matcher = Matcher::parse(&content, ctx)?;
                let end_span = matcher.span;
                Ok(Capture {
                    _hash_tag,
                    _paren,
                    binder,
                    matcher,
                    quantity,
                    edge: None,
                    span: start_span.join(end_span).unwrap_or(start_span),
                })
            } else {
                Err(content.error("expected ':' after capture name"))
            }
        } else {
            let mut quantity = Quantity::One;
            if content.peek(Token![?]) {
                quantity = Quantity::Optional;
                content.parse::<Token![?]>()?;
            } else if content.peek(Token![*]) {
                content.parse::<Token![*]>()?;
                if content.peek(token::Bracket) {
                    let separater_tokens;
                    let _br = bracketed!(separater_tokens in content);
                    if separater_tokens.is_empty() {
                        return Err(separater_tokens.error("expected '[<separator>]' like '[,]'"));
                    }
                    let separater = Keyword::parse(&separater_tokens, ctx)?;
                    quantity = Quantity::Many(Some(separater));
                } else {
                    return Err(content.error("expected '[<separator>]' like '[,]'"));
                };
            }
            let _colon = content.parse::<Token![:]>()?;
            let matcher = Matcher::parse(&content, ctx)?;
            let end_span = matcher.span;
            Ok(Capture {
                _hash_tag,
                _paren,
                quantity,
                matcher,
                binder: Binder::Anonymous,
                edge: None,
                span: start_span.join(end_span).unwrap_or(start_span),
            })
        }
    }
}

impl Matcher {
    pub fn parse(input: syn::parse::ParseStream, ctx: &mut ParseContext) -> syn::Result<Self> {
        let cap = if input.peek(Token![#]) {
            if !input.peek2(token::Paren) {
                let _hash_tag = input.parse::<Token![#]>()?;
                let start_span = _hash_tag.span;
                let pattern: Pattern = Pattern::parse(input)?;
                let end_span = pattern.span;
                let hash_tag_pattern = Pattern {
                    kind: PatternKind::Literal(Keyword::Rust(String::from("#"))),
                    span: _hash_tag.span,
                    meta: None,
                };
                let matcher = Matcher {
                    kind: MatcherKind::Nested(vec![hash_tag_pattern, pattern]),
                    span: start_span.join(end_span).unwrap_or(start_span),
                };
                return Ok(matcher);
            }
            let capture = Capture::parse(&input, ctx)?;
            let span = capture.span;
            let pattern = Pattern {
                kind: PatternKind::Capture(capture),
                span,
                meta: None,
            };
            Matcher {
                kind: MatcherKind::Nested(vec![pattern]),
                span,
            }
        } else if input.peek(Ident) {
            let ty: Type = input.parse()?;
            let span = ty.span();
            Matcher {
                kind: MatcherKind::SynType(ty),
                span,
            }
        } else {
            let pattern: Pattern = Pattern::parse(input)?;
            let span = pattern.span;
            Matcher {
                kind: MatcherKind::Nested(vec![pattern]),
                span,
            }
        };
        if !input.is_empty() {
            let start_span = cap.span;
            match cap.kind {
                MatcherKind::SynType(_) => Err(syn::Error::new(
                    input.span(),
                    format!("Unexpected '{}'", input.to_string()),
                )),
                MatcherKind::Nested(mut pattern_list) => {
                    let pattern: Pattern = Pattern::parse(input)?;
                    let end_span = pattern.span;
                    pattern_list.push(pattern);
                    let matcher = Matcher {
                        kind: MatcherKind::Nested(pattern_list),
                        span: start_span.join(end_span).unwrap_or(start_span),
                    };
                    Ok(matcher)
                }
            }
        } else {
            Ok(cap)
        }
    }
}
