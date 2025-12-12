use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    ast::{
        capture::{CaptureMode, CaptureSpec, CaptureType, ExposeMode},
        pattern::PatternList,
    },
    codegen::{logic::Compiler, output::generate_output},
    transform::lookahead::inject_lookahead,
};

impl Compiler {
    pub fn compile_capture_spec(&mut self, spec: &CaptureSpec) -> TokenStream {
        let mut tokens = TokenStream::new();
        let CaptureSpec { ty, mode, name, .. } = spec;
        let receiver = match &name {
            ExposeMode::Named(ident) => {
                quote! {#ident = }
            }
            ExposeMode::Inline(i) => {
                let id = format_ident!("_{}", i.to_string());
                quote! {#id = }
            }
            _ => quote! {},
        };
        let t = match (name, mode, ty) {
            (_, CaptureMode::Once, CaptureType::Type(ty)) => {
                quote! {
                    {
                        #receiver input.parse::<#ty>()?;
                    }
                }
            }
            (_, CaptureMode::Optional, CaptureType::Type(ty)) => {
                quote! {
                    {
                        let _fork = input.fork();
                        if ::std::result::Ok(_parsed) = _fork.parse::<#ty>() {
                            #receiver ::std::option::Option::Some(_parsed);
                        }
                    }
                }
            }
            (_, CaptureMode::Iter(separator), CaptureType::Type(ty)) => {
                quote! {
                    {
                        #[allow(non_local_definitions)]
                        impl _Parse for #ty {}
                        #receiver input.parse_terminated(#ty::parse, #separator)?;
                    }
                }
            }
            (ExposeMode::Anonymous, CaptureMode::Once, CaptureType::Joint(_patterns)) => {
                let optimized_list = inject_lookahead(_patterns.list.clone());

                let patterns = PatternList {
                    list: optimized_list,
                    capture_list: _patterns.capture_list.clone(),
                    parse_context: _patterns.parse_context.clone(),
                };
                let pattern_tokens = self.compile_pattern_list(&patterns);
                quote! {
                    {
                        #pattern_tokens
                    }
                }
            }
            (ExposeMode::Anonymous, CaptureMode::Optional, CaptureType::Joint(_patterns)) => {
                let optimized_list = inject_lookahead(_patterns.list.clone());

                let patterns = PatternList {
                    list: optimized_list,
                    capture_list: _patterns.capture_list.clone(),
                    parse_context: _patterns.parse_context.clone(),
                };

                let joint_token = self.compile_pattern_list(&patterns);
                let (capture_init, struct_def, struct_expr) =
                    generate_output(patterns.capture_list.clone(), None, &patterns.parse_context);
                let fields = patterns
                    .capture_list
                    .lock()
                    .unwrap()
                    .iter()
                    .map(|(name, ..)| name.clone())
                    .collect::<Vec<_>>();

                let assigns_err = fields.iter().map(|ident| {
                    quote! { #ident = ::std::option::Option::None; }
                });
                let assigns_ok = fields.iter().map(|ident| {
                    quote! { #ident = ::std::option::Option::Some(output.#ident); }
                });

                quote! {
                    #struct_def
                    let _parser = |input: ::syn::parse::ParseStream| -> ::syn::Result<Output> {
                        #capture_init
                        #joint_token
                        ::std::result::Result::Ok(#struct_expr)
                    };
                    match _parser(input) {
                        ::std::result::Result::Ok(output) => {
                            #(#assigns_ok)*
                        }
                        ::std::result::Result::Err(err) => {
                            #(#assigns_err)*
                        }
                    }
                    let _ = _parser(input);
                }
            }
            _ => quote! {},
        };
        tokens.extend(t);
        tokens
    }
}
