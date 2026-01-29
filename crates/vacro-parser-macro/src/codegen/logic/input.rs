use proc_macro2::TokenStream;
use quote::quote;
use syn::Local;

use crate::{
    ast::input::{BindInput, DefineInput},
    codegen::{logic::Compiler, output::generate_output},
};

/// 入口部分
impl Compiler {
    pub fn compile_capture_input(&mut self, input: &BindInput) -> TokenStream {
        let mut tokens = TokenStream::new();

        let BindInput {
            input,
            patterns,
            local: Local { let_token, pat, .. },
            suffix,
            ..
        } = input;
        let patterns_tokens = self.compile_pattern(patterns);
        let captures = patterns.collect_captures();

        let Compiler {
            shared_definition,
            scoped_definition,
        } = &self;

        let (capture_init, struct_def, struct_expr, _) = generate_output(&captures, None);
        tokens.extend(quote! {
            #(#shared_definition)*
            #let_token #pat = {
                #(#scoped_definition)*
                use ::syn::parse::Parse;
                trait _Parse: Parse {}
                #struct_def
                let parser = |input: ::syn::parse::ParseStream| -> ::syn::Result<Output> {
                    #capture_init
                    #patterns_tokens
                    ::std::result::Result::Ok(#struct_expr)
                };
                ::syn::parse::Parser::parse2(parser, #input.into())
            }#suffix
        });
        tokens
    }
    pub fn compile_define_input(&mut self, input: &DefineInput) -> TokenStream {
        let mut tokens = TokenStream::new();
        let DefineInput { name, patterns, .. } = input;
        let patterns_tokens = self.compile_pattern(patterns);

        let captures = patterns.collect_captures();

        let Compiler {
            shared_definition,
            scoped_definition,
        } = &self;

        let (capture_init, struct_def, struct_expr, _) =
            generate_output(&captures, Some(name.clone()));

        tokens.extend(quote! {
            #(#shared_definition)*
            #struct_def
            impl ::syn::parse::Parse for #name {
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    #(#scoped_definition)*
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
