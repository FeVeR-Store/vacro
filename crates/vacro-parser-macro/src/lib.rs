use proc_macro::TokenStream;

use crate::impls::{bind_impl, define_impl};

pub(crate) mod ast;
pub(crate) mod codegen;
mod impls;
pub(crate) mod syntax;
pub(crate) mod transform;

#[proc_macro]
pub fn bind(input: TokenStream) -> TokenStream {
    bind_impl(input)
}

#[proc_macro]
pub fn define(input: TokenStream) -> TokenStream {
    define_impl(input)
}
