use std::sync::{Arc, Mutex};

use proc_macro2::{Delimiter, TokenStream};
use quote::quote;

use crate::{
    ast::{
        keyword::KeywordMap,
        pattern::{Pattern, PatternList},
    },
    codegen::{logic::Compiler, output::generate_output},
};

impl Compiler {
    pub fn compile_pattern_list(&mut self, pattern_list: &PatternList) -> TokenStream {
        let mut tokens = TokenStream::new();
        let mut keyword_map = KeywordMap::new();
        // 1. 创建一个临时的 Buffer 来存放主体逻辑代码
        let mut body_stream = proc_macro2::TokenStream::new();

        pattern_list.list.iter().for_each(|pattern| {
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

                        let pattern_token = self.compile_pattern_list(patterns);
                        let (capture_init, struct_def, struct_expr) = generate_output(patterns.capture_list.clone(), None, &pattern_list.parse_context);

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

                        pattern_list.capture_list
                            .lock()
                            .unwrap()
                            .extend(patterns.capture_list.lock().unwrap().clone());
                    }
                    Capture(cap_spec, edge) => {
                        let capture_list = Arc::new(Mutex::new(Vec::new()));
                        cap_spec.add_capture(capture_list.clone());
                        let (capture_init, struct_def, struct_expr) = generate_output(capture_list, None, &pattern_list.parse_context);
                        let cap_tokens = self.compile_capture_spec(cap_spec);
                        match edge {
                            Some(keyword) => {
                                // 3. 你的 Lookahead 逻辑，现在追加到 body_stream
                                body_stream.extend(quote! {
                                    {
                                        let mut _input = ::proc_macro2::TokenStream::new();

                                        while !input.peek(#keyword) {
                                            _input.extend(::std::iter::once(
                                                input.parse::<::proc_macro2::TokenTree>()?
                                            ));
                                        }

                                        {
                                            #struct_def
                                            let parser = |input: ::syn::parse::ParseStream| -> ::syn::Result<Output> {
                                                #capture_init
                                                #cap_tokens
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
                                        #cap_tokens
                                    }
                                });
                            }
                        };
                        cap_spec.add_capture(pattern_list.capture_list.clone());
                    }
                }
            });
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
