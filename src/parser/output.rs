use std::sync::{Arc, Mutex};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type};

use crate::parser::{capture_group::is_inline, pattern::IsOptional};

pub fn generate_output(
    capture_list: Arc<Mutex<Vec<(Ident, Type, IsOptional)>>>,
    ident: Option<Ident>,
) -> (TokenStream, TokenStream, TokenStream) {
    let ident = ident.unwrap_or_else(|| format_ident!("Output"));
    let is_inline = is_inline();
    let mut capture_init = TokenStream::new();
    let capture_list = capture_list.lock().unwrap();
    capture_init.extend(capture_list.iter().map(|(id, ty, is_optional)| {
        if *is_optional {
            quote! {
                #[allow(unused_mut)]
                let mut #id: #ty = ::std::option::Option::None;
            }
        } else {
            quote! {
                let #id: #ty;
            }
        }
    }));
    let mut struct_fields = TokenStream::new();
    struct_fields.extend(capture_list.iter().map(|(id, ty, ..)| {
        if is_inline {
            quote! {#ty,}
        } else {
            quote! {
                pub #id: #ty,
            }
        }
    }));
    let mut struct_expr_fields = TokenStream::new();
    struct_expr_fields.extend(capture_list.iter().map(|(id, ..)| {
        quote! {
            #id,
        }
    }));

    if is_inline {
        (
            capture_init,
            quote! { type #ident = (#struct_fields); },
            quote! { (#struct_expr_fields) },
        )
    } else {
        (
            capture_init,
            quote! { struct #ident { #struct_fields } },
            quote! { #ident { #struct_expr_fields } },
        )
    }
}
