# Vacro

<div align="center">

**Rust 过程宏的渐进式 DevX (开发体验) 框架**

[<img alt="github" src="https://img.shields.io/badge/github-FeVeR_Store/vacro-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/FeVeR-Store/vacro)
[<img alt="crates.io" src="https://img.shields.io/crates/v/vacro.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/vacro)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-vacro-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/vacro)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/FeVeR-Store/vacro/publish.yml?style=for-the-badge" height="20">](https://github.com/FeVeR-Store/vacro/actions/workflows/publish.yml)

[English](./README.md) | [简体中文](./README_CN.md)

</div>

---

## 设计理念

编写 Rust 过程宏不应该是一场充满样板代码和黑盒调试的噩梦。
**Vacro** 从一个单纯的解析库进化为一套完整的工具链，旨在提升宏开发全生命周期的体验：

1.  **解析 (Parsing)**：使用声明式方式编写解析代码。
2.  **调试 (Debugging)**：可视化解析路径，看清宏内部发生了什么。
3.  **诊断 (Reporting)**：轻松生成优雅、精准的编译器错误信息。

## 生态系统

Vacro 采用模块化设计。你可以直接使用功能齐全的 `vacro` 入口，也可以按需单独使用底层组件。

| 功能     | 组件 (Crate)     | 描述                                                                 |
| :------- | :--------------- | :------------------------------------------------------------------- |
| **解析** | `vacro-parser`   | **声明式解析。** 类似 `macro_rules!` 的 DSL，自动实现 `syn::Parse`。 |
| **调试** | `vacro-trace`    | **可视化追踪。** 在终端生成解析状态树，专治复杂文法调试难题。        |
| **诊断** | `vacro-report`   | **错误报告。** 简化过程宏中的诊断信息构建与发射。                    |
| **文档** | `vacro-doc-i18n` | **国际化文档。** 为`Vacro`提供多语言文档。                           |

## 快速开始

在 `Cargo.toml` 中引入 `vacro` 并开启你需要的开发体验特性：

```toml
[dependencies]
vacro = { version = "0.2", features = ["full"] }

```

### 1. 声明式解析 (`vacro-parser`)

像写正则一样定义你的宏输入格式：

```rust
use vacro::prelude::*;

// 定义语法: "fn" <name> "(" <args> ")"
vacro::define!(MyMacroInput:
    fn
    #(name: syn::Ident)
    ( #(args*[,]: syn::Type) )
);

```

更多内容参见：[vacro-parser](https://docs.rs/vacro-parser)

### 2. 诊断报告 (`vacro-report`)

提供更优的错误报告能力，告别 `unexpected identifier`。

```rust

#[vacro::report::scope]
fn my_macro_impl(input: TokenStream) -> TokenStream {
    let name = Ident::new("foo", Span::call_site());

    // If this fails (e.g., invalid syntax constructed),
    // proc-macro panicked: parse_quote failed at file:line:column
    // Error message: “expected an expression”
    // Tokens:
    // fn foo () {>> invalid <<}
    let f: ItemFn = parse_quote!( fn #name () { >>invalid<< } );

    quote!(#f)
}

```

## 路线图 (Roadmap)

我们正处于快速开发状态，正在向Dev框架转型的过程中。

- [x] **文档能力**: 多语言支持 (`vacro-doc-i18n`).
- [x] **解析能力**: 核心 DSL 实现 (`vacro-parser`).
- [x] **诊断能力**: 错误报告集成 (`vacro-report`).
- [ ] **调试能力**: 开发 `vacro-trace`.

## 贡献

我们要打造 Rust 元编程领域最好的开发体验。如果你有关于如何减轻写宏痛苦的想法，欢迎提交 Issue！

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
