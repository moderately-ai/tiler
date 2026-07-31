//! The workspace package population has one authority that can say no.
//!
//! Crate counts appear throughout `docs/` and `tickets/`, and nothing
//! validated them: adding or removing a member silently staled every count
//! until a reader happened to recount. This test names the expected
//! population, derives the actual one from `cargo metadata`, and fails on
//! either a missing or an unexpected member — so the intended failure of
//! admitting a crate is updating this list in the same commit as the member.
//!
//! `workspace_members` is the field read, not `packages[].name`: package
//! objects also contain `targets[].name`, so a naive name scan over
//! `packages` matches target names too. The member IDs carry two forms and a
//! parse handling only one silently drops the other — a member whose package
//! name equals its directory renders as `…/crates/tiler#0.0.0`, while one
//! whose name differs renders as
//! `…/prototypes/serial-sum-compile#tiler-prototype-compile@0.0.0`. Both
//! prototype members take the second form.
//!
//! Hand-parsed for the same reason `dependency_direction.rs` hand-parses the
//! lockfile: the grammar needed is narrow and stable, and a JSON dependency
//! in the facade crate's graph is a cost nothing here justifies.

use std::collections::BTreeSet;
use std::process::Command;

/// Every workspace member, by package name.
///
/// Eleven production crates plus the two prototype proof executables. A
/// change to the workspace updates this list in the same commit — that is
/// the intended failure, not an obstacle.
const EXPECTED_MEMBERS: [&str; 13] = [
    "tiler",
    "tiler-artifact",
    "tiler-build",
    "tiler-cache",
    "tiler-compiler",
    "tiler-ir",
    "tiler-macros",
    "tiler-metal",
    "tiler-metal-aot",
    "tiler-prototype-compile",
    "tiler-prototype-run",
    "tiler-reference",
    "tiler-runtime",
];

#[test]
fn the_workspace_population_is_exactly_the_expected_one() {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .output()
        .expect("cargo metadata runs from the workspace root");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata = String::from_utf8(output.stdout).expect("cargo metadata emits UTF-8");

    let members = workspace_member_names(&metadata);

    // Count first: a parse that yields nothing must fail loudly rather than
    // agreeing with an accidentally empty expectation.
    assert_eq!(
        members.len(),
        EXPECTED_MEMBERS.len(),
        "the workspace holds {} members and this test expects {}; derived: {members:?}",
        members.len(),
        EXPECTED_MEMBERS.len(),
    );
    let expected: BTreeSet<&str> = EXPECTED_MEMBERS.into_iter().collect();
    assert_eq!(
        expected.len(),
        EXPECTED_MEMBERS.len(),
        "the expected list repeats a name"
    );
    let missing: Vec<&&str> = expected
        .iter()
        .filter(|name| !members.contains(**name))
        .collect();
    let unexpected: Vec<&String> = members
        .iter()
        .filter(|name| !expected.contains(name.as_str()))
        .collect();
    assert!(
        missing.is_empty() && unexpected.is_empty(),
        "expected members absent from the workspace: {missing:?}; workspace members this test \
         does not expect: {unexpected:?}. Update EXPECTED_MEMBERS in the same commit as the \
         member change."
    );
}

/// Extracts the package names from the `workspace_members` array.
///
/// The array is a flat list of JSON strings; each ID's name is the `#`
/// fragment when that fragment carries an `@`-separated name, and the final
/// path segment before the `#` otherwise.
fn workspace_member_names(metadata: &str) -> BTreeSet<String> {
    let key = "\"workspace_members\":[";
    let start = metadata
        .find(key)
        .expect("cargo metadata reports workspace_members")
        + key.len();
    let body = &metadata[start..];
    let end = body
        .find(']')
        .expect("the workspace_members array terminates");
    body[..end]
        .split(',')
        .map(|entry| member_name(entry.trim().trim_matches('"')))
        .collect()
}

/// Reads one member ID's package name, handling both ID forms.
fn member_name(id: &str) -> String {
    let (path, fragment) = id
        .rsplit_once('#')
        .expect("a workspace member ID carries a `#` fragment");
    match fragment.rsplit_once('@') {
        Some((name, _version)) => name.to_owned(),
        None => path
            .rsplit('/')
            .next()
            .expect("a member path has a final segment")
            .to_owned(),
    }
}
