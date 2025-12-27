use serde_json::json;

use crate::__private::cargo::metadata;
use crate::__private::constant::{self, MACRO_EXPAND};
use crate::__private::model::TraceEvent;
use crate::__private::utils::now;
use std::cell::RefCell;
use std::fs::{self, File};
use std::io::{BufWriter, Write};

thread_local! {
    static WRITER: RefCell<Option<BufWriter<File>>> = const { RefCell::new(None) };
    static CURRENT_CONTEXT: RefCell<Option<TraceSession>> = const { RefCell::new(None) };
}

fn create_writer(session: &TraceSession) -> Option<BufWriter<File>> {
    let target_directory = if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        std::path::PathBuf::from(dir)
    } else {
        match metadata() {
            Ok(metadata) => metadata.target_directory,
            Err(e) => {
                eprintln!("[Vacro Trace Warning] Failed to get metadata: {}", e);
                std::env::current_dir().ok()?.join("target")
            }
        }
    };
    let vacro_directory = target_directory.join("vacro");
    if let Err(e) = fs::create_dir_all(&vacro_directory) {
        eprintln!(
            "[Vacro Trace Warning] Failed to create vacro directory: {}",
            e
        );
        return None;
    }
    let trace_file = vacro_directory.join(format!("trace-{}.jsonl", session.id));
    match File::create(trace_file) {
        Ok(f) => Some(BufWriter::new(f)),
        Err(e) => {
            eprintln!("[Vacro Trace Error] Failed to create log file: {}", e);
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceSession {
    pub id: String,         // UUID，每一次宏展开都是一个 Session
    pub macro_name: String, // 例如 "my_macro"
    pub crate_name: String, // 调用宏的 crate
    pub timestamp: u64,
}

#[allow(dead_code)]
impl TraceSession {
    pub fn enter(macro_name: &str, crate_name: &str) -> SessionGuard {
        let mut session = Self::new();
        session.macro_name = macro_name.to_string();
        session.crate_name = crate_name.to_string();
        CURRENT_CONTEXT.with(|ctx| *ctx.borrow_mut() = Some(session));
        let event = TraceEvent::PhaseStart {
            name: MACRO_EXPAND.to_string(),
            time: now(),
        };
        Self::emit(&event);
        SessionGuard
    }
    pub fn new() -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let macro_name = String::new();

        let crate_name =
            std::env::var(constant::CARGO_CRATE_NAME).unwrap_or_else(|_| "unknown".to_string());
        let timestamp = now();

        Self {
            id,
            macro_name,
            crate_name,
            timestamp,
        }
    }
    pub fn macro_name(macro_name: &str) {
        CURRENT_CONTEXT.with(|ctx| {
            let mut borrow = ctx.borrow_mut();
            if let Some(ctx) = borrow.as_mut() {
                ctx.macro_name = macro_name.to_string();
            }
        });
    }
    pub fn crate_name(crate_name: &str) {
        CURRENT_CONTEXT.with(|ctx| {
            let mut borrow = ctx.borrow_mut();
            if let Some(ctx) = borrow.as_mut() {
                ctx.crate_name = crate_name.to_string();
            }
        });
    }
    pub fn get_session() -> Option<TraceSession> {
        CURRENT_CONTEXT.with(|ctx| ctx.borrow().clone())
    }
    pub fn emit(event: &TraceEvent) {
        if let Some(session) = Self::get_session() {
            WRITER.with(|cell| {
                let mut borrow = cell.borrow_mut();
                // Lazy Init: Only create file when we actually have something to write
                if borrow.is_none() {
                    *borrow = create_writer(&session);
                }

                let msg = json!({
                    "id": session.id,
                    "macro_name": session.macro_name,
                    "crate_name": session.crate_name,
                    "timestamp": session.timestamp,
                    "message": event
                });

                if let Some(writer) = borrow.as_mut() {
                    if let Err(e) = writeln!(writer, "{}", msg.to_string()) {
                        eprintln!("[Vacro Trace Error] Failed to write to log: {}", e);
                    }
                }
            });
        } else {
            eprintln!("[Vacro Trace Warning] writeln called but no session found.");
        }
    }
}

impl Default for TraceSession {
    fn default() -> Self {
        TraceSession::new()
    }
}

pub struct SessionGuard;

impl Drop for SessionGuard {
    fn drop(&mut self) {
        let event = TraceEvent::PhaseEnd {
            name: MACRO_EXPAND.to_string(),
            time: now(),
        };
        TraceSession::emit(&event);
        CURRENT_CONTEXT.with(|ctx| *ctx.borrow_mut() = None);
        WRITER.with(|cell| {
            if let Ok(mut borrow) = cell.try_borrow_mut() {
                // Take the writer out of the Option, leaving None behind.
                // This ensures the next session will create a FRESH writer/file.
                if let Some(mut writer) = borrow.take() {
                    if let Err(e) = writer.flush() {
                        eprintln!("[Vacro Trace Warning] Failed to flush trace log: {}", e);
                    }
                    // writer is dropped here, closing the file handle.
                }
            }
        });
    }
}
