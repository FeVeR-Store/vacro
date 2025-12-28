use crate::{
    app::{ActiveView, App, InputMode},
    widgets::{SessionList, TraceViewer},
};
use ratatui::prelude::*;
use unicode_width::UnicodeWidthStr;

/// UI 渲染主入口
///
/// 负责将 App 状态渲染到终端 Frame 上。
pub fn render(f: &mut Frame, app: &mut App) {
    // --- 1. 布局计算 ---

    // 判断是否显示搜索栏 (编辑模式或有搜索内容时显示)
    let show_search = app.input_mode == InputMode::Editing || !app.search_input.is_empty();

    // 垂直切分：主区域 (上) + 搜索栏 (下)
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            if show_search {
                Constraint::Length(3) // 搜索栏固定高度
            } else {
                Constraint::Length(0)
            },
        ])
        .split(f.area());

    // 水平切分主区域：会话列表 (左 30%) + 日志详情 (右 70%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(vertical_chunks[0]);

    // --- 2. 渲染左侧会话列表 ---
    f.render_stateful_widget(
        SessionList::new(
            &app.filtered_sessions,
            app.active_view == ActiveView::SessionList, // 是否高亮边框
        ),
        chunks[0],
        &mut app.list_state,
    );

    // --- 3. 渲染右侧日志详情 ---

    // 确定高亮关键词：
    // - 如果正在编辑 (InputMode::Editing)，使用实时输入内容实现"打字即高亮"
    // - 否则使用确认后的 search_query
    let highlight_query = if app.input_mode == InputMode::Editing {
        if app.search_input.is_empty() {
            None
        } else {
            Some(app.search_input.as_str())
        }
    } else if app.search_query.is_empty() {
        None
    } else {
        Some(app.search_query.as_str())
    };

    f.render_stateful_widget(
        TraceViewer::new(
            &app.filtered_log_entries,
            &app.expanded_items,
            app.active_view == ActiveView::TraceViewer,
            highlight_query,
        ),
        chunks[1],
        &mut app.detail_state,
    );

    // --- 4. 渲染底部搜索栏 (如果需要) ---
    if show_search {
        use ratatui::widgets::{Block, Borders, Paragraph};

        // 编辑模式下边框变黄
        let border_color = if app.input_mode == InputMode::Editing {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        let input = Paragraph::new(app.search_input.as_str())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Search ")
                    .border_style(Style::default().fg(border_color)),
            );

        f.render_widget(input, vertical_chunks[1]);

        // 渲染光标 (仅在编辑模式下)
        if app.input_mode == InputMode::Editing {
            f.set_cursor_position(
                // 计算光标位置: 区域左上角 x + 1 (边框) + 文本宽度
                ratatui::layout::Position::new(
                    vertical_chunks[1].x + 1 + app.search_input.width() as u16,
                    vertical_chunks[1].y + 1,
                ),
            );
        }
    }
}
