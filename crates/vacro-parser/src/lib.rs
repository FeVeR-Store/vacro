#![warn(missing_docs)]

//!<div class="doc-cn">
//!
#![doc = include_str!("docs/zh_cn.md")]
//!
//!</div>
//!
//! <div class="doc-en">
//!
#![doc = include_str!("docs/en.md")]
//!
//!</div>

#[doc(hidden)]
pub mod __private;

pub use vacro_parser_macro::bind;

pub use vacro_parser_macro::define;
