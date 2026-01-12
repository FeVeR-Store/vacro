#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
#[derive(Default, Clone)]
pub struct ParseContext {
    // 捕获模式状态
    pub capture_mode: CaptureMode,
    // 行内捕获计数器
    pub inline_counter: usize,
    // 自定义符号计数器
    pub custom_symbol_counter: usize,
    // 错误收集
    pub _errors: Vec<syn::Error>,
}

#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
#[derive(Default, Clone, PartialEq)]
pub enum CaptureMode {
    Inline,
    Named,
    #[default]
    Unknown,
}
