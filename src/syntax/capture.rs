use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::TokenStreamExt;
use syn::{
    Ident, Path, Token, Type, braced, bracketed, parenthesized,
    parse::{Parse, ParseStream, Parser, discouraged::Speculative},
    parse_quote,
    spanned::Spanned,
    token,
};

use crate::{
    ast::{
        capture::{Binder, Capture, EnumVariant, Matcher, MatcherKind, Quantity},
        keyword::Keyword,
        node::{Pattern, PatternKind},
    },
    syntax::context::{CaptureMode, ParseContext},
};

/// 捕获 #(...)
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
            let ty: Type = content.parse()?;
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
            // 具名捕获 <name: Capture> 与 行内捕获 <@: Capture> 及变体

            let i = ctx.inline_counter;
            let binder = if lookahead.peek(Ident) {
                let ident: Ident = content.parse()?;
                // 如果是行内捕获，那么报错
                if ctx.capture_mode == CaptureMode::Inline {
                    return Err(syn::Error::new(
                        ident.span(),
                        "unexpected named capture; previous captures were inline",
                    ));
                }
                ctx.capture_mode = CaptureMode::Named;
                Binder::Named(ident)
            } else {
                let _at = content.parse::<Token![@]>()?;
                // 如果是命名捕获，那么报错
                if ctx.capture_mode == CaptureMode::Named {
                    return Err(syn::Error::new(
                        _at.span(),
                        "unexpected inline capture; previous captures were named",
                    ));
                }
                ctx.capture_mode = CaptureMode::Inline;
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
                        return Err(syn::Error::new(
                            separator_tokens.span(),
                            "expected '[<separator>]' like '[,]'",
                        ));
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
            // 仅是一个 #，作为符号
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
            // 如果是 #(...)，则解析为 Capture
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
            if input.peek2(token::Brace) {
                let enum_name: Type = input.parse()?;
                let start_span = enum_name.span();

                let inner;
                let _brace = braced!(inner in input);
                let variants = inner.parse_terminated(EnumVariant::parse, Token![,])?;

                let span = if let Some(v) = variants.last() {
                    start_span.join(v.span()).unwrap_or(start_span)
                } else {
                    start_span
                };
                let variants = variants
                    .iter()
                    .map(|v| {
                        (
                            v.clone(),
                            Matcher {
                                span: v.span(),
                                kind: match &v {
                                    EnumVariant::Type { ty, .. } => {
                                        MatcherKind::SynType(ty.clone())
                                    }
                                    EnumVariant::Capture { pattern, .. } => {
                                        MatcherKind::Nested(vec![pattern.clone()])
                                    }
                                },
                            },
                        )
                    })
                    .collect();
                return Ok(Matcher {
                    kind: MatcherKind::Enum {
                        enum_name,
                        variants,
                    },
                    span,
                });
            }
            let ty: Type = input.parse()?;
            let span = ty.span();
            Matcher {
                kind: MatcherKind::SynType(ty),
                span,
            }
        } else {
            let pattern: Pattern = Pattern::parse(input)?;
            let span = pattern.span;
            // 对于单一的空组，进行拆包，减少一层嵌套
            if let PatternKind::Group {
                delimiter: Delimiter::None,
                children,
            } = pattern.kind
            {
                Matcher {
                    kind: MatcherKind::Nested(children),
                    span,
                }
            } else {
                Matcher {
                    kind: MatcherKind::Nested(vec![pattern]),
                    span,
                }
            }
        };
        if !input.is_empty() {
            let start_span = cap.span;
            match cap.kind {
                MatcherKind::SynType(_) | MatcherKind::Enum { .. } => Err(syn::Error::new(
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

impl Parse for EnumVariant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // 需要支持 Type | TypeName: Type | TypeName: Pattern

        // 可能是Type或TypeName
        let fork = input.fork();
        // 尝试解析为Type，如果解析后是','或空，则结束
        if let Ok(ty) = fork.parse::<Type>()
            && (fork.is_empty() || fork.peek(Token![,]))
        {
            input.advance_to(&fork);
            // 如果是Type，那么必须是可简写的模式，可解析为Path
            let path: Path = parse_quote!(#ty);
            let ident = path.segments.last().unwrap();
            let ident = parse_quote!(#ident);
            return Ok(EnumVariant::Type { ident, ty });
        }

        let ident: Ident = input.parse()?;
        let ident: Type = parse_quote!(#ident);

        // 否则需要是 ':'
        let _colon: Token![:] = input.parse()?;
        let fork = input.fork();
        // 可能是Type或Pattern
        if let Ok(ty) = fork.parse::<Type>() {
            input.advance_to(&fork);
            Ok(EnumVariant::Type { ident, ty })
        } else {
            let fork = input.fork();
            // 如果是Pattern，那需要确认边界，即找到最近的 ','
            // 否则Pattern会贪婪匹配，将其他分支吞掉

            // 如果Pattern中有逗号，可能会导致边界出错
            // 这要求Pattern中的','必须包裹在Group中
            let mut tokens = TokenStream::new();
            while !fork.peek(Token![,]) && !fork.is_empty() {
                tokens.append(fork.parse::<TokenTree>()?);
            }
            let parser = |input: ParseStream| -> syn::Result<Pattern> { Pattern::parse(&input) };
            let pattern = parser.parse2(tokens)?;
            input.advance_to(&fork);
            let captures = pattern.collect_captures();
            let named = if let Some(cap) = captures.first() {
                !cap.is_inline
            } else {
                false
            };
            Ok(EnumVariant::Capture {
                ident,
                named,
                fields: captures,
                pattern,
            })
        }
    }
}

impl EnumVariant {
    fn span(&self) -> proc_macro2::Span {
        match self {
            EnumVariant::Capture { ident, pattern, .. } => {
                ident.span().join(pattern.span).unwrap_or(ident.span())
            }
            EnumVariant::Type { ident, ty } => ident.span().join(ty.span()).unwrap_or(ident.span()),
        }
    }
}
