use proc_macro2::Span;
use syn::{
    token::{self},
    Ident, Token, Type,
};

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
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug, PartialEq))]
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

    /// 枚举结构 (e.g. `EnumName { Type1, Type2 }`)
    Enum {
        enum_name: Type,
        variants: Vec<(EnumVariant, Matcher)>,
    },
}

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub enum EnumVariant {
    Type {
        ident: Type,
        /// 实际是标识符（已判断），解析为Type仅易于开发
        ty: Type,
    },
    Capture {
        ident: Type,
        named: bool,
        fields: Vec<FieldDef>,
        pattern: Pattern,
    },
}

/// 数量限定
#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug, PartialEq))]
pub enum Quantity {
    One,                   // 默认
    Optional,              // ?
    Many(Option<Keyword>), // * 或 *[,]
}

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug, PartialEq))]
pub struct FieldDef {
    pub name: Ident,
    pub ty: Type,
    pub is_optional: bool, // 标记是否已被 Option 包裹
    pub is_inline: bool,
}

impl Capture {
    pub fn collect_captures(&self) -> Vec<FieldDef> {
        // 1. 先收集原始字段 (Base Fields)
        let mut fields = self.matcher.collect_captures(&self.binder);
        // 2. 根据当前的 Quantity 对字段类型进行“包装” (Type Wrapping)
        // 这就是解决 #(?: #(ret: Type)) 问题的关键
        self.apply_quantity_wrapping(&mut fields);

        fields
    }

    fn apply_quantity_wrapping(&self, fields: &mut Vec<FieldDef>) {
        if fields.is_empty() {
            return;
        }

        match &self.quantity {
            Quantity::One => {
                // 默认情况，不做改变
            }
            Quantity::Optional => {
                // 对应 ?: 或 ?
                for field in fields {
                    // 避免双重 Option (可选的优化)
                    if !field.is_optional {
                        let ty = &field.ty;
                        field.ty = syn::parse_quote!(::std::option::Option<#ty>);
                        field.is_optional = true;
                    }
                }
            }
            Quantity::Many(sep) => {
                // 对应 * 或 *[,]
                for field in fields {
                    let ty = &field.ty;
                    // Punctuated 本身就是容器，通常不需要再标 is_optional
                    if let Some(s) = sep {
                        field.ty = syn::parse_quote!(::syn::punctuated::Punctuated<#ty, #s>);
                    } else {
                        field.ty = syn::parse_quote!(::std::vec::Vec<#ty>);
                    }
                    // Many 模式下，字段通常初始化为空集合，所以不算 Optional (Option::None)
                    field.is_optional = false;
                }
            }
        }
    }
}

impl Matcher {
    fn collect_captures(&self, binder: &Binder) -> Vec<FieldDef> {
        match &self.kind {
            MatcherKind::SynType(ty) | MatcherKind::Enum { enum_name: ty, .. } => {
                generate_captures(ty, &binder)
                    .map(|def| vec![def])
                    .unwrap_or(vec![])
                // 处理叶子节点：只有 Named 和 Inline 产生字段
            }

            MatcherKind::Nested(children) => {
                // 处理嵌套节点：递归收集所有子 Pattern 的字段
                children.iter().flat_map(|p| p.collect_captures()).collect()
            }
        }
    }
}

fn generate_captures(ty: &Type, binder: &Binder) -> Option<FieldDef> {
    match binder {
        Binder::Named(ident) => Some(FieldDef {
            name: ident.clone(),
            ty: ty.clone(),
            is_optional: false, // 初始状态
            is_inline: false,
        }),
        Binder::Inline(idx) => Some(FieldDef {
            name: quote::format_ident!("_{}", idx),
            ty: ty.clone(),
            is_optional: false,
            is_inline: true,
        }),
        Binder::Anonymous => None, // _: Type 不产生字段
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{keyword::Keyword, node::PatternKind},
        codegen::logic::Compiler,
        syntax::context::ParseContext,
    };

    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{
        parse::{ParseStream, Parser},
        parse_quote, Result,
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
        assert_eq!(capture.quantity, Quantity::One);
        assert!(matches!(capture.matcher.kind, MatcherKind::SynType(_)));
    }

    #[test]
    fn test_parse_optional_named() {
        let ctx = &mut ParseContext::default();
        // 语法: name?: Type
        let input = quote! { #(maybe_val?: u32) };
        let capture: Capture = parse_capture(input, ctx).unwrap();
        assert_named(&capture, "maybe_val");
        assert_eq!(capture.quantity, Quantity::Optional);
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
        assert_eq!(spec1.quantity, Quantity::One);

        let input2 = quote! { #(@?: Ident) };
        let spec2: Capture = parse_capture(input2, ctx).unwrap();
        assert_inline(&spec2);
        assert_eq!(spec2.quantity, Quantity::Optional);
    }

    #[test]
    fn test_parse_anonymous() {
        let ctx = &mut ParseContext::default();

        // 语法: Type (无名称)
        let input = quote! { #(syn::Type) };
        let capture: Capture = parse_capture(input, ctx).unwrap();
        assert_anonymous(&capture);
        assert_eq!(capture.quantity, Quantity::One);

        // 语法: ?: Type
        let input2 = quote! { #(?: syn::Visibility) };
        let spec2: Capture = parse_capture(input2, ctx).unwrap();
        assert_anonymous(&spec2);
        assert_eq!(spec2.quantity, Quantity::Optional);
    }

    #[test]
    fn test_parse_joint_nested() {
        let ctx = &mut ParseContext::default();

        // 语法: #( ... )
        // 模拟 Nested 解析，注意这里依赖 pattern 模块的解析逻辑，假设 pattern 也能 parse
        let input = quote! { #(?: -> #( name: Ident )) };
        let result = parse_capture_matcher(input, ctx);

        match result {
            Ok(Matcher {
                kind: MatcherKind::Nested(pattern_list),
                ..
            }) => {
                assert!(!pattern_list.is_empty());
                // 捕获的内容应该是：
                // [可选捕获( -> 具名捕获(name: Ident))]
                if let PatternKind::Capture(cap) = &pattern_list[0].kind {
                    // 可选
                    assert_eq!(cap.quantity, Quantity::Optional);
                    match &cap.matcher.kind {
                        MatcherKind::Nested(nest) => {
                            match &nest[0].kind {
                                PatternKind::Literal(keyword) => {
                                    assert_eq!(keyword, &Keyword::Rust("->".to_string()))
                                }
                                _ => panic!("First element of patterns should be Literal"),
                            }
                            match &nest[1].kind {
                                PatternKind::Capture(capture) => {
                                    assert_eq!(capture.quantity, Quantity::One);
                                    assert_eq!(capture.binder, Binder::Named(parse_quote!(name)));
                                    match &capture.matcher.kind {
                                        MatcherKind::SynType(type_matcher) => {
                                            assert_eq!(type_matcher, &parse_quote!(Ident));
                                        }
                                        _ => panic!("expected SynType"),
                                    }
                                }
                                _ => panic!("Second element of patterns should be Capture"),
                            }
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

    #[test]
    fn test_parse_enum_capture() {
        let ctx = &mut ParseContext::default();

        let expect_enum_name: Type = parse_quote!(Enum);
        // 语法: #(args: EnumName { Type1, Type2 })
        let input = quote! { #(args: Enum { Type, syn::Ident }) };
        let result = parse_capture(input, ctx).unwrap();

        let MatcherKind::Enum {
            enum_name,
            variants,
        } = result.matcher.kind
        else {
            panic!("Expected Enum matcher kind");
        };

        assert_eq!(enum_name, expect_enum_name);
        assert_eq!(variants.len(), 2);

        match &variants[0].0 {
            EnumVariant::Type { ident, ty } => {
                assert_eq!(ident, &parse_quote!(Type));
                assert_eq!(ty, &parse_quote!(Type));
            }
            _ => panic!("Expected EnumVariant::Type"),
        }

        match &variants[1].0 {
            EnumVariant::Type { ident, ty } => {
                assert_eq!(ident, &parse_quote!(Ident));
                assert_eq!(ty, &parse_quote!(syn::Ident));
            }
            _ => panic!("Expected EnumVariant::Type"),
        }

        // 语法 #(args: EnumName { VariantName: Type })
        let input = quote! { #(args: Enum { Ty: Type, Id: Ident }) };
        let result = parse_capture(input, ctx).unwrap();

        let MatcherKind::Enum {
            enum_name,
            variants,
        } = result.matcher.kind
        else {
            panic!("Expected Enum matcher kind");
        };

        assert_eq!(enum_name, expect_enum_name);
        assert_eq!(variants.len(), 2);
        match &variants[0].0 {
            EnumVariant::Type { ident, ty } => {
                assert_eq!(ident, &parse_quote!(Ty));
                assert_eq!(ty, &parse_quote!(Type));
            }
            _ => panic!("Expected EnumVariant::Type"),
        }
        match &variants[1].0 {
            EnumVariant::Type { ident, ty } => {
                assert_eq!(ident, &parse_quote!(Id));
                assert_eq!(ty, &parse_quote!(Ident));
            }
            _ => panic!("Expected EnumVariant::Type"),
        }

        // 语法 #(args: EnumName { Capture: #(..) })
        let input = quote! {#(args: Enum { FnArg: #(id: Ident): #(ty: Type), WithDefault: #(name: Ident) = #(default: Expr) })};
        let result = parse_capture(input, ctx).unwrap();
        let MatcherKind::Enum {
            enum_name,
            variants,
        } = result.matcher.kind
        else {
            panic!("Expected Enum matcher kind");
        };

        assert_eq!(enum_name, expect_enum_name);
        assert_eq!(variants.len(), 2);

        match &variants[0].0 {
            EnumVariant::Capture {
                named,
                ident,
                fields,
                ..
            } => {
                assert!(named);
                assert_eq!(ident, &parse_quote!(FnArg));
                assert_eq!(
                    fields,
                    &vec![
                        FieldDef {
                            ty: parse_quote!(Ident),
                            name: parse_quote!(id),
                            is_inline: false,
                            is_optional: false,
                        },
                        FieldDef {
                            ty: parse_quote!(Type),
                            name: parse_quote!(ty),
                            is_inline: false,
                            is_optional: false,
                        },
                    ]
                );
            }
            _ => panic!("Expected EnumVariant::Capture"),
        }

        match &variants[1].0 {
            EnumVariant::Capture {
                named,
                ident,
                fields,
                ..
            } => {
                assert!(named);
                assert_eq!(ident, &parse_quote!(WithDefault));
                assert_eq!(
                    fields,
                    &vec![
                        FieldDef {
                            ty: parse_quote!(Ident),
                            name: parse_quote!(name),
                            is_inline: false,
                            is_optional: false,
                        },
                        FieldDef {
                            ty: parse_quote!(Expr),
                            name: parse_quote!(default),
                            is_inline: false,
                            is_optional: false,
                        },
                    ]
                );
            }
            _ => panic!("Expected EnumVariant::Capture"),
        }

        // 语法 #(args: EnumName { Ident, Expr: #(@: Ident): #(@: Expr) })
        let input = quote! {#(args: Enum { FnArg: #(@: Ident): #(@: Type), WithDefault: #(@: Ident) = #(@: Expr) })};
        let result = parse_capture(input, ctx).unwrap();
        let MatcherKind::Enum {
            enum_name,
            variants,
        } = result.matcher.kind
        else {
            panic!("Expected Enum matcher kind");
        };

        assert_eq!(enum_name, expect_enum_name);
        assert_eq!(variants.len(), 2);

        match &variants[0].0 {
            EnumVariant::Capture {
                named,
                ident,
                fields,
                ..
            } => {
                assert!(!named);
                assert_eq!(ident, &parse_quote!(FnArg));
                assert_eq!(
                    fields,
                    &vec![
                        FieldDef {
                            ty: parse_quote!(Ident),
                            name: parse_quote!(_0),
                            is_inline: true,
                            is_optional: false,
                        },
                        FieldDef {
                            ty: parse_quote!(Type),
                            name: parse_quote!(_1),
                            is_inline: true,
                            is_optional: false,
                        },
                    ]
                );
            }
            _ => panic!("Expected EnumVariant::Capture"),
        }
        match &variants[1].0 {
            EnumVariant::Capture {
                named,
                ident,
                fields,
                ..
            } => {
                assert!(!named);
                assert_eq!(ident, &parse_quote!(WithDefault));
                assert_eq!(
                    fields,
                    &vec![
                        FieldDef {
                            ty: parse_quote!(Ident),
                            name: parse_quote!(_0),
                            is_inline: true,
                            is_optional: false,
                        },
                        FieldDef {
                            ty: parse_quote!(Expr),
                            name: parse_quote!(_1),
                            is_inline: true,
                            is_optional: false,
                        },
                    ]
                );
            }
            _ => panic!("Expected EnumVariant::Capture"),
        }

        // 混合语法
        let input = quote! {#(args: Enum {
            syn::Ident,
            Ty: Type,
            FnArg: #(id: Ident): #(ty: Type),
            WithDefault: #(@: Ident) = #(@: Expr) })
        };
        let result = parse_capture(input, ctx).unwrap();
        let MatcherKind::Enum {
            enum_name,
            variants,
        } = result.matcher.kind
        else {
            panic!("Expected Enum matcher kind");
        };

        assert_eq!(enum_name, expect_enum_name);
        assert_eq!(variants.len(), 4);

        match &variants[0].0 {
            EnumVariant::Type { ident, ty } => {
                assert_eq!(ident, &parse_quote!(Ident));
                assert_eq!(ty, &parse_quote!(syn::Ident));
            }
            _ => panic!("Expected EnumVariant::Type"),
        }

        match &variants[1].0 {
            EnumVariant::Type { ident, ty } => {
                assert_eq!(ident, &parse_quote!(Ty));
                assert_eq!(ty, &parse_quote!(Type));
            }
            _ => panic!("Expected EnumVariant::Type"),
        }

        match &variants[2].0 {
            EnumVariant::Capture {
                named,
                ident,
                fields,
                ..
            } => {
                assert!(named);
                assert_eq!(ident, &parse_quote!(FnArg));
                assert_eq!(
                    fields,
                    &vec![
                        FieldDef {
                            ty: parse_quote!(Ident),
                            name: parse_quote!(id),
                            is_inline: false,
                            is_optional: false,
                        },
                        FieldDef {
                            ty: parse_quote!(Type),
                            name: parse_quote!(ty),
                            is_inline: false,
                            is_optional: false,
                        },
                    ]
                );
            }
            _ => panic!("Expected EnumVariant::Capture"),
        }

        match &variants[3].0 {
            EnumVariant::Capture {
                named,
                ident,
                fields,
                ..
            } => {
                assert!(!named);
                assert_eq!(ident, &parse_quote!(WithDefault));
                assert_eq!(
                    fields,
                    &vec![
                        FieldDef {
                            ty: parse_quote!(Ident),
                            name: parse_quote!(_0),
                            is_inline: true,
                            is_optional: false,
                        },
                        FieldDef {
                            ty: parse_quote!(Expr),
                            name: parse_quote!(_1),
                            is_inline: true,
                            is_optional: false,
                        },
                    ]
                );
            }
            _ => panic!("Expected EnumVariant::Capture"),
        }
    }
    // --- 2. ToTokens 代码生成冒烟测试 ---

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

    #[test]
    fn test_error_mixed_inline_and_named() {
        let ctx = &mut ParseContext::default();

        let input = quote! {#(named: Ident)};
        parse_capture(input, ctx).unwrap();
        let input = quote! {#(@: Ident)};
        let err = parse_capture(input, ctx).unwrap_err();
        assert!(err
            .to_string()
            .contains("unexpected inline capture; previous captures were named"));

        // 重置
        let ctx = &mut ParseContext::default();

        let input = quote! {#(@: Ident)};
        parse_capture(input, ctx).unwrap();
        let input = quote! {#(named: Ident)};
        let err = parse_capture(input, ctx).unwrap_err();
        assert!(err
            .to_string()
            .contains("unexpected named capture; previous captures were inline"));
    }

    #[test]
    fn test_error_missing_colon() {
        let ctx = &mut ParseContext::default();
        // 错误语法: #(name Ident) 缺少冒号
        let input = quote!(#(name Ident));
        let result = parse_capture(input, ctx);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "expected ':' after capture name"
        );
    }

    #[test]
    fn test_error_invalid_separator() {
        let ctx = &mut ParseContext::default();
        // 错误语法: #(name*[]: Ident) 分隔符为空
        let input = quote!(#(name*[]: Ident));
        let result = parse_capture(input, ctx);

        assert!(result.is_err());
        // 具体的错误信息取决于 bracketed! 空内容的判定
        assert!(result.unwrap_err().to_string().contains("expected"));
    }
}
