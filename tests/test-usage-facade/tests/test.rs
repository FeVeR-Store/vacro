use test_usage_facade::parse_help;

#[test]
fn test_facade() {
    assert_eq!(parse_help!(str("hello")), "hello")
}
