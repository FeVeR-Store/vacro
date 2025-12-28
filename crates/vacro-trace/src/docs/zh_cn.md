# Vacro Trace

**Rust 过程宏的可观测性工具集**

## 简介

`vacro-trace` 将熟悉的可观测性工具（日志、追踪、快照）引入到过程宏开发中。

它作为**捕获层**存在，设计上与 **`vacro-cli`**（可视化层）配合使用。`vacro-trace` 负责记录数据，而数据的查看和快照差异对比需要通过 `vacro-cli` 进行。

## 特性

- **结构化日志**: `error!`, `warn!`, `info!`, `debug!`, `trace!` 宏。
- **Token 快照**: 使用标签（Tag）捕获 `TokenStream` 状态。`vacro-cli` 会自动对比具有相同标签的快照差异（Diff）。
- **自动仪表化**: `#[instrument]` 属性，自动追踪函数调用。

## 使用方法

### 1. 仪表化 (Instrumentation)

需要为宏入口标记`#[instrument]`

```rust,ignore
# use vacro_trace::instrument;
#[instrument]
#[proc_macro]
fn parse_impl(input: proc_macro2::TokenStream) {
    // ...
}
# fn main() {}
```

### 2. 快照与 Diff (Snapshots)

使用 `snapshot!(tag, tokens)` 来捕获特定时间点的 Token 状态。

如果你使用**相同的标签**（例如 "transformation"）进行了多次快照，`vacro-cli` 会自动生成差异视图，展示 Token 是如何演变的。

```rust
# use vacro_trace::snapshot;
# use quote::quote;
# fn main() {
let mut tokens = quote! { fn hello() {} };
// 初始状态
snapshot!("my_macro", tokens);

// ... 修改 tokens ...
tokens = quote! { fn hello() { println!("world"); } };

// 最终状态 - vacro-cli 将展示这两次快照之间的 Diff
snapshot!("my_macro", tokens);
# }
```

### 3. 日志记录

```rust
# use vacro_trace::{info, warn};
# fn main() {
info!("开始展开宏...");
warn!("看起来有点可疑: {}", "ident_name");
# }
```

## 查看结果

要查看捕获的数据：

1. 安装 CLI: `cargo install vacro-cli` (或从源码编译)。
2. 运行构建: `cargo build`。
3. 打开 TUI: `vacro-cli`。
