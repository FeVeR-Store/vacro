use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

#[allow(dead_code)]
pub struct ExpansionPanel<W: Widget> {
    title: String,
    content: W,
    content_height: u16, // 这一项主要用于给父级提供高度建议
}
#[allow(dead_code)]
pub struct ExpansionPanelState {
    pub expanded: bool, // 设为 pub 方便外部修改
}

#[allow(dead_code)]
impl<W: Widget> ExpansionPanel<W> {
    pub fn new(title: &str, content: W, height: u16) -> Self {
        Self {
            title: title.to_string(),
            content,
            content_height: height,
        }
    }

    // [重要] 添加这个辅助方法，供父级布局计算使用
    pub fn get_render_height(&self, state: &ExpansionPanelState) -> u16 {
        if state.expanded {
            HEADER_HEIGHT + self.content_height
        } else {
            HEADER_HEIGHT
        }
    }
}

const HEADER_HEIGHT: u16 = 3;

impl<W: Widget> StatefulWidget for ExpansionPanel<W> {
    type State = ExpansionPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // 1. 定义布局：顶部是 Header，下面是 Content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(HEADER_HEIGHT), // 头部高度
                Constraint::Min(0),                // 剩余空间给内容
            ])
            .split(area);

        // 2. 渲染 Header (点击区域)
        let icon = if state.expanded { "▼" } else { "▶" };
        let title_text = format!("{} {}", icon, self.title);

        Paragraph::new(title_text)
            .block(Block::default().borders(Borders::ALL)) // 给 Header 加框
            .render(chunks[0], buf);

        // 3. 渲染 Content (仅在展开时)
        if state.expanded {
            // 注意：这里我们假设父级已经分配了足够的高度
            // 我们直接在 chunks[1] 这个区域渲染传入的任意组件
            self.content.render(chunks[1], buf);
        }
    }
}
