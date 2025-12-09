# Vacro

**Making Rust Procedural Macro Development Simple Again: A Declarative Parsing Library**

[](https://www.google.com/search?q=https://crates.io/crates/vacro)
[](https://www.google.com/search?q=https://docs.rs/vacro)
[](https://www.google.com/search?q=LICENSE)

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
define!(MyFn: 
    fn                                    // Match literal
    #(?: <#(generic*[,]: GenericParam)>)  // Optional generic param list (angle brackets + comma separated)
    #(name: Ident)                        // Named capture for function name
    ( #(args*[,]: FnArg) )                // Argument list (parentheses + comma separated)
    #(?: -> #(ret: Type))                 // Optional return value
);
```

If written in a single line:

```rust
define!(MyFn: fn #(?: <#(generic*[,]: GenericParam)>) #(name: Ident) (#(args*[,]: FnArg)) #(?: -> #(ret: Type)));
```

One line of code covers all complex parsing logic.

## Core Macros

Vacro provides two core macros for **defining structs** and **on-the-fly parsing**, respectively.

### 1\. `define!`: Define Parsing Structs

If you need to define a reusable AST node (i.e., define a `struct` and automatically implement `syn::parse::Parse`), use `define!`.

```rust
use syn::{Ident, Type, FnArg, GenericParam, Token, parse_macro_input};
use vacro::define;

// Define a struct named MyFn, it automatically implements the Parse trait
define!(MyFn: 
    fn 
    #(?: <#(generic*[,]: GenericParam)>)
    #(name: Ident) 
    ( #(args*[,]: FnArg) ) 
    #(?: -> #(ret: Type))
);

// Usage
let my_fn: MyFn = parse_macro_input!(input as MyFn);
println!("Function name: {}", my_fn.name);
```

### 2\. `capture!`: On-the-fly Stream Parsing

If you want to quickly consume a segment of a `TokenStream` within existing parsing logic, use `capture!`.

#### Named Capture

If the pattern uses the form `name: Type`, the macro generates a struct named `Output` containing all fields.

```rust
let captured = capture!(input -> 
    fn #(name: Ident) #(?: -> #(ret: Type))
)?;

// Access fields
captured.name; // Ident
captured.ret;  // Option<Type>
```

#### Inline Capture

If no name is specified in the pattern (or it contains only anonymous captures), the macro will return a tuple or a single value.

```rust
// Parse types only, no names needed
let (ident, ty) = capture!(input -> #(@:Ident): #(@:Type))?;

// Access fields
ident; // Ident
ty;    // Type
```

## Syntax Reference

Vacro's DSL design intuition comes from `macro_rules!` and regular expressions.

| Syntax | Type | Description | Result Type | Example |
| :--- | :--- | :--- | :--- | :--- |
| `literal` | Literal | Matches and consumes a Token (Rust keywords/symbols like `fn`, `->` or custom ones like `miku`, `<>`) | `!` | `fn`, `->`, `miku`, `<>` |
| `#(x: T)` | Named Capture | Captures a specific `syn` type | `T` (e.g. `Ident`, `Type`) | `#(name: Ident)` |
| `#(x?: T)` | Named Optional | Attempts to parse; skips if failed | `Option<T>` | `#(name?: Ident)` |
| `#(x*[sep]: T)` | Named Iter | Similar to `Punctuated`, parses by separator | `Punctuated<T, sep>` | `#(args*: Ident)` |
| `#(T)` | Anonymous | Captures a specific `syn` type, but for validation only | `!` | `#(Ident)` |
| `#(?: T)` | Anon Optional | Validation only; skips if failed | `!` | `#(?: Ident)` |
| `#(*[sep]: T)` | Anon Iter | Similar to `Punctuated`, parses by separator (validation only) | `!` | `#(*[,]: Ident)` |

---

# Vacro Roadmap

## 📅 Phase 1: Solidifying Foundations (v0.1.x) - Current Focus

**Goal:** Ensure existing core macros (`define!`, `capture!`) are stable and reliable, and establish a comprehensive testing and documentation system.

### 1. Improve Documentation (Documentation)

- [ ] **API Documentation**: Add detailed Rustdoc comments to core structures like `Pattern`, `CaptureInput`, and `Keyword` to ensure readability on `docs.rs`.
- [ ] **README Enhancement**: Integrate the latest README, add an `examples/` directory, and provide basic real-world examples (such as parsing simple structs and functions).
- [ ] **Error Reporting Optimization**: Optimize `syn::Error` generation to ensure that when DSL syntax errors occur (e.g., mismatched parentheses), users receive clear compiler error messages instead of internal panics.

### 2. Comprehensive Testing System (Testing)

- [ ] **Unit Tests**:
  - [ ] Cover edge cases in `inject_lookahead` (recursive Groups, consecutive Literals, etc.).
  - [ ] Test the `Keyword` parser's ability to handle special symbols (`->`, `=>`, `<`) and custom keywords.
- [ ] **UI Tests (Compile-fail Tests)**:
  - [ ] **Integrate `trybuild`**.
  - [ ] Write "negative test cases": Verify that the macro correctly intercepts and reports errors when input types do not match expectations (e.g., providing a `LitStr` when an `Ident` is expected).
- [ ] **Integration Tests**:
  - [ ] Simulate real-world scenarios to verify that structs generated by `define!` can correctly handle complex TokenStreams.

---

## 🚀 Phase 2: Architectural Innovation (v0.2.x) - Core Enhancements

**Goal:** Introduce advanced data structure mapping capabilities to solve "polymorphism" and "aggregation" issues in complex ASTs, enabling Vacro to handle complex syntax trees.

### 3. New Syntax Development (New Syntax)

#### A. Associative/Structural Capture

*Solves the "Array of Structs (AoS)" problem, i.e., capturing aggregated structures at once rather than scattered lists of fields.*

- [ ] **Syntax Implementation**: Support `#(~name...: ...)` syntax to mark aggregated captures.
- [ ] **Tuple Support**: Implement `#(~items*: #(@:Type) #(@:Ident))` to generate `Vec<(Type, Ident)>`.
- [ ] **Struct Support**: Support internal named captures to generate lists of anonymous structs.

#### B. Polymorphic Capture (Enum Parsing)

*Solves the "Polymorphic Parsing" problem, i.e., a position can be one of multiple types.*

- [ ] **Syntax Implementation**: Support `#(name: EnumName { VariantA, VariantB })` syntax.
- [ ] **Automatic Definition**: If `EnumName` is undefined, automatically generate an enum definition containing `VariantA(TypeA)`, `VariantB(TypeB)`.
- [ ] **Branch Parsing**: Generate attempt-parsing logic based on `input.fork()` or `peek`, automatically handling backtracking on failure.

---

## 🛠️ Phase 3: Ecosystem & Tools (v0.3.x) - Developer Experience

**Goal:** Provide peripheral tools to lower the learning curve and debugging costs of Vacro.

### 4. Toolchain Development (Toolchain)

- [ ] Coming soon

---

## License

MIT
