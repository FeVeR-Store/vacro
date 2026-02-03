use proc_macro2::TokenStream;
use syn::{Ident, Local, Token, Visibility};

use crate::ast::node::Pattern;

#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub struct BindInput {
    pub local: Local,
    pub input: Ident,
    pub _arrow: Token![->],
    pub patterns: Pattern,
    pub suffix: TokenStream,
}

#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub struct DefineInput {
    pub visibility: Visibility,
    pub name: Ident,
    pub _colon: Token![:],
    pub patterns: Pattern,
}

#[cfg(test)]
mod tests {
    use crate::codegen::logic::Compiler;

    use super::*;
    use quote::quote;

    use syn::parse2;

    // --- 1. 语法解析测试 (impl Parse) ---

    #[test]
    fn test_parse_valid_input() {
        // 测试标准语法: input_var -> pattern
        let stream = quote! { let res = (my_tokens -> name: Ident); };
        let result: BindInput = parse2(stream).unwrap();

        assert_eq!(result.input.to_string(), "my_tokens");
        // 验证 patterns 能够成功解析 (PatternList 的具体解析由它自己的测试保证)
        // 这里只要不 panic 且结构存在即可
    }

    #[test]
    fn test_parse_complex_patterns() {
        // 测试复杂模式: input -> #(a: Ident)
        let stream = quote! {
            let res = (input_stream -> #(x: Ident));
        };
        let result: BindInput = parse2(stream).unwrap();

        assert_eq!(result.input.to_string(), "input_stream");
    }

    #[test]
    fn test_parse_error_missing_arrow() {
        // 错误语法: 缺少箭头
        let stream = quote! { my_tokens name: Ident };
        let result = parse2::<BindInput>(stream);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "expected `let`");
    }

    #[test]
    fn test_parse_error_missing_input() {
        // 错误语法: 缺少输入变量名
        let stream = quote! { -> name: Ident };
        let result = parse2::<BindInput>(stream);

        assert!(result.is_err());
    }

    // --- 2. 代码生成测试 (impl ToTokens) ---

    #[test]
    fn test_to_tokens_structure() {
        let mut compiler = Compiler::new();
        // 构建一个有效的 CaptureInput
        let stream = quote! { let res = (tokens -> #(val: Ident)); };
        let capture_input: BindInput = parse2(stream).unwrap();

        // 生成代码
        let output = compiler.compile_capture_input(&capture_input);
        let output_str = output.to_string();

        // --- 静态检查生成的代码结构 ---

        // 1. 检查是否生成了辅助 Trait
        assert!(output_str.contains("trait _Parse"));

        // 2. 检查是否生成了 Output 结构体 (由 generate_output 生成)
        assert!(output_str.contains("struct Output"));

        // 3. 检查闭包定义
        // 闭包应该接受 ParseStream 并返回 Result<Output>
        assert!(output_str.contains("| input : :: syn :: parse :: ParseStream |"));
        assert!(output_str.contains("-> :: syn :: Result < Output >"));

        // 4. 检查是否调用了 syn::parse::Parser::parse2
        assert!(output_str.contains(":: syn :: parse :: Parser :: parse2"));

        // 5. 检查是否传入了正确的输入变量 (这里是 `tokens.into()`，因为要兼容proc_macro::TokenStream和proc_macro2::TokenStream)
        // parse2(parser, tokens . into ())
        assert!(output_str.contains("(parser , tokens . into ())"));
    }

    #[test]
    fn test_to_tokens_suffix() {
        let mut compiler = Compiler::new();
        // 确保生成的代码被包裹在一个独立的块 {} 中
        // 这样生成的 struct Output 不会污染外部作用域
        let stream = quote! { let res = (t -> v: Ident)?; };
        let capture_input: BindInput = parse2(stream).unwrap();
        let output = compiler.compile_capture_input(&capture_input);

        let output_str = output.to_string();
        assert!(output_str.trim().starts_with("let res ="));
        assert!(output_str.trim().ends_with("} ? ;"));
    }

    // 集成测试模拟：检查生成的逻辑是否包含捕获组初始化
    #[test]
    fn test_generated_logic_contains_initialization() {
        let mut compiler = Compiler::new();
        let stream = quote! { let res = (t -> #(val?: Ident)); };
        let capture_input: BindInput = parse2(stream).unwrap();
        let output = compiler.compile_capture_input(&capture_input).to_string();

        // generate_output 会生成类似 `let mut val = None;` 的代码
        // 虽然具体变量名可能因为 generate_output 的实现而不同(如果是 named 应该是 val)
        // 我们检查是否包含 Option::None 的初始化逻辑
        assert!(output.contains("Option :: None"));

        // 检查最终结果是否返回 Output 结构体
        assert!(output.contains(":: std :: result :: Result :: Ok"));
    }
}
