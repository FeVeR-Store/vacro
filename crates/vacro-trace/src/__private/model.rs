use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum TraceEvent {
    /// 阶段开始（例如 "Parsing", "Folding"）
    PhaseStart { name: String, time: u64 },
    /// 阶段结束（用于计算耗时）
    PhaseEnd { name: String, time: u64 },
    /// AST 快照
    Snapshot {
        tag: String,  // 标签，如 "Input", "After Fold", "Final Output"
        code: String, // stringified token stream
        time: u64,
    },
    /// 普通日志
    Log {
        level: String,
        message: String,
        time: u64,
    },
}
