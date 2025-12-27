#[doc(hidden)]
pub mod __private;

#[cfg(feature = "macros")]
pub use vacro_trace_macro::*;

#[cfg(feature = "macros")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
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
