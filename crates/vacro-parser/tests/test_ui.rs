#[test]
fn ui_tests() {
    let test = trybuild::TestCases::new();
    test.compile_fail("tests/ui/manual_construct_spanned.rs");
}
