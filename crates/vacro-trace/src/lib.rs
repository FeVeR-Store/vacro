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
/// @cn 日志级别。
/// @en Log level.
pub enum Level {
    /// @cn 错误级别 (Error)。
    /// @en Error level.
    Error,
    /// @cn 警告级别 (Warn)。
    /// @en Warn level.
    Warn,
    /// @cn 信息级别 (Info)。
    /// @en Info level.
    Info,
    /// @cn 调试级别 (Debug)。
    /// @en Debug level.
    Debug,
    /// @cn 追踪级别 (Trace)。
    /// @en Trace level.
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
