pub(crate) mod cargo;
pub(crate) mod constant;
pub(crate) mod error;
pub(crate) mod model;
pub(crate) mod states;
pub(crate) mod utils;

pub use quote::quote;
use rust_format::Formatter;

use crate::__private::{model::TraceEvent, utils::now};

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
        time: now(),
    };
    emit_event(&event);
}

pub fn log(level: String, message: String) {
    let event = TraceEvent::Log {
        level,
        message,
        time: now(),
    };
    emit_event(&event);
}

fn emit_event(event: &TraceEvent) {
    match &serde_json::to_string(event) {
        Ok(json) => {
            TraceSession::writeln(json);
        }
        Err(e) => {
            eprintln!("[Vacro Trace Error] Failed to serialize trace event: {}", e);
        }
    }
}
