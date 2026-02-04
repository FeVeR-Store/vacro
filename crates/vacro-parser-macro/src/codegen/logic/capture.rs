use proc_macro2::{Delimiter, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, punctuated::Punctuated, token::Comma, Expr, Ident, LitInt, Token, Type};

use crate::{
    ast::{
        capture::{Binder, Capture, EnumVariant, FieldDef, Matcher, MatcherKind, Quantity},
        node::{Pattern, PatternKind},
    },
    codegen::{logic::Compiler, output::generate_output},
    transform::lookahead::inject_lookahead,
    utils::resolve_crate_root,
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

        // 1. 生成接收变量的代码 (e.g., `let ident = ...`)
        let receiver = self.compile_binder_receiver(binder);

        // 2. 处理 Anonymous Binder
        if let Binder::Anonymous = binder {
            let t = match (quantity, &matcher.kind) {
                (Quantity::One, MatcherKind::Nested(patterns)) => {
                    let optimized_list = inject_lookahead(patterns.clone());
                    let patterns = Pattern {
                        kind: PatternKind::Group {
                            delimiter: Delimiter::None,
                            children: optimized_list,
                        },
                        span: *span,
                        meta: None,
                    };
                    self.compile_pattern(&patterns)
                }
                (Quantity::Optional, MatcherKind::Nested(patterns)) => {
                    self.compile_anonymous_optional_nested(patterns, span)
                }
                // 如果有 Anonymous + Many 或其他情况，可以在此补充
                _ => self.compile_general_matcher(binder, quantity, matcher, span, &receiver),
            };
            tokens.extend(t);
            return tokens;
        }

        // 3. 通用处理逻辑 (Named, Inline, 以及非 Nested 的 Anonymous)
        // 这些情况都可以归结为：解析一个具体的类型 T (Ty)
        let t = self.compile_general_matcher(binder, quantity, matcher, span, &receiver);
        tokens.extend(t);
        tokens
    }

    /// 处理通用的解析逻辑：先确定要解析的类型，再根据数量(Quantity)生成调用代码
    fn compile_general_matcher(
        &mut self,
        binder: &Binder,
        quantity: &Quantity,
        matcher: &Matcher,
        span: &Span,
        receiver: &TokenStream,
    ) -> TokenStream {
        // A. 获取要解析的目标类型 (Type)
        let (ty, is_scoped) = match &matcher.kind {
            MatcherKind::Enum { .. } | MatcherKind::SynType(_) => {
                (self.compile_matcher(matcher), false)
            }
            MatcherKind::Nested(patterns) => {
                // 根据 Binder 类型生成结构体名称
                let struct_ident = match binder {
                    Binder::Named(name) => format_ident!("{}_Item", name),
                    Binder::Inline(inline) => format_ident!("_{}", inline),
                    _ => format_ident!("_Anon_Item"), // Fallback，通常不会走到这里
                };
                // 生成嵌套结构体并返回其类型
                (
                    self.define_nested_parser(&struct_ident, patterns, *span),
                    true,
                )
            }
        };

        let ty = if is_scoped {
            quote! {<#ty>}
        } else {
            quote! {<#ty as ::syn::parse::Parse>}
        };
        // B. 根据数量 (Quantity) 生成解析动作
        match quantity {
            Quantity::One => {
                quote! {
                    #receiver #ty::parse(&input)?;
                }
            }
            Quantity::Optional => {
                quote! {
                    {
                        let _fork = input.fork();
                        if let ::std::result::Result::Ok(_parsed) = #ty::parse(&_fork) {
                            #receiver ::std::option::Option::Some(_parsed);
                            ::syn::parse::discouraged::Speculative::advance_to(input, &_fork);
                        }
                    }
                }
            }
            Quantity::Many(separator) => {
                quote! {
                    {
                        #receiver input.parse_terminated(#ty::parse, #separator)?;
                    }
                }
            }
        }
    }

    /// 定义嵌套的 Struct 及其 Parse 实现，并返回该 Struct 的类型
    fn define_nested_parser(
        &mut self,
        item_name: &Ident,
        patterns: &[Pattern],
        span: Span,
    ) -> TokenStream {
        let optimized_list = inject_lookahead(patterns.to_vec());
        let patterns_group = Pattern {
            kind: PatternKind::Group {
                delimiter: Delimiter::None,
                children: optimized_list,
            },
            span,
            meta: None,
        };

        let (capture_init, struct_def, struct_expr, _) = generate_output(
            &patterns_group.collect_captures(),
            Some(item_name.clone()),
            None,
        );

        let pattern_tokens = self.compile_pattern(&patterns_group);

        // 1. 定义 Struct
        self.define_invisible_item(parse_quote! {
            #[allow(non_camel_case_types)]
            pub #struct_def
        });

        // 2. 定义 Trait (为了避免命名冲突的习惯用法)
        let parse_trait = format_ident!("_{}_Parse", item_name);
        self.define_invisible_item(parse_quote! {
            #[allow(non_camel_case_types)]
            pub trait #parse_trait {
                fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<#item_name>;
            }
        });

        // 3. 实现 Trait / Parse
        self.define_invisible_item(parse_quote! {
            impl #parse_trait for #item_name {
                fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                    #capture_init
                    #pattern_tokens
                    ::std::result::Result::Ok(#struct_expr)
                }
            }
        });

        // 4. 返回类型路径
        if let Some(scope) = crate::scope_context::get_scope_ident() {
            quote!(#scope::#item_name)
        } else {
            quote!(#item_name)
        }
    }

    /// 辅助函数：生成 Binder 对应的接收器代码
    fn compile_binder_receiver(&self, binder: &Binder) -> TokenStream {
        match binder {
            Binder::Named(ident) => quote! { #ident = },
            Binder::Inline(i) => {
                let id = format_ident!("_{}", i.to_string());
                quote! { #id = }
            }
            _ => quote! {},
        }
    }

    /// 提取出复杂的 Anonymous + Optional 逻辑
    fn compile_anonymous_optional_nested(
        &mut self,
        patterns: &[Pattern],
        span: &Span,
    ) -> TokenStream {
        let optimized_list = inject_lookahead(patterns.to_vec());
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
            generate_output(&captures, None, None);

        let assigns_err = fields.iter().map(|ident| {
            quote! { #ident = ::std::option::Option::None; }
        });

        let assigns_ok = captures.iter().enumerate().map(|(i, cap)| {
            let ident = &fields[i];
            let access = if cap.is_inline {
                LitInt::new(&i.to_string(), Span::call_site()).into_token_stream()
            } else {
                quote! {#ident}
            };
            quote! { #ident = ::std::option::Option::Some(output.#access); }
        });

        quote! {
            #struct_def
            let _parser = |input: ::syn::parse::ParseStream| -> ::syn::Result<Output> {
                #capture_init
                #joint_token
                ::std::result::Result::Ok(#struct_expr)
            };
            let _fork = input.fork();
            match _parser(&_fork) {
                ::std::result::Result::Ok(output) => {
                    ::syn::parse::discouraged::Speculative::advance_to(input, &_fork);
                    #(#assigns_ok)*
                }
                ::std::result::Result::Err(_) => {
                    #(#assigns_err)*
                }
            }
        }
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
                    if let ::std::result::Result::Ok(v) = <#ty as ::syn::parse::Parse>::parse(&_fork) {
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
                let (capture_init, _, _, capture_list) = generate_output(fields, None, None);
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
        let pkg = resolve_crate_root();
        let mut fmt_str = vec![];
        let mut fmt_args = Punctuated::<Expr, Token![,]>::new();

        variants.iter().for_each(|(v, _)| match v {
            EnumVariant::Type { ty, .. } => {
                fmt_str.push("{}");
                fmt_args.push(parse_quote!(#pkg::__private::HelpQuery::<#ty>::new().get_message(&PriorityHigh)))
            }
            EnumVariant::Capture { .. } => fmt_str.push("pattern(not impl)"),
        });
        let fmt_str = fmt_str.join(", ").to_string();
        //
        quote! {
            ::std::result::Result::Err(
                ::syn::Error::new(
                    input.span(),
                    format!(
                        stringify!(Expected one of: {}, get: {}),
                        format!(#fmt_str, #fmt_args),
                        input
                    )
                )
            )
        }
    }
    fn define_enum_parse_impl(&mut self, variants: &[(EnumVariant, Matcher)], enum_name: &Type) {
        let parser = self.generate_parser(variants, enum_name);
        let err_tokens = self.generate_error_token(variants);
        let pkg = resolve_crate_root();

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
