//! Where the whole-program recognition boundary was, and what moved it.
//!
//! `select_supported_strategy` was a three-way template match. It tried a
//! scale-bias-then-strict-serial `Sum` over exactly one declared input, a
//! well-formed binary contraction, and a two-operation pointwise expression over
//! three leaves — and when all three refused, the program was refused before any
//! target-qualified explain trace existed. A program *composing* two of those
//! families was not a shape any of them spelled, so it was refused for a
//! property of the templates rather than for anything the compiler could not do.
//!
//! What landed is a recognizer that classifies the occurrence producing the
//! output and then walks outward through the occurrences feeding it. The
//! elementwise dimension is now the general `PointwiseF32Expression` vocabulary:
//! any depth, any number of declared inputs, mixed families, and shared reads.
//! This file is the evidence that a composed program reaches an emitted region
//! through the ordinary `tiler_compiler::session` entry point, and that the
//! programs still outside the boundary refuse under a rule that names what was
//! not recognized.
//!
//! # What is asserted, and what would make each assertion vacuous
//!
//! Every compilation here resolves its per-target outcome, so a pass is a
//! complete verified plan rather than successful strategy selection. Every
//! positive case travels with a one-input control the superseded template *did*
//! spell: without a program that compiles under the identical request, a pass
//! would be consistent with a permissive target rather than with anything about
//! recognition. Every refusal travels with the accepted
//! neighbour it differs from in exactly one occurrence, so the rule that fires is
//! attributable to that occurrence and not to the fixture.
//!
//! # The one contract that declines both mixed bodies
//!
//! `RelaxedF32` permits arithmetic contraction, and a body holding a multiply
//! adjacent to an add has no feasible plan under it — a decline the sibling
//! `multi_input_elementwise_boundary` file owns, pins, and records as eliminated
//! rather than deferred. The composed program and the scale-bias control are
//! *both* such bodies, so both are asserted to refuse under it and for the same
//! class. Asserting them together is the point: the decline reads the adjacency
//! and not the composition.
//!
//! # Where the boundary was: the two structural families
//!
//! Recognition generalized over the *expression* vocabulary, and the families
//! that compute nothing did not come with it: `tiler::reindex-f32@1` and
//! `tiler::broadcast-f32@1` each carried registered semantics, a registered
//! index-access lowering capability, and a `CoordinateRelation` fusion role, but
//! `tiler_ir::schedule::LogicalAccess` spelled no reindex map and no widening
//! broadcast, so a region containing either could not be written down and both
//! refused under `operation-set`. Two assertions here pinned those walls.
//!
//! `admit-the-structural-families-into-the-scheduled-region-vocabulary` landed
//! `LogicalAccess::ReindexBijection` and `LogicalAccess::BroadcastReplication`
//! and both assertions are flipped: each structural program now compiles, and
//! each still travels with the elementary neighbour it differs from in one
//! occurrence. **The pairs are kept rather than retired, and their purpose
//! inverts.** They no longer attribute a refusal; they show that a family
//! contributing *addressing* and a family contributing *arithmetic* reach a
//! region by different routes, which is what makes the two refusals this file
//! still carries — a structural operand the region would have to materialize,
//! and two outputs one walk would publish — attributable to their own rules
//! rather than to the structural family being present at all.
//!
//! # What is deliberately not asserted here
//!
//! Bit-level agreement with the reference evaluator is the in-crate conformance
//! work's, because it needs the crate-private lowering registry;
//! `pipeline::tests::a_broadcast_reaches_a_kernel_matching_the_reference_evaluator`
//! and its reindex siblings are where the programs below are compared against
//! the oracle. Which *plan* wins is `selection`'s. What this file does record
//! about plans is a vocabulary fact rather than a preference: a general prologue
//! has no fused single-region alternative at all, because
//! `ScalarProgram::FusedMultiplyAddSerialSum` applies one scale and one bias per
//! contributor and cannot spell `(a * b) + c`.

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileFailure, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::semantic::{
    BroadcastAxisMapping, BroadcastAxisSource, F32, F32Add, F32Broadcast, F32Constant, F32Multiply,
    F32Reindex, F32Silu, InputKey, OutputKey, ReindexForm, SemanticProgram, SemanticProgramBuilder,
    StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Extent, Shape};

/// Every numerical contract a caller can state.
///
/// Stated exhaustively rather than sampled, for the reason the sibling boundary
/// files state it: the outcome of *recognition* is structural, so a contract
/// that changed it would mean the boundary moved for a reason this file does not
/// model.
const CONTRACTS: [NumericalContract; 5] = [
    NumericalContract::STRICT_F32,
    NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    NumericalContract::RELAXED_F32,
    NumericalContract::REASSOCIATE_F32,
    NumericalContract::FLUSH_AND_REASSOCIATE_F32,
];

/// The one contract that permits arithmetic contraction.
///
/// A mixed multiply/add body is declined under it, which the sibling
/// `multi_input_elementwise_boundary` file pins and owns. Named here so the one
/// case below that skips it does so for that stated reason rather than silently.
const CONTRACTION_PERMITTED: NumericalContract = NumericalContract::RELAXED_F32;

/// The contributor domain of every fixture below.
///
/// Four elements in one row, and every extent is load-bearing rather than
/// arbitrary. The governed baseline profile declares a four-thread grid axis, so
/// the prologue's launch — one invocation per contributor — must be at most
/// four; and the fold's four contributors are what admit a balanced exact split,
/// which keeps every parallel strategy *offered* rather than declined. A
/// narrower domain would make each positive case below pass for a reason this
/// file does not model.
fn domain() -> Shape {
    Shape::from_dims([1, 4])
}

/// `sum((a * b) + c, axis 1)`: two admitted families in one program.
///
/// **This is the composed program.** Its prologue is the multi-input elementwise
/// body `admit-multi-input-elementwise-programs-at-the-compiler-boundary`
/// landed, and its fold is the strict serial reduction the serial-sum family
/// owns. No superseded normalization matched it: the serial-sum template
/// demanded exactly one declared input and the exact `x * scale + bias`
/// prologue, and the pointwise template refused any program containing a
/// reduction.
fn composed_region() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input::<F32>(InputKey::new(key).unwrap(), domain())
                .unwrap()
        })
        .collect();
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
    let biased = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, biased, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// `sum(x * 2.0 + 1.0, axis 1)`: the one shape the superseded template spelled.
///
/// It is the exact scale-bias prologue over one declared input that
/// `normalize_serial_sum` matched — the *only* program shape the superseded
/// template spelled — so it is the program every generalization below is
/// measured against.
///
/// It carries a multiply adjacent to an add, which the contraction-permitting
/// contract declines; the sibling `multi_input_elementwise_boundary` file pins
/// that decline for one-input mixed bodies, and [`homogeneous_control`] is the
/// neighbour that travels under that contract instead.
fn scale_bias_control() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain())
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let shifted = F32Add::apply(&mut builder, scaled, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, shifted, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// `sum((x * 2.0) * 3.0, axis 1)`: the neighbour every contract admits.
///
/// One declared input and a *homogeneous* prologue, so it carries no
/// multiply/add adjacency for the contraction-permitting contract to decline —
/// which is what [`scale_bias_control`] does carry. It is the control that
/// travels with every case below, because a neighbour that itself refuses under
/// one contract would leave that contract's assertions standing alone.
fn homogeneous_control() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain())
        .unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let three = F32Constant::apply(&mut builder, 3.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, two).unwrap();
    let root = F32Multiply::apply(&mut builder, scaled, three).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, root, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// `sum(a, axis 1)`: a fold whose contributor tensor is a declared input.
///
/// The simplest reduction there is, and the one program in this file whose wall
/// was genuinely *below* the request boundary rather than in it: `tiler-ir`'s
/// schedule verifier required a `ScalarProgram::StrictSerialSum` region's
/// contributor access to read `TensorRole::Intermediate`, so no region this profile
/// could build read the input directly. Synthesizing an identity prologue to
/// satisfy it was never the alternative — that would add a materialization, and its
/// observable rounding boundary, that the caller's program never asked for — so the
/// widening moved the verifier arm instead, and this program now compiles.
fn prologue_less_fold() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain())
        .unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// [`composed_region`] with the multiply replaced by the activation.
///
/// `sum(silu(a) + c, axis 1)`: three occurrences, two declared inputs, the same
/// domain, the same fold, the same output. It is the composed program's
/// *elementary* neighbour, and it compiles — the activation's per-point body is
/// projected into the expression vocabulary by the one authority that states it,
/// so a registered family with no node of its own still reaches a region.
///
/// The shape is deliberately preserved from [`composed_region`]: the activation
/// is elementwise, so nothing but the operation set differs between the two.
fn composed_region_with_an_activation() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain())
        .unwrap();
    let c = builder
        .input::<F32>(InputKey::new("c").unwrap(), domain())
        .unwrap();
    let activated = F32Silu::apply(&mut builder, a).unwrap();
    let biased = F32Add::apply(&mut builder, activated, c).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, biased, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// [`composed_region_with_an_activation`] with the activation made structural.
///
/// `sum(reverse(a) + c, axis 1)`: the activation becomes `tiler::reindex-f32@1`
/// over an axis reversal, which preserves the shape, so the two programs differ
/// in exactly one occurrence's operation and in nothing else.
///
/// **The pair is what makes each program's route attributable.** Both perturbed
/// occurrences are registered unary families with registered lowering
/// capabilities, and both now compile — but by different routes: the
/// activation's per-point body is *projected* into the expression vocabulary,
/// while the reindex contributes a read map and no body at all.
fn composed_region_with_a_structural_occurrence() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain())
        .unwrap();
    let c = builder
        .input::<F32>(InputKey::new("c").unwrap(), domain())
        .unwrap();
    let reversed = F32Reindex::apply(
        &mut builder,
        &ReindexForm::reverse_axis(Axis::new(1)).expect("an axis reversal is an admitted form"),
        a,
    )
    .unwrap();
    let biased = F32Add::apply(&mut builder, reversed, c).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, biased, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// [`composed_region_with_an_unspellable_occurrence`] with the reversal moved
/// behind the activation.
///
/// `sum(reverse(silu(a)) + c, axis 1)`: the same admitted reversal over a value
/// the program *computes* rather than declares. The region binds one read per
/// declared input, so there is no access for an intermediate the same region
/// would also produce — and materializing one would add an observable rounding
/// boundary the caller never asked for. It refuses under `structural-operand`.
fn reversal_of_a_computed_value() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain())
        .unwrap();
    let c = builder
        .input::<F32>(InputKey::new("c").unwrap(), domain())
        .unwrap();
    let activated = F32Silu::apply(&mut builder, a).unwrap();
    let reversed = F32Reindex::apply(
        &mut builder,
        &ReindexForm::reverse_axis(Axis::new(1)).expect("an axis reversal is an admitted form"),
        activated,
    )
    .unwrap();
    let biased = F32Add::apply(&mut builder, reversed, c).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, biased, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// The domain of the three weight-multiply fixtures below.
///
/// Separate from [`domain`] because a broadcast has to actually widen: a
/// `Replicate` axis of extent one is refused by `BroadcastAxisMapping` itself
/// with `RelationDoesNotWiden`, so the single-row domain the fold fixtures use
/// cannot express the occurrence at all. Four elements in two rows keeps the
/// launch inside the same governed four-thread grid axis [`domain`] is sized
/// against, so the trio below differs from the rest of this file in the shape of
/// its iteration space and in nothing that decides admission.
fn widening_domain() -> Shape {
    Shape::from_dims([2, 2])
}

/// `a * w`: two declared tensors at one shape, and the control for the trio.
///
/// The plainest program that reads two declared inputs in the position the two
/// fixtures below elaborate. Without it, a refusal there would be consistent
/// with the widened domain or the two-input shape being unrecognized rather than
/// with anything about the occurrence between them.
fn weighted_by_a_declared_tensor() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let activations = builder
        .input::<F32>(InputKey::new("a").unwrap(), widening_domain())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("w").unwrap(), widening_domain())
        .unwrap();
    let scaled = F32Multiply::apply(&mut builder, activations, weight).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), scaled)
        .unwrap();
    builder.build().unwrap()
}

/// `a * silu(w)`: a registered unary family feeding the multiply.
///
/// The accepted neighbour of [`weighted_by_a_broadcast`]. Both interpose one
/// registered unary family with a registered index-access lowering capability
/// between a declared weight and the same multiply, so the two differ in that
/// occurrence's operation — and, because widening is what a broadcast *is*, in
/// the weight's declared shape. Nothing else about them differs.
fn weighted_by_an_activation() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let activations = builder
        .input::<F32>(InputKey::new("a").unwrap(), widening_domain())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("w").unwrap(), widening_domain())
        .unwrap();
    let activated = F32Silu::apply(&mut builder, weight).unwrap();
    let scaled = F32Multiply::apply(&mut builder, activations, activated).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), scaled)
        .unwrap();
    builder.build().unwrap()
}

/// `a * broadcast(w)`: the workload's most frequent structural occurrence.
///
/// The weight is declared at the widened axis alone and read at every row, which
/// is the `[1024]`-against-`[T, 1024]` shape of the RMS-normalization weight
/// multiply — 113 of the pinned workload's 197 broadcast occurrences. It is the
/// program `reach-a-verified-kernel-through-the-structural-families` names, and
/// it compiles: `LogicalAccess::BroadcastReplication` reads the rank-one operand
/// across the widened axis, where the older `ScalarBroadcast` could only read a
/// rank-zero operand once.
///
/// This file asserts that it is *recognized*; its result is bit-compared against
/// the reference evaluator by
/// `pipeline::tests::a_broadcast_reaches_a_kernel_matching_the_reference_evaluator`,
/// which is the same program at the same extents.
fn weighted_by_a_broadcast() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let activations = builder
        .input::<F32>(InputKey::new("a").unwrap(), widening_domain())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("w").unwrap(), Shape::from_dims([2]))
        .unwrap();
    let mapping = BroadcastAxisMapping::new(
        [Extent::new(2), Extent::new(2)],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(Axis::new(0)),
        ],
    )
    .unwrap();
    let widened = F32Broadcast::apply(&mut builder, &mapping, weight).unwrap();
    let scaled = F32Multiply::apply(&mut builder, activations, widened).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), scaled)
        .unwrap();
    builder.build().unwrap()
}

/// Compiles one program under one contract against the governed profile.
///
/// Returns only after the per-target outcome resolves, so `Ok` is a complete
/// verified plan. A strategy-admission refusal is additionally required to
/// precede any target-qualified trace, which is what makes "refused at the
/// boundary" distinguishable from "refused after planning".
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
        Err(failure) => {
            assert!(
                failure.explain().is_none(),
                "a strategy-admission refusal precedes any target-qualified trace",
            );
            Err(failure.class())
        }
    }
}

/// The composed program compiles through the ordinary entry point.
///
/// This is the ticket's closing condition: a semantic program no normalization
/// matched — one composing the multi-input elementwise family with the
/// serial-sum family — reaches an emitted region rather than being refused at
/// the boundary.
#[test]
fn a_program_composing_two_admitted_families_compiles_through_the_session() {
    let composed = composed_region();
    assert_eq!(composed.input_count(), 3);
    assert_eq!(composed.operation_count(), 3);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&homogeneous_control(), contract),
            Ok(()),
            "{contract:?} refused the one-input control, so nothing this test \
             asserts about the composed program would be evidence about \
             recognition",
        );
        // Both mixed multiply/add bodies skip the contraction-permitting
        // contract, and they skip it *together*, which is what makes the skip a
        // statement about the adjacency rather than about the composition: the
        // shape the superseded template spelled and the shape it could not are
        // declined identically under it. The sibling
        // `multi_input_elementwise_boundary` file owns and pins that decline.
        if contract == CONTRACTION_PERMITTED {
            assert_eq!(
                compile_under(&scale_bias_control(), contract),
                Err(CompileFailureClass::NoFeasiblePlan),
            );
            assert_eq!(
                compile_under(&composed, contract),
                Err(CompileFailureClass::NoFeasiblePlan),
            );
            continue;
        }
        assert_eq!(
            compile_under(&scale_bias_control(), contract),
            Ok(()),
            "{contract:?} refused the one shape the superseded template spelled",
        );
        assert_eq!(
            compile_under(&composed, contract),
            Ok(()),
            "{contract:?} refused the composed program",
        );
    }
}

/// A fold over a declared input compiles, under every contract.
///
/// **The row this file's second boundary occupied, flipped.** Recognition
/// generalized over the expression vocabulary and stopped where the physical layer
/// did: `tiler-ir`'s schedule verifier required a `ScalarProgram::StrictSerialSum`
/// region's contributor access to read `TensorRole::Intermediate`, so `sum(a)` was
/// refused *at* the boundary under `reduction-prologue` rather than admitted and
/// failed mid-pipeline. `admit-a-reduction-over-a-declared-input-tensor` widened
/// that arm to the fold's *declared contributor domain* — the first input tensor
/// when the program folds it directly, the materialized intermediate when a
/// prologue region wrote it — and the rule no longer exists.
///
/// **The pair is kept and its purpose inverts**, exactly as this file's structural
/// rows did. [`homogeneous_control`] is the same fold over the same declared input
/// with an elementwise prologue between them, and it travels here so the admission
/// below reads as a statement about the missing prologue rather than about a
/// permissive target: the neighbour compiles through a prologue region and the fold
/// that reads what it staged, and this program compiles through one region binding
/// the input buffer directly, with no materialization at all.
///
/// Every contract, because the outcome is structural — a contract that changed it
/// would mean the widening landed somewhere this file does not model. Neither body
/// puts a multiply adjacent to an add, so neither joins the
/// contraction-permitting decline the mixed bodies above take.
#[test]
fn a_reduction_over_a_declared_input_compiles_through_the_session() {
    let fold = prologue_less_fold();
    assert_eq!(fold.input_count(), 1);
    assert_eq!(fold.operation_count(), 1);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&homogeneous_control(), contract),
            Ok(()),
            "{contract:?} refused the prologue-carrying neighbour, so the \
             admission below would not be evidence about the missing prologue",
        );
        assert_eq!(
            compile_under(&fold, contract),
            Ok(()),
            "{contract:?} refused a fold whose contributor tensor is its declared input",
        );
    }
}

/// A structural occurrence beside an elementary one compiles as a mapped read.
///
/// The perturbed program differs from [`composed_region_with_an_activation`] in
/// exactly one occurrence. It used to refuse under `operation-set` because
/// `LogicalAccess` spelled no reindex map; with
/// `LogicalAccess::ReindexBijection` landed it compiles, and this is the
/// assertion that records the flip.
///
/// **Both halves run under every contract, and the pair is still the
/// assertion.** Two registered unary families with registered index-access
/// lowering capabilities sit in the same position of the same program, and each
/// reaches a region by its own route — one by projecting a per-point body into
/// the expression vocabulary, one by contributing a coordinate map and no body.
/// Keeping the accepted half is what makes the structural half's *contract*
/// behaviour readable: they resolve differently under the contraction-permitting
/// contract, and only a pair shows why.
#[test]
fn a_structural_occurrence_beside_an_elementary_one_compiles_as_a_mapped_read() {
    let accepted = composed_region_with_an_activation();
    let perturbed = composed_region_with_a_structural_occurrence();
    assert_eq!(accepted.operation_count(), 3);
    assert_eq!(perturbed.operation_count(), 3);

    for contract in CONTRACTS {
        // The activation's body carries a multiply and an add inside the same
        // region as the fold, so it joins the two mixed bodies this file already
        // declines under the contraction-permitting contract — and it declines
        // as `NoFeasiblePlan`, *after* recognition admitted it, which is itself
        // the evidence that recognition admitted it. The refusal below is a
        // recognition refusal under every contract, so the pair still reads the
        // missing vocabulary on this row rather than losing its accepted half.
        assert_eq!(
            compile_under(&accepted, contract),
            if contract == CONTRACTION_PERMITTED {
                Err(CompileFailureClass::NoFeasiblePlan)
            } else {
                Ok(())
            },
            "{contract:?} did not resolve the elementary neighbour as expected, so \
             the refusal below would not be evidence about the missing vocabulary",
        );
        // **The row this ticket flipped.** The reindex no longer refuses: its
        // axis reversal is the read map of the region its neighbour's
        // arithmetic already fills.
        //
        // It compiles under *every* contract, including the
        // contraction-permitting one the activation declines — and the
        // difference is the point rather than an inconsistency. The activation
        // declines because `silu`'s projected body puts a multiply adjacent to
        // an add inside the fold's region; a reindex projects no body at all, so
        // `reverse(a) + c` carries one add and no adjacency for that contract to
        // decline. The two families differ in exactly what this file says they
        // differ in: one contributes arithmetic and the other contributes
        // addressing.
        assert_eq!(
            compile_under(&perturbed, contract),
            Ok(()),
            "{contract:?} did not admit the structural occurrence as a mapped read",
        );
    }
}

/// A structural occurrence over a *computed* value still refuses by name.
///
/// The neighbour that keeps the admission above attributable. Both programs
/// reverse an axis with the same admitted form; they differ only in whether the
/// operand is a declared input, and only the declared one has a read for the
/// region to bind. Without this row, the flip above would be consistent with the
/// boundary admitting every reindex — including one whose operand the region
/// would have to materialize, adding the observable rounding boundary the
/// family's admission exists to avoid.
#[test]
fn a_structural_occurrence_over_a_computed_value_refuses_by_name() {
    let over_computed = reversal_of_a_computed_value();
    assert_eq!(over_computed.operation_count(), 4);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&over_computed, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "structural-operand"
            }),
            "{contract:?} admitted a structural occurrence with no declared operand to read",
        );
    }
}

/// The broadcast that widens a declared weight compiles as a replication
/// relation.
///
/// **This is the workload's dominant structural occurrence, and until the
/// vocabulary landed it was the wall in front of it.** `tiler::broadcast-f32@1`
/// used to reach the request boundary only through `fusion_legality`'s in-crate
/// derivation, which proves a region containing it is *legal* to fuse and says
/// nothing about whether one can be spelled; the trio below was added to observe
/// its refusal, and now records its admission.
///
/// **The trio is still the assertion.** `a * w` compiles, so neither the widened
/// domain nor the two declared inputs is what decides the widened row. `a *
/// silu(w)` compiles, so a registered unary family with a registered
/// index-access lowering capability in that exact position is admitted. `a *
/// broadcast(w)` compiles under every contract, and it tracks the *plain
/// control* rather than the activation — which is the readable difference: a
/// broadcast introduces no arithmetic, so it carries no multiply/add adjacency
/// for the contraction-permitting contract to decline.
///
/// The activation declines under that contract for the reason the module header
/// states — its body carries the multiply/add adjacency — and it declines as
/// `NoFeasiblePlan`, *after* recognition admitted it, which is itself the
/// evidence that recognition admitted it. The plain control carries no such
/// adjacency and compiles under all five.
#[test]
fn a_broadcast_widening_a_declared_weight_compiles_as_a_replication_relation() {
    let control = weighted_by_a_declared_tensor();
    let accepted = weighted_by_an_activation();
    let widened = weighted_by_a_broadcast();
    assert_eq!((control.input_count(), control.operation_count()), (2, 1));
    assert_eq!((accepted.input_count(), accepted.operation_count()), (2, 2));
    assert_eq!((widened.input_count(), widened.operation_count()), (2, 2));

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&control, contract),
            Ok(()),
            "{contract:?} refused two declared tensors at the widened domain, so \
             nothing below would be evidence about the broadcast occurrence",
        );
        assert_eq!(
            compile_under(&accepted, contract),
            if contract == CONTRACTION_PERMITTED {
                Err(CompileFailureClass::NoFeasiblePlan)
            } else {
                Ok(())
            },
            "{contract:?} did not resolve the elementary neighbour as expected, so \
             the refusal below would not be evidence about the missing relation",
        );
        // **The row this ticket flipped, and the workload's dominant structural
        // occurrence.** The `[2]` weight is read across the `[2, 2]` domain by a
        // replication relation rather than refused, and it tracks the control
        // rather than the activation: a broadcast introduces no arithmetic at
        // all, so it carries no multiply/add adjacency for the
        // contraction-permitting contract to decline.
        assert_eq!(
            compile_under(&widened, contract),
            Ok(()),
            "{contract:?} did not admit the widening as a replication relation",
        );
    }
}

/// A second named output *inside the first's walk* refuses under its own rule.
///
/// The accepted neighbour is [`composed_region`] itself: the same three
/// occurrences over the same three inputs, with one further value named as an
/// output. What refuses is the sharing rather than the second output — the
/// declared arity guard is gone and independent ordered outputs compile, which
/// `pipeline::conformance` discharges. Here `biased` is consumed by the fold
/// that produces `out`, so the two outputs' recognition walks claim one
/// occurrence twice: whichever region owns that write would have to serve both
/// the materialization edge the fold reads across and the publication, and a
/// region writes one owning tensor.
#[test]
fn a_second_named_output_inside_the_first_s_walk_refuses() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input::<F32>(InputKey::new(key).unwrap(), domain())
                .unwrap()
        })
        .collect();
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
    let biased = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, biased, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder
        .output(OutputKey::new("biased").unwrap(), biased)
        .unwrap();
    let two_outputs = builder.build().unwrap();
    assert_eq!(two_outputs.output_count(), 2);

    for contract in CONTRACTS {
        assert_eq!(
            compile_under(&two_outputs, contract),
            Err(CompileFailureClass::UnsupportedCapability {
                rule: "output-partition-overlap"
            }),
            "{contract:?} admitted two outputs one walk would have to publish",
        );
    }
}
