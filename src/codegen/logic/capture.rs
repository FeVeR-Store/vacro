use proc_macro2::{Delimiter, TokenStream};
use quote::{format_ident, quote};

use crate::{
    ast::{
        capture::{Binder, Capture, MatcherKind, Quantity},
        node::{Pattern, PatternKind},
    },
    codegen::{logic::Compiler, output::generate_output},
    transform::lookahead::inject_lookahead,
};

impl Compiler {
    pub fn compile_capture(&mut self, capture: &Capture) -> TokenStream {
        let mut tokens = TokenStream::new();
        let Capture {
            binder,
            matcher,
            quantity,
            span,
            ..
        } = capture;
        let receiver = match &binder {
            Binder::Named(ident) => {
                quote! {#ident = }
            }
            Binder::Inline(i) => {
                let id = format_ident!("_{}", i.to_string());
                quote! {#id = }
            }
            _ => quote! {},
        };
        let t = match (binder, quantity, &matcher.kind) {
            (_, Quantity::One, MatcherKind::SynType(ty)) => {
                quote! {
                    {
                        #receiver input.parse::<#ty>()?;
                    }
                }
            }
            (_, Quantity::Optional, MatcherKind::SynType(ty)) => {
                quote! {
                    {
                        let _fork = input.fork();
                        if ::std::result::Ok(_parsed) = _fork.parse::<#ty>() {
                            #receiver ::std::option::Option::Some(_parsed);
                        }
                    }
                }
            }
            (_, Quantity::Many(separator), MatcherKind::SynType(ty)) => {
                quote! {
                    {
                        #[allow(non_local_definitions)]
                        impl _Parse for #ty {}
                        #receiver input.parse_terminated(#ty::parse, #separator)?;
                    }
                }
            }
            (Binder::Anonymous, Quantity::One, MatcherKind::Nested(_patterns)) => {
                let optimized_list = inject_lookahead(_patterns.clone());

                let patterns = Pattern {
                    kind: PatternKind::Group {
                        delimiter: Delimiter::None,
                        children: optimized_list,
                    },
                    span: *span,
                    meta: None,
                };
                let pattern_tokens = self.compile_pattern(&patterns);
                quote! {
                    {
                        #pattern_tokens
                    }
                }
            }
            (Binder::Anonymous, Quantity::Optional, MatcherKind::Nested(_patterns)) => {
                let optimized_list = inject_lookahead(_patterns.clone());

                let patterns = Pattern {
                    kind: PatternKind::Group {
                        delimiter: Delimiter::None,
                        children: optimized_list,
                    },
                    span: *span,
                    meta: None,
                };

                let joint_token = self.compile_pattern(&patterns);
                let captures = patterns.collect_captures();
                let (capture_init, struct_def, struct_expr, fields) =
                    generate_output(&captures, None);

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
                        ::std::result::Result::Err(_) => {
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
