//! Downstream compile-pass/fail contract for the narrow symbolic-inference surface.

#[test]
fn semantic_inference_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/semantic-inference/pass/*.rs");
    cases.compile_fail("tests/semantic-inference/fail/*.rs");
}
