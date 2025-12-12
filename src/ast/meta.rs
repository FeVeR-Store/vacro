//! ### SemanticInfo
//! <div class="doc-cn">
//!
//! 此处定义了`SemanticInfo`
//! `SemanticInfo`是用于生成更友好的错误信息的
//!
//! </div>

#[derive(Clone)]
#[cfg_attr(any(feature = "extra-traits", test), derive(Debug))]
pub struct SemanticInfo {
    /// 规则名称，用于报错。例如 "Function Name"
    pub name: Option<String>,
    /// 预期描述。例如 "expected a valid identifier"
    pub expectation: Option<String>,
    /// 示例代码。例如 "like: `my_fn`"
    pub example: Option<String>,
    /// 自定义错误模板
    pub custom_error: Option<String>,
}
