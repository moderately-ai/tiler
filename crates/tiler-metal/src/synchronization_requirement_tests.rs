//! Tests for the delivery-time synchronization-realization comparison.
//!
//! Every case is device-free, and unlike its index-arithmetic sibling that is
//! not merely convenient: a synchronization realization is a property of this
//! backend's kernel vocabulary rather than of a bound device, so there is no
//! observation a test would have to stand in for. See
//! [`crate::synchronization_requirement`] for why the two evidence classes are
//! kept apart.
//!
//! The negative cases name subjects no *locally derived* schedule produces —
//! today `required_subject` returns exactly one value — and that is the point.
//! The subject travels on a delivered artifact's own resource record, which the
//! artifact layer round-trips rather than re-derives, so an envelope reaching
//! this host can state any subject the neutral vocabulary can express. Each case
//! below is such an envelope's requirement.

use std::mem::variant_count;

use tiler_ir::kernel::{AddressSpace, BarrierOrdering, ExecutionScope, MemoryScope};
use tiler_ir::schedule::{
    FencedSpaces, MemoryOrdering, PhaseId, StagingId, SyncPointId, SynchronizationKind,
    SynchronizationScope, SynchronizationSubject, VisibilityEdge, required_subject,
};

use crate::diagnostic::BarrierRejection;
use crate::emit::barrier_call;
use crate::synchronization_requirement::{
    MetalSynchronizationRefusal, evaluate_synchronization, spell,
};

/// Every operation kind a subject can name.
///
/// Sized by [`variant_count`], so a kind added to the vocabulary and not to this
/// list is a build error here rather than a population that silently shrinks
/// while the census below still reports agreement.
const KINDS: [SynchronizationKind; variant_count::<SynchronizationKind>()] = [
    SynchronizationKind::ControlBarrier,
    SynchronizationKind::AsynchronousCopy,
    SynchronizationKind::SplitPhaseBarrier,
    SynchronizationKind::Collective,
    SynchronizationKind::Atomic,
    SynchronizationKind::InterDispatchDependency,
];

/// Every governed invocation set, used for arrival and for publication.
const SCOPES: [SynchronizationScope; variant_count::<SynchronizationScope>()] = [
    SynchronizationScope::Subgroup,
    SynchronizationScope::Workgroup,
    SynchronizationScope::Device,
];

/// Every ordering a subject can establish.
const ORDERINGS: [MemoryOrdering; variant_count::<MemoryOrdering>()] = [
    MemoryOrdering::Relaxed,
    MemoryOrdering::AcquireRelease,
    MemoryOrdering::SequentiallyConsistent,
];

/// Every fence a subject can name.
///
/// `FencedSpaces` is a struct, so [`variant_count`] does not apply; this is the
/// product of `bool`'s two inhabitants over its two fields, exhaustive by the
/// type's own definition. A third flag would leave this list at four entries and
/// [`POPULATION`]'s assertion is what would then fail.
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
/// Five independent public fields and no constructor invariant, so the
/// inhabitant count is exactly the product of theirs.
const POPULATION: usize =
    KINDS.len() * SCOPES.len() * SCOPES.len() * FENCES.len() * ORDERINGS.len();

/// Every [`SynchronizationSubject`] value, in a deterministic order.
fn every_subject() -> Vec<SynchronizationSubject> {
    let mut subjects = Vec::with_capacity(POPULATION);
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

/// The subject a real staged handoff derives, taken from the derivation itself.
///
/// Read through [`required_subject`] rather than written as a literal, so this
/// tracks what a cooperative tile actually requires. A literal here would keep
/// passing on the day the derivation changed, which is exactly when the positive
/// case stops being evidence that this backend can run what this workspace
/// builds.
fn derived_subject() -> SynchronizationSubject {
    let edge = VisibilityEdge {
        staging: StagingId::FIRST,
        produced_in: PhaseId::FIRST,
        consumed_in: PhaseId::new(1),
    };
    required_subject(&[edge]).expect("a non-empty edge set derives a subject")
}

/// The realization every cooperative tile in this workspace derives is deliverable.
///
/// The positive neighbour every negative case below is a perturbation of. Without
/// it the refusals would be evidence only that this comparison refuses things,
/// not that it admits the one subject that has to route.
#[test]
fn the_derived_staged_handoff_is_realized() {
    let subject = derived_subject();
    assert_eq!(
        evaluate_synchronization(Some(subject)),
        Ok(()),
        "the workgroup control barrier a staged handoff derives must route",
    );
    // The spelled barrier fences exactly the domain the subject named, checked
    // through the emitted text rather than through the spec fields: an inversion
    // that dropped the workgroup flag would still produce a spec that admits.
    let emitted = barrier_call(&spell(subject).expect("an admitted subject spells"))
        .expect("an admitted subject's spelling emits");
    assert_eq!(emitted, "threadgroup_barrier(mem_flags::mem_threadgroup);");
}

/// A region that stages nothing requires nothing, and that is not a zero.
#[test]
fn the_canonical_absence_requires_nothing() {
    assert_eq!(evaluate_synchronization(None), Ok(()));
}

/// No Metal barrier publishes device-wide, and that is refused before the commit.
///
/// The delivery-time counterpart of `no_metal_barrier_establishes_device_wide_visibility`,
/// which is an *emission* guarantee about bytes this workspace produced. This is
/// the case a delivered artifact actually has: an envelope requiring device-wide
/// publication reaching a host whose backend has no such barrier.
#[test]
fn a_device_wide_publication_is_refused_and_names_the_whole_subject() {
    let required = SynchronizationSubject {
        visibility_scope: SynchronizationScope::Device,
        ..derived_subject()
    };
    let refusal = evaluate_synchronization(Some(required))
        .expect_err("Metal establishes no device-wide visibility from an in-kernel barrier");
    assert_eq!(
        refusal,
        MetalSynchronizationRefusal::Unrealizable {
            required,
            reason: BarrierRejection::MemoryVisibility {
                execution: ExecutionScope::Workgroup,
                memory: MemoryScope::Device,
            },
        },
    );
    // The refusal names the realization that was required, whole. A reader must
    // not be able to conclude the other four dimensions were satisfied.
    assert_eq!(refusal.required(), required);
    assert_eq!(refusal.rule(), "metal.synchronization.unrealizable");
}

/// Every unadmitted kind is refused before any spelling is attempted.
///
/// The whole population rather than one example, and counted: a match that
/// admitted one unadmitted kind would pass a single-case test. The distinction
/// from a declined spelling is load-bearing — a collective carries a combine
/// order and a numerical realization no barrier field states, so lowering it as
/// a barrier would be a different computation, not a slower one.
#[test]
fn every_unadmitted_kind_is_refused_by_name() {
    let mut refused = 0;
    for kind in KINDS {
        if kind == SynchronizationKind::ControlBarrier {
            continue;
        }
        let required = SynchronizationSubject {
            kind,
            ..derived_subject()
        };
        assert_eq!(
            evaluate_synchronization(Some(required)),
            Err(MetalSynchronizationRefusal::UnadmittedKind { required, kind }),
            "{} has no Metal construct",
            kind.key(),
        );
        refused += 1;
    }
    assert_eq!(
        refused,
        KINDS.len() - 1,
        "every kind but the control barrier must have been refused",
    );
}

/// A device-wide arrival has no kernel spelling at all.
///
/// Told apart from a declined barrier because the repairs differ: this is a
/// construct the kernel vocabulary cannot state, so it never reaches emission's
/// own execution-scope arm.
#[test]
fn a_device_wide_arrival_has_no_kernel_spelling() {
    let required = SynchronizationSubject {
        execution_scope: SynchronizationScope::Device,
        ..derived_subject()
    };
    assert_eq!(
        evaluate_synchronization(Some(required)),
        Err(MetalSynchronizationRefusal::UnspellableExecutionScope {
            required,
            scope: SynchronizationScope::Device,
        }),
    );
}

/// Subgroup-wide publication has no kernel spelling.
///
/// A gap in the portable vocabulary rather than a Metal limitation — the
/// governed memory scopes name workgroup and device visibility and nothing
/// narrower — and it is reported as one.
#[test]
fn a_subgroup_publication_has_no_kernel_spelling() {
    let required = SynchronizationSubject {
        visibility_scope: SynchronizationScope::Subgroup,
        ..derived_subject()
    };
    assert_eq!(
        evaluate_synchronization(Some(required)),
        Err(MetalSynchronizationRefusal::UnspellableVisibilityScope {
            required,
            scope: SynchronizationScope::Subgroup,
        }),
    );
}

/// Neither a weaker nor a stronger ordering has a kernel spelling.
///
/// Both directions, because rounding either onto acquire-release is a distinct
/// defect: weakening drops the happens-before edge the handoff needs, and
/// strengthening claims a total order the barrier does not establish.
#[test]
fn no_ordering_but_acquire_release_has_a_kernel_spelling() {
    let mut refused = 0;
    for ordering in ORDERINGS {
        if ordering == MemoryOrdering::AcquireRelease {
            continue;
        }
        let required = SynchronizationSubject {
            ordering,
            ..derived_subject()
        };
        assert_eq!(
            evaluate_synchronization(Some(required)),
            Err(MetalSynchronizationRefusal::UnspellableOrdering { required, ordering }),
            "{} has no BarrierOrdering spelling",
            ordering.key(),
        );
        refused += 1;
    }
    assert_eq!(refused, ORDERINGS.len() - 1);
}

/// The whole subject domain partitions into a counted census.
///
/// Named and counted so "nothing ran" cannot look green, and each class is
/// derived independently from the vocabulary sizes rather than from the result,
/// so a perturbation to any one arm moves exactly one count. The population is
/// sized by [`variant_count`], so a widened vocabulary fails the build at the
/// enumeration rather than silently reporting agreement over a smaller domain.
#[test]
fn the_subject_domain_partitions_into_a_counted_census() {
    let subjects = every_subject();
    assert_eq!(subjects.len(), POPULATION);
    assert_eq!(
        POPULATION, 648,
        "6 kinds x 3 x 3 scopes x 4 fences x 3 orderings"
    );

    let (mut realized, mut kind, mut arrival, mut publication, mut ordering, mut declined) =
        (0, 0, 0, 0, 0, 0);
    for subject in &subjects {
        // Exhaustive with no wildcard. `#[non_exhaustive]` has no effect inside
        // the defining crate, so a refusal class added to the vocabulary is a
        // build error *here* — the one place that has to say how many of the
        // domain it accounts for — rather than a catch-all that would let the
        // totals below keep summing to the population while a class went
        // uncounted.
        match evaluate_synchronization(Some(*subject)) {
            Ok(()) => realized += 1,
            Err(MetalSynchronizationRefusal::UnadmittedKind { .. }) => kind += 1,
            Err(MetalSynchronizationRefusal::UnspellableExecutionScope { .. }) => arrival += 1,
            Err(MetalSynchronizationRefusal::UnspellableVisibilityScope { .. }) => publication += 1,
            Err(MetalSynchronizationRefusal::UnspellableOrdering { .. }) => ordering += 1,
            Err(MetalSynchronizationRefusal::Unrealizable { .. }) => declined += 1,
        }
    }
    println!(
        "census over {POPULATION} subjects: realized {realized}, unadmitted-kind {kind}, \
         unspellable-arrival {arrival}, unspellable-publication {publication}, \
         unspellable-ordering {ordering}, declined {declined}",
    );

    // Exactly the workgroup-arriving, workgroup-publishing, acquire-release
    // control barriers, over every fence — four of them, and the fence is the
    // only dimension this backend leaves free.
    assert_eq!(realized, FENCES.len(), "only the fence dimension is free");
    // Five unadmitted kinds over every other dimension.
    assert_eq!(
        kind,
        (KINDS.len() - 1) * SCOPES.len() * SCOPES.len() * FENCES.len() * ORDERINGS.len(),
    );
    // A control barrier arriving device-wide, over every other dimension.
    assert_eq!(arrival, SCOPES.len() * FENCES.len() * ORDERINGS.len());
    // A control barrier publishing subgroup-wide, for the two spellable arrivals.
    assert_eq!(publication, 2 * FENCES.len() * ORDERINGS.len());
    // The two unspellable orderings, for the two spellable arrivals and the two
    // spellable publications.
    assert_eq!(ordering, 2 * 2 * FENCES.len() * (ORDERINGS.len() - 1));
    // What is left: a spellable barrier Metal declines to couple that way.
    assert_eq!(declined, 2 * 2 * FENCES.len() - realized);
    assert_eq!(
        realized + kind + arrival + publication + ordering + declined,
        POPULATION,
        "the census must exhaust the domain",
    );
}

/// Every admitted subject spells a barrier this backend actually emits.
///
/// The property that keeps the delivery-time answer honest about emission: an
/// admission here is a claim that the kernel *could* have been written, so the
/// spelled specification must reach emitted text rather than merely pass the
/// decision. Counted, so an admission set that emptied could not look green.
#[test]
fn every_admitted_subject_reaches_emitted_text() {
    let mut emitted = 0;
    for subject in every_subject() {
        if evaluate_synchronization(Some(subject)).is_err() {
            continue;
        }
        let spec = spell(subject).expect("an admitted subject spells");
        assert_eq!(spec.ordering, BarrierOrdering::AcquireRelease);
        assert_eq!(spec.execution_scope, ExecutionScope::Workgroup);
        assert_eq!(spec.memory_scope, MemoryScope::Workgroup);
        assert_eq!(spec.point, SyncPointId::FIRST);
        // The fence carries exactly the domains the subject named, in ascending
        // governed order, so a barrier that ordered a domain the schedule did
        // not fence — or dropped one it did — fails here.
        let mut expected = Vec::new();
        if subject.fenced_spaces.device {
            expected.push(AddressSpace::Device);
        }
        if subject.fenced_spaces.workgroup {
            expected.push(AddressSpace::Workgroup);
        }
        assert_eq!(spec.fenced_spaces, expected);
        barrier_call(&spec).expect("an admitted subject's spelling emits");
        emitted += 1;
    }
    assert_eq!(emitted, FENCES.len(), "every admitted subject must emit");
}
