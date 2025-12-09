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

use proc_macro::TokenStream;
mod impls;
pub(crate) mod parser;

use crate::impls::{capture_impl, define_impl};

#[proc_macro]
pub fn capture(input: TokenStream) -> TokenStream {
    capture_impl(input)
}

#[proc_macro]
pub fn define(input: TokenStream) -> TokenStream {
    define_impl(input)
}
