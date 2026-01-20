use syn::Item;

mod capture;
mod input;
mod keyword;
mod pattern;

pub struct Compiler {
    pub shared_definition: Vec<Item>,
    pub scoped_definition: Vec<Item>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            shared_definition: vec![],
            scoped_definition: vec![],
        }
    }
}
