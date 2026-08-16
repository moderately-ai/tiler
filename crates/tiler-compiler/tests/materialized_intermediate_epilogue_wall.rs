//! Where the elementwise-epilogue wall actually is, measured rather than asserted.
//!
//! `admit-elementwise-epilogues-over-a-materialized-intermediate` was filed on a
//! premise this file refuted. Its "Why this exists" section stated the wall as
//! *the physical layer's, not the schedule IR's* — "`TensorRole::Intermediate`
//! is a per-region role, so nothing in `tiler-ir` forbids a chain that stages a
//! second temporary" — and `request.rs` carried the same claim in
//! [`select_supported_strategy`]'s own documentation. The claim is about the
//! role, and the role is indeed per-region; the conclusion drawn from it was
//! wrong, because what forbade the chain was not the role but the *access
//! contract* each scalar-program family declares around it.
//!
//! # The three walls, and which of them are open
//!
//! A chain `producer -> materialized intermediate -> elementwise epilogue` needs
//! a producer region that writes `TensorRole::Intermediate` and a pointwise
//! region that reads one. Three families are involved and they do not agree:
//!
//! | Region | Needs | Admitted by `tiler-ir` |
//! | --- | --- | --- |
//! | elementwise epilogue | read `TensorRole::Intermediate` | yes — [`a_pointwise_region_may_read_a_materialized_intermediate`] |
//! | serial-sum producer | write `TensorRole::Intermediate` | yes — [`a_strict_serial_sum_region_may_write_a_materialized_intermediate`] |
//! | contraction producer | write `TensorRole::Intermediate` | yes — [`a_contraction_region_can_already_write_a_materialized_intermediate`] |
//!
//! **The first row was `no` when this file was written, and
//! `admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`
//! flipped it.** At that step `verify_pointwise_region` required read access `i`
//! to be an input carrying declared ordinal `i`, which conflated the access
//! *position* — the expression leaf it serves — with the *declared input* its
//! role named. The role is fieldless now: the compiler projects each exact
//! access through the retained checked request subject, while the schedule still
//! admits at most one materialized intermediate. The epilogue's region is
//! expressible and a region with two intermediate reads, which nothing could
//! attribute to materialization edges, remains refused.
//!
//! **The second row was `no` too, and
//! `admit-a-strict-serial-fold-that-writes-a-materialized-intermediate` flipped
//! it.** `verify_access_and_semantics` admitted a fold under a
//! `ReductionTopology::Serial` only when `write.tensor == TensorRole::Output`,
//! which stated as a family fact something no family decides: where a fold's
//! result goes is a property of the surrounding cover, not of the fold. The rule
//! is now `CommittedTensor::CoverAssigned` at every committing pass — the four
//! serial arms, the split's final pass, and the cooperative tile — while the
//! split's *partial* pass keeps `CommittedTensor::Exactly(Intermediate)`, because
//! a partial is an unfolded fragment and is no cover's declared output.
//!
//! **With all three rows open, every region the chain needs is expressible in
//! `tiler-ir`**, and
//! `admit-elementwise-epilogues-over-a-materialized-intermediate` built the
//! compiler side on top of them: the recognizer names the staged value a folding
//! family produces and walks the epilogue against it, `RegionWrite` is threaded
//! into every producing spelling that hard-coded `TensorRole::Output`, and the
//! ordinary cover search assembles the chain.
//!
//! **The compiler still cannot route around the write by binding differently.** A
//! region cannot label a read with fieldless `TensorRole::Input` and let program
//! assembly bind a temporary there: `tiler_ir::program::ValueRole::fills`
//! refuses a `Temporary` value for an `Input` buffer, and
//! `KernelProgramBuilder::push_stage` is where that bites. The widening above did
//! not touch `fills` and did not need to — an epilogue's read now says
//! `TensorRole::Intermediate`, which a `Temporary` already fills. That mechanism
//! is pinned by `multi_output_boundary.rs`'s
//! `a_published_output_value_cannot_fill_an_intermediate_buffer`, so it is cited
//! here rather than re-asserted.
//!
//! # What the caller sees today
//!
//! Both epilogue shapes the ticket names compile. Each travels with the bare
//! producer as a control under the identical request, so a green run is evidence
//! about the epilogue rather than about the profile or the target, and each
//! travels with a neighbour that still refuses — the same program one
//! materialization boundary deeper — so the admission is bounded rather than
//! open-ended.
//!
//! **The two assertions below measured `operation-set` refusals until that
//! ticket landed**, and what they measured was real: the elementwise walk
//! reached an operand produced by a family its expression vocabulary has no node
//! for, and stopped. It now *names* that operand instead, which is the whole
//! change — the value a cover materializes is exactly the value the walk could
//! not absorb.
//!
//! A mapped-only structural occurrence over that same producer is not an
//! admitted epilogue today. It contributes addressing rather than a per-point
//! body, so the first walk asks whether the contraction result is already a leaf
//! before any occurrence has discovered it as a materialization boundary;
//! [`a_structural_read_of_a_materialized_contraction_refuses_by_name`] pins the
//! resulting `structural-operand` refusal beside the bare producer.

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContractionAxisSource, ContributorOrder, ExceptionalValueAssumption, ExecutionBinding,
    IndexRegion, KernelSchedule, LaunchPlan, LogicalAccess, NumericalPermission,
    NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    PointwiseF32Expression, PointwiseF32ExpressionBuilder, ReductionTopology, RegionId,
    ScalarProgram, ScheduledRegion, ScheduledRegionBuilder, ScheduledRegionDiagnostic,
    SubnormalMode, TailPolicy, TensorRole,
};
use tiler_ir::semantic::{
    CANONICAL_F32_ARITHMETIC_NAN_BITS, ContractionIndex, ContractionIndexStructure, F32,
    F32Constant, F32Multiply, F32Reindex, F32TensorContraction, InputKey, OutputKey, ReindexForm,
    SemanticProgram, SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// The five named F32 contract points this boundary suite exercises.
///
/// Named together rather than sampled at one preset because recognition is
/// structural: a point that changed the outcome would mean the boundary moved
/// for a reason this file does not model. This is not the complete population of
/// caller-composable numerical contracts.
const CONTRACTS: [NumericalContract; 5] = [
    NumericalContract::STRICT_F32,
    NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    NumericalContract::RELAXED_F32,
    NumericalContract::REASSOCIATE_F32,
    NumericalContract::FLUSH_AND_REASSOCIATE_F32,
];

// ---------------------------------------------------------------------------
// What the caller sees: admitted arithmetic chains and a bounded structural refusal
// ---------------------------------------------------------------------------

/// The contraction structure `mk,nk->mn`, the one `contraction_direct_path` uses.
fn projection_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [
            [ContractionIndex::new(19), ContractionIndex::new(3)],
            [ContractionIndex::new(14), ContractionIndex::new(3)],
        ],
        [ContractionIndex::new(19), ContractionIndex::new(14)],
    )
    .unwrap()
}

/// `projected = contract(a, b)`, the accepted control for the contraction pair.
///
/// The extents keep the output at four elements, which is the governed baseline
/// profile's grid-axis bound — the same bound every fixture in
/// `contraction_direct_path.rs` respects.
fn bare_contraction() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &projection_structure(), a, b).unwrap();
    builder
        .output(OutputKey::new("projected").unwrap(), projected)
        .unwrap();
    builder.build().unwrap()
}

/// `scaled = contract(a, b) * 2.0` — the ticket's first named shape.
///
/// Differs from [`bare_contraction`] by exactly the epilogue: same inputs, same
/// structure, same contraction occurrence, one further multiply on the result.
fn contraction_with_epilogue() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &projection_structure(), a, b).unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, projected, two).unwrap();
    builder
        .output(OutputKey::new("scaled").unwrap(), scaled)
        .unwrap();
    builder.build().unwrap()
}

/// `reversed = reverse(contract(a, b))` — a mapped-only structural read of a
/// result the contraction would otherwise materialize.
///
/// Unlike [`contraction_with_epilogue`], the outer occurrence contributes no
/// per-point arithmetic that can discover the producer as a materialization
/// boundary. The structural recognizer therefore sees the contraction result
/// before it is a staged leaf and refuses it under `structural-operand`.
fn contraction_with_structural_epilogue() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &projection_structure(), a, b).unwrap();
    let reversed = F32Reindex::apply(
        &mut builder,
        &ReindexForm::reverse_axis(Axis::new(1)).expect("an axis reversal is admitted"),
        projected,
    )
    .unwrap();
    builder
        .output(OutputKey::new("reversed").unwrap(), reversed)
        .unwrap();
    builder.build().unwrap()
}

/// The contributor domain of the reduction pair: one row of four.
///
/// Four contributors is what the sibling `composed_family_recognition.rs` uses
/// and for its reason — the prologue's one-invocation-per-contributor launch
/// must stay inside the governed profile's four-thread grid axis.
fn reduction_domain() -> Shape {
    Shape::from_dims([1, 4])
}

/// `reduced = sum(x * x, axis 1)`, the accepted control for the reduction pair.
fn bare_reduction() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let x = builder
        .input::<F32>(InputKey::new("x").unwrap(), reduction_domain())
        .unwrap();
    let squared = F32Multiply::apply(&mut builder, x, x).unwrap();
    let reduced = StrictSerialF32Sum::apply(&mut builder, squared, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("reduced").unwrap(), reduced)
        .unwrap();
    builder.build().unwrap()
}

/// `scaled = sum(x * x, axis 1) * 2.0` — the ticket's second named shape.
///
/// Differs from [`bare_reduction`] by exactly the epilogue. The fold reduces
/// axis 1 of a rank-two domain rather than every axis, so the epilogue's own
/// domain is rank one and the refusal below is the walk's rather than
/// `recognize_pointwise`'s `elementwise-rank` guard.
fn reduction_with_epilogue() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let x = builder
        .input::<F32>(InputKey::new("x").unwrap(), reduction_domain())
        .unwrap();
    let squared = F32Multiply::apply(&mut builder, x, x).unwrap();
    let reduced = StrictSerialF32Sum::apply(&mut builder, squared, [Axis::new(1)]).unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, reduced, two).unwrap();
    builder
        .output(OutputKey::new("scaled").unwrap(), scaled)
        .unwrap();
    builder.build().unwrap()
}

/// Compiles one program under one contract against the governed profile.
fn compile_under(
    program: &SemanticProgram,
    contract: NumericalContract,
) -> Result<(), CompileFailureClass> {
    let targets = TargetRequest::new([TargetProfile::governed()]).unwrap();
    match compile(CompileRequest::new(program, contract, targets)) {
        Ok(batch) => {
            let outcome = batch.targets().next().expect("one requested profile");
            outcome
                .outcome()
                .map(|_| ())
                .map_err(TargetCompileFailure::class)
        }
        Err(failure) => Err(failure.class()),
    }
}

/// An elementwise epilogue over a contraction compiles, and so does its producer.
///
/// The walk reaches the contraction occurrence as an operand and no
/// `PointwiseF32Node` spells a sum over indices shared by two operands, so there
/// is no leaf to mint for it — which is why the value is *staged* rather than
/// absorbed. The bare contraction travels with it under the identical request,
/// so a green run is evidence about the epilogue rather than about the profile
/// or the target.
#[test]
fn an_elementwise_epilogue_over_a_contraction_compiles_as_a_chain() {
    let control = bare_contraction();
    let epilogue = contraction_with_epilogue();
    // One further occurrence than the control, and nothing else moved.
    assert_eq!(
        epilogue.operation_count(),
        control.operation_count() + 2,
        "the epilogue adds exactly the constant and the multiply",
    );

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&control, contract),
            Ok(()),
            "{contract:?} refused the bare contraction, so nothing this test \
             asserts about the epilogue would be evidence about the epilogue",
        );
        assert_eq!(
            compile_under(&epilogue, contract),
            Ok(()),
            "{contract:?} refused the contraction epilogue",
        );
        assert_eq!(
            compile_under(&nested_contraction_chain(), contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "reduction-contributor-materialization"
            }),
            "{contract:?} did not name the materialization boundary in the reduction contributor",
        );
    }
}

/// A direct structural read of a materialized contraction result refuses by
/// name, while the same contraction without that read compiles.
///
/// This is distinct from a structural occurrence over a computed per-point
/// value: the contraction is a family the compiler can materialize as a producer
/// region, but a mapped-only walk never discovers that boundary before the
/// structural recognizer asks whether its operand is already a leaf.
#[test]
fn a_structural_read_of_a_materialized_contraction_refuses_by_name() {
    let control = bare_contraction();
    let structural = contraction_with_structural_epilogue();
    assert_eq!(
        structural.operation_count(),
        control.operation_count() + 1,
        "the refused program adds exactly the reindex occurrence",
    );

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&control, contract),
            Ok(()),
            "{contract:?} refused the bare contraction, so the structural refusal below would not be attributable to the staged operand",
        );
        assert_eq!(
            compile_under(&structural, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "structural-operand"
            }),
            "{contract:?} did not refuse the mapped-only structural read by name",
        );
    }
}

/// An elementwise epilogue over a reduction compiles, and so does its producer.
#[test]
fn an_elementwise_epilogue_over_a_reduction_compiles_as_a_chain() {
    let control = bare_reduction();
    let epilogue = reduction_with_epilogue();
    assert_eq!(
        epilogue.operation_count(),
        control.operation_count() + 2,
        "the epilogue adds exactly the constant and the multiply",
    );

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&control, contract),
            Ok(()),
            "{contract:?} refused the bare reduction, so nothing this test \
             asserts about the epilogue would be evidence about the epilogue",
        );
        assert_eq!(
            compile_under(&epilogue, contract),
            Ok(()),
            "{contract:?} refused the reduction epilogue",
        );
        assert_eq!(
            compile_under(&nested_reduction_chain(), contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "reduction-contributor-materialization"
            }),
            "{contract:?} did not name the materialization boundary in the reduction contributor",
        );
    }
}

/// `refolded = sum(contract(a, b) * 2.0, axis 0)` — one missing producer carrier.
///
/// This is not the two-intermediate-read width wall. Recognition discovers one
/// materialized contraction producer and the elementwise continuation, but the
/// serial reduction contributor has no producer relation on which to retain
/// that chain. `ElementwiseRefusal::Folded` is therefore flattened to
/// `reduction-contributor-materialization`. The separate carrier decision owns
/// admitting it; this fixture keeps the current refusal bounded beside the
/// admitted contraction epilogue.
fn nested_contraction_chain() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &projection_structure(), a, b).unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, projected, two).unwrap();
    let refolded = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(0)]).unwrap();
    builder
        .output(OutputKey::new("refolded").unwrap(), refolded)
        .unwrap();
    builder.build().unwrap()
}

/// `refolded = sum(sum(x * x, axis 1) * 2.0, axis 0)` — the reduction pair's
/// too-deep neighbour, for the reason [`nested_contraction_chain`] states.
fn nested_reduction_chain() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let x = builder
        .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2, 4]))
        .unwrap();
    let squared = F32Multiply::apply(&mut builder, x, x).unwrap();
    let reduced = StrictSerialF32Sum::apply(&mut builder, squared, [Axis::new(1)]).unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, reduced, two).unwrap();
    let refolded = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(0)]).unwrap();
    builder
        .output(OutputKey::new("refolded").unwrap(), refolded)
        .unwrap();
    builder.build().unwrap()
}

// ---------------------------------------------------------------------------
// Where the wall is: the schedule vocabulary's own access contracts
// ---------------------------------------------------------------------------

/// The strict `f32` realization every hand-built region below declares.
///
/// The profile key is a literal rather than a minted
/// [`tiler_ir::schedule::F32NumericalContractKey`] because no obligation under
/// test reads it: the intrinsic verifier compares the *permissions* against each
/// region's declared topology and never parses the key. Every region below —
/// refused and admitted alike — carries this same value, so no outcome here can
/// be attributed to the numerical declaration.
fn strict_f32_realization() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-f32",
        CANONICAL_F32_ARITHMETIC_NAN_BITS,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    )
}

/// The one-invocation-per-element schedule every region below is launched with.
fn linear_schedule(work_items: u64, owner: OwnershipWitnessId) -> KernelSchedule {
    KernelSchedule {
        binding: ExecutionBinding::GlobalLinearInvocation,
        work_items,
        threads_per_workgroup: 1,
        tail: TailPolicy::Exact,
        output_owner: owner,
        reduction: ReductionTopology::None,
        launch: LaunchPlan {
            grid_threads: work_items,
            threads_per_workgroup: 1,
            zero_work_skips_dispatch: true,
        },
    }
}

/// The product of `count` leaves, times `2.0` when there is only one.
///
/// One leaf per read, because the pointwise access contract requires exactly as
/// many reads as the expression has input leaves. The constant keeps the
/// one-read expression a legal two-operand tree.
fn product_expression(count: usize) -> PointwiseF32Expression {
    let mut builder = PointwiseF32ExpressionBuilder::new();
    let leaves: Vec<_> = (0..count)
        .map(|ordinal| {
            builder
                .input(AccessOrdinal::new(u32::try_from(ordinal).unwrap()))
                .unwrap()
        })
        .collect();
    let mut leaves = leaves.into_iter();
    let mut root = leaves.next().expect("at least one read");
    if count == 1 {
        let two = builder.constant(2.0_f32.to_bits()).unwrap();
        root = builder.multiply(root, two).unwrap();
    }
    for leaf in leaves {
        root = builder.multiply(root, leaf).unwrap();
    }
    builder.build(root).unwrap()
}

/// An elementwise region over `shape`, reading whichever tensors are named.
///
/// The *only* thing that varies between the regions below is `reads`, and it
/// varies in the accesses, their bounds proofs, and nowhere else — so the
/// verifier's verdict is attributable to the reads' boundary roles alone.
fn elementwise_region(reads: &[TensorRole], elements: u64) -> ScheduledRegion {
    let write_witness = u32::try_from(reads.len()).unwrap();
    let mut accesses: Vec<Access> = reads
        .iter()
        .enumerate()
        .map(|(position, tensor)| Access {
            tensor: *tensor,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(u32::try_from(position).unwrap()),
            ownership: None,
        })
        .collect();
    let mut bounds_proofs: Vec<BoundsProof> = reads
        .iter()
        .enumerate()
        .map(|(position, tensor)| BoundsProof {
            id: BoundsWitnessId::new(u32::try_from(position).unwrap()),
            tensor: *tensor,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: elements,
            },
        })
        .collect();
    accesses.push(Access {
        tensor: TensorRole::Output,
        component_role: None,
        mode: AccessMode::Write,
        map: LogicalAccess::LinearIdentity,
        bounds: BoundsWitnessId::new(write_witness),
        ownership: Some(OwnershipWitnessId::new(0)),
    });
    bounds_proofs.push(BoundsProof {
        id: BoundsWitnessId::new(write_witness),
        tensor: TensorRole::Output,
        component_role: None,
        kind: BoundsProofKind::LinearRange {
            element_count: elements,
        },
    });
    ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
            iteration_shape: Shape::from_dims([elements]),
            accesses,
            bounds_proofs,
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: elements,
                },
            },
            scalar_program: ScalarProgram::PointwiseF32(product_expression(reads.len())),
            numerical: strict_f32_realization(),
        },
        schedule: linear_schedule(elements, OwnershipWitnessId::new(0)),
    }
}

/// Verifies one hand-built region through the shared intrinsic verifier.
fn verify(region: ScheduledRegion) -> Result<(), Vec<ScheduledRegionDiagnostic>> {
    ScheduledRegionBuilder::from_region(region)
        .build()
        .map(|_| ())
        .map_err(|error| error.diagnostics().to_vec())
}

/// **The falsification, inverted.** A pointwise region may read a materialized
/// intermediate, so the epilogue this ticket owns has a region to be built as.
///
/// This assertion measured the opposite when the file was written, and
/// `admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`
/// lifted it: at that step `verify_pointwise_region` required read access `i` to
/// be an input carrying declared ordinal `i`, conflating the access position
/// with the declared input the role named. The role is fieldless now, while the
/// checked compiler subject retains the association; the schedule verifier
/// admits at most one intermediate. The control is the identical region reading
/// an input at access zero, so the admission is not evidence that the verifier
/// stopped refusing things.
///
/// The two refusals travel with it because the widening had to keep them. A
/// second intermediate read is *ambiguous*, not merely unsupported —
/// `TensorRole::Intermediate` carries no ordinal, so nothing says which
/// materialization edge each read binds, which is why `CoverAssembly::from_plan`
/// refuses it a layer up under `cover-intermediate-read-attribution` — and a
/// region reading the program's own output is refused by name rather than under
/// a wildcard.
#[test]
fn a_pointwise_region_may_read_a_materialized_intermediate() {
    let control = elementwise_region(&[TensorRole::Input], 4);
    assert_eq!(
        verify(control),
        Ok(()),
        "the input-reading control must verify, or the admission below is not \
         about the read's boundary role",
    );

    let epilogue = elementwise_region(&[TensorRole::Intermediate], 4);
    assert_eq!(
        verify(epilogue),
        Ok(()),
        "a pointwise region reading a materialized intermediate must verify: \
         it is the consumer half of every epilogue chain",
    );

    // The mixed list, which is what the separation actually buys: one leaf reads
    // what an earlier region staged, the other reads a declared input whose
    // ordinal is not its access position.
    let mixed = elementwise_region(&[TensorRole::Intermediate, TensorRole::Input], 4);
    assert_eq!(
        verify(mixed),
        Ok(()),
        "an epilogue reading a staged value and the program's third input must \
         verify, or the ordinal is still being read as the access position",
    );
    assert_eq!(
        verify(elementwise_region(
            &[
                TensorRole::Input,
                TensorRole::Intermediate,
                TensorRole::Input,
            ],
            4,
        )),
        Ok(()),
        "fieldless input roles do not impose declared-input ordering on the local read list",
    );

    for (reads, why) in [
        (
            vec![TensorRole::Intermediate, TensorRole::Intermediate],
            "two intermediate reads have nothing to attribute them to two edges",
        ),
        (
            vec![TensorRole::Output],
            "a region does not consume the output it publishes",
        ),
    ] {
        let Err(diagnostics) = verify(elementwise_region(&reads, 4)) else {
            panic!("{reads:?} must be refused by the intrinsic verifier: {why}");
        };
        assert!(
            diagnostics.contains(&ScheduledRegionDiagnostic::NumericalOrAccessRefinement),
            "expected the access-refinement diagnostic for {reads:?}, got {diagnostics:?}",
        );
    }
}

/// A strict serial fold may write a materialized intermediate.
///
/// The second wall, and it stood until
/// `admit-a-strict-serial-fold-that-writes-a-materialized-intermediate` lifted
/// it. `verify_access_and_semantics` admitted a `ScalarProgram::StrictSerialSum`
/// under a `ReductionTopology::Serial` only when the owning write targeted
/// `TensorRole::Output`, so `sum(x * x) * scale` had no producer region for the
/// value its epilogue reads even after the pointwise read opened. Both roles
/// verify now, and the region proves its ownership and bounds identically under
/// each — only which boundary tensor receives the committed value moves.
///
/// The write into a declared input travels with them, because the widening's
/// width is the assertion: a fold commits to one of the two *internal* boundary
/// tensors, never to a tensor the caller owns.
#[test]
fn a_strict_serial_sum_region_may_write_a_materialized_intermediate() {
    let input_shape = Shape::from_dims([1, 4]);
    let axes = vec![Axis::new(1)];
    let output_shape = input_shape.without_axes(&axes);
    let outputs = 1_u64;

    let fold = |write: TensorRole| ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(1),
            iteration_shape: output_shape.clone(),
            accesses: vec![
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: input_shape.clone(),
                        output_shape: output_shape.clone(),
                        axes: axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(0),
                    ownership: None,
                },
                Access {
                    tensor: write,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(1),
                    ownership: Some(OwnershipWitnessId::new(0)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(0),
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: input_shape.clone(),
                        output_shape: output_shape.clone(),
                        axes: axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(1),
                    tensor: write,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: outputs,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: write,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: outputs,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_F32_ARITHMETIC_NAN_BITS,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: strict_f32_realization(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(outputs, OwnershipWitnessId::new(0))
        },
    };

    assert_eq!(
        verify(fold(TensorRole::Output)),
        Ok(()),
        "the output-writing control must verify, or the admission below is not \
         about the write's boundary role",
    );
    assert_eq!(
        verify(fold(TensorRole::Intermediate)),
        Ok(()),
        "a strict serial sum staging its result is the producer region \
         `sum(x * x) * scale` needs",
    );
    let diagnostics = verify(fold(TensorRole::Input))
        .expect_err("a fold committing into a declared input must be refused");
    assert!(
        diagnostics.contains(&ScheduledRegionDiagnostic::NumericalOrAccessRefinement),
        "expected the access-refinement diagnostic, got {diagnostics:?}",
    );
}

/// A contraction may already write a materialized intermediate, and that bounds
/// the widening.
///
/// The producer half of `contract(a, b) * 2.0` needs no `tiler-ir` change at
/// all: `verify_contraction` admits `TensorRole::Intermediate | TensorRole::Output`
/// for the owning write. `crate::physical::contraction_region` hard-codes
/// `TensorRole::Output`, which is a compiler-side choice this ticket can move
/// once the consumer half exists.
///
/// Recorded so the blocking ticket is scoped to the two walls that are real
/// rather than to three, and so a reader does not re-derive which of the
/// producer families already compose.
#[test]
fn a_contraction_region_can_already_write_a_materialized_intermediate() {
    let operand = Shape::from_dims([2, 2]);
    let output_shape = Shape::from_dims([2, 2]);
    let contracted_shape = Shape::from_dims([2]);
    let operand_elements = 4_u64;
    let outputs = 4_u64;

    let contraction = |write: TensorRole| ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
            iteration_shape: output_shape.clone(),
            accesses: vec![
                Access {
                    tensor: TensorRole::Input,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ContractionOperand {
                        operand_shape: operand.clone(),
                        output_shape: output_shape.clone(),
                        contracted_shape: contracted_shape.clone(),
                        sources: vec![
                            ContractionAxisSource::Output { position: 0 },
                            ContractionAxisSource::Contracted { position: 0 },
                        ],
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(0),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Input,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ContractionOperand {
                        operand_shape: operand.clone(),
                        output_shape: output_shape.clone(),
                        contracted_shape: contracted_shape.clone(),
                        sources: vec![
                            ContractionAxisSource::Output { position: 1 },
                            ContractionAxisSource::Contracted { position: 0 },
                        ],
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(1),
                    ownership: None,
                },
                Access {
                    tensor: write,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(2),
                    ownership: Some(OwnershipWitnessId::new(0)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(0),
                    tensor: TensorRole::Input,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: operand_elements,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(1),
                    tensor: TensorRole::Input,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: operand_elements,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(2),
                    tensor: write,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: outputs,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: write,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: outputs,
                },
            },
            scalar_program: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted_shape.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_F32_ARITHMETIC_NAN_BITS,
            },
            numerical: strict_f32_realization(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Contraction {
                contracted_shape: contracted_shape.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(outputs, OwnershipWitnessId::new(0))
        },
    };

    // Both roles verify, which is the whole point: the producer half of the
    // chain is already expressible and only the consumer half is not.
    assert_eq!(verify(contraction(TensorRole::Output)), Ok(()));
    assert_eq!(verify(contraction(TensorRole::Intermediate)), Ok(()));
}
