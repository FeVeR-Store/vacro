use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Expr, Result, Token,
};

use crate::utils::crate_name;

struct LogInput {
    level: Expr,
    _comma: Option<Token![,]>,
    args: TokenStream,
}

impl Parse for LogInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let level: Expr = input.parse()?;
        let _comma: Option<Token![,]> = input.parse().ok();
        let args: TokenStream = input.parse()?;
        Ok(LogInput {
            level,
            _comma,
            args,
        })
    }
}

// 公共的代码生成逻辑
fn gen_log_code(level_expr: TokenStream, args: TokenStream) -> TokenStream {
    let pkg = crate_name();

    // 构造 format! 调用
    let msg_expr = if args.is_empty() {
        quote! { String::new() }
    } else {
        quote! { format!(#args) }
    };

    parse_quote! {
        if cfg!(debug_assertions) || ::std::env::var("VACRO_TRACE").is_ok() {
            #pkg::__private::log(#level_expr.to_string(), #msg_expr);
        }
    }
}

#[cfg_attr(test, vacro::report::scope)]
pub fn log_impl(input: TokenStream) -> Result<TokenStream> {
    let input: LogInput = syn::parse2(input)?;
    // 对于 log!(Level::Info, ...) 这种情况，level 是一个表达式
    let level = input.level;
    let args = input.args;
    Ok(gen_log_code(quote! { #level }, args))
}

// 快捷宏的入口，例如 info!("msg")
// 此时 input 只有 ("msg")，没有 level
pub fn shortcut_impl(level_str: &str, input: TokenStream) -> Result<TokenStream> {
    let pkg = crate_name();
    // 这里我们直接构造 Level 枚举的路径，例如 ::vacro::trace::Level::Info
    // 注意：我们需要确保 pkg 引用的是正确的 crate 根
    let level_variant = syn::Ident::new(level_str, proc_macro2::Span::call_site());
    let level_expr = quote! { #pkg::Level::#level_variant };

    Ok(gen_log_code(level_expr, input))
}
