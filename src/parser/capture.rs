use syn::{Ident, Type};

use crate::ast::{keyword::Keyword, pattern::PatternList};

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub enum CaptureType {
    Type(Type),
    Joint(PatternList),
}

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub enum ExposeMode {
    Inline(usize),
    Named(Ident),
    Anonymous,
}

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub struct CaptureSpec {
    pub name: ExposeMode,  // 暴露模式
    pub ty: CaptureType,   // 类型
    pub mode: CaptureMode, // Once, Optional, Iter
}

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub enum CaptureMode {
    Once,
    Optional,
    Iter(Keyword),
}
