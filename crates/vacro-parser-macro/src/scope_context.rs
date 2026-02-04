use std::{
    cell::RefCell,
    sync::atomic::{AtomicUsize, Ordering},
};
use syn::Ident;

thread_local! {
    static SCOPE_IDENT: RefCell<Option<Ident>> = const { RefCell::new(None) };
    static INLINE_COUNTER: AtomicUsize = const { AtomicUsize::new(0) };
}

pub fn next_inline_index() -> usize {
    INLINE_COUNTER.with(|i| i.fetch_add(1, Ordering::Relaxed))
}

#[cfg(test)]
pub fn reset_inline_counter() {
    INLINE_COUNTER.with(|i| i.store(0, Ordering::SeqCst))
}

/// Sets the current scope identifier.
pub fn set_scope_ident(ident: Option<Ident>) {
    SCOPE_IDENT.with(|f| *f.borrow_mut() = ident);
}

/// Gets the current scope identifier, if any.
pub fn get_scope_ident() -> Option<Ident> {
    SCOPE_IDENT.with(|f| f.borrow().clone())
}
