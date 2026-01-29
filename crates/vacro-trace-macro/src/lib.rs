use proc_macro::TokenStream;
use vacro_doc_i18n::doc_i18n;
pub(crate) mod impls;
pub(crate) mod utils;

#[proc_macro]
#[doc_i18n]
/// <div class="doc-cn"> 捕获 TokenStream 快照。</div>
/// <div class="doc-en"> Capture a TokenStream snapshot. </div>
///
/// ::: @cn
///
/// 将当前的 Token 状态记录下来，以便在 `vacro-cli` 中查看。
/// 如果使用相同的 tag 调用多次，会自动展示 Diff。
///
/// 用法：`snapshot!("tag", tokens)`
/// :::
///
/// ::: @en
///
/// Records the current token state for inspection in `vacro-cli`.
/// If called multiple times with the same tag, it will automatically show a Diff.
///
/// Usage: `snapshot!("tag", tokens)`
/// :::
pub fn snapshot(input: TokenStream) -> TokenStream {
    impls::snapshot::snapshot_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
#[doc_i18n]
/// <div class="doc-cn"> 通用日志宏。</div>
/// <div class="doc-en"> Generic log macro. </div>
///
/// ::: @cn
///
/// 记录一条指定级别的日志消息。
/// :::
///
/// ::: @cn
///
/// Logs a message at the specified level.
/// :::
pub fn log(input: TokenStream) -> TokenStream {
    impls::log::log_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
#[doc_i18n]
/// <div class="doc-cn"> 记录错误 (Error) 级别的日志。 </div>
/// <div class="doc-en"> Logs an Error level message. </div>
pub fn error(input: TokenStream) -> TokenStream {
    impls::log::shortcut_impl("Error", input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
#[doc_i18n]
/// <div class="doc-cn"> 记录警告 (Warn) 级别的日志。 </div>
/// <div class="doc-en"> Logs a Warn level message. </div>
pub fn warn(input: TokenStream) -> TokenStream {
    impls::log::shortcut_impl("Warn", input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
#[doc_i18n]
/// <div class="doc-cn"> 记录信息 (Info) 级别的日志。 </div>
/// <div class="doc-en"> Logs an Info level message. </div>
pub fn info(input: TokenStream) -> TokenStream {
    impls::log::shortcut_impl("Info", input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
#[doc_i18n]
/// <div class="doc-cn"> 记录调试 (Debug) 级别的日志。 </div>
/// <div class="doc-en"> Logs a Debug level message. </div>
pub fn debug(input: TokenStream) -> TokenStream {
    impls::log::shortcut_impl("Debug", input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
#[doc_i18n]
/// <div class="doc-cn"> 记录追踪 (Trace) 级别的日志。 </div>
/// <div class="doc-en"> Logs a Trace level message. </div>
pub fn trace(input: TokenStream) -> TokenStream {
    impls::log::shortcut_impl("Trace", input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_attribute]
#[doc_i18n]
/// <div class="doc-cn"> 函数仪表化属性。</div>
/// <div class="doc-en"> Function instrumentation attribute. </div>
///
/// ::: @cn
///
/// 自动追踪函数的进入和退出，并记录参数信息。
/// :::
/// ::: @en
///
/// Automatically traces function entry and exit, and logs argument information.
/// :::
pub fn instrument(attr: TokenStream, input: TokenStream) -> TokenStream {
    impls::instrument::instrument_impl(attr.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
