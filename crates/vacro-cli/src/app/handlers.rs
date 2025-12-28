use super::{ActiveView, App, InputMode};
use crossterm::event::{KeyCode, KeyEvent};

/// 处理键盘输入事件入口
pub fn handle_input(app: &mut App, key: KeyEvent) {
    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key),
        InputMode::Editing => handle_editing_mode(app, key),
    }
}

/// 处理普通模式下的按键
fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('/') => {
            app.input_mode = InputMode::Editing;
            // 保持之前的输入内容通常更友好，不做 clear
        }
        KeyCode::Tab => {
            // 切换视图时清空搜索，保证上下文隔离
            app.clear_search();
            app.active_view = match app.active_view {
                ActiveView::SessionList => ActiveView::TraceViewer,
                ActiveView::TraceViewer => ActiveView::SessionList,
            };
        }
        KeyCode::Char('j') | KeyCode::Down => match app.active_view {
            ActiveView::SessionList => app.next_session(),
            ActiveView::TraceViewer => app.scroll_detail(true),
        },
        KeyCode::Char('k') | KeyCode::Up => match app.active_view {
            ActiveView::SessionList => app.previous_session(),
            ActiveView::TraceViewer => app.scroll_detail(false),
        },
        // 切换展开/折叠 (仅在 TraceViewer 有效)
        KeyCode::Char(' ') => {
            if app.active_view == ActiveView::TraceViewer {
                app.toggle_detail_item();
            }
        }
        // 快速导航 (Snapshots)
        KeyCode::Char('s') => app.jump_snapshot(true),
        KeyCode::Char('S') => app.jump_snapshot(false),
        // 搜索结果跳转
        KeyCode::Char('n') => app.next_match(),
        KeyCode::Char('N') => app.prev_match(),

        KeyCode::PageDown => app.scroll_page(true),
        KeyCode::PageUp => app.scroll_page(false),
        KeyCode::Home => {
            if app.active_view == ActiveView::TraceViewer {
                app.detail_state.select(Some(0));
            }
        }
        KeyCode::End => {
            if app.active_view == ActiveView::TraceViewer {
                // 这里需要引用 widgets 模块的逻辑，或者在 App 中封装好获取 count 的方法
                // 由于 handler 是 App 的子模块，可以直接调用 crate::widgets
                let count = crate::widgets::TraceViewer::generate_items(
                    &app.filtered_log_entries,
                    &app.expanded_items,
                    None,
                )
                .0
                .len();
                app.detail_state.select(Some(count.saturating_sub(1)));
            }
        }
        _ => {}
    }
}

/// 处理编辑模式下的按键
fn handle_editing_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            app.perform_search();
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Backspace => {
            app.search_input.pop();
            if app.active_view == ActiveView::SessionList {
                app.update_filtered_sessions();
            } else if app.active_view == ActiveView::TraceViewer {
                app.update_filtered_logs();
            }
        }
        KeyCode::Char(c) => {
            app.search_input.push(c);
            if app.active_view == ActiveView::SessionList {
                app.update_filtered_sessions();
            } else if app.active_view == ActiveView::TraceViewer {
                app.update_filtered_logs();
            }
        }
        _ => {}
    }
}
