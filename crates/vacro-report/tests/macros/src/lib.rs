use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use vacro_report::{help, scope};

#[proc_macro]
#[scope]
pub fn parse_stmt(input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let output: Stmt = parse_quote!(#input);
    quote! {#output}.into()
}

#[proc_macro]
#[scope]
pub fn parse_stmt_spanned(input: TokenStream) -> TokenStream {
    let input: proc_macro2::TokenStream = input.into();
    let span = Span::call_site();
    let output: Stmt = parse_quote_spanned! {span => #input};
    quote! {#output}.into()
}

use syn::{
    parse::{Parse, Parser},
    parse_quote, parse_quote_spanned, Ident, LitBool, LitStr, Stmt,
};

help!(MyLitStr: LitStr {
    error: "这里需要一个String字面量",
    help: "应该在两侧添加双引号",
    example: "my-string"
});

#[proc_macro]
pub fn parse_help(input: TokenStream) -> TokenStream {
    match MyLitStr::parse.parse(input) {
        Ok(token) => quote! {#token}.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

help!(Bool:
    LitBool {
        error: "此处需要一个bool字面量，接收到的是：{input}",
        help: "尝试`true`或`false`",
        example: (true | false) // example 字段是要展示的示例字段，在生成错误信息与使用示例时使用；它接受一段TokenStream，并且将直接展示你传入的内容
    }
);

#[proc_macro]
pub fn parse_roles(input: TokenStream) -> TokenStream {
    use vacro_parser::bind;
    bind! {
        let result = (input -> {
            #(roles*[,]: #(pair: #(name: Ident): #(enable: Bool)))
        });
    };
    match result {
        Ok(_) => (),
        Err(err) => return err.into_compile_error().into(),
    }

    quote! {}.into()
}
