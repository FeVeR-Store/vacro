# Vacro Report

**为过程宏提供更优质的 Panic 错误报告**

## 简介

`vacro-report` 是一个专为 Rust 过程宏（Procedural Macros）设计的诊断增强工具。

目前它主要用于解决 `syn::parse_quote!` 或 `parse_quote_spanned` 产生的 Panic 信息晦涩难懂的问题。当 `parse_quote!` 或 `parse_quote_spanned` 解析失败时，通常只会抛出一个简单的 "failed to parse"。`vacro-report` 能够拦截这些调用，并在失败时打印出实际生成的已格式化的 Token 流，并高亮显示语法错误的位置。

## 使用方法

只需在你的函数上添加 `#[vacro_report::scope]` 属性宏。

在该作用域内，所有的 `parse_quote!` 和 `parse_quote_spanned` 调用都会被自动增强。无需修改内部代码。

```rust
# use syn::{parse_quote, Expr, Token};
# use vacro_report::scope;
#[scope]
fn generate_code() {
    // 如果此处失败，vacro-report 会打印出导致错误的具体 Token 代码
    let _expr: Expr = parse_quote! {
        1 + 2 + // 缺少操作数，通常会产生难以调试的 Panic
    };
}
```

## 特性

- **自动拦截**：自动重写 `#[scope]` 函数内部的 `parse_quote!` 与 `parse_quote_spanned!` 的调用。
- **详细诊断**：发生解析错误时，展示格式化后的生成代码及错误位置。
- **零开销（成功路径）**：仅在发生错误（Panic 路径）时才会有额外开销，在生产环境时，只有您写的源代码会被包含。

<div class="warning">   
我们通过 `debug_assertions` 进行判断，意味着，如果您启用了某些优化，可能会导致效果无法触发。
</div>

## `help!` 宏（v0.1.3）

此宏定义一个新的结构体（包装类型），它代理了底层解析类型（如 `syn::Ident` 或 `syn::Expr`）的行为，并允许你附加上下文相关的错误信息、帮助文本以及示例代码。

## 语法

```rust,ignore
help!(NewTypeName: BaseType {
    error: "简短的错误消息",
    help: "更详细的帮助文本/建议",
});

```

## 基础用法

```rust
# use vacro::help;
use syn::Ident;

help!(MyIdent: Ident {
    error: "expected a valid identifier",
    help: "identifiers must start with a letter or underscore"
});

```

## 为 [vacro-parser](https://www.google.com/url?sa=E&source=gmail&q=https://docs.rs/vacro-parser/latest/vacro_parser) 提供支持

当同时启用 `vacro` 的 `parser` 与 `report` 特性，或独立安装两个 crate 且启用 `vacro-report` 的 `parser` feature 时，你可以使用 `example` 字段为 `vacro-parser` 提供更多支持，以辅助其提供更详尽的帮助信息。

```rust
# use vacro::help;
use syn::Expr;

help!(Arithmetic: Expr {
    error: "expected an arithmetic expression",
    help: "try using explicit values like 1, 2 or operations like 1 + 2",
    // 这里的 example 会用于辅助 vacro-parser
    example: "1 + 2"
});

```

## 未来计划

- 支持更多的 `syn` 宏以及自定义解析逻辑的诊断增强。
