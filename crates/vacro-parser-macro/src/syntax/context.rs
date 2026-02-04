#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
#[derive(Default, Clone)]
pub struct ParseContext {
    // 自定义符号计数器
    pub custom_symbol_counter: usize,
    // 错误收集
    pub _errors: Vec<syn::Error>,
}
