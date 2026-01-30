use proc_macro2::{Delimiter, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_quote, punctuated::Punctuated, token::Comma, Expr, Token, Type};

use crate::{
    ast::{
        capture::{Binder, Capture, EnumVariant, FieldDef, Matcher, MatcherKind, Quantity},
        node::{Pattern, PatternKind},
    },
    codegen::{logic::Compiler, output::generate_output},
    transform::lookahead::inject_lookahead,
    utils::crate_name,
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
            (Binder::Named(name), Quantity::Many(separator), MatcherKind::Nested(_patterns)) => {
                let item_name = format_ident!("{}_Item", name);

                let optimized_list = inject_lookahead(_patterns.clone());
                let patterns = Pattern {
                    kind: PatternKind::Group {
                        delimiter: Delimiter::None,
                        children: optimized_list,
                    },
                    span: *span,
                    meta: None,
                };
                let (capture_init, struct_def, struct_expr, _) =
                    generate_output(&patterns.collect_captures(), Some(item_name.clone()));

                let pattern_tokens = self.compile_pattern(&patterns);

                self.define_invisible_item(parse_quote! {
                    #[allow(non_camel_case_types)]
                    pub #struct_def
                });
                self.define_invisible_item(parse_quote! {
                    impl ::syn::parse::Parse for #item_name {
                        fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                            trait _Parse: ::syn::parse::Parse {}
                            #capture_init
                            #pattern_tokens
                            ::std::result::Result::Ok(#struct_expr)
                        }
                    }
                });

                quote! {
                    {
                        #[allow(non_local_definitions)]
                        impl _Parse for #item_name {}
                        #receiver input.parse_terminated(#item_name::parse, #separator)?;
                    }
                }
            }
            (Binder::Named(name), Quantity::One, MatcherKind::Nested(_patterns)) => {
                let item_name = format_ident!("{}_Item", name);

                let optimized_list = inject_lookahead(_patterns.clone());
                let patterns = Pattern {
                    kind: PatternKind::Group {
                        delimiter: Delimiter::None,
                        children: optimized_list,
                    },
                    span: *span,
                    meta: None,
                };
                let (capture_init, struct_def, struct_expr, _) =
                    generate_output(&patterns.collect_captures(), Some(item_name.clone()));

                let pattern_tokens = self.compile_pattern(&patterns);

                self.define_invisible_item(parse_quote! {
                    #[allow(non_camel_case_types)]
                    pub #struct_def
                });
                self.define_invisible_item(parse_quote! {
                    impl ::syn::parse::Parse for #item_name {
                        fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                            trait _Parse: ::syn::parse::Parse {}
                            #capture_init
                            #pattern_tokens
                            ::std::result::Result::Ok(#struct_expr)
                        }
                    }
                });

                quote! {
                    {
                         #receiver input.parse::<#item_name>()?;
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
                    #pattern_tokens
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
                self.define_enum(enum_name, variants);
                self.define_enum_parse_impl(variants, enum_name);
                quote!(#enum_name)
            }
            MatcherKind::SynType(ty) => quote!(#ty),
            MatcherKind::Nested(_) => quote! {},
        }
    }
    fn generate_variant_struct(
        &self,
        variants: &[(EnumVariant, Matcher)],
    ) -> Punctuated<TokenStream, Token![,]> {
        variants
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
            .collect()
    }
    fn define_enum(&mut self, enum_name: &Type, variants: &[(EnumVariant, Matcher)]) {
        let variants_struct = self.generate_variant_struct(variants);
        self.shared_definition.push(parse_quote! {
            pub enum #enum_name {
                #variants_struct
            }
        })
    }
    fn generate_parser(
        &mut self,
        variants: &[(EnumVariant, Matcher)],
        enum_name: &Type,
    ) -> Vec<TokenStream> {
        variants.iter().map(|(v, ..)| match v {
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
                let (capture_init, _, _, capture_list) = generate_output(fields, None);
                let pattern_tokens = self.compile_pattern(pattern);
                let enum_expr_body = capture_list.iter().collect::<Punctuated<_, Token![,]>>();
                let enum_expr = if *named {
                    quote! {{#enum_expr_body}}
                } else {
                    quote! {(#enum_expr_body)}
                };
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
        }).collect()
    }
    fn generate_error_token(&self, variants: &[(EnumVariant, Matcher)]) -> TokenStream {
        let pkg = crate_name();
        let mut fmt_str = vec![];
        let mut fmt_args = Punctuated::<Expr, Token![,]>::new();

        variants.iter().for_each(|(v, _)| match v {
            EnumVariant::Type { ty, .. } => {
                fmt_str.push("{}");
                fmt_args.push(parse_quote!(#pkg::__private::HelpQuery::<#ty>::new().get_message(&PriorityHigh)))
            }
            EnumVariant::Capture { .. } => fmt_str.push("pattern(not impl)"),
        });
        let fmt_str = fmt_str.join("").to_string();
        //
        quote! {
            ::std::result::Result::Err(
                ::syn::Error::new(
                    input.span(),
                    format!(
                        stringify!(Expected one of: #fmt_str),
                        #fmt_args
                    )
                )
            )
        }
    }
    fn define_enum_parse_impl(&mut self, variants: &[(EnumVariant, Matcher)], enum_name: &Type) {
        let parser = self.generate_parser(variants, enum_name);
        let err_tokens = self.generate_error_token(variants);
        let pkg = crate_name();

        let parse_impl = parse_quote! {
            impl ::syn::parse::Parse for #enum_name {
                fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                    use #pkg::__private::{HelpQuery, PriorityHigh, HelpImplDefault, HelpImplCustom};
                    #(#parser)*
                    #err_tokens
                }
            }
        };
        self.shared_definition.push(parse_impl);
    }
}
