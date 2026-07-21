//! Compile-pass and compile-fail checks for the selected public spelling.

#[test]
fn dependent_static_shape_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/pass/*.rs");
    cases.compile_fail("tests/ui/fail/*.rs");
}
