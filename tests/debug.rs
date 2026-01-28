#[test]
fn debug() {
    let tb = trybuild::TestCases::new();
    tb.compile_fail("tests/debug/**/*.rs");
}
