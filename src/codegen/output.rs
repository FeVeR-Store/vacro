use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type};

use crate::ast::capture::{Binder, Capture, FieldDef, MatcherKind, Quantity};

pub fn generate_output(
    capture_list: &[FieldDef],
    ident: Option<Ident>,
) -> (TokenStream, TokenStream, TokenStream, Vec<Ident>) {
    let ident = ident.unwrap_or_else(|| format_ident!("Output"));
    let mut capture_init = TokenStream::new();
    let mut is_inline = false;

    capture_init.extend(capture_list.iter().map(
        |FieldDef {
             name,
             ty,
             is_optional,
             ..
         }| {
            if *is_optional {
                quote! {
                    #[allow(unused_mut)]
                    let mut #name: #ty = ::std::option::Option::None;
                }
            } else {
                quote! {
                    let #name: #ty;
                }
            }
        },
    ));
    let mut struct_fields = TokenStream::new();
    struct_fields.extend(capture_list.iter().map(
        |FieldDef {
             name,
             ty,
             is_inline,
             ..
         }| {
            if *is_inline {
                quote! {#name,}
            } else {
                quote! {#name: #ty}
            }
        },
    ));
    let mut struct_expr_fields = TokenStream::new();
    let capture_ident_list: Vec<Ident> = capture_list
        .iter()
        .map(|FieldDef { name, .. }| name.clone())
        .collect();
    struct_expr_fields.extend(capture_ident_list.iter().map(|ident| {
        quote! {#ident,}
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
