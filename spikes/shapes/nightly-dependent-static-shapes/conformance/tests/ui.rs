//! Compile-pass and compile-fail checks for the selected public spelling.

use std::fs;
use std::path::Path;

/// Every compile-fail case this suite must resolve, sorted.
///
/// `trybuild` takes its cases from a glob, so a fixture deleted from the tree is
/// simply no longer checked and the run still reports success.
/// `scripts/check_rust.py` compares the run transcript against what is on disk,
/// which rejects a glob that stopped matching but not a case removed from both.
/// Naming the inventory here is the independent statement that closes it, so
/// dropping a case a governed decision cites has to be a deliberate edit.
const FAIL_CASES: [&str; 4] = [
    "forge.rs",
    "implement_evidence.rs",
    "rank_array_length.rs",
    "unequal_shapes.rs",
];

/// Every compile-pass contrast this suite must resolve, sorted.
const PASS_CASES: [&str; 2] = ["cross_crate_identity.rs", "ranks.rs"];

fn sorted_case_names(directory: &Path, extension: &str) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(directory)
        .expect("trybuild case directory")
        .map(|entry| entry.expect("trybuild case directory entry").file_name())
        .filter_map(|name| name.into_string().ok())
        .filter(|name| name.ends_with(extension))
        .collect();
    names.sort();
    names
}

#[test]
fn retained_fixture_inventory_is_complete() {
    let fail = Path::new("tests/ui/fail");
    assert_eq!(sorted_case_names(fail, ".rs"), FAIL_CASES);
    assert_eq!(
        sorted_case_names(Path::new("tests/ui/pass"), ".rs"),
        PASS_CASES
    );

    // Each compile-fail case must retain exactly the diagnostic the Rust gate
    // reproduces for it, and a `.stderr` with no case beside it is evidence for
    // nothing this suite compiles.
    let expected: Vec<String> = FAIL_CASES
        .iter()
        .map(|name| name.replace(".rs", ".stderr"))
        .collect();
    assert_eq!(sorted_case_names(fail, ".stderr"), expected);
}

#[test]
fn dependent_static_shape_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/pass/*.rs");
    cases.compile_fail("tests/ui/fail/*.rs");
}
