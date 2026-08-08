//! What a host without Apple's toolchain still builds and runs, checked here.
//!
//! # The claim, and the two ways it fails
//!
//! The crate header states that a non-Apple host **builds and runs the
//! deterministic half and reports the measured half as unavailable rather than
//! skipping**. That is two claims, and they fail for unrelated reasons:
//!
//! 1. **The crate does not compile off Apple.** Then the claim is simply false,
//!    and nothing in this crate can observe it — a Rust compilation for another
//!    target is not something a running test can perform. The instrument is
//!    `cargo check` and `cargo clippy` against a non-Apple `--target`, recorded
//!    on `restore-the-conformance-crates-non-apple-build-and-lint-claim`.
//! 2. **It compiles and the deterministic tests silently vanish.** That one is
//!    worse, because the suite still reports green: gating one more module on
//!    the macOS predicate would remove twelve runs from every non-Apple host and
//!    no run anywhere would say so. A test population that collapses to zero and
//!    reports success is a check that cannot say no.
//!
//! This module is the instrument for the second, and it is a *source* census
//! rather than a harness one deliberately: a test cannot enumerate the harness's
//! own population, and it certainly cannot enumerate the population of a
//! compilation for a target this host does not run. What it can do is read the
//! gating decisions out of the source that makes them, on every host, and refuse
//! a collapse. It therefore holds on Apple and non-Apple hosts alike, which is
//! the property that matters: the collapse is *introduced* on the host that
//! cannot observe it.
//!
//! # Why the gated set is derived rather than listed
//!
//! A hand-written list of macOS-only modules is a second statement of the `cfg`
//! attributes, and the failure it would have to catch is exactly the one where
//! somebody adds a third gate and does not think about this file. So the set is
//! read back out of the module declarations themselves, and the census then
//! partitions the whole population by it.
//!
//! # The needles are assembled at run time
//!
//! Every string this scanner searches for would otherwise be a match against
//! this file's own source, which is the same reason
//! `crate::bf16_vertical::tests::the_unsafe_site_population_is_the_two_named_ones`
//! assembles its needle. That is also why the prose above says "the macOS
//! predicate" rather than spelling the attribute out.

use std::path::{Path, PathBuf};

/// How many test functions a non-Apple host must still run.
///
/// **A floor, and it sits one below the population deliberately.** The crate
/// declares 76 tests and the macOS predicate removes three of them, in
/// `dispatch`, so a non-Apple host runs 73. Seventy-two is what makes the
/// *smallest* collapse fail rather than only the large ones: the three smallest
/// device-free modules, `device_preflight`, `lints`, and `publication`'s own
/// tests, hold two tests each, so gating any of them drops the population to 71
/// and this refuses it. Gating `retained_record`'s tests drops it to 69,
/// `applicability` to 67, `publication::proof` to 64, `bf16_vertical`'s to 60,
/// and either `serial_sum`'s tests or `envelope`'s to 56.
///
/// The narrow margin is the cost of that sensitivity: removing two device-free
/// tests for any reason turns this red. Raising the floor with the population
/// is the ordinary edit; lowering it is a decision about what a non-Apple host
/// is held to, and belongs in a ticket rather than in this line.
///
/// It rose 53 → 64 on 2026-08-07 under
/// `route-the-realization-conformance-half-into-the-conformance-crate`, which
/// added eleven device-free tests: four in the new `retained_record`, five in
/// `envelope`'s (the record cross-check, the publishable-member derivation, the
/// payload-bound exclusion, the gate split, and the routed run's second half),
/// and two in `publication::proof` (the restated reference bound, and which
/// families state an iteration-step allowance).
///
/// It rose 64 → 67 on 2026-08-07 under
/// `state-a-subject-on-the-contraction-publication-path-s-reference-oracle`,
/// which added three device-free tests, all in `publication::proof`: the
/// packaged plan's contract reaching the oracle with its subject, the bridge
/// refusing a subject that plan's realization contradicts, and the published
/// expectations being unmoved by stating the contract. They are device-free
/// because deriving the oracle's contract needs `compile()` and neither a device
/// nor the offline Apple toolchain. **The floor moved with the population and by
/// the same three**, which is what preserves the two-test sensitivity above; it
/// was not moved to make anything pass, and the census counts the test attribute
/// out of the source whether or not a run carries `#[ignore]`. Spelling that
/// attribute here would make this file declare a test it does not have, which is
/// the trap the module header names.
///
/// It last rose 67 → 72 on 2026-08-07 under
/// `separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`,
/// which added five tests to `serial_sum`'s: the twelve-contributor operand
/// pair's counts, the refusal of the other parallel strategy's declared
/// grouping, the two strategies publishing different groupings at that count,
/// and the two dispatched runs there. **All five are device-free by this
/// census's rule and two of them are measured runs**, which is not a
/// contradiction: a measured run is a device-free *test* that reports its
/// measured half as unavailable when there is no device, which is exactly the
/// outcome this floor exists to keep observable. **The floor moved with the
/// population and by the same five**, and the two-test sensitivity above is
/// unchanged: the smallest gateable module still drops the population to 71.
const DEVICE_FREE_TEST_FLOOR: usize = 72;

/// A non-Apple host still runs the device-free test population.
///
/// The census is printed rather than only asserted, because the number this
/// guards is one a reader should be able to see move.
#[test]
fn a_non_apple_host_still_runs_the_device_free_test_population() {
    // Assembled so this scanner does not match its own source; see the module
    // header.
    let test_attribute = format!("{}{}", "#[", "test]");
    let predicate = format!("{}{}", "target", "_os");
    let runtime_predicate = format!("cfg!({predicate}");

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rust_sources(&root, &mut files);
    files.sort();
    assert!(
        files.len() >= 12,
        "the scan found {} source file(s), which is fewer than this crate has; a walk that \
         stopped finding files would report an empty population as an intact one",
        files.len(),
    );

    let gated = macos_gated_sources(&files, &predicate);
    assert!(
        !gated.is_empty(),
        "the scan derived no macOS-gated module from {} source file(s). Either the two gates the \
         crate header names are gone — which is a claim about this crate that belongs in that \
         header — or this derivation has stopped recognising the attribute it reads, in which \
         case every gated module below is being counted as device-free and the floor proves \
         nothing.",
        files.len(),
    );

    let mut device_free = 0_usize;
    let mut apple_only = 0_usize;
    for path in &files {
        let text = std::fs::read_to_string(path).expect("a crate source file is readable");
        let tests = text.matches(test_attribute.as_str()).count();
        if tests == 0 {
            continue;
        }
        if gated.contains(path) {
            apple_only += tests;
            continue;
        }
        // What makes the per-file partition above sound: a file that holds tests
        // and is not itself removed by the predicate must not mention the
        // predicate in an attribute, or some of its tests could be gated
        // individually and this census would count them as device-free. The
        // run-time `cfg!` form is a branch rather than a gate and is admitted;
        // it is how `crate::applicability` asks what host it is on.
        let mentions = text.matches(predicate.as_str()).count();
        let branches = text.matches(runtime_predicate.as_str()).count();
        assert_eq!(
            mentions,
            branches,
            "{}: holds {tests} test(s), is not a macOS-gated module, and names the macOS \
             predicate {mentions} time(s) of which only {branches} are the run-time branch form. \
             An attribute-form gate inside a device-free file removes tests that this census \
             would still be counting, so the partition has to be taught about it rather than \
             left to over-report.",
            path.display(),
        );
        device_free += tests;
    }

    let gated_names: Vec<_> = gated
        .iter()
        .filter_map(|path| path.strip_prefix(&root).ok())
        .map(|path| path.display().to_string())
        .collect();
    eprintln!(
        "portability census: {} source file(s); {device_free} device-free test(s) and \
         {apple_only} in the macOS-gated module(s) {gated_names:?}",
        files.len(),
    );

    assert!(
        device_free >= DEVICE_FREE_TEST_FLOOR,
        "a non-Apple host would run {device_free} test(s) and this crate's header claims it runs \
         the deterministic half; the floor is {DEVICE_FREE_TEST_FLOOR}. {apple_only} test(s) are \
         in macOS-gated modules {gated_names:?}. A module moved behind the macOS predicate takes \
         its tests off every non-Apple host silently — the suite stays green there and nothing \
         says which half it covered — so shrinking what that host runs is a decision to argue \
         rather than a number to lower.",
    );
}

/// Returns the source files a non-Apple build drops entirely.
///
/// Read out of the `mod` declarations themselves: a declaration whose `cfg`
/// names the macOS predicate — and does not negate it — removes the file it
/// names on every other host. The negated form is the *companion* module a
/// non-Apple host compiles instead, which is the opposite of gated.
///
/// Only file-backed modules are resolved. An inline `mod name { … }` block
/// carries no tests in this crate, and the mention check in the census above is
/// what keeps that true rather than assumed.
fn macos_gated_sources(files: &[PathBuf], predicate: &str) -> Vec<PathBuf> {
    let negated = format!("not({predicate}");
    let mut gated = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(path).expect("a crate source file is readable");
        let mut lines = text.lines().peekable();
        while let Some(line) = lines.next() {
            let attribute = line.trim();
            if !attribute.starts_with("#[cfg(")
                || !attribute.contains(predicate)
                || attribute.contains(negated.as_str())
            {
                continue;
            }
            let Some(declaration) = lines.peek().map(|next| next.trim()) else {
                continue;
            };
            let Some(name) = declaration
                .strip_suffix(';')
                .and_then(|item| item.rsplit_once("mod "))
                .map(|(_, name)| name)
            else {
                continue;
            };
            gated.extend(child_module_paths(path, name).into_iter().filter(|child| {
                // Resolved against the files the walk actually found, so a
                // declaration naming a module that no longer exists cannot
                // silently enlarge the gated set.
                files.contains(child)
            }));
        }
    }
    gated.sort();
    gated.dedup();
    gated
}

/// The two paths a child module of one source file may live at.
fn child_module_paths(parent: &Path, name: &str) -> Vec<PathBuf> {
    let stem = parent
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("a Rust source file has a stem");
    let directory = parent
        .parent()
        .expect("a crate source file has a parent directory");
    let directory = if stem == "lib" || stem == "mod" {
        directory.to_path_buf()
    } else {
        directory.join(stem)
    };
    vec![
        directory.join(format!("{name}.rs")),
        directory.join(name).join("mod.rs"),
    ]
}

/// Collects every `.rs` file beneath one directory.
///
/// Shared with
/// `crate::bf16_vertical::tests::the_unsafe_site_population_is_the_two_named_ones`,
/// which walks the same tree to count the crate's `unsafe` sites: two population
/// checks over one directory should not disagree about which files are in it.
pub(crate) fn collect_rust_sources(directory: &Path, into: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory).expect("the crate's source directory is readable");
    for entry in entries {
        let path = entry.expect("a directory entry is readable").path();
        if path.is_dir() {
            collect_rust_sources(&path, into);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            into.push(path);
        }
    }
}
