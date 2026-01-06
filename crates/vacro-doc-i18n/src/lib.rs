use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, AttrStyle, Attribute, Item, Token}; // 引入 AttrStyle

#[proc_macro_attribute]
pub fn doc_i18n(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_ast = parse_macro_input!(item as Item);

    // 1) 取出所有 attributes
    let mut attrs = take_attrs_mut(&mut item_ast);

    // [Change]: 分别提取 Outer (///) 和 Inner (//!) 文档
    let (outer_text, inner_text) = extract_doc_text_split(&attrs);

    // 2) 语言模式
    let mode = lang_mode_from_features();

    // 3) 分别过滤
    let filtered_outer = filter_doc_i18n(&outer_text, mode);
    let filtered_inner = filter_doc_i18n(&inner_text, mode);

    // 4) 删除旧 doc attrs
    remove_doc_attrs(&mut attrs);

    // [Change]: 分别写回。注意 Inner 必须插在 Outer 之后，或者顺序其实不严格，
    // 但为了保持原有语义，我们按照样式分别构造。
    push_doc_attrs(&mut attrs, &filtered_outer, AttrStyle::Outer);
    push_doc_attrs(
        &mut attrs,
        &filtered_inner,
        AttrStyle::Inner(Token![!](Span::call_site())),
    );

    // 5) 放回 attrs
    put_attrs_back(&mut item_ast, attrs);

    TokenStream::from(quote!(#item_ast))
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LangMode {
    En,
    Cn,
    All,
}

fn lang_mode_from_features() -> LangMode {
    if cfg!(feature = "doc-all") {
        LangMode::All
    } else if cfg!(feature = "doc-cn") {
        LangMode::Cn
    } else {
        LangMode::En
    }
}

// ---------------- attrs plumbing ----------------

fn take_attrs_mut(item: &mut Item) -> Vec<Attribute> {
    // 这里无需修改，syn 会自动把 `mod { //! ... }` 里的 inner attr 解析到 Item::Mod 的 attrs 中
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
        Item::Mod(i) => i.attrs = attrs, // 放回时，syn 会自动根据 AttrStyle 决定是打印在外面还是里面
        Item::Struct(i) => i.attrs = attrs,
        Item::Trait(i) => i.attrs = attrs,
        Item::Type(i) => i.attrs = attrs,
        Item::Use(i) => i.attrs = attrs,
        _ => {}
    }
}

/// [Change]: 返回 (OuterDoc, InnerDoc)
fn extract_doc_text_split(attrs: &[Attribute]) -> (String, String) {
    let mut outer = String::new();
    let mut inner = String::new();

    for a in attrs {
        if !a.path().is_ident("doc") {
            continue;
        }

        // 解析文档内容
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

        if content.is_empty() {
            continue;
        } // 忽略空行防止干扰

        // 根据样式追加到不同的 buffer
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

/// [Change]: 增加 style 参数，决定生成 #[doc] 还是 #![doc]
fn push_doc_attrs(attrs: &mut Vec<Attribute>, text: &str, style: AttrStyle) {
    if text.is_empty() {
        return;
    }

    for line in text.split('\n') {
        // 如果是最后一行空字符串，可以跳过，防止多余空行
        // if line.is_empty() { continue; }
        // 但为了保留原格式的空行，我们还是生成空 doc

        let lit = syn::LitStr::new(line, proc_macro2::Span::call_site());

        let attr: Attribute = match style {
            AttrStyle::Outer => syn::parse_quote!(#[doc = #lit]),
            AttrStyle::Inner(_) => syn::parse_quote!(#![doc = #lit]),
        };

        attrs.push(attr);
    }
}

// ---------------- 下面的过滤逻辑保持不变 ----------------

fn filter_doc_i18n(input: &str, mode: LangMode) -> String {
    if matches!(mode, LangMode::All) {
        return input.to_string();
    }

    let mut out = String::new();
    let mut in_block: Option<LangMode> = None;

    for line in input.lines() {
        let trimmed = line.trim();

        if in_block.is_some() && is_block_close(trimmed) {
            in_block = None;
            continue;
        }

        if let Some(block_lang) = in_block {
            if block_lang == mode {
                out.push_str(line);
                out.push('\n');
            }
            continue;
        }

        if trimmed.starts_with("<div") && trimmed.contains("</div>") {
            let replaced = replace_inline(line, mode);
            if !replaced.is_empty() {
                out.push_str(&replaced);
                out.push('\n');
            }
            continue;
        }

        if let Some(lang) = parse_block_open(trimmed) {
            in_block = Some(lang);
            continue;
        }

        let replaced = replace_inline(line, mode);
        out.push_str(&replaced);
        out.push('\n');
    }
    out
}

fn is_block_close(line: &str) -> bool {
    line.trim() == "</div>"
}

fn parse_block_open(line: &str) -> Option<LangMode> {
    let t = line.trim();
    if !(t.starts_with("<div") && t.ends_with('>')) {
        return None;
    }
    parse_lang_from_div_open_tag(t)
}

fn replace_inline(line: &str, mode: LangMode) -> String {
    let mut s = line.to_string();
    let mut cursor = 0usize;

    loop {
        let Some(start) = find_substr(&s, "<div", cursor) else {
            break;
        };
        let Some(gt) = s[start..].find('>').map(|i| start + i) else {
            break;
        };
        let open_tag = &s[start..=gt];
        let Some(lang) = parse_lang_from_div_open_tag(open_tag) else {
            cursor = gt + 1;
            continue;
        };
        let Some(end_rel) = s[gt + 1..].find("</div>").map(|i| gt + 1 + i) else {
            cursor = gt + 1;
            continue;
        };
        let close_end = end_rel + "</div>".len();

        let inner = &s[gt + 1..end_rel];
        let replacement = if lang == mode {
            inner.trim().to_string()
        } else {
            String::new()
        };

        s.replace_range(start..close_end, &replacement);
        cursor = start + replacement.len();
    }
    s.trim_end().to_string()
}

fn find_substr(hay: &str, needle: &str, from: usize) -> Option<usize> {
    hay.get(from..)?.find(needle).map(|i| from + i)
}

fn parse_lang_from_div_open_tag(tag: &str) -> Option<LangMode> {
    let class_val = extract_attr_quoted(tag, "class")?;
    for tok in class_val.split_whitespace() {
        if let Some(lang) = tok.strip_prefix("doc-") {
            return match lang {
                "en" => Some(LangMode::En),
                "cn" | "zh" => Some(LangMode::Cn),
                _ => None,
            };
        }
    }
    None
}

fn extract_attr_quoted(tag: &str, name: &str) -> Option<String> {
    let idx = tag.find(name)?;
    let after = &tag[idx + name.len()..];
    let after = after.trim_start();
    if !after.starts_with('=') {
        return None;
    }
    let after = after[1..].trim_start();
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &after[1..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}
