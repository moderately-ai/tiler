//! A two-region occurrence lowered through one index-access capability.
//!
//! **This file records two transitions rather than asserting one.** It began as
//! `two_region_occurrence_lowering_wall.rs`, whose finding was that the ceiling
//! [`lower-a-two-region-occurrence-through-one-index-access-capability`] was
//! filed against did not live where the ticket said. The ticket's premise was
//! that `IndexAccessLoweringProvider::lower` emitting one region per occurrence
//! is what held `tiler::rms-norm-f32@1` below R6, and that widening the
//! *compiler's* lowering vocabulary would release it. The first half was true
//! about the trait. The second was false: the refusal arrived from
//! `FrozenIndexRealizationLawRegistry::resolve`, one statement before
//! `refine_index_region` drove any provider, and the counter-instrumented
//! provider below observed exactly zero invocations.
//!
//! Both halves have now moved, and they moved in different directions:
//!
//! - **The vocabulary landed.** [`admit-a-multi-region-index-realization-law`]
//!   gave `tiler-ir` `StagedStrictSerialSumThenPointwiseF32` and
//!   `VerifiedIndexRegionSequence`, and this branch gave the compiler the
//!   consumer: a provider emits an ordered chain through
//!   `IndexAccessSequenceContext`, and `refine_index_region` proves the whole
//!   chain realizes the occurrence. The provider *is* driven, and the counter
//!   that used to read zero is what shows it.
//! - **The normalization is still held, by something else.** The staged law
//!   exists but no standard operation carries it: the normalization's fold is a
//!   sum of squares and its second stage applies a reciprocal square root, and
//!   no governed scalar operation spells either. Registering them is
//!   [`admit-the-rms-normalization-family`]'s work. So the refusal for
//!   `tiler::rms-norm-f32@1` is preserved below, retitled to say what it now
//!   attributes the ceiling to.
//!
//! Keeping the two together is the assertion. A staged occurrence that refines
//! and a normalization that does not, in one harness, is what distinguishes "the
//! lowering vocabulary cannot express a chain" — false since this branch — from
//! "this family has no law to be checked against", which is still true and is a
//! different ticket's work.
//!
//! [`lower-a-two-region-occurrence-through-one-index-access-capability`]: ../../../tickets/lower-a-two-region-occurrence-through-one-index-access-capability.md
//! [`admit-a-multi-region-index-realization-law`]: ../../../tickets/admit-a-multi-region-index-realization-law.md
//! [`admit-the-rms-normalization-family`]: ../../../tickets/admit-the-rms-normalization-family.md

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tiler_compiler::capability::{
    IndexAccessLoweringContext, IndexAccessLoweringProvider, IndexAccessSequenceContext,
    LoweringCapabilityRegistryBuilder, LoweringCapabilityRevision, LoweringEmitError,
    LoweringSignature,
};
use tiler_compiler::legality::{RefinementError, refine_index_region};
use tiler_ir::index::{
    DomainRole, FrozenIndexRealizationLawRegistry, FrozenScalarRegistry, IndexInteger,
    IndexRealizationLaw, IndexRefinementSubject, IndexRefinementVerificationError,
    IndexRegionSequenceError, NumericalContractIdentity, ScalarArity, ScalarAttributeSchema,
    ScalarAttributes, ScalarEffect, ScalarInferenceError, ScalarInferenceOutputs,
    ScalarInferenceRequest, ScalarOpKey, ScalarOperationContract, ScalarOperationDefinition,
    ScalarOperationInferencer, ScalarRegistryBuilder, StagedInputSource,
    canonicalize_nan_f32_scalar_op, multiply_f32_scalar_op,
};
use tiler_ir::semantic::{
    AttributeFieldId, CanonicalField, CanonicalValue, CanonicalValueKind, F32, F32Multiply,
    F32RmsNorm, FrozenSemanticRegistry, InputKey, NormativeDefinitionRef, OpKey, OperationArity,
    OperationAttributeSchema, OperationAttributes, OperationConformance, OperationDefinition,
    OperationDefinitionFacts, OperationEffect, OperationInferenceError, OperationInferenceOutputs,
    OperationInferenceRequest, OperationInferencer, OperationSchema, OutputKey,
    ProviderDiagnosticCode, ProviderIdentity, RegistryError, ResolvedValueType,
    SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
    SemanticRegistryRegistrar, multiply_f32_op, rms_norm_f32_op,
};
use tiler_ir::shape::{Axis, Extent, Shape};

/// The elementwise extent every staged fixture's second stage runs over.
const LENGTH: u64 = 4;

/// The folded extent.
///
/// One, deliberately. The staged law's fold is a strict lexicographic left fold,
/// and at a folded extent above one its tail is a reduction whose exact emission
/// a provider here would have to restate to match the law byte for byte. At
/// exactly one contributor the fold is the governed NaN canonicalization of a
/// single read — still a genuine stage publishing a rank-zero value the pass
/// consumes at every point, which is the whole shape under test, and stated in
/// four builder calls rather than fifteen. What is *not* under test here is the
/// tail fold's emission; `tiler-ir` owns that and pins it beside the law.
const FOLDED: u64 = 1;

/// Ordered axes attribute of the staged test operation's fold.
const STAGED_AXES_ATTRIBUTE: AttributeFieldId = AttributeFieldId::new(1);

fn contract() -> NumericalContractIdentity {
    NumericalContractIdentity::try_from_key(
        tiler_compiler::session::NumericalContract::STRICT_F32.key(),
    )
    .expect("the governed strict F32 contract key satisfies the IR bound")
}

fn f32_type() -> ResolvedValueType {
    F32::resolved_type()
}

fn provider(name: &str) -> ProviderIdentity {
    ProviderIdentity::new("example", name, 1).expect("the fixture provider identity is canonical")
}

fn revision() -> LoweringCapabilityRevision {
    LoweringCapabilityRevision::new(1).expect("revision 1 is nonzero")
}

/// The two-operand, one-result signature every family under test resolves.
fn binary_signature() -> LoweringSignature {
    LoweringSignature::new([f32_type(), f32_type()], [f32_type()])
        .expect("a two-operand signature is within the governed bound")
}

fn staged_operation() -> OpKey {
    OpKey::new("test", "staged-fold-then-pass", 1).expect("a canonical operation key")
}

fn staged_law() -> IndexRealizationLaw {
    IndexRealizationLaw::staged_strict_serial_sum_then_multiply_f32()
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

/// Result type and shape follow the *second* operand, the elementwise one.
struct StagedFoldThenPass;

impl OperationInferencer for StagedFoldThenPass {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let [_, elementwise] = request.operands() else {
            return Err(OperationInferenceError::new(
                ProviderDiagnosticCode::new("test.staged.signature")
                    .expect("a canonical diagnostic code"),
                "the staged test operation requires two operands",
            )
            .expect("a canonical inference error"));
        };
        outputs.try_push(elementwise.clone())
    }
}

/// Registers the staged test operation, optionally with a realization law.
///
/// The law row is optional so the fixture can also produce the *lawless*
/// spelling of the same operation, which is what the single-region perturbation
/// needs in order to isolate stage count from every other disagreement.
struct StagedOperationProvider(Option<IndexRealizationLaw>);

impl SemanticRegistryProvider for StagedOperationProvider {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("test", "staged-operation-provider", 1)
            .expect("a canonical provider identity")
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        let operation = staged_operation();
        registrar.register_operation(OperationDefinition::new(
            operation.clone(),
            OperationSchema::new(
                OperationArity::exact(2),
                OperationArity::exact(1),
                [OperationAttributeSchema::required(
                    STAGED_AXES_ATTRIBUTE,
                    CanonicalValueKind::Sequence,
                )],
            )
            .expect("the staged test schema is coherent"),
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

fn semantic_with_staged_operation(law: Option<IndexRealizationLaw>) -> FrozenSemanticRegistry {
    let mut builder =
        SemanticRegistryBuilder::standard().expect("the standard semantic registry is coherent");
    builder
        .register_provider(&StagedOperationProvider(law))
        .expect("registering one test operation against the standard authority succeeds");
    builder
        .freeze()
        .expect("the composed semantic authority is coherent")
}

fn binary_scalar(key: ScalarOpKey, normative: &str, operands: usize) -> ScalarOperationDefinition {
    ScalarOperationDefinition::new(
        key,
        NormativeDefinitionRef::from_owned(format!("urn:example:{normative}:v1"))
            .expect("a canonical normative reference"),
        ScalarOperationContract::new(
            ScalarAttributeSchema::empty(),
            ScalarArity::exact(operands).expect("a governed scalar operand arity"),
            ScalarArity::exact(1).expect("a one-result scalar arity"),
            ScalarEffect::Pure,
            CanonicalValue::record([]).expect("an empty canonical record"),
            CanonicalValue::record([]).expect("an empty canonical record"),
        ),
        Arc::new(SameType),
    )
}

/// The two scalar operations the staged realization reaches.
///
/// The fold's single contributor is canonicalized and the pass applies the
/// governed multiply; nothing here reaches the governed add, so registering it
/// would admit authority no stage under test uses.
fn scalar_registry(semantic: &FrozenSemanticRegistry) -> FrozenScalarRegistry {
    let mut builder = ScalarRegistryBuilder::new(semantic.clone());
    let scalars = provider("f32-scalars");
    for (key, normative, operands) in [
        (multiply_f32_scalar_op(), "multiply", 2),
        (canonicalize_nan_f32_scalar_op(), "canonicalize-nan", 1),
    ] {
        builder
            .register(scalars.clone(), binary_scalar(key, normative, operands))
            .expect("registering one scalar against the composed authority succeeds");
    }
    builder.freeze()
}

/// The scalar operations a staged capability declares it may emit.
fn staged_emitted() -> [ScalarOpKey; 2] {
    [multiply_f32_scalar_op(), canonicalize_nan_f32_scalar_op()]
}

// ---- the providers under test -------------------------------------------

/// Emits the fold stage: `out[] = canonicalize-nan(in[0])` over `[FOLDED]`.
///
/// The emission order mirrors the law's own — kept domain, input boundary,
/// output boundary, contributor read, canonicalization, write — because the
/// realization comparison is over exact canonical region identity and a
/// different order is a different region.
fn emit_fold(context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
    let input = context.input_tensor(f32_type(), Shape::from_dims([FOLDED]))?;
    let output = context.output_tensor(f32_type(), Shape::new([]))?;
    let zero = context.constant(IndexInteger::from_u64(0))?;
    let contributor = context.read(input, &[], &[zero])?;
    let canonical = context.apply(
        canonicalize_nan_f32_scalar_op(),
        ScalarAttributes::empty(),
        &[contributor],
    )?;
    let folded = canonical
        .get(0)
        .expect("the governed canonicalization yields one result");
    let write = context.write(output, &[], &[])?;
    context.output(write, folded)
}

/// Emits the pass stage: `out[i] = mul(in[i], folded)` over `[LENGTH]`.
fn emit_pass(context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
    let shape = Shape::from_dims([LENGTH]);
    let point = context.dimension(DomainRole::Parallel, Extent::new(LENGTH))?;
    let coordinate = context.dimension_expr(point)?;
    let elementwise = context.input_tensor(f32_type(), shape.clone())?;
    let intermediate = context.input_tensor(f32_type(), Shape::new([]))?;
    let left = context.read(elementwise, &[point], &[coordinate])?;
    let right = context.read(intermediate, &[], &[])?;
    let product = context.apply(
        multiply_f32_scalar_op(),
        ScalarAttributes::empty(),
        &[left, right],
    )?;
    let scaled = product
        .get(0)
        .expect("the governed multiply yields one result");
    let output = context.output_tensor(f32_type(), shape)?;
    let write = context.write(output, &[point], &[coordinate])?;
    context.output(write, scaled)
}

/// The ordered wiring the staged law declares.
fn staged_sources() -> [Vec<StagedInputSource>; 2] {
    [
        vec![StagedInputSource::Occurrence(0)],
        vec![
            StagedInputSource::Occurrence(1),
            StagedInputSource::Intermediate(0),
        ],
    ]
}

/// Emits the fold-then-pass chain, counting the times it is driven.
struct StagedProvider(Arc<AtomicUsize>);

impl IndexAccessLoweringProvider for StagedProvider {
    /// There is no single region realizing a staged occurrence, and saying so is
    /// the honest answer rather than dead code: it is exactly what
    /// `IndexRealizationLaw::realize` answers for the same law.
    fn lower(
        &self,
        _context: &mut IndexAccessLoweringContext<'_>,
    ) -> Result<(), LoweringEmitError> {
        Err(LoweringEmitError::Occurrence {
            rule: "staged-realization-requires-a-region-sequence",
        })
    }

    fn lower_sequence(
        &self,
        sequence: &mut IndexAccessSequenceContext<'_>,
    ) -> Result<(), LoweringEmitError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        let [fold, pass] = staged_sources();
        sequence.stage(&fold, emit_fold)?;
        sequence.stage(&pass, emit_pass)
    }
}

/// Emits the one region the multiply law builds.
struct SquareProvider;

impl IndexAccessLoweringProvider for SquareProvider {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        emit_square(context)
    }
}

/// Emits only the pass, which is the truncated half of the chain.
struct PassOnlyProvider;

impl IndexAccessLoweringProvider for PassOnlyProvider {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        emit_pass(context)
    }
}

/// Emits a well-formed chain whose second stage is the *same* region as its
/// first, which is a chain for an occurrence whose law declares one region.
struct DoubledProvider;

impl IndexAccessLoweringProvider for DoubledProvider {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        emit_square(context)
    }

    fn lower_sequence(
        &self,
        sequence: &mut IndexAccessSequenceContext<'_>,
    ) -> Result<(), LoweringEmitError> {
        sequence.stage(&[StagedInputSource::Occurrence(0)], emit_square)?;
        sequence.stage(&[StagedInputSource::Intermediate(0)], emit_square)
    }
}

/// Emits `out[i] = mul(in[i], in[i])`, the exact region the multiply law builds.
fn emit_square(context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
    let shape = Shape::from_dims([LENGTH]);
    let point = context.dimension(DomainRole::Parallel, Extent::new(LENGTH))?;
    let input = context.input_tensor(f32_type(), shape.clone())?;
    let output = context.output_tensor(f32_type(), shape)?;
    let coordinate = context.dimension_expr(point)?;
    let value = context.read(input, &[point], &[coordinate])?;
    let product = context.apply(
        multiply_f32_scalar_op(),
        ScalarAttributes::empty(),
        &[value, value],
    )?;
    let squared = product
        .get(0)
        .expect("the governed multiply yields one result");
    let write = context.write(output, &[point], &[coordinate])?;
    context.output(write, squared)
}

/// Emits a valid fold, then a second stage that declares no output at all and
/// discards the refusal the host handed back.
///
/// The discard is the point. A provider is trusted, and this one is wrong in the
/// specific way trust cannot cover: it loses the stage failure and reports
/// success, leaving a chain that is one stage short. What the host must not do
/// is compare that shorter chain against the law and report whatever the
/// comparison says.
struct SwallowingProvider;

impl IndexAccessLoweringProvider for SwallowingProvider {
    fn lower(
        &self,
        _context: &mut IndexAccessLoweringContext<'_>,
    ) -> Result<(), LoweringEmitError> {
        Err(LoweringEmitError::Occurrence {
            rule: "staged-realization-requires-a-region-sequence",
        })
    }

    fn lower_sequence(
        &self,
        sequence: &mut IndexAccessSequenceContext<'_>,
    ) -> Result<(), LoweringEmitError> {
        let [fold, pass] = staged_sources();
        sequence.stage(&fold, emit_fold)?;
        let _discarded = sequence.stage(&pass, |context| {
            context.input_tensor(f32_type(), Shape::from_dims([LENGTH]))?;
            context.input_tensor(f32_type(), Shape::new([]))?;
            Ok(())
        });
        Ok(())
    }
}

/// Refuses the occurrence before opening any stage.
///
/// The shape a provider takes when it reads the occurrence facts and finds them
/// outside the exact form it implements: there is nothing to emit, so it never
/// opens a stage and no stage failure is recorded.
struct RefusingSequenceProvider;

impl IndexAccessLoweringProvider for RefusingSequenceProvider {
    fn lower(
        &self,
        _context: &mut IndexAccessLoweringContext<'_>,
    ) -> Result<(), LoweringEmitError> {
        unreachable!("the host drives `lower_sequence`, which this provider overrides")
    }

    fn lower_sequence(
        &self,
        _sequence: &mut IndexAccessSequenceContext<'_>,
    ) -> Result<(), LoweringEmitError> {
        Err(LoweringEmitError::Occurrence {
            rule: "fixture-refuses-before-any-stage",
        })
    }
}

/// A provider that counts its invocations and otherwise emits nothing.
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

// ---- fixture assembly ----------------------------------------------------

fn subject_of(program: &tiler_ir::semantic::SemanticProgram) -> IndexRefinementSubject {
    let operation = program
        .operations()
        .next()
        .expect("the fixture program has one operation")
        .id();
    IndexRefinementSubject::derive(program, operation, contract())
        .expect("a standard occurrence derives a subject")
}

/// A one-occurrence staged program over `([FOLDED], [LENGTH]) -> [LENGTH]`.
fn staged_subject(semantic: &FrozenSemanticRegistry) -> IndexRefinementSubject {
    let mut builder = SemanticProgramBuilder::try_new(semantic.clone())
        .expect("a builder over the composed authority");
    let folded = builder
        .input::<F32>(
            InputKey::new("folded").expect("a canonical input key"),
            Shape::from_dims([FOLDED]),
        )
        .expect("declaring an input succeeds");
    let elementwise = builder
        .input::<F32>(
            InputKey::new("elementwise").expect("a canonical input key"),
            Shape::from_dims([LENGTH]),
        )
        .expect("declaring an input succeeds");
    let axes = CanonicalValue::sequence([CanonicalValue::unsigned_u32(0)])
        .expect("a one-element canonical sequence");
    let scaled = builder
        .apply(
            staged_operation(),
            OperationAttributes::new([CanonicalField::new(STAGED_AXES_ATTRIBUTE, axes)])
                .expect("one canonical attribute field"),
            &[folded.erase(), elementwise.erase()],
        )
        .expect("the staged operation applies")
        .pop()
        .expect("the staged operation yields one result");
    builder
        .output_resolved(
            OutputKey::new("scaled").expect("a canonical output key"),
            scaled,
        )
        .expect("declaring an output succeeds");
    subject_of(&builder.build().expect("the program verifies"))
}

/// A one-occurrence `tiler::multiply-f32@1` program over `[LENGTH]`.
fn multiply_subject(semantic: &FrozenSemanticRegistry) -> IndexRefinementSubject {
    let mut builder = SemanticProgramBuilder::try_new(semantic.clone())
        .expect("a builder over the composed authority");
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

/// A one-occurrence `tiler::rms-norm-f32@1` program over `[LENGTH]`.
fn rms_norm_subject(semantic: &FrozenSemanticRegistry) -> IndexRefinementSubject {
    let mut builder = SemanticProgramBuilder::try_new(semantic.clone())
        .expect("a builder over the composed authority");
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

/// Registers one index-access capability and resolves it against `operation`.
fn resolved_capability(
    semantic: &FrozenSemanticRegistry,
    scalars: &FrozenScalarRegistry,
    operation: &OpKey,
    emitted: &[ScalarOpKey],
    implementation: Arc<dyn IndexAccessLoweringProvider>,
) -> tiler_compiler::capability::ResolvedLoweringCapability {
    let mut builder = LoweringCapabilityRegistryBuilder::new(semantic.clone(), scalars.clone())
        .expect("the composed authorities cohere");
    builder
        .register_index_access(
            provider("index"),
            operation.clone(),
            binary_signature(),
            emitted,
            revision(),
            implementation,
        )
        .expect("registering an index-access capability succeeds");
    builder
        .freeze()
        .resolve_index_access(operation, &binary_signature())
        .expect("the registered capability resolves")
}

fn realization_laws(
    semantic: &FrozenSemanticRegistry,
    scalars: &FrozenScalarRegistry,
) -> FrozenIndexRealizationLawRegistry {
    FrozenIndexRealizationLawRegistry::from_semantic(semantic.clone(), scalars.clone())
        .expect("the composed semantic and scalar authorities cohere")
}

/// The complete staged authority set, with the law registered.
struct StagedFixture {
    semantic: FrozenSemanticRegistry,
    scalars: FrozenScalarRegistry,
    subject: IndexRefinementSubject,
}

impl StagedFixture {
    fn new() -> Self {
        let semantic = semantic_with_staged_operation(Some(staged_law()));
        let scalars = scalar_registry(&semantic);
        let subject = staged_subject(&semantic);
        Self {
            semantic,
            scalars,
            subject,
        }
    }

    fn refine(
        &self,
        implementation: Arc<dyn IndexAccessLoweringProvider>,
    ) -> Result<tiler_compiler::legality::IndexRefinementOutcome, RefinementError> {
        let resolved = resolved_capability(
            &self.semantic,
            &self.scalars,
            &staged_operation(),
            &staged_emitted(),
            implementation,
        );
        refine_index_region(
            &resolved,
            &self.subject,
            &realization_laws(&self.semantic, &self.scalars),
            &self.scalars,
        )
    }
}

// ---- the transition this file records ------------------------------------

/// The assertion that changed direction: the provider *is* driven, and the
/// two-region occurrence obtains verified coverage through the ordinary path.
///
/// The counter is retained from the wall this file used to record. It read zero
/// then because the law registry refused before `refine_index_region` reached
/// any provider. It reads one now, and every check below it is evidence the
/// chain the provider emitted is the chain the registered law requires — not
/// merely that a provider ran.
#[test]
fn a_two_region_occurrence_refines_and_binds_every_stage() {
    let fixture = StagedFixture::new();
    let driven = Arc::new(AtomicUsize::new(0));

    let refinement = fixture
        .refine(Arc::new(StagedProvider(driven.clone())))
        .expect("the staged occurrence refines through one index-access capability")
        .into_refined()
        .expect("the staged fixture discharges every index-domain predicate");

    assert_eq!(
        driven.load(Ordering::Relaxed),
        1,
        "the provider must be driven exactly once, which is what the wall this \
         file used to record made impossible"
    );

    // Two regions, one handed value, and the value is rank zero because the
    // fold removed the only axis and the pass reads it once per point.
    let realization = refinement.realization();
    assert_eq!(realization.stage_count(), 2);
    assert_eq!(refinement.content().stage_count(), 2);
    let [intermediate] = realization.intermediates() else {
        panic!("a two-stage chain hands exactly one value on")
    };
    assert_eq!(intermediate.producer(), 0);
    assert_eq!(intermediate.consumer(), 1);
    assert_eq!(intermediate.shape().rank(), 0);
    assert_eq!(intermediate.value_type(), &f32_type());

    // No single-region view of a chain is offered, because the final stage reads
    // a value no occurrence operand carries.
    assert!(refinement.single_region().is_none());

    // Every stage's reached scalar authority is retained, and the two genuinely
    // differ: the fold reaches the canonicalization and nothing else, the pass
    // the multiply and nothing else.
    let authorities = refinement.content().scalar_authorities();
    assert_eq!(authorities.len(), 2);
    assert_ne!(authorities[0], authorities[1]);
    assert_eq!(
        authorities[0].reached_operations(),
        [canonicalize_nan_f32_scalar_op()]
    );
    assert_eq!(
        authorities[1].reached_operations(),
        [multiply_f32_scalar_op()]
    );

    // The folded operand is read by the fold and the elementwise operand by the
    // pass, so the bindings name two different stages.
    assert_eq!(
        refinement
            .operand_bindings()
            .iter()
            .map(|binding| (binding.operand(), binding.stage()))
            .collect::<Vec<_>>(),
        vec![(0, 0), (1, 1)]
    );
    assert_eq!(refinement.result_bindings().len(), 1);
    assert_eq!(
        refinement.receipt().realization(),
        realization.identity(),
        "the compiler's retained realization is the one the IR receipt bound"
    );
}

/// One region for a two-region occurrence, and a chain for a one-region one.
///
/// The ticket's own perturbation, in both directions. Neither candidate is
/// malformed — each stage verifies on its own — so nothing about their own
/// construction says no.
///
/// **Where each direction is caught differs, and that is worth asserting
/// exactly rather than as "some refusal".** A truncated chain drops the fold, so
/// the pass's first boundary now claims to be the occurrence's first expanded
/// input and disagrees with it — the elementwise `[LENGTH]` against the folded
/// `[FOLDED]` — and the ordered interface check names that boundary one
/// statement before the realization comparison runs. A chain presented for a
/// one-region law binds cleanly at every interface and is caught by the
/// comparison itself. A test satisfied by any refusal would pass for a fixture
/// that had stopped building realizations at all.
#[test]
fn a_realization_of_the_wrong_stage_count_refuses_with_a_typed_reason() {
    let fixture = StagedFixture::new();
    let error = fixture
        .refine(Arc::new(PassOnlyProvider))
        .expect_err("one region cannot realize a staged occurrence");
    assert_eq!(
        error,
        RefinementError::OperandInterface { position: 0 },
        "a truncated chain must be refused at the boundary that disagrees, not \
         accepted as a shorter realization"
    );

    // The other direction: `tiler::multiply-f32@1` carries a single-region law,
    // and the doubled chain is well formed — squaring twice, the second stage
    // reading the first's result, agrees with the occurrence at every interface.
    // Nothing but the whole-realization comparison can refuse it.
    let semantic = semantic_with_staged_operation(Some(staged_law()));
    let scalars = scalar_registry(&semantic);
    let resolved = resolved_capability(
        &semantic,
        &scalars,
        &multiply_f32_op(),
        &[multiply_f32_scalar_op()],
        Arc::new(DoubledProvider),
    );
    let error = refine_index_region(
        &resolved,
        &multiply_subject(&semantic),
        &realization_laws(&semantic, &scalars),
        &scalars,
    )
    .expect_err("a chain cannot certify a law that declares one region");
    let RefinementError::IrVerifier(source) = &error else {
        panic!("the refusal must come from the IR realization authority; observed {error:?}")
    };
    assert!(
        matches!(
            **source,
            IndexRefinementVerificationError::SemanticRealizationSequenceMismatch { .. }
        ),
        "observed {source:?}"
    );
}

/// A chain's compiler-side identities are domain-separated from a region's.
///
/// **The one-stage half is the load-bearing one.** A one-stage sequence's
/// identity is its region's identity byte for byte and a one-stage realization
/// retains no leading stage, so every refinement content and occurrence binding
/// the compiler has ever minted must still encode under the tags it always did —
/// otherwise the staged vocabulary's arrival would silently invalidate every
/// pinned refinement identity in the corpus for a change none of them made. The
/// staged half is then written under its own tags, and neither tag is a prefix
/// of the other, so no chain can spell a single-region binding.
#[test]
fn a_chain_and_a_region_encode_under_disjoint_identity_domains() {
    const CONTENT: &[u8] = b"tiler.compiler.index-refinement-content.v2\0";
    const STAGED_CONTENT: &[u8] = b"tiler.compiler.index-refinement-content.staged.v1\0";
    const OCCURRENCE: &[u8] = b"tiler.compiler.index-refinement-occurrence.v2\0";
    const STAGED_OCCURRENCE: &[u8] = b"tiler.compiler.index-refinement-occurrence.staged.v1\0";

    assert!(!CONTENT.starts_with(STAGED_CONTENT) && !STAGED_CONTENT.starts_with(CONTENT));
    assert!(
        !OCCURRENCE.starts_with(STAGED_OCCURRENCE) && !STAGED_OCCURRENCE.starts_with(OCCURRENCE)
    );

    let fixture = StagedFixture::new();
    let staged = fixture
        .refine(Arc::new(StagedProvider(Arc::new(AtomicUsize::new(0)))))
        .expect("the staged occurrence refines")
        .into_refined()
        .expect("the staged fixture discharges every index-domain predicate");
    assert!(
        staged
            .content()
            .identity()
            .as_bytes()
            .starts_with(STAGED_CONTENT)
    );
    assert!(staged.identity().as_bytes().starts_with(STAGED_OCCURRENCE));

    let semantic = semantic_with_staged_operation(Some(staged_law()));
    let scalars = scalar_registry(&semantic);
    let resolved = resolved_capability(
        &semantic,
        &scalars,
        &multiply_f32_op(),
        &[multiply_f32_scalar_op()],
        Arc::new(SquareProvider),
    );
    let single = refine_index_region(
        &resolved,
        &multiply_subject(&semantic),
        &realization_laws(&semantic, &scalars),
        &scalars,
    )
    .expect("the multiply family carries a single-region realization law")
    .into_refined()
    .expect("the square fixture discharges every index-domain predicate");
    assert_eq!(single.content().stage_count(), 1);
    assert!(single.content().identity().as_bytes().starts_with(CONTENT));
    assert!(single.identity().as_bytes().starts_with(OCCURRENCE));
    // A one-stage realization identity is the region's own bytes, which is what
    // keeps every previously minted content identity unchanged.
    assert_eq!(
        single.content().realization_identity().as_bytes(),
        single
            .single_region()
            .expect("a one-stage realization offers its region")
            .canonical_identity()
            .as_bytes()
    );
}

/// A provider that loses a stage failure still refuses, and names the stage.
///
/// Deliberately perturbed in the direction that would be worst: the discarded
/// failure leaves a one-stage chain that is *structurally* fine, so a host
/// trusting the provider's `Ok` would run the realization comparison and report
/// a stage-count disagreement — a true statement about a chain the provider
/// never meant to emit, sending a reader to the law instead of to the region
/// that failed verification.
#[test]
fn a_swallowed_stage_failure_is_still_the_hosts_refusal() {
    let fixture = StagedFixture::new();
    let error = fixture
        .refine(Arc::new(SwallowingProvider))
        .expect_err("a stage that failed verification cannot be discarded by its provider");
    assert!(
        matches!(error, RefinementError::Build { stage: 1, .. }),
        "the refusal must name the stage whose region was rejected; observed {error:?}"
    );
}

/// A provider that emits nothing reports its own refusal, not an empty chain.
///
/// **Both routes to "nothing was emitted" are asserted, because they arrive by
/// different paths and only one of them records anything.** A provider taking
/// the default `lower_sequence` refuses *inside* the stage the host opened for
/// it, so the failure is recorded with that stage's ordinal and the host reads
/// it back. A provider overriding `lower_sequence` can refuse before opening any
/// stage at all — the natural shape for one that inspects the occurrence facts
/// and finds them outside the form it implements — and then nothing is recorded,
/// the retained stage list is empty, and composition would answer
/// `IndexRegionSequenceError::Empty`. Reporting that would replace "this
/// provider does not lower this occurrence" with "no stage was emitted", which
/// sends a reader to the chain instead of to the provider's stated rule.
#[test]
fn a_provider_that_emits_nothing_reports_its_own_refusal() {
    let fixture = StagedFixture::new();

    let driven = Arc::new(AtomicUsize::new(0));
    let error = fixture
        .refine(Arc::new(RecordingProvider(driven.clone())))
        .expect_err("a provider emitting nothing realizes nothing");
    assert_eq!(
        driven.load(Ordering::Relaxed),
        1,
        "the default `lower_sequence` opens one stage and drives `lower` inside it"
    );
    assert_eq!(
        error,
        RefinementError::Emit {
            stage: 0,
            source: LoweringEmitError::Occurrence {
                rule: "fixture-never-reached",
            },
        }
    );

    let error = fixture
        .refine(Arc::new(RefusingSequenceProvider))
        .expect_err("a provider that opens no stage realizes nothing");
    assert_eq!(
        error,
        RefinementError::Emit {
            stage: 0,
            source: LoweringEmitError::Occurrence {
                rule: "fixture-refuses-before-any-stage",
            },
        },
        "the provider's own rule must survive; an empty chain is the \
         composition's complaint about it, not the refusal"
    );
    // The composition's complaint is a distinct, reachable refusal rather than
    // the one above wearing another name.
    assert_ne!(
        error,
        RefinementError::Realization {
            source: IndexRegionSequenceError::Empty,
        }
    );
}

// ---- the half that has not moved -----------------------------------------

/// The normalization is still held, and now by exactly one thing.
///
/// **Retitled rather than deleted.** While the lowering vocabulary was the
/// suspect, this assertion's job was to acquit it. The vocabulary has since
/// landed and the staged occurrence above refines through it, so what this now
/// records is the residue: `tiler::rms-norm-f32@1` registers a lowering
/// capability and resolves it, and refinement still refuses before the provider
/// is driven, because the standard registration deliberately registers no
/// realization law for the family. The law it would carry is the staged form the
/// tests above exercise; what is missing is a governed scalar operation for its
/// sum of squares and its reciprocal square root, which is
/// `admit-the-rms-normalization-family`'s work and not this file's.
#[test]
fn the_normalization_still_refuses_for_an_absent_law_and_not_for_the_vocabulary() {
    let semantic =
        FrozenSemanticRegistry::standard().expect("the standard semantic registry is coherent");
    let scalars = scalar_registry(&semantic);
    let driven = Arc::new(AtomicUsize::new(0));
    let resolved = resolved_capability(
        &semantic,
        &scalars,
        &rms_norm_f32_op(),
        &staged_emitted(),
        Arc::new(RecordingProvider(driven.clone())),
    );

    // Registration and resolution succeed and drive nothing: the realization-law
    // row is recorded as an `Option` and never required, so a reader who saw
    // only the refusal below might otherwise conclude the registry had rejected
    // the family outright.
    assert_eq!(resolved.operation(), &rms_norm_f32_op());
    assert_eq!(driven.load(Ordering::Relaxed), 0);

    let error = refine_index_region(
        &resolved,
        &rms_norm_subject(&semantic),
        &realization_laws(&semantic, &scalars),
        &scalars,
    )
    .expect_err("a family with no realization law cannot be refined");

    let RefinementError::IrVerifier(source) = &error else {
        panic!("the refusal must come from the IR realization authority; observed {error:?}")
    };
    assert!(
        matches!(
            **source,
            IndexRefinementVerificationError::MissingRealizationLaw
        ),
        "the refusal must name the absent law; observed {source:?}"
    );
    assert_eq!(
        driven.load(Ordering::Relaxed),
        0,
        "resolution still precedes emission, so the ceiling this family is held \
         by remains its missing law rather than anything a provider emits"
    );
}
