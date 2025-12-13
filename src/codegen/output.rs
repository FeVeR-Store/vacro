use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::ast::capture::{Binder, Capture, MatcherKind, Quantity};

pub fn generate_output(
    capture_list: &[&Capture],
    ident: Option<Ident>,
) -> (TokenStream, TokenStream, TokenStream, Vec<Ident>) {
    let ident = ident.unwrap_or_else(|| format_ident!("Output"));
    let mut capture_init = TokenStream::new();
    let mut is_inline = false;

    capture_init.extend(capture_list.iter().map(
        |Capture {
             binder,
             quantity,
             matcher,
             ..
         }| {
            match (binder, quantity, &matcher.kind) {
                (
                    binder @ (Binder::Named(_) | Binder::Inline(_)),
                    quantity,
                    MatcherKind::SynType(ty),
                ) => {
                    let id = match binder {
                        Binder::Inline(i) => {
                            is_inline = true;
                            format_ident!("_{}", i.to_string())
                        }
                        Binder::Named(id) => id.clone(),
                        Binder::Anonymous => panic!(),
                    };
                    match quantity {
                        Quantity::Optional => quote! {
                            #[allow(unused_mut)]
                            let mut #id: #ty = ::std::option::Option::None;
                        },
                        Quantity::One => quote! {
                            let #id: #ty;
                        },
                        _ => quote! {},
                    }
                }
                _ => quote! {},
            }
        },
    ));
    let mut struct_fields = TokenStream::new();
    struct_fields.extend(capture_list.iter().map(
        |Capture {
             binder, matcher, ..
         }| {
            match (binder, &matcher.kind) {
                (binder @ (Binder::Named(_) | Binder::Inline(_)), MatcherKind::SynType(ty)) => {
                    match binder {
                        Binder::Inline(i) => {
                            let id = format_ident!("_{}", i.to_string());
                            quote! {#id,}
                        }
                        Binder::Named(id) => {
                            quote! {#id: #ty}
                        }
                        Binder::Anonymous => panic!(),
                    }
                }
                _ => quote! {},
            }
        },
    ));
    let mut struct_expr_fields = TokenStream::new();
    let capture_ident_list: Vec<Ident> = capture_list
        .iter()
        .map(|Capture { binder, .. }| match binder {
            Binder::Inline(i) => {
                format_ident!("_{}", i.to_string())
            }
            Binder::Named(id) => id.clone(),
            Binder::Anonymous => panic!(),
        })
        .collect();
    struct_expr_fields.extend(capture_ident_list.iter().map(|ident| {
        quote! {#ident}
    }));

    if is_inline {
        (
            capture_init,
            quote! { type #ident = (#struct_fields); },
            quote! { (#struct_expr_fields) },
            capture_ident_list,
        )
    } else {
        (
            capture_init,
            quote! { struct #ident { #struct_fields } },
            quote! { #ident { #struct_expr_fields } },
            capture_ident_list,
        )
    }
}
