use proc_macro2::TokenStream;
use quote::quote;

#[inline]
pub fn crate_name() -> TokenStream {
    if cfg!(feature = "standalone") {
        quote! {::vacro_trace}
    } else {
        quote! {::vacro::trace}
    }
}
