//! Every long `snake_case` name this crate's comments cite resolves to real code.
//!
//! **The failure this exists to catch is a comment that outlives what it cites.**
//! A test renamed in one commit leaves its old name spelled in whichever doc
//! comments pointed a reader at it, and nothing in the toolchain notices: a name
//! inside a comment is prose to `rustc`, to Clippy, and to rustdoc alike, so the
//! stale citation compiles, lints clean, and reads exactly like a live one. That
//! is worse than a missing comment, because a reader who tries to follow it
//! concludes the evidence was deleted rather than moved. Auditing this crate on
//! 2026-08-07 turned up seven such citations at once, two of them naming tests
//! the BF16 optimizer-legality widening had renamed, and the only reason they
//! were found is that somebody happened to grep.
//!
//! # What is checked
//!
//! Every maximal `[A-Za-z0-9_]` run appearing on a comment line under
//! `crates/tiler-compiler/` that is shaped like a four-or-more-word `snake_case`
//! identifier must also appear *outside* a comment somewhere under `crates/`, or
//! be a Rust file's own stem. Appearing outside a comment is the resolution
//! test, and it is deliberately coarser than a symbol lookup: a definition, a
//! call, a `use`, a string literal, and a module path all count. What it refuses
//! is a name that survives *only* in prose, which is exactly the shape a rename
//! leaves behind.
//!
//! Resolution ranges over the whole workspace rather than this crate, because
//! this crate's comments legitimately cite `tiler-ir`'s and `tiler-build`'s
//! items and tests by name.
//!
//! # Why four words
//!
//! It is a threshold on how test-shaped a name is, and it was measured rather
//! than guessed. This crate's tests are named as sentences, so four segments
//! admits every citation of one; below four the population fills with struct
//! fields, builder methods, and tensor identifiers whose names happen to contain
//! two underscores, and each would need its own exemption. At four the whole
//! unresolved population is two names, both deliberate, both listed below.
//!
//! # What is not checked
//!
//! That a cited name is a *test* rather than a helper, and that the sentence
//! around it is true. Both are outside what a scan can decide. This check is the
//! narrow, mechanical half — the name exists — and it is worth having precisely
//! because it is the half nobody re-verifies by hand.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// This file's own name, excluded from the resolution universe.
///
/// [`DELIBERATELY_ABSENT`] spells its entries as string literals, which sit on
/// non-comment lines and would otherwise resolve themselves — making the
/// allowlist self-satisfying and the assertion below vacuous.
const SELF_FILE: &str = "cited_names_resolve.rs";

/// Names this crate's comments cite *because* nothing defines them.
///
/// Each entry is a deliberate reference to something absent, so its
/// unresolvability is the point being made rather than a defect. The list is
/// kept short on purpose: an entry added to silence a real rename would be a
/// mask, so the second test below asserts every entry is genuinely unresolvable
/// and fails when one starts resolving.
const DELIBERATELY_ABSENT: [(&str, &str); 3] = [
    (
        "the_uncarried_elementary_dimensions_are_outside_the_realization",
        "`policy.rs`'s carried-and-consumed coherence test names the deleted \
         omission tripwire it succeeded, so a reader can trace which check \
         guarded the pre-carried state and why its firing condition is now \
         the delivered one",
    ),
    (
        "declare_barrier_execution_scope",
        "`crate::target` names the per-dimension synchronization spellings it \
         deliberately does not offer, so a reader can see the whole-subject \
         argument refusing a specific alternative rather than describing one",
    ),
    (
        "two_region_occurrence_lowering_wall",
        "`tests/two_region_occurrence_lowering.rs` records that it began under \
         that file name, whose finding was that the ceiling did not live where \
         its ticket said; the old name is the transition being recorded",
    ),
];

/// The lowest number of cited names a healthy scan finds.
///
/// A floor rather than an exact count, because the population moves with every
/// comment. It is what stops a broken walk, a moved manifest, or a shape
/// predicate that stopped matching from reporting an empty search as a pass —
/// the failure mode where nothing ran and everything looked green. The measured
/// population on 2026-08-07 was 136.
const MINIMUM_CITED_NAMES: usize = 100;

/// The lowest number of workspace Rust files a healthy scan reads.
///
/// The measured count on 2026-08-07 was 385.
const MINIMUM_SCANNED_FILES: usize = 300;

#[test]
fn every_cited_name_resolves_to_something_outside_a_comment() {
    let crates = workspace_crates();
    let files = rust_files(&crates);
    assert!(
        files.len() >= MINIMUM_SCANNED_FILES,
        "the walk read {} files under {}, which is below the floor a complete \
         workspace clears — the scan is broken rather than clean",
        files.len(),
        crates.display(),
    );

    let resolvable = resolvable_names(&files);
    let cited = cited_names(&crates, &files);
    assert!(
        cited.len() >= MINIMUM_CITED_NAMES,
        "only {} cited names were found, which is below the floor — an empty \
         search is not a passing one",
        cited.len(),
    );

    let allowed: BTreeSet<&str> = DELIBERATELY_ABSENT.iter().map(|(name, _)| *name).collect();
    let dangling: BTreeMap<&String, &Vec<String>> = cited
        .iter()
        .filter(|(name, _)| !resolvable.contains(name.as_str()))
        .filter(|(name, _)| !allowed.contains(name.as_str()))
        .collect();

    assert!(
        dangling.is_empty(),
        "{} cited name(s) exist only inside comments. Each is a citation that \
         outlived what it named — correct the comment to the current name, or \
         add the name to `DELIBERATELY_ABSENT` with the reason it is cited for \
         being absent:\n{}",
        dangling.len(),
        dangling
            .iter()
            .map(|(name, sites)| format!("  {name}\n    cited at {}", sites.join(", ")))
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

/// Every deliberately absent name is still absent.
///
/// The allowlist's own regression guard. Without it an entry would keep
/// exempting a name long after something started defining one by that spelling,
/// and the exemption would then be hiding a citation nobody is checking.
#[test]
fn the_deliberately_absent_names_are_still_absent() {
    let crates = workspace_crates();
    let files = rust_files(&crates);
    let resolvable = resolvable_names(&files);

    for (name, reason) in DELIBERATELY_ABSENT {
        assert!(
            !resolvable.contains(name),
            "`{name}` now resolves, so its exemption is masking a live citation \
             rather than recording a deliberate absence. It was listed because: \
             {reason}",
        );
    }
}

/// Returns the workspace's `crates/` directory.
fn workspace_crates() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crates = manifest
        .parent()
        .expect("this crate's manifest sits inside the workspace's `crates/`")
        .to_path_buf();
    assert!(
        crates.join("tiler-compiler").is_dir(),
        "{} is not the workspace's `crates/` directory",
        crates.display(),
    );
    crates
}

/// Returns every `.rs` file under `root`, in a deterministic order.
///
/// `target/` is skipped: a build directory holds generated and vendored sources
/// whose identifiers would enlarge the resolution universe with names no comment
/// in this workspace is talking about, and whose presence depends on whether
/// anyone has built yet.
fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect_rust_files(root, &mut found);
    found.sort();
    found
}

fn collect_rust_files(directory: &Path, found: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|cause| panic!("{} is unreadable: {cause}", directory.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|cause| {
                panic!("{} has an unreadable entry: {cause}", directory.display())
            })
            .path();
        let name = path.file_name().unwrap_or_default().to_owned();
        if path.is_dir() {
            if name != "target" {
                collect_rust_files(&path, found);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") && name != SELF_FILE {
            found.push(path);
        }
    }
}

/// Returns every identifier that appears outside a comment, plus every file stem.
fn resolvable_names(files: &[PathBuf]) -> BTreeSet<String> {
    let mut resolvable = BTreeSet::new();
    for path in files {
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            resolvable.insert(stem.to_owned());
        }
        for line in read(path).lines() {
            if is_comment(line) {
                continue;
            }
            for token in identifier_runs(line) {
                resolvable.insert(token.to_owned());
            }
        }
    }
    resolvable
}

/// Returns each test-shaped name cited in a `tiler-compiler` comment, with its
/// citation sites.
fn cited_names(crates: &Path, files: &[PathBuf]) -> BTreeMap<String, Vec<String>> {
    let compiler = crates.join("tiler-compiler");
    let mut cited: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in files.iter().filter(|path| path.starts_with(&compiler)) {
        let label = path.strip_prefix(crates).unwrap_or(path).display();
        for (number, line) in read(path).lines().enumerate() {
            if !is_comment(line) {
                continue;
            }
            for token in identifier_runs(line).filter(|token| is_test_shaped(token)) {
                cited
                    .entry(token.to_owned())
                    .or_default()
                    .push(format!("{label}:{}", number + 1));
            }
        }
    }
    cited
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|cause| panic!("{} is unreadable: {cause}", path.display()))
}

/// Whether a line is a comment line.
///
/// Trailing comments after code are deliberately not scanned. Including them
/// would mean splitting a line on `//` without knowing whether the `//` is
/// inside a string literal, and a wrong split would drop real identifiers from
/// the resolution universe and invent citations. The check is under-inclusive
/// there rather than unsound.
fn is_comment(line: &str) -> bool {
    line.trim_start().starts_with("//")
}

/// Returns each maximal run of identifier characters in `line`.
fn identifier_runs(line: &str) -> impl Iterator<Item = &str> {
    line.split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|token| !token.is_empty())
}

/// Whether a token is a lowercase `snake_case` name of four or more words.
fn is_test_shaped(token: &str) -> bool {
    if !token.starts_with(|character: char| character.is_ascii_lowercase()) {
        return false;
    }
    if !token.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
    }) {
        return false;
    }
    let words: Vec<&str> = token.split('_').collect();
    words.len() >= 4 && words.iter().all(|word| !word.is_empty())
}
