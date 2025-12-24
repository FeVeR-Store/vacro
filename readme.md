# Vacro

<div align="center">

**The Progressive DevX Framework for Rust Procedural Macros**

[<img alt="github" src="https://img.shields.io/badge/github-FeVeR_Store/vacro-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/FeVeR-Store/vacro)
[<img alt="crates.io" src="https://img.shields.io/crates/v/vacro.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/vacro)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-vacro-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/vacro)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/FeVeR-Store/vacro/publish.yml?style=for-the-badge" height="20">](https://github.com/FeVeR-Store/vacro/actions/workflows/publish.yml)

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

| Feature         | Crate            | Description                                                                                                   |
| :-------------- | :--------------- | :------------------------------------------------------------------------------------------------------------ |
| **Parsing**     | `vacro-parser`   | **Declarative Parsing.** A DSL similar to `macro_rules!` that automatically implements `syn::Parse`.          |
| **Debugging**   | `vacro-trace`    | **Visual Tracing.** Generates a parsing state tree in the terminal to solve complex grammar debugging issues. |
| **Diagnostics** | `vacro-report`   | **Error Reporting.** Simplifies the construction and emission of diagnostic messages in proc-macros.          |
| **Docs**        | `vacro-doc-i18n` | **I18n Docs.** Provides multi-language documentation support for `Vacro`.                                     |

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

### 2. Diagnostic Reporting (`vacro-report`)

Provides superior error reporting capabilities, saying goodbye to generic `unexpected identifier` errors.

```rust
use vacro::prelude::*;

#[vacro::report::scope]
fn my_macro_impl(input: TokenStream) -> TokenStream {
    let name = Ident::new("foo", Span::call_site());

    // If this fails (e.g., invalid syntax constructed),
    // instead of a raw panic like "proc-macro panicked: parse_quote failed at...",
    // Vacro will catch it and emit a precise error pointing to the code.
    // Error message: "expected an expression"
    // Tokens context will be shown.
    let f: ItemFn = parse_quote!( fn #name () { >>invalid<< } );

    quote!(#f)
}

```

## Roadmap

We are currently in active development, transitioning towards a DevX Framework.

- [x] **Documentation**: Multi-language support (`vacro-doc-i18n`).
- [x] **Parsing**: Core DSL implementation (`vacro-parser`).
- [x] **Diagnostics**: Error reporting integration (`vacro-report`).
- [ ] **Debugging**: Implementation of `vacro-trace`.

## Contributing

We are building the best developer experience for Rust metaprogramming. If you have ideas on how to reduce the pain of writing macros, please open an issue!

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
