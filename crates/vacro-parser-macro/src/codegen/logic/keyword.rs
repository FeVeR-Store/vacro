use proc_macro2::TokenStream;

use crate::{ast::keyword::KeywordMap, codegen::logic::Compiler};

impl Compiler {
    pub fn compile_keyword_map(&mut self, map: KeywordMap) -> TokenStream {
        let mut tokens = TokenStream::new();
        map.0.values().for_each(|t| tokens.extend(t.clone()));
        tokens
    }
}
