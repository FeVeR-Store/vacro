#[test]
fn ui_test() {
    let test = trybuild::TestCases::new();
    test.compile_fail("tests/ui/parse_quote.rs");
    test.compile_fail("tests/ui/parse_quote_spanned.rs");
    test.compile_fail("tests/ui/help.rs");
}
