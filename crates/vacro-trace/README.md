# Vacro Trace

**Observability for Rust Procedural Macros**

[![crates.io](https://img.shields.io/crates/v/vacro-trace.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/vacro-trace)
[![docs.rs](https://img.shields.io/badge/docs.rs-vacro--trace-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs)](https://docs.rs/vacro-trace)

## Introduction

`vacro-trace` brings familiar observability tools (logging, tracing, snapshots) to the world of Procedural Macro development.

It acts as the **capture layer**, designed to work hand-in-hand with **`vacro-cli`** (the visualization layer). While `vacro-trace` records the data, `vacro-cli` is required to view logs and inspect snapshot diffs.

## Installation

```toml
[dependencies]
vacro-trace = "0.1.2"
```

## Usage

### 1. Instrumentation

The macro entry needs to be marked with `#[instrument]`.

```rust
#[instrument]
#[proc_macro]
fn parse_impl(input: proc_macro2::TokenStream) {
    // ...
}
```

### 2. Snapshots & Diffing

Use `snapshot!(tag, tokens)` to capture the state of tokens at a specific point.

If you take multiple snapshots with the **same tag** (e.g., "transformation"), `vacro-cli` will automatically generate a diff view, showing how the tokens evolved.

```rust
let mut tokens = quote! { fn hello() {} };
// Initial state
snapshot!("my_macro", tokens);

// ... modify tokens ...
tokens = quote! { fn hello() { println!("world"); } };

// Final state - vacro-cli will show the diff between these two snapshots
snapshot!("my_macro", tokens);
```

### 3. Logging

```rust
info!("Start expanding macro...");
warn!("Something looks suspicious: {}", "ident_name");
```
