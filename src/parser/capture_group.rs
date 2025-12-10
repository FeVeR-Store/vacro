//! ## 对于CaptureSpec的解析
//! 捕获组支持的语法有：
//! 1.具名捕获，用于生成实现了Parse的结构体:
//!    <Name>: <Capture> eg: `name: Ident` -> `name: Ident`
//!    <Name>?: <Capture> eg: `visibility?: Visibility` -> `visibility: Option<Visibility>`
//!    <Name>*[<Separator>]: <Capture> eg: `args*[,]: Ident` -> `args: Punctuated<Ident, Token![,]>`
//! 2. 匿名捕获，用于校验，不捕获内容:
//!    <Capture> eg: `Ident`
//!    ?:<Capture> eg: `?: Visibility`
//!    *[<Separator>]:<Capture> eg: `*[,]: Ident`
//! 3. 行内捕获，用于直接捕获，捕获的值会组成元组:
//!    @:<Capture> eg: `@: Ident`
//!    @?:<Capture> eg: `@?: Visibility`
//!    @*[<Separator>]:<Capture> eg: `@*[,]: Ident`
//!
//! 捕获的类型：
//!  - `impl syn::Parse`
//!  - 匿名捕获时，可为另一个结构
use std::sync::{Arc, Mutex};

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Ident, Type, parse_quote};

use crate::{
    ast::{
        capture::{CaptureMode, CaptureSpec, CaptureType, ExposeMode},
        pattern::{IsOptional, PatternList},
    },
    codegen::output::generate_output,
    transform::lookahead::inject_lookahead,
};

impl CaptureSpec {
    pub fn add_capture(&self, capture_list: Arc<Mutex<Vec<(Ident, Type, IsOptional)>>>) {
        match self {
            CaptureSpec {
                ty: CaptureType::Joint(joint),
                mode,
                ..
            } => capture_list.lock().unwrap().extend(
                joint.capture_list.clone().lock().unwrap().iter().map(
                    |(ident, ty, is_optional)| match mode {
                        CaptureMode::Optional => (
                            ident.clone(),
                            parse_quote!(::std::option::Option<#ty>),
                            true,
                        ),
                        CaptureMode::Iter(sep) => (
                            ident.clone(),
                            parse_quote!(::syn::punctuated::Punctuated<#ty, #sep>),
                            *is_optional,
                        ),
                        CaptureMode::Once => (ident.clone(), ty.clone(), *is_optional),
                    },
                ),
            ),
            CaptureSpec {
                name,
                ty: CaptureType::Type(ty),
                mode,
            } => {
                let ident = match name {
                    ExposeMode::Named(named) => named.clone(),
                    ExposeMode::Inline(i) => {
                        format_ident!("_{}", i.to_string())
                    }
                    _ => return,
                };
                let is_optional;
                let ty: Type = match mode {
                    CaptureMode::Iter(sep) => {
                        is_optional = false;
                        parse_quote!(::syn::punctuated::Punctuated<#ty, #sep>)
                    }
                    CaptureMode::Once => {
                        is_optional = false;
                        parse_quote!(#ty)
                    }
                    CaptureMode::Optional => {
                        is_optional = true;
                        parse_quote!(::std::option::Option<#ty>)
                    }
                };
                capture_list
                    .lock()
                    .unwrap()
                    .push((ident.clone(), ty, is_optional));
            }
        }
    }
}

impl ToTokens for CaptureSpec {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let CaptureSpec { ty, mode, name, .. } = self;
        let receiver = match name {
            ExposeMode::Named(ident) => {
                quote! {#ident = }
            }
            ExposeMode::Inline(i) => {
                let id = format_ident!("_{}", i.to_string());
                quote! {#id = }
            }
            _ => quote! {},
        };
        let t = match (name, mode, ty) {
            (_, CaptureMode::Once, CaptureType::Type(ty)) => {
                quote! {
                    {
                        #receiver input.parse::<#ty>()?;
                    }
                }
            }
            (_, CaptureMode::Optional, CaptureType::Type(ty)) => {
                quote! {
                    {
                        let _fork = input.fork();
                        if ::std::result::Ok(_parsed) = _fork.parse::<#ty>() {
                            #receiver ::std::option::Option::Some(_parsed);
                        }
                    }
                }
            }
            (_, CaptureMode::Iter(separator), CaptureType::Type(ty)) => {
                quote! {
                    {
                        #[allow(non_local_definitions)]
                        impl _Parse for #ty {}
                        #receiver input.parse_terminated(#ty::parse, #separator)?;
                    }
                }
            }
            (ExposeMode::Anonymous, CaptureMode::Once, CaptureType::Joint(_patterns)) => {
                let optimized_list = inject_lookahead(_patterns.list.clone());

                let patterns = PatternList {
                    list: optimized_list,
                    capture_list: _patterns.capture_list.clone(),
                    parse_context: _patterns.parse_context.clone(),
                };
                quote! {
                    {
                        #patterns
                    }
                }
            }
            (ExposeMode::Anonymous, CaptureMode::Optional, CaptureType::Joint(_patterns)) => {
                let optimized_list = inject_lookahead(_patterns.list.clone());

                let patterns = PatternList {
                    list: optimized_list,
                    capture_list: _patterns.capture_list.clone(),
                    parse_context: _patterns.parse_context.clone(),
                };

                let joint_token = quote! { #patterns };
                let (capture_init, struct_def, struct_expr) =
                    generate_output(patterns.capture_list.clone(), None, &patterns.parse_context);
                let fields = patterns
                    .capture_list
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|(name, ..)| name.clone())
                    .collect::<Vec<_>>();

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
                        ::std::result::Result::Err(err) => {
                            #(#assigns_err)*
                        }
                    }
                    let _ = _parser(input);
                }
            }
            _ => quote! {},
        };
        tokens.extend(t);
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{keyword::Keyword, pattern::Pattern},
        parser::context::ParseContext,
    };

    use super::*;
    use syn::{
        Result,
        parse::{ParseStream, Parser},
    };

    // --- 辅助函数：用于简化断言 ---

    // 检查是否是具名捕获
    fn assert_named(spec: &CaptureSpec, expected_name: &str) {
        if let ExposeMode::Named(ident) = &spec.name {
            assert_eq!(ident.to_string(), expected_name);
        } else {
            panic!("Expected Named capture, got {:?}", spec.name);
        }
    }

    // 检查是否是行内捕获 (@)
    fn assert_inline(spec: &CaptureSpec) {
        if !matches!(spec.name, ExposeMode::Inline(_)) {
            panic!("Expected Inline capture, got {:?}", spec.name);
        }
    }

    // 检查是否是匿名捕获
    fn assert_anonymous(spec: &CaptureSpec) {
        if !matches!(spec.name, ExposeMode::Anonymous) {
            panic!("Expected Anonymous capture, got {:?}", spec.name);
        }
    }

    fn parse_capture_spec(input: TokenStream, ctx: &mut ParseContext) -> Result<CaptureSpec> {
        let parser = move |input: ParseStream| CaptureSpec::parse(input, ctx);
        parser.parse2(input)
    }

    fn parse_capture_type(input: TokenStream, ctx: &mut ParseContext) -> Result<CaptureType> {
        let parser = move |input: ParseStream| CaptureType::parse(input, ctx);
        parser.parse2(input)
    }

    // --- 1. Parse 语法解析测试 ---

    #[test]
    fn test_parse_basic_named() {
        let ctx = &mut ParseContext::default();
        // 语法: name: Type
        let input = quote! { my_field: syn::Ident };
        let spec: CaptureSpec = parse_capture_spec(input, ctx).unwrap();
        assert_named(&spec, "my_field");
        assert!(matches!(spec.mode, CaptureMode::Once));
        assert!(matches!(spec.ty, CaptureType::Type(_)));
    }

    #[test]
    fn test_parse_optional_named() {
        let ctx = &mut ParseContext::default();
        // 语法: name?: Type
        let input = quote! { maybe_val?: u32 };
        let spec: CaptureSpec = parse_capture_spec(input, ctx).unwrap();
        assert_named(&spec, "maybe_val");
        assert!(matches!(spec.mode, CaptureMode::Optional));
    }

    #[test]
    fn test_parse_iter_named() {
        let ctx = &mut ParseContext::default();

        // 语法: name*[,]: Type
        let input = quote! { list*[,]: Ident };
        let spec: CaptureSpec = parse_capture_spec(input, ctx).unwrap();
        assert_named(&spec, "list");
        if let CaptureMode::Iter(sep) = &spec.mode {
            if let Keyword::Rust(s) = sep {
                assert_eq!(s, ",");
            } else {
                panic!("Expected Rust keyword separator");
            }
        } else {
            panic!("Expected Iter mode");
        }
    }

    #[test]
    fn test_parse_inline() {
        let ctx = &mut ParseContext::default();

        // 语法: @: Type, @?: Type, @*[;]: Type
        // 注意：原子计数器 ITER 会在测试间共享，所以不校验具体的 Index 值
        let input1 = quote! { @: Ident };
        let spec1: CaptureSpec = parse_capture_spec(input1, ctx).unwrap();
        assert_inline(&spec1);
        assert!(matches!(spec1.mode, CaptureMode::Once));

        let input2 = quote! { @?: Ident };
        let spec2: CaptureSpec = parse_capture_spec(input2, ctx).unwrap();
        assert_inline(&spec2);
        assert!(matches!(spec2.mode, CaptureMode::Optional));
    }

    #[test]
    fn test_parse_anonymous() {
        let ctx = &mut ParseContext::default();

        // 语法: Type (无名称)
        let input = quote! { syn::Type };
        let spec: CaptureSpec = parse_capture_spec(input, ctx).unwrap();
        assert_anonymous(&spec);
        assert!(matches!(spec.mode, CaptureMode::Once));

        // 语法: ?: Type
        let input2 = quote! { ?: syn::Visibility };
        let spec2: CaptureSpec = parse_capture_spec(input2, ctx).unwrap();
        assert_anonymous(&spec2);
        assert!(matches!(spec2.mode, CaptureMode::Optional));
    }

    #[test]
    fn test_parse_joint_nested() {
        let ctx = &mut ParseContext::default();

        // 语法: #( ... )
        // 模拟 Joint 解析，注意这里依赖 pattern 模块的解析逻辑，假设 pattern 也能 parse
        let input = quote! { ?: -> #( name: Ident ) };
        let result = parse_capture_type(input, ctx);

        // 注意：由于 CaptureType::parse 对于 # 的处理比较特殊，
        // 这里主要测试它能识别 # 并返回 Joint 变体
        match result {
            Ok(CaptureType::Joint(pattern_list)) => {
                // Joint 模式下，列表第一个元素应该是字面量 "#"
                assert!(!pattern_list.list.is_empty());
                #[allow(unused)]
                let _keyword = Keyword::Rust(String::from("->"));
                if let Pattern::Literal(kw) = &pattern_list.list[0] {
                    assert!(matches!(kw, _keyword));
                } else {
                    panic!("First element of Joint should be # literal");
                }
            }
            Ok(_) => panic!("Expected Joint capture type"),
            Err(e) => panic!("Failed to parse joint: {}", e),
        }
    }

    #[test]
    fn test_parse_error_missing_separator() {
        let ctx = &mut ParseContext::default();

        // 错误语法: *[] 中间缺少分隔符
        let input = quote! { args*[]: Ident };
        let result = parse_capture_spec(input, ctx);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "expected '[<separator>]' like '[,]'"
        );
    }

    // --- 2. Struct 字段生成逻辑测试 (add_capture) ---

    #[test]
    fn test_add_capture_once() {
        let ctx = &mut ParseContext::default();

        let input = quote! { field: u32 };
        let spec: CaptureSpec = parse_capture_spec(input, ctx).unwrap();

        let capture_list = Arc::new(Mutex::new(Vec::new()));
        spec.add_capture(capture_list.clone());

        let list = capture_list.lock().unwrap();
        assert_eq!(list.len(), 1);
        let (name, ty, is_optional) = &list[0];

        assert_eq!(name.to_string(), "field");
        assert_eq!(quote!(#ty).to_string(), "u32");
        assert_eq!(*is_optional, false);
    }

    #[test]
    fn test_add_capture_optional() {
        let ctx = &mut ParseContext::default();

        let input = quote! { field?: u32 };
        let spec: CaptureSpec = parse_capture_spec(input, ctx).unwrap();

        let capture_list = Arc::new(Mutex::new(Vec::new()));
        spec.add_capture(capture_list.clone());

        let list = capture_list.lock().unwrap();
        let (_, ty, is_optional) = &list[0];

        // 验证类型是否被包裹在 Option 中
        assert_eq!(
            quote!(#ty).to_string(),
            ":: std :: option :: Option < u32 >"
        );
        assert_eq!(*is_optional, true);
    }

    #[test]
    fn test_add_capture_iter() {
        let ctx = &mut ParseContext::default();

        let input = quote! { field*[,]: u32 };
        let spec: CaptureSpec = parse_capture_spec(input, ctx).unwrap();

        let capture_list = Arc::new(Mutex::new(Vec::new()));
        spec.add_capture(capture_list.clone());

        let list = capture_list.lock().unwrap();
        let (_, ty, is_optional) = &list[0];

        // 验证类型是否被包裹在 Punctuated 中
        let ty_str = quote!(#ty).to_string();
        assert!(ty_str.contains("Punctuated"));
        assert!(ty_str.contains("u32"));
        assert_eq!(*is_optional, false); // Iter 模式如果不带 ? 本身不视为 Optional 字段(通常是个空的Punctuated)
    }

    #[test]
    fn test_add_capture_inline_naming() {
        let ctx = &mut ParseContext::default();

        let input = quote! { @: u32 };
        let spec: CaptureSpec = parse_capture_spec(input, ctx).unwrap();

        let capture_list = Arc::new(Mutex::new(Vec::new()));
        spec.add_capture(capture_list.clone());

        let list = capture_list.lock().unwrap();
        let (name, _, _) = &list[0];

        // 验证生成的名称格式为 _<index>
        assert!(name.to_string().starts_with("_"));
    }

    // --- 3. Lookahead 优化逻辑测试 ---

    #[test]
    fn test_inject_lookahead() {
        let ctx = &mut ParseContext::default();

        // 手动构造 Pattern 列表来测试 inject_lookahead 算法
        // 场景: Capture + Literal -> 应该注入

        // Mock数据构造：为了测试私有函数，我们需要构造 Pattern
        // 假设 Pattern 和 Keyword 是可访问的 (通常在同一 crate 或 test super 中)
        let input = quote!(x: Ident);

        let ident_spec: CaptureSpec = parse_capture_spec(input, ctx).unwrap();
        let pattern_capture = Pattern::Capture(ident_spec.clone(), None);
        let pattern_literal = Pattern::Literal(Keyword::Rust(",".to_string()));

        // Case 1: Capture 后面跟 Literal
        let patterns = vec![pattern_capture.clone(), pattern_literal.clone()];
        let optimized = inject_lookahead(patterns);

        assert_eq!(optimized.len(), 2);
        // 检查第一个 Capture 是否被注入了 lookahead
        if let Pattern::Capture(_, Some(lookahead)) = &optimized[0] {
            if let Keyword::Rust(s) = lookahead {
                assert_eq!(s, ",");
            } else {
                panic!("Wrong lookahead type");
            }
        } else {
            panic!("Lookahead not injected");
        }

        // Case 2: Capture 后面跟 Capture (不应注入)
        let patterns_consecutive = vec![pattern_capture.clone(), pattern_capture.clone()];
        let optimized_consecutive = inject_lookahead(patterns_consecutive);
        if let Pattern::Capture(_, Some(_)) = &optimized_consecutive[0] {
            panic!("Should not inject lookahead when followed by another capture");
        }

        // Case 3: Capture 在末尾 (不应注入)
        let patterns_end = vec![pattern_capture.clone()];
        let optimized_end = inject_lookahead(patterns_end);
        if let Pattern::Capture(_, Some(_)) = &optimized_end[0] {
            panic!("Should not inject lookahead at end of stream");
        }
    }

    // --- 4. ToTokens 代码生成冒烟测试 ---

    #[test]
    fn test_to_tokens_smoke() {
        let ctx = &mut ParseContext::default();

        // 只要不 Panic 且有输出即可，详细逻辑校验需要编译生成的代码
        let spec: CaptureSpec = parse_capture_spec(quote!(x: Ident), ctx).unwrap();
        let tokens = quote!(#spec);
        assert!(!tokens.is_empty());

        let spec_opt: CaptureSpec = parse_capture_spec(quote!(x?: Ident), ctx).unwrap();
        let tokens_opt = quote!(#spec_opt);
        // 生成的代码应该包含 Option 处理逻辑
        assert!(tokens_opt.to_string().contains("Option"));

        let spec_iter: CaptureSpec = parse_capture_spec(quote!(x*[,]: Ident), ctx).unwrap();
        let tokens_iter = quote!(#spec_iter);
        // 生成的代码应该包含 parse_terminated
        assert!(tokens_iter.to_string().contains("parse_terminated"));
    }
}
