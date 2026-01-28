#[test]
fn error() {
    let tb = trybuild::TestCases::new();
    tb.compile_fail("tests/error/**/*.rs");
}
