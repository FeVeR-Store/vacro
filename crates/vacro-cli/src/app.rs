mod enums;
mod handlers;

pub use enums::*;

use crate::data::{self, TraceSession};
use ratatui::widgets::ListState;
use std::path::PathBuf;

/// 应用程序核心状态管理
pub struct App {
    // --- UI 状态 ---
    /// 是否应退出程序
    pub should_quit: bool,
    /// 左侧会话列表的滚动状态
    pub list_state: ListState,
    /// 当前激活的视图区域 (焦点)
    pub active_view: ActiveView,
    /// 当前输入模式 (普通/编辑)
    pub input_mode: InputMode,
    /// 搜索框中的实时输入内容
    pub search_input: String,
    /// 确认后的搜索查询词 (用于高亮)
    pub search_query: String,
    /// 搜索匹配到的行号列表 (用于 n/N 跳转)
    pub search_matches: Vec<usize>,
    /// 当前选中的匹配项索引
    pub current_match_index: Option<usize>,

    // --- 数据源 ---
    /// 所有的会话列表 (原始数据)
    pub sessions: Vec<TraceSession>,
    /// 经过搜索过滤后的会话列表 (显示数据)
    pub filtered_sessions: Vec<TraceSession>,
    /// 目标监控目录
    pub target_dir: PathBuf,

    // --- 详情视图状态 ---
    /// 当前显示的日志文本内容 (已废弃，直接使用 entries 渲染)
    pub current_log_content: Vec<String>,
    /// 当前选中会话的完整日志条目
    pub current_entries: Vec<crate::data::AnalyzedEntry>,
    /// 经过搜索过滤后的日志条目
    pub filtered_log_entries: Vec<crate::data::AnalyzedEntry>,
    /// 右侧详情视图的滚动状态
    pub detail_state: ListState,
    /// 展开的 Snapshot 条目索引集合
    pub expanded_items: std::collections::HashSet<usize>,
}

impl App {
    /// 初始化应用程序状态
    pub fn new(target_dir: PathBuf) -> Self {
        let mut app = Self {
            should_quit: false,
            list_state: ListState::default(),
            active_view: ActiveView::SessionList,
            input_mode: InputMode::Normal,
            search_input: String::new(),
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match_index: None,
            sessions: Vec::new(),
            filtered_sessions: Vec::new(),
            target_dir,
            current_log_content: Vec::new(),
            current_entries: Vec::new(),
            filtered_log_entries: Vec::new(),
            detail_state: ListState::default(),
            expanded_items: std::collections::HashSet::new(),
        };
        // 启动时自动扫描一次
        app.refresh_sessions();
        // 默认选中第一个并加载
        if !app.sessions.is_empty() {
            app.load_selected_session();
        }
        app
    }

    /// 刷新会话列表 (重新扫描目录)
    pub fn refresh_sessions(&mut self) {
        // 假设日志存储在 target/vacro
        // TODO: 后面应该让这个路径可配置
        let trace_dir = self.target_dir.join("vacro");
        if let Ok(sessions) = data::scan_sessions(&trace_dir) {
            self.sessions = sessions;
            self.filtered_sessions = self.sessions.clone();
            if !self.sessions.is_empty() && self.list_state.selected().is_none() {
                self.list_state.select(Some(0));
            }
        }
    }

    /// 根据搜索输入过滤会话列表
    pub fn update_filtered_sessions(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_sessions = self.sessions.clone();
        } else {
            let query = self.search_input.to_lowercase();
            self.filtered_sessions = self
                .sessions
                .iter()
                .filter(|s| {
                    s.crate_name.to_lowercase().contains(&query)
                        || s.macro_name.to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
        }
        // 重置选中项，防止索引越界
        if !self.filtered_sessions.is_empty() {
            self.list_state.select(Some(0));
            self.load_selected_session();
        } else {
            self.list_state.select(None);
            self.current_entries.clear();
            self.filtered_log_entries.clear();
        }
    }

    /// 根据搜索输入过滤日志条目
    pub fn update_filtered_logs(&mut self) {
        // 需要清空匹配缓存，因为索引会变化
        self.search_matches.clear();
        self.current_match_index = None;
        self.expanded_items.clear(); // 清空展开状态

        if self.search_input.is_empty() {
            self.filtered_log_entries = self.current_entries.clone();
        } else {
            let query = self.search_input.to_lowercase();
            self.filtered_log_entries = self
                .current_entries
                .iter()
                .filter(|e| match &e.entry.message {
                    crate::data::TraceEvent::Log { level, message, .. } => {
                        level.to_lowercase().contains(&query)
                            || message.to_lowercase().contains(&query)
                    }
                    crate::data::TraceEvent::Snapshot { tag, code, .. } => {
                        "snapshot".contains(&query)
                            || tag.to_lowercase().contains(&query)
                            || code.to_lowercase().contains(&query)
                    }
                    crate::data::TraceEvent::PhaseStart { name, .. } => {
                        "phase start".contains(&query) || name.to_lowercase().contains(&query)
                    }
                    crate::data::TraceEvent::PhaseEnd { name, .. } => {
                        "phase end".contains(&query) || name.to_lowercase().contains(&query)
                    }
                })
                .cloned()
                .collect();

            // 自动展开所有匹配的 Snapshot，以便显示搜索内容
            for (idx, entry) in self.filtered_log_entries.iter().enumerate() {
                if let crate::data::TraceEvent::Snapshot { .. } = &entry.entry.message {
                    self.expanded_items.insert(idx);
                }
            }
        }
        // 重置详情视图滚动
        if !self.filtered_log_entries.is_empty() {
            self.detail_state.select(Some(0));
        }
    }

    /// 选中下一个会话
    pub fn next_session(&mut self) {
        if self.filtered_sessions.is_empty() {
            return;
        }
        let old_i = self.list_state.selected().unwrap_or(0);
        let next_i = if old_i >= self.filtered_sessions.len().saturating_sub(1) {
            0
        } else {
            old_i + 1
        };

        if old_i != next_i || self.current_entries.is_empty() {
            self.list_state.select(Some(next_i));
            self.load_selected_session();
        }
    }

    /// 选中上一个会话
    pub fn previous_session(&mut self) {
        if self.filtered_sessions.is_empty() {
            return;
        }
        let old_i = self.list_state.selected().unwrap_or(0);
        let next_i = if old_i == 0 {
            self.filtered_sessions.len().saturating_sub(1)
        } else {
            old_i - 1
        };

        if old_i != next_i || self.current_entries.is_empty() {
            self.list_state.select(Some(next_i));
            self.load_selected_session();
        }
    }

    /// 加载当前选中会话的详细日志
    pub fn load_selected_session(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(session) = self.filtered_sessions.get(i) {
                // 重置视图状态
                self.detail_state.select(Some(0));
                self.expanded_items.clear();

                // 使用 data 模块加载内容
                if let Ok(entries) = data::load_entries(&session.path) {
                    self.current_entries = entries.clone();
                    // 初始化日志过滤 (应用当前可能存在的过滤器)
                    self.filtered_log_entries = self.current_entries.clone();
                    self.update_filtered_logs();

                    self.current_log_content = entries
                        .iter()
                        .map(|e| {
                            format!(
                                "[{}] {}
{}",
                                e.entry.timestamp,
                                e.entry.macro_name,
                                serde_json::to_string_pretty(&e.entry.message).unwrap_or_default()
                            )
                        })
                        .collect();
                    self.detail_state.select(Some(0));
                    self.expanded_items.clear();
                }
            }
        }
    }

    /// 滚动详情视图
    pub fn scroll_detail(&mut self, down: bool) {
        let count = crate::widgets::TraceViewer::generate_items(
            &self.filtered_log_entries,
            &self.expanded_items,
            None,
        )
        .0
        .len();
        let current = self.detail_state.selected().unwrap_or(0);
        let new_idx = if down {
            (current + 1).min(count.saturating_sub(1))
        } else {
            current.saturating_sub(1)
        };
        self.detail_state.select(Some(new_idx));
    }

    /// 集中处理按键事件
    pub fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        handlers::handle_input(self, key);
    }

    fn scroll_page(&mut self, down: bool) {
        if self.active_view != ActiveView::TraceViewer {
            return;
        }

        let count = crate::widgets::TraceViewer::generate_items(
            &self.filtered_log_entries,
            &self.expanded_items,
            None,
        )
        .0
        .len();
        let current = self.detail_state.selected().unwrap_or(0);
        let page_size = 15; // 假设每页15行
        let new_idx = if down {
            (current + page_size).min(count.saturating_sub(1))
        } else {
            current.saturating_sub(page_size)
        };
        self.detail_state.select(Some(new_idx));
    }

    pub fn jump_snapshot(&mut self, next: bool) {
        let (_, mapping) = crate::widgets::TraceViewer::generate_items(
            &self.filtered_log_entries,
            &self.expanded_items,
            None,
        );

        if let Some(current_visual) = self.detail_state.selected() {
            let start = if next {
                current_visual + 1
            } else {
                current_visual.saturating_sub(1)
            };
            let items_count = mapping.len();

            let iter: Box<dyn Iterator<Item = usize>> = if next {
                Box::new(start..items_count)
            } else {
                Box::new((0..=start).rev())
            };

            for visual_idx in iter {
                if let Some(&entry_idx) = mapping.get(visual_idx) {
                    if let Some(entry) = self.filtered_log_entries.get(entry_idx) {
                        if let crate::data::TraceEvent::Snapshot { .. } = &entry.entry.message {
                            self.detail_state.select(Some(visual_idx));
                            return;
                        }
                    }
                }
            }
        }
    }

    /// 执行搜索并定位到第一个匹配项
    pub fn perform_search(&mut self) {
        self.search_matches.clear();
        self.current_match_index = None;
        self.search_query = self.search_input.clone();

        if self.search_query.is_empty() {
            return;
        }

        let (_, mapping) = crate::widgets::TraceViewer::generate_items(
            &self.filtered_log_entries,
            &self.expanded_items,
            Some(&self.search_query),
        );

        let query = self.search_query.to_lowercase();
        for (visual_idx, &entry_idx) in mapping.iter().enumerate() {
            if let Some(entry) = self.filtered_log_entries.get(entry_idx) {
                let match_found = match &entry.entry.message {
                    crate::data::TraceEvent::Log { message, .. } => {
                        message.to_lowercase().contains(&query)
                    }
                    crate::data::TraceEvent::Snapshot { tag, code, .. } => {
                        tag.to_lowercase().contains(&query) || code.to_lowercase().contains(&query)
                    }
                    crate::data::TraceEvent::PhaseStart { name, .. } => {
                        name.to_lowercase().contains(&query)
                    }
                    crate::data::TraceEvent::PhaseEnd { name, .. } => {
                        name.to_lowercase().contains(&query)
                    }
                };

                if match_found {
                    self.search_matches.push(visual_idx);
                }
            }
        }

        if !self.search_matches.is_empty() {
            self.current_match_index = Some(0);
            self.detail_state.select(Some(self.search_matches[0]));
        }
    }

    /// 跳转到下一个搜索匹配项
    pub fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let next_idx = match self.current_match_index {
            Some(i) => (i + 1) % self.search_matches.len(),
            None => 0,
        };
        self.current_match_index = Some(next_idx);
        self.detail_state
            .select(Some(self.search_matches[next_idx]));
    }

    /// 跳转到上一个搜索匹配项
    pub fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let prev_idx = match self.current_match_index {
            Some(i) => {
                if i == 0 {
                    self.search_matches.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.current_match_index = Some(prev_idx);
        self.detail_state
            .select(Some(self.search_matches[prev_idx]));
    }

    /// 清除搜索状态并重置视图
    pub fn clear_search(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_input.clear();
        self.search_query.clear();
        self.search_matches.clear();
        self.current_match_index = None;

        // 恢复完整列表
        self.filtered_sessions = self.sessions.clone();
        self.filtered_log_entries = self.current_entries.clone();
        self.expanded_items.clear();

        // 重置滚动条
        self.detail_state.select(Some(0));

        // 修正会话列表选中项
        if !self.filtered_sessions.is_empty() {
            self.list_state.select(Some(0));
            self.load_selected_session();
        }
    }

    /// 切换当前选中详情项的展开/折叠状态 (针对 Snapshot)
    pub fn toggle_detail_item(&mut self) {
        let (_, mapping) = crate::widgets::TraceViewer::generate_items(
            &self.filtered_log_entries,
            &self.expanded_items,
            None,
        );

        if let Some(visual_idx) = self.detail_state.selected() {
            if let Some(&entry_idx) = mapping.get(visual_idx) {
                if let Some(entry) = self.filtered_log_entries.get(entry_idx) {
                    if let crate::data::TraceEvent::Snapshot { .. } = &entry.entry.message {
                        if self.expanded_items.contains(&entry_idx) {
                            self.expanded_items.remove(&entry_idx);
                        } else {
                            self.expanded_items.insert(entry_idx);
                        }
                    }
                }
            }
        }
    }
}
