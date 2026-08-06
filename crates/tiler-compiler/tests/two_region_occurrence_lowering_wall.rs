//! Where the two-region occurrence actually stops, demonstrated.
//!
//! [`lower-a-two-region-occurrence-through-one-index-access-capability`] was filed
//! on the premise that `IndexAccessLoweringProvider::lower` emitting one region
//! per occurrence is what holds `tiler::rms-norm-f32@1` below R6, and that
//! widening the *compiler's* lowering vocabulary to an ordered region sequence
//! would release it. The first half of that premise is true as a statement about
//! the trait. The second half is false, and this file is why.
//!
//! The refusal for a normalization occurrence arrives from
//! `FrozenIndexRealizationLawRegistry::resolve`, one statement *before*
//! `refine_index_region` drives any provider at all
//! (`crates/tiler-compiler/src/legality.rs`, where `resolve` precedes
//! `emit_region`). `tiler-ir`'s standard registration deliberately registers no
//! `IndexRealizationLaw` for the normalization or the softmax
//! (`crates/tiler-ir/src/semantic/registry.rs`, the nine-row law sidecar, whose
//! own comment says absence "fails closed later"). So the provider's region arity
//! is not what refuses: the provider is never asked.
//!
//! **This file exists so that the attribution is checked rather than asserted.**
//! A ceiling attributed to the wrong layer is worse than one left unattributed,
//! because it makes work look reachable from a scope that cannot reach it — here,
//! `implementation/compiler`, when the layer that must move is
//! `implementation/ir`. The recording provider below is what turns "the provider
//! never runs" from a reading of the control flow into an observation.
//!
//! What would actually be required is recorded in
//! [`admit-a-multi-region-index-realization-law`]: `IndexRealizationLaw` is a
//! closed enum of atomic single-region templates, `realize` returns one
//! `VerifiedIndexRegion`, and `ResolvedIndexRealization::verify` accepts one
//! candidate region and demands its canonical identity equal the law's own
//! reconstruction. An ordered region sequence has no canonical identity that
//! comparison can consume, so the sequence vocabulary has to exist in the law and
//! receipt layer before a capability can usefully declare one.
//!
//! [`lower-a-two-region-occurrence-through-one-index-access-capability`]: ../../../tickets/lower-a-two-region-occurrence-through-one-index-access-capability.md
//! [`admit-a-multi-region-index-realization-law`]: ../../../tickets/admit-a-multi-region-index-realization-law.md

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tiler_compiler::capability::{
    IndexAccessLoweringContext, IndexAccessLoweringProvider, LoweringCapabilityRegistryBuilder,
    LoweringCapabilityRevision, LoweringEmitError, LoweringSignature,
};
use tiler_compiler::legality::{RefinementError, refine_index_region};
use tiler_ir::index::{
    DomainRole, FrozenIndexRealizationLawRegistry, FrozenScalarRegistry, IndexRefinementSubject,
    IndexRefinementVerificationError, NumericalContractIdentity, ScalarArity,
    ScalarAttributeSchema, ScalarAttributes, ScalarEffect, ScalarInferenceError,
    ScalarInferenceOutputs, ScalarInferenceRequest, ScalarOperationContract,
    ScalarOperationDefinition, ScalarOperationInferencer, ScalarRegistryBuilder,
    multiply_f32_scalar_op,
};
use tiler_ir::semantic::{
    CanonicalValue, F32, F32Multiply, F32RmsNorm, FrozenSemanticRegistry, InputKey,
    NormativeDefinitionRef, OutputKey, ProviderDiagnosticCode, ProviderIdentity, ResolvedValueType,
    SemanticProgramBuilder, multiply_f32_op, rms_norm_f32_op,
};
use tiler_ir::shape::{Axis, Extent, Shape};

/// The normalized extent used by every fixture here.
const LENGTH: u64 = 4;

/// The governed strict-F32 contract key, spelled through the compiler's session
/// surface so the fixture cannot drift from what a caller can actually state.
fn contract() -> NumericalContractIdentity {
    NumericalContractIdentity::try_from_key(
        tiler_compiler::session::NumericalContract::STRICT_F32.key(),
    )
    .expect("the governed strict F32 contract key satisfies the IR bound")
}

fn f32_type() -> ResolvedValueType {
    F32::resolved_type()
}

fn semantic() -> FrozenSemanticRegistry {
    FrozenSemanticRegistry::standard().expect("the standard semantic registry is coherent")
}

fn provider(name: &str) -> ProviderIdentity {
    ProviderIdentity::new("example", name, 1).expect("the fixture provider identity is canonical")
}

fn revision() -> LoweringCapabilityRevision {
    LoweringCapabilityRevision::new(1).expect("revision 1 is nonzero")
}

/// The two-operand, one-result signature both families under test resolve.
///
/// The normalization takes `(value, weight)` and the multiply takes `(a, b)`;
/// that they coincide is what lets one control attribute the other's refusal.
fn binary_signature() -> LoweringSignature {
    LoweringSignature::new([f32_type(), f32_type()], [f32_type()])
        .expect("a two-operand signature is within the governed bound")
}

struct SameType;

impl ScalarOperationInferencer for SameType {
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        let Some(first) = request.operands().first() else {
            return Err(ScalarInferenceError::new(
                ProviderDiagnosticCode::new("example.arity").expect("a canonical diagnostic code"),
                "at least one operand is required",
            )
            .expect("a canonical inference error"));
        };
        outputs.try_push(first.clone())
    }
}

/// A scalar authority carrying the one governed multiply the multiply law reaches.
fn scalar_registry() -> FrozenScalarRegistry {
    let mut builder = ScalarRegistryBuilder::new(semantic());
    builder
        .register(
            provider("f32-scalars"),
            ScalarOperationDefinition::new(
                multiply_f32_scalar_op(),
                NormativeDefinitionRef::from_owned("urn:example:multiply:v1".to_owned())
                    .expect("a canonical normative reference"),
                ScalarOperationContract::new(
                    ScalarAttributeSchema::empty(),
                    ScalarArity::exact(2).expect("a two-operand scalar arity"),
                    ScalarArity::exact(1).expect("a one-result scalar arity"),
                    ScalarEffect::Pure,
                    CanonicalValue::record([]).expect("an empty canonical record"),
                    CanonicalValue::record([]).expect("an empty canonical record"),
                ),
                Arc::new(SameType),
            ),
        )
        .expect("registering one scalar against the standard authority succeeds");
    builder.freeze()
}

/// Emits `out[i] = mul(in[i], in[i])`, the exact region the multiply law builds.
struct PointwiseSquare;

impl IndexAccessLoweringProvider for PointwiseSquare {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        let shape = Shape::from_dims([LENGTH]);
        let i = context.dimension(DomainRole::Parallel, Extent::new(LENGTH))?;
        let input = context.input_tensor(f32_type(), shape.clone())?;
        let output = context.output_tensor(f32_type(), shape)?;
        let row = context.dimension_expr(i)?;
        let value = context.read(input, &[i], &[row])?;
        let product = context.apply(
            multiply_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[value, value],
        )?;
        let squared = product.get(0).expect("multiply yields one result");
        let write = context.write(output, &[i], &[row])?;
        context.output(write, squared)
    }
}

/// A provider that counts its invocations and otherwise emits nothing.
///
/// Emitting nothing is deliberate. If the normalization's refusal were the
/// region-arity wall the ticket named, this provider would be driven and the
/// refusal would name its empty region — a `Build` or interface diagnostic.
/// Observing zero invocations instead is what distinguishes "the provider emitted
/// the wrong thing" from "the provider was never asked", and only the second is
/// consistent with the wall living in the law registry.
struct RecordingProvider(Arc<AtomicUsize>);

impl IndexAccessLoweringProvider for RecordingProvider {
    fn lower(
        &self,
        _context: &mut IndexAccessLoweringContext<'_>,
    ) -> Result<(), LoweringEmitError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Err(LoweringEmitError::Occurrence {
            rule: "fixture-never-reached",
        })
    }
}

/// Builds a one-occurrence program and derives its refinement subject.
fn subject_of(program: &tiler_ir::semantic::SemanticProgram) -> IndexRefinementSubject {
    let operation = program
        .operations()
        .next()
        .expect("the fixture program has one operation")
        .id();
    IndexRefinementSubject::derive(program, operation, contract())
        .expect("a standard occurrence derives a subject")
}

/// A one-occurrence `tiler::rms-norm-f32@1` program over `[LENGTH]`.
fn rms_norm_subject() -> IndexRefinementSubject {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder");
    let shape = Shape::from_dims([LENGTH]);
    let value = builder
        .input::<F32>(
            InputKey::new("x").expect("a canonical input key"),
            shape.clone(),
        )
        .expect("declaring an input succeeds");
    let weight = builder
        .input::<F32>(InputKey::new("w").expect("a canonical input key"), shape)
        .expect("declaring an input succeeds");
    let normalized = F32RmsNorm::apply(
        &mut builder,
        value,
        weight,
        Axis::new(0),
        1.0e-6_f32.to_bits(),
    )
    .expect("the normalization applies over its one axis");
    builder
        .output(
            OutputKey::new("y").expect("a canonical output key"),
            normalized,
        )
        .expect("declaring an output succeeds");
    subject_of(&builder.build().expect("the program verifies"))
}

/// A one-occurrence `tiler::multiply-f32@1` program over `[LENGTH]`.
fn multiply_subject() -> IndexRefinementSubject {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder");
    let input = builder
        .input::<F32>(
            InputKey::new("a").expect("a canonical input key"),
            Shape::from_dims([LENGTH]),
        )
        .expect("declaring an input succeeds");
    let result = F32Multiply::apply(&mut builder, input, input).expect("the multiply applies");
    builder
        .output(
            OutputKey::new("out").expect("a canonical output key"),
            result,
        )
        .expect("declaring an output succeeds");
    subject_of(&builder.build().expect("the program verifies"))
}

/// Registers one index-access capability and resolves it against `operation`.
fn resolved_capability(
    operation: &tiler_ir::semantic::OpKey,
    implementation: Arc<dyn IndexAccessLoweringProvider>,
    scalars: &FrozenScalarRegistry,
) -> tiler_compiler::capability::ResolvedLoweringCapability {
    let mut builder = LoweringCapabilityRegistryBuilder::new(semantic(), scalars.clone())
        .expect("the standard authorities cohere");
    builder
        .register_index_access(
            provider("index"),
            operation.clone(),
            binary_signature(),
            &[multiply_f32_scalar_op()],
            revision(),
            implementation,
        )
        .expect("registering an index-access capability succeeds");
    builder
        .freeze()
        .resolve_index_access(operation, &binary_signature())
        .expect("the registered capability resolves")
}

fn realization_laws(scalars: &FrozenScalarRegistry) -> FrozenIndexRealizationLawRegistry {
    FrozenIndexRealizationLawRegistry::from_semantic(semantic(), scalars.clone())
        .expect("the standard semantic and scalar authorities cohere")
}

/// Registering a normalization lowering capability is *not* what refuses.
///
/// This is the half that makes the later refusal attributable. `admit` records
/// the realization-law row as an `Option` and never requires one, so the
/// capability registers cleanly; a reader who saw only the refusal below might
/// otherwise conclude the registry had rejected the family outright.
#[test]
fn a_normalization_index_access_capability_registers_and_resolves() {
    let scalars = scalar_registry();
    let counter = Arc::new(AtomicUsize::new(0));
    let resolved = resolved_capability(
        &rms_norm_f32_op(),
        Arc::new(RecordingProvider(counter.clone())),
        &scalars,
    );

    assert_eq!(resolved.operation(), &rms_norm_f32_op());
    assert_eq!(resolved.revision(), revision());
    assert_eq!(
        counter.load(Ordering::Relaxed),
        0,
        "registration and resolution must not drive the provider"
    );
}

/// The normalization has no index realization law, and that is the refusal.
///
/// Stated against the law registry directly rather than only through
/// `refine_index_region`, so the finding names the exact authority that refuses
/// and the exact typed reason a reader can grep for in `tiler-ir`.
#[test]
fn the_normalization_resolves_no_index_realization_law() {
    let scalars = scalar_registry();
    let error = realization_laws(&scalars)
        .resolve(&rms_norm_subject())
        .expect_err("the standard registration registers no normalization law");

    assert!(
        matches!(
            error,
            IndexRefinementVerificationError::MissingRealizationLaw
        ),
        "the normalization must refuse for an absent law and not for some other \
         subject defect, or this file has attributed the ceiling to the wrong \
         authority; observed {error:?}"
    );
}

/// The premise-falsifying observation: refinement never drives the provider.
///
/// The ticket's inference was that a single-region provider would be asked to
/// realize a two-region occurrence and would emit a truncated or explosive
/// region. It is not asked. The refusal precedes emission, so widening the
/// provider's region vocabulary inside `crates/tiler-compiler` would change
/// nothing observable for this family.
#[test]
fn refining_the_normalization_refuses_before_the_provider_is_driven() {
    let scalars = scalar_registry();
    let counter = Arc::new(AtomicUsize::new(0));
    let resolved = resolved_capability(
        &rms_norm_f32_op(),
        Arc::new(RecordingProvider(counter.clone())),
        &scalars,
    );

    let error = refine_index_region(
        &resolved,
        &rms_norm_subject(),
        &realization_laws(&scalars),
        &scalars,
    )
    .expect_err("a family with no realization law cannot be refined");

    let RefinementError::IrVerifier(source) = &error else {
        panic!("the refusal must come from the IR realization authority; observed {error:?}");
    };
    assert!(
        matches!(
            **source,
            IndexRefinementVerificationError::MissingRealizationLaw
        ),
        "the refusal must name the absent law; observed {source:?}"
    );
    assert_eq!(
        counter.load(Ordering::Relaxed),
        0,
        "the provider must never be driven, which is what shows the region-arity \
         vocabulary is not the wall this family is held by"
    );
}

/// The control: a family that *does* carry a law refines through this harness.
///
/// Without it, "the normalization refuses" would be consistent with a scalar
/// registry that authorizes nothing, a signature that matches nothing, or a
/// subject derivation that never works — and this file would prove nothing about
/// where the ceiling is.
#[test]
fn a_family_carrying_a_law_refines_through_the_identical_harness() {
    let scalars = scalar_registry();
    let resolved = resolved_capability(&multiply_f32_op(), Arc::new(PointwiseSquare), &scalars);

    let refinement = refine_index_region(
        &resolved,
        &multiply_subject(),
        &realization_laws(&scalars),
        &scalars,
    )
    .expect("the multiply family carries a realization law")
    .into_refined()
    .expect("the fixture discharges every index-domain predicate");

    assert_eq!(refinement.provider(), &provider("index"));
    assert_eq!(refinement.result_bindings().len(), 1);
}
