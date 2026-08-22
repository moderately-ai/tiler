//! Admission, refusal, and identity tests for the scheduled gather relation.
//!
//! Every perturbation here breaks the **subject** — an ordinal, a relation, a
//! shape, a proof pairing — rather than the assertion, and each of the eight
//! association rules is driven separately so that a reddening perturbation
//! names which rule is load-bearing. A perturbation that reddened all eight
//! would show only that the gate runs.
//!
//! The fixture is a real vertical, not a hand-built proof: the retained
//! [`GatherIndexBoundsProof`] is minted by the index layer's verifier-private
//! deriver through [`IndexRegionBuilder::gather_read`], so a schedule here
//! carries evidence that the closed static argument actually ran. There is no
//! way to write one otherwise, which is exactly what the missing public
//! constructor buys.

use std::mem::variant_count;

use crate::index::{
    DomainRole, GatherIndexBoundsProof, GatherIndexBoundsProofKind, IndexRegionBuilder,
    TensorAccessView,
};
use crate::schedule::error::GatherAddressReadRule;
use crate::schedule::handles::{AccessOrdinal, BoundsWitnessId, OwnershipWitnessId, RegionId};
use crate::schedule::model::{
    Access, AccessMode, AxisDecode, BoundsProof, BoundsProofKind, ExecutionBinding, KernelSchedule,
    LaunchPlan, LogicalAccess, OwnershipProof, OwnershipProofKind, ReductionTopology,
    RegionProgram, ScalarProgram, TailPolicy, TensorRole, VerifiedScheduledRegion, element_count,
    gather_index_read_map,
};
use crate::schedule::{
    PointwiseF32ExpressionBuilder, ScheduledRegionBuilder, ScheduledRegionDiagnostic,
};
use crate::semantic::{F32, gather_index_resolved_type};
use crate::shape::{Axis, Shape};

/// The gathered extent of the fixture, chosen to reach `2^32`.
///
/// Only a statically proved gather reaches schedule formation, and the two
/// closed arguments are an empty result domain and a gathered extent containing
/// the whole U32 space. An empty domain would make every downstream count zero
/// and could hide an arithmetic defect, so the fixture takes the inhabited
/// argument.
const SOURCE_EXTENT: u64 = 1 << 32;
const RESULT_ELEMENTS: u64 = 6;

fn source_shape() -> Shape {
    Shape::from_dims([SOURCE_EXTENT, 3])
}
fn index_shape() -> Shape {
    Shape::from_dims([2])
}
fn result_shape() -> Shape {
    Shape::from_dims([2, 3])
}

/// Mints one real static gather proof through the index layer.
pub(crate) fn static_gather_proof() -> GatherIndexBoundsProof {
    let registry =
        crate::index::FrozenScalarRegistry::standard().expect("the scalar profile composes");
    let mut builder = IndexRegionBuilder::new(registry).expect("a builder is admitted");
    let dimensions: Vec<_> = result_shape()
        .extents()
        .iter()
        .map(|extent| {
            builder
                .dimension(DomainRole::Parallel, *extent)
                .expect("a parallel dimension is admitted")
        })
        .collect();
    let source = builder
        .tensor(
            crate::index::TensorRole::Input,
            F32::resolved_type().clone(),
            source_shape(),
        )
        .expect("the source boundary is admitted");
    let index = builder
        .tensor(
            crate::index::TensorRole::Input,
            gather_index_resolved_type(),
            index_shape(),
        )
        .expect("the index boundary is admitted");
    let output = builder
        .tensor(
            crate::index::TensorRole::Output,
            F32::resolved_type().clone(),
            result_shape(),
        )
        .expect("the output boundary is admitted");
    // Result axes are [source before axis | index | source after axis]. The
    // gather is on axis 0, so the index run leads and the one remaining source
    // axis trails: the source coordinate is result dimension 1.
    let source_coordinates = vec![
        builder
            .dimension_expr(dimensions[1])
            .expect("a dimension coordinate is admitted"),
    ];
    let index_coordinates = vec![
        builder
            .dimension_expr(dimensions[0])
            .expect("a dimension coordinate is admitted"),
    ];
    let value = builder
        .gather_read(
            source,
            index,
            &dimensions,
            &source_coordinates,
            &index_coordinates,
            Axis::new(0),
        )
        .expect("the gather is admitted");
    let write_coordinates: Vec<_> = dimensions
        .iter()
        .map(|dimension| {
            builder
                .dimension_expr(*dimension)
                .expect("a dimension coordinate is admitted")
        })
        .collect();
    let write = builder
        .write(output, &dimensions, &write_coordinates)
        .expect("the write is admitted");
    builder
        .output(write, value)
        .expect("the output is admitted");
    let region = builder.build().expect("the index region verifies");
    let proof = region
        .accesses()
        .find_map(|access| match access.view() {
            TensorAccessView::GatherRead(gather) => gather.bounds_resolution().statically_proved(),
            TensorAccessView::Direct(_) => None,
        })
        .expect("the gathered extent contains every U32 value, so the obligation is proved")
        .clone();
    assert_eq!(
        proof.kind(),
        GatherIndexBoundsProofKind::U32RangeContainedBySourceExtent,
        "the fixture must rest on the inhabited argument, not on vacuity",
    );
    proof
}

/// The relation the address read must carry, derived rather than restated.
fn derived_index_map() -> LogicalAccess {
    gather_index_read_map(&source_shape(), Axis::new(0), &index_shape())
        .expect("the fixture is a well-formed gather")
}

fn gather_relation_naming(index_access: AccessOrdinal) -> LogicalAccess {
    LogicalAccess::GatherSource {
        source_shape: source_shape(),
        result_shape: result_shape(),
        axis: Axis::new(0),
        index_access,
        index_shape: index_shape(),
    }
}

fn read(map: LogicalAccess, bounds: u32) -> Access {
    Access {
        tensor: TensorRole::Input,
        component_role: None,
        mode: AccessMode::Read,
        map,
        bounds: BoundsWitnessId::new(bounds),
        ownership: None,
    }
}

fn write_access(bounds: u32) -> Access {
    Access {
        tensor: TensorRole::Intermediate,
        component_role: None,
        mode: AccessMode::Write,
        map: LogicalAccess::LinearIdentity,
        bounds: BoundsWitnessId::new(bounds),
        ownership: Some(OwnershipWitnessId::new(0)),
    }
}

/// Derives each access's bounds proof from its own relation.
///
/// Deriving rather than restating is what lets a perturbation change one map
/// and leave the proofs correct, so the rule that fires is the one the
/// perturbation aimed at rather than a proof the test forgot to update.
fn proofs_for(accesses: &[Access]) -> Vec<BoundsProof> {
    accesses
        .iter()
        .map(|access| {
            let kind = match &access.map {
                LogicalAccess::GatherSource { index_access, .. } => BoundsProofKind::GatherSource {
                    source_shape: source_shape(),
                    result_shape: result_shape(),
                    axis: Axis::new(0),
                    index_access: *index_access,
                    index_shape: index_shape(),
                    proof: Box::new(static_gather_proof()),
                },
                LogicalAccess::BroadcastReplication { operand_shape, .. } => {
                    BoundsProofKind::LinearRange {
                        element_count: element_count(operand_shape).expect("a bounded operand"),
                    }
                }
                _ => BoundsProofKind::LinearRange {
                    element_count: RESULT_ELEMENTS,
                },
            };
            BoundsProof {
                id: access.bounds,
                tensor: access.tensor,
                component_role: None,
                kind,
            }
        })
        .collect()
}

/// Builds a region whose expression has `leaves` inputs, summed left to right.
fn build_with_proofs(
    accesses: Vec<Access>,
    proofs: Vec<BoundsProof>,
    leaves: usize,
) -> Result<VerifiedScheduledRegion, Vec<ScheduledRegionDiagnostic>> {
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(7));
    builder
        .iteration_shape(result_shape())
        .expect("the iteration shape is admitted");
    for access in accesses {
        builder.push_access(access).expect("an access is admitted");
    }
    for proof in proofs {
        builder
            .push_bounds_proof(proof)
            .expect("a bounds proof is admitted");
    }
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: RESULT_ELEMENTS,
            },
        })
        .expect("the ownership proof is admitted");
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let mut root = expression
        .input(AccessOrdinal::FIRST)
        .expect("one leaf is admitted");
    for ordinal in 1..leaves {
        let next = expression
            .input(AccessOrdinal::new(u32::try_from(ordinal).unwrap()))
            .expect("a further leaf is admitted");
        root = expression.add(root, next).expect("the sum composes");
    }
    let expression = expression.build(root).expect("the expression composes");
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(expression),
            numerical: super::tests::strict_numerical(),
        })
        .expect("the program is admitted");
    builder
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: RESULT_ELEMENTS,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: RESULT_ELEMENTS,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .expect("the schedule is admitted");
    builder
        .build()
        .map_err(|error| error.diagnostics().to_vec())
}

fn build(
    accesses: Vec<Access>,
    leaves: usize,
) -> Result<VerifiedScheduledRegion, Vec<ScheduledRegionDiagnostic>> {
    let proofs = proofs_for(&accesses);
    build_with_proofs(accesses, proofs, leaves)
}

/// The one gather, one address read, one write fixture.
fn one_gather_region() -> Vec<Access> {
    vec![
        read(gather_relation_naming(AccessOrdinal::new(1)), 0),
        read(derived_index_map(), 1),
        write_access(2),
    ]
}

/// Two gathers, each owning its own address read.
fn two_gather_region() -> Vec<Access> {
    vec![
        read(gather_relation_naming(AccessOrdinal::new(2)), 0),
        read(gather_relation_naming(AccessOrdinal::new(3)), 1),
        read(derived_index_map(), 2),
        read(derived_index_map(), 3),
        write_access(4),
    ]
}

/// Returns the single association rule a perturbed region reports.
fn association_rule(accesses: Vec<Access>, leaves: usize) -> GatherAddressReadRule {
    let diagnostics = build(accesses, leaves).expect_err("the perturbed region must refuse");
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    match diagnostics[0] {
        ScheduledRegionDiagnostic::GatherAddressRead { rule, .. } => rule,
        other => panic!("expected a gather association failure, got {other:?}"),
    }
}

#[test]
fn a_statically_proved_gather_region_verifies() {
    let verified = build(one_gather_region(), 1).expect("the fixture verifies");
    assert_eq!(verified.region().schedule.work_items, RESULT_ELEMENTS);
    // Three boundary effects: the gathered source, the address operand, and the
    // owning write. The address read is a binding of its own rather than a
    // coordinate folded into the source's.
    assert_eq!(verified.requirements().buffer_bindings, 3);
}

#[test]
fn two_independent_gathers_in_one_region_verify() {
    let verified = build(two_gather_region(), 2).expect("two gathers compose");
    assert_eq!(verified.requirements().buffer_bindings, 5);
}

/// The address relation is derived, in the three forms the accepted surface
/// states, rather than being whatever the fixture happens to hold.
#[test]
fn the_address_relation_is_derived_across_all_three_admitted_forms() {
    // Equal result and index shape reads the identity relation: source `[7]`
    // gathered on its only axis by an index of `[7]` has result `[7]`.
    assert_eq!(
        gather_index_read_map(&Shape::from_dims([7]), Axis::new(0), &Shape::from_dims([7])),
        Some(LogicalAccess::LinearIdentity),
    );
    // A rank-zero index holds exactly one address, read by every invocation.
    assert_eq!(
        gather_index_read_map(
            &Shape::from_dims([7, 4]),
            Axis::new(0),
            &Shape::from_dims([])
        ),
        Some(LogicalAccess::ScalarBroadcast),
    );
    // Otherwise the read widens: the index run names result axis 0 and the
    // trailing source axis is replicated.
    assert_eq!(
        derived_index_map(),
        LogicalAccess::BroadcastReplication {
            operand_shape: index_shape(),
            result_shape: result_shape(),
            axes: vec![AxisDecode {
                divisor: 3,
                modulus: 2,
                mirrored: false,
            }],
        },
    );
    // A malformed relation has no derivation at all rather than a defaulted
    // one: a rank-zero source and an out-of-range axis both answer `None`.
    assert_eq!(
        gather_index_read_map(&Shape::from_dims([]), Axis::new(0), &index_shape()),
        None,
    );
    assert_eq!(
        gather_index_read_map(&source_shape(), Axis::new(9), &index_shape()),
        None,
    );
}

#[test]
fn an_address_read_at_or_before_its_source_is_not_later() {
    let mut accesses = one_gather_region();
    accesses[0].map = gather_relation_naming(AccessOrdinal::new(0));
    assert_eq!(
        association_rule(accesses, 1),
        GatherAddressReadRule::IndexNotLater
    );
}

#[test]
fn an_address_ordinal_past_the_access_list_is_not_later() {
    let mut accesses = one_gather_region();
    accesses[0].map = gather_relation_naming(AccessOrdinal::new(9));
    assert_eq!(
        association_rule(accesses, 1),
        GatherAddressReadRule::IndexNotLater
    );
}

#[test]
fn an_address_read_on_an_intermediate_is_refused_by_mode() {
    let mut accesses = one_gather_region();
    accesses[1].tensor = TensorRole::Intermediate;
    assert_eq!(
        association_rule(accesses, 1),
        GatherAddressReadRule::IndexMode
    );
}

#[test]
fn an_address_read_carrying_another_relation_is_refused_by_relation() {
    let mut accesses = one_gather_region();
    accesses[1].map = LogicalAccess::LinearIdentity;
    assert_eq!(
        association_rule(accesses, 1),
        GatherAddressReadRule::IndexRelation
    );
}

/// A gather naming a read inside the leaf run is refused under its own rule.
///
/// Needs a two-leaf expression: with one leaf the gather source *is* that leaf,
/// so there is no earlier leaf an address ordinal could point at. The address
/// read keeps the derived relation so rule 3 passes and the scalar-leaf rule is
/// the one that fires.
#[test]
fn an_address_read_inside_the_leaf_run_is_refused_as_a_scalar_leaf() {
    let accesses = vec![
        read(gather_relation_naming(AccessOrdinal::new(1)), 0),
        read(derived_index_map(), 1),
        write_access(2),
    ];
    assert_eq!(
        association_rule(accesses, 2),
        GatherAddressReadRule::IndexUsedAsScalarLeaf
    );
}

/// Two gathers naming one address read is refused under its own rule.
#[test]
fn two_gathers_naming_one_address_read_are_refused_as_shared() {
    let mut accesses = two_gather_region();
    accesses[1].map = gather_relation_naming(AccessOrdinal::new(2));
    let diagnostics = build(accesses, 2).expect_err("the perturbed region must refuse");
    assert_eq!(
        diagnostics,
        vec![ScheduledRegionDiagnostic::GatherAddressRead {
            source_access: Some(AccessOrdinal::new(0)),
            index_access: AccessOrdinal::new(2),
            rule: GatherAddressReadRule::IndexShared,
        }],
        "the first claimant is named beside the contested address read",
    );
}

/// An address-only read that no gather names is refused as orphaned.
///
/// This is the one rule whose `source_access` is `None`, and reaching it is what
/// the read-count gate's deliberate asymmetry exists for: an equality against
/// `input_count + gathers` would refuse this region as a wrong count before the
/// bijection could name *which* read is unowned.
#[test]
fn an_address_read_no_gather_names_is_unowned() {
    let mut accesses = two_gather_region();
    // The second gather becomes an ordinary leaf, so access 3 is named by
    // nobody while access 2 keeps its owner.
    accesses[1].map = LogicalAccess::LinearIdentity;
    let diagnostics = build(accesses, 2).expect_err("the perturbed region must refuse");
    assert_eq!(
        diagnostics,
        vec![ScheduledRegionDiagnostic::GatherAddressRead {
            source_access: None,
            index_access: AccessOrdinal::new(3),
            rule: GatherAddressReadRule::IndexUnowned,
        }],
        "the orphan is named, and no source claims it",
    );
}

#[test]
fn a_relation_whose_result_shape_is_not_derived_fails_occurrence_binding() {
    let mut accesses = one_gather_region();
    // The source, axis, and index are left exactly as the fixture has them, so
    // `gather_index_read_map` derives the same address relation and rule 3
    // still passes. Only the *stated* result shape is wrong — it is the
    // transpose of the derived one, so it is neither the gather composition nor
    // the region's iteration domain. Perturbing the source shape instead would
    // change the derived address map and trip `IndexRelation` first, which is
    // the correct precedence but tests a different rule.
    accesses[0].map = LogicalAccess::GatherSource {
        source_shape: source_shape(),
        result_shape: Shape::from_dims([3, 2]),
        axis: Axis::new(0),
        index_access: AccessOrdinal::new(1),
        index_shape: index_shape(),
    };
    assert_eq!(
        association_rule(accesses, 1),
        GatherAddressReadRule::OccurrenceBinding
    );
}

#[test]
fn a_proof_restating_a_different_relation_is_a_proof_mismatch() {
    let accesses = one_gather_region();
    let mut proofs = proofs_for(&accesses);
    let BoundsProofKind::GatherSource { proof, .. } = proofs[0].kind.clone() else {
        unreachable!("the fixture's first proof is the gather's")
    };
    // The relation names access 1; the proof claims access 2.
    proofs[0].kind = BoundsProofKind::GatherSource {
        source_shape: source_shape(),
        result_shape: result_shape(),
        axis: Axis::new(0),
        index_access: AccessOrdinal::new(2),
        index_shape: index_shape(),
        proof,
    };
    let diagnostics =
        build_with_proofs(accesses, proofs, 1).expect_err("the perturbed region must refuse");
    assert_eq!(
        diagnostics,
        vec![ScheduledRegionDiagnostic::GatherAddressRead {
            source_access: Some(AccessOrdinal::new(0)),
            index_access: AccessOrdinal::new(1),
            rule: GatherAddressReadRule::ProofMismatch,
        }],
    );
}

/// A gather relation paired with a plain linear range is refused, not admitted.
#[test]
fn a_gather_relation_paired_with_a_linear_range_is_refused() {
    let accesses = one_gather_region();
    let mut proofs = proofs_for(&accesses);
    proofs[0].kind = BoundsProofKind::LinearRange {
        element_count: RESULT_ELEMENTS,
    };
    let diagnostics =
        build_with_proofs(accesses, proofs, 1).expect_err("the perturbed region must refuse");
    assert_eq!(
        diagnostics,
        vec![ScheduledRegionDiagnostic::GatherAddressRead {
            source_access: Some(AccessOrdinal::new(0)),
            index_access: AccessOrdinal::new(1),
            rule: GatherAddressReadRule::ProofMismatch,
        }],
        "the association gate owns proof pairing, and names it rather than a bucket",
    );
}

/// The eight accepted rules, sized from the enum rather than by hand.
///
/// A widened vocabulary is a length type error here rather than a census that
/// has quietly stopped covering its own domain.
#[test]
fn the_gather_address_read_rule_census_is_exactly_the_accepted_eight() {
    const RULES: [GatherAddressReadRule; variant_count::<GatherAddressReadRule>()] = [
        GatherAddressReadRule::IndexNotLater,
        GatherAddressReadRule::IndexMode,
        GatherAddressReadRule::IndexRelation,
        GatherAddressReadRule::IndexUsedAsScalarLeaf,
        GatherAddressReadRule::IndexShared,
        GatherAddressReadRule::IndexUnowned,
        GatherAddressReadRule::OccurrenceBinding,
        GatherAddressReadRule::ProofMismatch,
    ];
    assert_eq!(RULES.len(), 8, "the accepted census is exactly eight rules");
    let identifiers: Vec<&'static str> = RULES.iter().map(|rule| rule.rule()).collect();
    assert_eq!(
        identifiers,
        [
            "gather-address-read-not-later",
            "gather-address-read-mode",
            "gather-address-read-relation",
            "gather-address-read-scalar-leaf",
            "gather-address-read-shared",
            "gather-address-read-unowned",
            "gather-address-read-occurrence-binding",
            "gather-address-read-proof-mismatch",
        ],
        "the stable identifiers are pinned exactly",
    );
    let mut deduplicated = identifiers.clone();
    deduplicated.sort_unstable();
    deduplicated.dedup();
    assert_eq!(deduplicated.len(), RULES.len(), "no two rules share a name");
    // Each rule reaches `ScheduledRegionDiagnostic::rule()` unchanged, so a
    // consumer surfacing an explanation sees the association rule rather than a
    // bucket name.
    for rule in RULES {
        let diagnostic = ScheduledRegionDiagnostic::GatherAddressRead {
            source_access: None,
            index_access: AccessOrdinal::FIRST,
            rule,
        };
        assert_eq!(diagnostic.rule(), rule.rule());
    }
}

/// A gather's proof and an address read's proof may not share one witness id.
///
/// The reachable half of the witness-collision finding, and the one with a
/// consequence: both records stay individually well formed against their own
/// positional access — the gather's `GatherSource` proof refines its gather,
/// the address read's `LinearRange { element_count: 2 }` refines its
/// replication — so nothing in the refinement pass objects. What breaks is
/// *resolution*. [`crate::kernel::verify::access_elements`] looks a proof up by
/// id and takes the first record bearing it, so the address read resolves to
/// the **gather's** proof and sizes its buffer parameter from
/// `element_count(source_shape)` — `SOURCE_EXTENT * 3`, or 12884901888
/// elements, for a read of two addresses. That count reaches the emitted kernel
/// through `CanonicalPlan::read_elements`, so the collision is not a tidiness
/// defect.
///
/// The sibling collision between two *gathers* cannot get this far: rules 5 and
/// 8 already close it, which is the subject of the test below.
#[test]
fn a_gather_and_an_address_read_may_not_share_one_bounds_witness() {
    let mut accesses = two_gather_region();
    // The first gather's own witness, reused by the address read at access 2.
    accesses[2].bounds = BoundsWitnessId::new(0);
    let proofs = proofs_for(&accesses);
    // The two records the collision would make indistinguishable prove
    // genuinely different domains, so this is not a distinction without a
    // difference.
    assert!(matches!(
        proofs[0].kind,
        BoundsProofKind::GatherSource { .. }
    ));
    assert_eq!(
        proofs[2].kind,
        BoundsProofKind::LinearRange { element_count: 2 },
    );
    assert_eq!(
        build_with_proofs(accesses, proofs, 2).expect_err("the colliding region must refuse"),
        vec![ScheduledRegionDiagnostic::ProofReference],
    );
}

/// Two gathers sharing one witness id are refused, and by the association gate.
///
/// Pinned because the precedence is not obvious and a later reader could
/// otherwise assume the distinctness clause is what catches this. It is not:
/// `verify_gather_address_reads` runs first, and rule 8 resolves the second
/// gather's proof by id onto the *first* gather's record, whose `index_access`
/// names a different address read. Rule 5 has already forced those two ordinals
/// apart — two gathers may not name one address read — so the mismatch is
/// guaranteed rather than incidental. The witness collision is unreachable here
/// by pigeonhole, which is exactly why the reachable case above is the one that
/// carries a consequence.
#[test]
fn two_gathers_sharing_one_bounds_witness_are_refused_by_the_association_gate() {
    let mut accesses = two_gather_region();
    accesses[1].bounds = BoundsWitnessId::new(0);
    let proofs = proofs_for(&accesses);
    assert_eq!(
        build_with_proofs(accesses, proofs, 2).expect_err("the colliding region must refuse"),
        vec![ScheduledRegionDiagnostic::GatherAddressRead {
            source_access: Some(AccessOrdinal::new(1)),
            index_access: AccessOrdinal::new(3),
            rule: GatherAddressReadRule::ProofMismatch,
        }],
    );
}

/// Closing the collision leaves the two-gather fixture itself admitted.
///
/// The negative control for the two tests above: the distinctness clause must
/// refuse a reused witness without refusing the distinct witnesses every real
/// region carries.
#[test]
fn distinct_witnesses_across_two_gathers_and_their_address_reads_still_verify() {
    let verified = build(two_gather_region(), 2).expect("distinct witnesses stay admitted");
    assert_eq!(verified.requirements().buffer_bindings, 5);
}
