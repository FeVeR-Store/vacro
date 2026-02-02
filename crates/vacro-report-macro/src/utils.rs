use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
#[cfg(feature = "parser")]
use quote::quote;
use quote::{format_ident, quote_spanned};

pub fn resolve_crate_root(span: Span) -> TokenStream {
    let found_vacro = crate_name("vacro");

    if let Ok(FoundCrate::Name(name)) = found_vacro {
        let import_root = format_ident!("{name}");
        return quote_spanned!(span => ::#import_root::report );
    }

    let found_sub = crate_name("vacro-report");

    if let Ok(FoundCrate::Name(name)) = found_sub {
        let import_root = format_ident!("{name}");
        return quote_spanned!(span => ::#import_root );
    }

    if std::env::var("CARGO_PKG_NAME").unwrap_or_default() == "vacro-report" {
        return quote_spanned!(span => ::vacro_report);
    }

    quote_spanned!(span => ::vacro_report)
}

#[cfg(feature = "parser")]
pub fn resolve_vacro_parser_root() -> TokenStream {
    let found_vacro = crate_name("vacro");

    if let Ok(FoundCrate::Name(name)) = found_vacro {
        let import_root = format_ident!("{name}");
        return quote!(::#import_root::parser );
    }

    let found_sub = crate_name("vacro-parser");

    if let Ok(FoundCrate::Name(name)) = found_sub {
        let import_root = format_ident!("{name}");
        return quote!(::#import_root );
    }

    if std::env::var("CARGO_PKG_NAME").unwrap_or_default() == "vacro-parser" {
        return quote!(::vacro_parser);
    }

    quote!(::vacro_parser)
}
