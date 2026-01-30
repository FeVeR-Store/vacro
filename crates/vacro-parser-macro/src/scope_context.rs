use std::cell::RefCell;
use syn::Ident;

thread_local! {
    static SCOPE_IDENT: RefCell<Option<Ident>> = const { RefCell::new(None) };
}

/// Sets the current scope identifier.
pub fn set_scope_ident(ident: Option<Ident>) {
    SCOPE_IDENT.with(|f| *f.borrow_mut() = ident);
}

/// Gets the current scope identifier, if any.
pub fn get_scope_ident() -> Option<Ident> {
    SCOPE_IDENT.with(|f| f.borrow().clone())
}
