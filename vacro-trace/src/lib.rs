use rust_format::{Formatter, PrettyPlease};

#[track_caller]
pub fn parse_quote_traced<T: syn::parse::Parse>(tokens: proc_macro2::TokenStream) -> T {
    let actual = PrettyPlease::default()
        .format_tokens(tokens.clone())
        .unwrap_or(tokens.to_string());
    match std::panic::catch_unwind(|| syn::parse2(tokens)) {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => {
            let loc = std::panic::Location::caller();
            panic!(
                r#"`parse_quote` failed at {}:{}:{}
                Error message: "{}"
                Tokens:
                {}"#,
                loc.file(),
                loc.line(),
                loc.column(),
                e,
                actual
            );
        }
        Err(panic) => {
            let loc = std::panic::Location::caller();
            panic!(
                r#"`parse_quote` panicked at {}:{}:{}
                Error message: "{:?}"
                Tokens:
                {}"#,
                loc.file(),
                loc.line(),
                loc.column(),
                panic,
                actual
            );
        }
    }
}
