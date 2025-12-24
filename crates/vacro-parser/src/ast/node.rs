//! ### Pattern
//!
//! <div class="doc-cn">
//!
//! 此处定义了vacro解析的统一节点
//! `Pattern.kind`包含了解析的节点类型：
//! - `Literal`: 字面量，包含关键字与符号
//! - `Group`: 分组，对于`(...)`、`[...]`、`{...}`的解析
//! - `Capture`: 捕获
//!
//! `Pattern.span`包含了节点的源码位置, 用于提供更准确的报错位置
//! `Pattern.meta`包含了节点的语义信息, 用于提供更友好的错误信息
//!
//! </div>
//!

use proc_macro2::Span;

use crate::ast::{
    capture::{Capture, FieldDef},
    keyword::Keyword,
    meta::SemanticInfo,
};

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
    pub meta: Option<SemanticInfo>,
}

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub enum PatternKind {
    /// 字面量/关键字 (e.g. `fn`, `,`, `->`)
    Literal(Keyword),

    /// 分组 (e.g. `( ... )`)
    Group {
        delimiter: proc_macro2::Delimiter,
        children: Vec<Pattern>,
    },

    /// 捕获节点 (e.g. `#(name: Type)`)
    Capture(Capture),
}

impl Pattern {
    pub fn collect_captures(&self) -> Vec<FieldDef> {
        match &self.kind {
            PatternKind::Literal(_) => vec![],
            PatternKind::Group { children, .. } => {
                // Group (括号) 是透明的，直接透传
                children.iter().flat_map(|p| p.collect_captures()).collect()
            }
            PatternKind::Capture(cap) => cap.collect_captures(),
        }
    }
}
