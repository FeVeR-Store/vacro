# Vacro Report

**为过程宏提供更优质的 Panic 错误报告**

## 简介

`vacro-report` 是一个专为 Rust 过程宏（Procedural Macros）设计的诊断增强工具。

目前它主要用于解决 `syn::parse_quote!` 产生的 Panic 信息晦涩难懂的问题。当 `parse_quote!` 解析失败时，通常只会抛出一个简单的 "failed to parse"。`vacro-report` 能够拦截这些调用，并在失败时打印出实际生成的 Token 流（已格式化），并高亮显示语法错误的位置。

## 使用方法

只需在你的函数上添加 `#[vacro_report::scope]` 属性宏。

在该作用域内，所有的 `parse_quote!` 调用都会被自动增强。无需修改内部代码。

```rust,ignore
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

* **自动拦截**：自动重写 `#[scope]` 函数内部的 `parse_quote!` 调用。
* **详细诊断**：发生解析错误时，展示格式化后的生成代码及错误位置。
* **零开销（成功路径）**：仅在发生错误（Panic 路径）时才会有额外开销。

## 未来计划

* 支持更多的 `syn` 宏以及自定义解析逻辑的诊断增强。
