# Vacro

<div align="center">

**The Progressive DevX Framework for Rust Procedural Macros**

[<img alt="github" src="https://img.shields.io/badge/github-FeVeR_Store/vacro-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/FeVeR-Store/vacro)
[<img alt="crates.io" src="https://img.shields.io/crates/v/vacro.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/vacro)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-vacro-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/vacro)

[English](./README.md) | [简体中文](./README_CN.md)

</div>

---

## The Philosophy

Writing procedural macros in Rust shouldn't be a nightmare filled with boilerplate code and black-box debugging.

**Vacro** has evolved from a simple parsing library into a complete toolchain designed to improve the **Developer Experience (DevX)** across the entire macro lifecycle:

1.  **Parsing**: Write parsing logic in a declarative way.
2.  **Debugging**: Visualize the parsing path to see exactly what happens inside the macro.
3.  **Reporting**: Easily generate elegant and precise compiler error messages.

## The Ecosystem

Vacro is designed as a modular framework. You can use the fully-featured `vacro` entry point or pick specific underlying components as needed.

| Feature | Crate | Description |
| :--- | :--- | :--- |
| **Parsing** | [`vacro-parser`](./crates/vacro-parser) | **Declarative Parsing.** A DSL similar to `macro_rules!` that automatically implements `syn::Parse`. |
| **Debugging** | [`vacro-trace`](./crates/vacro-trace) | **Visual Tracing.** Captures snapshots and logs to solve complex grammar debugging issues. |
| **Visualization** | [`vacro-cli`](./crates/vacro-cli) | **TUI Tool.** A terminal interface to inspect traces and diff snapshots captured by `vacro-trace`. |
| **Diagnostics** | [`vacro-report`](./crates/vacro-report) | **Error Reporting.** Simplifies the construction and emission of diagnostic messages in proc-macros. |

## Quick Start

Add `vacro` to your `Cargo.toml` and enable the DevX features you need:

```toml
[dependencies]
vacro = { version = "0.2", features = ["full"] }
```

### 1. Declarative Parsing (`vacro-parser`)

Define your macro's input grammar like writing regex:

```rust
use vacro::prelude::*;

// Define syntax: "fn" <name> "(" <args> ")"
vacro::define!(MyMacroInput:
    fn
    #(name: syn::Ident)
    ( #(args*[,]: syn::Type) )
);
```

See more: [vacro-parser](https://docs.rs/vacro-parser)

### 2. Visual Debugging (`vacro-trace`)

Take snapshots of your TokenStream to see how it evolves. View the diffs in `vacro-cli`.

```rust
use vacro::prelude::*;

// Capture a snapshot with a tag.
// If called multiple times with the same tag, vacro-cli will show the diff.
vacro::snapshot!("expand", tokens);
```

See more: [vacro-trace](https://docs.rs/vacro-trace)

### 3. Diagnostic Reporting (`vacro-report`)

Provides superior error reporting capabilities, saying goodbye to generic `unexpected identifier` errors.

```rust
use vacro::prelude::*;

#[vacro::report::scope]
fn my_macro_impl(input: TokenStream) -> TokenStream {
    // If this fails (e.g., invalid syntax constructed),
    // Vacro will catch it and emit a precise error pointing to the code.
    let f: ItemFn = parse_quote!( fn foo () { >>invalid<< } );
    quote!(#f)
}
```

See more: [vacro-report](https://docs.rs/vacro-report)

### 4. Visualization Tool (`vacro-cli`)

Install and run the TUI tool to view trace data and snapshot diffs.

```bash
cargo install vacro-cli
# 1. Run tests to generate trace data
cargo test
# 2. Launch the visualizer
cargo vacro

```

Run the following test, then open the CLI to inspect the captured logs and snapshot evolution:

```rust
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
<img src="https://github.com/user-attachments/assets/7ae8261e-3959-42a4-92a6-fd212db86f0d" width="100%" alt="Vacro CLI Demo">
</div>

See more: [vacro-cli](https://crates.io/crates/vacro-cli)

## Roadmap

We are currently in active development, transitioning towards a DevX Framework.

- [x] **Documentation**: Multi-language support (`vacro-doc-i18n`).
- [x] **Parsing**: Core DSL implementation (`vacro-parser`).
- [x] **Diagnostics**: Error reporting integration (`vacro-report`).
- [x] **Debugging**: Implementation of `vacro-trace` and `vacro-cli`.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
