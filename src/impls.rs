use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::{
    ast::input::{BindInput, DefineInput},
    codegen::logic::Compiler,
};

pub fn bind_impl(input: TokenStream) -> TokenStream {
    let mut compiler = Compiler::new();
    let capture_input = parse_macro_input!(input as BindInput);

    compiler.compile_capture_input(&capture_input).into()
}

pub fn define_impl(input: TokenStream) -> TokenStream {
    let mut compiler = Compiler::new();
    let define_input = parse_macro_input!(input as DefineInput);

    compiler.compile_define_input(&define_input).into()
}
