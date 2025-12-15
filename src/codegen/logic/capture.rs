use proc_macro2::{Delimiter, TokenStream};
use quote::{format_ident, quote};
use syn::{Token, parse_quote, punctuated::Punctuated, token::Comma};

use crate::{
    ast::{
        capture::{Binder, Capture, EnumVariant, FieldDef, Matcher, MatcherKind, Quantity},
        node::{Pattern, PatternKind},
    },
    codegen::{logic::Compiler, output::generate_output},
    transform::lookahead::inject_lookahead,
};

impl Compiler {
    pub fn compile_capture(&mut self, capture: &Capture) -> TokenStream {
        let mut tokens = TokenStream::new();
        let Capture {
            binder,
            matcher,
            quantity,
            span,
            ..
        } = capture;
        let receiver = match &binder {
            Binder::Named(ident) => {
                quote! {#ident = }
            }
            Binder::Inline(i) => {
                let id = format_ident!("_{}", i.to_string());
                quote! {#id = }
            }
            _ => quote! {},
        };
        let t = match (binder, quantity, &matcher.kind) {
            (_, Quantity::One, MatcherKind::Enum { .. } | MatcherKind::SynType(_)) => {
                let ty = self.compile_matcher(matcher);
                quote! {
                    {
                        #receiver input.parse::<#ty>()?;
                    }
                }
            }
            (_, Quantity::Optional, MatcherKind::Enum { .. } | MatcherKind::SynType(_)) => {
                let ty = self.compile_matcher(matcher);
                quote! {
                    {
                        let _fork = input.fork();
                        if ::std::result::Ok(_parsed) = _fork.parse::<#ty>() {
                            #receiver ::std::option::Option::Some(_parsed);
                        }
                    }
                }
            }
            (_, Quantity::Many(separator), MatcherKind::Enum { .. } | MatcherKind::SynType(_)) => {
                let ty = self.compile_matcher(matcher);
                quote! {
                    {
                        #[allow(non_local_definitions)]
                        impl _Parse for #ty {}
                        #receiver input.parse_terminated(#ty::parse, #separator)?;
                    }
                }
            }
            (Binder::Anonymous, Quantity::One, MatcherKind::Nested(_patterns)) => {
                let optimized_list = inject_lookahead(_patterns.clone());

                let patterns = Pattern {
                    kind: PatternKind::Group {
                        delimiter: Delimiter::None,
                        children: optimized_list,
                    },
                    span: *span,
                    meta: None,
                };
                let pattern_tokens = self.compile_pattern(&patterns);
                quote! {
                    {
                        #pattern_tokens
                    }
                }
            }
            (Binder::Anonymous, Quantity::Optional, MatcherKind::Nested(_patterns)) => {
                let optimized_list = inject_lookahead(_patterns.clone());

                let patterns = Pattern {
                    kind: PatternKind::Group {
                        delimiter: Delimiter::None,
                        children: optimized_list,
                    },
                    span: *span,
                    meta: None,
                };

                let joint_token = self.compile_pattern(&patterns);
                let captures = patterns.collect_captures();
                let (capture_init, struct_def, struct_expr, fields) =
                    generate_output(&captures, None);

                let assigns_err = fields.iter().map(|ident| {
                    quote! { #ident = ::std::option::Option::None; }
                });
                let assigns_ok = fields.iter().map(|ident| {
                    quote! { #ident = ::std::option::Option::Some(output.#ident); }
                });

                quote! {
                    #struct_def
                    let _parser = |input: ::syn::parse::ParseStream| -> ::syn::Result<Output> {
                        #capture_init
                        #joint_token
                        ::std::result::Result::Ok(#struct_expr)
                    };
                    match _parser(input) {
                        ::std::result::Result::Ok(output) => {
                            #(#assigns_ok)*
                        }
                        ::std::result::Result::Err(_) => {
                            #(#assigns_err)*
                        }
                    }
                    let _ = _parser(input);
                }
            }
            _ => quote! {},
        };
        tokens.extend(t);
        tokens
    }
    fn compile_matcher(&mut self, matcher: &Matcher) -> TokenStream {
        match &matcher.kind {
            MatcherKind::Enum {
                enum_name,
                variants,
            } => {
                let variants_struct: Punctuated<TokenStream, Token![,]> = variants
                    .iter()
                    .map(|(v, _)| match v {
                        EnumVariant::Capture {
                            ident,
                            named,
                            fields,
                            ..
                        } => {
                            let body = if *named {
                                let fields: Punctuated<_, Comma> = fields
                                    .iter()
                                    .map(|FieldDef { name, ty, .. }| quote! {#name: #ty})
                                    .collect();
                                quote! {
                                    {
                                        #fields
                                    }
                                }
                            } else {
                                let fields: Punctuated<_, Comma> =
                                    fields.iter().map(|FieldDef { ty, .. }| ty).collect();
                                quote! {
                                    (#fields)
                                }
                            };
                            quote! { #ident #body }
                        }
                        EnumVariant::Type { ident, ty } => {
                            quote! {#ident(#ty)}
                        }
                    })
                    .collect();
                let enum_def = parse_quote! {
                    pub enum #enum_name {
                        #variants_struct
                    }
                };

                let parser = variants.iter().map(|(v, ..)| {
                    match v {
                        EnumVariant::Type { ident, ty } => {
                            quote! {
                                let _fork = input.fork();
                                if let ::std::result::Result::Ok(v) = _fork.parse::<#ty>() {
                                    ::syn::parse::discouraged::Speculative::advance_to(input, &_fork);
                                    return ::std::result::Result::Ok(#enum_name::#ident(v));
                                };
                            }
                        }
                        EnumVariant::Capture {
                            ident,
                            fields,
                            pattern,
                            named,
                            ..
                        } => {
                            let (capture_init, _, _, capture_list) =
                                generate_output(&fields, None);
                            let pattern_tokens = self.compile_pattern(pattern);
                            let enum_expr_body = capture_list.iter().collect::<Punctuated<_,Token![,]>>();
                            let enum_expr = if *named { quote! {{#enum_expr_body}}} else {quote! {(#enum_expr_body)}};
                            quote! {
                                let _fork = input.fork();
                                let parser = |input: ::syn::parse::ParseStream| -> ::syn::Result<#enum_name> {
                                    #capture_init
                                    #pattern_tokens
                                    return ::std::result::Result::Ok(#enum_name::#ident #enum_expr);
                                };
                                if let ::std::result::Result::Ok(v) = parser(&_fork) {
                                    ::syn::parse::discouraged::Speculative::advance_to(input, &_fork);
                                    return ::std::result::Result::Ok(v);
                                };
                            }
                        }
                    }
                });
                let err = variants
                    .iter()
                    .map(|(v, _)| match v {
                        EnumVariant::Type { ty, .. } => {
                            quote! {#ty}
                        }
                        EnumVariant::Capture { .. } => {
                            quote! {pattern(not impl)}
                        }
                    })
                    .collect::<Punctuated<TokenStream, Token![,]>>();
                let err_tokens = quote! {
                    ::std::result::Result::Err(::syn::Error::new(input.span(), stringify!(Expected one of: #err)))
                };

                let parse_impl = parse_quote! {
                    impl ::syn::parse::Parse for #enum_name {
                        fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                            #(#parser)*
                            #err_tokens
                        }
                    }
                };
                self.definition = vec![enum_def, parse_impl];

                quote!(#enum_name)
            }
            MatcherKind::SynType(ty) => quote!(#ty),
            MatcherKind::Nested(_) => quote! {},
        }
    }
}
