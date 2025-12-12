use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    ast::input::{CaptureInput, DefineInput},
    codegen::{logic::Compiler, output::generate_output},
};

/// 入口部分
impl Compiler {
    pub fn compile_capture_input(&mut self, input: &CaptureInput) -> TokenStream {
        let mut tokens = TokenStream::new();
        let CaptureInput {
            input, patterns, ..
        } = input;
        let patterns_tokens = self.compile_pattern(patterns);
        let captures = patterns.collect_captures();

        let (capture_init, struct_def, struct_expr, _) = generate_output(&captures, None);

        tokens.extend(quote! {
            {
                trait _Parse: ::syn::parse::Parse {}
                #capture_init
                #struct_def
                let parser = |input: ::syn::parse::ParseStream| -> ::syn::Result<Output> {
                    #capture_init
                    #patterns_tokens
                    ::std::result::Result::Ok(#struct_expr)
                };
                ::syn::parse::Parser::parse2(parser, #input)
            }
        });
        tokens
    }
    pub fn compile_define_input(&mut self, input: &DefineInput) -> TokenStream {
        let mut tokens = TokenStream::new();
        let DefineInput { name, patterns, .. } = input;
        let patterns_tokens = self.compile_pattern(patterns);

        let captures = patterns.collect_captures();

        let (capture_init, struct_def, struct_expr, _) =
            generate_output(&captures, Some(name.clone()));

        tokens.extend(quote! {
            #struct_def
            impl ::syn::parse::Parse for #name {
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    trait _Parse: ::syn::parse::Parse {}
                    #capture_init
                    #patterns_tokens
                    ::std::result::Result::Ok(#struct_expr)
                }
            }
        });
        tokens
    }
}
