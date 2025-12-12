use std::sync::{Arc, Mutex};

use proc_macro2::Delimiter;
use syn::{Ident, Type};

use crate::{
    ast::{capture::CaptureSpec, keyword::Keyword},
    syntax::context::ParseContext,
};

pub type IsOptional = bool;

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub struct PatternList {
    pub list: Vec<Pattern>,
    pub capture_list: Arc<Mutex<Vec<(Ident, Type, IsOptional)>>>,
    pub parse_context: ParseContext,
}

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub enum Pattern {
    // 关键字/符号：fn, struct, ;
    Literal(Keyword),
    // 括号组：( ... ), { ... }
    Group(Delimiter, PatternList),
    // 捕获：#( ... )
    Capture(CaptureSpec, Option<Keyword>),
}
