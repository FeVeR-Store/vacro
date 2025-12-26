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

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::{Span, TokenStream};
    use quote::quote;
    use syn::{
        parse::{ParseStream, Parser},
        Result,
    };

    use crate::{
        ast::{
            capture::Capture,
            keyword::Keyword,
            node::{Pattern, PatternKind},
        },
        syntax::context::ParseContext,
    };

    fn parse_capture(input: TokenStream, ctx: &mut ParseContext) -> Result<Capture> {
        let parser = move |input: ParseStream| Capture::parse(input, ctx);
        parser.parse2(input)
    }

    #[test]
    fn test_inject_lookahead() {
        let ctx = &mut ParseContext::default();

        // 手动构造 Pattern 列表来测试 inject_lookahead 算法
        // 场景: Capture + Literal -> 应该注入

        // Mock数据构造：为了测试私有函数，我们需要构造 Pattern
        // 假设 Pattern 和 Keyword 是可访问的 (通常在同一 crate 或 test super 中)
        let input = quote!(#(x: Ident));

        let capture: Capture = parse_capture(input, ctx).unwrap();
        let pattern_capture = PatternKind::Capture(Box::new(capture));
        let pattern_literal = PatternKind::Literal(Keyword::Rust(",".to_string()));

        // Case 1: Capture 后面跟 Literal
        let patterns = vec![
            Pattern {
                kind: pattern_capture.clone(),
                span: Span::call_site(),
                meta: None,
            },
            Pattern {
                kind: pattern_literal.clone(),
                span: Span::call_site(),
                meta: None,
            },
        ];
        let optimized = inject_lookahead(patterns);

        assert_eq!(optimized.len(), 2);
        // 检查第一个 Capture 是否被注入了 lookahead
        if let PatternKind::Capture(cap) = &optimized[0].kind {
            let Capture {
                edge: Some(Keyword::Rust(s)),
                ..
            } = *cap.clone()
            else {
                panic!("Wrong lookahead type");
            };
            assert_eq!(s, ",");
        } else {
            panic!("Lookahead not injected");
        }

        // Case 2: Capture 后面跟 Capture (不应注入)
        let patterns_consecutive = vec![
            Pattern {
                kind: pattern_capture.clone(),
                span: Span::call_site(),
                meta: None,
            },
            Pattern {
                kind: pattern_capture.clone(),
                span: Span::call_site(),
                meta: None,
            },
        ];
        let optimized_consecutive = inject_lookahead(patterns_consecutive);
        if let PatternKind::Capture(cap) = &optimized_consecutive[0].kind {
            if let Capture { edge: Some(_), .. } = **cap {
                panic!("Should not inject lookahead when followed by another capture");
            };
        }

        // Case 3: Capture 在末尾 (不应注入)
        let patterns_end = vec![Pattern {
            kind: pattern_capture.clone(),
            span: Span::call_site(),
            meta: None,
        }];
        let optimized_end = inject_lookahead(patterns_end);
        if let PatternKind::Capture(cap) = &optimized_end[0].kind {
            if let Capture { edge: Some(_), .. } = **cap {
                panic!("Should not inject lookahead at end of stream");
            };
        }
    }
    #[test]
    fn test_inject_lookahead_complex_sequence() {
        let ctx = &mut ParseContext::default();

        // 构造序列: [Capture(A), Literal(,), Capture(B), Literal(;)]
        // 期望: A 注入 ',', B 注入 ';'
        let cap_a = parse_capture(quote!(#(a: Ident)), ctx).unwrap();
        let lit_comma = PatternKind::Literal(Keyword::Rust(",".to_string()));
        let cap_b = parse_capture(quote!(#(b: Ident)), ctx).unwrap();
        let lit_semi = PatternKind::Literal(Keyword::Rust(";".to_string()));

        let patterns = vec![
            Pattern {
                kind: PatternKind::Capture(Box::new(cap_a)),
                span: Span::call_site(),
                meta: None,
            },
            Pattern {
                kind: lit_comma,
                span: Span::call_site(),
                meta: None,
            },
            Pattern {
                kind: PatternKind::Capture(Box::new(cap_b)),
                span: Span::call_site(),
                meta: None,
            },
            Pattern {
                kind: lit_semi,
                span: Span::call_site(),
                meta: None,
            },
        ];

        let optimized = inject_lookahead(patterns);

        assert_eq!(optimized.len(), 4);

        // 检查 A 是否注入了 ','
        if let PatternKind::Capture(c) = &optimized[0].kind {
            assert_eq!(c.edge, Some(Keyword::Rust(",".to_string())));
        } else {
            panic!("Expected Capture at index 0");
        }

        // 检查 B 是否注入了 ';'
        if let PatternKind::Capture(c) = &optimized[2].kind {
            assert_eq!(c.edge, Some(Keyword::Rust(";".to_string())));
        } else {
            panic!("Expected Capture at index 2");
        }
    }

    #[test]
    fn test_inject_lookahead_interrupted_by_group() {
        let ctx = &mut ParseContext::default();

        // 构造序列: [Capture(A), Group(...), Literal(,)]
        // 期望: A 不会被注入 ','，因为中间隔了一个 Group
        let cap_a = parse_capture(quote!(#(a: Ident)), ctx).unwrap();
        let group = PatternKind::Group {
            delimiter: proc_macro2::Delimiter::Parenthesis,
            children: vec![],
        };
        let lit_comma = PatternKind::Literal(Keyword::Rust(",".to_string()));

        let patterns = vec![
            Pattern {
                kind: PatternKind::Capture(Box::new(cap_a)),
                span: Span::call_site(),
                meta: None,
            },
            Pattern {
                kind: group,
                span: Span::call_site(),
                meta: None,
            },
            Pattern {
                kind: lit_comma,
                span: Span::call_site(),
                meta: None,
            },
        ];

        let optimized = inject_lookahead(patterns);

        // 检查 A 的 edge 应该是 None
        if let PatternKind::Capture(c) = &optimized[0].kind {
            assert!(
                c.edge.is_none(),
                "Capture should not consume literal across a group"
            );
        }
    }
}
