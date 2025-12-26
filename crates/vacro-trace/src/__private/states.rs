use std::cell::RefCell;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::time::SystemTime;

use crate::__private::cargo::metadata;

thread_local! {
    static WRITER: RefCell<BufWriter<File>> = {
        let session = match TraceSession::get_session() {
            Some(session) => session,
            None => {
                eprintln!("[Vacro Trace Warning] Failed to get session");
                TraceSession::new()
            }
        };
        let target_directory = match metadata() {
            Ok(metadata) => metadata.target_directory,
            Err(e) => {
                eprintln!("[Vacro Trace Warning] Failed to get metadata: {}", e);
                std::env::current_dir().unwrap().join("target")
            }
        };
        let vacro_directory = target_directory.join("vacro");
        if let Err(e) = fs::create_dir_all(&vacro_directory) {
            eprintln!("[Vacro Trace Warning] Failed to create vacro directory: {}", e);
        }
        let trace_file = vacro_directory.join(format!("trace-{}.jsonl", session.id));
        let writer = BufWriter::new(File::create(trace_file).unwrap());
        RefCell::new(writer)
    };
    static CURRENT_CONTEXT: RefCell<Option<TraceSession>> = RefCell::new(None);
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
    pub fn enter() -> SessionGuard {
        let session = Self::new();
        CURRENT_CONTEXT.with(|ctx| *ctx.borrow_mut() = Some(session));
        SessionGuard
    }
    pub fn new() -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let macro_name = String::new();
        let crate_name = String::new();
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

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
    pub fn writeln(message: &str) {
        if let Some(session) = Self::get_session() {
            WRITER.with_borrow_mut(|writer| {
                let _ = writeln!(
                    writer,
                    "{{ id: {}, macro_name: {}, crate_name: {}, timestamp: {}, message: {} }}",
                    session.id, session.macro_name, session.crate_name, session.timestamp, message
                );
            })
        }
    }
}

pub struct SessionGuard;

impl Drop for TraceSession {
    fn drop(&mut self) {
        CURRENT_CONTEXT.with(|ctx| *ctx.borrow_mut() = None);
        WRITER.with(|writer| {
            if let Err(e) = writer.borrow_mut().flush() {
                eprintln!("[Vacro Trace Warning] Failed to flush trace log: {}", e);
            }
        });
    }
}
