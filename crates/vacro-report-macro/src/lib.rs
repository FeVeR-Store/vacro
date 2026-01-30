use proc_macro::TokenStream;
use vacro_doc_i18n::doc_i18n;

use crate::impls::{help_impl, parse_quote_impl, report_scope_impl};

mod impls;
mod utils;

#[proc_macro_attribute]
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
pub fn scope(attr: TokenStream, item: TokenStream) -> TokenStream {
    report_scope_impl(attr.into(), item.into()).into()
}

#[proc_macro]
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
pub fn parse_quote(input: TokenStream) -> TokenStream {
    parse_quote_impl(input.into(), false).into()
}

#[proc_macro]
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
pub fn parse_quote_spanned(input: TokenStream) -> TokenStream {
    parse_quote_impl(input.into(), true).into()
}

#[proc_macro]
pub fn help(input: TokenStream) -> TokenStream {
    help_impl(input.into()).into()
}
