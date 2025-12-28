use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, StatefulWidget},
};

use crate::data::TraceSession;

pub struct SessionList<'a> {
    sessions: &'a [TraceSession],
    is_focused: bool,
}

impl<'a> SessionList<'a> {
    pub fn new(sessions: &'a [TraceSession], is_focused: bool) -> Self {
        Self {
            sessions,
            is_focused,
        }
    }
}

// 实现 StatefulWidget 因为我们需要读取/更新 ListState
impl<'a> StatefulWidget for SessionList<'a> {
    type State = ratatui::widgets::ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items: Vec<ListItem> = self
            .sessions
            .iter()
            .map(|s| {
                // 渲染逻辑封装在这里：格式化显示文本
                let content = format!("[{}] {}", s.crate_name, s.macro_name);
                ListItem::new(content)
            })
            .collect();

        let border_style = if self.is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Sessions ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )
            .highlight_symbol(">> ");

        // 代理给原生的 List widget 渲染
        StatefulWidget::render(list, area, buf, state);
    }
}
