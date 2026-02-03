# Vacro Parser

**Vacro 的声明式解析内核**

## 简介

**Vacro Parser** 是 Vacro 框架的核心声明式解析引擎。它提供了一种类似 `macro_rules!` 的 DSL，用于简化 Rust 过程宏中基于 `syn` 的解析逻辑。

它允许你声明式地定义 AST 结构和解析逻辑，消除了繁琐的 `input.parse()?` 调用。

## 安装

在你的 `Cargo.toml` 中添加:

```toml
[dependencies]
vacro-parser = "0.1.8"
```

## 核心功能

### 1. `define!`：定义解析结构体

使用 `define!` 定义一个结构体，它会自动实现 `syn::parse::Parse`。

```rust
use syn::{Ident, Type, GenericParam, FnArg, parse_quote, Token};
use vacro_parser::define;
// 定义一个名为 MyFn 的结构体，它会自动实现 Parse trait
vacro::define!(MyFn:
    fn
    #(?: <#(generic*[,]: GenericParam)>)
    #(name: Ident)
    ( #(args*[,]: FnArg) )
    #(?: -> #(ret: Type))
);

fn parse_my_fn(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    // 使用方式
    let my_fn: MyFn = parse_quote!(input);
    println!("Function name: {}", my_fn.name);
    proc_macro2::TokenStream::new()
}
fn main() {}
```

### 2. `bind!`：即时流解析

在现有的命令式逻辑中，使用 `bind!` 快速消费一段 `TokenStream`。

```rust
use syn::{Ident, Type, Token, Result};
use vacro_parser::bind;
fn parser(input: syn::parse::ParseStream) -> Result<()> {
    // 即时解析函数签名模式
    bind!(
        let captured = (input ->
            fn #(name: Ident) #(?: -> #(ret: Type))
        )?;
    );

    // 直接访问捕获的字段
    println!("Name: {}", captured.name);
    if let Some(ret_type) = captured.ret {
        // ...
    }
    Ok(())
}
fn main() {}
```

## 语法参考

| 语法            | 描述                                  | 解析结果类型         | 示例                 |
| :-------------- | :------------------------------------ | :------------------- | :------------------- |
| `literal`       | 匹配并消费确切的 Token                | `!`                  | `fn`, `->`, `struct` |
| `#(x: T)`       | **具名捕获**: 捕获类型 `T` 到字段 `x` | `T`                  | `#(name: Ident)`     |
| `#(x?: T)`      | **具名可选**: 尝试解析，失败则跳过    | `Option<T>`          | `#(ret?: Type)`      |
| `#(x*[sep]: T)` | **具名迭代**: 按分隔符解析            | `Punctuated<T, sep>` | `#(args*[,]: FnArg)` |
| `#(T)`          | **匿名捕获**: 验证 `T` 存在但不捕获   | `!`                  | `#(Ident)`           |
| `#(?: T)`       | **匿名可选**: 仅作验证                | `!`                  | `#(?: Ident)`        |
| `#(*[sep]: T)`  | **匿名迭代**: 仅作验证                | `!`                  | `#(*[,]: Ident)`     |

## 多态捕获 (Enum Parsing)

Vacro 支持解析“多态”结构，即输入流中的某个位置可能是多种类型之一。

```rust
use syn::{Ident, Expr, Type, LitInt};
use vacro_parser::define;
define!(MyPoly:
    #(data: MyEnum {
        Ident,                            // 1. 简写：匹配 Ident，生成 MyEnum::Ident(Ident)
        syn::Type,                        // 2. 简写：匹配 Type，生成 MyEnum::Type(syn::Type)
        Integer: syn::LitInt,             // 3. 别名：匹配 LitInt，生成 MyEnum::Integer(LitInt)
        Function: fn #(name: Ident),      // 4. 模式：生成 MyEnum::Function { name: Ident }
        Tuple: (#(@: Ident), #(@: Expr)), // 5. 模式：生成 MyEnum::Tuple(Ident, Expr)
    })
);
fn main() {}
```

## 更友好的提示 (v0.1.6)

你可以使用`vacro-report`的`help!`宏为内容提供更友好的提示，若你使用了`vacro`，只需要开启`report`feature即可。

```toml
vacro_parser = { version = "0.1.8" }
vacro_report = { version = "0.1.3", features = ["parser"] }

# vacro = { version = "0.2.3", features = ["parser", "report"] }
```

```rust
use vacro_parser::define;
use vacro_report::help;
use syn::{Ident, LitBool};

help!(Bool:
    LitBool {
        error: "此处需要一个bool字面量，接收到的是：{input}",
        help: "尝试`true`或`false`",
        example: (true | false) // example 字段是要展示的示例字段，在生成错误信息与使用示例时使用；它接受一段TokenStream，并且将直接展示你传入的内容
    }
);

define!(MyRoles: {
    #(roles*[,]: #(pair: #(name: Ident): #(enable: Bool)))
});

```
