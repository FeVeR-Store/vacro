use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::parser::input::{CaptureInput, DefineInput};
pub fn capture_impl(input: TokenStream) -> TokenStream {
    let capture_input = parse_macro_input!(input as CaptureInput);

    quote!(#capture_input).into()
}

pub fn define_impl(input: TokenStream) -> TokenStream {
    let define_input = parse_macro_input!(input as DefineInput);

    quote!(#define_input).into()
}
