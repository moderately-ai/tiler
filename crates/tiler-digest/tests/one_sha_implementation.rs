//! One SHA implementation in the Cargo workspace, and four callers that reach it.
//!
//! [ADR 0111](../../../docs/decisions/0111-separate-externally-specified-raw-hashes-from-governed-tiler-digests.md)
//! deleted four handwritten SHA-256 implementations from Cargo-workspace members
//! and routed their subjects through
//! `DigestAlgorithm::digest_external_record`. Nothing in the type system stops
//! them coming back: a copied compression function compiles anywhere, and a
//! caller that reverted to `GOVERNED.digest(b"", ...)` would produce identical
//! bytes today and pass every existing test. This file is what says no to both.
//!
//! # Why the census lives here rather than in `crates/tiler`
//!
//! `crates/tiler/tests/` already holds the workspace-wide source inventories —
//! the lint-inheritance reader and the unsafe-site walker — and a third would
//! sort naturally beside them. It is here instead because the property is this
//! crate's own: being the only place that maps the governed tag to an
//! implementation is what the crate documentation calls "the whole point", and
//! the check that holds it belongs with the claim rather than with the other
//! censuses. Reading sibling sources is a file read, not a dependency; this
//! crate's manifest is unchanged and it remains the bottom of the graph.
//!
//! # What would make this say no
//!
//! Each assertion below is reachable, and the reachability is stated because a
//! census that cannot fail is worse than none — it reports a population as clean
//! when it has stopped looking. Reintroducing any compression constant or `sha2`
//! reach outside this crate fails [`the_only_sha_implementation_is_this_crate`];
//! reverting a migrated caller to a local helper or to the empty-domain governed
//! spelling fails [`the_four_migrated_callers_still_name_the_external_path`]; and
//! a member list, directory layout, or file population that has quietly stopped
//! being walked fails the floor in either.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The four Cargo-workspace files ADR 0111 migrated, workspace-relative.
///
/// Written out rather than discovered, because the property is about *these*
/// four subjects: a discovered set would shrink silently to whatever still
/// matched. A fifth external-record consumer is a deliberate edit here, and
/// deleting one of these files fails [`existing`] rather than passing quietly.
const MIGRATED_CALLERS: [&str; 4] = [
    "crates/tiler-compiler/src/governed/contraction_conformance.rs",
    "crates/tiler-conformance/src/envelope.rs",
    "crates/tiler-reference/tests/contraction_profile_cells.rs",
    "prototypes/serial-sum-run/src/proof.rs",
];

/// The crate that owns the algorithm, and the only directory exempt below.
const OWNER: &str = "crates/tiler-digest";

/// Fragments that mean a file implements or directly reaches SHA-256.
///
/// The two constants are the SHA-256 initial hash value and the first round
/// constant. Both are written here split across a `concat!` so that *this* file
/// does not match its own patterns — the census excludes `crates/tiler-digest`,
/// but a pattern that matched the file stating it would be a check whose only
/// evidence was itself.
///
/// Underscore digit separators are why the fragments are short. The four deleted
/// copies spelled the initial value `0x6a09_e667` while the standalone spikes
/// spell it `0x6a09e667u32`, so a pattern anchored on either full spelling
/// silently misses the other. Matching the two halves independently catches both
/// and any future separator placement between them.
///
/// Matched against [`code_only`], so every fragment is written whitespace-free:
/// `use sha2` is spelled `usesha2` because the view these run over has had its
/// whitespace removed. A fragment carrying a space would match nothing at all,
/// which is the silent direction, so they are asserted to be space-free below.
fn implementation_markers() -> [String; 6] {
    let markers = [
        concat!("0x6a09", "_e667").to_owned(),
        concat!("0x6a09", "e667").to_owned(),
        concat!("0x428a", "_2f98").to_owned(),
        concat!("use", "sha2").to_owned(),
        concat!("externcrate", "sha2").to_owned(),
        concat!("sha2", "::").to_owned(),
    ];
    for marker in &markers {
        assert!(
            !marker.contains(char::is_whitespace),
            "the marker {marker:?} carries whitespace and is matched against a whitespace-free \
             view, so it could never match and would report a clean census by never looking",
        );
    }
    markers
}

/// The workspace root, checked against what it should contain.
///
/// Walking up two levels is only correct while this crate sits at
/// `crates/tiler-digest`, so the manifest it reaches is verified to declare a
/// workspace listing this crate. A layout change then surfaces as a named
/// failure rather than as a census over whatever directory that path reached.
fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("this crate's manifest directory sits two levels below the workspace root")
        .to_path_buf();
    let manifest = root.join("Cargo.toml");
    let text = read(&manifest);
    assert!(
        text.contains("[workspace]"),
        "{} declares no workspace, so walking up two directories from this crate no longer \
         reaches the root manifest and the census below would read the wrong tree",
        manifest.display(),
    );
    assert!(
        text.contains(&format!("\"{OWNER}\"")),
        "{} does not list \"{OWNER}\" among its members, so the manifest this census treats as \
         the governing workspace does not govern this crate",
        manifest.display(),
    );
    root
}

/// Every Cargo-workspace member directory, read from the root manifest.
///
/// **Derived rather than listed**, which is what stops the population shrinking
/// silently: a member added to `Cargo.toml` is scanned by this census on its
/// first run, with no edit here, and a hand-written list would have admitted it
/// unexamined. The parse is deliberately narrow — the quoted entries of the
/// `members` array — and asserts a floor, so a manifest whose shape moved past
/// this reader reports an empty population instead of a clean one.
fn member_directories(root: &Path) -> Vec<PathBuf> {
    let text = read(&root.join("Cargo.toml"));
    let after = text
        .split_once("members = [")
        .expect("the root manifest declares a workspace member array")
        .1;
    let body = after
        .split_once(']')
        .expect("the workspace member array is closed")
        .0;

    let mut members: Vec<PathBuf> = Vec::new();
    for entry in body.split(',') {
        let trimmed = entry.trim();
        let Some(quoted) = trimmed
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix('"'))
        else {
            continue;
        };
        members.push(root.join(quoted));
    }

    assert!(
        members.len() >= 16,
        "the root manifest parsed to {} workspace member(s), which is fewer than the sixteen \
         this census was written against — the member array's shape has moved past this reader \
         and the scan below would cover a population that is smaller than the workspace",
        members.len(),
    );
    for member in &members {
        assert!(
            member.join("Cargo.toml").is_file(),
            "{} parsed out of the workspace member array but holds no manifest, so the parse \
             above is producing paths rather than members",
            member.display(),
        );
    }
    members
}

/// Every `.rs` file under the Cargo-workspace members, workspace-relative.
///
/// `target/` directories are skipped because they hold build output rather than
/// workspace-authored source; nothing else is filtered, so a generated or
/// vendored file inside a member is scanned like any other.
fn member_sources(root: &Path) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for member in member_directories(root) {
        collect(&member, root, &mut found);
    }
    assert!(
        found.len() >= 200,
        "the walk found {} Rust source file(s) under the workspace members, which is far below \
         the population this census was written against — it is reading the wrong tree or the \
         recursion has stopped descending, and either would report an empty census as a clean one",
        found.len(),
    );
    found
}

fn collect(directory: &Path, root: &Path, found: &mut BTreeSet<String>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", directory.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|error| panic!("{} enumerates: {error}", directory.display()))
            .path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "target") {
                continue;
            }
            collect(&path, root, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.insert(relative(&path, root));
        }
    }
}

fn relative(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .expect("every scanned path lies under the workspace root")
        .to_string_lossy()
        .replace('\\', "/")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", path.display()))
}

/// One file's non-comment lines with all whitespace removed.
///
/// **Both halves of that were paid for by a perturbation this census failed to
/// catch.** Substituting `DigestAlgorithm::GOVERNED.digest(b"tiler...\0", ...)`
/// into a migrated caller made its retained comparison fail, as intended — and
/// left every assertion here green, because rustfmt had wrapped the call so that
/// `GOVERNED` and `.digest` sat on different lines and no single-line anchor
/// spanned them. Removing whitespace makes the anchor immune to where a
/// formatter chose to break, which is the failure AGENTS.md names for a matcher
/// that cannot see a construct that wraps.
///
/// Dropping comment lines is the other half. Each migrated file's header now
/// explains *why* the governed alias is the wrong authority for an external
/// record, so a census over raw text fails on the correction notes themselves —
/// a check satisfiable only by deleting its own explanation. A comment cannot
/// compute a digest, so excluding it costs nothing the census is for.
///
/// This is a lexical filter and not a parser: a `//` inside a string literal is
/// treated as a comment start, and a block comment is not recognized at all.
/// Both directions of that error are safe here. Over-stripping can only hide a
/// marker inside a string, and no marker below has a legitimate string use in a
/// member; under-stripping a block comment can only produce a loud failure that
/// a reader resolves by looking.
fn code_only(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

/// Reads one file that must exist, naming it if it does not.
///
/// Separate from [`read`] so a deleted migrated caller reports as a missing
/// subject rather than as an I/O error a reader would have to interpret.
fn existing(root: &Path, relative_path: &str) -> String {
    let path = root.join(relative_path);
    assert!(
        path.is_file(),
        "{relative_path} is named by this census as an ADR 0111 external-record caller but does \
         not exist; if the file moved, move the entry, because a census over a path that is gone \
         checks nothing",
    );
    read(&path)
}

/// No Cargo-workspace member outside `tiler-digest` implements or reaches SHA-256.
///
/// This is ADR 0111's structural claim and the one a copied helper defeats
/// silently: four byte-identical transcriptions passed their own vector checks
/// for months while adding three algorithm authorities. The census names its
/// population and its exemption so that "nothing matched" cannot be produced by
/// a walk that reached nothing.
///
/// The `spikes/` trees are outside the scan deliberately, and by construction
/// rather than by exclusion: they are not Cargo-workspace members, so the member
/// array above never reaches them. ADR 0111 records them as standalone
/// experimental producers whose local hashes stay; a spike promoted to a member
/// enters this census on the commit that adds it.
#[test]
fn the_only_sha_implementation_is_this_crate() {
    let root = workspace_root();
    let sources = member_sources(&root);
    let markers = implementation_markers();

    let mut offenders: Vec<String> = Vec::new();
    for source in &sources {
        if source.starts_with(OWNER) {
            continue;
        }
        let code = code_only(&read(&root.join(source)));
        for marker in &markers {
            if code.contains(marker.as_str()) {
                offenders.push(format!("{source} contains {marker}"));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "the Cargo workspace must hold exactly one SHA implementation, in {OWNER}, and these \
         source(s) reach or transcribe another:\n  {}\n\
         Route the subject through `DigestAlgorithm::digest_external_record` (an externally \
         specified raw record) or `digest`/`digest_qualified` (a governed Tiler subject) instead; \
         ADR 0111 is the accepted record.",
        offenders.join("\n  "),
    );

    // Print the population rather than only its verdict, so a run that scanned
    // less than it should is visible in the log instead of merely green.
    println!(
        "CENSUS {} Rust source file(s) across the Cargo-workspace members, {} outside {OWNER}",
        sources.len(),
        sources.iter().filter(|s| !s.starts_with(OWNER)).count(),
    );
}

/// The four migrated callers still reach the external path, and only that path.
///
/// The negative census above cannot see this. A caller reverted to
/// `GOVERNED.digest(b"", bytes)` transcribes nothing and reaches `sha2` through
/// no name of its own — it just produces the right bytes through the wrong
/// authority, which is exactly the substitution ADR 0111 rejected: `GOVERNED`
/// tracks whatever algorithm this build of Tiler writes, while every retained
/// `CC_SHA256` record means SHA-256 permanently.
///
/// Both halves are asserted per file. The positive half fails if a caller stops
/// using the external path at all; the negative half fails if it acquires the
/// governed alias or an empty-domain spelling beside it.
///
/// The anchors run over [`code_only`], which is what makes the bare name usable:
/// every migrated file's header now explains *why* the alias is the wrong
/// authority for an external record, so a raw-text anchor on `GOVERNED` failed on
/// the four correction notes rather than on any call. Excluding comments admits
/// the strongest anchor — the alias may not appear in these files' code at all —
/// instead of the narrower `GOVERNED.digest`, which a wrapped call slips past.
#[test]
fn the_four_migrated_callers_still_name_the_external_path() {
    let root = workspace_root();
    let sources = member_sources(&root);

    for caller in MIGRATED_CALLERS {
        assert!(
            sources.contains(caller),
            "{caller} is not among the Rust sources this census walked, so the assertions below \
             would read a file the workspace scan does not cover",
        );

        let code = code_only(&existing(&root, caller));
        assert!(
            code.contains("digest_external_record"),
            "{caller} reproduces an externally specified raw digest record and must reach it \
             through `DigestAlgorithm::digest_external_record`; it no longer names that path",
        );
        assert!(
            !code.contains("GOVERNED"),
            "{caller} names `DigestAlgorithm::GOVERNED` in code, which means the algorithm this \
             build of Tiler writes. The record it reproduces means SHA-256 permanently, so the \
             variant must be spelled `Sha256` — ADR 0111 rejects the alias at exactly this call.",
        );
        assert!(
            !code.contains(r#"digest(b"""#),
            "{caller} digests under the empty domain, which spells a raw external subject as a \
             governed Tiler one and publishes the empty-domain convention ADR 0111 refuses. Use \
             `digest_external_record`, whose result type keeps the two subjects apart.",
        );
    }

    println!(
        "CENSUS {} migrated caller(s) reach the external path and name no governed alias",
        MIGRATED_CALLERS.len(),
    );
}
