use std::time::{SystemTime, UNIX_EPOCH};

use quote::format_ident;
use syn::{Ident, Item};

mod capture;
mod input;
mod keyword;
mod pattern;

pub struct Compiler {
    pub shared_definition: Vec<Item>,
    pub scoped_definition: Vec<Item>,
    pub target: Ident,
}

impl Compiler {
    pub fn new() -> Self {
        let now = SystemTime::now();
        let duration_since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let timestamp = duration_since_epoch.as_secs();
        Self {
            shared_definition: vec![],
            scoped_definition: vec![],
            target: format_ident!("_{timestamp}"),
        }
    }
}
