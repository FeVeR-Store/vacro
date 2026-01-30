use proc_macro2::{Punct, TokenStream, TokenTree};
use quote::TokenStreamExt;
use syn::{
    parenthesized,
    parse::{discouraged::Speculative, Parse, Parser},
    Local, Stmt, Token,
};

use crate::ast::{
    input::{BindInput, DefineInput},
    node::Pattern,
};

impl Parse for BindInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        let mut tokens = TokenStream::new();
        if !fork.peek(Token![let]) {
            return Err(syn::Error::new(fork.span(), "expected `let`"));
        }
        while !fork.peek(Token![=]) && !fork.is_empty() {
            tokens.append(fork.parse::<TokenTree>()?);
        }
        tokens.append(Punct::new(';', proc_macro2::Spacing::Alone));

        input.advance_to(&fork);
        input.parse::<Token![=]>()?;
        let parser = |input: syn::parse::ParseStream| -> syn::Result<Local> {
            let stmt: Stmt = input.parse()?;
            let Stmt::Local(local) = stmt else {
                return Err(syn::Error::new(
                    input.span(),
                    "Expected a local variable declaration",
                ));
            };
            Ok(local)
        };
        let local = parser.parse2(tokens)?;
        let capture_group;
        let _paren = parenthesized!(capture_group in input);

        let ident = capture_group.parse()?;
        let _arrow = capture_group.parse()?;
        let patterns = Pattern::parse(&capture_group)?;

        let suffix: TokenStream = input.parse()?;
        Ok(BindInput {
            local,
            input: ident,
            _arrow,
            patterns,
            suffix,
        })
    }
}

impl Parse for DefineInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let _colon = input.parse()?;
        let patterns = Pattern::parse(input)?;
        Ok(DefineInput {
            name,
            _colon,
            patterns,
        })
    }
}
