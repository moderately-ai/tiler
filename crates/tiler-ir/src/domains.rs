//! Every `tiler.`-spelled literal this crate declares, pinned to exact bytes.
//!
//! # The property
//!
//! An identity domain is the separator a canonical encoding opens with, so it is
//! the one value that decides whether two encodings are two subjects or one. A
//! domain that is reverted, mistyped, or left un-stepped does not fail: it
//! quietly republishes a new subject under an old name, or splits one subject in
//! two. Nothing about the value announces itself — it is a string constant, and
//! a string constant that no test names can move in any direction.
//!
//! Several domains here are covered *indirectly*, by a digest golden that folds
//! them: reverting `tiler.semantic-graph.v3` to `v2` reddens two recorded
//! identities in `tiler-build` and `tiler-compiler`, and reverting
//! `tiler.index-region.v11` to `v10` reddens four across three crates. That
//! coverage is real but incidental — it exists where someone happened to pin a
//! digest over a subject that folds the domain, and it says "a digest moved"
//! rather than "a domain moved". Where no golden folds a domain, there is no
//! coverage at all: reverting `tiler.semantic-precondition-obligation.v2` to
//! `v1` passed the whole workspace suite, 3,184 tests, at commit
//! `c0829b41ad3eeea7a98aa2a25a18de547882491e`.
//!
//! # Why the population is read from the source rather than sized from a type
//!
//! [`variant_count`](core::mem::variant_count) sizes an enumeration from a type,
//! which is what `tiler-artifact`'s `domains` module does — every domain that
//! crate admits is a `const … : &[u8]` it can name, so a type can enumerate
//! them. That does not hold here. This crate spells identity domains three
//! different ways: named `&[u8]` constants (`INDEX_REGION_DOMAIN`), named
//! constants that are not called `_DOMAIN` at all (`RECEIPT_IDENTITY_TAG`,
//! `EXHAUSTIVE_DERIVATION`), and **inline literals with no constant behind
//! them** — `tiler.schedule.v6` is written directly into
//! `schedule::model::encode_identity`, and `tiler.resolved-value-type.v3`
//! appears at three separate sites. Fifteen of the sixty spellings pinned below
//! are of that third kind, so an enum could name at most forty-five of them
//! while reporting a complete population. A type cannot enumerate a literal that
//! no constant names, and an enumeration that cannot reach a quarter of its
//! subject is the failure this module exists to rule out rather than a way to
//! rule it out.
//!
//! So the population is the crate's own sources, and `AGENTS.md`'s instruction
//! for a population that cannot be typed applies: assert a floor and print the
//! census. Two assertions do the work, and they fail in opposite directions:
//!
//! 1. [`every_tiler_spelled_literal_is_pinned_or_classified`] walks every
//!    `tiler.`-spelled literal in `src/` and `tests/` and requires each to be
//!    pinned below or admitted by a classified non-domain prefix. A domain
//!    changed, mistyped, or newly added fails here.
//! 2. [`every_pinned_identity_domain_still_appears_in_the_source`] requires each
//!    pinned spelling to still be present. A domain reverted or deleted fails
//!    here. **This is the one a revert trips**, and it is why the pin compares
//!    bytes rather than lengths: the `v1` to `v2` step of the obligation domain
//!    left two of its three subjects the same length, because a rank-zero
//!    boundary has no extent to tag and `v1` and `v2` are the same width.
//!
//! A revert trips both, because the old spelling is unpinned and the new one is
//! absent.
//!
//! # What it would take for this to say *no* when every domain is correct
//!
//! Nothing, and that is checkable rather than hopeful. Both assertions quantify
//! over a difference that is empty when the pin matches the source — found
//! minus pinned, and pinned minus found — so a correct tree walks both
//! populations and reports no member. The reachable failures are exactly: a
//! spelling that moved in either direction, a `tiler.`-spelled literal in a
//! namespace nobody has classified, a source tree that shrank below the file
//! floor, and a pin table that is unsorted, duplicated, or shadowed by an
//! over-broad prefix. Each is exercised on this module's ticket.
//!
//! # Stepping a domain deliberately
//!
//! One edit beyond the ones the step already required: add the new spelling to
//! [`PINNED_IDENTITY_DOMAINS`] and remove the old one. Both assertions name the
//! file, the spelling, and this table in their failure text, so the edit is
//! located rather than searched for.
//!
//! # What this module declines to read, and why that is not silent
//!
//! The scan reads string and byte-string literals whose content opens `tiler.`.
//! It declines three things. A literal inside a `//` comment is skipped, because
//! prose quotes domains constantly and a doc comment is not a declaration —
//! `tiler.doc.strict-f32` in `schedule/mod.rs` is one such. A spelling outside
//! the `tiler.` namespace is out of population, which excludes
//! `CANONICAL_ARITHMETIC_NAN_PROFILE` (`tiler::canonical-arithmetic-nan-f32@1`,
//! a `tiler::`-spelled vocabulary key, not an identity separator) and the
//! relation vocabularies spelled as bare words. And an escape the unescaper does
//! not know panics rather than being passed through, because a literal read as
//! the wrong bytes is a false verdict rather than a gap.
//!
//! Everything else in the namespace is accounted for in one of the two tables.
//! [`ADMITTED_NON_DOMAIN_PREFIXES`] is what keeps the second group from being a
//! silent exclusion: a diagnostic code or a test fixture is admitted by its
//! namespace, but a literal in a namespace nobody has classified fails, so
//! "unrecognised" and "absent" cannot look the same. No admitted prefix may be a
//! prefix of a pinned spelling, which is asserted below — otherwise a broad
//! prefix could quietly swallow a pin.

use std::path::{Path, PathBuf};

/// Every identity domain spelling this crate declares, in exact bytes.
///
/// Sorted by content and free of duplicates, both asserted below. The rows carry
/// their NUL terminator where the declaration does, because the pin is over the
/// bytes an encoder writes rather than over a readable name; the three that do
/// not are the two numerical-contract key domains, which are `&str` and are
/// composed with a suffix rather than framed, and `tiler.scalar`, which is a
/// length-framed namespace run inside a realization law's key.
///
/// A spelling appears once however many sites write it. Some rows are
/// superseded spellings a test restates deliberately, and they are pinned for
/// the same reason the live ones are: `v6` through `v10` of
/// `tiler.kernel-program` sit beside the live `v11` because `program::tests`
/// names them to prove the six stay distinct, and
/// `tiler.ir.index-refinement-subject.v1` sits beside `v2` because
/// `LEGACY_SUBJECT_IDENTITY_TAG` — itself `#[cfg(test)]` — is what
/// `index::refinement`'s test re-encodes under to show the step separated two
/// occurrences that `v1` collapsed.
const PINNED_IDENTITY_DOMAINS: &[&[u8]] = &[
    b"tiler.accuracy-contract.v1\0",
    b"tiler.accuracy-domain.v1\0",
    b"tiler.accuracy-predicate.v1\0",
    b"tiler.artifact-program.abi-expr.v1\0",
    b"tiler.broadcast-axis-mapping.v2\0",
    b"tiler.conformance-evidence.v1\0",
    b"tiler.contract.bf16.v1",
    b"tiler.contract.f32.v2",
    b"tiler.contraction-index-structure.v1\0",
    b"tiler.index-domain-obligation-key.v1\0",
    b"tiler.index-region.v11\0",
    b"tiler.index.access-read.alpha.v1\0",
    b"tiler.index.access-read.v1\0",
    b"tiler.index.reducer-apply.v2\0",
    b"tiler.index.scalar-operation.alpha.v1\0",
    b"tiler.index.scalar-operation.v2\0",
    b"tiler.ir.exact-index-domain-enumeration.v1\0",
    b"tiler.ir.index-domain-counterexample.v1\0",
    b"tiler.ir.index-realization-authority.v1\0",
    b"tiler.ir.index-realization-law-registry.v1\0",
    b"tiler.ir.index-realization-resolution.v1\0",
    b"tiler.ir.index-refinement-coverage-graph.v1\0",
    b"tiler.ir.index-refinement-domain-proof.v1\0",
    b"tiler.ir.index-refinement-executable-coverage.v2\0",
    b"tiler.ir.index-refinement-receipt.v1\0",
    b"tiler.ir.index-refinement-staged-executable-coverage.v2\0",
    b"tiler.ir.index-refinement-staged-receipt.v1\0",
    b"tiler.ir.index-refinement-subject.v1\0",
    b"tiler.ir.index-refinement-subject.v2\0",
    b"tiler.ir.index-region-sequence.v1\0",
    b"tiler.kernel-program.abi-arena.v1\0",
    b"tiler.kernel-program.allocation.v1\0",
    b"tiler.kernel-program.stage.v2\0",
    b"tiler.kernel-program.v10\0",
    b"tiler.kernel-program.v11\0",
    b"tiler.kernel-program.v6\0",
    b"tiler.kernel-program.v7\0",
    b"tiler.kernel-program.v8\0",
    b"tiler.kernel-program.v9\0",
    b"tiler.kernel-program.value.v1\0",
    b"tiler.kernel-program.view.v1\0",
    b"tiler.kernel.v8\0",
    b"tiler.prepared-entry-target-requirement.v1\0",
    b"tiler.reindex-form.v1\0",
    b"tiler.resolved-value-type.v3\0",
    b"tiler.scalar",
    b"tiler.scalar-admission-provenance.v1\0",
    b"tiler.scalar-definition-projection.v2\0",
    b"tiler.scalar-registry-snapshot.v1\0",
    b"tiler.schedule.v6\0",
    b"tiler.semantic-admission-provenance.v1\0",
    b"tiler.semantic-definition-projection.v6\0",
    b"tiler.semantic-graph.v3\0",
    b"tiler.semantic-precondition-obligation.v2\0",
    b"tiler.semantic-registry.v8\0",
    b"tiler.shape-env.v3\0",
    b"tiler.slice-selection.v1\0",
    b"tiler.target-property-query.v1\0",
    b"tiler.value-conformance-evidence.v1\0",
    b"tiler.value-type-descriptor.v1\0",
];

/// Namespaces of `tiler.`-spelled literals that are not identity domains.
///
/// A literal opening with one of these is admitted without a pin, because these
/// namespaces hold diagnostic codes, registry keys, and test fixtures that are
/// added routinely and separate no subjects. The table exists so that admission
/// is a decision recorded here rather than a silence: a literal in an
/// unclassified namespace fails.
///
/// Two rows carry a version deliberately. The numerical-contract key goldens in
/// `schedule::numerics` open with their whole domain and continue with an
/// encoded key, so classifying them by `tiler.contract.` would admit a reverted
/// domain as well; the prefixes name the exact live domain plus its `.`
/// separator instead, which makes each golden a second reading of the same pin.
/// Stepping either domain therefore moves its row here as well as its row above.
///
/// Sorted and free of duplicates, and no row is a prefix of a pinned spelling —
/// all asserted below.
const ADMITTED_NON_DOMAIN_PREFIXES: &[&[u8]] = &[
    // Reference-conformance keys naming a governed scalar operation.
    b"tiler.conformance.",
    // The `bf16` numerical-contract key golden, under its live domain.
    b"tiler.contract.bf16.v1.",
    // The `f32` numerical-contract key golden, under its live domain.
    b"tiler.contract.f32.v2.",
    // Diagnostic codes a provider implementation can be rejected with.
    b"tiler.provider.",
    // Reference-authority keys used by the numerics tests.
    b"tiler.reference.",
    // Scalar-registry keys, limits, and diagnostic codes.
    b"tiler.scalar.",
    // Operation-schema diagnostic codes.
    b"tiler.schema.",
    // Shape diagnostic codes.
    b"tiler.shape.",
    // Target-property query keys.
    b"tiler.target.",
    // Test fixtures, including the test-only digest domain `index::law` uses to
    // keep its identity pins away from every governed domain.
    b"tiler.test",
    // Value-conformance diagnostic codes.
    b"tiler.value-conformance.",
];

/// Fewest `.rs` files the walk may find before its verdict is about the walk.
///
/// The crate had 130 across `src/` and `tests/` when this landed. A floor rather
/// than the exact count so that adding a file is not a failure, and a floor at
/// all so that a walk which stopped finding files cannot report an empty
/// population as an intact one.
const MINIMUM_SOURCE_FILES: usize = 100;

/// One `tiler.`-spelled literal, with where the scan read it.
#[derive(Clone, Debug)]
struct FoundLiteral {
    /// Path of the file the literal was read from.
    path: PathBuf,
    /// One-based line the literal opens on.
    line: usize,
    /// The literal's content, with its escapes resolved.
    content: Vec<u8>,
}

/// Every `tiler.`-spelled literal in this crate's `src/` and `tests/` trees.
///
/// This module's own source is excluded, and its absence is a failure rather
/// than a skip: the pin table below *is* a file full of the spellings it pins,
/// so a scan that read it would find every pinned domain in the pin itself and
/// [`every_pinned_identity_domain_still_appears_in_the_source`] would pass
/// vacuously. Renaming this file therefore fails here rather than quietly
/// hollowing that assertion out.
fn scan_crate_sources() -> Vec<FoundLiteral> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for tree in ["src", "tests"] {
        collect_rust_sources(&root.join(tree), &mut files);
    }
    files.sort();
    assert!(
        files.len() >= MINIMUM_SOURCE_FILES,
        "the walk found {} `.rs` file(s) across `src/` and `tests/`, fewer than the \
         {MINIMUM_SOURCE_FILES} this crate has. A walk that stopped finding files reports an \
         empty population as an intact one, so this is a verdict about the walk rather than \
         about the crate.",
        files.len(),
    );

    let this_module = root.join("src").join("domains.rs");
    let before = files.len();
    files.retain(|path| path != &this_module);
    assert_eq!(
        files.len() + 1,
        before,
        "the walk did not find this module at {}, so it removed nothing. The pin table in this \
         file restates every spelling it pins, so a scan that read it would satisfy the \
         presence assertion from the pin alone.",
        this_module.display(),
    );

    let mut found = Vec::new();
    for path in &files {
        let text = std::fs::read_to_string(path).expect("a crate source file is readable");
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            read_literals_opening_on(path, &lines, index, line, &mut found);
        }
    }
    found
}

/// Reads every `tiler.`-spelled literal opening on one line into `found`.
///
/// A literal opening inside a `//` comment is skipped; see the module header for
/// why prose is out of population.
fn read_literals_opening_on(
    path: &Path,
    lines: &[&str],
    index: usize,
    line: &str,
    found: &mut Vec<FoundLiteral>,
) {
    // Assembled at run time so that a future scan of this crate by the same
    // rules does not read the needle as a declaration of the namespace.
    let needle = format!("{}tiler.", '"');
    let mut from = 0_usize;
    while let Some(offset) = line[from..].find(needle.as_str()) {
        let at = from + offset;
        from = at + 1;
        if line[..at].contains("//") {
            continue;
        }
        found.push(FoundLiteral {
            path: path.to_path_buf(),
            line: index + 1,
            content: read_literal(path, lines, index, &line[at + 1..]),
        });
    }
}

/// Resolves one literal's content, following `\` line continuations.
///
/// A continuation is not a corner case here: both numerical-contract key
/// goldens are written across two lines, and a single-line reader would report
/// them as unterminated or read only their first half.
///
/// # Panics
///
/// On an unterminated literal, an escaped quote the scan cannot delimit on, or
/// an escape [`unescape`] does not resolve. Each would make the scan compare the
/// wrong bytes, which is a false verdict rather than a gap.
fn read_literal(path: &Path, lines: &[&str], index: usize, opening: &str) -> Vec<u8> {
    let mut raw = String::new();
    let mut rest = opening.to_owned();
    let mut cursor = index;
    loop {
        if let Some(closing) = rest.find('"') {
            assert!(
                !rest[..closing].ends_with('\\'),
                "{}:{}: a `tiler.`-spelled literal carries an escaped quote, which this scan \
                 delimits on. Teach it the escape rather than letting it compare a truncated \
                 literal against the pin.",
                path.display(),
                index + 1,
            );
            raw.push_str(&rest[..closing]);
            return unescape(path, index, &raw);
        }
        let trimmed = rest.trim_end();
        assert!(
            trimmed.ends_with('\\'),
            "{}:{}: a `tiler.`-spelled literal is unterminated and does not continue with `\\`.",
            path.display(),
            index + 1,
        );
        raw.push_str(&trimmed[..trimmed.len() - 1]);
        cursor += 1;
        rest = (*lines.get(cursor).unwrap_or_else(|| {
            panic!("{}:{}: literal runs past the file", path.display(), cursor)
        }))
        .trim_start()
        .to_owned();
    }
}

/// Resolves the escapes admitted in a `tiler.`-spelled literal.
///
/// Only `\0` occurs today, and an unrecognised escape panics rather than being
/// passed through, for the reason `tiler-artifact`'s equivalent does: a literal
/// this function misread would be compared against the pin as the wrong bytes.
///
/// # Panics
///
/// On an escape it does not resolve, or on a trailing backslash.
fn unescape(path: &Path, index: usize, literal: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(literal.len());
    let mut characters = literal.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            let mut buffer = [0_u8; 4];
            bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            continue;
        }
        match characters.next() {
            Some('0') => bytes.push(0),
            Some(other) => panic!(
                "{}:{}: the literal {literal:?} carries the escape `\\{other}`, which this scan \
                 does not resolve; teach it the escape rather than comparing the wrong bytes",
                path.display(),
                index + 1,
            ),
            None => panic!(
                "{}:{}: the literal {literal:?} ends in a trailing backslash",
                path.display(),
                index + 1,
            ),
        }
    }
    bytes
}

/// Collects every Rust source file under one directory, recursively.
fn collect_rust_sources(directory: &Path, into: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory).expect("the directory is readable");
    for entry in entries {
        let path = entry.expect("a directory entry is readable").path();
        if path.is_dir() {
            collect_rust_sources(&path, into);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            into.push(path);
        }
    }
}

/// Renders one spelling the way its declaration writes it.
fn render(content: &[u8]) -> String {
    let mut rendered = String::new();
    for byte in content {
        if *byte == 0 {
            rendered.push_str("\\0");
        } else {
            rendered.push(char::from(*byte));
        }
    }
    rendered
}

/// Every `tiler.`-spelled literal is pinned or classified as a non-domain.
///
/// The direction that catches a domain **changed** — mistyped, stepped without
/// its pin moving, or newly declared. The reverted spelling is what appears in
/// the source, and it is not in the pin.
#[test]
fn every_tiler_spelled_literal_is_pinned_or_classified() {
    let found = scan_crate_sources();
    assert!(
        found.len() >= PINNED_IDENTITY_DOMAINS.len(),
        "the scan read {} `tiler.`-spelled literal(s), fewer than the {} this module pins. The \
         scan has stopped recognising literals it once read, so its verdict is about the scan.",
        found.len(),
        PINNED_IDENTITY_DOMAINS.len(),
    );

    for literal in &found {
        let pinned = PINNED_IDENTITY_DOMAINS.contains(&literal.content.as_slice());
        let classified = ADMITTED_NON_DOMAIN_PREFIXES
            .iter()
            .any(|prefix| literal.content.starts_with(prefix));
        assert!(
            pinned || classified,
            "{}:{}: the literal `{}` is neither pinned in `PINNED_IDENTITY_DOMAINS` nor admitted \
             by a prefix in `ADMITTED_NON_DOMAIN_PREFIXES`. If a domain stepped, move its row in \
             the pin table; if this spelling separates no subjects, classify its namespace. \
             Leaving it unlisted is the third option this assertion exists to remove.",
            literal.path.display(),
            literal.line,
            render(&literal.content),
        );
    }
}

/// Every pinned identity domain is still declared somewhere in the crate.
///
/// The direction that catches a domain **reverted or deleted**. It compares
/// bytes, not lengths: the obligation domain's `v1` to `v2` step left two of its
/// three subjects the same length, so a length comparison would have reported
/// that both were unchanged.
#[test]
fn every_pinned_identity_domain_still_appears_in_the_source() {
    let found = scan_crate_sources();
    for pinned in PINNED_IDENTITY_DOMAINS {
        assert!(
            found
                .iter()
                .any(|literal| literal.content.as_slice() == *pinned),
            "`{}` is pinned as an identity domain of this crate, and no literal in `src/` or \
             `tests/` spells it. Either the domain stepped and its row here did not move with \
             it, or it was reverted, or the declaration left the crate. The scan read {} \
             `tiler.`-spelled literal(s) in total, so this is a statement about that spelling \
             rather than about the scan.",
            render(pinned),
            found.len(),
        );
    }
}

/// Both tables are sorted by content and free of duplicates.
///
/// Sorting keeps the pin reviewable as a diff — a stepped domain moves one row
/// to one place. The duplicate check matters more than it looks: a table that
/// named one spelling twice would still satisfy both assertions above while
/// covering one fewer domain than its length suggests.
#[test]
fn both_tables_are_sorted_and_free_of_duplicates() {
    for (name, table) in [
        ("PINNED_IDENTITY_DOMAINS", PINNED_IDENTITY_DOMAINS),
        ("ADMITTED_NON_DOMAIN_PREFIXES", ADMITTED_NON_DOMAIN_PREFIXES),
    ] {
        for pair in table.windows(2) {
            assert!(
                pair[0] < pair[1],
                "{name} is out of order or repeats itself at `{}` and `{}`.",
                render(pair[0]),
                render(pair[1]),
            );
        }
    }
}

/// No admitted prefix is a prefix of a pinned identity domain.
///
/// `ADMITTED_NON_DOMAIN_PREFIXES` is the escape hatch, and an escape hatch wide
/// enough to cover a pinned spelling would silently disarm that pin: the domain
/// could then be reverted, and the reverted spelling would be admitted by
/// namespace rather than compared against its row. Widening a prefix until it
/// swallows a domain fails here instead.
#[test]
fn no_admitted_prefix_swallows_a_pinned_domain() {
    for prefix in ADMITTED_NON_DOMAIN_PREFIXES {
        for pinned in PINNED_IDENTITY_DOMAINS {
            assert!(
                !pinned.starts_with(prefix),
                "the admitted prefix `{}` covers the pinned identity domain `{}`, so that \
                 domain's exact spelling is no longer compared against anything. Narrow the \
                 prefix rather than leaving the pin in place unenforced.",
                render(prefix),
                render(pinned),
            );
        }
    }
}
