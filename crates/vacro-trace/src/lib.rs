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

use vacro_doc_i18n::doc_i18n;

#[doc(hidden)]
pub mod __private;

#[cfg(feature = "macros")]
pub use vacro_trace_macro::*;

#[cfg(feature = "macros")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc_i18n]
/// <div class="doc-cn"> 日志级别。 </div>
/// <div class="doc-en"> Log level. </div>
pub enum Level {
    /// <div class="doc-cn"> 错误级别 (Error)。 </div>
    /// <div class="doc-en"> Error level. </div>
    Error,
    /// <div class="doc-cn"> 警告级别 (Warn)。 </div>
    /// <div class="doc-en"> Warn level. </div>
    Warn,
    /// <div class="doc-cn"> 信息级别 (Info)。 </div>
    /// <div class="doc-en"> Info level. </div>
    Info,
    /// <div class="doc-cn"> 调试级别 (Debug)。 </div>
    /// <div class="doc-en"> Debug level. </div>
    Debug,
    /// <div class="doc-cn"> 追踪级别 (Trace)。 </div>
    /// <div class="doc-en"> Trace level. </div>
    Trace,
}

#[cfg(feature = "macros")]
impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Error => write!(f, "ERROR"),
            Level::Warn => write!(f, "WARN"),
            Level::Info => write!(f, "INFO"),
            Level::Debug => write!(f, "DEBUG"),
            Level::Trace => write!(f, "TRACE"),
        }
    }
}
