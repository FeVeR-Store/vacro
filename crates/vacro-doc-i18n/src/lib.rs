use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, AttrStyle, Attribute, Item, Token};

#[proc_macro_attribute]
pub fn doc_i18n(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_ast = parse_macro_input!(item as Item);

    // 确定语言模式
    let mode = lang_mode_from_features();

    // 提取所有文档属性
    let mut attrs = take_attrs_mut(&mut item_ast);
    let (outer_text, inner_text) = extract_doc_text_split(&attrs);

    // 解析并转换
    let new_outer = process_doc_syntax(&outer_text, mode);
    let new_inner = process_doc_syntax(&inner_text, mode);

    // 清理旧属性并写回新属性
    remove_doc_attrs(&mut attrs);

    push_doc_attrs(&mut attrs, &new_outer, AttrStyle::Outer);
    push_doc_attrs(
        &mut attrs,
        &new_inner,
        AttrStyle::Inner(Token![!](Span::call_site())),
    );

    put_attrs_back(&mut item_ast, attrs);
    TokenStream::from(quote!(#item_ast))
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum LangMode {
    Cn,
    En,
    All,
}

fn lang_mode_from_features() -> LangMode {
    // 优先级：Doc-All > 指定 CN/EN
    if cfg!(feature = "doc-all") {
        LangMode::All
    } else if cfg!(feature = "doc-cn") {
        LangMode::Cn
    } else {
        LangMode::En
    }
}

fn process_doc_syntax(input: &str, mode: LangMode) -> String {
    let mut out = String::new();
    let mut current_block_lang: Option<LangMode> = None;

    for line in input.lines() {
        let trimmed = line.trim();

        // 处理块结束标记 `:::`
        if trimmed == ":::" {
            if current_block_lang.is_some() {
                // 如果是 All 模式，我们需要闭合 div，并强制双换行重置 Markdown 上下文
                if mode == LangMode::All {
                    out.push_str("</div>\n\n");
                }
                current_block_lang = None;
            }
            continue;
        }

        // 处理块开始标记 `::: @cn` 或 `::: @en`
        if let Some(lang) = parse_block_start(trimmed) {
            current_block_lang = Some(lang);

            // 如果是 All 模式，写入开标签，并强制换行
            if mode == LangMode::All {
                let class = if lang == LangMode::Cn {
                    "doc-cn"
                } else {
                    "doc-en"
                };
                // style="display: block" 配合 \n 确保后续内容被识别为块级
                out.push_str(&format!(
                    r#"<div class="{}" style="display: block; margin-bottom: 1em;">{}"#,
                    class, "\n"
                ));
            }
            continue;
        }

        // 处理块内内容
        if let Some(block_lang) = current_block_lang {
            if mode == LangMode::All || mode == block_lang {
                out.push_str(line);
                out.push('\n');
            }
            continue;
        }

        // 处理行内/单行标记 `@cn ...` 或 `@en ...`
        if let Some((lang, content)) = parse_inline_start(line) {
            if mode == LangMode::All {
                // All 模式：包裹 span
                let class = if lang == LangMode::Cn {
                    "doc-cn"
                } else {
                    "doc-en"
                };
                out.push_str(&format!(
                    r#"<span class="{}">{}</span>{}"#,
                    class, content, "\n"
                ));
            } else if mode == lang {
                out.push_str(content);
                out.push('\n');
            }
            continue;
        }

        // 公共内容
        out.push_str(line);
        out.push('\n');
    }

    out
}

// 解析 `::: @cn` 返回 Some(LangMode::Cn)
fn parse_block_start(line: &str) -> Option<LangMode> {
    if !line.starts_with(":::") {
        return None;
    }
    let rest = line[3..].trim();
    if rest == "@cn" || rest == "@zh" {
        Some(LangMode::Cn)
    } else if rest == "@en" {
        Some(LangMode::En)
    } else {
        None
    }
}

// 解析 `@cn 这是一段话` -> Some((Cn, "这是一段话"))
fn parse_inline_start(line: &str) -> Option<(LangMode, &str)> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("@cn ") || trimmed.starts_with("@zh ") {
        let content_start = line.find("@").unwrap() + 3;
        // 容错处理：如果后面没空格
        Some((LangMode::Cn, &line[content_start..]))
    } else if trimmed.starts_with("@en ") {
        let content_start = line.find("@").unwrap() + 3;
        Some((LangMode::En, &line[content_start..]))
    } else {
        None
    }
}

fn take_attrs_mut(item: &mut Item) -> Vec<Attribute> {
    match item {
        Item::Const(i) => std::mem::take(&mut i.attrs),
        Item::Enum(i) => std::mem::take(&mut i.attrs),
        Item::Fn(i) => std::mem::take(&mut i.attrs),
        Item::Impl(i) => std::mem::take(&mut i.attrs),
        Item::Mod(i) => std::mem::take(&mut i.attrs),
        Item::Struct(i) => std::mem::take(&mut i.attrs),
        Item::Trait(i) => std::mem::take(&mut i.attrs),
        Item::Type(i) => std::mem::take(&mut i.attrs),
        Item::Use(i) => std::mem::take(&mut i.attrs),
        Item::TraitAlias(i) => std::mem::take(&mut i.attrs),
        Item::ExternCrate(i) => std::mem::take(&mut i.attrs),
        Item::ForeignMod(i) => std::mem::take(&mut i.attrs),
        Item::Union(i) => std::mem::take(&mut i.attrs),
        Item::Static(i) => std::mem::take(&mut i.attrs),
        Item::Macro(i) => std::mem::take(&mut i.attrs),
        _ => vec![],
    }
}

fn put_attrs_back(item: &mut Item, attrs: Vec<Attribute>) {
    match item {
        Item::Const(i) => i.attrs = attrs,
        Item::Enum(i) => i.attrs = attrs,
        Item::Fn(i) => i.attrs = attrs,
        Item::Impl(i) => i.attrs = attrs,
        Item::Mod(i) => i.attrs = attrs,
        Item::Struct(i) => i.attrs = attrs,
        Item::Trait(i) => i.attrs = attrs,
        Item::Type(i) => i.attrs = attrs,
        Item::Use(i) => i.attrs = attrs,
        Item::TraitAlias(i) => i.attrs = attrs,
        Item::ExternCrate(i) => i.attrs = attrs,
        Item::ForeignMod(i) => i.attrs = attrs,
        Item::Union(i) => i.attrs = attrs,
        Item::Static(i) => i.attrs = attrs,
        Item::Macro(i) => i.attrs = attrs,
        _ => {}
    }
}

fn extract_doc_text_split(attrs: &[Attribute]) -> (String, String) {
    let mut outer = String::new();
    let mut inner = String::new();
    for a in attrs {
        if !a.path().is_ident("doc") {
            continue;
        }
        let mut content = String::new();
        if let Ok(s) = a.parse_args::<syn::LitStr>() {
            content = s.value();
        } else if let syn::Meta::NameValue(nv) = &a.meta {
            if let syn::Expr::Lit(expr_lit) = &nv.value {
                if let syn::Lit::Str(litstr) = &expr_lit.lit {
                    content = litstr.value();
                }
            }
        }
        // 保留空行，这对 Markdown 很重要
        match a.style {
            AttrStyle::Outer => {
                outer.push_str(&content);
                outer.push('\n');
            }
            AttrStyle::Inner(_) => {
                inner.push_str(&content);
                inner.push('\n');
            }
        }
    }
    (outer, inner)
}

fn remove_doc_attrs(attrs: &mut Vec<Attribute>) {
    attrs.retain(|a| !a.path().is_ident("doc"));
}

fn push_doc_attrs(attrs: &mut Vec<Attribute>, text: &str, style: AttrStyle) {
    // 允许空行，保留空行
    for line in text.split('\n') {
        let lit = syn::LitStr::new(line, proc_macro2::Span::call_site());
        let attr: Attribute = match style {
            AttrStyle::Outer => syn::parse_quote!(#[doc = #lit]),
            AttrStyle::Inner(_) => syn::parse_quote!(#![doc = #lit]),
        };
        attrs.push(attr);
    }
}
