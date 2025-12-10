use std::collections::HashMap;

use proc_macro2::TokenStream;
use syn::Ident;

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub enum Keyword {
    Rust(String),
    Custom {
        punctuation: bool,
        name: Ident,
        content: String,
    },
}

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub struct KeywordMap(pub HashMap<String, TokenStream>);

impl KeywordMap {
    pub fn new() -> Self {
        KeywordMap(HashMap::new())
    }
}
