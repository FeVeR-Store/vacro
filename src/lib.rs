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

#[cfg(feature = "parser")]
#[doc_i18n]
pub mod parser {
    //! <div class="doc-en"> Declarative parsing tools. </div>
    //! <div class="doc-cn"> 声明式解析工具。 </div>
    pub use vacro_parser::*;
}

#[cfg(feature = "report")]
#[doc_i18n]
pub mod report {
    //! <div class="doc-en"> Diagnostic reporting tools. </div>
    //! <div class="doc-cn"> 诊断报告工具。 </div>
    #[doc(hidden)]
    pub use vacro_report::__private;
    pub use vacro_report::*;
}

#[cfg(feature = "trace")]
#[doc_i18n]
pub mod trace {
    //! <div class="doc-en"> Observability and tracing tools. </div>
    //! <div class="doc-cn"> 可观测性追踪工具 </div>
    pub use vacro_trace::*;
}

#[doc_i18n]
/// <div class="doc-cn"> Vacro 常用功能的预导入模块。</div>
/// <div class="doc-en"> A prelude for convenient access to commonly used Vacro features. </div>
///
/// <div class="doc-cn">
///
/// 使用方式：
/// ```rust
/// use vacro::prelude::*;
/// ```
/// </div>
///
/// <div class="doc-en">
///
/// Usage:
/// ```rust
/// use vacro::prelude::*;
/// ```
/// </div>
pub mod prelude {
    #[cfg(feature = "parser")]
    pub use crate::parser::{bind, define};

    #[cfg(feature = "report")]
    pub use crate::report::scope as report_scope;

    #[cfg(feature = "trace")]
    pub use crate::trace::{debug, error, info, instrument, snapshot, trace, warn};
}

// Re-export specific macros at root level for backward compatibility or ease of use
#[cfg(feature = "parser")]
pub use parser::{bind, define};

#[cfg(feature = "trace")]
pub use trace::snapshot;
