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
                (Quantity::Many(separator), MatcherKind::Nested(patterns)) => {
                    self.compile_anonymous_many_nested(patterns, separator.as_ref(), span)
                }
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
        // A. 获取要解析的目标类型 (Type) 和对应的 parse trait
        let (ty, parse_trait) = match &matcher.kind {
            MatcherKind::Enum { .. } | MatcherKind::SynType(_) => {
                let ty = self.compile_matcher(matcher);
                (ty.clone(), quote! {<#ty as ::syn::parse::Parse>})
            }
            MatcherKind::Nested(patterns) => {
                // 根据 Binder 类型生成结构体名称
                let struct_ident = match binder {
                    Binder::Named(name) => format_ident!("{}_Item", name),
                    Binder::Inline(inline) => format_ident!("_{}", inline),
                    _ => format_ident!("_Anon_Item"), // Fallback，通常不会走到这里
                };
                let parse_trait_ident = format_ident!("_{}_Parse", struct_ident);
                // 生成嵌套结构体并返回其类型
                let ty = self.define_nested_parser(&struct_ident, patterns, *span);
                // 使用完全限定语法调用自定义 trait，无需将 trait 导入当前作用域
                let scope = crate::scope_context::get_scope_ident();
                let qualified = if let Some(scope) = scope {
                    quote! {<#ty as #scope::#parse_trait_ident>}
                } else {
                    quote! {<#ty as #parse_trait_ident>}
                };
                (ty, qualified)
            }
        };

        // B. 根据数量 (Quantity) 生成解析动作
        match quantity {
            Quantity::One => {
                quote! {
                    #receiver #parse_trait::parse(&input)?;
                }
            }
            Quantity::Optional => {
                if matches!(matcher.kind, MatcherKind::Enum { .. }) {
                    quote! {
                        {
                            if let ::std::result::Result::Ok(_parsed) = #parse_trait::parse(input) {
                                #receiver ::std::option::Option::Some(_parsed);
                            }
                        }
                    }
                } else {
                    quote! {
                        {
                            let _fork = input.fork();
                            if #parse_trait::parse(&_fork).is_ok() {
                                let _parsed = #parse_trait::parse(input)?;
                                #receiver ::std::option::Option::Some(_parsed);
                            }
                        }
                    }
                }
            }
            Quantity::Many(separator) => match separator {
                Some(separator) => {
                    if matches!(matcher.kind, MatcherKind::Enum { .. }) {
                        quote! {
                            {
                                let mut _items = ::syn::punctuated::Punctuated::<#ty, #separator>::new();
                                while !input.is_empty() {
                                    let mut _input = ::proc_macro2::TokenStream::new();
                                    let mut _angle_depth = 0usize;
                                    while !input.is_empty() && !(_angle_depth == 0 && input.peek(#separator)) {
                                        let _tree = input.parse::<::proc_macro2::TokenTree>()?;
                                        match &_tree {
                                            ::proc_macro2::TokenTree::Punct(_punct)
                                                if _punct.as_char() == '<' =>
                                            {
                                                _angle_depth += 1;
                                            }
                                            ::proc_macro2::TokenTree::Punct(_punct)
                                                if _punct.as_char() == '>' =>
                                            {
                                                _angle_depth = _angle_depth.saturating_sub(1);
                                            }
                                            _ => {}
                                        }
                                        _input.extend(::std::iter::once(_tree));
                                    }
                                    if _input.is_empty() {
                                        return ::std::result::Result::Err(::syn::Error::new(
                                            input.span(),
                                            "expected iterative capture item before separator",
                                        ));
                                    }
                                    let _parsed = ::syn::parse::Parser::parse2(#parse_trait::parse, _input)?;
                                    _items.push_value(_parsed);
                                    if input.is_empty() {
                                        break;
                                    }
                                    let _punct: #separator = input.parse()?;
                                    _items.push_punct(_punct);
                                }
                                #receiver _items;
                            }
                        }
                    } else {
                        quote! {
                            {
                                #receiver input.parse_terminated(#parse_trait::parse, #separator)?;
                            }
                        }
                    }
                }
                None => {
                    quote! {
                        {
                            let mut _items = ::std::vec::Vec::<#ty>::new();
                            while !input.is_empty() {
                                let _before = input.cursor();
                                let _parsed = #parse_trait::parse(&input)?;
                                if input.cursor() == _before {
                                    return ::std::result::Result::Err(::syn::Error::new(
                                        input.span(),
                                        "iterative capture did not consume any tokens",
                                    ));
                                }
                                _items.push(_parsed);
                            }
                            #receiver _items;
                        }
                    }
                }
            },
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

        let captures = patterns_group.collect_captures();

        let (capture_init, struct_def, struct_expr, _) =
            generate_output(&captures, Some(item_name.clone()), None);

        let pattern_tokens = self.compile_pattern(&patterns_group);

        let derive_attrs = &self.derive_attrs;

        // 1. 定义 Struct
        self.define_invisible_item(parse_quote! {
            #(#derive_attrs)*
            #[allow(non_camel_case_types)]
            pub #struct_def
        });

        // 2. 定义 Trait（避免与 syn::parse::Parse 冲突）
        let parse_trait = format_ident!("_{}_Parse", item_name);
        self.define_invisible_item(parse_quote! {
            #[allow(non_camel_case_types)]
            pub trait #parse_trait {
                fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<#item_name>;
            }
        });

        // 3. 实现 Trait
        self.define_invisible_item(parse_quote! {
            impl #parse_trait for #item_name {
                fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                    #capture_init
                    #pattern_tokens
                    ::std::result::Result::Ok(#struct_expr)
                }
            }
        });

        // 3. 返回类型路径
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
            if _parser(&_fork).is_ok() {
                match _parser(input) {
                    ::std::result::Result::Ok(output) => {
                        #(#assigns_ok)*
                    }
                    ::std::result::Result::Err(err) => {
                        return ::std::result::Result::Err(err);
                    }
                }
            } else {
                #(#assigns_err)*
            }
        }
    }
    fn compile_anonymous_many_nested(
        &mut self,
        patterns: &[Pattern],
        separator: Option<&crate::ast::keyword::Keyword>,
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

        let collection_names = fields
            .iter()
            .map(|ident| format_ident!("_{ident}_items"))
            .collect::<Vec<_>>();

        let collection_init = collection_names.iter().map(|ident| {
            if separator.is_some() {
                quote! {
                    let mut #ident = ::syn::punctuated::Punctuated::new();
                }
            } else {
                quote! {
                    let mut #ident = ::std::vec::Vec::new();
                }
            }
        });

        let push_values = captures.iter().enumerate().map(|(i, cap)| {
            let collection = &collection_names[i];
            let access = if cap.is_inline {
                LitInt::new(&i.to_string(), Span::call_site()).into_token_stream()
            } else {
                let ident = &fields[i];
                quote! {#ident}
            };
            if separator.is_some() {
                quote! {
                    #collection.push_value(output.#access);
                }
            } else {
                quote! {
                    #collection.push(output.#access);
                }
            }
        });

        let assign_collections =
            fields
                .iter()
                .zip(collection_names.iter())
                .map(|(field, collection)| {
                    quote! {
                        #field = #collection;
                    }
                });

        let parse_item = quote! {
            let _before = input.cursor();
            let output = _parser(input)?;
            if input.cursor() == _before {
                return ::std::result::Result::Err(::syn::Error::new(
                    input.span(),
                    "iterative capture did not consume any tokens",
                ));
            }
            #(#push_values)*
        };

        let parse_loop = if let Some(separator) = separator {
            let push_puncts = collection_names.iter().map(|collection| {
                quote! {
                    #collection.push_punct(::std::clone::Clone::clone(&_punct));
                }
            });
            quote! {
                loop {
                    if input.is_empty() {
                        break;
                    }
                    #parse_item
                    if input.is_empty() {
                        break;
                    }
                    let _punct: #separator = input.parse()?;
                    #(#push_puncts)*
                }
            }
        } else {
            quote! {
                while !input.is_empty() {
                    #parse_item
                }
            }
        };

        quote! {
            #struct_def
            let _parser = |input: ::syn::parse::ParseStream| -> ::syn::Result<Output> {
                #capture_init
                #joint_token
                ::std::result::Result::Ok(#struct_expr)
            };
            #(#collection_init)*
            #parse_loop
            #(#assign_collections)*
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
                EnumVariant::Capture { ident, pattern, .. } => {
                    // 在 codegen 阶段重新收集 captures，确保 scope 已设置
                    let fields = pattern.collect_captures();
                    let named = fields.first().map(|f| !f.is_inline).unwrap_or(false);
                    let body = if named {
                        let fields: Punctuated<_, Comma> = fields
                            .iter()
                            .map(|FieldDef { name, ty, .. }| quote! {#name: #ty})
                            .collect();
                        quote! {
                            {
                                #fields
                            }
                        }
                    } else if fields.is_empty() {
                        quote! {}
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
        let derive_attrs = &self.derive_attrs;
        self.shared_definition.push(parse_quote! {
            #(#derive_attrs)*
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
                    if <#ty as ::syn::parse::Parse>::parse(&_fork).is_ok() {
                        let v = <#ty as ::syn::parse::Parse>::parse(input)?;
                        return ::std::result::Result::Ok(#enum_name::#ident(v));
                    };
                }
            }
            EnumVariant::Capture {
                ident,
                pattern,
                ..
            } => {
                // 在 codegen 阶段重新收集 captures，确保 scope 已设置
                let fields = pattern.collect_captures();
                let named = fields.first().map(|f| !f.is_inline).unwrap_or(false);
                let (capture_init, _, _, capture_list) = generate_output(&fields, None, None);
                let pattern_tokens = self.compile_pattern(pattern);
                let enum_expr_body = capture_list.iter().collect::<Punctuated<_, Token![,]>>();
                let enum_expr = if named {
                    quote! {{#enum_expr_body}}
                } else if fields.is_empty() {
                    quote! {}
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
                    if parser(&_fork).is_ok() {
                        return parser(input);
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
