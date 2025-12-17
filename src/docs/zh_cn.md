# Vacro

**让 Rust 过程宏开发重归简单：声明式解析库**

[<img alt="github" src="https://img.shields.io/badge/github-FeVeR_Store/vacro-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/FeVeR-Store/vacro)
[<img alt="crates.io" src="https://img.shields.io/crates/v/vacro.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/vacro)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-vacro-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/vacro)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/FeVeR-Store/vacro/publish.yml?style=for-the-badge" height="20">](https://github.com/FeVeR-Store/vacro/actions/workflows/publish.yml)


## 简介

**Vacro** 是一个专为 Rust 过程宏（Procedural Macros）设计的声明式解析库。

如果你受够了使用 `syn` 时编写冗长的命令式代码（无数的 `input.parse()?`、手动的 `lookahead`、复杂的 `Punctuated` 处理），那么 **Vacro** 就是为你准备的。

**核心理念：站在巨人的肩膀上。**

Vacro 并不发明新的 AST 类型。所有的解析结果依然是标准的 `syn::Ident`、`syn::Type`、`syn::Expr` 等。我们只是提供了一种类似 `macro_rules!` 的**声明式语法**，自动生成底层的 `syn` 解析逻辑。

## 痛点对比

假设我们要解析一个带有泛型的函数签名：`fn my_func<T, U>(a: i32) -> bool`。

### ❌ 传统写法 (Raw Syn)

为了解析这个结构，你需要编写几十行样板代码来处理泛型、括号、逗号分隔符和可选返回值：

```rust
// 传统的 syn 解析逻辑：逻辑分散，容易出错
# use syn::{
#     FnArg, GenericParam, Ident, Result, Token, Type, parenthesized,
#     parse::{Parse, ParseStream},
#     punctuated::Punctuated,
# };
struct MyFn {
    name: Ident,
    generics: Option<Punctuated<GenericParam, Token![,]>>,
    args: Punctuated<FnArg, Token![,]>,
    ret: Option<Type>
}

impl Parse for MyFn {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![fn]>()?; // 1. 吃掉关键字
        // 2. 手动处理泛型 (Peek + 解析)
        let generics = if input.peek(Token![<]) {
             input.parse::<Token![<]>()?;
             let params = Punctuated::parse_terminated(input)?;
             input.parse::<Token![>]>()?;
             Some(params)
        } else {
             None
        };
        let name: Ident = input.parse()?; // 3. 解析名字
        let content;
        parenthesized!(content in input); // 4. 处理括号
        let args: Punctuated<FnArg, Token![,]> =
            content.parse_terminated(FnArg::parse, Token![,])?;
        // 5. 处理可选的返回值
        let ret = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            Some(input.parse::<Type>()?)
        } else {
            None
        };
        Ok(MyFn { name, generics, args, ret })
    }
}
```

### ✅ 使用 Vacro

使用 **Vacro**，你只需要描述语法长什么样；所见即所得。

```rust
# use syn::{Ident, Type, GenericParam, Token, FnArg, Result, punctuated::Punctuated};
vacro::define!(MyFn:
    fn                                    // 匹配字面量
    #(?: <#(generic*[,]: GenericParam)>)  // 可选的泛型参数列表（尖括号包裹 + 逗号分隔）
    #(name: Ident)                        // 具名捕获函数名
    ( #(args*[,]: FnArg) )                // 参数列表（圆括号包裹 + 逗号分隔）
    #(?: -> #(ret: Type))                 // 可选的返回值
);
```

如果写到一行：

```rust
# use syn::{Ident, Type, GenericParam, Token, FnArg, Result, punctuated::Punctuated};
vacro::define!(MyFn: fn #(?: <#(generic*[,]: GenericParam)>) #(name: Ident) (#(args*[,]: FnArg)) #(?: -> #(ret: Type)));
```

一行代码，涵盖了所有复杂的解析逻辑。

## 核心宏

Vacro 提供了两个核心宏，分别用于**定义结构体**和**即时解析**。

### 1\. `define!`：定义解析结构体

如果你需要定义一个可复用的 AST 节点（即定义一个 `struct` 并自动实现 `syn::parse::Parse`），请使用 `define!`。

```rust
# use syn::{Ident, Type, GenericParam, Token, FnArg, Result, punctuated::Punctuated, parse_macro_input};
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
    # proc_macro::TokenStream::new()
}
```

### 2\. `bind!`：即时流解析

如果你在现有的解析逻辑中，想要快速消费一段 `TokenStream`，请使用 `bind!`。

#### 命名捕获 (Named Capture)

如果在模式中使用了 `name: Type` 的形式，宏会生成一个包含所有字段的结构体 `Output`。

```rust
# use syn::{Ident, Type};
# fn proc_macro(input: proc_macro::TokenStream) -> syn::Result<()> {
vacro::bind!(
    let captured = (input ->
        fn #(name: Ident) #(?: -> #(ret: Type)))?;
);
// 访问字段
captured.name; // Ident
captured.ret;  // Option<Type>
# Ok(())
# }
```

#### 行内捕获 (Inline Capture)

如果模式中没有指定名称（或只包含单个匿名捕获），宏将返回元组或单个值。

```rust
# use syn::{Ident, Type};
# fn inline_capture(input: proc_macro::TokenStream) -> syn::Result<()> {
    // 仅解析类型，不需要名字
    vacro::bind!(
        let (ident, ty) = (input -> #(@:Ident): #(@:Type))?;
    );
    // 访问字段
    ident; // Ident
    ty;    // Type

    # Ok(())
# }
```

## 语法参考

Vacro 的 DSL 设计直觉来源于 `macro_rules!` 和正则表达式。

| 语法            | 类型     | 描述                                                                        | 解析结果类型             | 示例                     |
| :-------------- | :------- | :-------------------------------------------------------------------------- | :----------------------- | :----------------------- |
| `literal`       | 字面量   | 匹配并消费 Token (如 Rust 关键字/符号 `fn`, `->` 或自定义符号 `miku`, `<>`) | `!`                      | `fn`, `->`, `miku`, `<>` |
| `#(x: T)`       | 具名捕获 | 捕获一个特定的 `syn` 类型                                                   | `T` (如 `Ident`, `Type`) | `#(name: Ident)`         |
| `#(x?: T)`      | 具名可选 | 尝试解析，失败则跳过                                                        | `Option<T>`              | `#(name?: Ident)`        |
| `#(x*[sep]: T)` | 具名迭代 | 类似 `Punctuated`，按分隔符解析                                             | `Punctuated<T, sep>`     | `#(args*: Ident)`        |
| `#(T)`          | 匿名捕获 | 捕获一个特定的 `syn` 类型，但仅作验证（不返回）                             | `!`                      | `#(Ident)`               |
| `#(?: T)`       | 匿名可选 | 仅作验证，失败则跳过                                                        | `!`                      | `#(?: Ident)`            |
| `#(*[sep]: T)`  | 匿名迭代 | 类似 `Punctuated`，按分隔符解析（仅作验证）                                 | `!`                      | `#(*[,]: Ident)`         |

## 多态捕获 (Enum Parsing)

Vacro 支持解析“多态”结构，即输入流中的某个位置可能是多种类型之一。通过定义枚举变体，Vacro 会自动生成解析逻辑（使用 lookahead/forking）来尝试每种变体。

语法：`#(name: EnumName { Variant1, Variant2: Type, Variant3: Pattern })`

```rust
# use syn::{Ident, Expr};

vacro::define!(MyPoly:
    #(data: MyEnum {
        Ident,                            // 1. 简写：匹配 Ident，生成 MyEnum::Ident(Ident)
        syn::Type,                        // 2. 简写：匹配 syn::Type，生成 MyEnum::Type(syn::Type)
        Integer: syn::LitInt,             // 3. 别名：匹配 syn::LitInt，生成 MyEnum::Integer(syn::LitInt)
        Function: fn #(name: Ident),      // 4. 模式：匹配模式（具名），生成 MyEnum::Function { name: Ident }
        Tuple: (#(@: Ident), #(@: Expr)), // 5. 模式：匹配模式（行内），生成 MyEnum::Tuple(Ident, Expr)
    })
);

// 宏会自动生成如下 Enum 定义：
// pub enum MyEnum {
//     Ident(Ident),
//     Type(syn::Type),
//     Integer(syn::LitInt),
//     Function { name: Ident },
//     Tuple(Ident, Expr)
// }
```

## 端到端示例

这是一个演示如何解析自定义“服务定义”语法的完整示例。

**目标语法:**

```text
service MyService {
    version: "1.0",
    active: true
}
```

**Implementation / 实现代码:**

```rust
use syn::{parse::Parse, parse::ParseStream, Ident, LitStr, LitBool, Token, Result, parse_quote};
use vacro::define;
// 1. 使用 vacro DSL 定义 AST
define!(ServiceDef:
    service                   // Keyword "service"
    #(name: Ident)            // Captured Service Name
    {                         // Braced block
        version : #(ver: LitStr) ,  // "version" ":" <string> ","
        active : #(is_active: LitBool) // "active" ":" <bool>
    }
);
// 2. 模拟解析（在真实宏中，这来自输入的 TokenStream）
fn main() -> Result<()> {
    // 模拟输入: service MyService { version: "1.0", active: true }
    let input: proc_macro2::TokenStream = quote::quote! {
        service MyService {
            version: "1.0",
            active: true
        }
    };
    // 解析它！
    let service: ServiceDef = syn::parse2(input)?;
    // 3. 访问字段
    assert_eq!(service.name.to_string(), "MyService");
    assert_eq!(service.ver.value(), "1.0");
    assert!(service.is_active.value);
    println!("Successfully parsed service: {}", service.name);
    Ok(())
}
```

---

# Vacro 开发路线图 (Roadmap)

## 📅 阶段一：夯实基础 (v0.1.x) - 当前重点

**目标：** 确保现有核心宏（`define!`、`bind!`）稳定可靠，并建立完善的测试与文档体系。

### 1\. 完善文档 (Documentation)

- [x] **API 文档化**：为 `Pattern`、`BindInput` 和 `Keyword` 等核心结构添加详细的 Rustdoc 注释，确保 `docs.rs` 上的可读性。
- [x] **README 增强**：整合最新的 README，添加 `examples/` 目录，并提供基础的实战案例（如解析简单的结构体和函数）。
- [ ] **错误报告优化**：优化 `syn::Error` 的生成，确保当 DSL 语法错误（如括号不匹配）时，用户能收到清晰的编译器报错，而不是内部 panic。

### 2\. 完善测试体系 (Testing)

- [x] **单元测试 (Unit Tests)**：
  - [x] 覆盖 `inject_lookahead` 的边缘情况（递归 Group、连续 Literals 等）。
  - [x] 测试 `Keyword` 解析器处理特殊符号（`->`、`=>`、`<`）及自定义关键字的能力。
- [ ] **UI 测试 (Compile-fail Tests)**：
  - [ ] **集成 `trybuild`**。
  - [ ] 编写“反向测试用例”：验证当输入不符合预期类型时（例如期望 `Ident` 却提供了 `LitStr`），宏能否正确拦截并报告错误。
- [x] **集成测试 (Integration Tests)**：
  - [x] 模拟真实场景，验证 `define!` 生成的结构体能否正确处理复杂的 TokenStream。

---

## 🚀 阶段二：架构革新 (v0.2.x) - 核心增强

**目标：** 引入高级数据结构映射能力，解决复杂 AST 中的“多态”与“聚合”问题，使 Vacro 能够处理复杂的语法树。

### 3\. 新语法开发 (New Syntax)

#### A. 关联/结构化捕获 (Associative/Structural Capture)

_解决“结构体数组 (AoS)”问题，即一次性捕获聚合的结构，而不是分散的字段列表。_

- [ ] **语法实现**：支持 `#(~name...: ...)` 语法来标记聚合捕获。
- [ ] **元组支持**：实现 `#(~items*: #(@:Type) #(@:Ident))`，以生成 `Vec<(Type, Ident)>`。
- [ ] **结构体支持**：支持内部具名捕获，以生成匿名结构体列表。

#### B. 多态捕获 (Polymorphic Capture / Enum Parsing)

_解决“多态解析”问题，即一个位置可能是多种类型之一。_

- [x] **语法实现**：支持 `#(name: EnumName { VariantA, VariantB })` 语法。
- [x] **自动定义**：如果 `EnumName` 未定义，自动生成包含 `VariantA(TypeA)`、`VariantB(TypeB)` 的枚举定义。
- [x] **分支解析**：生成基于 `input.fork()` 或 `peek` 的尝试解析逻辑，自动处理失败时的回溯（backtracking）。

---

## 🛠️ 阶段三：生态与工具 (v0.3.x) - 开发者体验

**目标：** 提供周边工具，降低 Vacro 的学习曲线和调试成本。

### 4\. 工具链开发 (Toolchain)

- [ ] 敬请期待

---

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
