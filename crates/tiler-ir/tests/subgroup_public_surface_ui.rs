//! Compile-time boundary checks for the growing subgroup-transfer vocabulary.

#[test]
fn subgroup_transfer_requires_an_external_wildcard() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/subgroup-public-surface/fail/exhaustive_transfer_match.rs");
}
