//! Compile-pass and compile-fail checks for the proposed public boundary.

#[test]
fn public_shape_evidence_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/pass/*.rs");
    cases.compile_fail("tests/ui/fail/*.rs");
}
