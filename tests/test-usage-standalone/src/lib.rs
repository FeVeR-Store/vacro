use proc_macro::TokenStream;
use quote::quote;
use syn::LitStr;
use vacro_parser::bind;
use vacro_report::help;

help!(MyLitStr: LitStr {
    error: "这里需要一个String字面量",
    help: "应该在两侧添加双引号",
    example: "my-string"
});

#[proc_macro]
pub fn parse_help(input: TokenStream) -> TokenStream {
    // str("xxx")
    bind! {
        let result = (input -> str(#(lit: MyLitStr)));
    }

    match result {
        Ok(res) => {
            let lit = res.lit;
            quote!(#lit).into()
        }
        Err(err) => err.into_compile_error().into(),
    }
}
