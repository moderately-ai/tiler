//! Downstream compile-pass/fail contract for exact typed authoring handles.

#[test]
fn typed_authoring_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/typed-handles/pass/*.rs");
    cases.compile_fail("tests/typed-handles/fail/*.rs");
}
