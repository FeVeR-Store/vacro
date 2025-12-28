use ratatui::style::{Color, Style};
use ratatui::text::Span;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

// 使用 OnceLock 缓存加载开销较大的 SyntaxSet 和 ThemeSet，避免重复加载
static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

/// 对单行代码进行语法高亮
///
/// 输入:
/// - `line`: 代码行文本
/// - `syntax_extension`: 文件扩展名 (如 "rs" 代表 Rust)
///
/// 输出:
/// - `Vec<Span>`: 带有颜色的文本片段列表，可直接用于 Ratatui 渲染
pub fn highlight_line<'a>(line: &'a str, syntax_extension: &str) -> Vec<Span<'static>> {
    // 懒加载初始化
    let ps = SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines);
    let ts = THEME_SET.get_or_init(ThemeSet::load_defaults);

    // 查找语法定义，默认为 Rust，如果找不到则回退到纯文本
    let syntax = ps
        .find_syntax_by_extension(syntax_extension)
        .or_else(|| ps.find_syntax_by_extension("rs"))
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    // 选择颜色主题
    // "base16-ocean.dark" 在终端中效果较好且对比度适中
    // 其他可选: "base16-eighties.dark", "base16-mocha.dark", "InspiredGitHub"
    let theme = &ts.themes["base16-ocean.dark"];

    let mut h = HighlightLines::new(syntax, theme);
    // syntect 进行高亮分析
    let ranges: Vec<(syntect::highlighting::Style, &str)> =
        h.highlight_line(line, ps).unwrap_or_default();

    // 将 syntect 的样式转换为 Ratatui 的 Span
    ranges
        .into_iter()
        .map(|(style, content)| {
            Span::styled(
                content.to_string(), // 这里转换为 Owned String 以适配 Span<'static>
                Style::default().fg(to_ratatui_color(style.foreground)),
            )
        })
        .collect()
}

/// 辅助函数：将 syntect 的 RGB 颜色转换为 Ratatui 的 Color 枚举
fn to_ratatui_color(c: syntect::highlighting::Color) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}