# Vacro

**Making Rust Procedural Macro Development Simple Again: A Declarative Parsing Library**

[<img alt="github" src="https://img.shields.io/badge/github-FeVeR_Store/vacro-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/FeVeR-Store/vacro)
[<img alt="crates.io" src="https://img.shields.io/crates/v/vacro.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/vacro)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-vacro-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/vacro)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/FeVeR-Store/vacro/publish.yml?branch=master&style=for-the-badge" height="20">](https://github.com/FeVeR-Store/vacro/actions?query=branch%3Amaster)


## Introduction

**Vacro** is a declarative parsing library designed specifically for Rust Procedural Macros.

If you are tired of writing verbose imperative code when using `syn` (countless `input.parse()?` calls, manual `lookahead`, complex `Punctuated` handling), then **Vacro** is for you.

**Core Philosophy: Standing on the shoulders of giants.**

Vacro does not invent new AST types. All parsing results remain standard `syn::Ident`, `syn::Type`, `syn::Expr`, etc. We simply provide a **declarative syntax** similar to `macro_rules!` to automatically generate the underlying `syn` parsing logic.

## Comparison: The Pain Points

Suppose we want to parse a function signature with generics: `fn my_func<T, U>(a: i32) -> bool`.

### ❌ Traditional Approach (Raw Syn)

To parse this structure, you need to write dozens of lines of boilerplate code to handle generics, parentheses, comma separators, and optional return values:

```rust
// Traditional syn parsing logic: scattered logic, error-prone
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
        input.parse::<Token![fn]>()?; // 1. Consume keyword
        // 2. Manually handle generics (Peek + Parse)
        let generics = if input.peek(Token![<]) {
             input.parse::<Token![<]>()?;
             let params = Punctuated::parse_terminated(input)?;
             input.parse::<Token![>]>()?;
             Some(params)
        } else {
             None
        };
        let name: Ident = input.parse()?; // 3. Parse name
        let content;
        parenthesized!(content in input); // 4. Handle parentheses
        let args: Punctuated<FnArg, Token![,]> =
            content.parse_terminated(FnArg::parse, Token![,])?;
        // 5. Handle optional return value
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

### ✅ Using Vacro

With **Vacro**, you only need to describe what the syntax looks like; what you see is what you get.

```rust
# use syn::{Ident, Type, GenericParam, Token, FnArg, Result, punctuated::Punctuated};
vacro::define!(MyFn:
    fn                                    // Match literal
    #(?: <#(generic*[,]: GenericParam)>)  // Optional generic param list (angle brackets + comma separated)
    #(name: Ident)                        // Named capture for function name
    ( #(args*[,]: FnArg) )                // Argument list (parentheses + comma separated)
    #(?: -> #(ret: Type))                 // Optional return value
);
```

If written in a single line:

```rust
# use syn::{Ident, Type, GenericParam, Token, FnArg, Result, punctuated::Punctuated};
vacro::define!(MyFn: fn #(?: <#(generic*[,]: GenericParam)>) #(name: Ident) (#(args*[,]: FnArg)) #(?: -> #(ret: Type)));
```

One line of code covers all complex parsing logic.

## Core Macros

Vacro provides two core macros for **defining structs** and **on-the-fly parsing**, respectively.

### 1\. `define!`: Define Parsing Structs

If you need to define a reusable AST node (i.e., define a `struct` and automatically implement `syn::parse::Parse`), use `define!`.

```rust
# use syn::{Ident, Type, GenericParam, Token, FnArg, Result, punctuated::Punctuated, parse_macro_input};
// Define a struct named MyFn, it automatically implements the Parse trait
vacro::define!(MyFn:
    fn
    #(?: <#(generic*[,]: GenericParam)>)
    #(name: Ident)
    ( #(args*[,]: FnArg) )
    #(?: -> #(ret: Type))
);

fn parse_my_fn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Usage
    let my_fn = parse_macro_input!(input as MyFn);
    println!("Function name: {}", my_fn.name);
    # proc_macro::TokenStream::new()
}
```

### 2\. `bind!`: On-the-fly Stream Parsing

If you want to quickly consume a segment of a `TokenStream` within existing parsing logic, use `bind!`.

#### Named Capture

If the pattern uses the form `name: Type`, the macro generates a struct named `Output` containing all fields.

```rust
# use syn::{Ident, Type};
# fn proc_macro(input: proc_macro::TokenStream) -> syn::Result<()> {
vacro::bind!(
    let captured = (input ->
        fn #(name: Ident) #(?: -> #(ret: Type))
    )?;
);
// Access fields
captured.name; // Ident
captured.ret;  // Option<Type>
# Ok(())
# }

```

#### Inline Capture

If no name is specified in the pattern (or it contains only anonymous captures), the macro will return a tuple or a single value.

```rust
# use syn::{Ident, Type};
# fn inline_capture(input: proc_macro::TokenStream) -> syn::Result<()> {
    // Parse types only, no names needed
    vacro::bind!(
        let (ident, ty) = (input -> #(@:Ident): #(@:Type))?;
    );
    // Access fields
    ident; // Ident
    ty;    // Type

    # Ok(())
# }

```

## Syntax Reference

Vacro's DSL design intuition comes from `macro_rules!` and regular expressions.

| Syntax          | Type           | Description                                                                                           | Result Type                | Example                  |
| :-------------- | :------------- | :---------------------------------------------------------------------------------------------------- | :------------------------- | :----------------------- |
| `literal`       | Literal        | Matches and consumes a Token (Rust keywords/symbols like `fn`, `->` or custom ones like `miku`, `<>`) | `!`                        | `fn`, `->`, `miku`, `<>` |
| `#(x: T)`       | Named Capture  | Captures a specific `syn` type                                                                        | `T` (e.g. `Ident`, `Type`) | `#(name: Ident)`         |
| `#(x?: T)`      | Named Optional | Attempts to parse; skips if failed                                                                    | `Option<T>`                | `#(name?: Ident)`        |
| `#(x*[sep]: T)` | Named Iter     | Similar to `Punctuated`, parses by separator                                                          | `Punctuated<T, sep>`       | `#(args*: Ident)`        |
| `#(T)`          | Anonymous      | Captures a specific `syn` type, but for validation only                                               | `!`                        | `#(Ident)`               |
| `#(?: T)`       | Anon Optional  | Validation only; skips if failed                                                                      | `!`                        | `#(?: Ident)`            |
| `#(*[sep]: T)`  | Anon Iter      | Similar to `Punctuated`, parses by separator (validation only)                                        | `!`                        | `#(*[,]: Ident)`         |

## Polymorphic Capture (Enum Parsing)

Vacro supports parsing "polymorphic" structures, where a position in the input stream can be one of multiple types. By defining enum variants, Vacro automatically generates the parsing logic (using lookahead/forking) to try each variant.

Syntax: `#(name: EnumName { Variant1, Variant2: Type, Variant3: Pattern })`

```rust
# use syn::{Ident, Expr};

vacro::define!(MyPoly:
    #(data: MyEnum {
        Ident,                            // 1. Shorthand: Match Ident, produces MyEnum::Ident(Ident)
        syn::Type,                        // 2. Shorthand: Match Type, produces MyEnum::Type(syn::Type)
        Integer: syn::LitInt,             // 3. Alias: Match syn::LitInt, produces MyEnum::Integer(syn::LitInt)
        Function: fn #(name: Ident),      // 4. Pattern: Match pattern(named), produces MyEnum::Function { name: Ident }
        Tuple: (#(@: Ident), #(@: Expr)), // 5. Pattern: Match pattern(inline), produces MyEnum::Tuple(Ident, Expr)
    })
);

// The macro automatically generates the Enum definition:
// pub enum MyEnum {
//     Ident(Ident),
//     Type(syn::Type),
//     Integer(syn::LitInt),
//     Function { name: Ident },
//     Tuple(Ident, Expr)
// }
```

## End-to-End Example

Here is a complete example showing how to parse a custom "Service Definition" syntax.

**Target Syntax**

```text
service MyService {
    version: "1.0",
    active: true
}
```

**Implementation:**

```rust
use syn::{parse::Parse, parse::ParseStream, Ident, LitStr, LitBool, Token, Result, parse_quote};
use vacro::define;
// 1. Define the AST using vacro DSL
define!(ServiceDef:
    service                   // Keyword "service"
    #(name: Ident)            // Captured Service Name
    {                         // Braced block
        version : #(ver: LitStr) ,  // "version" ":" <string> ","
        active : #(is_active: LitBool) // "active" ":" <bool>
    }
);
// 2. Simulate parsing (In a real macro, this comes from the input TokenStream)
fn main() -> Result<()> {
    // Mock input: service MyService { version: "1.0", active: true }
    let input: proc_macro2::TokenStream = quote::quote! {
        service MyService {
            version: "1.0",
            active: true
        }
    };
    // Parse it! / 解析它！
    let service: ServiceDef = syn::parse2(input)?;
    // 3. Access the fields
    assert_eq!(service.name.to_string(), "MyService");
    assert_eq!(service.ver.value(), "1.0");
    assert!(service.is_active.value);
    println!("Successfully parsed service: {}", service.name);
    Ok(())
}
```

---

# Vacro Roadmap

## 📅 Phase 1: Solidifying Foundations (v0.1.x) - Current Focus

**Goal:** Ensure existing core macros (`define!`, `bind!`) are stable and reliable, and establish a comprehensive testing and documentation system.

### 1. Improve Documentation (Documentation)

- [x] **API Documentation**: Add detailed Rustdoc comments to core structures like `Pattern`, `BindInput`, and `Keyword` to ensure readability on `docs.rs`.
- [x] **README Enhancement**: Integrate the latest README, add an `examples/` directory, and provide basic real-world examples (such as parsing simple structs and functions).
- [ ] **Error Reporting Optimization**: Optimize `syn::Error` generation to ensure that when DSL syntax errors occur (e.g., mismatched parentheses), users receive clear compiler error messages instead of internal panics.

### 2. Comprehensive Testing System (Testing)

- [x] **Unit Tests**:
  - [x] Cover edge cases in `inject_lookahead` (recursive Groups, consecutive Literals, etc.).
  - [x] Test the `Keyword` parser's ability to handle special symbols (`->`, `=>`, `<`) and custom keywords.
- [ ] **UI Tests (Compile-fail Tests)**:
  - [ ] **Integrate `trybuild`**.
  - [ ] Write "negative test cases": Verify that the macro correctly intercepts and reports errors when input types do not match expectations (e.g., providing a `LitStr` when an `Ident` is expected).
- [x] **Integration Tests**:
  - [x] Simulate real-world scenarios to verify that structs generated by `define!` can correctly handle complex TokenStreams.

---

## 🚀 Phase 2: Architectural Innovation (v0.2.x) - Core Enhancements

**Goal:** Introduce advanced data structure mapping capabilities to solve "polymorphism" and "aggregation" issues in complex ASTs, enabling Vacro to handle complex syntax trees.

### 3. New Syntax Development (New Syntax)

#### A. Associative/Structural Capture

_Solves the "Array of Structs (AoS)" problem, i.e., capturing aggregated structures at once rather than scattered lists of fields._

- [ ] **Syntax Implementation**: Support `#(~name...: ...)` syntax to mark aggregated captures.
- [ ] **Tuple Support**: Implement `#(~items*: #(@:Type) #(@:Ident))` to generate `Vec<(Type, Ident)>`.
- [ ] **Struct Support**: Support internal named captures to generate lists of anonymous structs.

#### B. Polymorphic Capture (Enum Parsing)

_Solves the "Polymorphic Parsing" problem, i.e., a position can be one of multiple types._

- [x] **Syntax Implementation**: Support `#(name: EnumName { VariantA, VariantB })` syntax.
- [x] **Automatic Definition**: If `EnumName` is undefined, automatically generate an enum definition containing `VariantA(TypeA)`, `VariantB(TypeB)`.
- [x] **Branch Parsing**: Generate attempt-parsing logic based on `input.fork()` or `peek`, automatically handling backtracking on failure.

---

## 🛠️ Phase 3: Ecosystem & Tools (v0.3.x) - Developer Experience

**Goal:** Provide peripheral tools to lower the learning curve and debugging costs of Vacro.

### 4. Toolchain Development (Toolchain)

- [ ] Coming soon

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
