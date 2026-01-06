use rust_format::{Formatter, PrettyPlease};

pub use quote::quote;
pub use vacro_report_macro::parse_quote;

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
                "`parse_quote` failed at {}:{}:{}\n\
                Tokens:\n\
                ```\n\
                {}\n\
                ```\n\
                Error message: \"{}\"",
                loc.file(),
                loc.line(),
                loc.column(),
                actual,
                e,
            );
        }
        Err(panic) => {
            let loc = std::panic::Location::caller();
            panic!(
                "`parse_quote` panicked at {}:{}:{}\n\
                Tokens:\n\
                ```\n\
                {}\n\
                ```\n\
                Error message: \"{:?}\"",
                loc.file(),
                loc.line(),
                loc.column(),
                actual,
                panic,
            );
        }
    }
}
