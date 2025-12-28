use std::collections::HashMap;
use std::time::Duration;
use vacro_trace::__private::model::{TraceEntry, TraceEvent};

#[derive(Debug, Clone)]
pub struct AnalyzedEntry {
    pub entry: TraceEntry,
    pub meta: EntryMeta,
}

#[derive(Debug, Clone, Default)]
pub enum EntryMeta {
    #[default]
    None,
    /// 阶段结束，包含阶段持续时间
    PhaseEnd { duration: Duration },
    /// 快照，包含相同标签的快照的前一个会被储存，用于Diff
    Snapshot { previous_code: Option<String> },
}

pub struct Analyzer {
    /// 记录阶段的stack: (PhaseName, Timestamp)
    phase_stack: Vec<(String, u64)>,
    /// 记录快照与tag: Tag -> Code
    snapshot_history: HashMap<String, String>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            phase_stack: Vec::new(),
            snapshot_history: HashMap::new(),
        }
    }

    pub fn process_batch(&mut self, entries: Vec<TraceEntry>) -> Vec<AnalyzedEntry> {
        entries.into_iter().map(|e| self.process_one(e)).collect()
    }

    pub fn process_one(&mut self, entry: TraceEntry) -> AnalyzedEntry {
        let mut meta = EntryMeta::None;

        match &entry.message {
            TraceEvent::PhaseStart { name, .. } => {
                self.phase_stack.push((name.clone(), entry.timestamp));
            }
            TraceEvent::PhaseEnd { name, .. } => {
                // 尝试在栈中查找匹配的起始位置（从后往前搜索）
                // 如果嵌套阶段正确，则此方法可以正确处理嵌套阶段。
                if let Some(idx) = self.phase_stack.iter().rposition(|(n, _)| n == name) {
                    let (_, start_time) = self.phase_stack.remove(idx);
                    let duration_ms = entry.timestamp.saturating_sub(start_time);
                    meta = EntryMeta::PhaseEnd {
                        duration: Duration::from_millis(duration_ms),
                    };
                }
            }
            TraceEvent::Snapshot { tag, code, .. } => {
                let previous = self.snapshot_history.insert(tag.clone(), code.clone());
                meta = EntryMeta::Snapshot {
                    previous_code: previous,
                };
            }
            _ => {}
        }

        AnalyzedEntry { entry, meta }
    }
}

// Helper for one-shot processing
pub fn analyze_session(entries: Vec<TraceEntry>) -> Vec<AnalyzedEntry> {
    let mut analyzer = Analyzer::new();
    analyzer.process_batch(entries)
}
