use crate::ast::node::{Pattern, PatternKind};

/// 对模式列表进行“前瞻优化”：
/// 如果一个捕获组 (Capture) 紧跟着一个字面量 (Literal)，
/// 将该字面量注入到捕获组中作为 lookahead hint。
pub fn inject_lookahead(patterns: Vec<Pattern>) -> Vec<Pattern> {
    let mut optimized = Vec::with_capacity(patterns.len());
    // 缓冲区：存放等待查看下一个 Token 的捕获组
    let mut pending_capture: Option<Pattern> = None;

    for pattern in patterns {
        match &pattern.kind {
            // 情况 A: 遇到了字面量 (例如 ",")
            PatternKind::Literal(keyword) => {
                // 检查缓冲区里有没有正在等待前瞻的捕获组
                if let Some(Pattern {
                    kind: PatternKind::Capture(capture),
                    span,
                    meta,
                }) = pending_capture
                {
                    let mut optimized_capture = capture.clone();
                    optimized_capture.edge = Some(keyword.clone());
                    // 核心逻辑：注入前瞻信息
                    // 将原来的 Capture(spec, None) 变为 Capture(spec, Some(keyword))
                    optimized.push(Pattern {
                        kind: PatternKind::Capture(optimized_capture),
                        span,
                        meta,
                    });
                } else if let Some(other) = pending_capture {
                    // 防御性编程：虽然逻辑上 pending 只可能是 Capture，但如果有其他变体，原样推入
                    optimized.push(other);
                }

                // 缓冲区已清空/消费
                pending_capture = None;

                // 字面量本身也必须保留在流中
                optimized.push(pattern);
            }

            // 情况 B: 遇到了新的捕获组 (例如 #(name: Type))
            PatternKind::Capture(..) => {
                // 如果之前还有一个捕获组没等到字面量 (比如连续两个捕获组)
                if let Some(prev) = pending_capture {
                    optimized.push(prev); // 前一个只能原样提交
                }
                // 将当前捕获组放入缓冲区等待
                pending_capture = Some(pattern);
            }

            // 情况 C: 其他 Token (例如 Group)
            _ => {
                // 结算缓冲区
                if let Some(prev) = pending_capture {
                    optimized.push(prev);
                }
                pending_capture = None;
                optimized.push(pattern);
            }
        }
    }

    // 循环结束，处理缓冲区里最后残留的捕获组
    if let Some(last) = pending_capture {
        optimized.push(last);
    }

    optimized
}
