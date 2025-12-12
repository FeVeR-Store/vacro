use proc_macro2::TokenTree;
use quote::format_ident;

use crate::{ast::keyword::Keyword, syntax::context::ParseContext};

impl Keyword {
    pub fn parse(input: syn::parse::ParseStream, ctx: &mut ParseContext) -> syn::Result<Self> {
        let tt: TokenTree = input.parse()?;
        Ok(parse_keyword(tt, ctx))
    }
}

pub fn parse_keyword(input: impl ToString, ctx: &mut ParseContext) -> Keyword {
    match input.to_string().as_str() {
        keyword @ ("abstract" | "as" | "async" | "auto" | "await" | "become" | "box" | "break"
        | "const" | "continue" | "crate" | "default" | "do" | "dyn" | "else"
        | "enum" | "extern" | "final" | "fn" | "for" | "if" | "impl" | "in" | "let"
        | "loop" | "macro" | "match" | "mod" | "move" | "mut" | "override" | "priv"
        | "pub" | "raw" | "ref" | "return" | "Self" | "self" | "static" | "struct"
        | "super" | "trait" | "try" | "type" | "typeof" | "union" | "unsafe"
        | "unsized" | "use" | "virtual" | "where" | "while" | "yield" | "&" | "&&"
        | "&=" | "@" | "^" | "^=" | ":" | "," | "$" | "." | ".." | "..." | "..="
        | "=" | "==" | "=>" | ">=" | ">" | "<-" | "<=" | "<" | "-" | "-=" | "!="
        | "!" | "|" | "|=" | "||" | "::" | "%" | "%=" | "+" | "+=" | "#" | "?"
        | "->" | ";" | "<<" | "<<=" | ">>" | ">>=" | "/" | "/=" | "*" | "*=" | "~"
        | "_") => Keyword::Rust(keyword.to_string()),
        keyword => {
            let punctuation = !keyword.chars().next().unwrap().is_alphabetic();
            let name = if punctuation {
                let i = ctx.custom_symbol_counter;
                ctx.custom_symbol_counter += 1;
                format_ident!("Punt_{}", i)
            } else {
                format_ident!("{}", keyword)
            };

            Keyword::Custom {
                punctuation,
                name,
                content: keyword.to_string(),
            }
        }
    }
}
