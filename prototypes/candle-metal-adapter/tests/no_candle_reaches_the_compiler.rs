//! No Candle type can reach `tiler-compiler` or `tiler-ir`, checked on the graph.
//!
//! This is `prototype-candle-metal-adapter`'s criterion 7, and the ticket is
//! explicit that it must be reproduced with a dependency check rather than by
//! inspection: a working prototype is exactly the thing that most easily
//! violates it, and a reviewer reading two manifests cannot see a transitive
//! edge.
//!
//! `Cargo.lock` is the authority read here rather than the manifests, because it
//! is what Cargo actually resolved and it merges normal, build, and development
//! dependencies into one edge list per package. Unlike the frontend's
//! direct-edge check, this one walks the **transitive** closure: the claim is
//! not that the compiler names Candle, which nobody would write, but that no
//! path of any length reaches it.
//!
//! The direction that matters is the one this file asserts. This prototype
//! depending on `tiler-compiler` — which it does, through `tiler-build` — is the
//! consumer-agnostic architecture working: a consumer may depend on the
//! compiler, and the compiler may not depend on a consumer's framework.

use std::collections::{BTreeSet, VecDeque};

/// The packages whose dependency closure must never contain a Candle package.
const CONSUMER_NEUTRAL_PACKAGES: [&str; 2] = ["tiler-compiler", "tiler-ir"];

/// Package-name prefixes that are Candle or its Metal binding.
///
/// Prefixes rather than exact names, so `candle-nn`, `candle-transformers`, or
/// any future member of the family is caught without this list being maintained
/// against theirs. `objc2-metal` and `dispatch2` are here because they are the
/// Objective-C binding this adapter reaches Metal through: a compiler crate that
/// acquired one would be naming a runtime object, which is the same coupling
/// under a different name.
const CONSUMER_PACKAGE_PREFIXES: [&str; 3] = ["candle-", "objc2-metal", "dispatch2"];

/// The package whose closure must contain Candle, or this check proves nothing.
const ADAPTER_PACKAGE: &str = "tiler-prototype-candle";

/// One `[[package]]` block's name and its direct dependency names.
struct LockPackage {
    name: String,
    dependencies: Vec<String>,
}

/// Neither consumer-neutral crate reaches a Candle package by any path.
#[test]
fn no_candle_package_is_in_the_compiler_or_ir_closure() {
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.lock"))
        .expect("the workspace lockfile is readable from the adapter prototype");
    let packages = parse_lock_packages(&lock);

    // Without these three the check would also pass for an empty lockfile, a
    // misparsed one, or a workspace where nothing depends on Candle at all —
    // and a check that cannot say no is not evidence. Each names its population
    // before the absence below is trusted.
    let names: BTreeSet<&str> = packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    for required in CONSUMER_NEUTRAL_PACKAGES {
        assert!(
            names.contains(required),
            "`{required}` is not in the parsed lockfile; the parse or the workspace is wrong, not \
             the dependency direction",
        );
    }
    assert!(
        names.iter().any(|name| is_consumer_package(name)),
        "no Candle package is in the parsed lockfile at all, so this test would pass whether or \
         not the compiler depended on one",
    );
    let adapter_closure = closure(&packages, ADAPTER_PACKAGE);
    assert!(
        adapter_closure.iter().any(|name| is_consumer_package(name)),
        "`{ADAPTER_PACKAGE}`'s own closure contains no Candle package, so the closure walk is not \
         finding the edges this test is about",
    );

    for neutral in CONSUMER_NEUTRAL_PACKAGES {
        let offenders: Vec<&String> = closure(&packages, neutral)
            .into_iter()
            .filter(|name| is_consumer_package(name))
            .collect();
        assert!(
            offenders.is_empty(),
            "`{neutral}` must stay consumer-agnostic, and its resolved dependency closure \
             contains: {offenders:?}",
        );
    }
}

/// Returns whether a package name is Candle's or its Metal binding's.
fn is_consumer_package(name: &str) -> bool {
    CONSUMER_PACKAGE_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

/// Returns every package reachable from `root` by dependency edges of any length.
///
/// Breadth-first over the resolved graph, with a visited set, so a cycle in the
/// development-dependency edges the lockfile merges cannot hang the walk. The
/// root itself is excluded from the result, because a package trivially reaches
/// itself and including it would only complicate the assertions.
fn closure<'a>(packages: &'a [LockPackage], root: &str) -> BTreeSet<&'a String> {
    let mut reached = BTreeSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(root.to_owned());
    let mut seen: BTreeSet<String> = BTreeSet::new();
    seen.insert(root.to_owned());
    while let Some(current) = queue.pop_front() {
        for package in packages.iter().filter(|package| package.name == current) {
            for dependency in &package.dependencies {
                if seen.insert(dependency.clone()) {
                    reached.insert(dependency);
                    queue.push_back(dependency.clone());
                }
            }
        }
    }
    reached
}

/// Extracts every `[[package]]` block's name and direct dependency names.
///
/// The lockfile grammar this relies on is narrow and stable: `[[package]]` opens
/// a block, `name = "…"` names it, and `dependencies = [` opens a list of one
/// quoted entry per line terminated by `]`. Anything else, including a
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

/// Reads one dependency entry, dropping the version Cargo appends when a package
/// is resolved at more than one version.
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
