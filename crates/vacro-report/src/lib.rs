//! <div class="doc-en">
//!
#![doc = include_str!("docs/en.md")]
//! </div>
//!
//! <div class="doc-cn">
//!
#![doc = include_str!("docs/zh_cn.md")]
//!
//! </div>

#[doc(hidden)]
pub mod __private;

pub use vacro_report_macro::scope;
