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
        if ::std::env::var("VACRO_TRACE").is_ok() {
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
