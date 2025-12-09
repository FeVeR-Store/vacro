use std::{
    collections::HashMap,
    sync::atomic::{AtomicI32, Ordering},
};

use proc_macro2::{Punct, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use syn::{Ident, parse::Parse};

#[derive(Debug, Clone)]
pub enum Keyword {
    Rust(String),
    Custom {
        punctuation: bool,
        name: Ident,
        content: String,
    },
}

#[derive(Debug, Clone)]
pub struct KeywordMap(HashMap<String, TokenStream>);

impl KeywordMap {
    pub fn new() -> Self {
        KeywordMap(HashMap::new())
    }
}

impl ToTokens for KeywordMap {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.values().for_each(|t| tokens.extend(t.clone()));
    }
}

impl Keyword {
    pub fn get_definition(&self) -> TokenStream {
        match self {
            Keyword::Custom {
                punctuation,
                name,
                content,
            } => {
                if !*punctuation {
                    quote! {
                        ::syn::custom_keyword!(#self);
                    }
                } else {
                    let mut tokens = TokenStream::new();
                    for char in content.chars() {
                        let token = Punct::new(char, proc_macro2::Spacing::Joint);
                        tokens.append(token);
                    }
                    quote! {
                        ::syn::custom_punctuation!(#name, #tokens);
                    }
                }
            }
            Keyword::Rust(_) => {
                quote! {}
            }
        }
    }
    pub fn define(&self, map: &mut KeywordMap) {
        match self {
            Keyword::Custom { content, .. } => {
                map.0.insert(content.to_string(), self.get_definition());
            }
            _ => (),
        }
    }
}

impl ToTokens for Keyword {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t: TokenStream = match self {
            Keyword::Custom { name, .. } => {
                let keyword = format_ident!("{}", name);
                quote!(#keyword)
            }
            Keyword::Rust(keyword) => match keyword.as_str() {
                "&" => {
                    quote!(::syn::Token![&])
                }
                "&&" => {
                    quote!(::syn::Token![&&])
                }
                "&=" => {
                    quote!(::syn::Token![&=])
                }
                "@" => {
                    quote!(::syn::Token![@])
                }
                "^" => {
                    quote!(::syn::Token![^])
                }
                "^=" => {
                    quote!(::syn::Token![^=])
                }
                ":" => {
                    quote!(::syn::Token![:])
                }
                "," => {
                    quote!(::syn::Token![,])
                }
                "$" => {
                    quote!(::syn::Token![$])
                }
                "." => {
                    quote!(::syn::Token![.])
                }
                ".." => {
                    quote!(::syn::Token![..])
                }
                "..." => {
                    quote!(::syn::Token![...])
                }
                "..=" => {
                    quote!(::syn::Token![..=])
                }
                "=" => {
                    quote!(::syn::Token![=])
                }
                "==" => {
                    quote!(::syn::Token![==])
                }
                "=>" => {
                    quote!(::syn::Token![=>])
                }
                ">=" => {
                    quote!(::syn::Token![>=])
                }
                ">" => {
                    quote!(::syn::Token![>])
                }
                "<-" => {
                    quote!(::syn::Token![<-])
                }
                "<=" => {
                    quote!(::syn::Token![<=])
                }
                "<" => {
                    quote!(::syn::Token![<])
                }
                "-" => {
                    quote!(::syn::Token![-])
                }
                "-=" => {
                    quote!(::syn::Token![-=])
                }
                "!=" => {
                    quote!(::syn::Token![!=])
                }
                "!" => {
                    quote!(::syn::Token![!])
                }
                "|" => {
                    quote!(::syn::Token![|])
                }
                "|=" => {
                    quote!(::syn::Token![|=])
                }
                "||" => {
                    quote!(::syn::Token![||])
                }
                "::" => {
                    quote!(::syn::Token![::])
                }
                "%" => {
                    quote!(::syn::Token![%])
                }
                "%=" => {
                    quote!(::syn::Token![%=])
                }
                "+" => {
                    quote!(::syn::Token![+])
                }
                "+=" => {
                    quote!(::syn::Token![+=])
                }
                "#" => {
                    quote!(::syn::Token![#])
                }
                "?" => {
                    quote!(::syn::Token![?])
                }
                "->" => {
                    quote!(::syn::Token![->])
                }
                ";" => {
                    quote!(::syn::Token![;])
                }
                "<<" => {
                    quote!(::syn::Token![<<])
                }
                "<<=" => {
                    quote!(::syn::Token![<<=])
                }
                ">>" => {
                    quote!(::syn::Token![>>])
                }
                ">>=" => {
                    quote!(::syn::Token![>>=])
                }
                "/" => {
                    quote!(::syn::Token![/])
                }
                "/=" => {
                    quote!(::syn::Token![/=])
                }
                "*" => {
                    quote!(::syn::Token![*])
                }
                "*=" => {
                    quote!(::syn::Token![*=])
                }
                "~" => {
                    quote!(::syn::Token![~])
                }
                "_" => {
                    quote!(::syn::Token![_])
                }
                _ => {
                    let keyword = format_ident!("{}", keyword);
                    quote! {
                        ::syn::Token![#keyword]
                    }
                }
            },
        };
        tokens.extend(t);
    }
}

impl Parse for Keyword {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tt: TokenTree = input.parse()?;
        Ok(parse_keyword(tt))
    }
}

static ITER: AtomicI32 = AtomicI32::new(0);

pub fn parse_keyword(input: impl ToString) -> Keyword {
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
                let i = ITER.load(Ordering::Relaxed).to_string();
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

#[cfg(test)]
mod tests {
    use proc_macro2::Punct;
    use syn::{
        Result, Token,
        parse::{ParseStream, Parser},
        parse2,
    };

    use super::*;
    #[test]
    fn test_rust_keywords() {
        for _tokens in vec![quote! { fn }, quote! { let }, quote! { if }] {
            let keyword: Keyword = parse2(_tokens.clone()).unwrap();
            let _k = Keyword::Rust(_tokens.clone().to_string());
            let keyword_tokens = quote! {#keyword};
            assert_eq!(matches!(keyword, _k), true);
            assert_eq!(matches!(keyword_tokens, _tokens), true);
        }
    }
    #[test]
    fn test_custom_keywords() {
        for _tokens in vec![quote! { miku }, quote! { teto }, quote! { len }] {
            let keyword: Keyword = parse2(_tokens.clone()).unwrap();
            let _k = Keyword::Custom {
                punctuation: true,
                name: format_ident!("{}", _tokens.to_string()),
                content: _tokens.to_string(),
            };
            let keyword_tokens = quote! {#keyword};
            assert_eq!(matches!(keyword, _k), true);
            assert_eq!(matches!(keyword_tokens, _tokens), true);
        }
    }
    #[test]
    fn test_rust_punctuation() {
        for _tokens in vec![quote! { ! }, quote! { ? }, quote! { . }] {
            let keyword: Keyword = parse2(_tokens.clone()).unwrap();
            let _k = Keyword::Rust(_tokens.clone().to_string());
            let keyword_tokens = quote! {#keyword};
            assert_eq!(matches!(keyword, _k), true);
            assert_eq!(matches!(keyword_tokens, _tokens), true);
        }
    }
    #[test]
    fn test_custom_punctuation() {
        for _tokens in vec![quote! { <> }, quote! { ?! }, quote! { ~~> }] {
            // 与上面的解析不同，自定义符号的解析需要手动搜集，这里使用了pattern处的代码，但有修改
            // 因为quote会自动分词，'<>' -> '< >'，所以不再检查Spacing
            let parser = |input: ParseStream| -> Result<Keyword> {
                let mut collect = String::new();
                let mut punct: Punct = input.parse().expect("heeee");
                while !input.is_empty() {
                    println!("{}", punct.to_string());
                    if input.peek(Token![#]) {
                        break;
                    }
                    collect.push(punct.as_char());
                    println!("{}", input.to_string());
                    punct = input.parse().expect("hiiiii");
                }
                collect.push(punct.as_char());
                Ok(parse_keyword(collect))
            };

            let keyword = parser.parse2(_tokens.clone()).unwrap();

            let _k = Keyword::Custom {
                punctuation: true,
                name: format_ident!("Punt_{}", ITER.load(Ordering::Relaxed).to_string()),
                content: _tokens.to_string(),
            };
            let keyword_tokens = quote! {#keyword};
            assert_eq!(matches!(keyword, _k), true);
            assert_eq!(matches!(keyword_tokens, _tokens), true);
        }
    }
}
