//! The refinement verifier's own suite.
//!
//! Moved whole from the single-file module: every fixture, name, and
//! assertion is unchanged, and the suite still reads the module's vocabulary
//! through `use super::*`, which is why the spine states the names it takes.

use std::sync::Arc;

use super::*;
use crate::index::{
    DomainRole, IndexRegionBuilder, ScalarArity, ScalarAttributeField, ScalarAttributeSchema,
    ScalarEffect, ScalarInferenceError, ScalarInferenceOutputs, ScalarInferenceRequest,
    ScalarOperationContract, ScalarOperationDefinition, ScalarOperationInferencer,
    ScalarRegistryBuilder,
};
use crate::program::abi::AvailabilityPhase;
use crate::semantic::{
    AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueKind,
    EncodedComponentDeclaration, EncodedComponentRole, EncodedComponentShape,
    EncodedNumericContract, F32, F32Constant, F32Multiply, InputKey, NormativeDefinitionRef, OpKey,
    OperationArity, OperationConformance, OperationDefinition, OperationDefinitionFacts,
    OperationInferenceError, OperationInferenceOutputs, OperationInferenceRequest,
    OperationInferencer, OperationSchema, OutputKey, ProviderDiagnosticCode, QuantSchemeKey,
    SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
    SemanticRegistryRegistrar, TypeKey,
};
use crate::shape::{
    BindingSource, EXTENT_PHASE_CEILING, Extent, ExtentRelation, ExtentTerm, FactProvenance,
    InterfaceParameterKey, RootBinding, SemanticInputConstraint, ShapeEnvBuilder, ShapeSymbol,
    SourcedExtent, SymbolScope,
};

const LENGTH: u64 = 65_535;

struct PanicAfterBound {
    yielded: usize,
    value: ResolvedValueType,
}

impl Iterator for PanicAfterBound {
    type Item = ResolvedValueType;

    fn next(&mut self) -> Option<Self::Item> {
        assert!(
            self.yielded <= MAX_INDEX_REFINEMENT_SIGNATURE_VALUES,
            "the bounded signature constructor over-consumed its caller iterator"
        );
        self.yielded += 1;
        Some(self.value.clone())
    }
}

fn f32_type() -> ResolvedValueType {
    ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap())
}

fn encoded_boundary(components: usize) -> IndexRefinementBoundary {
    let field = CanonicalField::new(AttributeFieldId::new(1), CanonicalValue::boolean(true));
    let contract = if components == 0 {
        EncodedNumericContract::new([field]).unwrap()
    } else {
        EncodedNumericContract::with_components(
            [field],
            (1..=components).map(|role| {
                EncodedComponentDeclaration::new(
                    EncodedComponentRole::new(u32::try_from(role).unwrap()),
                    f32_type(),
                    EncodedComponentShape::LogicalValue,
                )
            }),
        )
        .unwrap()
    };
    IndexRefinementBoundary {
        value_type: ResolvedValueType::encoded_numeric(
            QuantSchemeKey::new("test", "resource-bound", 1).unwrap(),
            contract,
        )
        .unwrap(),
        shape: Shape::from_dims([1]),
        sourced: crate::shape::SourcedShape::from_shape(Shape::from_dims([1])),
    }
}

fn test_contract() -> NumericalContractIdentity {
    let key = F32NumericalContractKey::new(
        crate::schedule::SubnormalMode::Preserve,
        crate::schedule::SubnormalMode::Preserve,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::ApproximationEnvelope::Forbidden,
        crate::schedule::ExceptionalValueAssumption::MakeNoAssumption,
        crate::schedule::ExceptionalValueAssumption::MakeNoAssumption,
        crate::schedule::MaterializationRounding::NearestTiesToEven,
    )
    .unwrap();
    key.into()
}

#[test]
fn a_validated_contract_key_converts_without_reparsing() {
    let key = F32NumericalContractKey::new(
        crate::schedule::SubnormalMode::Preserve,
        crate::schedule::SubnormalMode::Preserve,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::ApproximationEnvelope::Forbidden,
        crate::schedule::ExceptionalValueAssumption::MakeNoAssumption,
        crate::schedule::ExceptionalValueAssumption::MakeNoAssumption,
        crate::schedule::MaterializationRounding::NearestTiesToEven,
    )
    .unwrap();
    let spelling = key.as_str().to_owned();
    let identity = NumericalContractIdentity::from(key);
    assert_eq!(identity.as_str(), spelling);
}

#[test]
fn signature_ingestion_stops_after_the_first_over_limit_value_on_each_side() {
    for side in [
        IndexRefinementSignatureSide::Operands,
        IndexRefinementSignatureSide::Results,
    ] {
        let unbounded = PanicAfterBound {
            yielded: 0,
            value: f32_type(),
        };
        let result = match side {
            IndexRefinementSignatureSide::Operands => IndexRefinementSignature::new(unbounded, []),
            IndexRefinementSignatureSide::Results => IndexRefinementSignature::new([], unbounded),
        };
        assert_eq!(
            result,
            Err(IndexRefinementVerificationError::SignatureTooLarge {
                side,
                actual: MAX_INDEX_REFINEMENT_SIGNATURE_VALUES + 1,
                limit: MAX_INDEX_REFINEMENT_SIGNATURE_VALUES,
            })
        );
    }
}

#[test]
fn raw_emitted_scalar_declarations_are_bounded_before_deduplication() {
    let semantic = FrozenSemanticRegistry::standard().unwrap();
    let scalars = FrozenScalarRegistry::standard().unwrap();
    let signature = IndexRefinementSignature::new([f32_type(), f32_type()], [f32_type()]).unwrap();
    let emitted =
        vec![super::super::multiply_f32_scalar_op(); MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS + 1];
    assert!(matches!(
        IndexRealizationAuthority::admit(
            &semantic,
            &scalars,
            crate::semantic::multiply_f32_op(),
            signature,
            &emitted,
        ),
        Err(IndexRefinementVerificationError::EmittedScalarOperationsTooLarge {
            actual,
            limit: MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS,
        }) if actual == MAX_REFINEMENT_EMITTED_SCALAR_OPERATIONS + 1
    ));
}

struct BinaryIdentity;

impl OperationInferencer for BinaryIdentity {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        if request.operands().len() == 2 && request.attributes().fields().is_empty() {
            outputs.try_push(request.operands()[0].clone())
        } else {
            Err(OperationInferenceError::new(
                ProviderDiagnosticCode::new("test.refinement-law.signature").unwrap(),
                "test operation requires two operands and no attributes",
            )
            .unwrap())
        }
    }
}

struct RefinementLawProvider(Option<super::super::IndexRealizationLaw>);

struct UnusedSemanticProvider(u32);

struct ReachedSemanticProvider(u32);

impl SemanticRegistryProvider for UnusedSemanticProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "unused-refinement-semantics", self.0).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_operation(OperationDefinition::new(
            OpKey::new("test", "unused-refinement-operation", 1).unwrap(),
            OperationSchema::new(OperationArity::exact(2), OperationArity::exact(1), []).unwrap(),
            NormativeDefinitionRef::new("unused refinement operation")?,
            OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
            OperationConformance::new(CanonicalValue::boolean(true)),
            OperationEffect::Pure,
            Arc::new(BinaryIdentity),
        ))
    }
}

fn reached_semantic_operation() -> OpKey {
    OpKey::new("test", "reached-refinement-operation", 1).unwrap()
}

impl SemanticRegistryProvider for ReachedSemanticProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "reached-refinement-semantics", self.0).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        let operation = reached_semantic_operation();
        registrar.register_operation(OperationDefinition::new(
            operation.clone(),
            OperationSchema::new(OperationArity::exact(2), OperationArity::exact(1), []).unwrap(),
            NormativeDefinitionRef::new("test reached refinement operation")?,
            OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
            OperationConformance::new(CanonicalValue::boolean(true)),
            OperationEffect::Pure,
            Arc::new(BinaryIdentity),
        ))?;
        registrar.register_index_realization_law(
            operation,
            1,
            super::super::IndexRealizationLaw::multiply_f32(),
        )
    }
}

struct TestScalarConstant;

impl ScalarOperationInferencer for TestScalarConstant {
    fn infer(
        &self,
        _request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        outputs.try_push(f32_type())
    }
}

fn test_scalar_definition(key: ScalarOpKey, normative: &str) -> ScalarOperationDefinition {
    ScalarOperationDefinition::new(
        key,
        NormativeDefinitionRef::new(normative).unwrap(),
        ScalarOperationContract::new(
            ScalarAttributeSchema::new([ScalarAttributeField::required(
                crate::semantic::F32_CONSTANT_BITS_ATTRIBUTE,
                CanonicalValueKind::FloatBits,
            )])
            .unwrap(),
            ScalarArity::exact(0).unwrap(),
            ScalarArity::exact(1).unwrap(),
            ScalarEffect::Pure,
            CanonicalValue::boolean(true),
            CanonicalValue::boolean(true),
        ),
        Arc::new(TestScalarConstant),
    )
}

fn test_binary_scalar_definition(key: ScalarOpKey, normative: &str) -> ScalarOperationDefinition {
    ScalarOperationDefinition::new(
        key,
        NormativeDefinitionRef::new(normative).unwrap(),
        ScalarOperationContract::new(
            ScalarAttributeSchema::empty(),
            ScalarArity::exact(2).unwrap(),
            ScalarArity::exact(1).unwrap(),
            ScalarEffect::Pure,
            CanonicalValue::boolean(true),
            CanonicalValue::boolean(true),
        ),
        Arc::new(TestScalarConstant),
    )
}

fn reached_semantic_fixture(
    revision: u32,
) -> (
    IndexRefinementSubject,
    ResolvedIndexRealization,
    VerifiedIndexRegionSequence,
    IndexRefinementReceipt,
) {
    let mut semantic = SemanticRegistryBuilder::standard().unwrap();
    semantic
        .register_provider(&ReachedSemanticProvider(revision))
        .unwrap();
    let semantic = semantic.freeze().unwrap();
    let mut scalars = ScalarRegistryBuilder::new(semantic.clone());
    scalars
        .register(
            ProviderIdentity::new("test", "selected-binary-scalar", 1).unwrap(),
            test_binary_scalar_definition(
                super::super::multiply_f32_scalar_op(),
                "test selected multiply scalar",
            ),
        )
        .unwrap();
    let scalars = scalars.freeze();
    let mut program = SemanticProgramBuilder::try_new(semantic.clone()).unwrap();
    let input = program
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([1]))
        .unwrap();
    let result = program
        .apply(
            reached_semantic_operation(),
            OperationAttributes::empty(),
            &[input.erase(), input.erase()],
        )
        .unwrap()
        .pop()
        .unwrap();
    program
        .output_resolved(OutputKey::new("output").unwrap(), result)
        .unwrap();
    let program = program.build().unwrap();
    let subject = IndexRefinementSubject::derive(
        &program,
        program.operations().next().unwrap().id(),
        test_contract(),
    )
    .unwrap();
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(semantic.clone(), scalars.clone())
        .unwrap();
    let resolution = laws.resolve(&subject).unwrap();
    let authority = IndexRealizationAuthority::admit(
        &semantic,
        &scalars,
        subject.operation().clone(),
        subject.signature().clone(),
        &[super::super::multiply_f32_scalar_op()],
    )
    .unwrap();
    let region = super::super::IndexRealizationLaw::multiply_f32()
        .realize(&subject, &scalars)
        .unwrap();
    let IndexRefinementVerificationOutcome::Verified(receipt) =
        resolution.verify(&authority, &region).unwrap()
    else {
        panic!("the reached fixture retains no residual proof")
    };
    (
        subject,
        resolution,
        VerifiedIndexRegionSequence::single(region),
        *receipt,
    )
}

fn constant_receipt_with_unused_authority(
    unused_semantic_revision: Option<u32>,
    constant_scalar_revision: u32,
    unused_scalar_revision: Option<u32>,
) -> IndexRefinementReceipt {
    let mut semantic = SemanticRegistryBuilder::standard().unwrap();
    if let Some(revision) = unused_semantic_revision {
        semantic
            .register_provider(&UnusedSemanticProvider(revision))
            .unwrap();
    }
    let semantic = semantic.freeze().unwrap();
    let mut scalars = ScalarRegistryBuilder::new(semantic.clone());
    scalars
        .register(
            ProviderIdentity::new("test", "selected-scalar", constant_scalar_revision).unwrap(),
            test_scalar_definition(
                super::super::constant_f32_scalar_op(),
                "test selected constant scalar",
            ),
        )
        .unwrap();
    if let Some(revision) = unused_scalar_revision {
        scalars
            .register(
                ProviderIdentity::new("test", "unused-scalar", revision).unwrap(),
                test_scalar_definition(
                    ScalarOpKey::new("test", "unused-scalar", 1).unwrap(),
                    "test unused scalar",
                ),
            )
            .unwrap();
    }
    let scalars = scalars.freeze();
    let mut program = SemanticProgramBuilder::try_new(semantic.clone()).unwrap();
    let value = F32Constant::apply(&mut program, 1.0_f32.to_bits()).unwrap();
    program
        .output(OutputKey::new("value").unwrap(), value)
        .unwrap();
    let program = program.build().unwrap();
    let subject = IndexRefinementSubject::derive(
        &program,
        program.operations().next().unwrap().id(),
        test_contract(),
    )
    .unwrap();
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(semantic.clone(), scalars.clone())
        .unwrap();
    let resolution = laws.resolve(&subject).unwrap();
    let authority = IndexRealizationAuthority::admit(
        &semantic,
        &scalars,
        subject.operation().clone(),
        subject.signature().clone(),
        &[super::super::constant_f32_scalar_op()],
    )
    .unwrap();
    let region = super::super::IndexRealizationLaw::constant_f32()
        .realize(&subject, &scalars)
        .unwrap();
    let IndexRefinementVerificationOutcome::Verified(receipt) =
        resolution.verify(&authority, &region).unwrap()
    else {
        panic!("a constant realization retains no residual proof")
    };
    *receipt
}

#[test]
fn executable_coverage_excludes_unused_authority_but_retains_reached_scalar_provenance() {
    let baseline = constant_receipt_with_unused_authority(None, 1, None);
    let unused_semantic = constant_receipt_with_unused_authority(Some(1), 1, None);
    let unused_semantic_revision = constant_receipt_with_unused_authority(Some(2), 1, None);
    let unused_scalar = constant_receipt_with_unused_authority(None, 1, Some(1));
    let unused_scalar_revision = constant_receipt_with_unused_authority(None, 1, Some(2));
    let reached_scalar_revision = constant_receipt_with_unused_authority(None, 2, None);

    for receipt in [
        &unused_semantic,
        &unused_semantic_revision,
        &unused_scalar,
        &unused_scalar_revision,
    ] {
        assert_eq!(
            baseline.executable_coverage_identity(),
            receipt.executable_coverage_identity()
        );
        assert_ne!(baseline.identity(), receipt.identity());
    }
    assert_ne!(
        baseline.executable_coverage_identity(),
        reached_scalar_revision.executable_coverage_identity()
    );
    let (_, _, _, reached_semantic) = reached_semantic_fixture(1);
    let (_, _, _, reached_semantic_revision) = reached_semantic_fixture(2);
    assert_eq!(reached_semantic.graph(), reached_semantic_revision.graph());
    assert_eq!(
        reached_semantic.final_stage(),
        reached_semantic_revision.final_stage()
    );
    assert_ne!(
        reached_semantic.executable_coverage_identity(),
        reached_semantic_revision.executable_coverage_identity()
    );
}

fn alternate_test_contract() -> NumericalContractIdentity {
    F32NumericalContractKey::new(
        crate::schedule::SubnormalMode::Preserve,
        crate::schedule::SubnormalMode::Preserve,
        crate::schedule::NumericalPermission::Permitted,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::NumericalPermission::Forbidden,
        crate::schedule::ApproximationEnvelope::Forbidden,
        crate::schedule::ExceptionalValueAssumption::MakeNoAssumption,
        crate::schedule::ExceptionalValueAssumption::MakeNoAssumption,
        crate::schedule::MaterializationRounding::NearestTiesToEven,
    )
    .unwrap()
    .into()
}

#[test]
fn executable_coverage_retains_each_replay_and_substitution_boundary() {
    let (subject, resolution, realization, receipt) = reached_semantic_fixture(1);
    let encode = |subject: &IndexRefinementSubject,
                  resolution: &ResolvedIndexRealization,
                  realization: &VerifiedIndexRegionSequence,
                  operands: &[OperandBinding],
                  results: &[ResultBinding],
                  proofs: &[IndexRefinementDomainProof]| {
        encode_executable_coverage_identity(
            subject,
            resolution,
            realization,
            &receipt.scalar_authorities(),
            operands,
            results,
            proofs,
        )
    };
    let baseline = encode(
        &subject,
        &resolution,
        &realization,
        receipt.operand_bindings(),
        receipt.result_bindings(),
        receipt.index_domain_proofs(),
    );
    assert_eq!(baseline, receipt.executable_coverage_identity().as_bytes());

    // A provider revision is excluded from graph meaning, so the graph
    // perturbation needs a program with a genuinely different selected
    // operation rather than another revision of the same one.
    let foreign = constant_receipt_with_unused_authority(None, 1, None);
    assert_ne!(subject.graph(), foreign.graph());
    let mut changed = subject.clone();
    changed.graph = foreign.graph().clone();
    assert_ne!(
        baseline,
        encode(
            &changed,
            &resolution,
            &realization,
            receipt.operand_bindings(),
            receipt.result_bindings(),
            &[]
        )
    );

    let mut changed = subject.clone();
    changed.occurrence = SemanticOccurrence::new(subject.occurrence().get() + 1);
    assert_ne!(
        baseline,
        encode(
            &changed,
            &resolution,
            &realization,
            receipt.operand_bindings(),
            receipt.result_bindings(),
            &[]
        )
    );

    let mut changed = subject.clone();
    changed.numerical_contract = alternate_test_contract();
    assert_ne!(
        baseline,
        encode(
            &changed,
            &resolution,
            &realization,
            receipt.operand_bindings(),
            receipt.result_bindings(),
            &[]
        )
    );

    let changed_region = VerifiedIndexRegionSequence::single(residual_region(1, 5, 0));
    assert_ne!(
        baseline,
        encode(
            &subject,
            &resolution,
            &changed_region,
            receipt.operand_bindings(),
            receipt.result_bindings(),
            &[]
        )
    );

    let mut changed = subject.clone();
    let law_row = changed
        .realization_law_row
        .as_mut()
        .expect("the reached operation carries a law row");
    let mut changed_law_row = law_row.to_vec();
    changed_law_row.push(0xff);
    *law_row = changed_law_row.into_boxed_slice();
    assert_ne!(
        baseline,
        encode(
            &changed,
            &resolution,
            &realization,
            receipt.operand_bindings(),
            receipt.result_bindings(),
            &[]
        )
    );

    let mut changed_resolution = resolution.clone();
    changed_resolution.provider =
        ProviderIdentity::new("test", "different-reached-law-provider", 1).unwrap();
    assert_ne!(
        baseline,
        encode(
            &subject,
            &changed_resolution,
            &realization,
            receipt.operand_bindings(),
            receipt.result_bindings(),
            &[]
        )
    );

    let mut changed_resolution = resolution.clone();
    changed_resolution.revision += 1;
    assert_ne!(
        baseline,
        encode(
            &subject,
            &changed_resolution,
            &realization,
            receipt.operand_bindings(),
            receipt.result_bindings(),
            &[]
        )
    );

    let mut operands = receipt.operand_bindings().to_vec();
    operands[0].operand += 1;
    assert_ne!(
        baseline,
        encode(
            &subject,
            &resolution,
            &realization,
            &operands,
            receipt.result_bindings(),
            &[]
        )
    );

    let mut results = receipt.result_bindings().to_vec();
    results[0].result += 1;
    assert_ne!(
        baseline,
        encode(
            &subject,
            &resolution,
            &realization,
            receipt.operand_bindings(),
            &results,
            &[]
        )
    );

    let proof_region = residual_region(1, 5, 0);
    let obligation = proof_region
        .unknown_index_domain_predicates()
        .next()
        .expect("the proof fixture retains one obligation");
    let authority = Arc::new(IndexDomainProofAuthority::exact_finite());
    let proof = IndexDomainProofEvidence::ExhaustiveFinite {
        points: 2,
        derivation: EXHAUSTIVE_DERIVATION.into(),
    };
    let proof = IndexRefinementDomainProof {
        stage: 0,
        obligation,
        authority: authority.clone(),
        identity: IndexRefinementDomainProofIdentity(
            encode_proof_identity(&proof_region, obligation, &authority, &proof).into_boxed_slice(),
        ),
        proof,
    };
    assert_ne!(
        baseline,
        encode(
            &subject,
            &resolution,
            &realization,
            receipt.operand_bindings(),
            receipt.result_bindings(),
            &[proof]
        )
    );
}

/// The digest is what separates two graphs at one occurrence ordinal.
///
/// ADR 0104 replaced the framed `SemanticGraphIdentity` at the head of every
/// coverage record with a fixed-width digest of it. The record's documented
/// claim — that it names "this occurrence of *this* graph" — then rests on
/// the digest rather than on a restatement, so this pins all three halves of
/// that: the preimage is gone from the bytes, the digest is present at the
/// exact position it left, and two graphs sharing one occurrence ordinal
/// still mint different coverage identities.
///
/// The neighbouring replay-and-substitution test already perturbs the graph
/// and watches the bytes move, and it would keep passing if the encoder had
/// written the graph identity whole. It is the *position* assertion here
/// that says which encoding produced the difference, which is the fact the
/// linear identity curve depends on.
#[test]
fn one_occurrence_of_two_graphs_is_separated_by_the_folded_graph_digest() {
    let (subject, resolution, realization, receipt) = reached_semantic_fixture(1);
    let encode = |subject: &IndexRefinementSubject| {
        encode_executable_coverage_identity(
            subject,
            &resolution,
            &realization,
            &receipt.scalar_authorities(),
            receipt.operand_bindings(),
            receipt.result_bindings(),
            &[],
        )
    };

    let baseline = encode(&subject);
    let head = EXECUTABLE_COVERAGE_IDENTITY_TAG.len();
    assert_eq!(
        &baseline[head..head + DIGEST_BYTES],
        DigestAlgorithm::GOVERNED
            .digest(COVERAGE_GRAPH_DIGEST_DOMAIN, subject.graph.as_bytes())
            .as_bytes(),
        "the record opens with the governed digest of its bound graph",
    );

    let graph_preimage = subject.graph.as_bytes();
    assert!(
        !baseline
            .windows(graph_preimage.len())
            .any(|window| window == graph_preimage),
        "the graph identity preimage still occurs in the coverage record",
    );

    // A second graph at the same occurrence ordinal. The constant fixture
    // selects a different operation, which is what makes its graph identity
    // genuinely different rather than another revision of one.
    let foreign = constant_receipt_with_unused_authority(None, 1, None);
    assert_ne!(subject.graph(), foreign.graph());
    let mut other = subject.clone();
    other.graph = foreign.graph().clone();
    let separated = encode(&other);
    assert_eq!(
        other.occurrence, subject.occurrence,
        "the ordinal is held fixed so the graph is the only thing that moved",
    );
    assert_ne!(
        baseline, separated,
        "two graphs at one occurrence ordinal minted equal coverage bytes",
    );
    assert_eq!(
        baseline[head + DIGEST_BYTES..],
        separated[head + DIGEST_BYTES..],
        "the graph digest is the only field that moved",
    );
}

fn test_law_operation() -> OpKey {
    OpKey::new("test", "refinement-law-row", 1).unwrap()
}

impl SemanticRegistryProvider for RefinementLawProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "refinement-law-provider", 1).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        let operation = test_law_operation();
        registrar.register_operation(OperationDefinition::new(
            operation.clone(),
            OperationSchema::new(OperationArity::exact(2), OperationArity::exact(1), []).unwrap(),
            NormativeDefinitionRef::new("test refinement-law-row v1")?,
            OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
            OperationConformance::new(CanonicalValue::boolean(true)),
            OperationEffect::Pure,
            Arc::new(BinaryIdentity),
        ))?;
        if let Some(law) = &self.0 {
            registrar.register_index_realization_law(operation, 1, law.clone())?;
        }
        Ok(())
    }
}

fn semantic_with_test_law(
    law: Option<super::super::IndexRealizationLaw>,
) -> FrozenSemanticRegistry {
    let mut builder = SemanticRegistryBuilder::standard().unwrap();
    builder
        .register_provider(&RefinementLawProvider(law))
        .unwrap();
    builder.freeze().unwrap()
}

// ---- The staged realization vocabulary -------------------------------
//
// These exercise a law form no standard operation carries: registering the
// normalization that will carry it belongs to that family's own ticket, and
// needs a governed reciprocal square root that does not yet exist. What is
// tested here is the vocabulary the family will be stated in — the ordered
// chain, its identity, the receipt that binds every stage, and the refusals.

/// Ordered axes attribute for the staged test operation's fold.
const STAGED_AXES_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);

/// Length of the folded axis in every staged fixture.
const STAGED_LENGTH: u64 = 4;

fn staged_test_operation() -> OpKey {
    OpKey::new("test", "staged-fold-then-pass", 1).unwrap()
}

/// Result type and shape follow the *second* operand, the elementwise one.
struct StagedFoldThenPass;

impl OperationInferencer for StagedFoldThenPass {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        if request.operands().len() == 2 {
            outputs.try_push(request.operands()[1].clone())
        } else {
            Err(OperationInferenceError::new(
                ProviderDiagnosticCode::new("test.staged.signature").unwrap(),
                "the staged test operation requires two operands",
            )
            .unwrap())
        }
    }
}

struct StagedLawProvider(Option<super::super::IndexRealizationLaw>);

impl SemanticRegistryProvider for StagedLawProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "staged-law-provider", 1).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        let operation = staged_test_operation();
        registrar.register_operation(OperationDefinition::new(
            operation.clone(),
            OperationSchema::new(
                OperationArity::exact(2),
                OperationArity::exact(1),
                [crate::semantic::OperationAttributeSchema::required(
                    STAGED_AXES_ATTRIBUTE,
                    CanonicalValueKind::Sequence,
                )],
            )
            .unwrap(),
            NormativeDefinitionRef::new("test staged-fold-then-pass v1")?,
            OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
            OperationConformance::new(CanonicalValue::boolean(true)),
            OperationEffect::Pure,
            Arc::new(StagedFoldThenPass),
        ))?;
        if let Some(law) = &self.0 {
            registrar.register_index_realization_law(operation, 1, law.clone())?;
        }
        Ok(())
    }
}

fn staged_law() -> super::super::IndexRealizationLaw {
    super::super::IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 {
        axes_attribute: STAGED_AXES_ATTRIBUTE,
        scalar: super::super::multiply_f32_scalar_op(),
    }
}

/// Complete authorities and subject for one staged-law occurrence.
struct StagedFixture {
    scalars: FrozenScalarRegistry,
    subject: IndexRefinementSubject,
    resolution: ResolvedIndexRealization,
    authority: IndexRealizationAuthority,
}

/// The two scalar operations the staged vocabulary reaches.
///
/// The fold's tail combine is the governed add and the pass applies the
/// governed multiply; neither the empty-domain constant nor the
/// single-contributor canonicalization is reachable at a folded extent above
/// one, so registering them would admit authority nothing here uses.
fn staged_scalars(semantic: &FrozenSemanticRegistry) -> FrozenScalarRegistry {
    let mut scalars = ScalarRegistryBuilder::new(semantic.clone());
    let provider = ProviderIdentity::new("test", "staged-scalars", 1).unwrap();
    for (key, normative) in [
        (
            super::super::multiply_f32_scalar_op(),
            "test staged multiply",
        ),
        (super::super::add_f32_scalar_op(), "test staged add"),
    ] {
        scalars
            .register(
                provider.clone(),
                test_binary_scalar_definition(key, normative),
            )
            .unwrap();
    }
    scalars.freeze()
}

fn staged_fixture(law: super::super::IndexRealizationLaw) -> StagedFixture {
    let mut builder = SemanticRegistryBuilder::standard().unwrap();
    builder
        .register_provider(&StagedLawProvider(Some(law)))
        .unwrap();
    let semantic = builder.freeze().unwrap();
    let scalars = staged_scalars(&semantic);

    let mut program = SemanticProgramBuilder::try_new(semantic.clone()).unwrap();
    let folded = program
        .input::<F32>(
            InputKey::new("folded").unwrap(),
            Shape::from_dims([STAGED_LENGTH]),
        )
        .unwrap();
    let elementwise = program
        .input::<F32>(
            InputKey::new("elementwise").unwrap(),
            Shape::from_dims([STAGED_LENGTH]),
        )
        .unwrap();
    let axes = CanonicalValue::sequence([CanonicalValue::unsigned_u32(0)]).unwrap();
    let value = program
        .apply(
            staged_test_operation(),
            OperationAttributes::new([CanonicalField::new(STAGED_AXES_ATTRIBUTE, axes)]).unwrap(),
            &[folded.erase(), elementwise.erase()],
        )
        .unwrap()
        .pop()
        .unwrap();
    program
        .output_resolved(OutputKey::new("scaled").unwrap(), value)
        .unwrap();
    let program = program.build().unwrap();
    let subject = IndexRefinementSubject::derive(
        &program,
        program.operations().next().unwrap().id(),
        test_contract(),
    )
    .unwrap();
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(semantic.clone(), scalars.clone())
        .unwrap();
    let resolution = laws.resolve(&subject).unwrap();
    let authority = IndexRealizationAuthority::admit(
        &semantic,
        &scalars,
        subject.operation().clone(),
        subject.signature().clone(),
        &[
            super::super::multiply_f32_scalar_op(),
            super::super::add_f32_scalar_op(),
        ],
    )
    .unwrap();
    StagedFixture {
        scalars,
        subject,
        resolution,
        authority,
    }
}

impl StagedFixture {
    fn realized(&self) -> VerifiedIndexRegionSequence {
        self.resolution
            .law
            .realize_sequence(&self.subject, &self.scalars)
            .expect("the staged law realizes its occurrence")
    }

    fn verify(
        &self,
        realization: &VerifiedIndexRegionSequence,
    ) -> Result<IndexRefinementVerificationOutcome, IndexRefinementVerificationError> {
        self.resolution
            .verify_sequence(&self.authority, realization)
    }
}

/// The whole point: an occurrence whose realization is two regions gets a
/// receipt that binds both of them.
#[test]
fn a_staged_occurrence_verifies_and_binds_every_region() {
    let fixture = staged_fixture(staged_law());
    let realization = fixture.realized();
    assert_eq!(realization.stage_count(), 2);
    assert_eq!(realization.intermediates().len(), 1);
    // The fold removed the only axis, so what it hands on is rank zero and
    // the pass reads it once per point.
    assert_eq!(realization.intermediates()[0].shape().rank(), 0);

    let IndexRefinementVerificationOutcome::Verified(receipt) = fixture
        .verify(&realization)
        .expect("the law's own realization verifies")
    else {
        panic!("the staged fixture retains no residual obligation")
    };
    assert_eq!(receipt.regions().len(), 2);
    assert_eq!(
        receipt.regions(),
        realization
            .stages()
            .map(|stage| stage.canonical_identity().clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(receipt.realization(), realization.identity());
    assert_eq!(
        receipt.final_stage(),
        realization.final_stage().canonical_identity()
    );
    // Both stages' scalar authorities are retained, and they genuinely
    // differ: at this folded extent the fold's tail combine reaches the
    // governed add and nothing else, and the pass reaches the multiply and
    // nothing else.
    assert_eq!(receipt.scalar_authorities().len(), 2);
    assert_ne!(
        receipt.scalar_authorities()[0],
        receipt.scalar_authorities()[1]
    );

    // The folded operand is read by the fold and the elementwise operand by
    // the pass, so the bindings name two different stages.
    let stages = receipt
        .operand_bindings()
        .iter()
        .map(|binding| (binding.operand(), binding.stage()))
        .collect::<Vec<_>>();
    assert_eq!(stages, vec![(0, 0), (1, 1)]);
    assert_eq!(receipt.result_bindings().len(), 1);

    // Domain separation, checked rather than only argued: a staged receipt
    // and its coverage are written under their own tags, so no staged
    // encoding can spell a single-region one — which is what lets the
    // one-stage encoding stay exactly the bytes it has always been.
    assert!(
        receipt
            .identity()
            .as_bytes()
            .starts_with(STAGED_RECEIPT_IDENTITY_TAG)
    );
    assert!(
        receipt
            .executable_coverage_identity()
            .as_bytes()
            .starts_with(STAGED_EXECUTABLE_COVERAGE_IDENTITY_TAG)
    );
    let (_, _, _, single_stage) = reached_semantic_fixture(1);
    assert!(
        single_stage
            .identity()
            .as_bytes()
            .starts_with(RECEIPT_IDENTITY_TAG)
    );
    assert!(
        single_stage
            .executable_coverage_identity()
            .as_bytes()
            .starts_with(EXECUTABLE_COVERAGE_IDENTITY_TAG)
    );
    assert!(!RECEIPT_IDENTITY_TAG.starts_with(STAGED_RECEIPT_IDENTITY_TAG));
    assert!(!STAGED_RECEIPT_IDENTITY_TAG.starts_with(RECEIPT_IDENTITY_TAG));
    assert!(!EXECUTABLE_COVERAGE_IDENTITY_TAG.starts_with(STAGED_EXECUTABLE_COVERAGE_IDENTITY_TAG));
    assert!(!STAGED_EXECUTABLE_COVERAGE_IDENTITY_TAG.starts_with(EXECUTABLE_COVERAGE_IDENTITY_TAG));
}

/// A staged realization's containment check covers every stage.
///
/// Admitting only the multiply the *pass* reaches leaves the fold's own
/// governed additions unadmitted, and the realization is refused as a whole.
#[test]
fn an_unadmitted_scalar_in_an_earlier_stage_refuses_the_realization() {
    let fixture = staged_fixture(staged_law());
    let realization = fixture.realized();
    let narrow = IndexRealizationAuthority::admit(
        &FrozenSemanticRegistry::standard().unwrap(),
        &fixture.scalars,
        fixture.subject.operation().clone(),
        fixture.subject.signature().clone(),
        &[super::super::multiply_f32_scalar_op()],
    );
    // The narrow authority is built over the standard registry, which does
    // not define the staged test operation at all, so admission itself is
    // what refuses first; rebuild it over the fixture's own authority.
    assert!(narrow.is_err());

    let mut builder = SemanticRegistryBuilder::standard().unwrap();
    builder
        .register_provider(&StagedLawProvider(Some(staged_law())))
        .unwrap();
    let semantic = builder.freeze().unwrap();
    let narrow = IndexRealizationAuthority::admit(
        &semantic,
        &fixture.scalars,
        fixture.subject.operation().clone(),
        fixture.subject.signature().clone(),
        &[super::super::multiply_f32_scalar_op()],
    )
    .unwrap();
    assert_eq!(
        fixture
            .resolution
            .verify_sequence(&narrow, &realization)
            .unwrap_err(),
        IndexRefinementVerificationError::ScalarAuthorityConformance
    );
}

/// The rubber-stamp perturbation: a well-formed chain that realizes
/// something else is refused.
///
/// Both candidates below are structurally valid region sequences —
/// [`VerifiedIndexRegionSequence::try_new`] accepted them — so nothing about
/// their own construction says no. What refuses is the comparison against
/// the law's own realization, which is the only thing standing between a
/// receipt and a provider that emitted a plausible chain for the wrong
/// operation.
#[test]
fn a_chain_that_does_not_realize_the_occurrence_is_refused() {
    let fixture = staged_fixture(staged_law());
    let realization = fixture.realized();

    // Cross-family: the chain built for the *other* scalar. Every stage is
    // well formed and the wiring is identical; only the pass's arithmetic
    // differs, and that is enough.
    let other = staged_fixture(
        super::super::IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 {
            axes_attribute: STAGED_AXES_ATTRIBUTE,
            scalar: super::super::add_f32_scalar_op(),
        },
    );
    let foreign = other.realized();
    assert_ne!(realization.identity(), foreign.identity());
    let refusal = fixture.verify(&foreign).unwrap_err();
    assert!(
        matches!(
            refusal,
            IndexRefinementVerificationError::SemanticRealizationSequenceMismatch { .. }
        ),
        "observed {refusal:?}"
    );

    // Wrong order: the same two regions, chained the other way round. The
    // reversal composes — `try_new` accepts it — but running the pass first
    // means the occurrence's folded operand would have to be read through a
    // boundary shaped like the fold's *result*, and the ordered interface
    // check reaches that one statement before the identity comparison does.
    //
    // Asserted at the exact position rather than "some refusal": the
    // boundary that disagrees is the evidence the order was wrong, and a
    // test satisfied by any refusal would pass for a fixture that had
    // stopped building chains at all.
    let stages = realization.stages().cloned().collect::<Vec<_>>();
    let reversed = VerifiedIndexRegionSequence::try_new(
        vec![stages[1].clone(), stages[0].clone()],
        vec![
            vec![
                StagedInputSource::Occurrence(1),
                StagedInputSource::Occurrence(0),
            ],
            vec![StagedInputSource::Intermediate(0)],
        ],
    )
    .expect("the reversed chain is structurally well formed");
    assert_ne!(realization.identity(), reversed.identity());
    assert_eq!(
        fixture.verify(&reversed).unwrap_err(),
        IndexRefinementVerificationError::OperandInterface { position: 0 }
    );
}

/// One region for a two-region occurrence, and two for a one-region one.
///
/// The ticket's own perturbation, in both directions: a chain cannot be
/// presented for a law that declares one region, and one region cannot
/// certify a law whose realization is a chain.
///
/// **Where each direction is caught differs, and that is worth recording.**
/// A truncated chain drops the fold, so the pass's handed input boundary now
/// claims to be an occurrence input and disagrees with it — the ordered
/// interface check names that boundary before the realization comparison
/// runs. A chain presented for a one-region law binds cleanly and is caught
/// by the comparison itself. Both are typed refusals and neither mints a
/// receipt; what would be wrong is a candidate that reached one of these
/// paths and was approved by the other.
#[test]
fn region_count_disagreements_refuse_in_both_directions() {
    let fixture = staged_fixture(staged_law());
    let realization = fixture.realized();
    let stages = realization.stages().cloned().collect::<Vec<_>>();

    // A staged law against the pass alone. Its second boundary is the fold's
    // rank-zero result, which the occurrence's second input is not.
    let truncated = VerifiedIndexRegionSequence::single(stages[1].clone());
    assert_eq!(
        fixture.verify(&truncated).unwrap_err(),
        IndexRefinementVerificationError::OperandInterface { position: 1 }
    );
    // And through the single-region entry point, where the law refuses
    // before any comparison is reached. Not the compiler's path — it drives
    // `verify_sequence` — but a public one, so the refusal is reachable.
    let refusal = fixture
        .resolution
        .verify(&fixture.authority, &stages[1])
        .unwrap_err();
    assert!(
        matches!(
            refusal,
            IndexRefinementVerificationError::SemanticRealizationLawRefused {
                rule: "staged-law-requires-region-sequence",
                ..
            }
        ),
        "observed {refusal:?}"
    );

    // A single-region law against a two-region candidate.
    let semantic = semantic_with_test_law(Some(super::super::IndexRealizationLaw::multiply_f32()));
    let scalars = staged_scalars(&semantic);
    let mut program = SemanticProgramBuilder::try_new(semantic.clone()).unwrap();
    let input = program
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([1]))
        .unwrap();
    let value = program
        .apply(
            test_law_operation(),
            OperationAttributes::empty(),
            &[input.erase(), input.erase()],
        )
        .unwrap()
        .pop()
        .unwrap();
    program
        .output_resolved(OutputKey::new("output").unwrap(), value)
        .unwrap();
    let program = program.build().unwrap();
    let subject = IndexRefinementSubject::derive(
        &program,
        program.operations().next().unwrap().id(),
        test_contract(),
    )
    .unwrap();
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(semantic.clone(), scalars.clone())
        .unwrap();
    let resolution = laws.resolve(&subject).unwrap();
    let authority = IndexRealizationAuthority::admit(
        &semantic,
        &scalars,
        subject.operation().clone(),
        subject.signature().clone(),
        &[super::super::multiply_f32_scalar_op()],
    )
    .unwrap();
    let single = super::super::IndexRealizationLaw::multiply_f32()
        .realize(&subject, &scalars)
        .unwrap();
    // The one-stage candidate the law does expect still verifies, which is
    // what makes the two-stage refusal below attributable to stage count
    // rather than to a broken fixture.
    assert!(matches!(
        resolution
            .verify_sequence(
                &authority,
                &VerifiedIndexRegionSequence::single(single.clone())
            )
            .unwrap(),
        IndexRefinementVerificationOutcome::Verified(_)
    ));
    // The operation aliases one input into both operands, so its region has
    // one input boundary; running the region twice, the second copy reading
    // the first's result, is a chain whose every interface agrees with the
    // occurrence. Nothing but the whole-realization comparison can refuse
    // it, which is exactly what makes it the case worth stating.
    let doubled = VerifiedIndexRegionSequence::try_new(
        vec![single.clone(), single],
        vec![
            vec![StagedInputSource::Occurrence(0)],
            vec![StagedInputSource::Intermediate(0)],
        ],
    )
    .expect("squaring twice, the second reading the first, is a well-formed chain");
    let refusal = resolution
        .verify_sequence(&authority, &doubled)
        .unwrap_err();
    assert!(
        matches!(
            refusal,
            IndexRefinementVerificationError::SemanticRealizationSequenceMismatch { .. }
        ),
        "observed {refusal:?}"
    );
}

#[test]
fn operation_specific_law_rows_are_checked_across_public_registry_boundaries() {
    let semantic_a =
        semantic_with_test_law(Some(super::super::IndexRealizationLaw::multiply_f32()));
    let semantic_b = semantic_with_test_law(Some(super::super::IndexRealizationLaw::add_f32()));
    let semantic_absent = semantic_with_test_law(None);
    assert_eq!(
        semantic_a.snapshot_identity(),
        semantic_b.snapshot_identity()
    );
    assert_eq!(
        semantic_a.snapshot_identity(),
        semantic_absent.snapshot_identity()
    );
    let scalars_a = ScalarRegistryBuilder::new(semantic_a.clone()).freeze();
    let scalars_b = ScalarRegistryBuilder::new(semantic_b.clone()).freeze();
    let scalars_absent = ScalarRegistryBuilder::new(semantic_absent.clone()).freeze();
    assert_eq!(scalars_a.snapshot_identity(), scalars_b.snapshot_identity());
    assert_eq!(
        scalars_a.snapshot_identity(),
        scalars_absent.snapshot_identity()
    );
    let mut program = SemanticProgramBuilder::try_new(semantic_a.clone()).unwrap();
    let input = program
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([1]))
        .unwrap();
    let value = program
        .apply(
            test_law_operation(),
            OperationAttributes::empty(),
            &[input.erase(), input.erase()],
        )
        .unwrap()
        .pop()
        .unwrap();
    program
        .output_resolved(OutputKey::new("output").unwrap(), value)
        .unwrap();
    let program = program.build().unwrap();
    let subject = IndexRefinementSubject::derive(
        &program,
        program.operations().next().unwrap().id(),
        test_contract(),
    )
    .unwrap();
    let laws_a = FrozenIndexRealizationLawRegistry::from_semantic(semantic_a, scalars_a).unwrap();
    for (semantic, scalars) in [
        (semantic_b.clone(), scalars_b.clone()),
        (semantic_absent, scalars_absent),
    ] {
        let laws = FrozenIndexRealizationLawRegistry::from_semantic(semantic, scalars).unwrap();
        assert!(matches!(
            laws.resolve(&subject),
            Err(IndexRefinementVerificationError::SubjectRealizationLawMismatch)
        ));
    }
    let resolution = laws_a.resolve(&subject).unwrap();
    let signature = subject.signature().clone();
    let lowering = IndexRealizationAuthority::admit(
        &semantic_b,
        &scalars_b,
        test_law_operation(),
        signature,
        &[],
    )
    .unwrap();
    assert_eq!(
        check_lowering_authority(&subject, &resolution, &lowering),
        Err(IndexRefinementVerificationError::SubjectRealizationLawMismatch)
    );
}

fn residual_region(second_extent: u64, rounds: usize, offset: i128) -> VerifiedIndexRegion {
    residual_region_with_extents(
        &[LENGTH, second_extent],
        0,
        rounds,
        1_i128.into(),
        offset.into(),
    )
}

fn residual_region_with_extents(
    extents: &[u64],
    target_axis: usize,
    rounds: usize,
    multiplier: IndexInteger,
    offset: IndexInteger,
) -> VerifiedIndexRegion {
    let mut builder = IndexRegionBuilder::new(FrozenScalarRegistry::standard().unwrap())
        .expect("the fixture receives a fresh builder identity");
    let dimensions = extents
        .iter()
        .map(|extent| {
            builder
                .dimension(DomainRole::Parallel, Extent::new(*extent))
                .unwrap()
        })
        .collect::<Vec<_>>();
    let shape = Shape::try_from_dims(extents.iter().copied()).unwrap();
    let value_type = ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap());
    let input = builder
        .tensor(TensorRole::Input, value_type.clone(), shape.clone())
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, value_type, shape)
        .unwrap();
    let coordinates = dimensions
        .iter()
        .map(|dimension| builder.dimension_expr(*dimension).unwrap())
        .collect::<Vec<_>>();
    let mut conservative = coordinates[target_axis];
    for _ in 0..rounds {
        let two = SourcedExtent::Static(Extent::new(2));
        let modulo = builder.modulo(conservative, two.clone()).unwrap();
        let quotient = builder.floor_div(conservative, two).unwrap();
        conservative = builder
            .linear_combination(
                0_i128.into(),
                &[(2_i128.into(), quotient), (1_i128.into(), modulo)],
            )
            .unwrap();
    }
    if multiplier != 1_i128.into() {
        conservative = builder
            .linear_combination(0_i128.into(), &[(multiplier, conservative)])
            .unwrap();
    }
    if offset != 0_i128.into() {
        conservative = builder
            .linear_combination(offset, &[(1_i128.into(), conservative)])
            .unwrap();
    }
    let mut read_coordinates = coordinates.clone();
    read_coordinates[target_axis] = conservative;
    let value = builder.read(input, &dimensions, &read_coordinates).unwrap();
    let write = builder.write(output, &dimensions, &coordinates).unwrap();
    builder.output(write, value).unwrap();
    let region = builder.build().unwrap();
    assert_eq!(region.unknown_index_domain_predicates().len(), 1);
    region
}

fn two_domain_residual_region() -> VerifiedIndexRegion {
    let mut builder = IndexRegionBuilder::new(FrozenScalarRegistry::standard().unwrap())
        .expect("the fixture receives a fresh builder identity");
    let value_type = f32_type();
    let mut dimensions = Vec::new();
    let mut coordinates = Vec::new();
    let mut values = Vec::new();
    for _ in 0..2 {
        let dimension = builder
            .dimension(DomainRole::Parallel, Extent::new(LENGTH))
            .unwrap();
        let shape = Shape::from_dims([LENGTH]);
        let input = builder
            .tensor(TensorRole::Input, value_type.clone(), shape.clone())
            .unwrap();
        let coordinate = builder.dimension_expr(dimension).unwrap();
        let mut conservative = coordinate;
        for _ in 0..5 {
            let two = SourcedExtent::Static(Extent::new(2));
            let modulo = builder.modulo(conservative, two.clone()).unwrap();
            let quotient = builder.floor_div(conservative, two).unwrap();
            conservative = builder
                .linear_combination(
                    0_i128.into(),
                    &[(2_i128.into(), quotient), (1_i128.into(), modulo)],
                )
                .unwrap();
        }
        let value = builder.read(input, &[dimension], &[conservative]).unwrap();
        dimensions.push(dimension);
        coordinates.push(coordinate);
        values.push(value);
    }
    let sum = builder
        .apply(
            super::super::add_f32_scalar_op(),
            super::super::ScalarAttributes::empty(),
            &values,
        )
        .unwrap();
    let output = builder
        .tensor(
            TensorRole::Output,
            value_type,
            Shape::from_dims([LENGTH, LENGTH]),
        )
        .unwrap();
    let write = builder.write(output, &dimensions, &coordinates).unwrap();
    builder.output(write, sum.get(0).unwrap()).unwrap();
    let region = builder.build().unwrap();
    assert_eq!(region.unknown_index_domain_predicates().len(), 2);
    region
}

fn assess(region: &VerifiedIndexRegion, cells: u64, integer_bytes: u64) -> IndexDomainProofClaim {
    let obligation = region
        .unknown_index_domain_predicates()
        .next()
        .expect("the fixture retains one residual");
    assess_finite_domains(
        region,
        &[obligation],
        IndexDomainProofBudget::try_new(cells, integer_bytes).unwrap(),
    )
    .pop()
    .unwrap()
}

#[test]
fn exact_finite_evaluation_refuses_when_conservative_work_exceeds_hard_limit() {
    let claim = assess(
        &residual_region(1, 5, 0),
        MAX_FINITE_DOMAIN_PROOF_CELLS,
        MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
    );
    assert!(matches!(
        claim,
        IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
            resource: super::super::ProofResource::IntegerBytes,
            required,
            limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        }) if required > u128::from(MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES)
    ));
}

#[test]
fn exact_finite_evaluation_returns_the_first_counterexample() {
    let region = residual_region(1, 5, 1);
    let first = assess(
        &region,
        MAX_FINITE_DOMAIN_PROOF_CELLS,
        MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
    );
    let second = assess(
        &region,
        MAX_FINITE_DOMAIN_PROOF_CELLS,
        MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
    );
    assert_eq!(first, second);
    assert!(matches!(
        first,
        IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
            resource: super::super::ProofResource::IntegerBytes,
            required,
            limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        }) if required > u128::from(MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES)
    ));
}

#[test]
fn wide_counterexample_is_encoded_by_exact_point_ordinal() {
    let encoded = encode_counterexample(0);
    assert!(encoded.len() <= MAX_DOMAIN_EVIDENCE_BYTES);
    assert_eq!(&encoded[..COUNTEREXAMPLE_TAG.len()], COUNTEREXAMPLE_TAG);
}

#[test]
fn an_empty_domain_is_vacuously_proved_before_overflowing_prefixes() {
    assert_eq!(finite_point_count(&[u64::MAX, u64::MAX, 0]), Some(0));
}

#[test]
fn exact_finite_evaluation_fails_closed_at_the_callers_budget() {
    let claim = assess(
        &residual_region(1, 5, 0),
        1,
        MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
    );
    assert!(matches!(
        claim,
        IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
            resource: super::super::ProofResource::Cells,
            required,
            limit: 1,
        }) if required > 1
    ));
}

#[test]
fn exact_finite_evaluation_charges_integer_byte_work() {
    let claim = assess(&residual_region(1, 5, 0), MAX_FINITE_DOMAIN_PROOF_CELLS, 1);
    assert!(matches!(
        claim,
        IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
            resource: super::super::ProofResource::IntegerBytes,
            required,
            limit: 1,
        }) if required > 1
    ));
}

#[test]
fn linear_integer_work_reports_one_exact_preflight_charge() {
    let region = residual_region(1, 5, 0);
    let obligation = region.unknown_index_domain_predicates().next().unwrap();
    let required = match assess_finite_domains(
        &region,
        &[obligation],
        IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, 1).unwrap(),
    )[0]
    {
        IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
            resource: super::super::ProofResource::IntegerBytes,
            required,
            ..
        }) => u64::try_from(required).unwrap(),
        ref claim => panic!("one-byte perturbation did not expose charge: {claim:?}"),
    };
    assert!(required > MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES);
}

#[test]
fn whole_call_ledger_fills_every_obligation_when_the_first_group_exhausts() {
    let region = two_domain_residual_region();
    let obligations = region.unknown_index_domain_predicates().collect::<Vec<_>>();
    let claims = assess_finite_domains(
        &region,
        &obligations,
        IndexDomainProofBudget::try_new(
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        )
        .unwrap(),
    );
    assert!(claims.iter().all(|claim| matches!(
        claim,
        IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
            resource: super::super::ProofResource::IntegerBytes,
            required,
            limit,
        }) if *required > u128::from(MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES)
            && *limit == MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES
    )));
}

#[test]
fn whole_call_ledger_preserves_an_earlier_group_before_later_exhaustion() {
    // The builder discharges the one-point predicate immediately, so this
    // test states that valid group explicitly to isolate the shared ledger.
    // Its subject, dimension, expression, and bound all belong to the
    // verified earlier region; the later group is the retained residual of
    // a second verified region, as completion encounters across stages.
    let mut builder = IndexRegionBuilder::new(FrozenScalarRegistry::standard().unwrap())
        .expect("the fixture receives a fresh builder identity");
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(1))
        .unwrap();
    let tensor = builder
        .tensor(TensorRole::Input, f32_type(), Shape::from_dims([1]))
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, f32_type(), Shape::from_dims([1]))
        .unwrap();
    let coordinate = builder.dimension_expr(dimension).unwrap();
    let value = builder.read(tensor, &[dimension], &[coordinate]).unwrap();
    let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
    builder.output(write, value).unwrap();
    let earlier_region = builder.build().unwrap();
    let earlier_access_ref = earlier_region.accesses().next().unwrap();
    let earlier_access = earlier_access_ref.id();
    let dimension = earlier_access_ref.domain().next().unwrap();
    let coordinate = earlier_access_ref.coordinates().next().unwrap();
    let earlier_obligation = UnknownIndexDomainPredicate {
        subject: earlier_access,
        predicate: IndexDomainPredicate::LessThanExtent {
            expression: coordinate,
            extent: IndexExtentRef::Dimension(dimension),
        },
        reason: IndexDomainUnknownReason::InsufficientFacts,
    };
    let earlier_group = IndexDomainGroup {
        domain: IndexDomainKey(vec![(dimension, 1)]),
        points: 1,
        obligations: vec![PlannedDomainObligation {
            slot: 0,
            obligation: earlier_obligation,
            upper_bound: Some(1),
        }],
    };
    let later_region = residual_region(1, 5, 0);
    let later_obligation = later_region
        .unknown_index_domain_predicates()
        .next()
        .unwrap();
    let later_dimension = later_region
        .access(later_obligation.subject())
        .unwrap()
        .domain()
        .next()
        .unwrap();
    let later_bound = match later_obligation.predicate() {
        IndexDomainPredicate::LessThanExtent { extent, .. } => {
            resolve_extent(&later_region, extent).unwrap()
        }
        IndexDomainPredicate::NonNegative { .. } => {
            panic!("fixture must retain an upper bound")
        }
    };
    let later_group = IndexDomainGroup {
        domain: IndexDomainKey(vec![(later_dimension, LENGTH)]),
        points: LENGTH,
        obligations: vec![PlannedDomainObligation {
            slot: 1,
            obligation: later_obligation,
            upper_bound: Some(later_bound),
        }],
    };
    let mut ledger = IndexDomainProofLedger::new(
        IndexDomainProofBudget::try_new(
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        )
        .unwrap(),
    );
    let earlier = assess_domain_group(&earlier_region, &earlier_group, &mut ledger).unwrap();
    assert!(
        matches!(earlier.as_slice(), [IndexDomainProofClaim::Proved(_)]),
        "the earlier group must finish: {earlier:?}"
    );
    let Err(ProofPlanningFailure::Exhausted(exhaustion)) =
        assess_domain_group(&later_region, &later_group, &mut ledger)
    else {
        panic!("the later group must exhaust the shared ledger")
    };
    let later = proof_resource_limit(exhaustion);
    assert!(matches!(
        later,
        IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
            resource: super::super::ProofResource::IntegerBytes,
            required,
            limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        }) if required > u128::from(MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES)
    ));
}

#[test]
fn disproof_precedes_later_resource_limit_and_retains_both_assessments() {
    // Classification is deliberately tested apart from evaluation. The
    // evaluator's exact disproof construction is covered at its group
    // boundary; this population pins the whole-call rule that consumes the
    // already-produced claims without making either disappear.
    let region = two_domain_residual_region();
    let obligations = region.unknown_index_domain_predicates().collect::<Vec<_>>();
    let authority = Arc::new(IndexDomainProofAuthority::exact_finite());
    let assessments = vec![
        IndexDomainProofAssessment {
            obligation: obligations[0],
            authority: authority.clone(),
            claim: IndexDomainProofClaim::Disproved(
                IndexDomainDisproof::new("test-counterexample", encode_counterexample(0))
                    .unwrap()
                    .with_point_ordinal(0),
            ),
        },
        IndexDomainProofAssessment {
            obligation: obligations[1],
            authority,
            claim: IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
                resource: super::super::ProofResource::IntegerBytes,
                required: u128::from(MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES) + 1,
                limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            }),
        },
    ];
    let refusal = retain_complete_assessments(assessments)
        .expect_err("an assessed counterexample must refuse completion");
    assert_eq!(refusal.kind(), IndexDomainProofRefusalKind::Disproved);
    assert!(matches!(
        refusal.assessments(),
        [
            IndexDomainProofAssessment {
                claim: IndexDomainProofClaim::Disproved(_),
                ..
            },
            IndexDomainProofAssessment {
                claim: IndexDomainProofClaim::Unknown(
                    IndexDomainUnknownReason::ResourceLimit { .. }
                ),
                ..
            }
        ]
    ));
}

#[test]
fn unsupported_root_does_not_poison_same_group_siblings() {
    let region = two_domain_residual_region();
    let obligations = region.unknown_index_domain_predicates().collect::<Vec<_>>();
    let second_dimension = region
        .access(obligations[1].subject())
        .unwrap()
        .domain()
        .next()
        .unwrap();
    let second_bound = match obligations[1].predicate() {
        IndexDomainPredicate::LessThanExtent { extent, .. } => {
            resolve_extent(&region, extent).unwrap()
        }
        IndexDomainPredicate::NonNegative { .. } => panic!("fixture must retain upper bound"),
    };
    let group = IndexDomainGroup {
        domain: IndexDomainKey(vec![(second_dimension, 1)]),
        points: 1,
        obligations: vec![
            PlannedDomainObligation {
                slot: 0,
                obligation: obligations[0],
                upper_bound: Some(second_bound),
            },
            PlannedDomainObligation {
                slot: 1,
                obligation: obligations[1],
                upper_bound: Some(second_bound),
            },
            PlannedDomainObligation {
                slot: 2,
                obligation: obligations[1],
                upper_bound: Some(0),
            },
        ],
    };
    let mut ledger = IndexDomainProofLedger::new(
        IndexDomainProofBudget::try_new(
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        )
        .unwrap(),
    );
    let claims = assess_domain_group(&region, &group, &mut ledger).unwrap();
    assert!(matches!(
        claims.as_slice(),
        [
            IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment),
            IndexDomainProofClaim::Proved(_),
            IndexDomainProofClaim::Disproved(_),
        ]
    ));
}

#[test]
fn all_unsupported_group_skips_large_domain_evaluation_reservation() {
    let region = two_domain_residual_region();
    let obligation = region.unknown_index_domain_predicates().next().unwrap();
    let group = IndexDomainGroup {
        domain: IndexDomainKey(Vec::new()),
        points: u64::MAX,
        obligations: vec![PlannedDomainObligation {
            slot: 0,
            obligation,
            upper_bound: Some(LENGTH),
        }],
    };
    let mut ledger = IndexDomainProofLedger::new(IndexDomainProofBudget::try_new(128, 1).unwrap());
    let claims = assess_domain_group(&region, &group, &mut ledger).unwrap();
    assert!(matches!(
        claims.as_slice(),
        [IndexDomainProofClaim::Unknown(
            IndexDomainUnknownReason::UnsupportedFragment
        )]
    ));
    assert_eq!(ledger.used_integer_bytes, 0);
    assert!(ledger.exhaustion.is_none());
}

#[test]
fn manageable_shared_dag_has_exact_grouped_and_minus_one_charges() {
    let region = two_domain_residual_region();
    let obligation = region.unknown_index_domain_predicates().next().unwrap();
    let access = region.access(obligation.subject()).unwrap();
    let dimension = access.domain().next().unwrap();
    let upper_bound = match obligation.predicate() {
        IndexDomainPredicate::LessThanExtent { extent, .. } => {
            resolve_extent(&region, extent).unwrap()
        }
        IndexDomainPredicate::NonNegative { .. } => panic!("fixture must retain upper bound"),
    };
    let make_group = |copies: usize| IndexDomainGroup {
        domain: IndexDomainKey(vec![(dimension, LENGTH)]),
        points: 1,
        obligations: (0..copies)
            .map(|slot| PlannedDomainObligation {
                slot,
                obligation,
                upper_bound: Some(upper_bound),
            })
            .collect(),
    };
    let required = |copies| {
        let mut ledger = IndexDomainProofLedger::new(
            IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, 1).unwrap(),
        );
        let Err(ProofPlanningFailure::Exhausted(exhaustion)) =
            assess_domain_group(&region, &make_group(copies), &mut ledger)
        else {
            panic!("one-byte budget must expose exact grouped charge")
        };
        u64::try_from(exhaustion.required).unwrap()
    };
    let grouped = required(2);
    let separate = required(1).checked_mul(2).unwrap();
    assert!(grouped < separate);
    let mut exact = IndexDomainProofLedger::new(
        IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, grouped).unwrap(),
    );
    assert!(assess_domain_group(&region, &make_group(2), &mut exact).is_ok());
    let mut short = IndexDomainProofLedger::new(
        IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, grouped - 1).unwrap(),
    );
    assert!(matches!(
        assess_domain_group(&region, &make_group(2), &mut short),
        Err(ProofPlanningFailure::Exhausted(IndexDomainProofExhaustion {
            resource: super::super::ProofResource::IntegerBytes,
            required,
            limit,
        })) if required == u128::from(grouped) && limit == grouped - 1
    ));
}

#[test]
fn equivalent_authoring_orders_retain_directional_canonical_occurrences() {
    let build = |reverse: bool| {
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let (one, two) = if reverse {
            let two = F32Constant::apply(&mut program, 2.0_f32.to_bits()).unwrap();
            let one = F32Constant::apply(&mut program, 1.0_f32.to_bits()).unwrap();
            (one, two)
        } else {
            let one = F32Constant::apply(&mut program, 1.0_f32.to_bits()).unwrap();
            let two = F32Constant::apply(&mut program, 2.0_f32.to_bits()).unwrap();
            (one, two)
        };
        program.output(OutputKey::new("one").unwrap(), one).unwrap();
        program.output(OutputKey::new("two").unwrap(), two).unwrap();
        program.build().unwrap()
    };
    let receipt = |program: &SemanticProgram, storage: usize| {
        let semantic = FrozenSemanticRegistry::standard().unwrap();
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic.clone(), scalars.clone())
                .unwrap();
        let operation = program.operations().nth(storage).unwrap().id();
        let subject = IndexRefinementSubject::derive(program, operation, test_contract()).unwrap();
        let authority = IndexRealizationAuthority::admit(
            &semantic,
            &scalars,
            subject.operation().clone(),
            subject.signature().clone(),
            &[super::super::constant_f32_scalar_op()],
        )
        .unwrap();
        let resolution = laws.resolve(&subject).unwrap();
        let region = super::super::IndexRealizationLaw::constant_f32()
            .realize(&subject, &scalars)
            .unwrap();
        let IndexRefinementVerificationOutcome::Verified(receipt) =
            resolution.verify(&authority, &region).unwrap()
        else {
            panic!("a rank-zero constant retains no residual obligation")
        };
        receipt
    };

    let forward = build(false);
    let reversed = build(true);
    assert_eq!(
        forward.semantic_identity().graph(),
        reversed.semantic_identity().graph()
    );
    assert_ne!(
        forward.operations().next().unwrap().id(),
        reversed.operations().nth(1).unwrap().id(),
        "the same named operation is selected by graph-owned handles, not a shared ordinal"
    );

    // `one` is storage operation 0 in the forward graph and 1 in the
    // reversed graph; `two` moves in the opposite direction. Compare each
    // direction explicitly so a crossed mapping cannot be sorted away.
    let forward_one = receipt(&forward, 0);
    let forward_two = receipt(&forward, 1);
    let reversed_two = receipt(&reversed, 0);
    let reversed_one = receipt(&reversed, 1);
    assert_eq!(forward_one.occurrence(), reversed_one.occurrence());
    assert_eq!(forward_one.identity(), reversed_one.identity());
    assert_eq!(
        forward_one.executable_coverage_identity(),
        reversed_one.executable_coverage_identity()
    );
    assert_eq!(forward_two.occurrence(), reversed_two.occurrence());
    assert_eq!(forward_two.identity(), reversed_two.identity());
    assert_eq!(
        forward_two.executable_coverage_identity(),
        reversed_two.executable_coverage_identity()
    );
    assert_ne!(forward_one.occurrence(), forward_two.occurrence());
    assert_ne!(forward_one.identity(), forward_two.identity());
    assert_ne!(
        forward_one.executable_coverage_identity(),
        forward_two.executable_coverage_identity()
    );

    let other = IndexRefinementSubject::derive(
        &forward,
        forward.operations().nth(1).unwrap().id(),
        test_contract(),
    )
    .unwrap();
    assert_eq!(other.occurrence(), forward_two.occurrence());
    assert_ne!(other.occurrence(), forward_one.occurrence());

    let foreign = reversed.operations().next().unwrap().id();
    assert!(matches!(
        IndexRefinementSubject::derive(&forward, foreign, test_contract()),
        Err(IndexRefinementVerificationError::SemanticHandle(
            crate::semantic::HandleError::ForeignGraph {
                entity: crate::semantic::EntityKind::Operation
            }
        ))
    ));
}

#[test]
fn v2_subject_domain_separates_the_v1_storage_ordinal_collision() {
    let build = |reverse: bool| {
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let first = F32Constant::apply(&mut program, 1.0_f32.to_bits()).unwrap();
        let second = F32Constant::apply(&mut program, 1.0_f32.to_bits()).unwrap();
        let (alpha, beta) = if reverse {
            (second, first)
        } else {
            (first, second)
        };
        program
            .output(OutputKey::new("alpha").unwrap(), alpha)
            .unwrap();
        program
            .output(OutputKey::new("beta").unwrap(), beta)
            .unwrap();
        program.build().unwrap()
    };
    let forward = build(false);
    let reversed = build(true);
    let forward_subject = IndexRefinementSubject::derive(
        &forward,
        forward.operations().next().unwrap().id(),
        test_contract(),
    )
    .unwrap();
    let reversed_subject = IndexRefinementSubject::derive(
        &reversed,
        reversed.operations().next().unwrap().id(),
        test_contract(),
    )
    .unwrap();
    assert_eq!(
        forward.semantic_identity().graph(),
        reversed.semantic_identity().graph()
    );
    assert_ne!(
        forward_subject.occurrence(),
        reversed_subject.occurrence(),
        "fixed output names distinguish two otherwise identical occurrences canonically"
    );

    let storage_zero = SemanticOccurrence::new(0);
    let old_forward =
        encode_subject_identity_with(&forward_subject, LEGACY_SUBJECT_IDENTITY_TAG, storage_zero);
    let old_reversed =
        encode_subject_identity_with(&reversed_subject, LEGACY_SUBJECT_IDENTITY_TAG, storage_zero);
    assert_eq!(
        old_forward, old_reversed,
        "v1 gave storage occurrence zero one byte spelling for two canonical occurrences"
    );
    assert_ne!(forward_subject.identity, reversed_subject.identity);
    assert!(forward_subject.identity.starts_with(SUBJECT_IDENTITY_TAG));
    assert!(
        !forward_subject
            .identity
            .starts_with(LEGACY_SUBJECT_IDENTITY_TAG)
    );
}

#[test]
fn wide_program_derives_all_occurrences_from_one_linear_cache() {
    const OPERATIONS: usize = 1_024;
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    for ordinal in 0..OPERATIONS {
        let value = F32Constant::apply(&mut builder, u32::try_from(ordinal).unwrap()).unwrap();
        builder
            .output(
                OutputKey::new(format!("value-{ordinal:04}")).unwrap(),
                value,
            )
            .unwrap();
    }
    let program = builder.build().unwrap();
    assert_eq!(program.canonical_operation_ordinal_count(), OPERATIONS);
    let subjects = program
        .operations()
        .map(|operation| {
            IndexRefinementSubject::derive(&program, operation.id(), test_contract()).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(subjects.len(), OPERATIONS);
    let mut occurrences = subjects
        .iter()
        .map(IndexRefinementSubject::occurrence)
        .collect::<Vec<_>>();
    occurrences.sort_unstable();
    occurrences.dedup();
    assert_eq!(occurrences.len(), OPERATIONS);
}

#[test]
fn completion_receipts_cannot_be_cross_wired_between_real_occurrences() {
    let mut program = SemanticProgramBuilder::try_standard().unwrap();
    let input = program
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([LENGTH]))
        .unwrap();
    let first_value = F32Multiply::apply(&mut program, input, input).unwrap();
    let second_value = F32Multiply::apply(&mut program, input, input).unwrap();
    program
        .output(OutputKey::new("first").unwrap(), first_value)
        .unwrap();
    program
        .output(OutputKey::new("second").unwrap(), second_value)
        .unwrap();
    let program = program.build().unwrap();
    let first_subject = IndexRefinementSubject::derive(
        &program,
        program.operations().next().unwrap().id(),
        test_contract(),
    )
    .unwrap();
    let second_subject = IndexRefinementSubject::derive(
        &program,
        program.operations().nth(1).unwrap().id(),
        test_contract(),
    )
    .unwrap();
    let semantic = FrozenSemanticRegistry::standard().unwrap();
    let scalars = FrozenScalarRegistry::standard().unwrap();
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(semantic, scalars.clone()).unwrap();
    let region = two_domain_residual_region();
    let scalar_authority = scalars.revalidate_region(&region).unwrap();
    let realization = VerifiedIndexRegionSequence::single(region);
    let pending = |resolution| PendingIndexRefinementReceipt {
        resolution,
        leading_scalar_authorities: Vec::new(),
        scalar_authority: scalar_authority.clone(),
        operand_bindings: Vec::new(),
        result_bindings: Vec::new(),
        realization: realization.clone(),
    };
    let first = pending(laws.resolve(&first_subject).unwrap());
    let second = pending(laws.resolve(&second_subject).unwrap());
    let mint = |pending: &PendingIndexRefinementReceipt| {
        mint_receipt(
            pending.subject(),
            &pending.resolution,
            &pending.realization,
            pending.scalar_authorities(),
            pending.operand_bindings.clone(),
            pending.result_bindings.clone(),
            Vec::new(),
        )
    };
    let first_receipt = mint(&first);
    let second_receipt = mint(&second);
    assert_eq!(
        second.verify_completion(&first_receipt),
        Err(IndexRefinementVerificationError::CompletionReceiptMismatch)
    );

    // The two occurrences agree on every other subject the executable
    // projection reads, so a coverage identity that failed to separate them
    // would be crossable between real, equally-shaped occurrences.
    assert_eq!(first_receipt.graph(), second_receipt.graph());
    assert_eq!(first_receipt.final_stage(), second_receipt.final_stage());
    assert_eq!(
        first_receipt.final_scalar_authority(),
        second_receipt.final_scalar_authority()
    );
    assert_eq!(
        first_receipt.operand_bindings(),
        second_receipt.operand_bindings()
    );
    assert_eq!(
        first_receipt.result_bindings(),
        second_receipt.result_bindings()
    );
    assert_ne!(first_receipt.occurrence(), second_receipt.occurrence());
    assert_ne!(
        first_receipt.executable_coverage_identity(),
        second_receipt.executable_coverage_identity()
    );
}

/// The contract-free family query answers off the registered law row.
///
/// **Four rows and one agreement, and the agreement is the load-bearing
/// half.** `tiler::rms-norm-f32@1` carries `StagedRootMeanSquareScaleF32`
/// and `tiler::softmax-f32@1` carries `StagedSoftmaxF32`, so both answer
/// true; `tiler::multiply-f32@1` carries a single-region law and answers
/// false; `tiler::slice-f32@1` carries its literal selection law and also
/// answers false because that law realizes one region. The agreement then
/// shows the query is the same fact read from the same row rather than a
/// second account of it: for a derived subject,
/// [`ResolvedIndexRealization::realizes_region_sequence`] answers
/// identically for both families.
#[test]
fn the_family_region_sequence_query_agrees_with_the_resolved_law() {
    use crate::semantic::{F32Multiply, F32RmsNorm};
    use crate::shape::Axis;

    let semantic = FrozenSemanticRegistry::standard().unwrap();
    let scalars = FrozenScalarRegistry::standard().unwrap();
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(semantic, scalars.clone()).unwrap();

    assert!(laws.family_realizes_region_sequence(&crate::semantic::rms_norm_f32_op()));
    assert!(laws.family_realizes_region_sequence(&crate::semantic::softmax_f32_op()));
    assert!(!laws.family_realizes_region_sequence(&crate::semantic::multiply_f32_op()));
    assert!(
        !laws.family_realizes_region_sequence(&crate::semantic::slice_f32_op()),
        "the registered literal slice law realizes one region, not a sequence"
    );
    assert_eq!(
        laws.family_realization_law(&crate::semantic::slice_f32_op()),
        Some(&super::super::IndexRealizationLaw::slice_f32())
    );

    // One program holding both families, so the resolved answers come from
    // occurrences the same authority actually admits.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([4]);
    let value = builder
        .input::<F32>(InputKey::new("x").unwrap(), shape.clone())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("w").unwrap(), shape)
        .unwrap();
    let normalized = F32RmsNorm::apply(
        &mut builder,
        value,
        weight,
        Axis::new(0),
        1.0e-6_f32.to_bits(),
    )
    .unwrap();
    let scaled = F32Multiply::apply(&mut builder, normalized, value).unwrap();
    builder
        .output(OutputKey::new("y").unwrap(), scaled)
        .unwrap();
    let program = builder.build().unwrap();

    for (position, expected) in [(0_usize, true), (1, false)] {
        let operation = program.operations().nth(position).unwrap();
        let key = operation.key().clone();
        let subject =
            IndexRefinementSubject::derive(&program, operation.id(), test_contract()).unwrap();
        let resolved = laws.resolve(&subject).unwrap();
        assert_eq!(resolved.realizes_region_sequence(), expected);
        assert_eq!(
            laws.family_realizes_region_sequence(&key),
            resolved.realizes_region_sequence(),
            "the contract-free query must answer what the resolved law answers for {key}"
        );
    }
}

/// A residual association reaches executable coverage only through proof.
///
/// The compile-fail doctest on [`PendingIndexRefinementReceipt`] carries the
/// structural half — a pending value exposes no coverage accessor. This
/// carries the behavioural half: an undischarged residual leaves `complete`
/// with no receipt to project, so no coverage identity exists to name.
///
/// Only the `Unknown` refusal is reachable from a verified region here.
/// `IndexRegionBuilder` runs its own exhaustive fallback under
/// [`MAX_EXHAUSTIVE_PROOF_CELLS`], and an access it can walk it either
/// discharges or refuses as `CoordinateOutOfBounds` at build time, so a
/// small disprovable region never becomes a `VerifiedIndexRegion` at all.
/// A `Disproved` completion needs a region inside the cell window between
/// that bound and [`MAX_FINITE_DOMAIN_PROOF_CELLS`] whose per-point integer
/// work still fits [`MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES`]; both refusal
/// arms leave `complete` through the same `Err` return, so the coverage
/// claim does not depend on exhibiting the second one.
#[test]
fn pending_and_refused_proofs_have_no_executable_coverage_spelling() {
    let mut program = SemanticProgramBuilder::try_standard().unwrap();
    let input = program
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([LENGTH]))
        .unwrap();
    let value = F32Multiply::apply(&mut program, input, input).unwrap();
    program
        .output(OutputKey::new("output").unwrap(), value)
        .unwrap();
    let program = program.build().unwrap();
    let subject = IndexRefinementSubject::derive(
        &program,
        program.operations().next().unwrap().id(),
        test_contract(),
    )
    .unwrap();
    let semantic = FrozenSemanticRegistry::standard().unwrap();
    let scalars = FrozenScalarRegistry::standard().unwrap();
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(semantic, scalars.clone()).unwrap();
    let resolution = laws.resolve(&subject).unwrap();
    let pending = |region: VerifiedIndexRegion| PendingIndexRefinementReceipt {
        resolution: resolution.clone(),
        leading_scalar_authorities: Vec::new(),
        scalar_authority: scalars.revalidate_region(&region).unwrap(),
        operand_bindings: Vec::new(),
        result_bindings: Vec::new(),
        realization: VerifiedIndexRegionSequence::single(region),
    };
    let budget = IndexDomainProofBudget::try_new(
        MAX_FINITE_DOMAIN_PROOF_CELLS,
        MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
    )
    .unwrap();

    let unprovable = pending(residual_region(1, 5, 0));
    assert_eq!(unprovable.obligations().len(), 1);
    let refusal = ResolvedIndexRealization::complete(&unprovable, budget)
        .expect_err("a residual beyond the hard integer budget cannot be discharged");
    assert_eq!(refusal.kind(), IndexDomainProofRefusalKind::Unknown);
    assert_eq!(refusal.assessments().len(), 1);
    // The same association mints coverage only once its obligations are
    // discharged, so the refusal above is the difference between a spelling
    // and none — not a difference in the association itself.
    let discharged = mint_receipt(
        unprovable.subject(),
        &unprovable.resolution,
        &unprovable.realization,
        unprovable.scalar_authorities(),
        unprovable.operand_bindings.clone(),
        unprovable.result_bindings.clone(),
        proofs_for(unprovable.final_stage()),
    );
    assert_eq!(discharged.index_domain_proofs().len(), 1);
    let unproved = mint_receipt(
        unprovable.subject(),
        &unprovable.resolution,
        &unprovable.realization,
        unprovable.scalar_authorities(),
        unprovable.operand_bindings.clone(),
        unprovable.result_bindings.clone(),
        Vec::new(),
    );
    assert_ne!(
        discharged.executable_coverage_identity(),
        unproved.executable_coverage_identity()
    );
}

/// Seals one exact-finite proof per retained obligation of `region`.
///
/// The completion algorithm's own budget is not the subject here; this
/// supplies the proof records a discharged association would carry so the
/// minted coverage can be compared against the same association's
/// undischarged encoding.
fn proofs_for(region: &VerifiedIndexRegion) -> Vec<IndexRefinementDomainProof> {
    let authority = Arc::new(IndexDomainProofAuthority::exact_finite());
    region
        .unknown_index_domain_predicates()
        .map(|obligation| {
            let proof = IndexDomainProofEvidence::ExhaustiveFinite {
                points: 2,
                derivation: EXHAUSTIVE_DERIVATION.into(),
            };
            IndexRefinementDomainProof {
                stage: 0,
                obligation,
                authority: authority.clone(),
                identity: IndexRefinementDomainProofIdentity(
                    encode_proof_identity(region, obligation, &authority, &proof)
                        .into_boxed_slice(),
                ),
                proof,
            }
        })
        .collect()
}

#[test]
fn wide_domain_environment_work_reaches_the_cell_hard_limit() {
    let mut extents = vec![1; 256];
    extents[0] = 65_535;
    let region = residual_region_with_extents(&extents, 0, 5, 1_i128.into(), 0_i128.into());
    assert!(matches!(
        assess(
            &region,
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        ),
        IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
            resource: super::super::ProofResource::Cells,
            required,
            limit: MAX_FINITE_DOMAIN_PROOF_CELLS,
        }) if required > u128::from(MAX_FINITE_DOMAIN_PROOF_CELLS)
    ));
}

#[test]
fn large_exact_counterexample_stays_bounded_and_disproved() {
    const POINTS: u64 = 257;
    let mut magnitude = vec![0; MAX_DOMAIN_EVIDENCE_BYTES + 1];
    magnitude[0] = 1;
    let large =
        IndexInteger::from_sign_magnitude(super::super::IndexIntegerSign::Positive, &magnitude)
            .unwrap();
    let negative_large =
        IndexInteger::from_sign_magnitude(super::super::IndexIntegerSign::Negative, &magnitude)
            .unwrap();
    assert!(large.magnitude_byte_len() > MAX_DOMAIN_EVIDENCE_BYTES);
    let mut builder = IndexRegionBuilder::new(FrozenScalarRegistry::standard().unwrap())
        .expect("the fixture receives a fresh builder identity");
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(POINTS))
        .unwrap();
    let shape = Shape::from_dims([POINTS]);
    let value_type = ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap());
    let input = builder
        .tensor(TensorRole::Input, value_type.clone(), shape.clone())
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, value_type, shape)
        .unwrap();
    let coordinate = builder.dimension_expr(dimension).unwrap();
    let mut equivalents = Vec::with_capacity(2_048);
    for index in 0_u64..2_048 {
        let equivalent = builder
            .modulo(
                coordinate,
                SourcedExtent::Static(Extent::new(POINTS + index + 1)),
            )
            .unwrap();
        equivalents.push(equivalent);
    }
    let mut cancellations = equivalents
        .as_chunks::<2>()
        .0
        .iter()
        .enumerate()
        .map(|(index, pair)| {
            let coefficients = if index == 0 {
                (large.clone(), negative_large.clone())
            } else {
                (1_i128.into(), (-1_i128).into())
            };
            let cancellation = builder
                .linear_combination(
                    0_i128.into(),
                    &[(coefficients.0, pair[0]), (coefficients.1, pair[1])],
                )
                .unwrap();
            builder
                .modulo(
                    cancellation,
                    SourcedExtent::Static(Extent::new(POINTS + 2_049 + index as u64)),
                )
                .unwrap()
        })
        .collect::<Vec<_>>();
    while cancellations.len() > 1 {
        cancellations = cancellations
            .chunks(2)
            .map(|pair| {
                if pair.len() == 1 {
                    pair[0]
                } else {
                    let sum = builder
                        .linear_combination(
                            0_i128.into(),
                            &[(1_i128.into(), pair[0]), (1_i128.into(), pair[1])],
                        )
                        .unwrap();
                    builder
                        .modulo(sum, SourcedExtent::Static(Extent::new(POINTS + 4_096)))
                        .unwrap()
                }
            })
            .collect();
    }
    let second_zero = builder
        .modulo(
            cancellations[0],
            SourcedExtent::Static(Extent::new(POINTS + 4_097)),
        )
        .unwrap();
    let exact_large = builder
        .linear_combination(
            1_i128.into(),
            &[
                (1_i128.into(), cancellations[0]),
                ((-1_i128).into(), second_zero),
                (1_i128.into(), coordinate),
            ],
        )
        .unwrap();
    let value = builder.read(input, &[dimension], &[exact_large]).unwrap();
    let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
    builder.output(write, value).unwrap();
    let region = builder.build().unwrap();
    let obligations = region.unknown_index_domain_predicates().collect::<Vec<_>>();
    assert_eq!(obligations.len(), 2);
    let obligation = obligations
        .iter()
        .copied()
        .find(|obligation| {
            matches!(
                obligation.predicate(),
                IndexDomainPredicate::LessThanExtent { .. }
            )
        })
        .expect("the exact upper-bound residual is retained");
    let claim = assess_finite_domains(
        &region,
        &[obligation],
        IndexDomainProofBudget::try_new(
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        )
        .unwrap(),
    )
    .pop()
    .unwrap();
    assert!(
        matches!(
            &claim,
            IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
                resource: super::super::ProofResource::IntegerBytes,
                required,
                limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
            }) if *required > u128::from(MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES)
        ),
        "{claim:?}"
    );

    let required_bytes = |obligations: &[UnknownIndexDomainPredicate]| {
        let claims = assess_finite_domains(
            &region,
            obligations,
            IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, 1).unwrap(),
        );
        let IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::ResourceLimit {
            resource: super::super::ProofResource::IntegerBytes,
            required,
            limit: 1,
        }) = claims[0]
        else {
            panic!("the one-byte perturbation must stop at group reservation")
        };
        u64::try_from(required).unwrap()
    };
    let grouped_bytes = required_bytes(&obligations);
    assert!(grouped_bytes > MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES);
}

#[test]
fn nonlinear_integer_work_refuses_a_one_mebibyte_product_preflight() {
    let mebibyte = 1024_u128 * 1024;
    let product = checked_add(mebibyte, mebibyte).unwrap();
    let required = multiplication_cost(mebibyte, mebibyte, product).unwrap();
    assert!(required > u128::from(MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES));
}

#[test]
fn negative_floor_division_and_modulo_have_exact_charge_boundaries() {
    let dividend = BigInt::from(-17_i32);
    assert_eq!(
        dividend.div_floor(&BigInt::from(5_u64)),
        BigInt::from(-4_i32)
    );
    assert_eq!(
        dividend.mod_floor(&BigInt::from(5_u64)),
        BigInt::from(3_u32)
    );
    for result_width in [2_u128, 8] {
        let required = division_cost(2, result_width).unwrap();
        let budget = IndexDomainProofBudget::try_new(
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            u64::try_from(required).unwrap(),
        )
        .unwrap();
        let mut exact = IndexDomainProofLedger::new(budget);
        assert!(exact.reserve_evaluation(0, required).is_ok());
        let mut short = IndexDomainProofLedger::new(
            IndexDomainProofBudget::try_new(
                MAX_FINITE_DOMAIN_PROOF_CELLS,
                u64::try_from(required - 1).unwrap(),
            )
            .unwrap(),
        );
        assert!(matches!(
            short.reserve_evaluation(0, required),
            Err(ProofPlanningFailure::Exhausted(IndexDomainProofExhaustion {
                resource: super::super::ProofResource::IntegerBytes,
                required: actual,
                limit,
            })) if actual == required && u128::from(limit) == required - 1
        ));
    }
}

#[test]
fn integer_work_overflow_refuses_before_evaluation() {
    assert!(matches!(
        multiplication_cost(u128::MAX, u128::MAX, u128::MAX),
        Err(ProofPlanningFailure::Unsupported)
    ));
}

#[test]
fn invalid_budgets_are_rejected_before_evaluation() {
    assert_eq!(
        IndexDomainProofBudget::try_new(0, MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES),
        Err(IndexRefinementVerificationError::InvalidDomainProofBudget {
            resource: super::super::ProofResource::Cells,
            actual: 0,
            limit: MAX_FINITE_DOMAIN_PROOF_CELLS,
        })
    );
    assert_eq!(
        IndexDomainProofBudget::try_new(
            MAX_FINITE_DOMAIN_PROOF_CELLS + 1,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        ),
        Err(IndexRefinementVerificationError::InvalidDomainProofBudget {
            resource: super::super::ProofResource::Cells,
            actual: MAX_FINITE_DOMAIN_PROOF_CELLS + 1,
            limit: MAX_FINITE_DOMAIN_PROOF_CELLS,
        })
    );
    assert_eq!(
        IndexDomainProofBudget::try_new(MAX_FINITE_DOMAIN_PROOF_CELLS, 0),
        Err(IndexRefinementVerificationError::InvalidDomainProofBudget {
            resource: super::super::ProofResource::IntegerBytes,
            actual: 0,
            limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        })
    );
    assert_eq!(
        IndexDomainProofBudget::try_new(
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES + 1,
        ),
        Err(IndexRefinementVerificationError::InvalidDomainProofBudget {
            resource: super::super::ProofResource::IntegerBytes,
            actual: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES + 1,
            limit: MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        })
    );
}

#[test]
fn residual_obligation_limit_refuses_before_pending_allocation() {
    assert_eq!(
        check_residual_obligation_count(MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS + 1),
        Err(
            IndexRefinementVerificationError::ResidualObligationsTooLarge {
                actual: MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS + 1,
                limit: MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS,
            }
        )
    );
    assert_eq!(
        check_residual_obligation_count(MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS),
        Ok(())
    );
}

#[test]
fn a_symbolic_domain_is_unsupported_and_mints_no_proof() {
    let symbol = ShapeSymbol::new(SymbolScope::new("proof/0").unwrap(), "n").unwrap();
    let mut environment = ShapeEnvBuilder::new();
    environment.declare(symbol.clone()).unwrap();
    environment
        .bind(
            &symbol,
            RootBinding::new(
                BindingSource::InterfaceParameter {
                    key: InterfaceParameterKey::new("n").unwrap(),
                },
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )
            .unwrap(),
        )
        .unwrap();
    environment
        .require(SemanticInputConstraint::new(
            ExtentRelation::interval(ExtentTerm::Symbol(symbol.clone()), 1, 16).unwrap(),
            FactProvenance::FrontendRequired,
        ))
        .unwrap();
    let environment = Arc::new(environment.build().unwrap());
    let mut builder = IndexRegionBuilder::new_with_shape_environment(
        FrozenScalarRegistry::standard().unwrap(),
        environment,
    )
    .unwrap();
    let value_type = ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap());
    let input = builder
        .tensor(TensorRole::Input, value_type.clone(), Shape::from_dims([8]))
        .unwrap();
    let output = builder
        .sourced_tensor(
            TensorRole::Output,
            value_type,
            vec![SourcedExtent::Symbol(symbol.clone())],
        )
        .unwrap();
    let dimension = builder
        .symbolic_dimension(DomainRole::Parallel, symbol)
        .unwrap();
    let coordinate = builder.dimension_expr(dimension).unwrap();
    let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
    let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
    builder.output(write, value).unwrap();
    let region = builder.build().unwrap();
    assert_eq!(EXTENT_PHASE_CEILING, AvailabilityPhase::LiveDevicePreflight);
    assert!(matches!(
        assess(
            &region,
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        ),
        IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment)
    ));
}

#[test]
fn a_static_domain_with_symbolic_tensor_extent_is_unsupported() {
    let symbol = ShapeSymbol::new(SymbolScope::new("proof/axis").unwrap(), "n").unwrap();
    let mut environment = ShapeEnvBuilder::new();
    environment.declare(symbol.clone()).unwrap();
    environment
        .bind(
            &symbol,
            RootBinding::new(
                BindingSource::InterfaceParameter {
                    key: InterfaceParameterKey::new("n").unwrap(),
                },
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )
            .unwrap(),
        )
        .unwrap();
    environment
        .require(SemanticInputConstraint::new(
            ExtentRelation::interval(ExtentTerm::Symbol(symbol.clone()), 1, 16).unwrap(),
            FactProvenance::FrontendRequired,
        ))
        .unwrap();
    let mut builder = IndexRegionBuilder::new_with_shape_environment(
        FrozenScalarRegistry::standard().unwrap(),
        Arc::new(environment.build().unwrap()),
    )
    .unwrap();
    let value_type = ResolvedValueType::nominal(TypeKey::new("tiler", "f32", 1).unwrap());
    let input = builder
        .sourced_tensor(
            TensorRole::Input,
            value_type.clone(),
            vec![SourcedExtent::Symbol(symbol)],
        )
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, value_type, Shape::from_dims([8]))
        .unwrap();
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(8))
        .unwrap();
    let coordinate = builder.dimension_expr(dimension).unwrap();
    let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
    let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
    builder.output(write, value).unwrap();
    let region = builder.build().unwrap();
    assert_eq!(region.unknown_index_domain_predicates().len(), 1);
    assert!(matches!(
        assess(
            &region,
            MAX_FINITE_DOMAIN_PROOF_CELLS,
            MAX_FINITE_DOMAIN_PROOF_INTEGER_BYTES,
        ),
        IndexDomainProofClaim::Unknown(IndexDomainUnknownReason::UnsupportedFragment)
    ));
}

#[test]
fn operand_errors_name_the_expanded_semantic_boundary() {
    assert_eq!(
        IndexRefinementVerificationError::OperandArity {
            region_inputs: 1,
            expanded_inputs: 3,
        }
        .to_string(),
        "region declares 1 inputs for 3 expanded semantic input boundaries"
    );
    assert_eq!(
        IndexRefinementVerificationError::OperandInterface { position: 2 }.to_string(),
        "region input 2 does not match its expanded semantic input boundary"
    );
    assert_eq!(
        count_expanded_inputs(&[encoded_boundary(0)], 0),
        Err(IndexRefinementVerificationError::EmptyEncodedOperandComponents { input: 0 })
    );
    assert_eq!(
        IndexRefinementVerificationError::EmptyEncodedOperandComponents { input: 2 }.to_string(),
        "encoded semantic input 2 declares no component boundaries"
    );
    assert_eq!(
        IndexRefinementVerificationError::OperandBindingsTooLarge {
            actual: 17_408,
            limit: MAX_INDEX_REFINEMENT_OPERAND_BINDINGS,
        }
        .to_string(),
        "expanded operand bindings 17408 exceed receipt limit 16384"
    );
}

#[test]
fn expanded_input_count_is_bounded_before_component_shapes_are_materialized() {
    // Sixteen maximum-size encoded contracts exactly fill the verified
    // region boundary population. A seventeenth crosses the public region
    // limit while this pass is still counting component declarations; no
    // component shape has been derived or retained yet.
    let boundary = encoded_boundary(1_024);
    let maximal = vec![boundary.clone(); 16];
    assert_eq!(
        count_expanded_inputs(&maximal, MAX_BOUNDARY_TENSORS),
        Ok(MAX_BOUNDARY_TENSORS)
    );
    let oversized = vec![boundary; 17];
    assert_eq!(
        count_expanded_inputs(&oversized, MAX_BOUNDARY_TENSORS),
        Err(IndexRefinementVerificationError::OperandArity {
            region_inputs: MAX_BOUNDARY_TENSORS,
            expanded_inputs: 17 * 1_024,
        })
    );
}

#[test]
fn operand_binding_population_is_bounded_before_collection() {
    // One maximum-size encoded semantic input may be aliased sixteen times
    // and exactly fill the receipt binding population. A seventeenth use
    // crosses the independent receipt limit even though the distinct
    // expanded input population remains only 1,024. This count-only pass
    // runs before the final binding Vec is allocated.
    let component_counts = [1_024];
    assert_eq!(
        count_operand_bindings(&[0; 16], &component_counts),
        Ok(MAX_INDEX_REFINEMENT_OPERAND_BINDINGS)
    );
    assert_eq!(
        count_operand_bindings(&[0; 17], &component_counts),
        Err(IndexRefinementVerificationError::OperandBindingsTooLarge {
            actual: 17 * 1_024,
            limit: MAX_INDEX_REFINEMENT_OPERAND_BINDINGS,
        })
    );
    assert_eq!(
        count_operand_bindings(&[0, 0], &[usize::MAX]),
        Err(IndexRefinementVerificationError::OperandBindingsTooLarge {
            actual: usize::MAX,
            limit: MAX_INDEX_REFINEMENT_OPERAND_BINDINGS,
        })
    );
}

/// Derives a one-result subject whose result boundary is `[extent]`, so a
/// region writing that boundary — whole or in pieces — can be bound against
/// a real occurrence rather than a hand-assembled one.
fn partitioned_subject(extent: u64) -> IndexRefinementSubject {
    let mut semantic = SemanticRegistryBuilder::standard().unwrap();
    semantic
        .register_provider(&ReachedSemanticProvider(1))
        .unwrap();
    let semantic = semantic.freeze().unwrap();
    let mut program = SemanticProgramBuilder::try_new(semantic).unwrap();
    let input = program
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([extent]))
        .unwrap();
    let result = program
        .apply(
            reached_semantic_operation(),
            OperationAttributes::empty(),
            &[input.erase(), input.erase()],
        )
        .unwrap()
        .pop()
        .unwrap();
    program
        .output_resolved(OutputKey::new("output").unwrap(), result)
        .unwrap();
    let program = program.build().unwrap();
    IndexRefinementSubject::derive(
        &program,
        program.operations().next().unwrap().id(),
        test_contract(),
    )
    .unwrap()
}

/// Builds one f32 output of `[boundary]` written by `roots`, each
/// `(extent, offset)` iterating its own parallel dimension and writing
/// `d + offset` — the contiguous unequal partition the write-ownership
/// contract admits. A single root spanning the whole boundary is the
/// degenerate case and takes the whole-boundary ownership path.
fn partitioned_region(boundary: u64, roots: &[(u64, i128)]) -> VerifiedIndexRegion {
    let mut builder = IndexRegionBuilder::new(FrozenScalarRegistry::standard().unwrap())
        .expect("the fixture receives a fresh builder identity");
    let value_type = f32_type();
    let shape = Shape::from_dims([boundary]);
    let input = builder
        .tensor(TensorRole::Input, value_type.clone(), shape.clone())
        .unwrap();
    let output = builder
        .tensor(TensorRole::Output, value_type, shape)
        .unwrap();
    for (extent, offset) in roots {
        let dimension = builder
            .dimension(DomainRole::Parallel, Extent::new(*extent))
            .unwrap();
        let expression = builder.dimension_expr(dimension).unwrap();
        let coordinate = builder
            .linear_combination((*offset).into(), &[(1_i128.into(), expression)])
            .unwrap();
        let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
        let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
    }
    builder.build().unwrap()
}

fn ownership(
    region: &VerifiedIndexRegion,
    binding: &ResultBinding,
) -> Option<super::super::WriteOwnershipProofView> {
    region
        .access(binding.write_access())
        .unwrap()
        .write_ownership_proof()
}

/// The refusal this ticket exists to remove: three roots into eight is one
/// well-formed output, and every member of it is named by the binding rather
/// than one of them being chosen.
#[test]
fn a_partitioned_result_binds_one_binding_per_root() {
    let subject = partitioned_subject(8);
    let region = partitioned_region(8, &[(3, 0), (5, 3)]);
    assert_eq!(region.outputs().len(), 2, "the fixture is partitioned");

    let bindings = bind_results(&subject, &region).expect("a partitioned output binds");
    assert_eq!(bindings.len(), 2);
    assert!(
        bindings.iter().all(|binding| binding.result() == 0),
        "both members answer the one semantic result: {bindings:?}"
    );
    assert_eq!(bindings[0].output_tensor(), bindings[1].output_tensor());
    assert_ne!(
        bindings[0].write_access(),
        bindings[1].write_access(),
        "each member carries its own write rather than a shared one"
    );

    // What makes the receipt justified for the whole output: every named
    // write carries partition-relative totality, and the joint obligation
    // behind it was discharged by the region verifier.
    for binding in &bindings {
        assert!(
            matches!(
                ownership(&region, binding),
                Some(super::super::WriteOwnershipProofView::PartitionMember { .. })
            ),
            "member {binding:?} owns its partition"
        );
    }
}

/// The sole-root binding is what it was before grouping existed: one entry,
/// result zero, that root's own write and value. This is the case every
/// pinned executable-coverage identity encodes, so it must not move.
#[test]
fn a_sole_root_binds_exactly_one_result_to_its_whole_output() {
    let subject = partitioned_subject(8);
    let region = partitioned_region(8, &[(8, 0)]);
    let root = region.outputs().next().expect("the region has one root");

    let bindings = bind_results(&subject, &region).expect("a whole-output write binds");
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].result(), 0);
    assert_eq!(bindings[0].write_access(), root.access());
    assert_eq!(bindings[0].written_value(), root.value());
    assert!(matches!(
        ownership(&region, &bindings[0]),
        Some(
            super::super::WriteOwnershipProofView::CoordinatePermutation {
                facts: super::super::IndexDomainFactSource::Program,
            }
        )
    ));
}

/// Grouping is by output *tensor*, so two roots writing two genuinely
/// different outputs remain two outputs and still disagree with one result.
#[test]
fn two_distinct_output_tensors_still_disagree_with_one_result() {
    let subject = partitioned_subject(8);
    let mut builder = IndexRegionBuilder::new(FrozenScalarRegistry::standard().unwrap())
        .expect("the fixture receives a fresh builder identity");
    let value_type = f32_type();
    let shape = Shape::from_dims([8]);
    let input = builder
        .tensor(TensorRole::Input, value_type.clone(), shape.clone())
        .unwrap();
    let dimension = builder
        .dimension(DomainRole::Parallel, Extent::new(8))
        .unwrap();
    let coordinate = builder.dimension_expr(dimension).unwrap();
    let value = builder.read(input, &[dimension], &[coordinate]).unwrap();
    for _ in 0..2 {
        let output = builder
            .tensor(TensorRole::Output, value_type.clone(), shape.clone())
            .unwrap();
        let write = builder.write(output, &[dimension], &[coordinate]).unwrap();
        builder.output(write, value).unwrap();
    }
    let region = builder.build().unwrap();

    assert_eq!(
        bind_results(&subject, &region),
        Err(IndexRefinementVerificationError::ResultArity {
            region_outputs: 2,
            results: 1,
        })
    );
}

/// Every member reaches executable coverage, so a binding that named only
/// one root of a partition would mint different bytes — which is why the
/// receipt cannot quietly drop a member.
#[test]
fn dropping_one_partition_member_changes_executable_coverage() {
    let (_, resolution, realization, receipt) = reached_semantic_fixture(1);
    let subject = partitioned_subject(8);
    let region = partitioned_region(8, &[(3, 0), (5, 3)]);
    let bindings = bind_results(&subject, &region).expect("a partitioned output binds");

    let encode = |results: &[ResultBinding]| {
        encode_executable_coverage_identity(
            &subject,
            &resolution,
            &realization,
            &receipt.scalar_authorities(),
            receipt.operand_bindings(),
            results,
            &[],
        )
    };
    assert_ne!(encode(&bindings), encode(&bindings[..1]));
    assert_ne!(encode(&bindings), encode(&bindings[1..]));
}
