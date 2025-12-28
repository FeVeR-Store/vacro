/// 当前激活的视图区域
#[derive(PartialEq, Clone, Copy)]
pub enum ActiveView {
    /// 左侧会话列表
    SessionList,
    /// 右侧日志详情流
    TraceViewer,
}

/// 全局输入模式
#[derive(PartialEq, Clone, Copy)]
pub enum InputMode {
    /// 普通浏览模式 (支持导航快捷键)
    Normal,
    /// 编辑模式 (搜索框输入)
    Editing,
}
