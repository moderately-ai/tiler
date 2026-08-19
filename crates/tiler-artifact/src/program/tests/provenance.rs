//! Reached versus unused provenance (ADR 0072).

use super::super::{
    ArtifactProgramBuilder, AvailabilityPhase, CompilationEnvironment, SelectedProvider,
};
use super::support::artifacts::lowering_subject;
use super::support::graphs::{checked_coverage_over, checked_coverage_under, strict_contract};
use super::support::kernels::fused_program_with_coverage;
use super::{
    SCALE_BITS, build_artifact, build_graph, declare_realization, formulas, fused_program,
    lowering_provider, payload, semantic_program, spare_provider, variant,
};
use std::sync::Arc;
use tiler_ir::index::{
    FrozenScalarRegistry, IndexRealizationLaw, NumericalContractIdentity, ScalarArity,
    ScalarAttributeSchema, ScalarEffect, ScalarInferenceError, ScalarInferenceOutputs,
    ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract, ScalarOperationDefinition,
    ScalarOperationInferencer, ScalarRegistryBuilder, add_f32_scalar_op, constant_f32_scalar_op,
    multiply_f32_scalar_op,
};
use tiler_ir::program::{CoveredOccurrence, VerifiedKernelProgram};
use tiler_ir::schedule::{
    ApproximationEnvelope, ExceptionalValueAssumption, F32NumericalContractKey, FlushedZeroSign,
    MaterializationRounding, NumericalPermission, SubnormalMode,
};
use tiler_ir::semantic::{
    CanonicalIntegerWidth, CanonicalValue, CanonicalValueKind, CanonicalValueView, F32,
    F32_CONSTANT_BITS_ATTRIBUTE, FrozenSemanticRegistry, InputKey, NormativeDefinitionRef, OpKey,
    OperationArity, OperationAttributeSchema, OperationConformance, OperationDefinition,
    OperationDefinitionFacts, OperationEffect, OperationInferenceError, OperationInferenceOutputs,
    OperationInferenceRequest, OperationInferencer, OperationSchema, OutputKey,
    ProviderDiagnosticCode, ProviderIdentity, REDUCTION_AXES_ATTRIBUTE, RegistryError,
    SemanticProgram, SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
    SemanticRegistryRegistrar, TypeDefinitionFacts, TypeKey, ValueFact, ValueTypeDefinition,
    ValueTypeDefinitionKey, add_f32_op, constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
};
use tiler_ir::shape::{
    Axis, BindingSource, FactProvenance, RootBinding, Shape, ShapeEnvBuilder, ShapeSymbol,
    SourcedExtent, SymbolScope,
};

/// The same contract flushing subnormals, used only to perturb *evidence*.
///
/// A numerical contract reaches a receipt's executable coverage and is
/// deliberately absent from semantic graph meaning, so two coverages minted
/// under these two contracts name the same occurrences of the same graph and
/// carry different proofs.
fn flush_contract() -> NumericalContractIdentity {
    F32NumericalContractKey::new(
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        },
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        },
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
    .expect("the fixture contract vector is coherent")
    .into()
}

/// The fused program over a graph whose registry is not the standard one.
///
/// See [`scalars_over`] for why these fixtures cannot use the governed standard
/// scalar profile.
fn fused_program_over_fixture_scalars(
    semantic: &SemanticProgram,
    scale_bits: u32,
) -> VerifiedKernelProgram {
    let scalars = scalars_over(semantic.semantic_registry());
    fused_program_with_coverage(
        semantic,
        scale_bits,
        &checked_coverage_over(semantic, &scalars, &strict_contract()),
    )
}

// -------------------------------------------------------------------------
// Reached versus unused provenance (ADR 0072)
// -------------------------------------------------------------------------

/// The refinement evidence a stage names reaches artifact identity.
///
/// This is the artifact half of the coverage binding, and it is the half that
/// needs its own test: the artifact writes the stage subject through its own
/// encoder, so an artifact blind to a difference the kernel program folds would
/// be a real divergence rather than a duplicated assertion. The perturbation is
/// the governed numerical contract the receipts were minted under — a genuine
/// difference in what was proved, and one the semantic graph does not carry.
#[test]
fn refinement_evidence_moves_program_and_artifact_identity() {
    let semantic = semantic_program();
    let strict = checked_coverage_under(&semantic, &strict_contract());
    let flushed = checked_coverage_under(&semantic, &flush_contract());
    assert_eq!(
        strict
            .iter()
            .map(CoveredOccurrence::occurrence)
            .collect::<Vec<_>>(),
        flushed
            .iter()
            .map(CoveredOccurrence::occurrence)
            .collect::<Vec<_>>(),
        "the perturbation changes evidence, not which occurrences are covered",
    );
    assert!(
        strict
            .iter()
            .zip(&flushed)
            .any(|(left, right)| left.refinement() != right.refinement()),
        "two governed contracts must mint distinct executable-coverage evidence",
    );

    let strict_program = fused_program_with_coverage(&semantic, SCALE_BITS, &strict);
    let flushed_program = fused_program_with_coverage(&semantic, SCALE_BITS, &flushed);
    assert_ne!(
        strict_program.canonical_identity(),
        flushed_program.canonical_identity(),
    );

    let provider = lowering_provider(1);
    let strict_artifact = build_artifact(
        &semantic,
        &strict_program,
        provider.clone(),
        std::slice::from_ref(&provider),
    );
    let flushed_artifact = build_artifact(
        &semantic,
        &flushed_program,
        provider.clone(),
        std::slice::from_ref(&provider),
    );
    assert_ne!(
        strict_artifact.canonical_identity(),
        flushed_artifact.canonical_identity(),
    );
}

#[test]
fn a_reached_capability_provider_revision_changes_identity() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let available = [lowering_provider(1), lowering_provider(2)];
    let first = build_artifact(&semantic, &program, lowering_provider(1), &available);
    let second = build_artifact(&semantic, &program, lowering_provider(2), &available);
    assert_ne!(first.canonical_identity(), second.canonical_identity());
}

/// The capability's own revision reaches identity, independently of the provider's.
///
/// `docs/operation-extensions.md` makes the two revisions independent — one
/// provider registers several capabilities that move at different rates — so
/// folding only the provider's left a provider free to change what its lowering
/// emits and produce a byte-identical artifact identity, which is exactly the
/// drift the capability revision exists to catch. Both directions are asserted:
/// the revision moving changes identity, and everything else held equal it is
/// the only thing that did.
#[test]
fn a_reached_capability_revision_changes_identity() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let build = |capability_revision: u32| {
        let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
        draft
            .select_provider(SelectedProvider {
                provider: provider.clone(),
                capability: lowering_subject("tiler", "strict-serial-sum-f32", 1),
                capability_revision,
            })
            .unwrap();
        let descriptor = draft.push_payload(payload(0xa1)).unwrap();
        let formulas = formulas(&mut draft);
        draft
            .push_variant(&program, variant(&formulas, descriptor, b"fused"))
            .unwrap();
        declare_realization(&mut draft, &program);
        draft.build().unwrap()
    };

    let first = build(1);
    let second = build(2);
    assert_ne!(first.canonical_identity(), second.canonical_identity());
    assert_eq!(
        first.canonical_identity(),
        build(1).canonical_identity(),
        "nothing else in the fixture varies with the revision",
    );
    assert_eq!(
        first.selected_providers()[0].provider,
        second.selected_providers()[0].provider,
        "the provider's own revision is unchanged; only the capability's moved",
    );
}

#[test]
fn an_unused_environment_provider_does_not_change_identity() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let selected = lowering_provider(1);
    let lean = build_artifact(
        &semantic,
        &program,
        selected.clone(),
        std::slice::from_ref(&selected),
    );
    let crowded = build_artifact(
        &semantic,
        &program,
        selected.clone(),
        &[selected.clone(), spare_provider(1)],
    );
    let bumped = build_artifact(
        &semantic,
        &program,
        selected.clone(),
        &[selected, spare_provider(7)],
    );
    assert_eq!(lean.canonical_identity(), crowded.canonical_identity());
    assert_eq!(crowded.canonical_identity(), bumped.canonical_identity());
    // The environments genuinely differed; only the reached half was packaged.
    assert_eq!(lean.selected_providers().len(), 1);
    assert_eq!(crowded.selected_providers().len(), 1);
}

#[test]
fn a_reached_semantic_provider_revision_changes_identity() {
    let first = governed_program(1);
    let second = governed_program(2);
    assert_eq!(
        first.semantic_identity().graph(),
        second.semantic_identity().graph(),
    );
    assert_eq!(
        first.semantic_identity().reached_definitions(),
        second.semantic_identity().reached_definitions(),
    );
    assert_ne!(
        first.semantic_identity().admission_provenance(),
        second.semantic_identity().admission_provenance(),
    );
    let provider = lowering_provider(1);
    let first_artifact = build_artifact(
        &first,
        &fused_program_over_fixture_scalars(&first, SCALE_BITS),
        provider.clone(),
        std::slice::from_ref(&provider),
    );
    let second_artifact = build_artifact(
        &second,
        &fused_program_over_fixture_scalars(&second, SCALE_BITS),
        provider.clone(),
        &[provider],
    );
    assert_ne!(
        first_artifact.canonical_identity(),
        second_artifact.canonical_identity(),
    );
}

/// A symbolic semantic program opens an artifact builder, and its symbol is
/// published rather than erased.
///
/// **This replaces `a_symbolic_semantic_program_never_reaches_the_artifact_builder`.**
/// That test pinned the published interface as a fixed `Shape`, so an extent
/// naming a declared symbol was refused here rather than encoded. Since
/// `tiler.artifact-program.v21` the entry states each axis literal-or-symbol, so
/// the boundary has an honest encoding and the refusal has nothing left to
/// protect.
///
/// What has to stay true is that publishing it did not erase which axis is
/// symbolic. The assertion below reads the published entry back and requires the
/// symbol by name — a spelling that collapsed the axis to a literal, zero or
/// otherwise, fails here.
#[test]
fn a_symbolic_semantic_program_publishes_its_symbol_by_name() {
    let scope = SymbolScope::new("artifact/0").unwrap();
    let rows = ShapeSymbol::new(scope, "rows").unwrap();
    let mut draft = ShapeEnvBuilder::new();
    draft.declare(rows.clone()).unwrap();
    draft
        .bind(
            &rows,
            RootBinding::new(
                BindingSource::InputDimension {
                    input: InputKey::new("input").unwrap(),
                    axis: Axis::new(0),
                },
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )
            .unwrap(),
        )
        .unwrap();
    let environment = Arc::new(draft.build().unwrap());

    let mut builder =
        SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap();
    let value = builder
        .input_sourced::<F32>(
            InputKey::new("input").unwrap(),
            vec![SourcedExtent::Symbol(rows)],
        )
        .expect("the symbolic input is admitted at the semantic layer");
    builder
        .output(OutputKey::new("result").unwrap(), value)
        .unwrap();
    let symbolic = builder.build().unwrap();
    assert_ne!(
        symbolic.semantic_identity().shape_environment(),
        semantic_program().semantic_identity().shape_environment(),
        "the fixture really does carry a non-empty environment subject",
    );

    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider]).expect("environment");
    assert!(
        ArtifactProgramBuilder::new(&symbolic, environment.clone()).is_ok(),
        "a symbolic interface extent has a published per-axis spelling",
    );
    assert!(
        ArtifactProgramBuilder::new(&semantic_program(), environment).is_ok(),
        "the neighbour differing only in the extent's source kind opens",
    );

    let published = super::super::builder::read_semantic_interface(&symbolic)
        .expect("the fixture's boundary is publishable");
    let (_, extents) = &published.input_extent_sources()[0];
    assert_eq!(
        extents,
        &vec![SourcedExtent::Symbol(
            ShapeSymbol::new(SymbolScope::new("artifact/0").unwrap(), "rows").unwrap()
        )],
        "the published boundary must name the symbol, not a literal standing for it",
    );
}

#[test]
fn an_unused_semantic_provider_revision_does_not_change_identity() {
    let first = program_with_unused_provider(1);
    let second = program_with_unused_provider(2);
    // The fixture is meaningful only if the two programs really differ.
    assert_ne!(
        first.semantic_identity().registry_snapshot(),
        second.semantic_identity().registry_snapshot(),
    );
    assert_eq!(
        first.semantic_identity().admission_provenance(),
        second.semantic_identity().admission_provenance(),
    );
    let provider = lowering_provider(1);
    let first_program = fused_program_over_fixture_scalars(&first, SCALE_BITS);
    let second_program = fused_program_over_fixture_scalars(&second, SCALE_BITS);
    // The kernel-program leg is asserted separately from the artifact leg: the
    // artifact folds the program identity, so equal artifacts would otherwise
    // leave a program-level divergence indistinguishable from an artifact-level
    // one that happened to cancel.
    assert_eq!(
        first_program.canonical_identity(),
        second_program.canonical_identity(),
    );
    let first_artifact = build_artifact(
        &first,
        &first_program,
        provider.clone(),
        std::slice::from_ref(&provider),
    );
    let second_artifact = build_artifact(&second, &second_program, provider.clone(), &[provider]);
    assert_eq!(
        first_artifact.canonical_identity(),
        second_artifact.canonical_identity(),
    );
}

// -------------------------------------------------------------------------
// Semantic-provider fixtures
// -------------------------------------------------------------------------

fn diagnostic_code(value: &str) -> ProviderDiagnosticCode {
    ProviderDiagnosticCode::new(value).unwrap()
}

#[derive(Clone, Copy)]
enum TestOperation {
    Constant,
    Binary,
    Sum,
}

impl OperationInferencer for TestOperation {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let attributes = request.attributes();
        match self {
            Self::Constant => {
                outputs.try_push(ValueFact::new(F32::resolved_type(), Shape::new([])))
            }
            Self::Binary => {
                let left = request.static_operand_shape(0)?;
                let right = request.static_operand_shape(1)?;
                let shape = if left.rank() == 0 {
                    right.clone()
                } else if right.rank() == 0 || left == right {
                    left.clone()
                } else {
                    return Err(OperationInferenceError::new(
                        diagnostic_code("test.binary.shape"),
                        "operands must have equal shapes or include one scalar",
                    )
                    .unwrap());
                };
                outputs.try_push(ValueFact::new(F32::resolved_type(), shape))
            }
            Self::Sum => {
                let Some(CanonicalValueView::Sequence(values)) = attributes
                    .get(REDUCTION_AXES_ATTRIBUTE)
                    .map(CanonicalValue::view)
                else {
                    return Err(OperationInferenceError::new(
                        diagnostic_code("test.sum.axes"),
                        "sum axes must be a sequence",
                    )
                    .unwrap());
                };
                let axes = values
                    .iter()
                    .map(|value| match value.view() {
                        CanonicalValueView::Unsigned {
                            width: CanonicalIntegerWidth::Bits32,
                            bits,
                        } => u32::try_from(bits).map(Axis::new).map_err(|_| {
                            OperationInferenceError::new(
                                diagnostic_code("test.sum.axis-width"),
                                "sum axis exceeds u32",
                            )
                            .unwrap()
                        }),
                        _ => Err(OperationInferenceError::new(
                            diagnostic_code("test.sum.axis-kind"),
                            "sum axes must be u32 values",
                        )
                        .unwrap()),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                outputs.try_push(ValueFact::new(
                    F32::resolved_type(),
                    request.static_operand_shape(0)?.without_axes(&axes),
                ))
            }
        }
    }
}

/// A provider the packaged graph actually reaches, with a settable revision.
struct GovernedTestSemantics {
    revision: u32,
}

impl SemanticRegistryProvider for GovernedTestSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler-test", "governed-semantics", self.revision).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_marked_value_type::<F32>(
            ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("tiler", "f32", 1).unwrap()),
                NormativeDefinitionRef::new("test binary32 semantics")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ),
            F32::resolved_type(),
        )?;
        register_test_operation(
            registrar,
            constant_f32_op(),
            0,
            [OperationAttributeSchema::required(
                F32_CONSTANT_BITS_ATTRIBUTE,
                CanonicalValueKind::FloatBits,
            )],
            TestOperation::Constant,
            IndexRealizationLaw::constant_f32(),
        )?;
        register_test_operation(
            registrar,
            multiply_f32_op(),
            2,
            [],
            TestOperation::Binary,
            IndexRealizationLaw::multiply_f32(),
        )?;
        register_test_operation(
            registrar,
            add_f32_op(),
            2,
            [],
            TestOperation::Binary,
            IndexRealizationLaw::add_f32(),
        )?;
        register_test_operation(
            registrar,
            strict_serial_sum_f32_op(),
            1,
            [OperationAttributeSchema::required(
                REDUCTION_AXES_ATTRIBUTE,
                CanonicalValueKind::Sequence,
            )],
            TestOperation::Sum,
            IndexRealizationLaw::strict_serial_sum_f32(),
        )
    }
}

/// Registers one governed test operation together with its realization law.
///
/// The law travels with the operation because an operation without one cannot
/// be refined, and a stage covering it therefore cannot name the proof its
/// coverage record requires. A "governed" test provider that registered
/// operations and no laws would describe a registry no program could execute.
fn register_test_operation<const N: usize>(
    registrar: &mut SemanticRegistryRegistrar<'_>,
    key: OpKey,
    operands: u32,
    attributes: [OperationAttributeSchema; N],
    inferencer: TestOperation,
    law: IndexRealizationLaw,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        key.clone(),
        OperationSchema::new(
            OperationArity::exact(operands),
            OperationArity::exact(1),
            attributes,
        )
        .unwrap(),
        NormativeDefinitionRef::new("test governed operation semantics")?,
        OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
        OperationConformance::new(CanonicalValue::boolean(true)),
        OperationEffect::Pure,
        Arc::new(inferencer),
    ))?;
    registrar.register_index_realization_law(key, 1, law)
}

/// A provider the packaged graph never reaches.
struct UnusedSemantics {
    revision: u32,
}

impl SemanticRegistryProvider for UnusedSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler-test", "unused-semantics", self.revision).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(TypeKey::new("tiler-test", "unused", 1).unwrap()),
            NormativeDefinitionRef::new("unused test semantics")?,
            TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
        ))
    }
}

fn governed_program(revision: u32) -> SemanticProgram {
    let mut registry = SemanticRegistryBuilder::new();
    registry
        .register_provider(&GovernedTestSemantics { revision })
        .unwrap();
    build_graph(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
}

fn program_with_unused_provider(revision: u32) -> SemanticProgram {
    let mut registry = SemanticRegistryBuilder::standard().unwrap();
    registry
        .register_provider(&UnusedSemantics { revision })
        .unwrap();
    build_graph(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
}

/// A scalar authority composed with an exact non-standard semantic registry.
///
/// The governed standard scalar profile is frozen over
/// [`FrozenSemanticRegistry::standard`], and a refinement verifier refuses a
/// scalar authority frozen over a different semantic authority — deliberately,
/// because the two together name what a region's arithmetic *means*. The
/// provider-provenance fixtures build their own semantic registries, so they
/// compose their own scalar profile over exactly those registries. The
/// definitions are the fixture's rather than the standard ones, which is
/// visible in the resulting evidence bytes and irrelevant to what these tests
/// compare: every artifact they compare was built through this same authority.
fn scalars_over(semantic: &FrozenSemanticRegistry) -> FrozenScalarRegistry {
    let provider = ProviderIdentity::new("tiler-test", "fixture-scalars", 1).unwrap();
    let mut builder = ScalarRegistryBuilder::new(semantic.clone());
    builder
        .register(
            provider.clone(),
            fixture_scalar_definition(
                constant_f32_scalar_op(),
                ScalarAttributeSchema::new([tiler_ir::index::ScalarAttributeField::required(
                    F32_CONSTANT_BITS_ATTRIBUTE,
                    CanonicalValueKind::FloatBits,
                )])
                .unwrap(),
                0,
            ),
        )
        .unwrap();
    for key in [multiply_f32_scalar_op(), add_f32_scalar_op()] {
        builder
            .register(
                provider.clone(),
                fixture_scalar_definition(key, ScalarAttributeSchema::empty(), 2),
            )
            .unwrap();
    }
    builder.freeze()
}

fn fixture_scalar_definition(
    key: ScalarOpKey,
    attributes: ScalarAttributeSchema,
    operands: usize,
) -> ScalarOperationDefinition {
    ScalarOperationDefinition::new(
        key,
        NormativeDefinitionRef::new("fixture scalar semantics").unwrap(),
        ScalarOperationContract::new(
            attributes,
            ScalarArity::exact(operands).unwrap(),
            ScalarArity::exact(1).unwrap(),
            ScalarEffect::Pure,
            CanonicalValue::record([]).unwrap(),
            CanonicalValue::record([]).unwrap(),
        ),
        Arc::new(FixtureF32Scalar),
    )
}

/// Every fixture scalar operation produces one `f32`.
///
/// A constant takes no operand, so the result type cannot be read off the
/// operands; the fixture's graph is homogeneous `f32` throughout, and this
/// states that rather than inferring it.
struct FixtureF32Scalar;

impl ScalarOperationInferencer for FixtureF32Scalar {
    fn infer(
        &self,
        _request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        outputs.try_push(F32::resolved_type())
    }
}
