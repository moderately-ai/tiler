//! The path-shared adapter-route fixture, compiled from a second root.
//!
//! # What this target is evidence for
//!
//! Two modules of the adapter-route suite are compiled into roots that are not
//! that suite's own `main.rs`, through `#[path]`, so one assembly authority
//! exists rather than a copy. `tests/identity_join/main.rs` takes the image
//! decoder that way, and
//! `spikes/target-profiles/metal-subgroup-width-route-gate/src/main.rs` takes
//! both the decoder and the fixture from outside this workspace entirely.
//!
//! Nothing checked that the arrangement kept working. Commit `2cb7c83c` added
//! four `crate::adapter::ScalarEnvironmentSchema` references to the fixture;
//! the owning suite has an `adapter` module so it stayed green, and the
//! out-of-workspace consumer was left with four
//! `error[E0433]: cannot find adapter in crate` that nothing reported until a
//! person ran it months later. That is the defect this target closes.
//!
//! # Why a whole target rather than an assertion
//!
//! Portability here is a property of *name resolution*, so the only honest
//! checker is rustc. A grep for `crate::` would miss an alias, a
//! macro-expanded path, and a re-export, and would pass on a root that could
//! not resolve. Compiling the shared set from a root that deliberately carries
//! nothing else reproduces the consumers' failure exactly, in the ordinary
//! package gate. `prototypes/serial-sum-run/tests/lint_table.rs` is the same
//! idiom one layer up: a target that exists to compile a shared module from a
//! second root, because a second copy drifts against the first exactly as the
//! two roots drift without it.
//!
//! The module list below is hand-written, so on its own it could quietly stop
//! covering the arrangement — a consumer that started sharing a third module
//! would compile nowhere and redden nothing.
//! [`the_shared_set_is_exactly_what_every_path_consumer_takes`] is what stops
//! that: it enumerates every reference to the owning directory in every Rust
//! source in the repository and requires the union to equal this target's own
//! set, failing closed on any spelling it cannot resolve to a module name.
//!
//! # What this target deliberately does not do
//!
//! It runs none of the suite's route cases. Those belong to the owning suite,
//! which owns the fixture; restating them here would make two suites answer one
//! question and would drift. This target answers "does the shared set resolve
//! outside its owner", and nothing else.

// Every item these modules carry is unused from this root by construction: it
// exists to resolve them, not to route anything.
#[path = "adapter_route/image.rs"]
#[allow(dead_code)]
mod image;

#[path = "adapter_route/fixture.rs"]
#[allow(dead_code)]
mod fixture;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The directory whose modules are shared, relative to the workspace root.
const OWNING_DIRECTORY: &str = "crates/tiler-runtime/tests/adapter_route";

/// The directory name a `#[path]` literal must reach through, with separator.
const OWNING_SEGMENT: &str = "adapter_route/";

/// This file, relative to its package root, so the walk can skip itself.
const OWN_SOURCE: &str = "tests/adapter_route_portability.rs";

/// The only `#[path]` spelling this reader resolves.
const PATH_ATTRIBUTE_OPEN: &str = "#[path = \"";

/// The workspace root, established rather than assumed.
///
/// # Panics
///
/// Panics unless the directory two levels above this package is a Cargo
/// workspace root listing this package. Every path below is stated relative to
/// it, and a wrong root would walk a smaller tree and report no consumers.
fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest
        .parent()
        .and_then(Path::parent)
        .expect("the package sits two levels below the workspace root")
        .to_path_buf();
    let manifest_text = std::fs::read_to_string(root.join("Cargo.toml"))
        .expect("the workspace root carries a Cargo.toml");
    assert!(
        manifest_text.contains("[workspace]"),
        "{} is not a workspace root",
        root.display(),
    );
    assert!(
        manifest_text.contains("\"crates/tiler-runtime\""),
        "{} does not list this package as a member",
        root.display(),
    );
    root
}

/// Every `.rs` file under `root`, skipping build output and version control.
fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut found = Vec::new();
    while let Some(directory) = pending.pop() {
        let entries = std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("{} is readable: {error}", directory.display()));
        for entry in entries {
            let entry = entry.expect("a readable directory entry");
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.is_dir() {
                // `target` holds compiled copies of sources already walked and
                // `.git` holds every historical revision of them; either would
                // report consumers that do not exist in the working tree. The
                // comparison is exact, so `spikes/target-profiles` is walked.
                if name != "target" && name != ".git" {
                    pending.push(path);
                }
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                found.push(path);
            }
        }
    }
    found
}

/// The module names one Rust source takes from the owning directory.
///
/// Line comments are removed first, so prose naming the directory — which every
/// document of this arrangement contains — is not mistaken for a consumer. What
/// survives is code, and in code a reference to the owning directory is either
/// a resolvable `#[path]` attribute or something this reader must not guess at.
///
/// `strict` is `false` only for this file, whose own constants above name the
/// directory in code. The reader cannot be its own fail-closed subject, and it
/// does not need to be: rustc resolves this file's `#[path]` lines, so a
/// spelling this parser missed here can only *understate* the compiled set,
/// which the equality below reports rather than hides.
///
/// # Panics
///
/// Under `strict`, panics on any code reference to the owning directory it
/// cannot resolve to one flat module name. Failing closed is the point — an
/// unrecognized spelling is a consumer whose coverage is unknown, and reporting
/// none would restore exactly the silence this target removes.
fn shared_modules(path: &Path, source: &str, strict: bool) -> BTreeSet<String> {
    let mut modules = BTreeSet::new();
    for (index, line) in source.lines().enumerate() {
        let code = line.split("//").next().unwrap_or_default();
        if !code.contains(OWNING_SEGMENT) {
            continue;
        }
        let line_number = index + 1;
        let Some(opened) = code.find(PATH_ATTRIBUTE_OPEN) else {
            assert!(
                !strict,
                "{}:{line_number}: names the shared directory in code outside a literal \
                 `{PATH_ATTRIBUTE_OPEN}…\"]` attribute, so whether it is a consumer, and of \
                 which modules, cannot be established by reading: {}",
                path.display(),
                code.trim(),
            );
            continue;
        };
        let rest = &code[opened + PATH_ATTRIBUTE_OPEN.len()..];
        let closed = rest.find('"').unwrap_or_else(|| {
            panic!(
                "{}:{line_number}: the `#[path]` literal is not closed on its own line; \
                 wrapped and escaped forms are unsupported: {}",
                path.display(),
                code.trim(),
            )
        });
        let literal = &rest[..closed];
        let module = literal
            .rsplit_once(OWNING_SEGMENT)
            .map(|(_, tail)| tail)
            .and_then(|tail| tail.strip_suffix(".rs"))
            .unwrap_or_else(|| {
                panic!(
                    "{}:{line_number}: `{literal}` reaches the shared directory but does not \
                     name one `<module>.rs` inside it",
                    path.display(),
                )
            });
        assert!(
            !module.contains('/'),
            "{}:{line_number}: `{literal}` reaches below the shared directory; the arrangement \
             is flat and a nested source under it has no stated owner",
            path.display(),
        );
        modules.insert(module.to_owned());
    }
    modules
}

/// The shared set this target compiles must equal the union every `#[path]`
/// consumer in the repository takes from the owning directory.
///
/// Both directions matter. A module a consumer takes and this target does not
/// compile is unguarded — the `2cb7c83c` failure exactly. A module this target
/// compiles that no consumer takes means the arrangement has ended and the
/// coverage here is a claim about nothing, which should be removed deliberately
/// rather than carried as apparent evidence.
#[test]
fn the_shared_set_is_exactly_what_every_path_consumer_takes() {
    let root = workspace_root();
    let own_source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(OWN_SOURCE);
    let owning_directory = root.join(OWNING_DIRECTORY);
    assert!(
        owning_directory.is_dir(),
        "{} does not exist, so this target guards a directory that is gone",
        owning_directory.display(),
    );

    let own_text = std::fs::read_to_string(&own_source).unwrap_or_else(|error| {
        panic!(
            "{} is readable; if this target was renamed, `OWN_SOURCE` must follow it: {error}",
            own_source.display(),
        )
    });
    let compiled_here = shared_modules(&own_source, &own_text, false);
    assert!(
        !compiled_here.is_empty(),
        "{} declares no `#[path]` module, so this target compiles nothing and guards nothing",
        own_source.display(),
    );

    let mut consumers: BTreeMap<PathBuf, BTreeSet<String>> = BTreeMap::new();
    let mut scanned = 0_usize;
    for source in rust_sources(&root) {
        if source == own_source || source.starts_with(&owning_directory) {
            continue;
        }
        scanned += 1;
        let text = std::fs::read_to_string(&source)
            .unwrap_or_else(|error| panic!("{} is readable UTF-8: {error}", source.display()));
        let modules = shared_modules(&source, &text, true);
        if !modules.is_empty() {
            consumers.insert(source, modules);
        }
    }
    assert!(
        scanned > 1,
        "the walk reached {scanned} Rust source(s) outside the owning directory, which is not a \
         tree this repository could have; it did not reach its subject",
    );

    let census: Vec<String> = consumers
        .iter()
        .map(|(source, modules)| {
            format!(
                "{} takes {:?}",
                source.strip_prefix(&root).unwrap_or(source).display(),
                modules,
            )
        })
        .collect();
    assert!(
        !consumers.is_empty(),
        "no `#[path]` consumer of {OWNING_DIRECTORY} was found among {scanned} Rust source(s); \
         either the sharing arrangement ended, and this target should be removed with it, or \
         the only consumers spell their include in a form this reader no longer resolves",
    );

    let taken: BTreeSet<String> = consumers.values().flatten().cloned().collect();
    assert_eq!(
        taken,
        compiled_here,
        "the shared set drifted. {} consumer(s) among {scanned} Rust source(s): {census:#?}. \
         This target compiles {compiled_here:?}. Every module a consumer takes must be compiled \
         here, or its portability is unchecked and the next `crate::`-rooted reference added to \
         that module breaks the consumer silently.",
        consumers.len(),
    );
}
