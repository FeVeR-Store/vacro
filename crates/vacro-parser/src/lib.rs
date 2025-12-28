#![warn(missing_docs)]

//!<div class="doc-cn">
//!
#![doc = include_str!("docs/zh_cn.md")]
//!
//!</div>
//!
//! <div class="doc-en">
//!
#![doc = include_str!("docs/en.md")]
//!
//!</div>

pub(crate) mod ast;
pub(crate) mod codegen;
mod impls;
pub(crate) mod syntax;
pub(crate) mod transform;

use proc_macro::TokenStream;
use vacro_doc_i18n::doc_i18n;

use crate::impls::{bind_impl, define_impl};

#[doc_i18n]
/// <div class="doc-cn"> 即时解析宏：在现有的解析逻辑中快速消费 `TokenStream` </div>
/// <div class="doc-en"> On-the-fly parsing macro: Quickly consume a `TokenStream` within existing parsing logic </div>
///
/// <div class="doc-cn">
///
/// `bind!` 宏模拟了一个 `let` 绑定语句。其中的核心部分 `(<input> -> <pattern>)` 会被展开为一个表达式，
/// 其求值结果为 `syn::Result<Output>`。
///
/// 这意味着你可以像处理普通 `Result` 一样处理它，例如在后面直接接 `?` 操作符、`.unwrap()` 或者 `.map(...)`。
///
/// # 语法
///
/// ```text
/// vacro::bind!(let <绑定模式> = (<输入流> -> <Vacro模式>) <后续操作>);
/// ```
///
/// * **绑定模式**: 标准 Rust 模式，用于接收解析成功后的内容（如变量名 `res` 或元组 `(a, b)`）。
/// * **输入流**: 实现了 `Into<TokenStream>` 的变量。
/// * **Vacro模式**: 描述语法的模式。
/// * **后续操作**: 针对 `Result` 的操作，如 `?;`、`.unwrap();` 等。
///
/// # 捕获规则
///
/// * **具名捕获** (`#(name: Type)`): 宏会生成一个包含这些字段的临时结构体 `Output`。
/// * **行内捕获** (`#(@: Type)`): 宏会返回一个元组（或单个值），按照定义的顺序包含捕获的内容。
///
/// # 示例
///
/// ```rust
/// # use syn::{Ident, Type, Token, Result};
/// # use vacro::bind;
/// # use proc_macro2::TokenStream;
/// fn parser(input: TokenStream) -> Result<()> {
///     // 场景 1: 配合 `?` 使用 (推荐)
///     // 表达式 `(input -> ...)` 返回 Result，后面接 `?` 传播错误
///     let input1 = input.clone();
///     bind!(
///         let func = (input1 -> fn #(name: Ident))?;
///     );
///     println!("Function: {}", func.name);
///
///     // 场景 2: 配合 `unwrap` 使用
///     // 如果你确定这里一定能解析成功
///     let input2 = input.clone();
///     bind!(
///         let (arrow, ty) = (input2 -> #(@: Token![->]) #(@: Type)).unwrap();
///     );
///
///     // 场景 3: 手动处理 Result
///     let input3 = input.clone();
///     bind!(
///         let result = (input3 -> #(kw: Token![mod]));
///     );
///     if let Ok(output) = result {
///         // ...
///     }
///
///     Ok(())
/// }
/// ```
/// </div>
/// <div class="doc-en">
///
/// The `bind!` macro mimics a `let` binding statement. The core expression `(<input> -> <pattern>)`
/// expands to a block that evaluates to `syn::Result<Output>`.
///
/// This means you can treat it like any other `Result` expression, appending `?`, `.unwrap()`,
/// or `.map(...)` directly after it.
///
/// # Syntax
///
/// ```text
/// vacro::bind!(let <binding> = (<input> -> <pattern>) <operations>);
/// ```
///
/// * **binding**: Standard Rust pattern to receive the parsed content (e.g., `res` or `(a, b)`).
/// * **input**: A variable implementing `Into<TokenStream>`.
/// * **pattern**: The Vacro pattern description.
/// * **operations**: Operations on the `Result`, such as `?;`, `.unwrap();`, etc.
///
/// # Capture Rules
///
/// * **Named Capture** (`#(name: Type)`): Generates a temporary `Output` struct containing these fields.
/// * **Inline Capture** (`#(@: Type)`): Returns a tuple (or single value) containing captured items in order.
///
/// # Example
///
/// ```rust
/// # use syn::{Ident, Type, Token, Result};
/// # use vacro::bind;
/// # use quote::ToTokens;
/// # use proc_macro2::TokenStream;
/// fn parser(input: TokenStream) -> Result<()> {
///     // Case 1: Using with `?` (Recommended)
///     // The expression `(input -> ...)` returns Result, `?` propagates error
///     let input1 = input.clone();
///     bind!(
///         let func = (input1 -> fn #(name: Ident))?;
///     );
///     println!("Function: {}", func.name);
///
///     // Case 2: Using with `unwrap`
///     // If you are sure parsing will succeed
///     let input2 = input.clone();
///     bind!(
///         let (arrow, ty) = (input2 -> #(@: Token![->]) #(@: Type)).unwrap();
///     );
///
///     // Case 3: Manually handling Result
///     let input3 = input.clone();
///     bind!(
///         let result = (input3 -> #(kw: Token![mod]));
///     );
///     if let Ok(output) = result {
///         // ...
///     }
///
///     Ok(())
/// }
/// ```
/// </div>
#[proc_macro]
pub fn bind(input: TokenStream) -> TokenStream {
    bind_impl(input)
}
#[doc_i18n]
/// <div class="doc-cn"> 结构体定义宏：定义一个新的 AST 节点并自动实现 `syn::parse::Parse` </div>
/// <div class="doc-en"> Struct definition macro: Define a new AST node and implement `syn::parse::Parse` automatically </div>
///
/// <div class="doc-cn">
///
///
/// `define!` 宏用于创建可复用的语法结构。它会根据提供的模式生成一个结构体，
/// 并生成相应的解析逻辑，使其可以直接通过 `syn::parse_macro_input!` 或 `input.parse()` 使用。
///
/// # 语法
///
/// ```text
/// vacro::define!(StructName: <Pattern>);
/// ```
///
/// 宏会自动生成：
/// 1. `struct StructName { ... }`：包含所有**具名捕获**的字段。
/// 2. `impl syn::parse::Parse for StructName { ... }`：包含解析逻辑。
///
/// # 注意事项
///
/// * `define!` 中通常使用**具名捕获** (`#(name: Type)`) 来生成结构体字段。
///
/// # 示例
///
/// ```rust
/// # use syn::{Ident, Type, Token, parse::Parse, parse::ParseStream, Result};
/// # use vacro::define;
/// # use quote::quote;
/// # use proc_macro2::TokenStream;
/// // 定义一个简单的常量定义结构
/// // const NAME: Type = Value;
/// define!(MyConst:
///     const
///     #(name: Ident)
///     :
///     #(ty: Type)
///     =
///     #(value: syn::Expr)
///     ;
/// );
///
/// fn parser(input: ParseStream) -> Result<()> {
///     // MyConst 自动实现了 Parse trait
///     let MyConst { name, ty, value } = input.parse()?;
///     println!("Const {} has type {}, value: {}", name, quote!(#ty), quote!(#value));
///     Ok(())
/// }
/// ```
/// </div>
/// <div class="doc-en">
///
/// The `define!` macro is used to create reusable syntax structures. It generates a struct
/// based on the provided pattern and implements the parsing logic, making it usable
/// directly via `syn::parse_macro_input!` or `input.parse()`.
///
/// # Syntax
///
/// ```text
/// vacro::define!(StructName: <Pattern>);
/// ```
///
/// The macro automatically generates:
/// 1. `struct StructName { ... }`: Containing all **named captured** fields.
/// 2. `impl syn::parse::Parse for StructName { ... }`: Containing the parsing logic.
///
/// # Notes
///
/// * `define!` typically uses **Named Captures** (`#(name: Type)`) to generate struct fields.
///
/// # Example
///
/// ```rust
/// # use syn::{Ident, Type, Token, parse::Parse, parse::ParseStream, Result};
/// # use vacro::define;
/// # use quote::quote;
/// # use proc_macro2::TokenStream;
/// // Define a simple constant definition structure
/// // const NAME: Type = Value;
/// define!(MyConst:
///     const
///     #(name: Ident)
///     :
///     #(ty: Type)
///     =
///     #(value: syn::Expr)
///     ;
/// );
///
/// fn parser(input: ParseStream) -> Result<()> {
///     // MyConst automatically implements the Parse trait
///     let MyConst { name, ty, value } = input.parse()?;
///     println!("Const {} has type {}, value: {}", name, quote!(#ty), quote!(#value));
///     Ok(())
/// }
///
/// ```
/// </div>
#[proc_macro]
pub fn define(input: TokenStream) -> TokenStream {
    define_impl(input)
}
