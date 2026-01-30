# Vacro Report

**为过程宏提供更优质的 Panic 错误报告**

[![crates.io](https://img.shields.io/crates/v/vacro-report.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/vacro-report)
[![docs.rs](https://img.shields.io/badge/docs.rs-vacro--report-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs)](https://docs.rs/vacro-report)

## 简介

`vacro-report` 是一个专为 Rust 过程宏（Procedural Macros）设计的诊断增强工具。

目前它主要用于解决 `syn::parse_quote!` 或 `parse_quote_spanned` 产生的 Panic 信息晦涩难懂的问题。当 `parse_quote!` 或 `parse_quote_spanned` 解析失败时，通常只会抛出一个简单的 "failed to parse"。`vacro-report` 能够拦截这些调用，并在失败时打印出实际生成的已格式化的 Token 流，并高亮显示语法错误的位置。

## 安装

```toml
[dependencies]
vacro-report = "0.1.2"
```

## 使用方法

只需在你的函数上添加 `#[vacro_report::scope]` 属性宏。

在该作用域内，所有的 `parse_quote!` 和 `parse_quote_spanned` 调用都会被自动增强。无需修改内部代码。

```rust
use syn::{parse_quote, Expr, Token};
use vacro_report::scope;

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
  > ⚠️ Warning
  >
  > 我们通过 `debug_assertions` 进行判断，意味着，如果您启用了某些优化，可能会导致效果无法触发
