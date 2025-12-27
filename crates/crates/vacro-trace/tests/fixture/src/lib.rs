use vacro_trace::{error, info, instrument, snapshot, warn};

#[instrument]
fn run_trace_logic() {
    info!("Function started");
    warn!("This is a warning");
    error!("This is an error");

    let code_snippet = "struct A { x: i32 }";
    snapshot!("Input Code", code_snippet);

    let x = 1 + 1;
    vacro_trace::debug!("Calculation result: {}", x);
}

#[test]
fn trigger_trace() {
    // 运行被 instrument 的函数，触发 Trace 写入
    run_trace_logic();
}
