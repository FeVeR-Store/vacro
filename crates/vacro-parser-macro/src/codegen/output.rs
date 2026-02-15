use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Visibility};

use crate::ast::capture::{ExampleItem, FieldDef};

type CaptureInit = TokenStream;
type StructDef = TokenStream;
type StructExpr = TokenStream;
type CaptureList = Vec<Ident>;

pub fn generate_output(
    capture_list: &[FieldDef],
    ident: Option<Ident>,
    visibility: Option<Visibility>,
) -> (CaptureInit, StructDef, StructExpr, CaptureList) {
    let ident = ident.unwrap_or_else(|| format_ident!("Output"));
    let visibility = visibility.unwrap_or(Visibility::Inherited);
    let mut capture_init = TokenStream::new();

    let is_inline = capture_list.first().map(|f| f.is_inline).unwrap_or(false);

    capture_init.extend(capture_list.iter().map(
        |FieldDef {
             name,
             ty,
             is_optional,
             ..
         }| {
            if *is_optional {
                quote! {
                    #[allow(unused)]
                    let mut #name: #ty = ::std::option::Option::None;
                }
            } else {
                quote! {
                    let #name: #ty;
                }
            }
        },
    ));

    let mut struct_fields = TokenStream::new();
    struct_fields.extend(capture_list.iter().map(
        |FieldDef {
             name,
             ty,
             is_inline,
             ..
         }| {
            if *is_inline {
                quote! { #ty, }
            } else {
                quote! { pub #name: #ty,}
            }
        },
    ));

    let mut struct_expr_fields = TokenStream::new();
    let capture_ident_list: Vec<Ident> = capture_list
        .iter()
        .map(|FieldDef { name, .. }| name.clone())
        .collect();
    struct_expr_fields.extend(capture_ident_list.iter().map(|ident| {
        quote! {#ident,}
    }));

    if is_inline {
        (
            capture_init,
            quote! { #visibility type #ident = (#struct_fields); },
            quote! { (#struct_expr_fields) },
            capture_ident_list,
        )
    } else {
        (
            capture_init,
            quote! { #visibility struct #ident { #struct_fields } },
            quote! { #ident { #struct_expr_fields } },
            capture_ident_list,
        )
    }
}

pub fn generate_example(
    example_items: &[ExampleItem],
    is_block: bool,
    is_extra: bool,
    is_inline: bool,
) -> (String, Vec<String>) {
    let mut example = String::new();
    let mut extra_example = vec![];
    let last_index = example_items.len() - 1;
    for (i, item) in example_items.iter().enumerate() {
        example += &match item {
            ExampleItem::Literal(lit) => {
                if is_extra {
                    format!("{} ", lit)
                } else {
                    format!("**{}** ", lit)
                }
            }
            ExampleItem::Block {
                optional,
                example,
                iter,
            } => {
                let (inner, mut extra) = generate_example(example, true, is_extra, false);
                extra_example.append(&mut extra);
                let suffix = format!(
                    "{}{}",
                    if iter.is_empty() {
                        String::new()
                    } else {
                        format!("{} ...", iter)
                    },
                    if *optional { "?" } else { "" }
                );
                if is_extra {
                    format!("{}\n\n{}{}\n\n{}", "```", inner, suffix, "```")
                } else if is_block {
                    format!(" {}{} ", inner, suffix)
                } else {
                    format!("*[{}{}]()* ", inner, suffix)
                }
            }
            ExampleItem::Capture { name, ty } => {
                let ty = ty.replace(" ", ""); // 间隔优化
                if name.is_empty() {
                    if is_extra {
                        ty.to_string()
                    } else {
                        format!("*{}* ", ty)
                    }
                } else if is_extra {
                    if is_inline {
                        format!("{}@{},", name, ty)
                    } else {
                        format!("    {}@{},", name, ty)
                    }
                } else if is_block {
                    format!("*{}@`{}`* ", name, ty)
                } else {
                    format!("*[{}@`{}`]()* ", name, ty)
                }
            }
            ExampleItem::Poly {
                name,
                syntex_name,
                example,
            } => {
                let (inner, mut extra) = generate_example(example, is_block, true, false);
                extra_example.push(format!("```\n{} {{\n{}\n}}\n```\n", syntex_name, inner));

                extra_example.append(&mut extra);
                if is_extra {
                    syntex_name.to_string()
                } else if name.is_empty() {
                    format!("*{}* ", syntex_name)
                } else {
                    format!("*{}@`{}`* ", name, syntex_name)
                }
            }
            ExampleItem::Group { delimiter, example } => {
                let (inner, mut extra) = generate_example(example, is_block, is_extra, is_extra);
                extra_example.append(&mut extra);
                if is_extra {
                    if delimiter.0.is_empty() {
                        if is_inline {
                            inner
                        } else {
                            format!("    {inner}")
                        }
                    } else if is_inline {
                        format!("{}{}{}", delimiter.0, inner, delimiter.1)
                    } else {
                        format!("{}\n    {}\n{}", delimiter.0, inner, delimiter.1)
                    }
                } else if delimiter.0.is_empty() {
                    format!("{} ", inner)
                } else {
                    format!(
                        "**{}**\n\n&nbsp;&nbsp;&nbsp;&nbsp;{}\n\n**{}** ",
                        delimiter.0, inner, delimiter.1
                    )
                }
            }
        };
        example += if !is_inline && is_extra && last_index != i {
            "\n"
        } else {
            ""
        };
    }
    (example, extra_example)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use syn::{parse_quote, token::Pub};

    // 辅助函数：快速创建 FieldDef
    fn field(name: &str, ty: &str, is_optional: bool, is_inline: bool) -> FieldDef {
        FieldDef {
            name: syn::parse_str(name).unwrap(),
            ty: syn::parse_str(ty).unwrap(),
            is_optional,
            is_inline,
        }
    }

    #[test]
    fn test_generate_struct_fields_comma() {
        // 测试场景：标准具名捕获 (MyFn { name: Ident, ret: Type })
        // 目标：验证生成的字段之间是否有逗号
        let fields = vec![
            field("name", "Ident", false, false),
            field("ret", "Type", false, false),
        ];

        let (_, struct_def, _, _) = generate_output(&fields, Some(parse_quote!(MyStruct)), None);
        let output = struct_def.to_string();

        // 验证生成的代码结构
        // 我们期望看到: struct MyStruct { name : Ident , ret : Type , }
        assert!(output.contains("struct MyStruct"));
        assert!(output.contains("name : Ident ,")); // 关键点：要有逗号
        assert!(output.contains("ret : Type ,")); // 关键点：要有逗号

        let (_, struct_def, _, _) = generate_output(
            &fields,
            Some(parse_quote!(MyStruct)),
            Some(Visibility::Public(Pub {
                span: Span::call_site(),
            })),
        );
        let output = struct_def.to_string();

        assert!(output.contains("pub struct MyStruct"));

        let (_, struct_def, _, _) = generate_output(
            &fields,
            Some(parse_quote!(MyStruct)),
            Some(parse_quote! {pub(in a::b::c)}),
        );
        let output = struct_def.to_string();

        assert!(output.contains("pub (in a :: b :: c) struct MyStruct"));
    }

    #[test]
    fn test_generate_inline_tuple_type() {
        // 测试场景：行内捕获 (type Output = (Ident, Type);)
        // 目标：验证生成的是 Type 列表而不是变量名列表，且格式正确
        let fields = vec![
            field("_0", "Ident", false, true),
            field("_1", "Type", false, true),
        ];

        let (_, struct_def, _, _) = generate_output(&fields, Some(parse_quote!(MyTuple)), None);
        let output = struct_def.to_string();

        // 验证生成的代码结构
        // 我们期望看到: type MyTuple = (Ident , Type ,);
        assert!(output.contains("type MyTuple ="));
        assert!(output.contains("(Ident , Type ,)"));

        // 确保没有生成变量名 (比如 _0, _1)
        assert!(!output.contains("_0"));
    }

    #[test]
    fn test_generate_optional_fields() {
        // 测试场景：可选字段初始化逻辑
        let fields = vec![
            field("opt_val", "u32", true, false), // is_optional = true
        ];

        let (capture_init, _, _, _) = generate_output(&fields, None, None);
        let init_code = capture_init.to_string();

        // 验证初始化逻辑是否包含 Option::None
        assert!(init_code.contains("let mut opt_val : u32 ="));
        assert!(init_code.contains("Option :: None"));
    }

    #[test]
    fn test_generate_empty_capture() {
        let fields = vec![];
        let (_, struct_def, struct_expr, _) =
            generate_output(&fields, Some(parse_quote!(MyEmpty)), None);

        let def_str = struct_def.to_string();
        let expr_str = struct_expr.to_string();

        // 期望: struct MyEmpty {} 或 struct MyEmpty { }
        assert!(def_str.contains("struct MyEmpty"));
        // 确保没有多余的逗号
        assert!(!def_str.contains(", }"));

        // 期望: MyEmpty {}
        assert!(expr_str.contains("MyEmpty"));
    }
}
