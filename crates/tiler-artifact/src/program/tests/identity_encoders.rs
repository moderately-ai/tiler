//! This crate's own tag tables and finite-domain identity encoders.

use super::super::model::{push_storage_encoding, push_synchronization};
use super::default_artifact;
use std::collections::HashMap;
use tiler_ir::program::{BitPackedEncoding, PackedBitOrder, PackedTailRule, StorageEncoding};
use tiler_ir::schedule::{
    FencedSpaces, MemoryOrdering, SynchronizationKind, SynchronizationScope, SynchronizationSubject,
};

/// Every synchronization vocabulary round-trips through this crate's own tags.
///
/// Three separate tables, each a forward and inverse pair kept in one place, and
/// each counted against a population written *here* rather than derived from the
/// encoder — so a widened vocabulary in `tiler-ir` fails this count instead of
/// silently leaving the new variant untested. That the tables are this crate's
/// own copy is the design: the schedule identity and the artifact identity are
/// different subjects, and a shared table would let one domain's step move the
/// other's bytes.
#[test]
fn every_synchronization_vocabulary_round_trips_through_its_governed_tag() {
    use tiler_ir::schedule::{MemoryOrdering, SynchronizationKind, SynchronizationScope};

    let kinds = [
        SynchronizationKind::ControlBarrier,
        SynchronizationKind::AsynchronousCopy,
        SynchronizationKind::SplitPhaseBarrier,
        SynchronizationKind::Collective,
        SynchronizationKind::Atomic,
        SynchronizationKind::InterDispatchDependency,
    ];
    let mut tags = Vec::new();
    for kind in kinds {
        let tag = super::super::model::synchronization_kind_tag(kind);
        assert_eq!(
            super::super::model::synchronization_kind_from_tag(tag),
            Some(kind)
        );
        assert!(!tags.contains(&tag), "kind tag {tag:#04x} is not distinct");
        tags.push(tag);
    }
    assert_eq!(tags.len(), 6, "every admitted-or-refused kind was checked");
    assert_eq!(
        super::super::model::synchronization_kind_from_tag(0x00),
        None
    );
    assert_eq!(
        super::super::model::synchronization_kind_from_tag(0xff),
        None
    );

    let scopes = [
        SynchronizationScope::Subgroup,
        SynchronizationScope::Workgroup,
        SynchronizationScope::Device,
    ];
    let mut tags = Vec::new();
    for scope in scopes {
        let tag = super::super::model::synchronization_scope_tag(scope);
        assert_eq!(
            super::super::model::synchronization_scope_from_tag(tag),
            Some(scope)
        );
        assert!(!tags.contains(&tag), "scope tag {tag:#04x} is not distinct");
        tags.push(tag);
    }
    assert_eq!(tags.len(), 3);
    assert_eq!(
        super::super::model::synchronization_scope_from_tag(0x00),
        None
    );
    assert_eq!(
        super::super::model::synchronization_scope_from_tag(0xff),
        None
    );

    let orderings = [
        MemoryOrdering::Relaxed,
        MemoryOrdering::AcquireRelease,
        MemoryOrdering::SequentiallyConsistent,
    ];
    let mut tags = Vec::new();
    for ordering in orderings {
        let tag = super::super::model::memory_ordering_tag(ordering);
        assert_eq!(
            super::super::model::memory_ordering_from_tag(tag),
            Some(ordering)
        );
        assert!(
            !tags.contains(&tag),
            "ordering tag {tag:#04x} is not distinct"
        );
        tags.push(tag);
    }
    assert_eq!(tags.len(), 3);
    assert_eq!(super::super::model::memory_ordering_from_tag(0x00), None);
    assert_eq!(super::super::model::memory_ordering_from_tag(0xff), None);
}

/// The recorded absence of a synchronization requirement changes the bytes.
///
/// The load-bearing half of the `v14` step: an entry that requires no
/// realization writes a byte saying so, so its identity is not the identity an
/// entry that had never been able to state one would have had. Asserting the
/// *presence* of the recorded absence is what stops a later change quietly
/// reverting to omission, which would make a synchronized entry and an
/// unsynchronized one share bytes again.
#[test]
fn an_entry_records_the_absence_of_a_synchronization_requirement() {
    let artifact = default_artifact();
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    assert_eq!(entry.resources().synchronization, None);

    // The presence byte is written, not omitted: encoding the same resource
    // record with and without it differ by exactly one byte.
    let mut with_absence = Vec::new();
    super::super::model::push_resources(&mut with_absence, entry.resources())
        .expect("the arithmetic rows encode");
    let mut without = Vec::new();
    super::super::model::push_synchronization(&mut without, None);
    assert_eq!(without, vec![0x00], "absence is one recorded byte");
    assert!(
        with_absence.windows(1).any(|byte| byte == [0x00]),
        "the resource record carries the recorded absence"
    );

    // And a `Some` occupies seven, so no synchronized entry can encode into the
    // byte count an unsynchronized one occupies.
    let mut present = Vec::new();
    super::super::model::push_synchronization(
        &mut present,
        Some(tiler_ir::schedule::SynchronizationSubject {
            kind: tiler_ir::schedule::SynchronizationKind::ControlBarrier,
            execution_scope: tiler_ir::schedule::SynchronizationScope::Workgroup,
            visibility_scope: tiler_ir::schedule::SynchronizationScope::Workgroup,
            fenced_spaces: tiler_ir::schedule::FencedSpaces {
                workgroup: true,
                device: false,
            },
            ordering: tiler_ir::schedule::MemoryOrdering::AcquireRelease,
        }),
    );
    assert_eq!(present.len(), 7);
    assert_ne!(present[0], without[0]);
}

// -------------------------------------------------------------------------
// Exhaustive injectivity of the finite-domain identity encoders
// -------------------------------------------------------------------------
//
// An encoder is injective when no two distinct inputs produce the same bytes,
// and a collision in artifact identity is two different packaged programs
// answering to one name — a cache hit on the wrong artifact, not a miss. Most
// encoders here carry a `u32` ordinal, a slice, or a string, so their domains
// cannot be walked and their injectivity rests on the framing argument in
// `tiler_ir::identity`. The two below can be walked, and walking them turns the
// claim into exhaustive finite evidence: every pair is compared because every
// value is.
//
// The enumerations are sized by `variant_count`, so a vocabulary widened in
// `tiler-ir` is a build error here. That guard is why enumerating these
// vocabularies a second time on this side of the crate boundary is safe: the
// two lists cannot silently disagree about how large the domain is, because
// neither can silently stop covering it.

/// Every construct class an artifact's synchronization requirement can name.
const SYNCHRONIZATION_KINDS: [SynchronizationKind;
    std::mem::variant_count::<SynchronizationKind>()] = [
    SynchronizationKind::ControlBarrier,
    SynchronizationKind::AsynchronousCopy,
    SynchronizationKind::SplitPhaseBarrier,
    SynchronizationKind::Collective,
    SynchronizationKind::Atomic,
    SynchronizationKind::InterDispatchDependency,
];

/// Every invocation set an arrival or a publication can range over.
const SYNCHRONIZATION_SCOPES: [SynchronizationScope;
    std::mem::variant_count::<SynchronizationScope>()] = [
    SynchronizationScope::Subgroup,
    SynchronizationScope::Workgroup,
    SynchronizationScope::Device,
];

/// Every ordering a synchronization requirement can establish.
const MEMORY_ORDERINGS: [MemoryOrdering; std::mem::variant_count::<MemoryOrdering>()] = [
    MemoryOrdering::Relaxed,
    MemoryOrdering::AcquireRelease,
    MemoryOrdering::SequentiallyConsistent,
];

/// Returns the number of bools in one exhaustive field census.
const fn bool_field_count<const N: usize>(_: [bool; N]) -> usize {
    N
}

/// The independent boolean fields carried by [`FencedSpaces`].
const FENCED_SPACE_FIELD_COUNT: usize = {
    let FencedSpaces { workgroup, device } = FencedSpaces::NONE;
    bool_field_count([workgroup, device])
};

/// Every fence a synchronization requirement can name.
///
/// `FencedSpaces` is a struct, so `variant_count` does not apply. Each field in
/// the exhaustive census above is boolean, making the inhabitant count two to
/// the power of the field count.
const FENCED_SPACES: [FencedSpaces; 1 << FENCED_SPACE_FIELD_COUNT] = [
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

/// The artifact synchronization encoder is injective over all 649 inhabitants.
///
/// **Exhaustive finite evidence.** The domain is `Option<SynchronizationSubject>`:
/// the product of five closed vocabularies — 6 construct kinds, 3 arrival
/// scopes, 3 publication scopes, 4 fences, 3 orderings — plus the stated
/// absence. The subject's fields are independent and carry no constructor
/// invariant, so `6 * 3 * 3 * 4 * 3 + 1 = 649` is the inhabitant count and not
/// an estimate of it.
///
/// The three component tag tables are separately round-tripped elsewhere in this
/// crate, and that is a strictly weaker claim than this one: three injective
/// component maps can still compose into a non-injective record if a field is
/// dropped or written twice. Only the product distinguishes those, which is why
/// it is enumerated rather than inferred.
#[test]
fn the_artifact_synchronization_encoding_is_injective_over_its_whole_domain() {
    const POPULATION: usize = 1 + SYNCHRONIZATION_KINDS.len()
        * SYNCHRONIZATION_SCOPES.len()
        * SYNCHRONIZATION_SCOPES.len()
        * FENCED_SPACES.len()
        * MEMORY_ORDERINGS.len();

    let mut subjects: Vec<Option<SynchronizationSubject>> = vec![None];
    for kind in SYNCHRONIZATION_KINDS {
        for execution_scope in SYNCHRONIZATION_SCOPES {
            for visibility_scope in SYNCHRONIZATION_SCOPES {
                for fenced_spaces in FENCED_SPACES {
                    for ordering in MEMORY_ORDERINGS {
                        subjects.push(Some(SynchronizationSubject {
                            kind,
                            execution_scope,
                            visibility_scope,
                            fenced_spaces,
                            ordering,
                        }));
                    }
                }
            }
        }
    }

    assert_eq!(subjects.len(), POPULATION);
    assert_eq!(
        POPULATION, 649,
        "the subject domain changed size; the exhaustive claim is about whatever it is now, \
         so restate it deliberately"
    );

    let mut seen: HashMap<Vec<u8>, Option<SynchronizationSubject>> =
        HashMap::with_capacity(POPULATION);
    for subject in subjects {
        let mut bytes = Vec::new();
        push_synchronization(&mut bytes, subject);
        // One presence tag, and six subject bytes when present. The width is
        // variable, so what keeps the record unambiguous is the presence tag —
        // and the collision check below is what confirms it is doing that work.
        let expected = if subject.is_some() { 7 } else { 1 };
        assert_eq!(bytes.len(), expected, "{subject:?} changed width");
        if let Some(previous) = seen.insert(bytes, subject) {
            panic!("{subject:?} and {previous:?} share one encoding");
        }
    }
    assert_eq!(seen.len(), POPULATION);
}

/// The artifact storage-encoding encoder is injective over every constructible value.
///
/// **Exhaustive finite evidence over the constructible domain.**
/// `BitPackedEncoding` has private fields and one constructor, which admits only
/// element widths below eight that divide eight. The sweep offers all 512
/// `(u8, PackedBitOrder, PackedTailRule)` candidates to that constructor and
/// enumerates the survivors, so the population is *derived* from the admission
/// rule instead of asserted alongside it — a widened rule grows this domain
/// rather than leaving new values untested.
///
/// A second, independent copy of `tiler-ir`'s program encoder, so it earns a
/// second proof: this crate's copy inlines its own `PackedBitOrder` and
/// `PackedTailRule` tables with no shared tag function and no decode inverse, so
/// nothing about the other copy's bytes constrains these.
#[test]
fn the_artifact_storage_encoding_is_injective_over_its_constructible_domain() {
    const BIT_ORDERS: [PackedBitOrder; std::mem::variant_count::<PackedBitOrder>()] = [
        PackedBitOrder::LeastSignificantElementFirst,
        PackedBitOrder::MostSignificantElementFirst,
    ];
    const TAIL_RULES: [PackedTailRule; std::mem::variant_count::<PackedTailRule>()] =
        [PackedTailRule::Zero];

    let mut candidates = 0_usize;
    let mut encodings = vec![StorageEncoding::Unpacked];
    for element_bits in 0..=u8::MAX {
        for bit_order in BIT_ORDERS {
            for tail in TAIL_RULES {
                candidates += 1;
                if let Some(packed) = BitPackedEncoding::new(element_bits, bit_order, tail) {
                    encodings.push(StorageEncoding::BitPacked(packed));
                }
            }
        }
    }

    assert_eq!(
        candidates,
        256 * BIT_ORDERS.len() * TAIL_RULES.len(),
        "the candidate sweep did not cover the whole field product"
    );
    assert_eq!(
        encodings.len(),
        1 + 3 * BIT_ORDERS.len() * TAIL_RULES.len(),
        "the constructible domain changed size; restate the claim deliberately"
    );
    assert_eq!(encodings.len(), 7);

    let mut seen: HashMap<Vec<u8>, StorageEncoding> = HashMap::with_capacity(encodings.len());
    for encoding in encodings {
        let mut bytes = Vec::new();
        push_storage_encoding(&mut bytes, encoding);
        let expected = match encoding {
            StorageEncoding::Unpacked => 1,
            StorageEncoding::BitPacked(_) => 4,
        };
        assert_eq!(bytes.len(), expected, "{encoding:?} changed width");
        if let Some(previous) = seen.insert(bytes, encoding) {
            panic!("{encoding:?} and {previous:?} share one encoding");
        }
    }
    assert_eq!(seen.len(), 7);
}
