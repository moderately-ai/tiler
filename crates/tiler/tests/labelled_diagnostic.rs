//! The label this crate publishes is the one the runtime proof prints.
//!
//! `PRODUCER_DECLARED_EQUALITY` and `prototypes/serial-sum-run` make the same
//! claim about the same thing: that a route settled against an artifact's
//! declared profile is *producer-declared equality* and not *host-earned
//! eligibility*, because ADR 0086 refuses every macOS row. Two copies of one
//! sentence drift, and a paraphrase that lost the negation would read as the
//! opposite claim while still looking like a diagnostic.
//!
//! So the two are compared rather than merely both written. The prototype is the
//! authority — it printed the words first, and its ticket's recorded outcome
//! cites them — and this crate's constant must be a substring of its source.
//!
//! # Why the file is read rather than the crate imported
//!
//! `crates/tiler/tests/dependency_direction.rs` forbids any workspace package
//! from depending on this one, and the prototype is a workspace package. The
//! dependency cannot run in either direction, so the text is the only thing the
//! two can share, and reading it is the only way to compare them. That file also
//! reads a sibling path (`../../Cargo.lock`) for the same reason.

/// The prototype whose printed words this crate's constant must match.
const PROOF: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../prototypes/serial-sum-run/src/proof.rs"
);

/// Collapses a Rust source file's string-continuation and doc-comment noise.
///
/// The prototype writes the sentence across a `\`-continued string literal, so
/// the bytes on disk are not the bytes it prints. Removing the continuation —
/// a backslash, a newline, and the indentation that follows, which is exactly
/// what rustc removes — reconstructs the printed form.
fn as_printed(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '\\' || chars.peek() != Some(&'\n') {
            out.push(character);
            continue;
        }
        chars.next();
        while chars
            .peek()
            .is_some_and(|next| *next == ' ' || *next == '\t')
        {
            chars.next();
        }
    }
    out
}

/// The published label is the prototype's own sentence, unchanged.
#[test]
fn the_published_label_is_the_runtime_proofs_own_words() {
    let source = std::fs::read_to_string(PROOF).unwrap_or_else(|error| {
        panic!("the runtime proof must be readable from the facade crate at {PROOF}: {error}")
    });

    // Without this, a prototype that stopped printing the distinction entirely
    // would make the comparison below vacuous rather than failing: an empty or
    // moved file has no disagreeing sentence in it either.
    assert!(
        source.contains("host-earned eligibility"),
        "the runtime proof no longer mentions host-earned eligibility, so this test is comparing \
         against a file that has stopped making the claim; re-derive the label before trusting it"
    );

    let printed = as_printed(&source);
    assert!(
        printed.contains(tiler::__private::PRODUCER_DECLARED_EQUALITY),
        "the facade publishes {:?}, which the runtime proof no longer prints; the two state one \
         fact and must state it in one wording",
        tiler::__private::PRODUCER_DECLARED_EQUALITY,
    );
}

/// The continuation reconstruction is itself checked, on a case that must fail
/// without it.
///
/// The comparison above would pass for the wrong reason if `as_printed` were the
/// identity function on a source that happened to contain the sentence on one
/// line. This pins the transformation to a case where the raw text does *not*
/// contain the sentence and the reconstructed text does.
#[test]
fn a_continued_literal_is_reconstructed_before_it_is_compared() {
    let raw = "\"producer-declared equality against x, NOT \\\n         host-earned eligibility\"";
    assert!(
        !raw.contains("NOT host-earned eligibility"),
        "this fixture is only meaningful while the raw form differs from the printed one",
    );
    assert!(
        as_printed(raw).contains("NOT host-earned eligibility"),
        "the continuation was not reconstructed: {}",
        as_printed(raw),
    );
}
