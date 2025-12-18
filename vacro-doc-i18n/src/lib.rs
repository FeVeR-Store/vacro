use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Item};

#[proc_macro_attribute]
pub fn doc_i18n(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_ast = parse_macro_input!(item as Item);

    // 1) 取出所有 doc attrs → 合并成 String
    let mut attrs = take_attrs_mut(&mut item_ast);
    let doc_text = extract_doc_text(&attrs);

    // 2) 语言模式（feature-only）
    let mode = lang_mode_from_features();

    // 3) 过滤/剥离
    let filtered = filter_doc_i18n(&doc_text, mode);

    // 4) 删除旧 doc attrs + 写入新 doc attrs
    remove_doc_attrs(&mut attrs);
    push_doc_attrs(&mut attrs, &filtered);

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
    // 注意：cfg!(feature=...) 在 proc-macro crate 里取的是“proc-macro crate 自己的 feature”
    // 用户在依赖时启用 features，会传递到该 proc-macro crate，因此可用。
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
        _ => {}
    }
}

fn extract_doc_text(attrs: &[Attribute]) -> String {
    // rustdoc 的 doc attrs 是多条 #[doc = "..."]，逐条追加即可
    let mut out = String::new();
    for a in attrs {
        if !a.path().is_ident("doc") {
            continue;
        }
        // #[doc = "..."]
        if let Ok(s) = a.parse_args::<syn::LitStr>() {
            out.push_str(&s.value());
            out.push('\n');
        } else if let syn::Meta::NameValue(nv) = &a.meta {
            if let syn::Expr::Lit(expr_lit) = &nv.value {
                if let syn::Lit::Str(litstr) = &expr_lit.lit {
                    out.push_str(&litstr.value());
                    out.push('\n');
                }
            }
        }
    }
    out
}

fn remove_doc_attrs(attrs: &mut Vec<Attribute>) {
    attrs.retain(|a| !a.path().is_ident("doc"));
}

fn push_doc_attrs(attrs: &mut Vec<Attribute>, text: &str) {
    // 每一行生成一条 #[doc="..."]，空行也保留（保证段落结构）
    for line in text.split('\n') {
        let lit = syn::LitStr::new(line, proc_macro2::Span::call_site());
        attrs.push(syn::parse_quote!(#[doc = #lit]));
    }
}

fn filter_doc_i18n(input: &str, mode: LangMode) -> String {
    // LangMode::All 直接原样返回（先别做 JS 结构优化）
    if matches!(mode, LangMode::All) {
        return input.to_string();
    }

    let mut out = String::new();
    let mut in_block: Option<LangMode> = None;

    for raw_line in input.lines() {
        let line = raw_line;

        // 1) block close
        if in_block.is_some() && is_block_close(line) {
            in_block = None;
            continue;
        }

        // 2) if in block: emit or skip
        if let Some(block_lang) = in_block {
            if block_lang == mode {
                out.push_str(line);
                out.push('\n');
            }
            continue;
        }

        // 3) block open (must be standalone line)
        if let Some(lang) = parse_block_open(line) {
            in_block = Some(lang);
            continue;
        }

        // 4) inline replace (may be multiple per line)
        let replaced = replace_inline(line, mode);
        if !replaced.is_empty() {
            out.push_str(&replaced);
        }
        out.push('\n');
    }

    out
}

fn is_block_close(line: &str) -> bool {
    line.trim() == "</div>"
}

/// 仅当整行（trim 后）是 `<div ...>` 才算 block open
fn parse_block_open(line: &str) -> Option<LangMode> {
    let t = line.trim();
    if !(t.starts_with("<div") && t.ends_with('>')) {
        return None;
    }
    // block 要求独占一行：除了 tag 之外不能有内容
    // 由于 t 已经是整行 trim，这里等价于“整行就是 tag”
    parse_lang_from_div_open_tag(t)
}

/// inline：同一行内 `<div class="doc-xx"> ... </div>`
/// 只保留当前 mode 的内容，其他语言块删除（替换为空）
fn replace_inline(line: &str, mode: LangMode) -> String {
    let mut s = line.to_string();
    let mut cursor = 0usize;

    loop {
        let Some(start) = find_substr(&s, "<div", cursor) else {
            break;
        };

        // 找到 open tag 结束 '>'
        let Some(gt) = s[start..].find('>').map(|i| start + i) else {
            break;
        };
        let open_tag = &s[start..=gt];

        // 从 open_tag 解析语言
        let Some(lang) = parse_lang_from_div_open_tag(open_tag) else {
            cursor = gt + 1;
            continue;
        };

        // 找到同一行内 close
        let Some(end_rel) = s[gt + 1..].find("</div>").map(|i| gt + 1 + i) else {
            // 没有同一行闭合：不按 inline 处理
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

        // cursor 继续往 start+replacement 长度走，避免死循环
        cursor = start + replacement.len();
    }

    s.trim_end().to_string()
}

fn find_substr(hay: &str, needle: &str, from: usize) -> Option<usize> {
    hay.get(from..)?.find(needle).map(|i| from + i)
}

fn parse_lang_from_div_open_tag(tag: &str) -> Option<LangMode> {
    // 只解析 class="..." 或 class='...'
    let class_val = extract_attr_quoted(tag, "class")?;
    // class 可能是 "x doc-cn y"
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
    // 找 `name=`，然后读引号内容
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
