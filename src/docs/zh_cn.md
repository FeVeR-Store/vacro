# Vacro

**Rust 过程宏的渐进式 DevX 框架**

## 设计理念

在 Rust 中编写过程宏不应该是一场充满样板代码和黑盒调试的噩梦。

**Vacro** 已经从一个简单的解析库演进为一个完整的工具链，旨在提升过程宏开发全生命周期的 **开发者体验 (DevX)**：

1. **解析 (Parsing)**：以声明式的方式编写解析逻辑。
2. **调试 (Debugging)**：可视化解析路径，精准洞察宏内部发生了什么。
3. **报告 (Reporting)**：轻松生成优雅且精确的编译器错误信息。

## 生态系统

Vacro 被设计为一个模块化框架。你可以使用功能齐全的 `vacro` 入口，也可以根据需要挑选特定的底层组件。

| 功能              | Crate          | 描述                                                                          |
| :---------------- | :------------- | :---------------------------------------------------------------------------- |
| **Parsing**       | `vacro-parser` | **声明式解析**。类似 `macro_rules!` 的 DSL，自动实现 `syn::Parse`。           |
| **Debugging**     | `vacro-trace`  | **可视化追踪**。捕获快照和日志，以解决复杂的语法调试问题。                    |
| **Visualization** | `vacro-cli`    | **终端工具**。一个 TUI 界面，用于检查由 `vacro-trace` 捕获的追踪和快照 Diff。 |
| **Diagnostics**   | `vacro-report` | **错误报告**。简化过程宏中诊断信息的构建和发射。                              |

## 快速开始

### 1. 声明式解析 (`vacro-parser`)

像写正则一样定义你的宏输入语法：

```rust
# #[cfg(feature = "parser")]
# mod parser_example {
use vacro::prelude::*;

// 定义语法: "fn" <name> "(" <args> ")"
vacro::define!(MyMacroInput:
    fn
    #(name: syn::Ident)
    ( #(args*[,]: syn::Type) )
);
# }
```

更多内容参见：[vacro-parser](https://docs.rs/vacro-parser)

### 2. 可视化调试 (`vacro-trace`)

对你的 TokenStream 进行快照，观察它如何演变。在 `vacro-cli` 中查看 Diff。

```rust
# #[cfg(feature = "trace")]
# mod trace_example {
use vacro::prelude::*;

# fn main() {
let tokens = quote::quote! { struct A; };
// 使用 tag 捕获快照。
// 如果使用相同的 tag 调用多次，vacro-cli 将展示它们之间的 Diff。
vacro::snapshot!("expand", tokens);
# }
# }
```

更多内容参见：[vacro-trace](https://docs.rs/vacro-trace)

### 3. 诊断报告 (`vacro-report`)

提供卓越的错误报告能力，告别通用的 `unexpected identifier` 错误。

```rust
# #[cfg(feature = "report")]
# mod report_example {
use vacro::prelude::*;

#[vacro::report::scope]
fn my_macro_impl(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    // 如果这里失败（例如构造了无效语法），
    // Vacro 会捕获它并发出指向代码的精确错误。
    // let f: syn::ItemFn = parse_quote!( fn foo () { >>invalid<< } );
    quote::quote!()
}
# }
```

更多内容参见：[vacro-report](https://docs.rs/vacro-report)

### 4. 可视化工具 (`vacro-cli`)

安装并运行 TUI 工具来查看追踪数据和快照对比。

```bash
cargo install vacro-cli
# 1. 运行测试生成追踪数据
cargo test
# 2. 启动可视化界面
cargo vacro

```

编写如下测试代码，运行测试后即可在 CLI 中查看捕获的日志和快照演变：

```rust
use vacro::trace::{debug, error, info, instrument, snapshot, warn};

#[test]
#[instrument]
fn test_function() {
    // 1. Log
    info!("Function started");
    warn!("This is a warning");
    error!("This is an error");

    // 2. Snapshot
    // Initial state
    let code_snippet = quote! { x: i32 };
    snapshot!("Field", code_snippet);

    // State change: Wrap in a struct
    // vacro-cli will automatically diff multiple snapshots with the "Struct" tag
    let code_snippet = quote! { struct A { #code_snippet }};
    snapshot!("Struct", code_snippet);

    // State change: Add derive
    let code_snippet = quote! {
        #[derive(Debug)]
        #code_snippet
    };
    snapshot!("Struct", code_snippet);

    let x = 1 + 1;
    debug!("Calculation result: {}", x);
}

```

<div align="center">
<img src="https://github.com/FeVeR-Store/vacro/blob/master/assets/vacro-cli.gif?raw=true" width="100%" alt="Vacro CLI 演示">
</div>

更多内容参见：[vacro-cli](https://crates.io/crates/vacro-cli)
