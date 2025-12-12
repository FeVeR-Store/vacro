use std::sync::{Arc, Mutex};

use syn::{Ident, Token, Type, bracketed, parenthesized, parse_quote, spanned::Spanned, token};

use crate::{
    ast::{
        capture::{CaptureMode, CaptureSpec, CaptureType, ExposeMode},
        keyword::Keyword,
        pattern::{Pattern, PatternList},
    },
    syntax::context::ParseContext,
};

impl CaptureSpec {
    pub fn parse(input: syn::parse::ParseStream, ctx: &mut ParseContext) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let fork = input.fork();
        if fork.parse::<Type>().is_ok() && fork.is_empty() {
            // 匿名捕获 <Capture> 类型
            let ty = CaptureType::parse(input, ctx)?;
            let mode = CaptureMode::Once;
            Ok(CaptureSpec {
                name: ExposeMode::Anonymous,
                ty,
                mode,
            })
        } else if lookahead.peek(Ident) || lookahead.peek(Token![@]) {
            let i = ctx.inline_counter;
            let inline_mode = ctx.inline_mode;
            let name: ExposeMode = if lookahead.peek(Ident) {
                let ident: Ident = input.parse()?;
                if i != 0 {
                    return Err(syn::Error::new(
                        ident.span(),
                        "unexpected named capture; previous captures were inline",
                    ));
                }
                if !inline_mode {
                    ctx.inline_mode = false;
                }
                ExposeMode::Named(ident)
            } else {
                let _at = input.parse::<Token![@]>()?;
                if inline_mode {
                    return Err(syn::Error::new(
                        _at.span(),
                        "unexpected inline capture; previous captures were named",
                    ));
                }
                ctx.inline_counter += 1;
                ExposeMode::Inline(i)
            };
            let mut mode = CaptureMode::Once;
            if input.peek(Token![?]) {
                mode = CaptureMode::Optional;
                input.parse::<Token![?]>()?;
            } else if input.peek(Token![*]) {
                input.parse::<Token![*]>()?;
                if input.peek(token::Bracket) {
                    let content;
                    let _br = bracketed!(content in input);
                    if content.is_empty() {
                        return Err(input.error("expected '[<separator>]' like '[,]'"));
                    }
                    let separater = Keyword::parse(&content, ctx)?;
                    mode = CaptureMode::Iter(separater);
                } else {
                    return Err(input.error("expected '[<separator>]' like '[,]'"));
                };
            }
            if input.peek(Token![:]) {
                let _colon = input.parse::<Token![:]>()?;
                let ty: Type = input.parse()?;
                Ok(CaptureSpec {
                    name,
                    ty: CaptureType::Type(ty),
                    mode,
                })
            } else {
                Err(input.error("expected ':' after capture name"))
            }
        } else {
            let mut mode = CaptureMode::Once;
            if input.peek(Token![?]) {
                mode = CaptureMode::Optional;
                input.parse::<Token![?]>()?;
            } else if input.peek(Token![*]) {
                input.parse::<Token![*]>()?;
                if input.peek(token::Bracket) {
                    let content;
                    let _br = bracketed!(content in input);
                    if content.is_empty() {
                        return Err(input.error("expected '[<separator>]' like '[,]'"));
                    }
                    let separater = Keyword::parse(&content, ctx)?;
                    mode = CaptureMode::Iter(separater);
                } else {
                    return Err(input.error("expected '[<separator>]' like '[,]'"));
                };
            }
            let _colon = input.parse::<Token![:]>()?;
            let ty = CaptureType::parse(input, ctx)?;
            Ok(CaptureSpec {
                name: ExposeMode::Anonymous,
                ty,
                mode,
            })
        }
    }
}

impl CaptureType {
    pub fn parse(input: syn::parse::ParseStream, ctx: &mut ParseContext) -> syn::Result<Self> {
        let cap = if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            if !input.peek(token::Paren) {
                let mut pattern_list = PatternList::parse(input)?;
                pattern_list
                    .list
                    .insert(0, Pattern::Literal(Keyword::Rust(String::from("#"))));
                return Ok(Self::Joint(pattern_list));
            }
            let content;
            let _paren = parenthesized!(content in input);
            let spec = CaptureSpec::parse(&content, ctx)?;

            Self::Joint(PatternList {
                list: vec![Pattern::Capture(spec, None)],
                capture_list: Arc::new(Mutex::new(vec![])),
                parse_context: ParseContext::default(),
            })
        } else if input.peek(Ident) {
            let ident: Type = input.parse()?;
            Self::Type(parse_quote!(#ident))
        } else {
            let pattern_list = PatternList::parse(input)?;
            Self::Joint(pattern_list)
        };
        if !input.is_empty() {
            match cap {
                CaptureType::Type(_) => Err(syn::Error::new(
                    input.span(),
                    format!("Unexpected '{}'", input.to_string()),
                )),
                CaptureType::Joint(mut joint) => {
                    let pattern_list = PatternList::parse(input)?;
                    joint.list.extend(pattern_list.list);
                    Ok(Self::Joint(joint))
                }
            }
        } else {
            Ok(cap)
        }
    }
}
