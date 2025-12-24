#[cfg(feature = "parser")]
pub mod parser {
    pub use vacro_parser::bind;
    pub use vacro_parser::define;
}

#[cfg(feature = "report")]
pub mod report {
    pub use vacro_report::__private;
    pub use vacro_report::scope;
}

#[cfg(feature = "parser")]
pub use parser::*;
