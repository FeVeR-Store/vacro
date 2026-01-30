use quote::{format_ident, ToTokens};
use syn::{parse_quote, Ident, Item, Pat};

use crate::codegen::logic::Compiler;

impl Compiler {
    pub fn define_invisible_item(&mut self, item: Item) {
        let mod_ident = self.get_private_scope_ident();
        if let Some(mod_item) = self.shared_definition.get_mut(0) {
            if let Item::Mod(m) = mod_item {
                if let Some((_, items)) = m.content.as_mut() {
                    items.push(item)
                }
            }
        } else {
            let mod_definition: Item = parse_quote! {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                pub mod #mod_ident {
                    use super::*;
                    #item
                }
            };
            self.scoped_definition.push(parse_quote! {
                use #mod_ident::*;
            });
            self.shared_definition.insert(0, mod_definition);
        }
    }
    pub fn get_private_scope_ident(&self) -> Ident {
        format_ident!("__private_scope_for_{}", self.target)
    }
    pub fn pat_to_ident(pat: &Pat) -> Ident {
        // 1. 将 Pattern 转为 TokenStream 字符串
        let raw_string = pat.to_token_stream().to_string();

        // 2. 处理字符
        let mut sanitized = String::with_capacity(raw_string.len());
        let mut last_was_underscore = false;

        for c in raw_string.chars() {
            if c.is_alphanumeric() {
                sanitized.push(c.to_ascii_lowercase());
                last_was_underscore = false;
            } else {
                // 只有当上一个字符不是下划线时，才添加下划线（去重）
                if !last_was_underscore {
                    sanitized.push('_');
                    last_was_underscore = true;
                }
            }
        }

        // 3. 去除首尾的下划线
        let final_str = sanitized.trim_matches('_');

        // 4. 处理特殊情况：如果结果为空（例如 pattern 只有符号），默认为 "_"
        let valid_name = if final_str.is_empty() { "_" } else { final_str };

        // 5. 创建 Ident，保留原始 Pattern 的 Span 以便于错误定位
        format_ident!("{valid_name}")
    }
}
