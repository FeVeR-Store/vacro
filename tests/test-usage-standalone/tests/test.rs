use test_usage_standalone::parse_help;

#[test]
fn test_standalone() {
    assert_eq!(parse_help!(str("hello")), "hello")
}
