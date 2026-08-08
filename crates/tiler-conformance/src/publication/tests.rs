//! What the publication can be held to without an Apple toolchain.
//!
//! Publishing a member reaches `metal` and `metallib`, so the members themselves
//! are produced inside the measured half and reported through
//! [`crate::measurement::Measured`] like any other device-bound work. What runs
//! here is what decides whether that half would be *measuring the right thing*:
//! the row count that makes an artifact-derived shape distinguishable from this
//! crate's own, and the temporary directory whose removal is the whole of this
//! module's disk hygiene.

use super::{PUBLISHED_ROWS, Published};
use crate::envelope::{PLAN_ROLES, REDUCTION_CLASSES};
use crate::serial_sum::{COLUMNS, ROWS};

/// The published rows and the direct path's own rows disagree, deliberately.
///
/// **This inequality is a check, not a coincidence.** `crate::envelope`'s
/// `compile_for_declared_shape` compiles the shape *the artifact declares* and
/// requires the packaged program's canonical identity to be one of the
/// alternatives that compilation retained. A regression that compiled this
/// crate's own [`ROWS`] instead would be completely invisible if the two numbers
/// agreed, and the defect it guards against is one this vertical actually
/// suffered: a consumer compiled four rows against a one-row publication for a
/// month, so every packaged program was foreign and the whole matrix proved
/// nothing.
///
/// The reduced extents are asserted equal for the opposite reason: the
/// `nontrivial` class exists to publish the same contributor count the direct
/// path reduces, so a divergence there would leave the matrix's one non-boundary
/// class describing a shape nothing else in this crate runs.
#[test]
fn the_published_rows_are_not_the_direct_paths_own() {
    assert_eq!(PUBLISHED_ROWS, 1);
    assert_eq!(ROWS, 4);
    assert_ne!(
        PUBLISHED_ROWS, ROWS,
        "a consumer substituting its own row count for the artifact's declared one is only \
         detectable while the two differ",
    );
    assert_eq!(
        REDUCTION_CLASSES,
        [
            ("empty-domain", 0),
            ("singleton", 1),
            ("nontrivial", COLUMNS)
        ],
        "the nontrivial class publishes the contributor count the direct path reduces",
    );
    assert_eq!(REDUCTION_CLASSES.len() * PLAN_ROLES.len(), 6);
}

/// The publication directory exists while it is held and is gone once it is not.
///
/// **Disk hygiene is a gate obligation here rather than tidiness.** Every routed
/// run on every Apple host writes eight envelopes and their records, one of which
/// is a four-megabyte operand stream, and the `#[ignore]`d prefill run writes
/// four more totalling forty-eight megabytes; a guard that failed to remove them
/// would accumulate that on each `make full`. The removal is asserted rather than
/// assumed because it happens in a `Drop` that no other test would notice
/// failing.
#[test]
fn a_publication_directory_is_removed_when_it_is_dropped() {
    let published = Published::open("hygiene").expect("a private directory is creatable");
    let directory = published.directory.clone();
    let base = published.base().to_path_buf();
    assert!(directory.is_dir());
    assert!(
        base.starts_with(&directory),
        "every published member must live under the directory the guard removes: {}",
        base.display(),
    );
    // A file the guard has to take with it, so the removal is proved recursive
    // rather than proved on an empty directory.
    std::fs::write(&base, b"an envelope's worth of bytes").expect("the base path is writable");

    drop(published);
    assert!(
        !directory.exists(),
        "{} survived its guard, so a routed run leaves its members behind",
        directory.display(),
    );
}
