use proc_macro2::{Delimiter, TokenStream};
use quote::quote;

use crate::{
    ast::{
        keyword::KeywordMap,
        node::{Pattern, PatternKind},
    },
    codegen::{logic::Compiler, output::generate_output},
};

impl Compiler {
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
                let (capture_init, struct_def, struct_expr, ..) = generate_output(&captures, None);

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
                        #struct_expr = parser(&_input)?;
                    }
                });
            }
            PatternKind::Capture(capture) => {
                let captures = capture.collect_captures();
                let (capture_init, struct_def, struct_expr, ..) = generate_output(&captures, None);
                let cap_tokens = self.compile_capture(capture);
                match &capture.edge {
                    Some(keyword) => {
                        // 3. Lookahead 逻辑，现在追加到 body_stream
                        body_stream.extend(quote! {
                                    {
                                        let mut _input = ::proc_macro2::TokenStream::new();
                                        while !input.peek(#keyword) {
                                            _input.extend(::std::iter::once(
                                                input.parse::<::proc_macro2::TokenTree>()?
                                            ));
                                        }

                                        #struct_def
                                        let parser = |input: ::syn::parse::ParseStream| -> ::syn::Result<Output> {
                                            #capture_init
                                            #cap_tokens
                                            ::std::result::Result::Ok(#struct_expr)
                                        };
                                        // 这里解析刚才吃进去的流
                                        #struct_expr = ::syn::parse::Parser::parse2(parser, _input)?;
                                    }
                                });
                    }
                    None => {
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
            {
                #keyword_map_tokens
                #body_stream
            }
        });
        tokens
    }
}
