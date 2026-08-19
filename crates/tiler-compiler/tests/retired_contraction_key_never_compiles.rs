//! The retired contraction key never reaches a compiled plan, and nothing
//! falls back to the successor on its behalf.
//!
//! [ADR 0112](../../../docs/decisions/0112-replace-the-strict-contraction-key-with-a-permission-indexed-successor.md)
//! retires `tiler::strict-tensor-contraction-f32@1` and replaces it with
//! `tiler::tensor-contraction-f32@1`, stating that the retired key "has no
//! alias, equivalence rule, fallback, or duplicate selection policy anywhere in
//! the standard semantic, reference, law, lowering, compiler-recognition, or
//! frontend vertical", and that "an installed standard compiler or reference
//! has no authority to compile or execute the retired operation".
//!
//! That is a claim about an absent behaviour, so it needs two assertions rather
//! than one, because a single one of them is satisfiable for the wrong reason:
//!
//! 1. [`the_standard_semantic_authority_has_no_retired_contraction_key`] — the
//!    retired key cannot even be applied against the standard registry, so no
//!    ordinary frontend program can carry it. This is the key-reaching half:
//!    the *only* difference from a program that builds is the key string.
//! 2. [`a_retired_key_program_built_through_an_extension_provider_is_refused`]
//!    — a caller who supplies the retired definition themselves, through the
//!    public extension machinery, still gets a typed refusal out of the
//!    ordinary [`compile`] entry rather than a plan. This is the no-fallback
//!    half: the compiler does not recognize the retired occurrence as its
//!    successor and does not compile it as one.
//!
//! **The positive control is not duplicated here.** `contraction_direct_path`'s
//! `a_contraction_compiles_through_the_ordinary_entry_point` compiles the same
//! `td,od->to` workload under the successor key, through the same [`compile`]
//! entry and the same [`TargetProfile::governed`] target, under every stated
//! numerical contract. That test and this file are the pair that holds the
//! no-fallback property: without it, both assertions below would also pass in a
//! build where *no* contraction compiled at all, which is a different defect
//! wearing the same green.
//!
//! # What refusal 2's rule is, exactly, and what it is not
//!
//! The observed class is `UnsupportedCapability { rule:
//! "semantic-authority-pairing" }`, raised at request preflight before any
//! target-qualified explain trace exists. That rule is the authority-pairing
//! check — the installed lowering registry's semantic snapshot against the
//! program's own — and it is *not* keyed on the retired operation. Any program
//! built over a registry the installed capabilities were not paired with
//! refuses identically; `pipeline::conformance`'s
//! `externally_registered_operations_require_their_own_realization_authority`
//! pins that same rule for an unrelated external family.
//!
//! This is stated rather than papered over, because it is what makes the pair
//! of assertions necessary. Registering the retired key is inseparable from
//! leaving the standard authority — that is precisely what "retired from the
//! standard vertical" means — so there is no public spelling in which a retired
//! occurrence reaches recognition with a coherent authority behind it. The two
//! refusals together are the complete statement: the standard authority does
//! not have the key, and an authority that does have it is not one this
//! compiler will plan against.

use std::sync::Arc;

use tiler_compiler::session::{CompileFailureClass, CompileRequest, NumericalContract, compile};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::semantic::{
    BuildError, CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CanonicalField, CanonicalValue,
    CanonicalValueKind, ContractionIndex, ContractionIndexStructure, F32, InputKey,
    NormativeDefinitionRef, OpKey, OperationArity, OperationAttributeSchema, OperationAttributes,
    OperationConformance, OperationDefinition, OperationDefinitionFacts, OperationEffect,
    OperationInferenceError, OperationInferenceOutputs, OperationInferenceRequest,
    OperationInferencer, OperationSchema, OutputKey, ProviderDiagnosticCode, ProviderIdentity,
    RegistryError, SemanticProgram, SemanticProgramBuilder, SemanticRegistryBuilder,
    SemanticRegistryProvider, SemanticRegistryRegistrar, ValueFact,
};
use tiler_ir::shape::Shape;

/// The retired key ADR 0112 removed from the standard vertical.
const RETIRED_NAME: &str = "strict-tensor-contraction-f32";

/// The accepted successor, named here only so the two spellings sit together.
const SUCCESSOR_NAME: &str = "tensor-contraction-f32";

/// The same extents `contraction_direct_path`'s control compiles under.
const M: u64 = 2;
const N: u64 = 2;
const K: u64 = 4;

fn contraction_key(name: &str) -> OpKey {
    OpKey::new("tiler", name, 1).expect("a two-segment contraction key is bounded")
}

/// The pinned `td,od->to` structure with arbitrary frontend labels.
fn projection_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [
            [ContractionIndex::new(19), ContractionIndex::new(3)],
            [ContractionIndex::new(14), ContractionIndex::new(3)],
        ],
        [ContractionIndex::new(19), ContractionIndex::new(14)],
    )
    .expect("td,od->to is admitted")
}

/// The one required attribute of either contraction spelling.
fn structure_attribute() -> OperationAttributes {
    OperationAttributes::new([CanonicalField::new(
        CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE,
        projection_structure().canonical_value().clone(),
    )])
    .expect("a single-field attribute record is canonical")
}

/// Derives `[m, n]` from operands `[m, k]` and `[n, k]`.
///
/// The provider below needs *an* inferencer to be a well-formed registration;
/// what it computes is not the subject. It is written to agree with the
/// contraction's real result shape anyway, so the program under test differs
/// from a compiling one in its operation key and in nothing a later reader has
/// to discount.
struct ProjectionInferencer;

impl OperationInferencer for ProjectionInferencer {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let [left, right] = request.operands() else {
            return Err(OperationInferenceError::new(
                ProviderDiagnosticCode::new("operand-arity")
                    .expect("a test diagnostic code is bounded"),
                "a binary contraction takes exactly two operands",
            )
            .expect("a test diagnostic message is bounded"));
        };
        let leading = |fact: &ValueFact| {
            fact.shape()
                .as_static()
                .expect("a static test operand")
                .extents()[0]
                .get()
        };
        let rows = leading(left);
        let columns = leading(right);
        outputs.try_push(ValueFact::new(
            F32::resolved_type(),
            Shape::from_dims([rows, columns]),
        ))
    }
}

/// An extension provider carrying the retired key.
///
/// Its definition content is deliberately ordinary: a binary, single-result
/// schema with the structure attribute, exactly the shape a caller reading
/// historical bytes would reconstruct. Nothing here is a governed registration,
/// which is the whole point — the standard registrar has no such key to offer.
struct RetiredKeyProvider;

impl SemanticRegistryProvider for RetiredKeyProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "retired-contraction", 1)
            .expect("a test provider identity is bounded")
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_operation(OperationDefinition::new(
            contraction_key(RETIRED_NAME),
            OperationSchema::new(
                OperationArity::exact(2),
                OperationArity::exact(1),
                [OperationAttributeSchema::required(
                    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE,
                    CanonicalValueKind::Record,
                )],
            )
            .expect("a binary single-result schema is valid"),
            NormativeDefinitionRef::new("test retired strict contraction key")?,
            OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
            OperationConformance::new(CanonicalValue::boolean(true)),
            OperationEffect::Pure,
            Arc::new(ProjectionInferencer),
        ))
    }
}

/// The retired key is not applicable against the standard semantic authority.
///
/// The builder is the ordinary [`SemanticProgramBuilder::try_standard`] one, the
/// operands and attribute are the ones the successor compiles under, and the
/// only thing that differs from a program that builds is the operation name.
///
/// **Subject perturbations, run 2026-08-19 at `0f0100f7`.** The two assertions
/// below guard independent properties, so each was reddened on its own by
/// changing the subject rather than the assertion.
///
/// - Replacing `RETIRED_NAME` with `SUCCESSOR_NAME` in the first `apply` call
///   makes the application succeed, so `expect_err` panics with `the standard
///   registry has no retired contraction key: [ValueId { owner: GraphId(1),
///   index: ValueIndex(2) }]`. The refusal therefore reaches the key rather than
///   the operand binding, the attribute record, or the builder's own admission
///   rules, each of which is identical across the flip.
/// - Replacing it with an unrelated absent name (`some-other-absent-key`) keeps
///   `expect_err` satisfied but reddens the `matches!` below: `expected a
///   missing-authority refusal naming the retired key, got
///   SemanticRegistry(UnregisteredOperationAuthority { key:
///   OpKey(TypeKey(Key { namespace: "tiler", name: "some-other-absent-key",
///   semantic_version: 1 })) })`. The variant check is thus pinning *this* key's
///   absence and not merely "some key is absent".
#[test]
fn the_standard_semantic_authority_has_no_retired_contraction_key() {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder builds");
    let left = builder
        .input::<F32>(
            InputKey::new("activations").unwrap(),
            Shape::from_dims([M, K]),
        )
        .unwrap()
        .erase();
    let right = builder
        .input::<F32>(InputKey::new("weights").unwrap(), Shape::from_dims([N, K]))
        .unwrap()
        .erase();

    let error = builder
        .apply(
            contraction_key(RETIRED_NAME),
            structure_attribute(),
            &[left, right],
        )
        .expect_err("the standard registry has no retired contraction key");

    // The refusal is the registry's own missing-authority answer, not a shape,
    // arity, or attribute diagnostic that a differently-broken program could
    // also produce.
    assert!(
        matches!(
            error,
            BuildError::SemanticRegistry(RegistryError::UnregisteredOperationAuthority {
                ref key
            }) if **key == contraction_key(RETIRED_NAME)
        ),
        "expected a missing-authority refusal naming the retired key, got {error:?}"
    );

    // The successor spelling of the same application is the one the standard
    // authority does have; `contraction_direct_path` compiles it end to end.
    builder
        .apply(
            contraction_key(SUCCESSOR_NAME),
            structure_attribute(),
            &[left, right],
        )
        .expect("the successor key is the standard authority's contraction");
}

/// Builds the retired-key program through the public extension machinery.
fn retired_key_program() -> SemanticProgram {
    let mut registry = SemanticRegistryBuilder::standard().expect("the standard registry builds");
    registry
        .register_provider(&RetiredKeyProvider)
        .expect("an extension provider may add a non-standard key");
    let mut builder =
        SemanticProgramBuilder::try_new(registry.freeze().expect("the registry freezes"))
            .expect("a builder over the extended registry");
    let left = builder
        .input::<F32>(
            InputKey::new("activations").unwrap(),
            Shape::from_dims([M, K]),
        )
        .unwrap()
        .erase();
    let right = builder
        .input::<F32>(InputKey::new("weights").unwrap(), Shape::from_dims([N, K]))
        .unwrap()
        .erase();
    let results = builder
        .apply(
            contraction_key(RETIRED_NAME),
            structure_attribute(),
            &[left, right],
        )
        .expect("the extension provider admits its own key");
    let [projected] = results.as_slice() else {
        panic!("the registered schema declares exactly one result");
    };
    builder
        .output_resolved(OutputKey::new("projected").unwrap(), *projected)
        .unwrap();
    builder
        .build()
        .expect("the retired-key program is well formed")
}

/// A caller who supplies the retired definition themselves still gets a typed
/// refusal from the ordinary entry point, never a plan.
///
/// The program is built entirely through the public extension machinery —
/// [`SemanticRegistryBuilder::standard`], [`SemanticRegistryProvider`],
/// [`SemanticProgramBuilder::try_new`] — so this is the route a frontend
/// resurrecting the retired key would actually take, not a crate-internal
/// fixture.
///
/// **Subject perturbation, run 2026-08-19 at `0f0100f7`.** With
/// `RetiredKeyProvider::register` and `retired_key_program` both flipped to
/// `SUCCESSOR_NAME` — nothing else changed, the same schema, facts, conformance,
/// effect, and inferencer — the refusal disappears from this boundary entirely.
/// Registration fails first, at `register_provider`, with `an extension provider
/// may add a non-standard key: InvalidGovernedContractionDescriptor { source:
/// MalformedFacts { actual: Bool } }`.
///
/// That is the sharpest evidence available that the key is what this test
/// reaches. The registrar admits *arbitrary* fact bytes under the retired name
/// and refuses the identical bytes under the successor name, because the
/// successor is the governed contraction key and
/// `ContractionF32ReductionDescriptor::decode` is its mandatory gate. One
/// string decides which of two entirely different code paths runs, and the
/// `compile` assertion below is reachable only along the retired one.
///
/// **The same flip at the `compile` boundary itself, same run.** Replacing
/// `retired_key_program()` with the identical `td,od->to` program spelled
/// against `try_standard` under `SUCCESSOR_NAME` — the spelling the retirement
/// leaves as the only one the standard registrar admits — makes `compile`
/// return `Ok`, so `expect_err` panics with `no installed capability compiles
/// the retired contraction key: CompilationBatch { targets: [ ... ] }` carrying
/// a full `tiler.prototype-target-neutral-baseline.v1` compilation. The refusal
/// is therefore a property of the key under test and not of this request, this
/// target, or this contract, each of which is unchanged across the flip.
#[test]
fn a_retired_key_program_built_through_an_extension_provider_is_refused() {
    let program = retired_key_program();
    let targets = TargetRequest::new([TargetProfile::governed()]).unwrap();

    let failure = compile(CompileRequest::new(
        &program,
        NumericalContract::STRICT_F32,
        targets,
    ))
    .expect_err("no installed capability compiles the retired contraction key");

    assert_eq!(
        failure.class(),
        CompileFailureClass::UnsupportedCapability {
            rule: "semantic-authority-pairing"
        }
    );
    assert!(
        failure.explain().is_none(),
        "the refusal precedes any target-qualified explain trace"
    );
}
