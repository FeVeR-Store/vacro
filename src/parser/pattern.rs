use std::sync::{Arc, Mutex};

use proc_macro2::{Delimiter, Punct, Spacing, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Ident, Token, Type, braced, bracketed, ext::IdentExt, parenthesized, parse::Parse, token,
};

use crate::parser::{
    capture_group::CaptureSpec,
    keyword::{Keyword, KeywordMap, parse_keyword},
    output::generate_output,
};

pub type IsOptional = bool;

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub struct PatternList {
    pub list: Vec<Pattern>,
    pub capture_list: Arc<Mutex<Vec<(Ident, Type, IsOptional)>>>,
}

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub enum Pattern {
    // 关键字/符号：fn, struct, ;
    Literal(Keyword),
    // 括号组：( ... ), { ... }
    Group(Delimiter, PatternList),
    // 捕获：#( ... )
    Capture(CaptureSpec, Option<Keyword>),
}

impl Parse for PatternList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
                let spec: CaptureSpec = content.parse()?;
                pattern_list.push(Pattern::Capture(spec, None));
            } else if lookahead.peek(Ident::peek_any) {
                let id = Ident::parse_any(input)?;
                let keyword = parse_keyword(id);
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
                let inner: PatternList = content.parse()?;
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
                pattern_list.push(Pattern::Literal(parse_keyword(collect)));
            }
        }
        Ok(PatternList {
            list: pattern_list,
            capture_list: Arc::new(Mutex::new(vec![])),
        })
    }
}

impl ToTokens for PatternList {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut keyword_map = KeywordMap::new();
        // 1. 创建一个临时的 Buffer 来存放主体逻辑代码
        let mut body_stream = proc_macro2::TokenStream::new();

        self.list.iter().for_each(|pattern| {
            use Pattern::*;
            match pattern {
                Literal(keyword) => {
                    keyword.define(&mut keyword_map);
                    // 2. 使用 extend 追加到 body_stream，而不是替换
                    body_stream.extend(quote! {
                        input.parse::<#keyword>()?;
                    });
                }
                Group(delimiter, patterns) => {
                    let mac: proc_macro2::TokenStream = match delimiter {
                        Delimiter::Brace => quote! { ::syn::braced! },
                        Delimiter::Bracket => quote! { ::syn::bracketed! },
                        Delimiter::Parenthesis => quote! { ::syn::parenthesized! },
                        Delimiter::None => quote! {},
                    };

                    let pattern_token = quote! {#patterns};
                    let (capture_init, struct_def, struct_expr) = generate_output(patterns.capture_list.clone(), None);

                    // 追加到 body_stream
                    body_stream.extend(quote! {
                        {
                            #struct_def
                            let _input;
                            let _ = #mac(_input in input);
                            let parser = |input: ::syn::parse::ParseBuffer<'_>| -> ::syn::Result<Output> {
                                #capture_init
                                #pattern_token
                                ::std::result::Result::Ok(#struct_expr)
                            };
                            #struct_expr = parser(_input)?;
                        }
                    });

                    self.capture_list
                        .lock()
                        .unwrap()
                        .extend(patterns.capture_list.lock().unwrap().clone());
                }
                Capture(cap, edge) => {
                    let capture_list = Arc::new(Mutex::new(Vec::new()));
                    cap.add_capture(capture_list.clone());
                    let (capture_init, struct_def, struct_expr) = generate_output(capture_list, None);

                    match edge {
                        Some(keyword) => {
                            // 3. 你的 Lookahead 逻辑，现在追加到 body_stream
                            body_stream.extend(quote! {
                                {
                                    // 这里使用 _input 可能会与 Group 里的 _input 冲突
                                    // 建议改名为 lookahead_buf 之类的，或者依赖 Rust 的块级作用域遮蔽
                                    let mut _input = ::proc_macro2::TokenStream::new();

                                    // 注意：这里 input.peek 消耗 Token 吗？
                                    // 如果 input 是 ParseStream，peek 不消耗。
                                    // 下面的 parse 消耗。逻辑看起来是对的：只要不是 keyword，就吃到 _input 里。
                                    while !input.peek(#keyword) {
                                        _input.extend(::std::iter::once(
                                            input.parse::<::proc_macro2::TokenTree>()?
                                        ));
                                    }

                                    {
                                        #struct_def
                                        let parser = |input: ::syn::parse::ParseStream| -> ::syn::Result<Output> {
                                            #capture_init
                                            #cap
                                            ::std::result::Result::Ok(#struct_expr)
                                        };
                                        // 这里解析刚才吃进去的流
                                        #struct_expr = ::syn::parse::Parser::parse2(parser, _input)?;
                                    }
                                }
                            });
                        }
                        None => {
                            body_stream.extend(quote! {
                                {
                                    #cap
                                }
                            });
                        }
                    };
                    cap.add_capture(self.capture_list.clone());
                }
            }
        });

        // 4. 最后一次性把所有东西包装起来塞给 tokens
        tokens.extend(quote! {
            {
                #keyword_map
                #body_stream
            }
        });
    }
}
