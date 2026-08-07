//! Counted enumerations of the finite vocabularies identity encoders write.
//!
//! # What a test built on this module does and does not establish
//!
//! An encoder is **injective** when no two distinct inputs produce the same
//! bytes. Identity in this workspace is a digest over a canonical encoding, so a
//! collision is not a cache miss: it is two different subjects answering to one
//! name, and every consumer downstream — a cached artifact, a target
//! feasibility fact, a published kernel — cannot tell that from a genuine match.
//!
//! Most encoders here carry a `u32` ordinal, a slice, or a string, so their
//! domains are astronomically large and their injectivity rests on the framing
//! argument in [`crate::identity`]. A few do not. For those the domain can be
//! *enumerated*, and enumerating it turns the claim from an argument about the
//! encoder's shape into **exhaustive finite evidence**: every pair is compared,
//! because every value is. That is a proof over the whole domain, and it is the
//! strongest class this repository recognizes short of `SoundProof`.
//!
//! It is a proof **only while the enumeration is the domain**. A list that has
//! silently stopped covering its type reports no collision for the same reason
//! an empty list does, and "nothing ran" then looks exactly like "nothing
//! collided". Two things hold the enumerations below honest:
//!
//! - every array over a plain enum is sized by
//!   [`variant_count`](std::mem::variant_count), so a widened vocabulary is a
//!   build error here rather than a population that quietly shrinks; and
//! - every test asserts the population it walked, so a domain that changed size
//!   fails and has to be restated deliberately.
//!
//! # Why the vocabularies are enumerated here rather than at each encoder
//!
//! `SubnormalMode`, `NumericalPermission`, `ExceptionalValueAssumption`, and
//! `SynchronizationSubject` are each written by *three* independent encoders —
//! the scheduled region's, the structured kernel's, and (across the crate
//! boundary) the artifact's. The encoders are deliberately separate copies so
//! one identity domain's step cannot move another's bytes. The *domains* they
//! range over are not copies: they are one vocabulary, so this module enumerates
//! each once and both in-crate encoders' tests walk the same list.
//!
//! `tiler-artifact` cannot reach this module — it is `cfg(test)` and crate
//! private — so it enumerates the subject a second time. That is safe rather
//! than a second chance to under-count, and the `variant_count` sizing is why:
//! neither list can silently stop covering the vocabulary, so the two cannot
//! silently disagree about how large the domain is. Making this module reachable
//! would mean a public test-support surface, which is Tom's call and buys
//! nothing the guard does not already give.

use std::collections::HashMap;
use std::fmt::Debug;
use std::mem::variant_count;

use crate::schedule::{
    ApproximationEnvelope, ExceptionalValueAssumption, FencedSpaces, FlushedZeroSign,
    MaterializationRounding, MemoryOrdering, NumericalPermission, SubnormalMode,
    SynchronizationKind, SynchronizationScope, SynchronizationSubject, ValueDomainProvenance,
};

/// Every subnormal treatment an arithmetic dimension can declare.
pub(crate) const SUBNORMAL_MODES: [SubnormalMode; 3] = [
    SubnormalMode::Preserve,
    SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    },
    SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::AlwaysPositive,
    },
];

/// Every permission a transform dimension can carry.
pub(crate) const PERMISSIONS: [NumericalPermission; variant_count::<NumericalPermission>()] = [
    NumericalPermission::Forbidden,
    NumericalPermission::Permitted,
];

/// Every exceptional-value assumption, over every evidence class.
pub(crate) const EXCEPTIONAL_ASSUMPTIONS: [ExceptionalValueAssumption; 4] = [
    ExceptionalValueAssumption::MakeNoAssumption,
    ExceptionalValueAssumption::AssumeAbsent {
        provenance: ValueDomainProvenance::CompilerProven,
    },
    ExceptionalValueAssumption::AssumeAbsent {
        provenance: ValueDomainProvenance::RuntimeValidated,
    },
    ExceptionalValueAssumption::AssumeAbsent {
        provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
    },
];

/// Every accuracy envelope an approximate-intrinsic permission resolves to.
pub(crate) const APPROXIMATION_ENVELOPES: [ApproximationEnvelope;
    variant_count::<ApproximationEnvelope>()] = [
    ApproximationEnvelope::Forbidden,
    ApproximationEnvelope::BackendElementary,
];

/// Every rounding an observable materialization boundary can apply.
pub(crate) const MATERIALIZATION_ROUNDINGS: [MaterializationRounding;
    variant_count::<MaterializationRounding>()] = [MaterializationRounding::NearestTiesToEven];

/// Every construct class a synchronization subject can name.
const KINDS: [SynchronizationKind; variant_count::<SynchronizationKind>()] = [
    SynchronizationKind::ControlBarrier,
    SynchronizationKind::AsynchronousCopy,
    SynchronizationKind::SplitPhaseBarrier,
    SynchronizationKind::Collective,
    SynchronizationKind::Atomic,
    SynchronizationKind::InterDispatchDependency,
];

/// Every invocation set an arrival or a publication can range over.
const SCOPES: [SynchronizationScope; variant_count::<SynchronizationScope>()] = [
    SynchronizationScope::Subgroup,
    SynchronizationScope::Workgroup,
    SynchronizationScope::Device,
];

/// Every ordering a point can establish over the effects it fences.
const ORDERINGS: [MemoryOrdering; variant_count::<MemoryOrdering>()] = [
    MemoryOrdering::Relaxed,
    MemoryOrdering::AcquireRelease,
    MemoryOrdering::SequentiallyConsistent,
];

/// Every fence a point can name.
///
/// `FencedSpaces` is a struct, so `variant_count` does not apply; this is
/// instead the product of `bool`'s two inhabitants over its two fields, which is
/// exhaustive by the type's own definition. A third flag would leave this list
/// at four entries, and [`SUBJECT_POPULATION`]'s assertion is what would fail.
const FENCES: [FencedSpaces; 4] = [
    FencedSpaces {
        workgroup: false,
        device: false,
    },
    FencedSpaces {
        workgroup: false,
        device: true,
    },
    FencedSpaces {
        workgroup: true,
        device: false,
    },
    FencedSpaces {
        workgroup: true,
        device: true,
    },
];

/// The number of distinct [`SynchronizationSubject`] values that exist.
///
/// The struct has five independent public fields and no constructor invariant,
/// so its inhabitant count is exactly the product of theirs.
pub(crate) const SUBJECT_POPULATION: usize =
    KINDS.len() * SCOPES.len() * SCOPES.len() * FENCES.len() * ORDERINGS.len();

/// Every [`SynchronizationSubject`] value, in a deterministic order.
///
/// The caller is expected to assert [`SUBJECT_POPULATION`] against the length,
/// so a vocabulary that changed size cannot pass as the one this was counted on.
pub(crate) fn every_synchronization_subject() -> Vec<SynchronizationSubject> {
    let mut subjects = Vec::with_capacity(SUBJECT_POPULATION);
    for kind in KINDS {
        for execution_scope in SCOPES {
            for visibility_scope in SCOPES {
                for fenced_spaces in FENCES {
                    for ordering in ORDERINGS {
                        subjects.push(SynchronizationSubject {
                            kind,
                            execution_scope,
                            visibility_scope,
                            fenced_spaces,
                            ordering,
                        });
                    }
                }
            }
        }
    }
    subjects
}

/// Asserts `values` encode to `values.len()` pairwise distinct byte runs.
///
/// Panics on the first collision naming both colliding values, because a count
/// mismatch alone would not say which pair of the domain the encoder cannot tell
/// apart — and the pair is what identifies the dropped field.
///
/// The final length assertion covers the *other* way an exhaustive claim fails:
/// an enumeration that repeated a value would compare fewer pairs than it looks
/// like it did, and every repeat would present as a collision the encoder is not
/// responsible for.
pub(crate) fn assert_injective<T: Copy + Debug>(
    values: &[T],
    mut push: impl FnMut(&mut Vec<u8>, T),
) {
    let mut seen: HashMap<Vec<u8>, T> = HashMap::with_capacity(values.len());
    for &value in values {
        let mut bytes = Vec::new();
        push(&mut bytes, value);
        assert!(!bytes.is_empty(), "{value:?} encoded to no bytes at all");
        if let Some(previous) = seen.insert(bytes, value) {
            panic!("{value:?} and {previous:?} share one encoding");
        }
    }
    assert_eq!(
        seen.len(),
        values.len(),
        "the enumeration repeated a value, so it compared fewer pairs than its length claims"
    );
}

/// Asserts injectivity as [`assert_injective`] does, and a constant width too.
///
/// Fixed width is a separate property from injectivity, and a composite encoding
/// needs both. A component written into the middle of a record with no length
/// prefix of its own can shift every field after it, so a variable-width
/// component can let two distinguishable records concatenate to one byte string
/// even when the component encoder is perfectly injective.
pub(crate) fn assert_injective_fixed_width<T: Copy + Debug>(
    values: &[T],
    width: usize,
    mut push: impl FnMut(&mut Vec<u8>, T),
) {
    for &value in values {
        let mut bytes = Vec::new();
        push(&mut bytes, value);
        assert_eq!(
            bytes.len(),
            width,
            "{value:?} encoded to {} bytes, not the fixed {width}",
            bytes.len()
        );
    }
    assert_injective(values, push);
}
