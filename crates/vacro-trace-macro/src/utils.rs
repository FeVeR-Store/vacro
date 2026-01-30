use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};

pub fn resolve_crate_root() -> TokenStream {
    let found_vacro = crate_name("vacro");

    if let Ok(FoundCrate::Name(name)) = found_vacro {
        let import_root = format_ident!("{name}");
        return quote!( ::#import_root::trace );
    }

    let found_sub = crate_name("vacro-trace");

    if let Ok(FoundCrate::Name(name)) = found_sub {
        let import_root = format_ident!("{name}");
        return quote!( ::#import_root );
    }

    quote!(::vacro_trace)
}
