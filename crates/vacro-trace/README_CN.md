# Vacro Trace

**Rust 过程宏的可观测性工具集**

[![crates.io](https://img.shields.io/crates/v/vacro-trace.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/vacro-trace)
[![docs.rs](https://img.shields.io/badge/docs.rs-vacro--trace-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs)](https://docs.rs/vacro-trace)

## 简介

`vacro-trace` 将熟悉的可观测性工具（日志、追踪、快照）引入到过程宏开发中。

它作为**捕获层**存在，设计上与 **`vacro-cli`**（可视化层）配合使用。`vacro-trace` 负责记录数据，而数据的查看和快照差异对比需要通过 `vacro-cli` 进行。

## 安装

```toml
[dependencies]
vacro-trace = "0.1.2"
```

## 使用方法

### 1. 仪表化 (Instrumentation)

需要为宏入口标记`#[instrument]`

```rust
#[instrument]
#[proc_macro]
fn parse_impl(input: proc_macro2::TokenStream) {
    // ...
}
```

### 2. 快照与 Diff (Snapshots)

使用 `snapshot!(tag, tokens)` 来捕获特定时间点的 Token 状态。

如果你使用**相同的标签**（例如 "transformation"）进行了多次快照，`vacro-cli` 会自动生成差异视图，展示 Token 是如何演变的。

```rust
let mut tokens = quote! { fn hello() {} };
// 初始状态
snapshot!("my_macro", tokens);

// ... 修改 tokens ...
tokens = quote! { fn hello() { println!("world"); } };

// 最终状态 - vacro-cli 将展示这两次快照之间的 Diff
snapshot!("my_macro", tokens);
```

### 3. 日志记录

```rust
info!("开始展开宏...");
warn!("看起来有点可疑: {}", "ident_name");
```
