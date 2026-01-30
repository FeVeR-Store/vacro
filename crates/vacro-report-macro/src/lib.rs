use proc_macro::TokenStream;
use vacro_doc_i18n::doc_i18n;

use crate::impls::{help_impl, parse_quote_impl, report_scope_impl};

mod impls;
mod utils;

#[doc_i18n]
/// @cn 作用域标记宏。
/// @en Scope marker macro.
///
/// ::: @cn
///
/// 将此属性添加到函数上，以启用该作用域内 `parse_quote!` 调用的增强错误报告功能。
/// 当 `parse_quote!` 解析失败时，它会打印生成的 TokenStream 并高亮错误位置，而不是仅仅 Panic。
///
/// :::
///
/// ::: @en
///
/// Attach this attribute to a function to enable enhanced error reporting for `parse_quote!` calls within that scope.
/// When `parse_quote!` fails, it will print the generated TokenStream and highlight the error, instead of just panicking.
/// :::
#[proc_macro_attribute]
pub fn scope(attr: TokenStream, item: TokenStream) -> TokenStream {
    report_scope_impl(attr.into(), item.into()).into()
}

#[doc_i18n]
/// ::: @cn
/// 增强版 `parse_quote!`（内部使用）。
///
/// 该宏通常不直接由用户调用，而是通过 `#[scope]` 宏重写标准的 `syn::parse_quote!` 调用来自动使用。
/// :::
/// ::: @en
/// Enhanced `parse_quote!` (Internal Use).
///
/// This macro is usually not called directly. It is automatically used when `#[scope]` rewrites
/// standard `syn::parse_quote!` calls.
/// :::
#[proc_macro]
pub fn parse_quote(input: TokenStream) -> TokenStream {
    parse_quote_impl(input.into(), false).into()
}

#[doc_i18n]
/// ::: @cn
/// 增强版 `parse_quote_spanned!`（内部使用）。
///
/// 该宏通常不直接由用户调用，而是通过 `#[scope]` 宏重写标准的 `syn::parse_quote_spanned!` 调用来自动使用。
/// :::
/// ::: @en
/// Enhanced `parse_quote_spanned!` (Internal Use).
///
/// This macro is usually not called directly. It is automatically used when `#[scope]` rewrites
/// standard `syn::parse_quote_spanned!` calls.
/// :::
#[proc_macro]
pub fn parse_quote_spanned(input: TokenStream) -> TokenStream {
    parse_quote_impl(input.into(), true).into()
}

#[doc_i18n]
/// @cn 创建带有增强错误报告和智能提示的包装类型。
/// @en Creates a wrapper type with enhanced error reporting and smart hints.
///
/// ::: @cn
///
/// 此宏定义一个代理底层类型（如 `syn::Ident`）的包装结构体，允许附加自定义的错误信息和帮助文本。
///
/// 若启用了 `vacro-parser` 支持，还可以提供 `example` 字段。这将激活智能提示联动，使解析器在报错时显示具体的代码示例而非类型名。
///
/// ```rust
/// # use vacro::help;
/// help!(Arithmetic: syn::Expr {
///     error: "expected an arithmetic expression",
///     help: "try using explicit values or operations",
///     example: "1 + 2" // 可选：用于 vacro-parser 智能提示
/// });
/// ```
/// :::
///
/// ::: @en
///
/// Defines a wrapper struct that proxies an underlying type (like `syn::Ident`), allowing custom error messages and help text to be attached.
///
/// If `vacro-parser` support is enabled, the `example` field can be provided. This activates smart hint synergy, causing the parser to show specific code examples instead of type names when errors occur.
///
/// ```rust
/// # use vacro::help;
/// help!(Arithmetic: syn::Expr {
///     error: "expected an arithmetic expression",
///     help: "try using explicit values or operations",
///     example: "1 + 2" // Optional: for vacro-parser smart hints
/// });
/// ```
/// :::
#[proc_macro]
pub fn help(input: TokenStream) -> TokenStream {
    help_impl(input.into()).into()
}
