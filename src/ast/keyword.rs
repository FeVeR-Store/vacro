use std::collections::HashMap;

use proc_macro2::{Punct, TokenStream};
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use syn::Ident;

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug, PartialEq))]
pub enum Keyword {
    Rust(String),
    Custom {
        punctuation: bool,
        name: Ident,
        content: String,
    },
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
        tokens.extend(match self {
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
        });
    }
}

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub struct KeywordMap(pub HashMap<String, TokenStream>);

impl KeywordMap {
    pub fn new() -> Self {
        KeywordMap(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use proc_macro2::Punct;
    use quote::format_ident;
    use syn::{
        Result, Token,
        parse::{ParseStream, Parser},
    };

    use crate::syntax::{context::ParseContext, keyword};

    use super::*;

    fn parse_keyword<'a>(tokens: TokenStream, ctx: &mut ParseContext) -> Result<Keyword> {
        let parser = move |input: ParseStream| -> Result<Keyword> { Keyword::parse(input, ctx) };
        parser.parse2(tokens)
    }

    #[test]
    fn test_rust_keywords() {
        let ctx = &mut ParseContext::default();
        for tokens in vec![quote! { fn }, quote! { let }, quote! { if }] {
            let keyword: Keyword = parse_keyword(tokens.clone(), ctx).unwrap();
            let keyword_tokens = quote! {#keyword};

            assert_eq!(keyword, Keyword::Rust(tokens.to_string()));
            assert_eq!(
                keyword_tokens.to_string(),
                quote! {::syn::Token![#tokens]}.to_string()
            );
        }
    }
    #[test]
    fn test_custom_keywords() {
        let ctx = &mut ParseContext::default();
        for tokens in vec![quote! { miku }, quote! { teto }, quote! { len }] {
            let keyword: Keyword = parse_keyword(tokens.clone(), ctx).unwrap();
            let keyword_tokens = quote! {#keyword};
            assert_eq!(
                keyword,
                Keyword::Custom {
                    punctuation: false,
                    name: format_ident!("{}", tokens.to_string()),
                    content: tokens.to_string(),
                }
            );
            assert_eq!(keyword_tokens.to_string(), tokens.to_string());
        }
    }
    #[test]
    fn test_rust_punctuation() {
        let ctx = &mut ParseContext::default();
        for tokens in vec![quote! { ! }, quote! { ? }, quote! { . }] {
            let keyword: Keyword = parse_keyword(tokens.clone(), ctx).unwrap();
            let keyword_tokens = quote! {#keyword};
            assert_eq!(keyword, Keyword::Rust(tokens.clone().to_string()));
            assert_eq!(
                keyword_tokens.to_string(),
                quote! {::syn::Token![#tokens]}.to_string()
            );
        }
    }
    #[test]
    fn test_custom_punctuation() {
        let ctx = &mut ParseContext::default();
        for tokens in vec![quote! { <> }, quote! { ?! }, quote! { ~~> }] {
            // 与上面的解析不同，自定义符号的解析需要手动搜集，这里使用了pattern处的代码，但有修改
            // 因为quote会自动分词，'<>' -> '< >'，所以不再检查Spacing
            let parser = |input: ParseStream| -> Result<Keyword> {
                let mut collect = String::new();
                let mut punct: Punct = input.parse()?;
                while !input.is_empty() {
                    if input.peek(Token![#]) {
                        break;
                    }
                    collect.push(punct.as_char());
                    punct = input.parse()?;
                }
                collect.push(punct.as_char());
                Ok(keyword::parse_keyword(collect, ctx))
            };

            let keyword = parser.parse2(tokens.clone()).unwrap();

            let keyword_tokens = quote! {#keyword};

            let name = format_ident!("Punt_{}", (ctx.custom_symbol_counter - 1).to_string());

            assert_eq!(keyword_tokens.to_string(), name.to_string());

            assert_eq!(
                keyword,
                Keyword::Custom {
                    punctuation: true,
                    name,
                    content: tokens.to_string().replace(" ", ""),
                }
            );
        }
    }

    #[test]
    fn test_parse_complex_operators() {
        let ctx = &mut ParseContext::default();
        // 测试 Rust 的多字符运算符，确保它们被识别为单一的 Keyword::Rust
        let ops: Vec<_> = vec!["->", "=>", "::", "..", "..=", "&&", "||", "<<", ">>"];

        for op in ops {
            // 注意：quote! 会自动分词，所以这里直接测试字符串解析逻辑可能更准，
            // 或者构造 TokenStream。这里复用 parse_keyword 函数。
            let kw = keyword::parse_keyword(op, ctx);
            match kw {
                Keyword::Rust(s) => assert_eq!(s, op.to_string()),
                _ => panic!("Operator {} should be parsed as Rust keyword", op),
            }
        }
    }

    #[test]
    fn test_ident_collision() {
        let ctx = &mut ParseContext::default();
        // 测试看似像关键字但实际上是自定义标识符的情况
        let inputs: Vec<_> = vec!["match_", "fn_name", "structA"]
            .iter()
            .map(|op| TokenStream::from_str(op).unwrap())
            .collect();

        for input in inputs {
            let kw = keyword::parse_keyword(input.clone(), ctx);
            match kw {
                Keyword::Custom { content, .. } => assert_eq!(content, input.to_string()),
                Keyword::Rust(_) => {
                    panic!("Identifier {} should NOT be parsed as Rust keyword", input)
                }
            }
        }
    }

    #[test]
    fn test_underscore() {
        let ctx = &mut ParseContext::default();
        let kw = keyword::parse_keyword(TokenStream::from_str("_").unwrap(), ctx);
        assert_eq!(kw, Keyword::Rust("_".to_string()));
    }
}
