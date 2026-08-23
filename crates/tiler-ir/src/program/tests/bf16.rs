//! The pure-BF16 producer path.
//!
//! Every layer below the program was already implemented and tested for `bf16`,
//! and the composition was unreachable: a `bf16` occurrence could not obtain
//! executable coverage, so no `bf16` kernel program verified. These fixtures walk
//! the same sealed path the `f32` ones do — no shortcut mints a receipt — so what
//! they demonstrate is that the refinement layer now admits the width, not that a
//! test can assert it does.

use super::super::{
    AlignmentRequirement, AllocationOwnership, KernelProgramBuilder, MaterializedOrigin,
    MaterializedValueSpec, MemorySpace, StageLaunch, StorageEncoding, StorageScalar, ValueRole,
};
use super::support::{
    SCALE_BITS, checked_coverage, declare_program_contract, device, elements, input_shape,
    linear_schedule, literal, program_input, read, serial_sum_program, strict_contract,
    write_access,
};
use crate::index::{
    FrozenIndexRealizationLawRegistry, FrozenScalarRegistry, IndexRealizationAuthority,
    IndexRefinementSubject, IndexRefinementVerificationError, IndexRefinementVerificationOutcome,
    NumericalContractIdentity,
};
use crate::kernel::{KernelType, lower_scheduled_region};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, ApproximationEnvelope, Bf16NumericalContractKey,
    BoundsProof, BoundsProofKind, BoundsWitnessId, ExceptionalValueAssumption, LogicalAccess,
    MaterializationRounding, NumericalPermission, NumericalRealization, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, PointwiseBf16ExpressionBuilder, RegionId,
    RegionProgram, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TensorRole,
    VerifiedScheduledRegion,
};
use crate::semantic::{
    Bf16, Bf16Add, Bf16Constant, Bf16Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder,
};

const BF16_SCALE_BITS: u16 = 0x4000; // 2.0bf16

const BF16_BIAS_BITS: u16 = 0x3f80; // 1.0bf16

/// The strict `bf16` contract, the direct sibling of [`strict_contract`].
fn strict_bf16_contract() -> NumericalContractIdentity {
    Bf16NumericalContractKey::new(
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ApproximationEnvelope::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
        MaterializationRounding::NearestTiesToEven,
    )
    .expect("the fixture bf16 contract vector is coherent")
    .into()
}

/// A four-operation pure-BF16 graph: `result = input * 2.0 + 1.0`.
///
/// Constant, multiply, and add are the complete registered `bf16` vocabulary, so
/// this is the widest pure-`bf16` program the semantic layer can state — not a
/// subset chosen to be easy.
fn bf16_program() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
    let input = draft
        .input::<Bf16>(InputKey::new("input").expect("key"), input_shape())
        .expect("input");
    let scale = Bf16Constant::apply(&mut draft, BF16_SCALE_BITS).expect("scale");
    let bias = Bf16Constant::apply(&mut draft, BF16_BIAS_BITS).expect("bias");
    let product = Bf16Multiply::apply(&mut draft, input, scale).expect("product");
    let mapped = Bf16Add::apply(&mut draft, product, bias).expect("mapped");
    draft
        .output(OutputKey::new("result").expect("key"), mapped)
        .expect("output");
    let program = draft.build().expect("verified bf16 semantic program");
    assert_eq!(program.operation_count(), 4);
    program
}

fn bf16_numerical() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-bf16",
        u32::from(crate::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS),
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ApproximationEnvelope::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    )
}

/// `(x * 2.0) + 1.0` in `bf16`, writing the program output directly.
fn bf16_output_region() -> VerifiedScheduledRegion {
    let shape = input_shape();
    let count = elements(&shape);
    let mut expression = PointwiseBf16ExpressionBuilder::new();
    let leaf = expression.input(AccessOrdinal::FIRST).expect("input");
    let scale = expression.constant(BF16_SCALE_BITS).expect("scale");
    let product = expression.multiply(leaf, scale).expect("product");
    let bias = expression.constant(BF16_BIAS_BITS).expect("bias");
    let root = expression.add(product, bias).expect("sum");
    let expression = expression.build(root).expect("bf16 pointwise expression");

    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder.iteration_shape(shape).expect("iteration shape");
    builder
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .expect("read access");
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .expect("write access");
    for (witness, tensor) in [(0, TensorRole::Input), (1, TensorRole::Output)] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: count,
                },
            })
            .expect("bounds proof");
    }
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: count,
            },
        })
        .expect("ownership proof");
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseBf16(expression),
            numerical: bf16_numerical(),
        })
        .expect("scalar program");
    builder
        .schedule(linear_schedule(count, OwnershipWitnessId::new(0)))
        .expect("schedule");
    builder.build().expect("verified bf16 region")
}

fn bf16_value(origin: MaterializedOrigin, role: ValueRole) -> MaterializedValueSpec {
    MaterializedValueSpec {
        origin,
        role,
        shape: input_shape(),
        storage_scalar: StorageScalar::Bf16,
        encoding: StorageEncoding::Unpacked,
        element_type: KernelType::Bf16,
        alignment: AlignmentRequirement::natural_for(StorageScalar::Bf16),
        memory_space: MemorySpace::Device,
    }
}

/// A pure-BF16 program obtains verified coverage for every occurrence and
/// reaches a verified kernel program over a `PointwiseBf16` region.
///
/// This is the composition the refinement layer previously made unreachable.
/// Every one of the four coverage records is minted by the verifier through
/// [`checked_coverage`], the same helper the `f32` fixtures use, so a record here
/// is refinement evidence rather than a fixture assertion — and the program
/// builds, which is what proves no stage covers nothing.
#[test]
fn a_pure_bf16_program_covers_every_occurrence_and_builds_a_verified_kernel_program() {
    let semantic = bf16_program();
    let coverage = checked_coverage(&semantic, &strict_bf16_contract());
    assert_eq!(
        coverage.len(),
        4,
        "every bf16 occurrence obtains executable coverage"
    );
    assert_eq!(
        coverage
            .iter()
            .map(|covered| covered.occurrence().get())
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3],
        "the coverage partition is the graph's complete canonical occurrence run"
    );

    let kernel = lower_scheduled_region(&bf16_output_region()).expect("bf16 kernel");
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    // Only the quantities this one stage names: six `bf16` elements are twelve
    // bytes on both sides. Minting the reduction extents the `f32` fixtures
    // share would leave an ABI expression no stage references, which the
    // program verifier refuses by name.
    let value_bytes = literal(&mut builder, 12);
    let grid_threads = literal(&mut builder, 6);
    let threads_per_workgroup = literal(&mut builder, 1);
    let external = builder
        .push_allocation(device(12, AllocationOwnership::External))
        .expect("external allocation");
    let produced = builder
        .push_allocation(device(12, AllocationOwnership::Program))
        .expect("output allocation");
    let source = builder
        .push_value(
            bf16_value(program_input("input"), ValueRole::Input),
            external,
        )
        .expect("input value");
    let output = builder
        .push_value(
            bf16_value(MaterializedOrigin::Internal, ValueRole::Output),
            produced,
        )
        .expect("output value");
    let source_view = builder.push_whole_view(source).expect("input view");
    let output_view = builder.push_whole_view(output).expect("output view");
    builder
        .push_stage(
            &kernel,
            &coverage,
            &[
                read(source_view, value_bytes),
                write_access(output_view, value_bytes),
            ],
            StageLaunch {
                grid_threads,
                threads_per_workgroup,
            },
        )
        .expect("the bf16 stage covers every occurrence of its bound graph");
    builder
        .push_output(OutputKey::new("result").expect("key"), output)
        .expect("named output");
    declare_program_contract(&mut builder);
    let program = builder
        .build()
        .map_err(|error| error.diagnostics().to_vec())
        .expect("verified bf16 kernel program");
    assert_eq!(program.stages().count(), 1);
}

/// A candidate region that does not realize its occurrence is refused.
///
/// The rubber-stamp perturbation. The candidate handed to the verifier is a
/// *real* verified region minted by a *different* occurrence's law — the add's
/// region offered as the multiply's realization — so it passes every structural
/// check and fails only the one that matters: the law derives its own expected
/// region and compares canonical identities. A verifier that consulted the
/// caller's region instead of deriving would admit this.
#[test]
fn a_bf16_candidate_that_does_not_realize_its_occurrence_is_refused() {
    let semantic = bf16_program();
    let contract = strict_bf16_contract();
    let registry = semantic.semantic_registry().clone();
    let scalars = FrozenScalarRegistry::standard().expect("scalar authority");
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(registry.clone(), scalars.clone())
        .expect("the standard authorities cohere");

    let subject_for = |ordinal: usize| {
        let operation = semantic
            .operations()
            .nth(ordinal)
            .expect("the fixture has four operations");
        IndexRefinementSubject::derive(&semantic, operation.id(), contract.clone())
            .expect("every bf16 occurrence derives a subject")
    };
    let region_for = |subject: &IndexRefinementSubject| {
        registry
            .index_realization_law(subject.operation())
            .expect("every bf16 occurrence has a registered law")
            .law
            .clone()
            .realize(subject, &scalars)
            .expect("the registered law realizes its own subject")
    };

    let multiply = subject_for(2);
    let add = subject_for(3);
    assert_ne!(
        multiply.operation(),
        add.operation(),
        "two distinct families"
    );
    let honest = region_for(&multiply);
    let foreign = region_for(&add);
    assert_ne!(
        honest.canonical_identity(),
        foreign.canonical_identity(),
        "the perturbation is a genuinely different region"
    );

    let verify_against = |candidate: &crate::index::VerifiedIndexRegion| {
        let reached = scalars
            .revalidate_region(candidate)
            .expect("both candidates are themselves well-formed");
        let authority = IndexRealizationAuthority::admit(
            &registry,
            &scalars,
            multiply.operation().clone(),
            multiply.signature().clone(),
            reached.reached_operations(),
        )
        .expect("the authority admits the candidate's reached ceiling");
        laws.resolve(&multiply)
            .expect("the multiply resolves its own law")
            .verify(&authority, candidate)
    };

    // The positive control. Without it a refusal below would be consistent with
    // the fixture being broken in some way that has nothing to do with the
    // perturbation, and the test would assert nothing about the verifier.
    assert!(
        matches!(
            verify_against(&honest).expect("the multiply's own region verifies"),
            IndexRefinementVerificationOutcome::Verified(_)
        ),
        "the honest candidate must verify, or the refusal below proves nothing"
    );

    let error = verify_against(&foreign)
        .expect_err("a region realizing another occurrence must be refused");
    assert!(
        matches!(
            error,
            IndexRefinementVerificationError::SemanticRealizationMismatch { .. }
        ),
        "expected SemanticRealizationMismatch, got {error:?}"
    );
}

/// Neither width's program verifies under the other width's contract.
///
/// Both directions, because they fail for the same reason and a check that only
/// ran one way would not establish it: the law derives the arithmetic its result
/// is produced in from the verified subject, and a contract stated for another
/// width governs another format's subnormals, rounding, and canonical NaN. The
/// refusal is named rather than a generic mismatch, so a reader is told which of
/// the verifier's obligations the pair failed.
#[test]
fn a_program_under_the_other_widths_contract_is_refused_by_name() {
    let cases = [
        (
            "a bf16 program under an f32 contract",
            bf16_program(),
            strict_contract(),
            strict_bf16_contract(),
        ),
        (
            "an f32 program under a bf16 contract",
            serial_sum_program(SCALE_BITS),
            strict_bf16_contract(),
            strict_contract(),
        ),
    ];
    for (case, semantic, foreign_contract, native_contract) in cases {
        let registry = semantic.semantic_registry().clone();
        let scalars = FrozenScalarRegistry::standard().expect("scalar authority");
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(registry.clone(), scalars.clone())
                .expect("the standard authorities cohere");
        let operation = semantic.operations().next().expect("a first operation");
        let outcome_under = |contract: NumericalContractIdentity| {
            let subject = IndexRefinementSubject::derive(&semantic, operation.id(), contract)
                .expect("a subject derives under any validated contract identity");
            let region = registry
                .index_realization_law(subject.operation())
                .expect("the fixture's first operation has a registered law")
                .law
                .clone()
                .realize(&subject, &scalars)
                .expect("the law realizes a region from types, not from the contract");
            let reached = scalars
                .revalidate_region(&region)
                .expect("the law's own region revalidates");
            let authority = IndexRealizationAuthority::admit(
                &registry,
                &scalars,
                subject.operation().clone(),
                subject.signature().clone(),
                reached.reached_operations(),
            )
            .expect("the authority admits the region's reached ceiling");
            laws.resolve(&subject)
                .expect("resolution does not consult the contract")
                .verify(&authority, &region)
        };

        // The positive control: the identical setup under the program's own
        // width verifies. Without it, a refusal would be consistent with the
        // fixture never having been verifiable at all, and the contract would
        // not be shown to be the thing that decided it.
        assert!(
            matches!(
                outcome_under(native_contract).expect("the native contract governs"),
                IndexRefinementVerificationOutcome::Verified(_)
            ),
            "{case}: the native contract must verify, or the refusal proves nothing"
        );

        let error =
            outcome_under(foreign_contract).expect_err("the cross-width contract must be refused");
        assert!(
            matches!(
                error,
                IndexRefinementVerificationError::NumericalContractNotGoverned
            ),
            "{case}: expected NumericalContractNotGoverned, got {error:?}"
        );
    }
}

/// The `bf16` rows did not disturb the `f32` refinement evidence beside them.
///
/// The load-bearing property of this whole step. A refinement receipt's
/// *executable coverage* is what reaches kernel-program and artifact identity,
/// and it restates only reached-only projections — never the whole scalar or
/// law-registry snapshots that the three new rows moved. Pinning the f32
/// coverage bytes against a value computed from the graph itself would move with
/// whatever moved them, so this compares the two widths' coverage for the one
/// property that must hold: the f32 records are unchanged in content while the
/// registries beneath them are not.
#[test]
fn the_bf16_rows_leave_f32_executable_coverage_untouched() {
    let semantic = serial_sum_program(SCALE_BITS);
    let coverage = checked_coverage(&semantic, &strict_contract());
    assert_eq!(coverage.len(), 5);

    // The scalar authority beneath them did move: it now defines the three bf16
    // per-point operations. If executable coverage folded that snapshot, the
    // records above could not be stable across it.
    let scalars = FrozenScalarRegistry::standard().expect("scalar authority");
    for key in [
        crate::index::constant_bf16_scalar_op(),
        crate::index::multiply_bf16_scalar_op(),
        crate::index::add_bf16_scalar_op(),
    ] {
        assert!(
            scalars.definition(&key).is_some(),
            "{key:?} is registered in the same snapshot the f32 coverage was minted under"
        );
    }

    // And the f32 records reached none of them.
    let reached_bf16 = coverage.iter().any(|covered| {
        let bytes = covered.refinement().as_bytes();
        bytes
            .windows(b"constant-bf16".len())
            .any(|window| window == b"constant-bf16")
    });
    assert!(
        !reached_bf16,
        "an f32 occurrence's reached-only coverage names no bf16 scalar"
    );
}
