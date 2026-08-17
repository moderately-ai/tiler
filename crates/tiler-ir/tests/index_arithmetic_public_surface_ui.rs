//! Compile-time boundary checks for the derived index-arithmetic requirement.

#[test]
fn index_arithmetic_cannot_be_rederived_from_kir_outside_tiler_ir() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/index-arithmetic-public-surface/fail/derive_from_kernel_type.rs");
}
