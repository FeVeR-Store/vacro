use anyhow::Result;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
pub use vacro_analysis::{analyze_session, AnalyzedEntry, EntryMeta};
pub use vacro_trace::__private::model::{TraceEntry, TraceEvent};
pub use vacro_trace::__private::states::TraceSession;

// 扫描指定目录下的所有 .jsonl 文件
pub fn scan_sessions(dir: &Path) -> Result<Vec<TraceSession>> {
    let mut sessions = Vec::new();

    if !dir.exists() {
        return Ok(sessions);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            // 只需要读取第一行来获取 Session 的元数据
            if let Some(first_line) = read_first_line(&path)? {
                // 尝试解析第一行
                if let Ok(entry_data) = serde_json::from_str::<TraceEntry>(&first_line) {
                    sessions.push(TraceSession {
                        id: entry_data.id,
                        macro_name: entry_data.macro_name,
                        crate_name: entry_data.crate_name,
                        timestamp: entry_data.timestamp,
                        path: path.clone(),
                    });
                }
            }
        }
    }

    // 按时间倒序排列 (最新的在最上面)
    sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(sessions)
}

fn read_first_line(path: &Path) -> Result<Option<String>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    if reader.read_line(&mut line)? > 0 {
        Ok(Some(line))
    } else {
        Ok(None)
    }
}

// 读取某个 Session 的完整日志
pub fn load_entries(path: &Path) -> Result<Vec<AnalyzedEntry>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if let Ok(entry) = serde_json::from_str::<TraceEntry>(&line) {
            entries.push(entry);
        }
    }
    Ok(analyze_session(entries))
}
