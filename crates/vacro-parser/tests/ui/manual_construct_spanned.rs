use quote::quote;
use syn::{parse2, Ident};
use vacro_parser::define;

define!(ManualConstructed:
    #(name: Ident)
);

fn main() {
    let parsed: ManualConstructed = parse2(quote!(value)).unwrap();
    let _ = parsed.span();

    let name = syn::parse_str::<Ident>("value").unwrap();
    let _manual = ManualConstructed { name };
}
