use crate::data::{AnalyzedEntry, EntryMeta, TraceEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, StatefulWidget},
};
use similar::{ChangeTag, TextDiff};
use std::collections::HashSet;

/// 辅助函数：高亮普通文本中的搜索匹配项
///
/// 将输入文本按搜索关键词分割，匹配部分应用 `highlight_style`，
/// 其余部分应用 `base_style`。
fn highlight_text(
    text: &str,
    query: Option<&str>,
    base_style: Style,
    highlight_style: Style,
) -> Vec<Span<'static>> {
    if let Some(q) = query {
        if !q.is_empty() {
            let lower_text = text.to_lowercase();
            let lower_query = q.to_lowercase();
            let mut spans = Vec::new();
            let mut last_idx = 0;

            // 遍历所有匹配位置
            for (idx, _) in lower_text.match_indices(&lower_query) {
                // 添加匹配前的普通文本
                if idx > last_idx {
                    spans.push(Span::styled(text[last_idx..idx].to_string(), base_style));
                }
                // 添加高亮的匹配文本
                spans.push(Span::styled(
                    text[idx..idx + lower_query.len()].to_string(),
                    highlight_style,
                ));
                last_idx = idx + lower_query.len();
            }
            // 添加剩余的文本
            if last_idx < text.len() {
                spans.push(Span::styled(text[last_idx..].to_string(), base_style));
            }
            return spans;
        }
    }
    // 无搜索词时直接返回原样
    vec![Span::styled(text.to_string(), base_style)]
}

/// 辅助函数：在已有的语法高亮 Spans 中叠加搜索高亮
///
/// 这是一个难点：我们需要在保留原有的语法颜色（如关键字颜色）的同时，
/// 仅改变匹配部分的背景色。由于 Span 不可变，我们需要拆分 Span。
fn highlight_spans(
    spans: Vec<Span<'static>>,
    query: Option<&str>,
    highlight_style: Style,
) -> Vec<Span<'static>> {
    if let Some(q) = query {
        if q.is_empty() {
            return spans;
        }
        let lower_q = q.to_lowercase();
        let mut new_spans = Vec::new();

        for span in spans {
            // 修复 E0382: 使用 as_ref() 借用内容而不是移动所有权
            let text = span.content.as_ref();
            let lower_text = text.to_lowercase();
            let mut last_idx = 0;

            // 如果当前 Span 包含搜索词，则需要切割
            for (idx, _) in lower_text.match_indices(&lower_q) {
                if idx > last_idx {
                    let mut pre = span.clone();
                    pre.content = text[last_idx..idx].to_string().into();
                    new_spans.push(pre);
                }
                // 匹配部分：保留原有前景色的同时，叠加高亮样式（通常是背景色）
                let mut matched = span.clone();
                matched.content = text[idx..idx + lower_q.len()].to_string().into();
                matched.style = matched.style.patch(highlight_style);
                new_spans.push(matched);
                last_idx = idx + lower_q.len();
            }
            // 剩余部分
            if last_idx < text.len() {
                let mut post = span.clone();
                post.content = text[last_idx..].to_string().into();
                new_spans.push(post);
            }
        }
        return new_spans;
    }
    spans
}

/// 日志流查看器组件
pub struct TraceViewer<'a> {
    /// 待渲染的日志条目列表
    entries: &'a [AnalyzedEntry],
    /// 已展开的 Snapshot 索引集合
    expanded: &'a HashSet<usize>,
    /// 组件是否获得焦点
    is_focused: bool,
    /// 当前搜索关键词 (用于高亮)
    search_query: Option<&'a str>,
}

impl<'a> TraceViewer<'a> {
    pub fn new(
        entries: &'a [AnalyzedEntry],
        expanded: &'a HashSet<usize>,
        is_focused: bool,
        search_query: Option<&'a str>,
    ) -> Self {
        Self {
            entries,
            expanded,
            is_focused,
            search_query,
        }
    }

    /// 核心逻辑：生成列表项 (ListItem) 和索引映射
    ///
    /// 返回值：
    /// 1. `Vec<ListItem>`: 用于 UI 渲染的列表项
    /// 2. `Vec<usize>`: 视觉行索引到数据源 entries 索引的映射 (用于点击跳转)
    pub fn generate_items(
        entries: &'a [AnalyzedEntry],
        expanded: &HashSet<usize>,
        search_query: Option<&str>,
    ) -> (Vec<ListItem<'a>>, Vec<usize>) {
        let mut items = Vec::new();
        let mut mapping = Vec::new();
        let start_time = entries.first().map(|e| e.entry.timestamp).unwrap_or(0);

        // --- 样式常量定义 ---
        let time_col = Color::DarkGray;
        let snapshot_header_bg = Color::Rgb(45, 45, 55);
        let code_bg = Color::Rgb(30, 30, 30);
        // 搜索高亮样式：黄底黑字，最醒目
        let highlight_style = Style::default().bg(Color::Yellow).fg(Color::Black);

        for (entry_idx, entry) in entries.iter().enumerate() {
            // 计算相对时间 (+0005ms)
            let time_str = format!("+{:04}ms", entry.entry.timestamp.saturating_sub(start_time));

            match &entry.entry.message {
                // --- 普通日志事件 ---
                TraceEvent::Log { level, message, .. } => {
                    // 根据日志级别选择颜色和标签
                    let (badge_bg, badge_fg, label) = match level.to_uppercase().as_str() {
                        "ERROR" => (Color::Red, Color::White, " ERR "),
                        "WARN" => (Color::Yellow, Color::Black, " WRN "),
                        "INFO" => (Color::Blue, Color::White, " INF "),
                        "DEBUG" => (Color::DarkGray, Color::White, " DBG "),
                        _ => (Color::Reset, Color::Reset, " LOG "),
                    };

                    let mut line_spans = vec![Span::styled(
                        format!(" {} ", time_str),
                        Style::default().fg(time_col),
                    )];
                    // 高亮日志级别标签
                    line_spans.extend(highlight_text(
                        label,
                        search_query,
                        Style::default()
                            .bg(badge_bg)
                            .fg(badge_fg)
                            .add_modifier(Modifier::BOLD),
                        highlight_style,
                    ));
                    line_spans.push(Span::raw(" "));

                    // 高亮日志内容
                    line_spans.extend(highlight_text(
                        message,
                        search_query,
                        Style::default(),
                        highlight_style,
                    ));

                    items.push(ListItem::new(Line::from(line_spans)));
                    mapping.push(entry_idx);
                }
                // --- 快照事件 (代码/Diff) ---
                TraceEvent::Snapshot { tag, code, .. } => {
                    let is_expanded = expanded.contains(&entry_idx);

                    // 1. 渲染快照头部 (Header)
                    let icon = if is_expanded { "▼" } else { "▶" };

                    let mut header_spans = vec![Span::styled(
                        format!(" {} ", time_str),
                        Style::default().fg(time_col),
                    )];
                    // 高亮 "SNAPSHOT" 静态文本
                    let snapshot_label = format!(" {} SNAPSHOT ", icon);
                    header_spans.extend(highlight_text(
                        &snapshot_label,
                        search_query,
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                        highlight_style,
                    ));
                    // 高亮 Tag (如 "Before Expansion")
                    header_spans.push(Span::styled(" ", Style::default()));
                    header_spans.extend(highlight_text(
                        tag,
                        search_query,
                        Style::default().fg(Color::White),
                        highlight_style,
                    ));
                    header_spans.push(Span::styled(" ", Style::default()));

                    // 操作提示
                    header_spans.push(Span::styled(
                        if is_expanded {
                            " (SPACE to collapse) "
                        } else {
                            " (SPACE to expand) "
                        },
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::ITALIC),
                    ));

                    let header = Line::from(header_spans);
                    items
                        .push(ListItem::new(header).style(Style::default().bg(snapshot_header_bg)));
                    mapping.push(entry_idx);

                    // 2. 渲染快照内容 (Body) - 仅当展开时
                    if is_expanded {
                        // 检查是否存在上一次的代码，如果存在则进行 Diff
                        let previous_code_opt = match &entry.meta {
                            EntryMeta::Snapshot { previous_code } => previous_code.as_deref(),
                            _ => None,
                        };

                        if let Some(old_code) = previous_code_opt {
                            // 对比模式: 计算 Diff
                            // trim() 避免因为首尾空行导致的大片空白
                            let diff = TextDiff::from_lines(old_code.trim(), code.trim());
                            for change in diff.iter_all_changes() {
                                let (symbol, diff_bg) = match change.tag() {
                                    ChangeTag::Delete => ("-", Some(Color::Rgb(50, 15, 15))),
                                    ChangeTag::Insert => ("+", Some(Color::Rgb(15, 50, 15))),
                                    ChangeTag::Equal => (" ", None),
                                };

                                // 确定背景色 (Diff色 或 默认代码底色)
                                let final_bg = diff_bg.unwrap_or(code_bg);

                                // 行首行号区域 (Gutter)
                                let mut spans = vec![
                                    Span::styled("      ", Style::default().fg(Color::DarkGray)),
                                    Span::styled(
                                        format!(" {} ", symbol),
                                        Style::default().fg(Color::DarkGray),
                                    ),
                                ];

                                // 代码语法高亮
                                let code_line = change.value().trim_end_matches('\n');
                                let highlighted_spans =
                                    crate::syntax::highlight_line(code_line, "rs");

                                // 叠加搜索高亮
                                let processed_spans = highlight_spans(
                                    highlighted_spans,
                                    search_query,
                                    highlight_style,
                                );
                                spans.extend(processed_spans);

                                // 注意: 这里将背景色应用到 ListItem 上，以保持整行色块的视觉连贯性
                                // 之前的问题已通过 highlight_style = Underlined 解决，不会遮挡高亮
                                items.push(
                                    ListItem::new(Line::from(spans))
                                        .style(Style::default().bg(final_bg)),
                                );
                                mapping.push(entry_idx);
                            }
                        } else {
                            // 完整代码模式: 直接渲染
                            for line in code.trim().lines() {
                                let mut spans = vec![Span::styled(
                                    "        ",
                                    Style::default().fg(Color::DarkGray),
                                )];
                                let highlighted_spans = crate::syntax::highlight_line(line, "rs");

                                // 叠加搜索高亮
                                let processed_spans = highlight_spans(
                                    highlighted_spans,
                                    search_query,
                                    highlight_style,
                                );
                                spans.extend(processed_spans);

                                items.push(
                                    ListItem::new(Line::from(spans))
                                        .style(Style::default().bg(code_bg)),
                                );
                                mapping.push(entry_idx);
                            }
                        }
                    }
                }
                // --- 阶段开始事件 ---
                TraceEvent::PhaseStart { name, .. } => {
                    let mut spans = vec![
                        Span::styled(format!(" {} ", time_str), Style::default().fg(time_col)),
                        Span::styled(" ┌── ", Style::default().fg(Color::Cyan)), // 树状结构连接线
                    ];
                    spans.extend(highlight_text(
                        " PHASE START: ",
                        search_query,
                        Style::default()
                            .bg(Color::Cyan)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                        highlight_style,
                    ));
                    // 高亮阶段名称
                    spans.extend(highlight_text(
                        name,
                        search_query,
                        Style::default()
                            .bg(Color::Cyan)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                        highlight_style,
                    ));
                    spans.push(Span::styled(" ", Style::default().bg(Color::Cyan)));

                    items.push(ListItem::new(Line::from(spans)));
                    mapping.push(entry_idx);
                }
                // --- 阶段结束事件 ---
                TraceEvent::PhaseEnd { name, .. } => {
                    let mut spans = vec![
                        Span::styled(format!(" {} ", time_str), Style::default().fg(time_col)),
                        Span::styled(" └── ", Style::default().fg(Color::Cyan)),
                    ];
                    spans.extend(highlight_text(
                        " PHASE END: ",
                        search_query,
                        Style::default().fg(Color::Cyan),
                        highlight_style,
                    ));
                    // 高亮阶段名称
                    spans.extend(highlight_text(
                        name,
                        search_query,
                        Style::default().fg(Color::Cyan),
                        highlight_style,
                    ));

                    // 如果有耗时信息，显示在末尾
                    if let EntryMeta::PhaseEnd { duration } = &entry.meta {
                        spans.push(Span::styled(
                            format!(" ({:.2?})", duration),
                            Style::default().fg(Color::Yellow),
                        ));
                    }

                    items.push(ListItem::new(Line::from(spans)));
                    mapping.push(entry_idx);
                }
            }
        }
        (items, mapping)
    }
}

impl<'a> StatefulWidget for TraceViewer<'a> {
    type State = ratatui::widgets::ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let (items, _) = Self::generate_items(self.entries, self.expanded, self.search_query);

        let border_style = if self.is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Stream View ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            // 关键点：使用粗体+下划线作为选中样式，而不是反色或背景色
            // 这样可以让底层的黄色搜索高亮透过选中态显示出来
            .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

        StatefulWidget::render(list, area, buf, state);
    }
}
