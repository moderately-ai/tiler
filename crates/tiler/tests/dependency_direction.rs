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
