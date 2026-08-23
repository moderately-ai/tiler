use super::super::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, ContributorArrival,
    ContributorCoverage, CooperativeTileRule, FamilyTopology, KernelSchedule, LogicalAccess,
    MAX_COOPERATIVE_ROUNDS, NumericalRealization, OwnershipProof, OwnershipProofKind,
    ParallelFamily, ReductionPass, ReductionTopology, RegionId, RegionProgram, ScalarProgram,
    ScheduledRegionBuilder, ScheduledRegionDiagnostic, TensorRole, element_count, encode_identity,
    split_family,
};
use super::support::{
    NEG_ZERO, PADDED_SPLIT, SPLIT, bare_sum, cooperative_builder, cooperative_builder_parts,
    cooperative_rejection, cooperative_tile_fixture, cooperative_topology_arriving,
    extrema_cooperative_builder, extrema_partial_builder, final_pass_builder, float_rows,
    linear_schedule, maximum_scalar, partial_pass_builder, read_from, reassociating_numerical,
    round_perturbed, scale_epilogue, serial_reduction_builder, set_numerical, set_scalar,
    squared_partial_pass_builder, squared_sum_with_epilogue, strict_numerical,
};
use crate::schedule::PointwiseF32ExpressionBuilder;
use crate::schedule::handles::{BoundsWitnessId, OwnershipWitnessId};
use crate::schedule::model::{ContributorOrder, ContributorPartition};
use crate::schedule::numerics::{ArithmeticType, NumericalPermission};
use crate::shape::{Axis, Shape};

/// A squaring fold carrying an epilogue verifies as a serial pass.
///
/// The control every refusal below is stated against: the fold's own
/// obligations are the squaring sum's, unchanged, and the epilogue adds two
/// of its own without changing what the region reads or writes — one read of
/// the contributor domain, one owning write.
#[test]
fn a_fold_carrying_an_epilogue_verifies_as_a_serial_pass() {
    let region = serial_reduction_builder(squared_sum_with_epilogue(scale_epilogue()))
        .build()
        .expect("a squaring fold with a scalar epilogue verifies");
    assert!(matches!(
        region.region().index.program,
        RegionProgram::Numerical {
            scalar: ScalarProgram::SquaredSerialSumThenEpilogue { .. },
            ..
        }
    ));
    // One read and one write, exactly as the bare fold declares: the
    // epilogue's leaf is the folded value, so it binds no buffer.
    assert_eq!(region.region().index.accesses.len(), 2);
    assert_eq!(region.requirements().buffer_bindings, 2);
}

/// An epilogue that computes nothing is refused rather than admitted.
///
/// **The canonicality rule this variant owes.** An expression whose root is
/// its own input leaf returns the fold's value unchanged, which is exactly
/// what [`ScalarProgram::SquaredSerialSum`] computes — so admitting it would
/// give one program two spellings and two canonical identities, and a cache
/// holding either would miss the other for the same computation.
#[test]
fn a_fold_epilogue_that_computes_nothing_is_refused() {
    let mut builder = PointwiseF32ExpressionBuilder::new();
    let leaf = builder.input(AccessOrdinal::FIRST).unwrap();
    let identity = builder.build(leaf).unwrap();
    assert_eq!(
        serial_reduction_builder(squared_sum_with_epilogue(identity))
            .build()
            .unwrap_err()
            .diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
    );
}

/// An epilogue naming a second input is refused rather than bound.
///
/// A fold region reads exactly one boundary tensor, and the epilogue's sole
/// ordinal is the folded value rather than a buffer — so a second leaf names
/// an input nothing binds, and the lowering would have no value to supply for
/// it. Refusing it here is what keeps that from being a handle error deep in
/// the kernel builder.
#[test]
fn a_fold_epilogue_reading_a_second_input_is_refused() {
    let mut builder = PointwiseF32ExpressionBuilder::new();
    let total = builder.input(AccessOrdinal::FIRST).unwrap();
    let other = builder.input(AccessOrdinal::new(1)).unwrap();
    let sum = builder.add(total, other).unwrap();
    let two_leaves = builder.build(sum).unwrap();
    assert_eq!(two_leaves.input_count(), 2);
    assert_eq!(
        serial_reduction_builder(squared_sum_with_epilogue(two_leaves))
            .build()
            .unwrap_err()
            .diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
    );
}

/// A fold carrying an epilogue does not share identity with the bare fold.
///
/// The two regions differ in nothing but their scalar program — same access
/// relation, same contributor order, same numerical realization — so an
/// appended tag that had collided with `0x26` would make these equal. It is
/// the check behind "the schedule domain did not step": the new tag
/// separates, and every earlier tag keeps its meaning.
///
/// The second pair separates two *epilogues*: a chain dividing by six and one
/// dividing by seven are different functions, so the expression payload has
/// to reach the identity bytes rather than only the tag.
#[test]
fn a_fold_epilogue_separates_scheduled_region_identity() {
    let bare = serial_reduction_builder(ScalarProgram::SquaredSerialSum {
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0.0_f32.to_bits(),
    })
    .build()
    .unwrap();
    let scaled = serial_reduction_builder(squared_sum_with_epilogue(scale_epilogue()))
        .build()
        .unwrap();
    assert_ne!(
        bare.canonical_identity().as_bytes(),
        scaled.canonical_identity().as_bytes(),
    );

    let mut other = PointwiseF32ExpressionBuilder::new();
    let total = other.input(AccessOrdinal::FIRST).unwrap();
    let extent = other.constant(7.0_f32.to_bits()).unwrap();
    let mean = other.divide(total, extent).unwrap();
    let bias = other.constant(1.0e-6_f32.to_bits()).unwrap();
    let biased = other.add(mean, bias).unwrap();
    let root = other.rsqrt(biased).unwrap();
    let seventh = serial_reduction_builder(squared_sum_with_epilogue(other.build(root).unwrap()))
        .build()
        .unwrap();
    assert_ne!(
        scaled.canonical_identity().as_bytes(),
        seventh.canonical_identity().as_bytes(),
    );
}

/// No parallel topology may split a fold that carries an epilogue.
///
/// **The refusal is the family's algebra rather than caution.** The epilogue
/// applies to the *complete* fold, so a partial pass applying it would
/// transform a fragment and one that did not would be computing
/// [`ScalarProgram::SquaredSerialSum`] under this variant's name. Both split
/// admissions therefore answer `None` for it, and the topology is refused at
/// the same rule an unadmitted family is.
#[test]
fn a_fold_carrying_an_epilogue_admits_no_parallel_topology() {
    let scalar = squared_sum_with_epilogue(scale_epilogue());
    let family = split_family(&scalar).expect("the serial family is derived");
    assert_eq!(family.parallel, ParallelFamily::SerialOnly);
    assert!(
        family
            .read_tensor(FamilyTopology::MultiPass(ReductionPass::Partial))
            .is_none()
    );
    assert!(
        family
            .read_tensor(FamilyTopology::MultiPass(ReductionPass::Final))
            .is_none()
    );
    assert!(family.read_tensor(FamilyTopology::Cooperative).is_none());

    // Stated against a partial pass that is otherwise *correct*: the fixture
    // is the squaring fold's own verified partial pass with its scalar
    // program exchanged for the epilogue-carrying one, so the family is the
    // only difference between an admitted region and this refusal.
    let mut split = squared_partial_pass_builder(SPLIT);
    set_scalar(&mut split, squared_sum_with_epilogue(scale_epilogue()));
    assert_eq!(
        split.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "a fold whose epilogue applies to the whole value has no partial pass",
    );
}

/// Both passes of a split verify, and neither needs a barrier to do so.
///
/// The partial pass runs one invocation per (output, partition) pair and the
/// final pass one per output; the values move between them through the
/// materialized partial tensor alone, which is what makes the split a
/// dispatch-boundary strategy rather than a workgroup one.
#[test]
fn both_passes_of_a_split_reduction_verify() {
    let partial = partial_pass_builder(SPLIT).build().unwrap();
    let combine = final_pass_builder(SPLIT).build().unwrap();
    assert_eq!(partial.region().schedule.work_items, 6);
    assert_eq!(combine.region().schedule.work_items, 2);
    assert_eq!(partial.requirements().local_memory_bytes, 0);
    assert_eq!(combine.requirements().local_memory_bytes, 0);
    // The split reports the freedom it consumes and only that freedom.
    assert_eq!(
        float_rows(&partial.requirements()).reassociation,
        NumericalPermission::Permitted
    );
    assert_eq!(
        float_rows(&partial.requirements()).permutation,
        NumericalPermission::Forbidden
    );
}

/// Restates the complete serial fixture over one input shape and axis list.
///
/// Every construction and consumption site moves together so the resulting
/// region isolates whether serial empty-domain admission newly requires a
/// contributor count. The base serial arms compared these facts structurally
/// but did not canonicalize or multiply the axes of an identity-seeded fold.
fn restate_serial_reduction_domain(
    builder: &mut ScheduledRegionBuilder,
    input: Shape,
    axes: Vec<Axis>,
) {
    let output = input.without_axes(&axes);
    let output_elements = element_count(&output).expect("the retained fixture shape fits u64");
    builder.iteration_shape = Some(output.clone());

    let LogicalAccess::ReductionContributor {
        input_shape,
        output_shape,
        axes: access_axes,
        ..
    } = &mut builder.accesses[0].map
    else {
        panic!("the serial fixture has a contributor access")
    };
    *input_shape = input.clone();
    *output_shape = output.clone();
    *access_axes = axes.clone();

    let BoundsProofKind::ReductionDomain {
        input_shape,
        output_shape,
        axes: proof_axes,
        ..
    } = &mut builder.bounds_proofs[0].kind
    else {
        panic!("the serial fixture has a contributor proof")
    };
    *input_shape = input;
    *output_shape = output;
    *proof_axes = axes.clone();
    builder.bounds_proofs[1].kind = BoundsProofKind::LinearRange {
        element_count: output_elements,
    };
    builder.ownership_proof.as_mut().unwrap().kind =
        OwnershipProofKind::OneGlobalInvocationPerOutput {
            output_count: output_elements,
        };

    let Some(RegionProgram::Numerical { scalar, .. }) = builder.program.as_mut() else {
        panic!("the serial fixture has an arithmetic program")
    };
    match scalar {
        ScalarProgram::StrictSerialSum {
            axes: scalar_axes, ..
        }
        | ScalarProgram::FusedMultiplyAddSerialSum {
            axes: scalar_axes, ..
        }
        | ScalarProgram::SquaredSerialSum {
            axes: scalar_axes, ..
        }
        | ScalarProgram::SquaredSerialSumThenEpilogue {
            axes: scalar_axes, ..
        }
        | ScalarProgram::StrictSerialMaximum {
            axes: scalar_axes, ..
        } => *scalar_axes = axes.clone(),
        ScalarProgram::PointwiseF32(_)
        | ScalarProgram::PointwiseBf16(_)
        | ScalarProgram::StrictAffineU4Dequantize { .. }
        | ScalarProgram::StrictTensorContraction { .. } => {
            panic!("the fixture has a serial fold program")
        }
    }
    let schedule = builder
        .schedule
        .as_mut()
        .expect("the serial fixture has a schedule");
    let ReductionTopology::Serial {
        axes: scheduled_axes,
        ..
    } = &mut schedule.reduction
    else {
        panic!("the serial fixture has a serial topology")
    };
    *scheduled_axes = axes;
    schedule.work_items = output_elements;
    schedule.launch.grid_threads = output_elements;
}

/// Identity-seeded serial folds preserve the exact base admission boundary:
/// empty-domain verification validates their identity without counting the
/// contributors.
///
/// Duplicate and out-of-range axes and an overflowing reduced-extent product
/// are not endorsed as a new contract here; they are deliberately pinned as
/// admitted because this private refactor may not narrow the pre-existing
/// serial set. Maximum is the adjacent control: its missing identity makes a
/// successful contributor count load-bearing, so the same duplicate axes are
/// refused. A wrong sum identity is the other control and stays refused even
/// though no count is derived.
#[test]
fn identity_seeded_serial_folds_do_not_require_a_contributor_count() {
    let duplicate_axes = vec![Axis::new(1), Axis::new(1)];
    for (name, scalar) in serial_fold_families()
        .into_iter()
        .filter(|(_, scalar)| !matches!(scalar, ScalarProgram::StrictSerialMaximum { .. }))
    {
        let mut builder = serial_reduction_builder(scalar);
        restate_serial_reduction_domain(
            &mut builder,
            Shape::from_dims([2, 6]),
            duplicate_axes.clone(),
        );
        builder
            .build()
            .unwrap_or_else(|error| panic!("{name} narrowed on duplicate axes: {error:?}"));
    }

    for (name, input, axes) in [
        (
            "out-of-range axis",
            Shape::from_dims([2, 6]),
            vec![Axis::new(2)],
        ),
        (
            "overflowing contributor product",
            Shape::from_dims([u64::MAX, 2]),
            vec![Axis::new(0), Axis::new(1)],
        ),
    ] {
        let mut builder = serial_reduction_builder(bare_sum(vec![Axis::new(1)]));
        restate_serial_reduction_domain(&mut builder, input, axes);
        builder
            .build()
            .unwrap_or_else(|error| panic!("serial sum narrowed on {name}: {error:?}"));
    }

    let mut maximum = serial_reduction_builder(maximum_scalar());
    restate_serial_reduction_domain(&mut maximum, Shape::from_dims([2, 6]), duplicate_axes);
    assert_eq!(
        maximum.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "an identity-less fold still owes a countable, non-empty domain",
    );

    let mut wrong_identity = bare_sum(vec![Axis::new(1)]);
    let ScalarProgram::StrictSerialSum {
        empty_identity_bits,
        ..
    } = &mut wrong_identity
    else {
        unreachable!()
    };
    *empty_identity_bits = (-0.0_f32).to_bits();
    assert_eq!(
        serial_reduction_builder(wrong_identity)
            .build()
            .unwrap_err()
            .diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "skipping the count does not skip identity validation",
    );
}

/// A bare serial sum folds an input access or a materialized domain.
///
/// **The widening, and its exact width.** `ScalarProgram::StrictSerialSum`
/// carries no prologue, so it says how contributors combine and nothing about
/// where they live: `sum(x)` over any declared input tensor and the same fold
/// over a prologue region's materialized result are one scalar program over
/// several possible boundary tensors. What the widening is *not* is "any tensor" —
/// a program output remains refused because no fold reads one as a
/// contributor domain.
#[test]
fn a_bare_serial_sum_folds_a_declared_input_or_a_materialized_domain() {
    assert!(
        serial_reduction_builder(bare_sum(vec![Axis::new(1)]))
            .build()
            .is_ok(),
        "a fold over the first declared input has no prologue region to read",
    );

    let mut materialized = serial_reduction_builder(bare_sum(vec![Axis::new(1)]));
    read_from(&mut materialized, TensorRole::Intermediate);
    assert!(
        materialized.build().is_ok(),
        "the prologue-carrying plan still folds the intermediate it staged",
    );

    let input = serial_reduction_builder(bare_sum(vec![Axis::new(1)]))
        .build()
        .unwrap();
    assert_eq!(input.region().index.accesses[0].tensor, TensorRole::Input);

    let mut output = serial_reduction_builder(bare_sum(vec![Axis::new(1)]));
    read_from(&mut output, TensorRole::Output);
    assert_eq!(
        output.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "a fold cannot read its contributor domain from a program output",
    );
}

/// A bare fold still proves its contributor access against its own reduction.
///
/// The widening moved which *tensor* the read may bind and nothing else. The
/// declared reduction and the access relation still have to state the same
/// reduced axes, so a region folding a declared input over one axis while
/// addressing another is refused exactly as the intermediate-reading one always
/// was.
///
/// The fold's own declaration moves here rather than the access's, because the
/// bounds proof refines the *access*: perturbing the access alone is caught one
/// authority earlier and would report the proof reference instead of the
/// disagreement under test.
#[test]
fn a_bare_fold_over_an_input_still_proves_its_contributor_access() {
    let mut mismatched = serial_reduction_builder(bare_sum(vec![Axis::new(0)]));
    let Some(ReductionTopology::Serial { axes, .. }) = mismatched
        .schedule
        .as_mut()
        .map(|schedule| &mut schedule.reduction)
    else {
        panic!("the fixture schedules a serial reduction");
    };
    *axes = vec![Axis::new(0)];
    assert_eq!(
        mismatched.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "a fold declaring one reduced axis while addressing another is not that fold",
    );
}

/// Rebinds a region's owning write to another boundary tensor.
///
/// The write access, its bounds proof, and the ownership proof move together
/// because [`verify_proof_records`] requires all three to name one tensor:
/// moving fewer would report the proof reference and prove nothing about the
/// boundary role under test. The write is the last access by the same
/// convention [`verify_intrinsic`] destructures it under.
fn write_to(builder: &mut ScheduledRegionBuilder, tensor: TensorRole) {
    let write = builder.accesses.len() - 1;
    builder.accesses[write].tensor = tensor;
    builder.bounds_proofs[write].tensor = tensor;
    builder.ownership_proof.as_mut().unwrap().tensor = tensor;
}

/// Every serial fold family, over the shared serial fixture's reduced axis.
///
/// Named and returned as a population rather than asserted one family at a
/// time, so the test below counts what it covered: a write rule that reached
/// three of these five would otherwise pass a spot check and leave the rest
/// silently narrower.
fn serial_fold_families() -> Vec<(&'static str, ScalarProgram)> {
    let axes = vec![Axis::new(1)];
    vec![
        ("strict serial sum", bare_sum(axes.clone())),
        ("extrema fold", maximum_scalar()),
        (
            "squaring prologue",
            ScalarProgram::SquaredSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
        ),
        (
            "squaring prologue with an epilogue",
            squared_sum_with_epilogue(scale_epilogue()),
        ),
        (
            "scale-bias prologue",
            ScalarProgram::FusedMultiplyAddSerialSum {
                scale_bits: 1.0_f32.to_bits(),
                bias_bits: 0.0_f32.to_bits(),
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
                contraction: false,
            },
        ),
    ]
}

/// Every serial fold may commit its result to a materialized intermediate.
///
/// **The widening, and its exact width.** Where a fold's result goes is a
/// property of the surrounding cover and not of the fold: `sum(x * x)` whose
/// value the caller asked for and the same fold whose value an epilogue
/// scales are one computation committing to two boundary tensors. All five
/// families widen together because none of their algebras distinguishes the
/// two — admitting only the bare sum would say a squaring prologue's result
/// is inherently the program's answer, which is false, and would leave the
/// *fused* alternative unspellable for every reduction an epilogue consumes
/// while the materialized-prologue alternative compiled.
///
/// What the widening is *not* is "any tensor": a write to a declared input
/// stays refused, because a region committing there would mutate a tensor the
/// caller owns whatever it folded to get there.
#[test]
fn every_serial_fold_family_may_commit_to_a_materialized_intermediate() {
    let families = serial_fold_families();
    assert_eq!(
        families.len(),
        5,
        "the serial match has five fold arms, and each must be driven",
    );
    for (name, scalar) in families {
        assert!(
            serial_reduction_builder(scalar.clone()).build().is_ok(),
            "the output-writing control for the {name} must verify, \
             or neither case below is evidence",
        );

        let mut staged = serial_reduction_builder(scalar.clone());
        write_to(&mut staged, TensorRole::Intermediate);
        assert!(
            staged.build().is_ok(),
            "the {name} has a producer region for the value an epilogue reads",
        );

        let mut into_input = serial_reduction_builder(scalar);
        write_to(&mut into_input, TensorRole::Input);
        assert_eq!(
            into_input.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "no fold commits its result into a tensor the caller owns ({name})",
        );
    }
}

/// Committing to an intermediate is a distinct region, not a free relabel.
///
/// The write role reaches `encode_identity` through the access list, the
/// write's bounds proof, and the ownership proof, so a staged fold and a
/// published one are different canonical regions. Without this, a plan that
/// materialized a fold's result and one that published it could share a
/// cache entry and one would be served for the other.
#[test]
fn the_committed_tensor_separates_scheduled_region_identity() {
    let published = serial_reduction_builder(bare_sum(vec![Axis::new(1)]))
        .build()
        .unwrap();
    let mut builder = serial_reduction_builder(bare_sum(vec![Axis::new(1)]));
    write_to(&mut builder, TensorRole::Intermediate);
    let staged = builder.build().unwrap();
    assert_ne!(
        published.canonical_identity().as_bytes(),
        staged.canonical_identity().as_bytes(),
    );
}

/// A split's committing pass chooses its write tensor; its staging pass does not.
///
/// **The asymmetry is the assertion, and it is the write counterpart of the
/// read asymmetry [`only_the_partial_pass_of_a_split_may_fold_a_declared_input`]
/// pins.** The final pass commits the reduction's own result, so the cover
/// decides where it lands exactly as it does for the serial fold this split
/// replaces — which is what keeps a split alternative available for a
/// reduction whose result an epilogue consumes. The partial pass commits an
/// unfolded fragment, which is no cover's output;
/// [`a_partial_pass_may_not_write_the_program_output`] is the narrow pin for
/// that half and is unchanged by this widening.
#[test]
fn only_the_committing_pass_of_a_split_chooses_its_write_tensor() {
    assert!(
        final_pass_builder(SPLIT).build().is_ok(),
        "the output-writing control must verify, or neither case below is evidence",
    );

    let mut staged = final_pass_builder(SPLIT);
    write_to(&mut staged, TensorRole::Intermediate);
    assert!(
        staged.build().is_ok(),
        "a split fold whose result an epilogue reads stages it from its final pass",
    );

    let mut into_input = final_pass_builder(SPLIT);
    write_to(&mut into_input, TensorRole::Input);
    assert_eq!(
        into_input.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "no pass commits its result into a tensor the caller owns",
    );

    let mut partial = partial_pass_builder(SPLIT);
    write_to(&mut partial, TensorRole::Output);
    assert_eq!(
        partial.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "a partial is an unfolded fragment and is no cover's declared output",
    );
}

/// A split's partial pass may fold a declared input; its final pass may not.
///
/// **The asymmetry is the assertion.** The partial pass folds the region's
/// declared contributor domain, which lives wherever the plan put it. The final
/// pass folds values the partial pass *staged*, and those exist only because it
/// staged them — so a final pass claiming a declared input holds them describes
/// a handoff no dispatch performed.
#[test]
fn only_the_partial_pass_of_a_split_may_fold_a_declared_input() {
    assert!(
        partial_pass_builder(SPLIT).build().is_ok(),
        "the intermediate-reading control must verify, or neither case below is evidence",
    );
    assert!(final_pass_builder(SPLIT).build().is_ok());

    let mut partial = partial_pass_builder(SPLIT);
    read_from(&mut partial, TensorRole::Input);
    assert!(
        partial.build().is_ok(),
        "a prologue-less fold's partial pass retains the declared input it folds",
    );

    let mut combine = final_pass_builder(SPLIT);
    read_from(&mut combine, TensorRole::Input);
    assert_eq!(
        combine.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "no declared input holds partials a dispatch staged",
    );
}

/// A cooperative tile may fold a declared input, and only a declared one.
///
/// The tile stages its partials in workgroup memory rather than in a boundary
/// tensor, so its single read is the declared contributor domain whatever the
/// plan staged — which is why it carries no pass distinction where the
/// multi-pass split has one.
#[test]
fn a_cooperative_tile_may_fold_a_declared_input() {
    assert!(
        cooperative_builder(cooperative_tile_fixture())
            .build()
            .is_ok(),
        "the intermediate-reading control must verify, or neither case below is evidence",
    );

    let mut input = cooperative_builder(cooperative_tile_fixture());
    read_from(&mut input, TensorRole::Input);
    assert!(input.build().is_ok());
}

/// A fused affine fold reads an input access in every supported topology.
#[test]
fn an_affine_fold_reads_any_declared_input_in_serial_and_parallel_forms() {
    let affine = ScalarProgram::FusedMultiplyAddSerialSum {
        scale_bits: 2.0_f32.to_bits(),
        bias_bits: 1.0_f32.to_bits(),
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0.0_f32.to_bits(),
        contraction: false,
    };

    let mut serial = serial_reduction_builder(affine.clone());
    read_from(&mut serial, TensorRole::Input);
    serial
        .build()
        .expect("the serial affine fold reads input one");

    let mut partial = partial_pass_builder(SPLIT);
    set_scalar(&mut partial, affine.clone());
    read_from(&mut partial, TensorRole::Input);
    partial
        .build()
        .expect("the affine partial pass reads input one");

    let mut cooperative = cooperative_builder(cooperative_tile_fixture());
    set_scalar(&mut cooperative, affine);
    read_from(&mut cooperative, TensorRole::Input);
    cooperative
        .build()
        .expect("the affine cooperative tile reads input one");
}

/// Parallel affine folds read a declared input, never an intermediate.
///
/// The bare sum's parallel forms admit an intermediate because it may hold a
/// materialized prologue. The affine family carries that prologue inside its
/// scalar program, so admitting the intermediate would apply the affine body
/// to a value that was already transformed or to an unbound staging edge.
#[test]
fn affine_parallel_folds_reject_an_intermediate_contributor() {
    let affine = ScalarProgram::FusedMultiplyAddSerialSum {
        scale_bits: 2.0_f32.to_bits(),
        bias_bits: 1.0_f32.to_bits(),
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0.0_f32.to_bits(),
        contraction: false,
    };

    let mut partial = partial_pass_builder(SPLIT);
    set_scalar(&mut partial, affine.clone());
    assert_eq!(
        partial.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "an affine partial pass cannot read a materialized contributor",
    );

    let mut cooperative = cooperative_builder(cooperative_tile_fixture());
    set_scalar(&mut cooperative, affine);
    assert_eq!(
        cooperative.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "an affine cooperative tile cannot read a materialized contributor",
    );
}

/// Family-specific input-role rules remain exact across serial families.
#[test]
fn squared_and_maximum_folds_require_an_input_access() {
    for scalar in [
        ScalarProgram::SquaredSerialSum {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        },
        squared_sum_with_epilogue(scale_epilogue()),
        maximum_scalar(),
    ] {
        let region = serial_reduction_builder(scalar.clone());
        assert!(region.build().is_ok());
        let mut intermediate = serial_reduction_builder(scalar);
        read_from(&mut intermediate, TensorRole::Intermediate);
        assert_eq!(
            intermediate.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement,]
        );
    }
}

/// The squared parallel family keeps its own input-role rule.
///
/// This is separate from the serial control above because the split and
/// cooperative family tables are independent match arms. Widening either to
/// every declared input would otherwise leave the serial check green.
#[test]
fn squared_parallel_folds_require_an_input_access() {
    let mut partial = squared_partial_pass_builder(SPLIT);
    read_from(&mut partial, TensorRole::Intermediate);
    assert_eq!(
        partial.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "a squared partial pass cannot read an intermediate",
    );

    let squared = ScalarProgram::SquaredSerialSum {
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0.0_f32.to_bits(),
    };
    let mut cooperative = cooperative_builder(cooperative_tile_fixture());
    set_scalar(&mut cooperative, squared);
    read_from(&mut cooperative, TensorRole::Intermediate);
    assert_eq!(
        cooperative.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "a squared cooperative tile cannot read an intermediate",
    );
}

/// The maximum parallel family keeps its own input-role rule.
///
/// Maximum has independent split and cooperative family-table arms, so the
/// serial maximum control does not prove that either parallel obligation
/// still refuses a later declared input.
#[test]
fn maximum_parallel_folds_require_an_input_access() {
    let mut partial = extrema_partial_builder(SPLIT);
    read_from(&mut partial, TensorRole::Intermediate);
    assert_eq!(
        partial.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "a maximum partial pass cannot read an intermediate",
    );

    let mut cooperative = extrema_cooperative_builder();
    read_from(&mut cooperative, TensorRole::Intermediate);
    assert_eq!(
        cooperative.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "a maximum cooperative tile cannot read an intermediate",
    );
}

/// A cooperative tile may commit its result to a materialized intermediate.
///
/// A tile is both halves of a split in one dispatch, so its single write is
/// the fold's committing write and carries the same cover-assigned obligation
/// the serial fold and the split's final pass carry. It has no staging pass
/// whose target the split structure fixes, because it stages in workgroup
/// memory rather than in a boundary tensor — which is why it needs no pass
/// distinction here, exactly as it needs none for its read.
#[test]
fn a_cooperative_tile_may_commit_to_a_materialized_intermediate() {
    assert!(
        cooperative_builder(cooperative_tile_fixture())
            .build()
            .is_ok(),
        "the output-writing control must verify, or neither case below is evidence",
    );

    let mut staged = cooperative_builder(cooperative_tile_fixture());
    write_to(&mut staged, TensorRole::Intermediate);
    assert!(
        staged.build().is_ok(),
        "a tiled fold whose result an epilogue reads stages it from its commit",
    );

    let mut into_input = cooperative_builder(cooperative_tile_fixture());
    write_to(&mut into_input, TensorRole::Input);
    assert_eq!(
        into_input.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "no tile commits its result into a tensor the caller owns",
    );
}

/// A split has an identity distinct from the serial reduction it replaces.
#[test]
fn a_split_pass_is_not_identical_to_a_serial_pass() {
    let split = partial_pass_builder(SPLIT).build().unwrap();
    let mut serial = partial_pass_builder(SPLIT);
    // The same region under the same contract, differing only in whether
    // its contributor sequence is split.
    serial.schedule.as_mut().unwrap().reduction = ReductionTopology::Serial {
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
        permits_reassociation: true,
        permits_permutation: false,
    };
    // The serial reading of that region is itself rejected, and the rule it
    // names is the bounds proof: a serial reduction's iteration domain *is*
    // its reduction domain, so a proof over `[2]` no longer refines an
    // access whose region iterates `[2, 3]`. The two topologies are
    // therefore not interchangeable even before identity is compared.
    assert_eq!(
        serial.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::BoundsProof]
    );
    let final_pass = final_pass_builder(SPLIT).build().unwrap();
    assert_ne!(
        split.canonical_identity().as_bytes(),
        final_pass.canonical_identity().as_bytes()
    );
}

/// Reassociation is what a split consumes, and denying it rejects the split.
#[test]
fn a_split_is_rejected_when_reassociation_is_denied() {
    for mut builder in [partial_pass_builder(SPLIT), final_pass_builder(SPLIT)] {
        set_numerical(&mut builder, strict_numerical());
        let ReductionTopology::MultiPass {
            permits_reassociation,
            ..
        } = &mut builder.schedule.as_mut().unwrap().reduction
        else {
            panic!("expected a split topology")
        };
        *permits_reassociation = false;
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );
    }
}

/// Permutation is a separate permission the split neither needs nor uses.
///
/// Both directions are driven, because checking one permission and
/// consuming the other is invisible when only the permitted case is tested:
/// a contract that permits permutation but forbids reassociation must still
/// reject the split, and one that forbids permutation but permits
/// reassociation must still admit it.
#[test]
fn permutation_neither_admits_nor_blocks_a_split() {
    let mut permuting_only = partial_pass_builder(SPLIT);
    set_numerical(
        &mut permuting_only,
        NumericalRealization {
            permutation: NumericalPermission::Permitted,
            ..strict_numerical()
        },
    );
    let ReductionTopology::MultiPass {
        permits_reassociation,
        permits_permutation,
        ..
    } = &mut permuting_only.schedule.as_mut().unwrap().reduction
    else {
        panic!("expected a split topology")
    };
    *permits_reassociation = false;
    *permits_permutation = true;
    assert_eq!(
        permuting_only.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "permutation must not stand in for the reassociation a split consumes"
    );

    // The complementary direction: the default fixture already forbids
    // permutation and permits reassociation, and it verifies.
    assert!(partial_pass_builder(SPLIT).build().is_ok());
}

/// An accumulation narrower than the element width is rejected, not accepted.
///
/// **Refused under its own name**, which criterion 3 of
/// `implement-parallel-reduction-strategies` requires: the diagnostic names
/// the accumulator and carries both widths, so a producer can tell this from
/// the wrong axis set or the wrong contributor order that
/// [`ScheduledRegionDiagnostic::NumericalOrAccessRefinement`] also reports.
/// A wider declaration is refused by the same rule, and it is driven here
/// because "narrower" is the criterion's wording and not the check's.
#[test]
fn a_narrowed_accumulation_width_is_rejected() {
    for wrong in [
        ArithmeticType::F16,
        ArithmeticType::Bf16,
        ArithmeticType::F64,
    ] {
        let mut builder = partial_pass_builder(SPLIT);
        let ReductionTopology::MultiPass { accumulation, .. } =
            &mut builder.schedule.as_mut().unwrap().reduction
        else {
            panic!("expected a split topology")
        };
        *accumulation = wrong;
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::AccumulationWidth {
                declared: wrong,
                required: ArithmeticType::F32,
            }],
            "{wrong:?} is not the width this region computes in"
        );
    }
    // The control: the same builder at the declared width verifies, so the
    // refusals above are about the accumulator and not about the fixture.
    assert!(partial_pass_builder(SPLIT).build().is_ok());
}

/// The final pass must combine exactly one contributor per partition.
#[test]
fn a_final_pass_reading_the_wrong_partition_count_is_rejected() {
    let mut builder = final_pass_builder(SPLIT);
    // A partial tensor with a fourth partition the split never produced.
    let LogicalAccess::ReductionContributor { input_shape, .. } = &mut builder.accesses[0].map
    else {
        panic!("expected a reduction access")
    };
    *input_shape = Shape::from_dims([2, 4]);
    let BoundsProofKind::ReductionDomain { input_shape, .. } = &mut builder.bounds_proofs[0].kind
    else {
        panic!("expected a reduction proof")
    };
    *input_shape = Shape::from_dims([2, 4]);
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );
}

/// A partial pass that writes the program output is not a partial pass.
#[test]
fn a_partial_pass_may_not_write_the_program_output() {
    let mut builder = partial_pass_builder(SPLIT);
    builder.accesses[1].tensor = TensorRole::Output;
    builder.bounds_proofs[1].tensor = TensorRole::Output;
    builder.ownership_proof.as_mut().unwrap().tensor = TensorRole::Output;
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );
}

/// Every field of a split separates canonical scheduled-region identity.
#[test]
fn every_split_field_separates_scheduled_region_identity() {
    let baseline = partial_pass_builder(SPLIT)
        .build()
        .unwrap()
        .region()
        .clone();
    let mut seen = vec![encode_identity(&baseline)];
    for reduction in [
        ReductionTopology::MultiPass {
            pass: ReductionPass::Final,
            coverage: ContributorCoverage::Exact(SPLIT),
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            accumulation: ArithmeticType::F32,
            permits_reassociation: true,
            permits_permutation: false,
        },
        ReductionTopology::MultiPass {
            pass: ReductionPass::Partial,
            coverage: ContributorCoverage::Exact(ContributorPartition {
                partitions: 2,
                contributors_per_partition: 3,
            }),
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            accumulation: ArithmeticType::F32,
            permits_reassociation: true,
            permits_permutation: false,
        },
        ReductionTopology::MultiPass {
            pass: ReductionPass::Partial,
            coverage: ContributorCoverage::Exact(SPLIT),
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            accumulation: ArithmeticType::F64,
            permits_reassociation: true,
            permits_permutation: false,
        },
        ReductionTopology::MultiPass {
            pass: ReductionPass::Partial,
            coverage: ContributorCoverage::Exact(SPLIT),
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            accumulation: ArithmeticType::F32,
            permits_reassociation: true,
            permits_permutation: true,
        },
        ReductionTopology::MultiPass {
            pass: ReductionPass::Partial,
            coverage: ContributorCoverage::IdentityPadded {
                partition: PADDED_SPLIT,
                identity: NEG_ZERO,
            },
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            accumulation: ArithmeticType::F32,
            permits_reassociation: true,
            permits_permutation: false,
        },
        ReductionTopology::Serial {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            permits_reassociation: true,
            permits_permutation: false,
        },
    ] {
        let mut candidate = baseline.clone();
        candidate.schedule.reduction = reduction.clone();
        let identity = encode_identity(&candidate);
        assert!(
            !seen.contains(&identity),
            "{reduction:?} collided with an earlier topology"
        );
        seen.push(identity);
    }
}

/// The fixture region under one chosen arrival and permutation resolution.
///
/// Both are varied together because the rule under test is exactly their
/// composition: the topology records what the contract resolved, and the
/// verifier requires the two to agree before it asks what the arrival
/// consumes.
fn arriving_builder(
    arrival: ContributorArrival,
    permutation_permitted: bool,
) -> ScheduledRegionBuilder {
    let numerical = NumericalRealization {
        permutation: if permutation_permitted {
            NumericalPermission::Permitted
        } else {
            NumericalPermission::Forbidden
        },
        ..reassociating_numerical()
    };
    let ReductionTopology::CooperativeWorkgroup {
        coverage,
        tile,
        axes,
        order,
        accumulation,
        permits_reassociation,
        ..
    } = cooperative_topology_arriving(cooperative_tile_fixture(), SPLIT, arrival)
    else {
        panic!("the cooperative fixture builds a cooperative topology")
    };
    cooperative_builder_parts(
        SPLIT,
        6,
        ReductionTopology::CooperativeWorkgroup {
            coverage,
            tile,
            axes,
            order,
            accumulation,
            permits_reassociation,
            permits_permutation: permutation_permitted,
            arrival,
        },
        numerical,
    )
}

/// The tile's accumulator is refused under the same name the split's is.
///
/// **The second of the two sites, driven separately.**
/// `verify_accumulation_width` is the single authority both parallel gates
/// reach, so a test on the split alone would pass while the tile's own call
/// was deleted. This asserts the tile refuses, with the same diagnostic and
/// the same payload, on a topology whose other fields are untouched.
///
/// The tile's control is `one_cooperative_tile_verifies_and_derives_its_workgroup_storage`
/// below, which builds this exact fixture unperturbed.
#[test]
fn a_cooperative_tile_declaring_the_wrong_accumulation_width_is_rejected() {
    for wrong in [
        ArithmeticType::F16,
        ArithmeticType::Bf16,
        ArithmeticType::F64,
    ] {
        let mut builder = cooperative_builder(cooperative_tile_fixture());
        let ReductionTopology::CooperativeWorkgroup { accumulation, .. } =
            &mut builder.schedule.as_mut().unwrap().reduction
        else {
            panic!("expected a cooperative topology")
        };
        *accumulation = wrong;
        assert_eq!(
            cooperative_rejection(builder),
            ScheduledRegionDiagnostic::AccumulationWidth {
                declared: wrong,
                required: ArithmeticType::F32,
            },
            "{wrong:?} is not the width this tile's region computes in"
        );
    }
}

/// The two permissions stay independent, and the arrival is what separates
/// them.
///
/// The admitted arrival is fixed by the program, so it consumes
/// reassociation alone and a contract forbidding permutation admits it. An
/// arrival the program does not fix consumes permutation *as well*, and the
/// two refusals are distinct: withholding the permission names the
/// permission, and granting it still names the construct nothing realizes.
#[test]
fn an_unfixed_arrival_order_consumes_permutation_and_is_refused_by_name() {
    // The control: the same fixture with the admitted arrival verifies under
    // a contract that forbids permutation, so neither refusal below is
    // something the fixture would have earned anyway.
    assert!(
        arriving_builder(ContributorArrival::AscendingParticipant, false)
            .build()
            .is_ok()
    );
    // And granting permutation neither breaks nor is required by it: the
    // recorded permission simply tracks the contract.
    assert!(
        arriving_builder(ContributorArrival::AscendingParticipant, true)
            .build()
            .is_ok()
    );
    for arrival in [
        ContributorArrival::NondeterministicArrival,
        ContributorArrival::AtomicAccumulation,
    ] {
        assert!(arrival.requires_permutation());
        assert_eq!(
            cooperative_rejection(arriving_builder(arrival, false)),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::ArrivalPermission,
            },
            "{} was admitted under a contract that forbids permutation",
            arrival.key()
        );
        // Granting permutation moves the refusal to the construct, which is
        // the check that would be dead if the two were collapsed.
        assert_eq!(
            cooperative_rejection(arriving_builder(arrival, true)),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::UnadmittedArrival,
            },
            "{} was admitted as a realizable construct",
            arrival.key()
        );
    }
}

/// A round count of zero, or beyond the governed bound, is refused.
#[test]
fn a_round_count_outside_the_governed_profile_is_refused() {
    for rounds in [0, MAX_COOPERATIVE_ROUNDS.saturating_add(1)] {
        assert_eq!(
            cooperative_rejection(round_perturbed(|tile| tile.rounds = rounds)),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::RoundStructure,
            },
            "round count {rounds}"
        );
    }
}

/// A zero-extent input keeps the reducer's identity and stages nothing.
///
/// The empty result is `+0.0`, which every arm of this verifier requires as
/// `empty_identity_bits`, and the serial topology commits it from one
/// invocation with no fold — so the empty case needs no staging, no phase,
/// and no visibility edge. A cooperative tile over the same domain is
/// refused rather than made to stage values no participant produces.
#[test]
fn a_zero_extent_reduction_keeps_its_identity_without_a_tile() {
    let mut serial = ScheduledRegionBuilder::new(RegionId::new(5));
    serial.iteration_shape(Shape::from_dims([2])).unwrap();
    serial
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: Shape::from_dims([2, 0]),
                output_shape: Shape::from_dims([2]),
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    serial
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    serial
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: Shape::from_dims([2, 0]),
                output_shape: Shape::from_dims([2]),
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    serial
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 2 },
        })
        .unwrap();
    serial
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
        })
        .unwrap();
    serial
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictSerialSum {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: strict_numerical(),
        })
        .unwrap();
    serial
        .schedule(KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(2, OwnershipWitnessId::new(0))
        })
        .unwrap();
    let empty = serial
        .clone()
        .build()
        .expect("the empty reduction verifies");
    assert_eq!(empty.requirements().local_memory_bytes, 0);
    let RegionProgram::Numerical {
        scalar:
            ScalarProgram::StrictSerialSum {
                empty_identity_bits,
                ..
            },
        ..
    } = &empty.region().index.program
    else {
        panic!("expected a strict serial sum")
    };
    assert_eq!(*empty_identity_bits, 0.0_f32.to_bits());

    // The same empty domain declared cooperative, with every launch,
    // ownership, and proof fact left exactly as the well-formed fixture
    // states them: nothing to stage, so the tile is refused instead of
    // describing a handoff of values that do not exist.
    let mut cooperative = cooperative_builder(cooperative_tile_fixture());
    let empty_contributors = LogicalAccess::ReductionContributor {
        input_shape: Shape::from_dims([2, 0]),
        output_shape: Shape::from_dims([2]),
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
    };
    cooperative.accesses[0].map = empty_contributors;
    cooperative.bounds_proofs[0].kind = BoundsProofKind::ReductionDomain {
        input_shape: Shape::from_dims([2, 0]),
        output_shape: Shape::from_dims([2]),
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
    };
    assert_eq!(
        cooperative_rejection(cooperative),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::EmptyContributorDomain,
        }
    );
}

#[test]
fn reduction_contributor_count_handles_late_zero_extents() {
    let access = LogicalAccess::ReductionContributor {
        input_shape: Shape::from_dims([u64::MAX, 2, 0]),
        output_shape: Shape::from_dims([]),
        axes: vec![Axis::new(0), Axis::new(1), Axis::new(2)],
        order: ContributorOrder::OriginalAxisLexicographic,
    };
    assert_eq!(
        crate::schedule::contributor_count(&[Axis::new(0), Axis::new(1), Axis::new(2)], &access),
        Ok(0)
    );
}
