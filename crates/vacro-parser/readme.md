# Vacro Parser

**The Declarative Parsing Kernel for Vacro**

[![crates.io](https://img.shields.io/crates/v/vacro-parser.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/vacro-parser)
[![docs.rs](https://img.shields.io/badge/docs.rs-vacro--parser-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs)](https://docs.rs/vacro-parser)

## Introduction

**Vacro Parser** is the core declarative parsing engine of the Vacro framework. It provides a `macro_rules!`-like DSL to simplify the writing of `syn`-based parsers for Rust Procedural Macros.

It allows you to define AST structures and parsing logic declaratively, eliminating the boilerplate of imperative `input.parse()?` calls.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vacro-parser = "0.1.7"
```

## Core Features

### 1. `define!`: Define Parsing Structs

Use `define!` to define a struct that automatically implements `syn::parse::Parse`.

```rust
use syn::{Ident, Type, GenericParam, FnArg, parse_quote};

// Define a struct named MyFn, it automatically implements the Parse trait
vacro::define!(MyFn:
    fn
    #(?: <#(generic*[,]: GenericParam)>)
    #(name: Ident)
    ( #(args*[,]: FnArg) )
    #(?: -> #(ret: Type))
);

// Usage in a proc-macro
fn parse_my_fn(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let my_fn: MyFn = parse_quote!(input);
    println!("Function name: {}", my_fn.name);
    proc_macro2::TokenStream::new()
}
```

### 2. `bind!`: On-the-fly Parsing

Use `bind!` to consume a portion of a `TokenStream` within existing imperative logic.

```rust
use syn::{Ident, Type, Token};
use vacro::bind;

fn parser(input: syn::parse::ParseStream) -> syn::Result<()> {
    // Parse a function signature pattern on the fly
    bind!(
        let captured = (input ->
            fn #(name: Ident) #(?: -> #(ret: Type))
        )?;
    );

    // Access captured fields directly
    println!("Name: {}", captured.name);
    if let Some(ret_type) = captured.ret {
        // ...
    }
    Ok(())
}
```

## Syntax Reference

| Syntax          | Description                                                   | Example              |
| :-------------- | :------------------------------------------------------------ | :------------------- |
| `literal`       | Matches exact tokens                                          | `fn`, `->`, `struct` |
| `#(x: T)`       | **Named Capture**: Captures type `T` into field `x`           | `#(name: Ident)`     |
| `#(x?: T)`      | **Optional Capture**: `Option<T>`                             | `#(ret?: Type)`      |
| `#(x*[sep]: T)` | **Iterative Capture**: `Punctuated<T, sep>`                   | `#(args*[,]: FnArg)` |
| `#(T)`          | **Anonymous Match**: Validates `T` exists but doesn't capture | `#(Ident)`           |

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

## More user-friendly prompts (v0.1.6)

You can use the `help!` macro of `vacro-report` to provide more helpful suggestions for the content. If you are using `vacro`, you only need to enable the `report` feature.

```toml
vacro = { version: "0.2.2", features: ["parser", "report"] }
```

```rust
use vacro::{help, define};
use syn::{Ident, LitBool};

help! (Bool:
    LitBool {
        error: "A boolean literal is required here; the received value is: {input}".
        help: "Try `true` or `false`",
        example: (true | false) // The example field is the sample field to be displayed, used when generating error messages and usage examples; it accepts a TokenStream and will directly display the content you pass in.
    }
)

define!(MyRoles: {
    #(roles*[,]: #(pair: #(name: Ident): #(enable: Bool)))
});

```

> ⚠️ Warning
>
> This example fails to compile. The associated capture syntax `#(pair: #(name: Ident): #(enable: BoolLit))` used here will be implemented later; see [Associated Captures](https://github.com/FeVeR-Store/vacro/issues/38)
