use std::time::SystemTime;
pub(crate) mod cargo;
pub(crate) mod error;
pub(crate) mod model;
pub(crate) mod states;

pub use quote::quote;
use rust_format::Formatter;

use crate::__private::model::TraceEvent;

pub use states::TraceSession;

fn fmt(tokens: String) -> String {
    rust_format::PrettyPlease::default()
        .format_str(&tokens)
        .unwrap_or(tokens)
}

pub fn snapshot(tag: &str, ast: String) {
    let formatted = fmt(ast);
    let event = TraceEvent::Snapshot {
        tag: tag.to_string(),
        code: formatted,
        time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };
    match &serde_json::to_string(&event) {
        Ok(json) => {
            TraceSession::writeln(json);
        }
        Err(e) => {
            eprintln!("Failed to serialize trace event: {}", e);
        }
    }
}
