use proc_macro2::{Delimiter, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::LitInt;

use crate::{
    ast::{
        capture::Quantity,
        keyword::{Keyword, KeywordMap},
        node::{Pattern, PatternKind},
    },
    codegen::{logic::Compiler, output::generate_output},
    transform::lookahead::inject_lookahead,
};

impl Compiler {
    fn compile_edge_peek(&self, keyword: &Keyword) -> TokenStream {
        match keyword {
            Keyword::Custom {
                punctuation: true,
                content,
                ..
            } => {
                let parse_punct = content.chars().map(|ch| {
                    let punct = Keyword::Rust(ch.to_string());
                    quote! {
                        _edge_fork.parse::<#punct>()?;
                    }
                });

                quote! {
                    {
                        let _edge_fork = input.fork();
                        (|| -> ::syn::Result<()> {
                            #(#parse_punct)*
                            ::std::result::Result::Ok(())
                        })().is_ok()
                    }
                }
            }
            _ => quote! {
                input.peek(#keyword)
            },
        }
    }

    pub fn compile_pattern(&mut self, pattern: &Pattern) -> TokenStream {
        let mut tokens = TokenStream::new();
        let mut keyword_map = KeywordMap::new();
        // 1. 创建一个临时的 Buffer 来存放主体逻辑代码
        let mut body_stream = TokenStream::new();

        match &pattern.kind {
            PatternKind::Literal(keyword) => {
                keyword.define(&mut keyword_map);
                // 2. 使用 extend 追加到 body_stream，而不是替换
                body_stream.extend(quote! {
                    input.parse::<#keyword>()?;
                });
            }
            PatternKind::Group {
                delimiter,
                children,
            } => {
                let children = inject_lookahead(children.clone());
                let mac: proc_macro2::TokenStream = match delimiter {
                    Delimiter::Brace => quote! { ::syn::braced! },
                    Delimiter::Bracket => quote! { ::syn::bracketed! },
                    Delimiter::Parenthesis => quote! { ::syn::parenthesized! },
                    Delimiter::None => quote! {},
                };

                let mut pattern_token = TokenStream::new();
                pattern_token.extend(children.iter().map(|pattern| self.compile_pattern(pattern)));

                if matches!(delimiter, Delimiter::None) {
                    tokens.extend(pattern_token);
                    return tokens;
                }
                let captures = pattern.collect_captures();
                let (capture_init, struct_def, struct_expr, fields) =
                    generate_output(&captures, None, None);
                let assign_outputs = captures.iter().enumerate().map(|(i, capture)| {
                    let ident = &fields[i];
                    let access = if capture.is_inline {
                        LitInt::new(&i.to_string(), Span::call_site()).into_token_stream()
                    } else {
                        quote! { #ident }
                    };
                    quote! {
                        #ident = output.#access;
                    }
                });

                // 追加到 body_stream
                body_stream.extend(quote! {
                    {
                        #struct_def
                        let _input;
                        let _ = #mac(_input in input);
                        let parser = |input: ::syn::parse::ParseStream| -> ::syn::Result<Output> {
                            #capture_init
                            #pattern_token
                            ::std::result::Result::Ok(#struct_expr)
                        };
                        let output = parser(&_input)?;
                        #(#assign_outputs)*
                    }
                });
            }
            PatternKind::Capture(capture) => {
                let captures = capture.collect_captures();
                let (capture_init, struct_def, struct_expr, fields) =
                    generate_output(&captures, None, None);
                let assign_outputs = captures.iter().enumerate().map(|(i, capture)| {
                    let ident = &fields[i];
                    let access = if capture.is_inline {
                        LitInt::new(&i.to_string(), Span::call_site()).into_token_stream()
                    } else {
                        quote! { #ident }
                    };
                    quote! {
                        #ident = output.#access;
                    }
                });
                let cap_tokens = self.compile_capture(capture);
                match &capture.edge {
                    Some(keyword) if !matches!(capture.quantity, Quantity::Optional) => {
                        let edge_peek = self.compile_edge_peek(keyword);
                        // 3. Lookahead 逻辑，现在追加到 body_stream
                        body_stream.extend(quote! {
                            {
                                let mut _input_tokens = ::std::vec::Vec::<::proc_macro2::TokenTree>::new();
                                while !(#edge_peek) {
                                    _input_tokens.push(input.parse::<::proc_macro2::TokenTree>()?);
                                }
                                if let ::std::option::Option::Some(::proc_macro2::TokenTree::Punct(_punct)) =
                                    _input_tokens.last_mut()
                                {
                                    if _punct.spacing() == ::proc_macro2::Spacing::Joint {
                                        let mut _alone = ::proc_macro2::Punct::new(
                                            _punct.as_char(),
                                            ::proc_macro2::Spacing::Alone,
                                        );
                                        _alone.set_span(_punct.span());
                                        *_punct = _alone;
                                    }
                                }
                                let _input = _input_tokens
                                    .into_iter()
                                    .collect::<::proc_macro2::TokenStream>();

                                #struct_def
                                let parser = |input: ::syn::parse::ParseStream| -> ::syn::Result<Output> {
                                    #capture_init
                                    {
                                        #cap_tokens
                                    }
                                    ::std::result::Result::Ok(#struct_expr)
                                };
                                // 这里解析刚才吃进去的流
                                let output = ::syn::parse::Parser::parse2(parser, _input)?;
                                #(#assign_outputs)*
                            }
                        });
                    }
                    Some(_) | None => {
                        body_stream.extend(quote! {
                            {
                                #cap_tokens
                            }
                        });
                    }
                };
            }
        }

        let keyword_map_tokens = self.compile_keyword_map(keyword_map);
        // 4. 最后一次性把所有东西包装起来塞给 tokens
        tokens.extend(quote! {
            #keyword_map_tokens
            #body_stream
        });
        tokens
    }
}
