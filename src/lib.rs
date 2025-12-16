#![warn(missing_docs)]

//!<div class="doc-cn">
//!
#![doc = include_str!("docs/zh_cn.md")]
//!
//!</div>
//!
//! <div class="doc-en">
//!
#![doc = include_str!("../readme.md")]
//!
//!</div>

pub(crate) mod ast;
pub(crate) mod codegen;
mod impls;
pub(crate) mod syntax;
pub(crate) mod transform;

use proc_macro::TokenStream;

use crate::impls::{bind_impl, define_impl};

#[proc_macro]
pub fn bind(input: TokenStream) -> TokenStream {
    bind_impl(input)
}

#[proc_macro]
pub fn define(input: TokenStream) -> TokenStream {
    define_impl(input)
}
