#[cfg(test)]
mod test {
    use quote::quote;
    use vacro_trace::{error, info, instrument, snapshot, warn};
    #[test]
    #[instrument]
    fn test_function() {
        // 1. 测试普通日志
        info!("Function started");
        warn!("This is a warning");
        error!("This is an error");

        // 2. 测试快照
        let code_snippet = quote! { x: i32 };
        snapshot!("Field", code_snippet);

        let code_snippet = quote! { struct A { #code_snippet } };
        snapshot!("Struct", code_snippet);

        let code_snippet = quote! {
            #[derive(Debug)]
            #code_snippet
        };
        snapshot!("Struct", code_snippet);
        // 模拟一些逻辑
        let x = 1 + 1;
        vacro_trace::debug!("Calculation result: {}", x);
    }
}
