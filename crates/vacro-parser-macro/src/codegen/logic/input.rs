use proc_macro2::TokenStream;
use quote::quote;
use syn::Local;

use crate::{
    ast::input::{BindInput, DefineInput},
    codegen::{
        logic::Compiler,
        output::{generate_example, generate_output},
    },
    scope_context,
};

/// 入口部分
impl Compiler {
    pub fn compile_capture_input(&mut self, input: &BindInput) -> TokenStream {
        scope_context::set_scope_ident(None);
        let mut tokens = TokenStream::new();

        let BindInput {
            input,
            patterns,
            local: Local { let_token, pat, .. },
            suffix,
            ..
        } = input;

        self.target = Self::pat_to_ident(pat);

        let patterns_tokens = self.compile_pattern(patterns);
        let captures = patterns.collect_captures();
        let example_items = patterns.collect_example();

        let Compiler {
            shared_definition,
            scoped_definition,
            ..
        } = &self;

        let (capture_init, struct_def, struct_expr, _) = generate_output(&captures, None, None);
        let (example_doc, extra) = generate_example(&example_items, false, false, false);
        let extra = extra.iter().map(|e| {
            quote! {
                #[doc = #e]
            }
        });
        tokens.extend(quote! {
            #(#shared_definition)*
            #let_token #pat = {
                #(#scoped_definition)*
                use ::syn::parse::Parse;
                #[doc = #example_doc]
                #(#extra)*
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
        let DefineInput {
            name,
            patterns,
            visibility,
            ..
        } = input;

        self.target = name.clone();
        scope_context::set_scope_ident(Some(self.get_private_scope_ident()));

        let patterns_tokens = self.compile_pattern(patterns);

        let captures = patterns.collect_captures();
        let example_items = patterns.collect_example();

        let Compiler {
            shared_definition,
            scoped_definition,
            ..
        } = &self;

        let (capture_init, struct_def, struct_expr, _) =
            generate_output(&captures, Some(name.clone()), Some(visibility.clone()));

        let (example_doc, extra) = generate_example(&example_items, false, false, false);
        let extra = extra.iter().map(|e| {
            quote! {
                #[doc = #e]
            }
        });

        tokens.extend(quote! {
            #(#shared_definition)*
            #[doc = #example_doc]
            #(#extra)*
            #struct_def
            impl ::syn::parse::Parse for #name {
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    #(#scoped_definition)*
                    #capture_init
                    #patterns_tokens
                    ::std::result::Result::Ok(#struct_expr)
                }
            }
        });
        scope_context::set_scope_ident(None);
        tokens
    }
}
