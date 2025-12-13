use proc_macro2::Span;
use syn::{Token, token};

use crate::ast::{keyword::Keyword, node::Pattern};

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub struct Capture {
    pub _hash_tag: Token![#],
    pub _paren: token::Paren,

    pub binder: Binder,     // 1. 绑定给谁？
    pub matcher: Matcher,   // 2. 解析什么？
    pub quantity: Quantity, // 3. 解析多少次？

    // 用于标记边缘
    pub edge: Option<Keyword>,

    pub span: Span,
}

/// 绑定模式
#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub enum Binder {
    Named(syn::Ident), // name: ...
    Inline(usize),     // @: ...
    Anonymous,         // _: ...
}

/// 匹配器
#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub struct Matcher {
    pub kind: MatcherKind,
    pub span: Span,
}

/// 匹配器
#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub enum MatcherKind {
    /// 标准 Syn 类型 (e.g. `Ident`, `Type`)
    SynType(syn::Type),

    /// 嵌套结构 (e.g. `#( ... )`)
    Nested(Vec<Pattern>),
}

/// 数量限定
#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub enum Quantity {
    One,                   // 默认
    Optional,              // ?
    Many(Option<Keyword>), // * 或 *[,]
}

impl Capture {
    pub fn collect_captures(&self) -> Vec<&Capture> {
        let mut collector = vec![];
        self.visit_captures(&mut collector);
        collector
    }
    pub fn visit_captures<'a>(&'a self, collector: &mut Vec<&'a Capture>) {
        // 核心分支：看 Matcher 是什么类型
        match &self.matcher.kind {
            MatcherKind::Nested(children) => {
                for child in children {
                    child.visit_captures(collector);
                }
            }
            MatcherKind::SynType(_) => {
                collector.push(self);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{keyword::Keyword, node::PatternKind},
        codegen::logic::Compiler,
        syntax::context::ParseContext,
        transform::lookahead::inject_lookahead,
    };

    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{
        Result,
        parse::{ParseStream, Parser},
    };

    // --- 辅助函数：用于简化断言 ---

    // 检查是否是具名捕获
    fn assert_named(capture: &Capture, expected_name: &str) {
        if let Binder::Named(ident) = &capture.binder {
            assert_eq!(ident.to_string(), expected_name);
        } else {
            panic!("Expected Named capture, got {:?}", capture.binder);
        }
    }

    // 检查是否是行内捕获 (@)
    fn assert_inline(capture: &Capture) {
        if !matches!(capture.binder, Binder::Inline(_)) {
            panic!("Expected Inline capture, got {:?}", capture.binder);
        }
    }

    // 检查是否是匿名捕获
    fn assert_anonymous(capture: &Capture) {
        if !matches!(capture.binder, Binder::Anonymous) {
            panic!("Expected Anonymous capture, got {:?}", capture.binder);
        }
    }

    fn parse_capture(input: TokenStream, ctx: &mut ParseContext) -> Result<Capture> {
        let parser = move |input: ParseStream| Capture::parse(input, ctx);
        parser.parse2(input)
    }

    fn parse_capture_matcher(input: TokenStream, ctx: &mut ParseContext) -> Result<Matcher> {
        let parser = move |input: ParseStream| Matcher::parse(input, ctx);
        parser.parse2(input)
    }

    // --- 1. Parse 语法解析测试 ---

    #[test]
    fn test_parse_basic_named() {
        let ctx = &mut ParseContext::default();
        // 语法: name: Type
        let input = quote! { #(my_field: syn::Ident) };
        let capture: Capture = parse_capture(input, ctx).unwrap();
        assert_named(&capture, "my_field");
        assert!(matches!(capture.quantity, Quantity::One));
        assert!(matches!(capture.matcher.kind, MatcherKind::SynType(_)));
    }

    #[test]
    fn test_parse_optional_named() {
        let ctx = &mut ParseContext::default();
        // 语法: name?: Type
        let input = quote! { #(maybe_val?: u32) };
        let capture: Capture = parse_capture(input, ctx).unwrap();
        assert_named(&capture, "maybe_val");
        assert!(matches!(capture.quantity, Quantity::Optional));
    }

    #[test]
    fn test_parse_iter_named() {
        let ctx = &mut ParseContext::default();

        // 语法: name*[,]: Type
        let input = quote! { #(list*[,]: Ident) };
        let capture: Capture = parse_capture(input, ctx).unwrap();
        assert_named(&capture, "list");
        if let Quantity::Many(sep) = &capture.quantity {
            if let Some(Keyword::Rust(s)) = sep {
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
        let input1 = quote! { #(@: Ident) };
        let spec1: Capture = parse_capture(input1, ctx).unwrap();
        assert_inline(&spec1);
        assert!(matches!(spec1.quantity, Quantity::One));

        let input2 = quote! { #(@?: Ident) };
        let spec2: Capture = parse_capture(input2, ctx).unwrap();
        assert_inline(&spec2);
        assert!(matches!(spec2.quantity, Quantity::Optional));
    }

    #[test]
    fn test_parse_anonymous() {
        let ctx = &mut ParseContext::default();

        // 语法: Type (无名称)
        let input = quote! { #(syn::Type) };
        let capture: Capture = parse_capture(input, ctx).unwrap();
        assert_anonymous(&capture);
        assert!(matches!(capture.quantity, Quantity::One));

        // 语法: ?: Type
        let input2 = quote! { #(?: syn::Visibility) };
        let spec2: Capture = parse_capture(input2, ctx).unwrap();
        assert_anonymous(&spec2);
        assert!(matches!(spec2.quantity, Quantity::Optional));
    }

    #[test]
    fn test_parse_joint_nested() {
        let ctx = &mut ParseContext::default();

        // 语法: #( ... )
        // 模拟 Nested 解析，注意这里依赖 pattern 模块的解析逻辑，假设 pattern 也能 parse
        let input = quote! { #(?: -> #( name: Ident )) };
        let result = parse_capture_matcher(input, ctx);

        match dbg!(result) {
            Ok(Matcher {
                kind: MatcherKind::Nested(pattern_list),
                ..
            }) => {
                assert!(!pattern_list.is_empty());
                // 捕获的内容应该是：
                // [可选捕获( -> 具名捕获(name: Ident))]
                if let PatternKind::Capture(cap) = &pattern_list[0].kind {
                    // 可选
                    assert!(matches!(cap.quantity, Quantity::Optional));
                    match &cap.matcher.kind {
                        MatcherKind::Nested(nest) => {
                            let _keyword = Keyword::Rust("->".to_string());
                            assert!(matches!(&nest[0].kind, PatternKind::Literal(_keyword)));
                            let _capture = parse_capture(quote! {#(name: Ident)}, ctx);
                            assert!(matches!(&nest[1].kind, PatternKind::Capture(_capture)))
                        }
                        _ => panic!("Capture matcher should be -> literal"),
                    }
                } else {
                    panic!("First element of patterns should be Capture");
                }
            }
            Ok(_) => panic!("Expected Nested capture type"),
            Err(e) => panic!("Failed to parse joint: {}", e),
        }
    }

    #[test]
    fn test_parse_error_missing_separator() {
        let ctx = &mut ParseContext::default();

        // 错误语法: *[] 中间缺少分隔符
        let input = quote! { #(args*[]: Ident) };
        let result = parse_capture(input, ctx);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "expected '[<separator>]' like '[,]'"
        );
    }

    // --- 2. Lookahead 优化逻辑测试 ---

    #[test]
    fn test_inject_lookahead() {
        let ctx = &mut ParseContext::default();

        // 手动构造 Pattern 列表来测试 inject_lookahead 算法
        // 场景: Capture + Literal -> 应该注入

        // Mock数据构造：为了测试私有函数，我们需要构造 Pattern
        // 假设 Pattern 和 Keyword 是可访问的 (通常在同一 crate 或 test super 中)
        let input = quote!(#(x: Ident));

        let capture: Capture = parse_capture(input, ctx).unwrap();
        let pattern_capture = PatternKind::Capture(capture);
        let pattern_literal = PatternKind::Literal(Keyword::Rust(",".to_string()));

        // Case 1: Capture 后面跟 Literal
        let patterns = vec![
            Pattern {
                kind: pattern_capture.clone(),
                span: Span::call_site(),
                meta: None,
            },
            Pattern {
                kind: pattern_literal.clone(),
                span: Span::call_site(),
                meta: None,
            },
        ];
        let optimized = inject_lookahead(patterns);

        assert_eq!(optimized.len(), 2);
        // 检查第一个 Capture 是否被注入了 lookahead
        if let PatternKind::Capture(Capture {
            edge: Some(edge), ..
        }) = &optimized[0].kind
        {
            if let Keyword::Rust(s) = edge {
                assert_eq!(s, ",");
            } else {
                panic!("Wrong lookahead type");
            }
        } else {
            panic!("Lookahead not injected");
        }

        // Case 2: Capture 后面跟 Capture (不应注入)
        let patterns_consecutive = vec![
            Pattern {
                kind: pattern_capture.clone(),
                span: Span::call_site(),
                meta: None,
            },
            Pattern {
                kind: pattern_capture.clone(),
                span: Span::call_site(),
                meta: None,
            },
        ];
        let optimized_consecutive = inject_lookahead(patterns_consecutive);
        if let PatternKind::Capture(Capture { edge: Some(_), .. }) = &optimized_consecutive[0].kind
        {
            panic!("Should not inject lookahead when followed by another capture");
        }

        // Case 3: Capture 在末尾 (不应注入)
        let patterns_end = vec![Pattern {
            kind: pattern_capture.clone(),
            span: Span::call_site(),
            meta: None,
        }];
        let optimized_end = inject_lookahead(patterns_end);
        if let PatternKind::Capture(Capture { edge: Some(_), .. }) = &optimized_end[0].kind {
            panic!("Should not inject lookahead at end of stream");
        }
    }

    // --- 4. ToTokens 代码生成冒烟测试 ---

    #[test]
    fn test_to_tokens_smoke() {
        let mut compiler = Compiler::new();
        let ctx = &mut ParseContext::default();

        // 只要不 Panic 且有输出即可，详细逻辑校验需要编译生成的代码
        let capture: Capture = parse_capture(quote!(#(x: Ident)), ctx).unwrap();
        let tokens = compiler.compile_capture(&capture);
        assert!(!tokens.is_empty());

        let spec_opt: Capture = parse_capture(quote!(#(x?: Ident)), ctx).unwrap();
        let tokens_opt = compiler.compile_capture(&spec_opt);
        // 生成的代码应该包含 Option 处理逻辑
        assert!(tokens_opt.to_string().contains("Option"));

        let spec_iter: Capture = parse_capture(quote!(#(x*[,]: Ident)), ctx).unwrap();
        let tokens_iter = compiler.compile_capture(&spec_iter);
        // 生成的代码应该包含 parse_terminated
        assert!(tokens_iter.to_string().contains("parse_terminated"));
    }
}
