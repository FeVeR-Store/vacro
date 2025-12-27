use std::fs;
use std::path::Path;
use std::process::Command;

use rust_format::Formatter;

#[test]
fn test_end_to_end_logging() {
    // 1. 定位 Fixture 项目路径
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let fixture_path = Path::new(&manifest_dir).join("tests/fixture");

    // 2. 显式指定 Target 目录 (就在 fixture 目录下)
    let target_dir = fixture_path.join("target");

    // 3. 清理旧的构建产物 (强制重新编译以触发宏)
    if target_dir.exists() {
        let _ = fs::remove_dir_all(&target_dir);
    }

    // 4. 运行 cargo test
    // 我们运行测试来触发代码执行，从而生成 Trace 文件
    let output = Command::new("cargo")
        .arg("test")
        .current_dir(&fixture_path)
        .env("VACRO_TRACE", "1")
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .expect("Failed to execute cargo test");

    // 打印输出以便调试
    println!(
        "--- Cargo Stdout ---\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!(
        "--- Cargo Stderr ---\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output.status.success(), "Fixture project failed to compile");

    // 5. 查找生成的 Trace 文件
    // 因为我们指定了 target_dir，所以路径是固定的：
    // fixture/target/vacro/trace-UUID.jsonl
    let vacro_dir = target_dir.join("vacro");

    assert!(
        vacro_dir.exists(),
        "Vacro directory not found at {:?}",
        vacro_dir
    );

    let mut jsonl_files = vec![];
    if let Ok(entries) = fs::read_dir(&vacro_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
                jsonl_files.push(path);
            }
        }
    }

    assert!(!jsonl_files.is_empty(), "No .jsonl trace files generated!");

    // 6. 验证文件内容 (取第一个文件)
    let log_content = fs::read_to_string(&jsonl_files[0]).expect("Failed to read trace file");
    println!("Trace Content:\n{}", log_content);

    // 验证关键信息
    assert!(
        log_content.contains(r#""level":"INFO","message":"Function started""#),
        "Missing INFO log"
    );
    assert!(
        log_content.contains(r#""level":"WARN","message":"This is a warning""#),
        "Missing WARN log"
    );
    assert!(
        log_content.contains(r#""level":"ERROR","message":"This is an error""#),
        "Missing ERROR log"
    );
    assert!(
        log_content.contains(r#""tag":"Input Code""#),
        "Missing Snapshot tag"
    );

    let pretty_code = rust_format::PrettyPlease::default()
        .format_str(r#"struct A { x: i32 }"#)
        .unwrap_or("struct A {\n    x: i32,\n}\n".to_string());
    let formatted_code = pretty_code.replace("\n", "\\n");
    assert!(
        log_content.contains(&formatted_code),
        "Missing Snapshot code content"
    );
    assert!(
        log_content.contains(r#""crate_name":"vacro_trace_fixture""#),
        "Correct crate name not found"
    );
}
