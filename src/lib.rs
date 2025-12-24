#[cfg(feature = "parser")]
pub mod parser {
    pub use vacro_parser::bind;
    pub use vacro_parser::define;
}

#[cfg(feature = "parser")]
pub use parser::*;
