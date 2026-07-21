//! Compile-time boundary checks for verified index regions.

#[test]
fn verified_index_region_boundary() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/index-region/fail/*.rs");
}
