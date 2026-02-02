# Vacro Parser

**The Declarative Parsing Kernel for Vacro**

## Introduction

**Vacro Parser** is the core declarative parsing engine of the Vacro framework. It provides a `macro_rules!`-like DSL to simplify the writing of `syn`-based parsers for Rust Procedural Macros.

It allows you to define AST structures and parsing logic declaratively, eliminating the boilerplate of imperative `input.parse()?` calls.

## Core Features

### 1. `define!`: Define Parsing Structs

Use `define!` to define a struct that automatically implements `syn::parse::Parse`.

```rust
# use syn::{Ident, Type, GenericParam, FnArg, parse_quote, Token};
# use vacro_parser::define;
// Define a struct named MyFn, it automatically implements the Parse trait
define!(MyFn:
    fn
    #(?: <#(generic*[,]: GenericParam)>)
    #(name: Ident)
    ( #(args*[,]: FnArg) )
    #(?: -> #(ret: Type))
);

fn parse_my_fn(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    // Usage
    let my_fn: MyFn = parse_quote!(input);
    println!("Function name: {}", my_fn.name);
    proc_macro2::TokenStream::new()
}
# fn main() {}
```

### 2. `bind!`: On-the-fly Parsing

Use `bind!` to consume a portion of a `TokenStream` within existing imperative logic.

```rust
# use syn::{Ident, Type, Token, Result};
# use vacro_parser::bind;
# fn parser(input: proc_macro2::TokenStream) -> Result<()> {
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
# Ok(())
# }
# fn main() {}
```

## Syntax Reference

| Syntax          | Description                                                   | Result Type          | Example              |
| :-------------- | :------------------------------------------------------------ | :------------------- | :------------------- |
| `literal`       | Matches and consumes exact tokens                             | `!`                  | `fn`, `->`, `struct` |
| `#(x: T)`       | **Named Capture**: Captures type `T` into field `x`           | `T`                  | `#(name: Ident)`     |
| `#(x?: T)`      | **Named Optional**: Attempts to parse; skips if failed        | `Option<T>`          | `#(ret?: Type)`      |
| `#(x*[sep]: T)` | **Named Iter**: Parses by separator                           | `Punctuated<T, sep>` | `#(args*[,]: FnArg)` |
| `#(T)`          | **Anonymous Match**: Validates `T` exists but doesn't capture | `!`                  | `#(Ident)`           |
| `#(?: T)`       | **Anonymous Optional**: Validation only                       | `!`                  | `#(?: Ident)`        |
| `#(*[sep]: T)`  | **Anonymous Iter**: Validation only                           | `!`                  | `#(*[,]: Ident)`     |

## Polymorphic Capture (Enum Parsing)

Vacro supports parsing "polymorphic" structures, where a position in the input stream can be one of multiple types.

```rust
# use syn::{Ident, Expr, Type, LitInt};
# use vacro_parser::define;
define!(MyPoly:
    #(data: MyEnum {
        Ident,                            // 1. Shorthand: Match Ident, produces MyEnum::Ident(Ident)
        syn::Type,                        // 2. Shorthand: Match Type, produces MyEnum::Type(syn::Type)
        Integer: syn::LitInt,             // 3. Alias: Match LitInt, produces MyEnum::Integer(LitInt)
        Function: fn #(name: Ident),      // 4. Pattern: Produces MyEnum::Function { name: Ident }
        Tuple: (#(@: Ident), #(@: Expr)), // 5. Pattern: Produces MyEnum::Tuple(Ident, Expr)
    })
);
# fn main() {}
```

## More user-friendly prompts (v0.1.6)

You can use the `help!` macro of `vacro-report` to provide more helpful suggestions for the content. If you are using `vacro`, you only need to enable the `report` feature.

```toml
vacro_parser = { version: "0.1.7" }
vacro_report = { version: "0.1.3", features: ["parser"] }

# vacro = { version: "0.2.2", features: ["parser", "report"] }
```

```rust,ignore
use vacro_parser::define;
use vacro_report::help;
# use syn::{Ident, LitBool};

help!(Bool:
    LitBool {
        error: "A boolean literal is required here; the received value is: {input}",
        help: "Try `true` or `false`",
        example: (true | false) // The example field is the sample field to be displayed, used when generating error messages and usage examples; it accepts a TokenStream and will directly display the content you pass in.
    }
);

define!(MyRoles: {
    #(roles*[,]: #(pair: #(name: syn::Ident): #(enable: Bool)))
});

```

<div class="warning">

This example fails to compile. The associated capture syntax `#(pair: #(name: Ident): #(enable: BoolLit))` used here will be implemented later; see [Associated Captures](https://github.com/FeVeR-Store/vacro/issues/38)

</div>
