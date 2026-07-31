//! The frontend sits at the top of the workspace graph, and nothing points
//! back at it.
//!
//! The compiler and IR are consumer-agnostic by contract: an edge from any
//! internal crate to `tiler` or `tiler-macros` would put a frontend's macro,
//! grammar, and expansion machinery inside the compiler's dependency closure,
//! which is the coupling the split exists to prevent. Reviewing manifests
//! catches that only for as long as someone reviews them, so this checks the
//! resolved graph instead.
//!
//! `Cargo.lock` is the authority read here rather than the manifests, because
//! it is what Cargo actually resolved and it merges normal, build, and dev
//! dependencies into one edge list per package. Direct edges are sufficient
//! evidence: if no package has a direct edge to a frontend package, none can
//! have an indirect one either.

use std::collections::BTreeSet;

/// The packages no other workspace member may depend on.
const FRONTEND_PACKAGES: [&str; 2] = ["tiler", "tiler-macros"];

/// Packages this facade must not acquire an edge to, and why.
///
/// This is the outward half of the frontier. The inward half above keeps the
/// compiler consumer-agnostic; this one keeps the *consumer's* build graph free
/// of things it never asked for. `tiler-metal-aot` spawns `xcrun metal` and
/// `xcrun metallib`, and a normal edge from this crate would compile that driver
/// into every consumer on every platform — the cost ADR 0077 item 4 already
/// refused when it kept `tiler-metal`'s edge to the driver development-only.
///
/// `tiler-macros` may hold that edge and does: a `proc-macro` crate and its
/// dependencies are built for the host, so they never reach a consumer's target
/// build graph. This list is therefore about this crate specifically, not about
/// the frontend as a whole.
///
/// **Draft.** The placement it encodes is presented for acceptance by
/// `promote-artifact-family-selection-for-the-frontend` and is not yet ratified.
const FACADE_FORBIDDEN_DEPENDENCIES: [&str; 1] = ["tiler-metal-aot"];

/// One `[[package]]` block's name and its direct dependency names.
struct LockPackage {
    name: String,
    dependencies: Vec<String>,
}

#[test]
fn no_package_depends_on_the_frontend() {
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.lock"))
        .expect("the workspace lockfile is readable from the facade crate");
    let packages = parse_lock_packages(&lock);

    // Without these two, "no offending edge" would also be what an empty or
    // misparsed lockfile reports, so the check has to name its population
    // before it can be trusted to say no.
    let names: BTreeSet<&str> = packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    for frontend in FRONTEND_PACKAGES {
        assert!(
            names.contains(frontend),
            "`{frontend}` is not in the parsed lockfile; the parse or the workspace is wrong, \
             not the dependency direction"
        );
    }
    assert!(
        packages.iter().any(|package| package.name == "tiler"
            && package.dependencies.iter().any(|dep| dep == "tiler-macros")),
        "the facade's edge to `tiler-macros` is missing from the parsed lockfile, so this test \
         is not reading dependency lists"
    );

    let offenders: Vec<String> = packages
        .iter()
        .filter(|package| !FRONTEND_PACKAGES.contains(&package.name.as_str()))
        .flat_map(|package| {
            package
                .dependencies
                .iter()
                .filter(|dep| FRONTEND_PACKAGES.contains(&dep.as_str()))
                .map(move |dep| format!("{} -> {dep}", package.name))
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "the frontend must stay a leaf consumers depend on, but these edges point back at it: \
         {offenders:?}"
    );
}

/// The offline Apple toolchain driver stays out of the consumer's build graph.
///
/// The frontend needs the canonical `ArtifactFamilySelection` the driver owns,
/// and `tiler-macros` is where that edge is paid for, because a `proc-macro`
/// crate's dependencies are host-built. This asserts the two halves together:
/// the facade has no such edge, and the macro crate does — so a future change
/// that "simplified" the split by re-exporting the driver's vocabulary through
/// the facade fails here rather than being noticed in a manifest review.
#[test]
fn the_facade_does_not_carry_the_offline_apple_driver() {
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.lock"))
        .expect("the workspace lockfile is readable from the facade crate");
    let packages = parse_lock_packages(&lock);

    let facade = packages
        .iter()
        .find(|package| package.name == "tiler")
        .expect("`tiler` is not in the parsed lockfile; the parse or the workspace is wrong");
    let macros = packages
        .iter()
        .find(|package| package.name == "tiler-macros")
        .expect(
            "`tiler-macros` is not in the parsed lockfile; the parse or the workspace is wrong",
        );

    // Without this, "the facade depends on nothing forbidden" would also be
    // what an unresolvable package name reports. The edge named here is the one
    // the split exists to place, so its absence means the split is gone rather
    // than satisfied.
    for forbidden in FACADE_FORBIDDEN_DEPENDENCIES {
        assert!(
            macros.dependencies.iter().any(|dep| dep == forbidden),
            "`tiler-macros` no longer depends on `{forbidden}`, so this test is asserting the \
             absence of an edge nothing in the frontend holds; re-derive the placement before \
             trusting it"
        );
    }

    let offenders: Vec<&String> = facade
        .dependencies
        .iter()
        .filter(|dep| FACADE_FORBIDDEN_DEPENDENCIES.contains(&dep.as_str()))
        .collect();

    assert!(
        offenders.is_empty(),
        "the facade must not put a host-only build-time dependency into every consumer's build \
         graph, but it depends on: {offenders:?}"
    );
}

/// Extracts every `[[package]]` block's name and direct dependency names.
///
/// The lockfile grammar this relies on is narrow and stable: `[[package]]`
/// opens a block, `name = "…"` names it, and `dependencies = [` opens a list
/// of one quoted entry per line terminated by `]`. Anything else, including a
/// non-package table, closes the block being read.
fn parse_lock_packages(lock: &str) -> Vec<LockPackage> {
    let mut packages = Vec::new();
    let mut current: Option<LockPackage> = None;
    let mut in_dependencies = false;

    for line in lock.lines() {
        let trimmed = line.trim();

        if in_dependencies {
            if trimmed == "]" {
                in_dependencies = false;
            } else if let Some(package) = current.as_mut() {
                package.dependencies.push(dependency_name(trimmed));
            }
            continue;
        }

        if trimmed.starts_with('[') {
            packages.extend(current.take());
            if trimmed == "[[package]]" {
                current = Some(LockPackage {
                    name: String::new(),
                    dependencies: Vec::new(),
                });
            }
            continue;
        }

        let Some(package) = current.as_mut() else {
            continue;
        };
        if let Some(name) = trimmed.strip_prefix("name = ") {
            unquote(name).clone_into(&mut package.name);
        } else if trimmed == "dependencies = [" {
            in_dependencies = true;
        }
    }

    packages.extend(current);
    packages
}

/// Reads one dependency entry, dropping the version Cargo appends when a
/// package is resolved at more than one version.
fn dependency_name(entry: &str) -> String {
    let unquoted = unquote(entry.trim_end_matches(','));
    unquoted
        .split_whitespace()
        .next()
        .unwrap_or(unquoted)
        .to_owned()
}

/// Strips the surrounding quotes from a lockfile string value.
fn unquote(value: &str) -> &str {
    value.trim().trim_matches('"')
}
