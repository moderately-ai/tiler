//! Downstream compile-pass/fail contract for shape evidence.

#[test]
fn shape_evidence_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/shape-evidence/pass/*.rs");
    cases.compile_fail("tests/shape-evidence/fail/*.rs");
}
