# Vacro Trace

**Observability for Rust Procedural Macros**

## Introduction

`vacro-trace` brings familiar observability tools (logging, tracing, snapshots) to the world of Procedural Macro development.

It acts as the **capture layer**, designed to work hand-in-hand with **`vacro-cli`** (the visualization layer). While `vacro-trace` records the data, `vacro-cli` is required to view logs and inspect snapshot diffs.

## Features

- **Structured Logging**: `error!`, `warn!`, `info!`, `debug!`, `trace!` macros.
- **Token Snapshots**: Capture `TokenStream` states with tags. Same-tag snapshots are automatically diffed in `vacro-cli`.
- **Auto Instrumentation**: `#[instrument]` attribute to automatically trace function calls.

## Usage

### 1. Instrumentation

The macro entry needs to be marked with `#[instrument]`.

```rust,ignore
# use vacro_trace::instrument;
#[instrument]
#[proc_macro]
fn parse_impl(input: proc_macro2::TokenStream) {
    // ...
}
# fn main() {}
```

### 2. Snapshots & Diffing

Use `snapshot!(tag, tokens)` to capture the state of tokens at a specific point.

If you take multiple snapshots with the **same tag** (e.g., "transformation"), `vacro-cli` will automatically generate a diff view, showing how the tokens evolved.

```rust
# use vacro_trace::snapshot;
# use quote::quote;
# fn main() {
let mut tokens = quote! { fn hello() {} };
// Initial state
snapshot!("my_macro", tokens);

// ... modify tokens ...
tokens = quote! { fn hello() { println!("world"); } };

// Final state - vacro-cli will show the diff between these two snapshots
snapshot!("my_macro", tokens);
# }
```

### 3. Logging

```rust
# use vacro_trace::{info, warn};
# fn main() {
info!("Start expanding macro...");
warn!("Something looks suspicious: {}", "ident_name");
# }
```

## Viewing Results

To view the captured data:

1. Install the CLI: `cargo install vacro-cli` (or build from source).
2. Run your build: `cargo build`.
3. Open the TUI: `vacro-cli`.
