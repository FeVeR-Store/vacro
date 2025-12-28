# Vacro Parser

**Vacro 的声明式解析内核**

## 简介

**Vacro Parser** 是 Vacro 框架的核心声明式解析引擎。它提供了一种类似 `macro_rules!` 的 DSL，用于简化 Rust 过程宏中基于 `syn` 的解析逻辑。

它允许你声明式地定义 AST 结构和解析逻辑，消除了繁琐的 `input.parse()?` 调用。

## 核心功能

### 1. `define!`：定义解析结构体

使用 `define!` 定义一个结构体，它会自动实现 `syn::parse::Parse`。

```rust
use syn::{Ident, Type, GenericParam, FnArg, parse_macro_input, Token};
use vacro::define;
// 定义一个名为 MyFn 的结构体，它会自动实现 Parse trait
vacro::define!(MyFn:
    fn
    #(?: <#(generic*[,]: GenericParam)>)
    #(name: Ident)
    ( #(args*[,]: FnArg) )
    #(?: -> #(ret: Type))
);

fn parse_my_fn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // 使用方式
    let my_fn = parse_macro_input!(input as MyFn);
    println!("Function name: {}", my_fn.name);
    proc_macro::TokenStream::new()
}
fn main() {}
```

### 2. `bind!`：即时流解析

在现有的命令式逻辑中，使用 `bind!` 快速消费一段 `TokenStream`。

```rust
use syn::{Ident, Type, Token, Result};
use vacro::bind;
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

| 语法 | 描述 | 解析结果类型 | 示例 |
| :--- | :--- | :--- | :--- |
| `literal` | 匹配并消费确切的 Token | `!` | `fn`, `->`, `struct` |
| `#(x: T)` | **具名捕获**: 捕获类型 `T` 到字段 `x` | `T` | `#(name: Ident)` |
| `#(x?: T)` | **具名可选**: 尝试解析，失败则跳过 | `Option<T>` | `#(ret?: Type)` |
| `#(x*[sep]: T)` | **具名迭代**: 按分隔符解析 | `Punctuated<T, sep>` | `#(args*[,]: FnArg)` |
| `#(T)` | **匿名捕获**: 验证 `T` 存在但不捕获 | `!` | `#(Ident)` |
| `#(?: T)` | **匿名可选**: 仅作验证 | `!` | `#(?: Ident)` |
| `#(*[sep]: T)` | **匿名迭代**: 仅作验证 | `!` | `#(*[,]: Ident)` |

## 多态捕获 (Enum Parsing)

Vacro 支持解析“多态”结构，即输入流中的某个位置可能是多种类型之一。

```rust
use syn::{Ident, Expr, Type, LitInt};
use vacro::define;
vacro::define!(MyPoly:
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
