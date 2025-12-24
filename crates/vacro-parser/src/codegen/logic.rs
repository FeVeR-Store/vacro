use syn::Item;

mod capture;
mod input;
mod keyword;
mod pattern;

pub struct Compiler {
    pub definition: Vec<Item>,
}

impl Compiler {
    pub fn new() -> Self {
        Self { definition: vec![] }
    }
}
