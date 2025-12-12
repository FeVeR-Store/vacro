use syn::parse::Parse;

use crate::ast::{
    input::{CaptureInput, DefineInput},
    pattern::PatternList,
};

impl Parse for CaptureInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let _arrow = input.parse()?;
        let patterns = PatternList::parse(input)?;
        Ok(CaptureInput {
            input: ident,
            _arrow,
            patterns,
        })
    }
}

impl Parse for DefineInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let _colon = input.parse()?;
        let patterns = PatternList::parse(input)?;
        Ok(DefineInput {
            name,
            _colon,
            patterns,
        })
    }
}
