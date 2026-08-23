use super::super::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, ExecutionBinding,
    LogicalAccess, ReductionTopology, RegionProgram, ScalarProgram, ScheduledRegionBuilder,
    TensorRole, encode_identity,
};
use super::support::{
    STRICT_F32_REGION_IDENTITY_HEX, admitted_lanes, partitioned_copy_builder,
    scale_bias_expression, strict_numerical,
};
use crate::schedule::handles::BoundsWitnessId;
use crate::schedule::model::{
    ContributorOrder, CopyElement, CopyMember, PartitionedCopyProgram, RegionNumericalRequirements,
};
use crate::shape::{Axis, Shape};
use std::fmt::Write as _;

/// The one partitioned-copy diagnostic each perturbation below must name.
#[track_caller]
fn assert_copy_rule(builder: ScheduledRegionBuilder, rule: &str) {
    let diagnostics = builder.build().unwrap_err().diagnostics().to_vec();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].rule(), rule, "{diagnostics:?}");
}

/// Recorded canonical identity of the arity-2 `[2, 3]` axis-0 copy fixture.
///
/// One member per operand at extents 1 + 1 over two distinct reads. Pinned
/// so a later payload or tag move fails this pin rather than only the
/// injectivity arguments.
const ARITY_TWO_COPY_IDENTITY_HEX: &str = "74696c65722e7363686564756c652e76370000000000000000020000000000000002000000000000000300000000000000030100010d00000000000100010d0000000100030002010000000201000000000000000000000003000000000100110000000000000003000000010100110000000000000003000000020300110000000000000006000000000300000000000000062b010000000000000000000000020000000000000000000000010000000100000000000000010100000000000000060000000101000000003100000000000000060000000101";

#[test]
fn an_arity_two_partitioned_copy_verifies_and_derives_the_copy_requirements() {
    let verified = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)])
        .build()
        .expect("an arity-2 partitioned copy verifies");
    // Structural rows stay unconditional; the numerical requirement is the
    // proved absence, never a fabricated strict row.
    let requirements = verified.requirements();
    assert_eq!(requirements.buffer_bindings, 3);
    assert!(requirements.requires_device_memory);
    assert_eq!(
        requirements.numerical,
        RegionNumericalRequirements::BitPreservingCopy
    );
    assert_eq!(requirements.synchronization, None);
    assert_eq!(requirements.local_memory_bytes, 0);
    // The fail-closed freedom answer: nothing proves a copy's payloads
    // bounded away from the subnormal range.
    assert_eq!(
        verified.subnormal_freedom(),
        crate::schedule::SubnormalFreedom::Unproven
    );
}

#[test]
fn an_arity_eight_partitioned_copy_verifies_at_every_axis_position() {
    // Arity 8 with unequal extents, including a zero-extent member that
    // stays in identity and executes no access.
    let extents = [3, 1, 0, 2, 5, 1, 4, 2];
    let members: Vec<(u32, u64)> = extents
        .iter()
        .enumerate()
        .map(|(source, extent)| (u32::try_from(source).unwrap(), *extent))
        .collect();
    let total: u64 = extents.iter().sum();
    for axis in 0..3_u32 {
        let mut dims = [2_u64, 3, 4];
        dims[usize::try_from(axis).unwrap()] = total;
        let verified = partitioned_copy_builder(&Shape::from_dims(dims), axis, &members)
            .build()
            .expect("an arity-8 partitioned copy verifies at every axis position");
        assert_eq!(verified.region().index.accesses.len(), 9);
    }
}

#[test]
fn an_all_zero_partitioned_copy_verifies_and_skips_dispatch() {
    let verified = partitioned_copy_builder(&Shape::from_dims([0, 3]), 0, &[(0, 0), (1, 0)])
        .build()
        .expect("an all-zero partitioned copy verifies");
    assert_eq!(verified.region().schedule.work_items, 0);
    assert!(verified.region().schedule.launch.zero_work_skips_dispatch);
}

#[test]
fn a_repeated_operand_copy_is_two_members_over_one_deduplicated_read() {
    // `concat(x, x)`: two ordered members, one boundary read.
    let verified = partitioned_copy_builder(&Shape::from_dims([4, 3]), 0, &[(0, 2), (0, 2)])
        .build()
        .expect("concat(x, x) verifies as two members over one read");
    assert_eq!(verified.region().index.accesses.len(), 2);
}

/// A legal reorder of otherwise-identical operand extents is a different
/// program with a different canonical identity: member order is semantic.
#[test]
fn reordering_members_moves_the_canonical_identity() {
    let forward = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)])
        .build()
        .expect("the forward order verifies");
    let reordered = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 3), (1, 1)])
        .build()
        .expect("the reordered program verifies");
    assert_ne!(
        forward.canonical_identity().as_bytes(),
        reordered.canonical_identity().as_bytes()
    );
}

#[test]
fn the_arity_two_copy_has_its_recorded_canonical_identity() {
    let verified = partitioned_copy_builder(&Shape::from_dims([2, 3]), 0, &[(0, 1), (1, 1)])
        .build()
        .expect("the pinned fixture verifies");
    let mut hex = String::new();
    for byte in verified.canonical_identity().as_bytes() {
        write!(hex, "{byte:02x}").unwrap();
    }
    assert_eq!(hex, ARITY_TWO_COPY_IDENTITY_HEX);
}

/// Cross-arm discrimination at the program position, on equal prefixes.
///
/// Two regions sharing every byte before the program position — same
/// shape, accesses, proofs, and ownership — first diverge exactly at the
/// program tag: `0x24` (pointwise) against `0x2B` (partitioned copy). The
/// arithmetic twin is deliberately unverifiable (a pointwise region may
/// not carry the copy-source map), so the control encodes both directly:
/// injectivity is a property of the encoder over every encodable region,
/// not only the verifiable ones.
#[test]
fn the_copy_and_numerical_arms_first_diverge_at_the_program_position_byte() {
    let copy = partitioned_copy_builder(&Shape::from_dims([2, 3]), 0, &[(0, 1), (0, 1)])
        .build()
        .expect("the copy twin verifies");
    let copy_bytes = copy.canonical_identity().as_bytes().to_vec();
    let mut twin = copy.region().clone();
    twin.index.program = RegionProgram::Numerical {
        scalar: ScalarProgram::PointwiseF32(scale_bias_expression(
            2.0_f32.to_bits(),
            1.0_f32.to_bits(),
        )),
        numerical: strict_numerical(),
    };
    let twin_bytes = encode_identity(&twin);
    let twin_bytes = twin_bytes.as_bytes();
    let diverge = copy_bytes
        .iter()
        .zip(twin_bytes)
        .position(|(copy, twin)| copy != twin)
        .expect("the two arms cannot share an identity");
    assert_eq!(copy_bytes[diverge], 0x2b, "the copy arm's program tag");
    assert_eq!(twin_bytes[diverge], 0x24, "the pointwise program tag");
}

/// Against the pinned strict-`f32` identity the first divergence sits at
/// the first read's access-map byte, ahead of the program position: the
/// copy read carries the appended `0x0d` map tag where the pinned region's
/// read is `0x01` (`LinearIdentity`). The packet's byte control named the
/// program-position byte, which no fixture can realize against this pin —
/// the access map is encoded first — so the realizable control is stated
/// and the program-position discrimination is proved on equal prefixes
/// above.
#[test]
fn against_the_pinned_strict_identity_the_copy_first_diverges_at_the_access_map_byte() {
    let copy = partitioned_copy_builder(&Shape::from_dims([2, 3]), 0, &[(0, 1), (0, 1)])
        .build()
        .expect("the two-access copy fixture verifies");
    let copy_bytes = copy.canonical_identity().as_bytes();
    let pinned: Vec<u8> = (0..STRICT_F32_REGION_IDENTITY_HEX.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&STRICT_F32_REGION_IDENTITY_HEX[index..index + 2], 16).unwrap()
        })
        .collect();
    let diverge = copy_bytes
        .iter()
        .zip(&pinned)
        .position(|(copy, pinned)| copy != pinned)
        .expect("a copy and an arithmetic region cannot share an identity");
    // Domain (18) + framed shape (8 + 16) + access count (8) + the first
    // access's role, component, and mode bytes (3) = 53: the map tag.
    assert_eq!(diverge, 53);
    assert_eq!(copy_bytes[diverge], 0x0d, "the appended copy-source tag");
    assert_eq!(pinned[diverge], 0x01, "the pinned read's linear identity");
}

#[test]
fn derived_offsets_and_source_shapes_answer_and_refuse_overflow_by_absence() {
    let program = PartitionedCopyProgram {
        element: CopyElement::F32,
        axis: Axis::new(0),
        members: vec![
            CopyMember {
                source: AccessOrdinal::new(0),
                extent: 1,
            },
            CopyMember {
                source: AccessOrdinal::new(1),
                extent: 3,
            },
        ],
    };
    assert_eq!(program.member_offsets(), Some(vec![0, 1]));
    let shape = Shape::from_dims([4, 5]);
    assert_eq!(
        program.member_source_shape(&shape, 1),
        Some(Shape::from_dims([3, 5]))
    );
    assert_eq!(program.member_source_shape(&shape, 2), None);
    let overflowing = PartitionedCopyProgram {
        element: CopyElement::F32,
        axis: Axis::new(0),
        members: vec![
            CopyMember {
                source: AccessOrdinal::new(0),
                extent: u64::MAX,
            },
            CopyMember {
                source: AccessOrdinal::new(1),
                extent: 2,
            },
        ],
    };
    assert_eq!(overflowing.member_offsets(), None);
}

#[test]
fn a_copy_with_a_serial_reduction_is_refused_by_topology() {
    let mut builder = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)]);
    let schedule = builder.schedule.as_mut().unwrap();
    schedule.reduction = ReductionTopology::Serial {
        axes: vec![Axis::new(0)],
        order: ContributorOrder::OriginalAxisLexicographic,
        permits_reassociation: false,
        permits_permutation: false,
    };
    assert_copy_rule(builder, "partitioned-copy-topology");
}

/// The binding clause of the topology rule, watched through the one
/// binding the shared gates admit over `ReductionTopology::None`: the
/// fixed-vector map. The blocked binding and the predicated tail are
/// refused by the shared gates ahead of the copy dispatch, so the
/// reduction clause and this one are the independently reachable clauses.
#[test]
fn a_copy_under_a_fixed_vector_binding_is_refused_by_topology() {
    let mut builder = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)]);
    let schedule = builder.schedule.as_mut().unwrap();
    schedule.binding = ExecutionBinding::FixedVectorMap {
        lanes: admitted_lanes(2),
    };
    schedule.launch.grid_threads = 10;
    assert_copy_rule(builder, "partitioned-copy-topology");
}

#[test]
fn a_copy_reading_an_intermediate_is_refused_by_read_tensor() {
    let mut builder = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)]);
    builder.accesses[0].tensor = TensorRole::Intermediate;
    builder.bounds_proofs[0].tensor = TensorRole::Intermediate;
    assert_copy_rule(builder, "partitioned-copy-read-tensor");
}

#[test]
fn a_copy_writing_an_intermediate_is_refused_by_write_tensor() {
    let mut builder = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)]);
    builder.accesses[2].tensor = TensorRole::Intermediate;
    builder.bounds_proofs[2].tensor = TensorRole::Intermediate;
    builder.ownership_proof.as_mut().unwrap().tensor = TensorRole::Intermediate;
    assert_copy_rule(builder, "partitioned-copy-write-tensor");
}

#[test]
fn a_single_member_copy_is_refused_by_member_count() {
    // Dropping one member of an arity-2 fixture leaves one member: the
    // count rule fires ahead of the coverage sum.
    let mut builder = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)]);
    let Some(RegionProgram::PartitionedCopy(program)) = builder.program.as_mut() else {
        panic!("the fixture is a partitioned copy");
    };
    program.members.pop();
    assert_copy_rule(builder, "partitioned-copy-member-count");
}

#[test]
fn an_out_of_rank_axis_is_refused_by_axis_range() {
    let mut builder = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)]);
    let Some(RegionProgram::PartitionedCopy(program)) = builder.program.as_mut() else {
        panic!("the fixture is a partitioned copy");
    };
    program.axis = Axis::new(2);
    assert_copy_rule(builder, "partitioned-copy-axis-range");
}

#[test]
fn a_member_naming_the_write_is_refused_by_source_reference() {
    let mut builder = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)]);
    let Some(RegionProgram::PartitionedCopy(program)) = builder.program.as_mut() else {
        panic!("the fixture is a partitioned copy");
    };
    // Ordinal 2 is the write's position in the access list.
    program.members[1].source = AccessOrdinal::new(2);
    assert_copy_rule(builder, "partitioned-copy-source-reference");
}

#[test]
fn a_permuted_read_list_is_refused_by_source_order() {
    // First references 1 then 0: one meaning would otherwise have two
    // identities under a permuted read list with renumbered ordinals.
    let mut builder = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 3), (1, 1)]);
    let Some(RegionProgram::PartitionedCopy(program)) = builder.program.as_mut() else {
        panic!("the fixture is a partitioned copy");
    };
    program.members[0].source = AccessOrdinal::new(1);
    program.members[1].source = AccessOrdinal::new(0);
    assert_copy_rule(builder, "partitioned-copy-source-order");
}

#[test]
fn an_unreferenced_read_is_refused_by_unreferenced_source() {
    let mut builder = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)]);
    // A third read no member references, spliced in ahead of the write
    // with its structural proof.
    builder.accesses.insert(
        2,
        Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::PartitionedCopySource,
            bounds: BoundsWitnessId::new(3),
            ownership: None,
        },
    );
    builder.bounds_proofs.insert(
        2,
        BoundsProof {
            id: BoundsWitnessId::new(3),
            tensor: TensorRole::Input,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 0 },
        },
    );
    assert_copy_rule(builder, "partitioned-copy-unreferenced-source");
}

#[test]
fn an_overflowing_extent_is_refused_by_extent_overflow() {
    let mut builder = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)]);
    let Some(RegionProgram::PartitionedCopy(program)) = builder.program.as_mut() else {
        panic!("the fixture is a partitioned copy");
    };
    program.members[0].extent = u64::MAX;
    program.members[1].extent = 2;
    assert_copy_rule(builder, "partitioned-copy-extent-overflow");
}

#[test]
fn a_wrong_extent_total_is_refused_by_coverage_sum_and_the_compensated_domain_passes() {
    // One member's extent moved by one: the only representable coverage
    // defect, covering both the would-be gap and the would-be overlap.
    let mut short = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)]);
    let Some(RegionProgram::PartitionedCopy(program)) = short.program.as_mut() else {
        panic!("the fixture is a partitioned copy");
    };
    program.members[1].extent = 2;
    assert_copy_rule(short, "partitioned-copy-coverage-sum");
    // Dropping one member of an arity-3 fixture is the same defect at a
    // legal member count.
    let mut dropped =
        partitioned_copy_builder(&Shape::from_dims([6, 5]), 0, &[(0, 1), (1, 3), (2, 2)]);
    let Some(RegionProgram::PartitionedCopy(program)) = dropped.program.as_mut() else {
        panic!("the fixture is a partitioned copy");
    };
    program.members.pop();
    dropped.accesses.remove(2);
    dropped.bounds_proofs.remove(2);
    dropped.accesses[2].bounds = BoundsWitnessId::new(2);
    dropped.bounds_proofs[2].id = BoundsWitnessId::new(2);
    assert_copy_rule(dropped, "partitioned-copy-coverage-sum");
    // The compensated axis extent readmits the perturbed extents, so the
    // refusal above is the coverage fact and not a broken fixture.
    partitioned_copy_builder(&Shape::from_dims([3, 5]), 0, &[(0, 1), (1, 2)])
        .build()
        .expect("the compensated domain verifies");
}

#[test]
fn disagreeing_member_extents_and_a_wrong_proof_count_are_refused_by_source_shape() {
    // Two members of one read with disagreeing extents.
    let mut disagreeing = partitioned_copy_builder(&Shape::from_dims([5, 3]), 0, &[(0, 2), (0, 3)]);
    let Some(RegionProgram::PartitionedCopy(_)) = disagreeing.program.as_mut() else {
        panic!("the fixture is a partitioned copy");
    };
    assert_copy_rule(disagreeing, "partitioned-copy-source-shape");
    // A read's bounds-proof element count disagreeing with the derived
    // source element count — the exactness the structural refinement arm
    // deliberately defers to this rule.
    let mut wrong_proof = partitioned_copy_builder(&Shape::from_dims([4, 5]), 0, &[(0, 1), (1, 3)]);
    let BoundsProofKind::LinearRange { element_count } = &mut wrong_proof.bounds_proofs[0].kind
    else {
        panic!("the fixture proof is a linear range");
    };
    *element_count += 1;
    assert_copy_rule(wrong_proof, "partitioned-copy-source-shape");
}
