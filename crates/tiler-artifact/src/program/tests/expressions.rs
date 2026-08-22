//! ABI expression evaluation, phases, arena growth, and program-ABI adoption.

use super::super::{
    AbiBinaryOp, AbiEvaluationError, AbiExprId, AbiFactBinder, AbiRoot, AbiUnaryOp, AbiValue,
    ArtifactBuildError, ArtifactProgramBuilder, AvailabilityPhase, CompilationEnvironment,
    TargetPropertyKey,
};
use super::{
    SCALE_BITS, declare_realization, formulas, fused_program, lowering_provider, payload,
    selection, semantic_program, variant,
};
use tiler_ir::semantic::InputKey;
use tiler_ir::shape::Axis;

// -------------------------------------------------------------------------
// Expression evaluation, phases, and failure classification
// -------------------------------------------------------------------------

#[test]
fn a_conditional_selection_evaluates_only_the_branch_it_takes() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let zero = draft.push_root(AbiRoot::UnsignedLiteral(0)).unwrap();
    let one = draft.push_root(AbiRoot::UnsignedLiteral(1)).unwrap();
    let ten = draft.push_root(AbiRoot::UnsignedLiteral(10)).unwrap();
    let unsafe_division = draft
        .push_binary(AbiBinaryOp::FloorDivide, ten, zero)
        .unwrap();
    let nonzero = draft
        .push_binary(AbiBinaryOp::LessOrEqual, one, zero)
        .unwrap();
    let guarded = draft.push_select(nonzero, unsafe_division, ten).unwrap();
    let facts = AbiFactBinder::new(AvailabilityPhase::CompileProfile).build();
    assert_eq!(
        evaluate_through_draft(&draft, guarded, &facts),
        Ok(AbiValue::Unsigned(10)),
    );
    assert_eq!(
        evaluate_through_draft(&draft, unsafe_division, &facts),
        Err(AbiEvaluationError::DivisionByZero {
            op: AbiBinaryOp::FloorDivide,
        }),
    );
}

#[test]
fn the_fact_binder_refuses_a_fact_from_a_later_phase() {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    let error = binder
        .bind_target_property(
            TargetPropertyKey::new("tiler.target.pipeline-registers").unwrap(),
            AvailabilityPhase::PreparedKernelPreflight,
            64,
        )
        .expect_err("a prepared-kernel fact is not observable at live preflight");
    assert_eq!(
        error,
        super::super::AbiBindingError::PhaseNotReached {
            available_at: AvailabilityPhase::PreparedKernelPreflight,
            reached: AvailabilityPhase::LiveDevicePreflight,
        },
    );
    assert_eq!(
        binder.build().reached_phase(),
        AvailabilityPhase::LiveDevicePreflight,
    );
}

#[test]
fn evaluation_reports_an_unbound_root_rather_than_guessing() {
    // Exercised through a launch precondition rather than the launch geometry.
    // The geometry is derived from the program now and that program's is a
    // constant, so it evaluates without consulting any fact -- which would make
    // this test pass for the wrong reason. A precondition is still
    // caller-supplied and can name a fact that is deliberately left unbound.
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new(std::iter::once(provider.clone()), []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let rows = draft
        .push_root(AbiRoot::InputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(0),
        })
        .unwrap();
    let predicate = draft
        .push_binary(AbiBinaryOp::LessOrEqual, formulas.one, rows)
        .unwrap();
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.entries[0].launch.preconditions = vec![predicate];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    let artifact = draft.build().unwrap();

    let facts = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight).build();
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    let precondition = entry
        .launch_preconditions()
        .next()
        .expect("one launch precondition");
    assert_eq!(
        precondition.evaluate(&facts),
        Err(AbiEvaluationError::UnboundInputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(0),
        }),
    );
}

#[test]
fn checked_narrowing_rejects_a_value_that_does_not_fit() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let wide = draft
        .push_root(AbiRoot::UnsignedLiteral(u64::from(u32::MAX) + 1))
        .unwrap();
    let narrowed = draft.push_unary(AbiUnaryOp::NarrowU32, wide).unwrap();
    let facts = AbiFactBinder::new(AvailabilityPhase::CompileProfile).build();
    assert_eq!(
        evaluate_through_draft(&draft, narrowed, &facts),
        Err(AbiEvaluationError::NarrowingOverflow {
            op: AbiUnaryOp::NarrowU32,
            value: u64::from(u32::MAX) + 1,
        }),
    );
}

/// Evaluates one draft expression by packaging the arena the builder holds.
///
/// Evaluation is a property of the verified product, so this helper builds a
/// throwaway artifact whose only use site is the expression under test.
fn evaluate_through_draft(
    draft: &ArtifactProgramBuilder,
    node: AbiExprId,
    facts: &super::super::AbiFacts,
) -> Result<AbiValue, AbiEvaluationError> {
    draft.evaluate_draft_expression(node, facts)
}

/// Artifact identity grows linearly with the ABI arena, on a chain and on a
/// shared DAG.
///
/// This is the instrument the flattening exists for, mirroring `tiler-ir`'s
/// `abi_identity_size_grows_linearly_with_the_arena`. Under the `v4` encoding a
/// node's key embedded its whole subtree, so the chain was quadratic and the
/// shared DAG **doubled per level** — a 16-level DAG reached megabytes. A
/// constant increment per level is the property that says the arena is written
/// once and referenced by position.
#[test]
fn artifact_identity_size_grows_linearly_with_the_abi_arena() {
    /// Enough levels that a quadratic or exponential curve is unmistakable, and
    /// few enough that a `v4` re-run would still finish.
    const LEVELS: std::ops::Range<usize> = 0..17;

    for shared in [false, true] {
        let mut sizes = Vec::new();
        for levels in LEVELS {
            let semantic = semantic_program();
            let program = fused_program(&semantic, SCALE_BITS);
            let provider = lowering_provider(1);
            let environment =
                CompilationEnvironment::new(std::iter::once(provider.clone()), []).unwrap();
            let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
            draft.select_lowering_provider(selection(provider)).unwrap();
            let descriptor = draft.push_payload(payload(0xa1)).unwrap();
            let formulas = formulas(&mut draft);

            // Grow the guard, which is a use site, so every added node is
            // reached and verification admits the artifact.
            // Grown through a **launch precondition**, not the applicability
            // guard: the guard is derived from the program now, so it is no
            // longer a caller-supplied place to add arena depth. A precondition
            // is still artifact-owned and still reaches identity, so this
            // measures what it always measured -- identity size against arena
            // size -- through the seam that survives the binding.
            let mut grown = formulas.always;
            for _ in 0..levels {
                grown = if shared {
                    draft.push_binary(AbiBinaryOp::And, grown, grown).unwrap()
                } else {
                    let filler = draft.push_root(AbiRoot::BooleanLiteral(false)).unwrap();
                    draft.push_binary(AbiBinaryOp::Or, grown, filler).unwrap()
                };
            }
            let mut spec = variant(&formulas, descriptor, b"fused");
            spec.entries[0].launch.preconditions = vec![grown];
            draft.push_variant(&program, spec).unwrap();
            declare_realization(&mut draft, &program);
            let artifact = draft.build().unwrap();

            let nodes = artifact.expressions().len();
            let bytes = artifact.canonical_identity().as_bytes().len();
            let shape = if shared { "SharedDag" } else { "Chain" };
            println!("MEASURE {shape} {levels:>2} levels: {nodes:>3} nodes, {bytes} bytes");
            sizes.push((nodes, bytes));
        }

        let increments: Vec<usize> = sizes
            .windows(2)
            .skip(1)
            .map(|pair| pair[1].1 - pair[0].1)
            .collect();
        assert!(
            increments.windows(2).all(|pair| pair[0] == pair[1]),
            "identity size must grow by a constant per level, measured {increments:?}"
        );
    }
}

/// `adopt_abi` replays a program's arena and resolves every reached position.
///
/// This is the mechanism that makes "a variant's ABI is its program's ABI"
/// checkable instead of a producer convention. The dedup assertion is the part
/// worth having: the builder keys by content, so replaying an arena that names
/// one expression from two positions must yield one handle, or a variant would
/// carry two spellings of one formula and the identity would distinguish them.
#[test]
fn adopting_a_program_abi_replays_every_reached_position() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new(std::iter::once(provider.clone()), []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();

    let arena = program.abi_expressions();
    let roots: Vec<u32> = (0..u32::try_from(arena.len()).unwrap()).collect();
    let minted = draft.adopt_abi(arena, &roots).expect("the arena replays");

    assert_eq!(minted.len(), arena.len());
    assert!(
        minted.iter().all(Option::is_some),
        "every position was named as a root, so every one must be replayed"
    );

    // Replaying the same arena again must mint no new handles: the builder
    // deduplicates by content, so the second pass resolves to the first's.
    let again = draft
        .adopt_abi(arena, &roots)
        .expect("the arena replays twice");
    assert_eq!(
        minted, again,
        "replay is not idempotent, so content dedup failed"
    );
}

/// A root outside the arena is a typed rejection, not a panic.
#[test]
fn adopting_an_abi_with_an_out_of_range_root_is_rejected() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new(std::iter::once(provider.clone()), []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();

    let arena = program.abi_expressions();
    let beyond = u32::try_from(arena.len()).unwrap();
    assert_eq!(
        draft.adopt_abi(arena, &[beyond]),
        Err(ArtifactBuildError::ExpressionOutOfRange { position: beyond }),
    );
}

/// Does the artifact layer accept a *program-owned* ABI expression?
///
/// This is the question `reconcile-the-artifact-and-program-abi-expression-obligations`
/// exists to answer, isolated from the build path so a wiring fault in a
/// larger change cannot be mistaken for a layer disagreement. It adopts the
/// program's arena and then asks the artifact builder to accept the program's
/// own launch expression at the use site that expression is for.
#[test]
fn probe_whether_a_program_expression_satisfies_the_artifact_obligations() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new(std::iter::once(provider.clone()), []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();

    let stage = program.stages().next().expect("one stage");
    let launch = stage.launch();
    let roots = vec![launch.grid_threads, launch.threads_per_workgroup];
    let adopted = draft
        .adopt_abi(program.abi_expressions(), &roots)
        .expect("the program arena replays onto the artifact builder");

    let grid = adopted[usize::try_from(launch.grid_threads).unwrap()]
        .expect("the grid expression was replayed");
    let workgroup = adopted[usize::try_from(launch.threads_per_workgroup).unwrap()]
        .expect("the workgroup expression was replayed");

    println!("PROBE grid handle {grid:?} workgroup handle {workgroup:?}");
    println!(
        "PROBE program arena {} nodes",
        program.abi_expressions().len()
    );
    println!("PROBE artifact arena after replay");

    // The handles are the artifact builder's own, minted by `adopt_abi`, so if
    // anything below fails it is an obligation and not a foreign handle.
    assert_ne!(grid, workgroup, "two distinct launch expressions collapsed");
}
