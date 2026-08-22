//! The vertical, carried end to end.
//!
//! One bounded scalar CPU implementation runs from a declared target profile,
//! through the compiler's verified physical work, into an independently
//! identified executable representation, into a real artifact payload, through
//! device-free validation, through a live host execution context, across the
//! routing commit, into execution, and finishes at a bitwise comparison against
//! `tiler-reference`.
//!
//! # What this vertical is evidence about, and what it is not
//!
//! It is evidence that the accepted target-profile, artifact, and runtime
//! contracts admit a **second, materially different backend** without editing
//! them: a different governed profile key and descriptor, a different backend
//! family and executable representation, a different execution model, a
//! different numerical row, and a different second-stage preflight, all through
//! the same public boundaries `prototypes/serial-sum-run` drives for Metal.
//! Where the fit was imperfect, the README records the exact seam rather than
//! this file working around it silently.
//!
//! It is **not** a production CPU backend, a claim that this profile is complete,
//! a performance claim, or a claim about any host other than the one a run was
//! taken on. No `tiler-cpu` crate is scaffolded and no production support is
//! implied.
//!
//! # One process, and where that weakens the evidence
//!
//! `prototypes/serial-sum-compile` and `prototypes/serial-sum-run` are separate
//! crates that share no code, so their agreement is evidence about a delivery
//! mechanism. This spike is one binary, so the artifact identity it checks
//! against is one it computed itself, and that check is correspondingly a
//! tautology. It is retained anyway because the *rest* of the load path is not:
//! the envelope really is encoded to bytes and decoded back through
//! `tiler_artifact`, the payload really is serialized and decoded through an
//! encoding that knows nothing about `VerifiedKernel`, and every perturbation
//! below acts on those bytes. Splitting the producer from the consumer is the
//! obvious next increment and is recorded in the README as a boundary rather
//! than claimed.

use std::collections::BTreeMap;
use std::fmt;

use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, ApproximationEnvelope, ArithmeticType, ArtifactBuildError,
    ArtifactCodecFailure, ArtifactExecutionPolicy, ArtifactProgramBuilder, AvailabilityPhase,
    BackendEntryKey, BackendEntryRef, BackendKey, BindingKind, BindingSpec, BindingTarget,
    CANONICAL_DIMENSIONS, CapabilityFamilyKey, CompilationEnvironment, DIMENSION_COUNT,
    DeliveredRealizationBuilder, DeliveredRealizationRecord, DimensionBehaviour, EntryRealization,
    EntrySpec, FactSourceProvenance, FeasibilityRuleSetKey, FeasibilityRuleSetRef, HonouringMeans,
    LaunchSpec, LoweringCapabilitySubject, MaterializationRounding, NumericalDimension,
    NumericalObligationKey, NumericalPermission, PayloadContent, PayloadEntryMapping,
    PayloadMetadata, PayloadPlatform, PayloadProvenance, PolicyLocus, ProvenanceIdentity,
    RecordedArtifactIdentityError, RecordedArtifactProgramIdentity, RepresentationKey,
    ScalarArithmeticSubject, SchemaVersion, SelectedProvider, SemanticOccurrence,
    TargetEvidenceDeclaration, TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
    ToolComponent, VariantSpec, VerifiedArtifactProgram, overlapping_behaviour,
};
use tiler_compiler::session::{
    Compilation, CompileFailure, CompileRequest, NumericalContract, PlanAlternative, compile,
};
use tiler_compiler::target::{TargetProfileBuildError, TargetRequest};
use tiler_ir::kernel::{AddressSpace, BufferAccess, BufferParameter, KernelType};
use tiler_ir::schedule::TensorRole;
use tiler_ir::semantic::{
    F32, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};
use tiler_runtime::load::{
    DTypeDispatch, DecodedProgram, ExecutionEnvironment, FilteredVariant, LoadRejection, Preflight,
    TargetCompatibility, VariantIneligibility,
};

use crate::host::{HostExecutionContext, HostRefusal};
use crate::image::{
    self, ImageDecodeError, ImageNumerics, Instruction, ScalarEntry, ScalarImage, TranslationError,
};
use crate::interpret::{self, ExecutionError, Placement};
use crate::profile;

/// Rows of the workload; each row is three independent elements.
const ROWS: u64 = 4;
/// Columns of the workload.
const COLUMNS: u64 = 3;
/// Interface key of the program's one input.
const INPUT_KEY: &str = "input";
/// Interface key of the program's one output.
const OUTPUT_KEY: &str = "result";
/// Byte width of one `f32`.
const F32_BYTES: u64 = 4;

/// The one delivery position this vertical's artifact is built for.
///
/// A delivery position is the ordered slot a consumer's build target resolves
/// to, and [`assemble`] pushes exactly one carried payload and names it once per
/// entry, so this artifact declares one position and zero is the only one in
/// range — `DecodedProgram::decode` refuses any other by construction rather
/// than by convention. Named rather than written as a bare `0` at each call,
/// because the argument decides *which compiled object* is loaded and a literal
/// there says nothing about why that one; this is the spelling
/// `prototypes/serial-sum-run` and `prototypes/candle-metal-adapter` already use
/// for the same single-target case.
const SOLE_DELIVERY: usize = 0;

/// The operand pattern the workload is filled from.
///
/// Chosen to exercise the numerical contract rather than to be arithmetically
/// convenient. Under the strict contract this profile declares, every one of
/// these is a value where a backend either honours the contract or is silently
/// wrong: a negative zero whose sign must survive a multiply by one, the least
/// positive subnormal that a flushing host would turn into a zero, a
/// non-canonical NaN payload that arithmetic must canonicalize to the
/// realization's exact pattern, and an infinity.
const OPERANDS: [u32; 12] = [
    0x3f80_0000, // 1.0
    0x8000_0000, // -0.0
    0x0000_0001, // least positive subnormal
    0x7fc0_1234, // a non-canonical NaN payload
    0x7f80_0000, // +inf
    0xff80_0000, // -inf
    0xbf80_0000, // -1.0
    0x8000_0001, // least negative subnormal
    0x4000_0000, // 2.0
    0x0080_0000, // least positive normal
    0x477f_e000, // 65504.0
    0xc0c0_0000, // -6.0
];

/// The backend entry symbol one packaged stage is published under.
///
/// The scalar image is looked up by this symbol exactly as a `metallib` is
/// looked up by a function name — the artifact's neutral entry key names the
/// kernel, and the payload's own mapping names the backend's spelling of it.
/// Deriving the symbol from the stage ordinal rather than from the kernel
/// identity keeps the payload's spelling the backend's own, which is what the
/// entry mapping exists to express.
fn entry_symbol(stage: usize) -> String {
    format!("tiler_cpu_scalar_entry_{stage}")
}

/// Builds the smallest scalar program this build's semantic, normalization, and
/// reference layers all admit.
///
/// **Smallest is a measured claim, not a preference.** The compiler's
/// normalization admits exactly two program shapes, and the pointwise one is
/// bounded to four operations forming a root binary operation over a child of
/// *the same operation family* plus its constants: `crates/tiler-compiler/src/
/// request.rs`'s `normalize_pointwise` refuses `signature` for any other
/// operation count and `pointwise-association` when the child's key differs
/// from the root's. `(input * 2.0) * 1.0` is that shape at its minimum. The
/// obvious smaller-looking `(input * 2.0) + 1.0` was tried first and refused
/// with `UnsupportedCapability { rule: "pointwise-association" }`; the reduction
/// program `prototypes/serial-sum-run` uses is one operation larger.
fn scalar_program(rows: u64, columns: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed semantic profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new(INPUT_KEY).expect("the input key is valid"),
            Shape::from_dims([rows, columns]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).expect("the scale applies");
    let unit = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the unit applies");
    let scaled = F32Multiply::apply(&mut builder, input, scale).expect("the scaling applies");
    let mapped = F32Multiply::apply(&mut builder, scaled, unit).expect("the unit multiply applies");
    builder
        .output(
            OutputKey::new(OUTPUT_KEY).expect("the output key is valid"),
            mapped,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Evaluates the same semantic program through the independent oracle.
///
/// Returns the output bits **and** the reference registry's own canonical
/// identity. The two identities are deliberately reported side by side and
/// never conflated: the artifact identity names *what was packaged and
/// executed*, and the reference registry identity names *which oracle the
/// comparison was against*. A run that recorded one number could not say which
/// of the two changed when the answer did.
fn reference_bits(
    program: &SemanticProgram,
    operands: &[u32],
    rows: u64,
    columns: u64,
) -> (Vec<u32>, Vec<u8>) {
    let key = InputKey::new(INPUT_KEY).expect("the input key is valid");
    let tensor = Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([rows, columns]),
        operands
            .iter()
            .map(|value| {
                ReferenceElement::from_float_bits(
                    value.to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("the operand is a valid f32 pattern")
            })
            .collect(),
    )
    .expect("the input tensor is well formed");
    let evaluator =
        ReferenceEvaluator::standard().expect("the governed reference profile composes");
    let identity = evaluator
        .registry()
        .canonical_identity()
        .as_bytes()
        .to_vec();
    let outputs = evaluator
        .evaluate(program, &[InputBinding::new(&key, &tensor)])
        .expect("the reference evaluates the program");
    let TensorPayloadView::Dense(elements) = outputs[0].payload() else {
        panic!("this program declares one dense f32 output");
    };
    let bits = elements
        .iter()
        .map(|element| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
            )
        })
        .collect();
    (bits, identity)
}

/// States what this host offers, from the compiler's own target authority.
///
/// The profile half comes from the compilation rather than from the artifact,
/// for the reason `prototypes/serial-sum-run` states: reading it back out of the
/// artifact would make `ExecutionEnvironment::classify` a tautology. The backend
/// and representation halves are this backend's own governed keys, and they are
/// what make the environment refuse a Metal payload without any profile
/// comparison being involved.
fn host_environment(compilation: &Compilation) -> Result<ExecutionEnvironment, VerticalError> {
    Ok(ExecutionEnvironment {
        target_profile: TargetProfileRef {
            key: TargetProfileKey::new(compilation.target_profile_key())
                .map_err(|_| VerticalError::HostProfile)?,
            descriptor: TargetProfileDescriptorDigest::from_bytes(
                compilation.target_profile_descriptor(),
            )
            .map_err(|_| VerticalError::HostProfile)?,
        },
        backend: BackendKey::new(profile::BACKEND_KEY).map_err(|_| VerticalError::HostProfile)?,
        representation: RepresentationKey::new(profile::REPRESENTATION_KEY)
            .map_err(|_| VerticalError::HostProfile)?,
        // This vertical's own scalar CPU backend interprets `f32` and nothing
        // else, so it declares that and stays silent about every other width —
        // silence being the fail-closed answer rather than a permissive default.
        dtype_dispatch: BTreeMap::from([(ArithmeticType::F32, DTypeDispatch::Dispatchable)]),
    })
}

/// Packages one plan alternative and its scalar image as an artifact.
///
/// The payload's compilation subject names what actually produced it: the
/// governed source representation is the list of canonical kernel identities the
/// image was translated from, the retained "source" is those exact bytes, and
/// the provenance names this spike and this Rust toolchain rather than an SDK
/// nobody consulted. That matters because the payload digest folds the whole
/// subject: a payload claiming an Apple SDK it never saw would give two
/// materially different compilations one identity.
#[allow(
    clippy::too_many_lines,
    reason = "one artifact is assembled top to bottom in the order the builder requires, and that order is the readable part"
)]
fn assemble(
    semantic: &SemanticProgram,
    compilation: &Compilation,
    plan: PlanAlternative<'_>,
    image_bytes: Vec<u8>,
    source: Vec<u8>,
) -> Result<VerifiedArtifactProgram, VerticalError> {
    let profile_ref = TargetProfileRef {
        key: TargetProfileKey::new(compilation.target_profile_key())
            .map_err(|_| VerticalError::HostProfile)?,
        descriptor: TargetProfileDescriptorDigest::from_bytes(
            compilation.target_profile_descriptor(),
        )
        .map_err(|_| VerticalError::HostProfile)?,
    };
    let rules = FeasibilityRuleSetRef {
        key: FeasibilityRuleSetKey::new(compilation.feasibility_rule_set_key())
            .map_err(|_| VerticalError::HostProfile)?,
        revision: compilation.feasibility_rule_set_revision(),
    };

    let environment = CompilationEnvironment::new(
        plan.selected_capabilities()
            .map(|selected| selected.provider().clone()),
    )
    .map_err(|_| VerticalError::Package("the offered providers do not compose"))?;
    let mut builder = ArtifactProgramBuilder::new(semantic, environment)
        .map_err(|_| VerticalError::Package("a builder identity was not available"))?;
    for selected in plan.selected_capabilities() {
        let subject = selected.subject();
        builder
            .select_provider(SelectedProvider {
                provider: selected.provider().clone(),
                capability: LoweringCapabilitySubject {
                    family: CapabilityFamilyKey::new(subject.family().key_token())
                        .map_err(VerticalError::ArtifactBuild)?,
                    operation: subject.operation().clone(),
                },
                capability_revision: selected.capability_revision(),
            })
            .map_err(|_| VerticalError::Package("a selected provider was not offered"))?;
    }

    let program = plan.abi().kernel_program();
    let mut mappings: Vec<PayloadEntryMapping> = program
        .stages()
        .enumerate()
        .map(|(position, stage)| {
            Ok(PayloadEntryMapping {
                entry_key: BackendEntryKey::from_bytes(
                    stage.kernel().canonical_identity().as_bytes(),
                )
                .map_err(|_| VerticalError::Package("a kernel identity exceeds an entry key"))?,
                symbol: entry_symbol(position),
                // The identity mapping, and it is a choice rather than an
                // absence: a scalar entry's transports *are* its ABI slots
                // because a host binds storage by position in the signature and
                // there is no argument table to place them in. Metal's mapping
                // is not the identity in general, which is exactly why the
                // artifact carries one instead of assuming it.
                transports: (0..u32::try_from(stage.accesses().len())
                    .map_err(|_| VerticalError::Package("a binding count exceeds a u32"))?)
                    .collect(),
            })
        })
        .collect::<Result<_, _>>()?;
    mappings.sort_by(|left, right| left.entry_key.cmp(&right.entry_key));

    let payload = builder
        .push_carried_payload(
            BackendKey::new(profile::BACKEND_KEY).map_err(|_| VerticalError::HostProfile)?,
            RepresentationKey::new(profile::REPRESENTATION_KEY)
                .map_err(|_| VerticalError::HostProfile)?,
            SchemaVersion::new(1, 0),
            profile_ref.clone(),
            // A native image, and the claim is narrow rather than convenient.
            // `ArtifactExecutionPolicy`'s own documentation says this value
            // means "the target's own API loads these bytes as they stand", and
            // that the enum answers delivery alone rather than whether a device
            // does work of its own between a load and a dispatch. This image
            // satisfies the strict reading of that one question: the bytes are
            // decoded and executed as they stand, with no target-specific
            // translation, specialization, or pipeline object between them and
            // the dispatch. The README records that the vocabulary is
            // nonetheless GPU-shaped, with no way to say "an interpreted image",
            // "a JIT input", or "a dynamically linked object".
            ArtifactExecutionPolicy::NativeImage,
            // No target-environment declaration. A `TargetEnvironmentDeclaration`
            // is a declaration and never an attestation, and positive support
            // needs an independently selected authority exposing the exact
            // `TargetEnvironmentDescriptorSchema`. This spike registers no
            // provider schema, so stating one here would assert a compatibility
            // class nothing can validate. `None` keeps every cell `Unclaimed`
            // and routable while claiming nothing.
            None,
            PayloadContent {
                metadata: PayloadMetadata {
                    source_representation: RepresentationKey::new(
                        profile::SOURCE_REPRESENTATION_KEY,
                    )
                    .map_err(|_| VerticalError::HostProfile)?,
                    source,
                    provenance: PayloadProvenance {
                        toolchain: "tiler.cpu.scalar-image-translator".to_owned(),
                        target: profile::TARGET_TRIPLE.to_owned(),
                        family: "tiler.cpu.scalar".to_owned(),
                        language: "tiler.kernel-ir.v4".to_owned(),
                        // This translator resolves against no SDK and requests
                        // no platform deployment minimum, and it now says so.
                        // Finding 7 recorded that it could not: the fields were
                        // unconditional, and this spike stated its own
                        // representation version in them.
                        platform: PayloadPlatform::Unversioned,
                        components: vec![ToolComponent {
                            role: "translator".to_owned(),
                            version: "1".to_owned(),
                        }],
                        compile_flags: Vec::new(),
                        link_flags: Vec::new(),
                    },
                    entries: mappings,
                    obligations: Vec::new(),
                },
                code: image_bytes,
            },
        )
        .map_err(|_| VerticalError::Package("the scalar payload was refused"))?;

    let entries: Vec<EntrySpec> = program
        .stages()
        .map(|stage| {
            Ok(EntrySpec {
                bindings: stage
                    .accesses()
                    .map(|_| BindingSpec {
                        kind: BindingKind::Buffer,
                    })
                    .collect(),
                launch: LaunchSpec {
                    zero_work_skips_dispatch: true,
                    preconditions: Vec::new(),
                },
                implementation: BackendEntryRef {
                    // One payload, because this spike compiles one object for
                    // one consumer build target. The list is positional against
                    // the artifact's delivery positions, so a second entry here
                    // would be a second compiled object rather than a second
                    // plan — and the position a run loads is `SOLE_DELIVERY`.
                    payloads: vec![payload],
                    entry_key: BackendEntryKey::from_bytes(
                        stage.kernel().canonical_identity().as_bytes(),
                    )
                    .map_err(|_| {
                        VerticalError::Package("a kernel identity exceeds an entry key")
                    })?,
                },
            })
        })
        .collect::<Result<_, _>>()?;

    builder
        .push_variant(
            program,
            VariantSpec {
                target_profile: profile_ref.clone(),
                feasibility_rules: rules,
                // Empty, and that is the measured CPU result rather than an
                // omission: this profile declares its workgroup bound as an
                // available compile-time fact, so the plan defers no
                // prepared-entry property and the device-free `preflight` path
                // is sufficient. The Metal profile cannot do this.
                deferred_predicates: plan
                    .prepared_entry_target_requirements()
                    .map(
                        |requirement| tiler_artifact::program::DeferredPredicateSpec {
                            requirement: requirement.requirement().clone(),
                            entry: requirement.entry(),
                        },
                    )
                    .collect(),
                entries,
            },
        )
        .map_err(|_| VerticalError::Package("the variant does not package the bound plan"))?;
    builder
        .declare_realization(realization_record(
            &profile_ref,
            EntryRealization::of(
                program
                    .stages()
                    .next()
                    .ok_or(VerticalError::Package("the packaged program has no stage"))?
                    .kernel()
                    .numerical(),
            ),
            u32::try_from(program.stages().len())
                .map_err(|_| VerticalError::Package("the stage table exceeds a u32"))?,
        ))
        .map_err(|_| VerticalError::Package("the record does not agree with the portfolio"))?;
    builder
        .build()
        .map_err(|_| VerticalError::Package("the assembled artifact does not verify"))
}

/// Builds the delivered-realization record every executable artifact carries.
///
/// The eleven resolutions are derived from the packaged program's own scheduled
/// realization rather than restated here, so this harness cannot describe a
/// contract its plan does not schedule. `entries` is the flat declared
/// packaged-entry count; one variant means it is the program's stage count.
///
/// The obligation is stated at the computation locus of occurrence 0 with
/// `SupportedExactly` means, which is what the strict contract this spike
/// compiles under actually rests on. A harness that invented a relaxation it did
/// not need would be writing a fact rather than recording one.
fn realization_record(
    profile: &TargetProfileRef,
    entry: EntryRealization,
    entries: u32,
) -> DeliveredRealizationRecord {
    let mut resolutions =
        [DimensionBehaviour::Transform(NumericalPermission::Forbidden); DIMENSION_COUNT];
    for dimension in CANONICAL_DIMENSIONS {
        resolutions[dimension.index()] =
            overlapping_behaviour(dimension, entry).unwrap_or(match dimension {
                NumericalDimension::ApproximateIntrinsics => {
                    DimensionBehaviour::Approximation(ApproximationEnvelope::Forbidden)
                }
                NumericalDimension::MaterializationRounding => {
                    DimensionBehaviour::Rounding(MaterializationRounding::NearestTiesToEven)
                }
                // Reciprocal transform, the third dimension no scheduled
                // realization carries. The remaining arm rather than a wildcard
                // over all eleven, so a dimension leaving the overlapping set
                // stops the build here.
                _ => DimensionBehaviour::Transform(NumericalPermission::Forbidden),
            });
    }
    let subject = ScalarArithmeticSubject::f32().identity();
    let mut record = DeliveredRealizationBuilder::new(profile.clone());
    record
        .declare_scalar_arithmetic(subject.clone(), resolutions)
        .expect("the selected scalar contract");
    record
        .require(
            &subject,
            NumericalDimension::Contraction,
            NumericalObligationKey::new(SemanticOccurrence::new(0), PolicyLocus::Computation),
            resolutions[NumericalDimension::Contraction.index()],
            TargetEvidenceDeclaration {
                declared: resolutions[NumericalDimension::Contraction.index()],
                means: HonouringMeans::SupportedExactly,
                profile: profile.clone(),
                source: FactSourceProvenance::governed(
                    ProvenanceIdentity::new(profile.key.as_str(), 1),
                    ProvenanceIdentity::new("tiler.spike.strict-f32-guarantee", 1),
                ),
            },
        )
        .expect("the obligation this packaged route relies on");
    for entry in 0..entries {
        record
            .bind_entry(entry, &subject)
            .expect("a packaged entry");
    }
    record.build().expect("the record this artifact delivers")
}

/// Reads the artifact's declared interface and binds its extents.
///
/// Read rather than asserted, for the reason `prototypes/serial-sum-run` states:
/// asserting a shape here would replace the artifact's declaration with this
/// build's expectation, and the two halves would then agree because they were
/// told to.
fn bind_interface(decoded: &DecodedProgram) -> Result<(u64, u64, AbiFacts), VerticalError> {
    let f32_type = F32::resolved_type().canonical_encoding();
    let inputs: Vec<_> = decoded.inputs().collect();
    let [input] = inputs.as_slice() else {
        return Err(VerticalError::Interface("the artifact declares one input"));
    };
    // The declared boundary is total over the interface vocabulary, so an axis
    // may name a `ShapeEnv` symbol rather than a literal. This spike executes on
    // the host and compares against a reference it sizes itself, so it needs the
    // extents as numbers; a symbolic boundary is refused by name rather than
    // defaulted, because any value invented here would be compared against a
    // reference built from the same invention and the two would agree.
    let Some(input_shape) = input.static_shape() else {
        return Err(VerticalError::Interface(
            "the artifact's input boundary is wholly literal",
        ));
    };
    let [rows, columns] = input_shape.extents() else {
        return Err(VerticalError::Interface("the artifact's input is rank two"));
    };
    if input.key().as_str() != INPUT_KEY || input.resolved_type_encoding() != f32_type.as_bytes() {
        return Err(VerticalError::Interface(
            "the artifact's input is this program's, by key and resolved type",
        ));
    }
    let outputs: Vec<_> = decoded.outputs().collect();
    let [output] = outputs.as_slice() else {
        return Err(VerticalError::Interface("the artifact declares one output"));
    };
    let Some(output_shape) = output.static_shape() else {
        return Err(VerticalError::Interface(
            "the artifact's output boundary is wholly literal",
        ));
    };
    let published: u64 = output_shape
        .extents()
        .iter()
        .map(|extent| extent.get())
        .product();
    if output.key().as_str() != OUTPUT_KEY
        || output.resolved_type_encoding() != f32_type.as_bytes()
        || published != rows.get() * columns.get()
    {
        return Err(VerticalError::Interface(
            "the artifact's output is this program's, by key, resolved type, and extent",
        ));
    }

    // `LiveDevicePreflight` is the phase, and there is no CPU spelling of it:
    // `AvailabilityPhase` offers compile profile, artifact evidence, live device
    // preflight, prepared kernel preflight, and launch preflight, so a fact
    // known once a *host process* is bound has to borrow the device phase. The
    // README records this as a naming seam rather than a functional one.
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_shape(input.key(), &input_shape)
        .map_err(|_| VerticalError::Interface("the declared input shape binds"))?;
    Ok((rows.get(), columns.get(), binder.build()))
}

/// One routed entry, resolved to host storage and a decoded scalar entry.
struct PreparedEntry<'a> {
    entry: &'a ScalarEntry,
    placements: Vec<Placement>,
    grid_threads: u64,
    skipped: bool,
}

/// Everything the dispatch needs, decided while the preflight is still held.
struct PreparedRoute<'a> {
    entries: Vec<PreparedEntry<'a>>,
    allocations: Vec<Vec<u8>>,
    /// Allocation holding the program output, and how many elements to read.
    output: (usize, usize),
    context: HostExecutionContext,
}

/// Decides whether this host can carry out a route, while declining is still
/// permitted.
///
/// Everything a scalar CPU host can answer is answered here: the payload's
/// bytes decode into an executable image, the image publishes an entry for
/// every routed symbol, the live process honours the numerical realization the
/// image declares, every routed binding resolves to storage this host supplies,
/// and every allocation is made and sized. What is left after the commit is
/// arithmetic.
fn prepare_route<'a>(
    preflight: &Preflight<'_>,
    images: &'a [ScalarImage],
    operands: &[u32],
) -> Result<PreparedRoute<'a>, VerticalError> {
    let context = HostExecutionContext::bind();
    let mut entries = Vec::with_capacity(preflight.entries().len());
    let mut allocations: Vec<Vec<u8>> = Vec::new();
    let mut output = None;

    for (position, routed) in preflight.entries().iter().enumerate() {
        let image = &images[position];
        let entry =
            image
                .entry(routed.entry_symbol())
                .ok_or_else(|| VerticalError::SymbolAbsent {
                    entry: position,
                    symbol: routed.entry_symbol().to_owned(),
                })?;
        context
            .admits(
                entry.numerics,
                profile::ADDRESS_WIDTH.bits(),
                profile::TARGET_TRIPLE,
            )
            .map_err(|refusal| VerticalError::Host(Box::new(refusal)))?;

        let launch = routed.launch();
        if launch.grid_threads() == 0 && !launch.zero_work_skips_dispatch() {
            return Err(VerticalError::EmptyLaunch { entry: position });
        }

        let mut placements = Vec::with_capacity(routed.bindings().len());
        for binding in routed.bindings() {
            let needed = binding.accessible_offset() + binding.accessible_bytes();
            let allocation = allocations.len();
            let mut storage =
                vec![0_u8; usize::try_from(needed).expect("a bounded allocation fits a usize")];
            match binding.binding().target() {
                BindingTarget::ProgramInput(key) if key.as_str() == INPUT_KEY => {
                    let base = usize::try_from(binding.accessible_offset())
                        .expect("a bounded offset fits a usize");
                    for (position, bits) in operands.iter().enumerate() {
                        let at = base + position * 4;
                        storage[at..at + 4].copy_from_slice(&bits.to_le_bytes());
                    }
                }
                BindingTarget::ProgramOutput(keys)
                    if keys.len() == 1 && keys[0].as_str() == OUTPUT_KEY =>
                {
                    output = Some((
                        allocation,
                        usize::try_from(binding.accessible_bytes() / F32_BYTES)
                            .expect("a bounded element count fits a usize"),
                    ));
                }
                other => {
                    return Err(VerticalError::UnboundBinding {
                        entry: position,
                        slot: binding.slot(),
                        target: format!("{other:?}"),
                    });
                }
            }
            allocations.push(storage);
            placements.push(Placement {
                allocation,
                offset: binding.accessible_offset(),
                bytes: binding.accessible_bytes(),
            });
        }

        entries.push(PreparedEntry {
            entry,
            placements,
            grid_threads: launch.grid_threads(),
            skipped: launch.grid_threads() == 0,
        });
    }

    Ok(PreparedRoute {
        entries,
        allocations,
        output: output.ok_or(VerticalError::NoOutputBinding)?,
        context,
    })
}

/// Why the vertical did not complete.
///
/// The stages stay apart for the same reason they do in
/// `prototypes/serial-sum-run`: a program this build cannot compile, a kernel
/// with no scalar realization, an artifact this host refuses, a payload that is
/// not an executable image, a host that cannot honour the declared numerics, a
/// failed dispatch, and a numerical disagreement are different things to do
/// next, and only the last is a claim about arithmetic.
#[derive(Debug)]
pub enum VerticalError {
    Profile(TargetProfileBuildError),
    Compile(Box<CompileFailure>),
    TargetRefused(String),
    NoSelection,
    Translate(TranslationError),
    ArtifactBuild(ArtifactBuildError),
    Package(&'static str),
    HostProfile,
    Encode,
    /// The identity stated as the expected one is not statable as a recording.
    RecordedIdentity(RecordedArtifactIdentityError),
    Load(LoadRejection),
    ProbeBaseline(LoadRejection),
    NotFailedClosed {
        probe: &'static str,
        outcome: String,
    },
    ProbeAccepted(&'static str),
    Interface(&'static str),
    ImageDecode(ImageDecodeError),
    SymbolAbsent {
        entry: usize,
        symbol: String,
    },
    Host(Box<HostRefusal>),
    UnboundBinding {
        entry: usize,
        slot: usize,
        target: String,
    },
    NoOutputBinding,
    EmptyLaunch {
        entry: usize,
    },
    Execute(ExecutionError),
    ForeignProgram,
    Mismatch {
        backend: Vec<u32>,
        reference: Vec<u32>,
    },
}

impl fmt::Display for VerticalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(error) => write!(formatter, "the CPU profile does not compose: {error}"),
            Self::Compile(failure) => write!(formatter, "the program did not compile: {failure}"),
            Self::TargetRefused(detail) => {
                write!(formatter, "the CPU target refused the program: {detail}")
            }
            Self::NoSelection => formatter.write_str("the portfolio retained no selected plan"),
            Self::Translate(error) => {
                write!(
                    formatter,
                    "the kernel has no scalar CPU realization: {error}"
                )
            }
            Self::ArtifactBuild(error) => {
                write!(
                    formatter,
                    "the compiler capability cannot be packaged: {error}"
                )
            }
            Self::Package(detail) => write!(formatter, "the artifact was not packaged: {detail}"),
            Self::HostProfile => {
                formatter.write_str("the compiler's target profile does not compose an environment")
            }
            Self::Encode => formatter.write_str("the artifact envelope did not encode"),
            Self::RecordedIdentity(error) => write!(
                formatter,
                "the expected artifact identity is not statable as a recording: {error}"
            ),
            Self::Load(rejection) => write!(formatter, "the artifact was refused: {rejection}"),
            Self::ProbeBaseline(rejection) => write!(
                formatter,
                "the fail-closed probes have no accepted neighbour: the unperturbed subject was \
                 itself refused: {rejection}",
            ),
            Self::NotFailedClosed { probe, outcome } => {
                write!(
                    formatter,
                    "the loader did not fail closed on {probe}: {outcome}"
                )
            }
            Self::ProbeAccepted(probe) => write!(
                formatter,
                "a probe was accepted rather than refused: {probe}, so that probe proves nothing",
            ),
            Self::Interface(claim) => {
                write!(
                    formatter,
                    "the artifact's interface is not this program's: {claim}"
                )
            }
            Self::ImageDecode(error) => {
                write!(
                    formatter,
                    "the carried payload is not an executable image: {error}"
                )
            }
            Self::SymbolAbsent { entry, symbol } => {
                write!(formatter, "entry {entry}'s image publishes no {symbol:?}")
            }
            Self::Host(refusal) => {
                write!(
                    formatter,
                    "this host refused the route before the commit: {refusal}"
                )
            }
            Self::UnboundBinding {
                entry,
                slot,
                target,
            } => write!(
                formatter,
                "entry {entry}'s ABI slot {slot} addresses {target}, which this spike binds no \
                 storage for",
            ),
            Self::NoOutputBinding => {
                formatter.write_str("no entry of this route binds the program output")
            }
            Self::EmptyLaunch { entry } => write!(
                formatter,
                "entry {entry}'s routed launch covers no threads and is not skippable",
            ),
            Self::Execute(error) => write!(formatter, "the dispatch failed: {error}"),
            Self::ForeignProgram => formatter
                .write_str("the artifact packages a kernel program this process did not derive"),
            Self::Mismatch { backend, reference } => write!(
                formatter,
                "the scalar CPU backend returned {backend:08x?}, the reference requires \
                 {reference:08x?}",
            ),
        }
    }
}

impl std::error::Error for VerticalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Profile(error) => Some(error),
            Self::Compile(error) => Some(error.as_ref()),
            Self::Translate(error) => Some(error),
            Self::ArtifactBuild(error) => Some(error),
            Self::RecordedIdentity(error) => Some(error),
            Self::Load(error) | Self::ProbeBaseline(error) => Some(error),
            Self::ImageDecode(error) => Some(error),
            Self::Host(error) => Some(error),
            Self::Execute(error) => Some(error),
            Self::TargetRefused(_)
            | Self::NoSelection
            | Self::Package(_)
            | Self::HostProfile
            | Self::Encode
            | Self::NotFailedClosed { .. }
            | Self::ProbeAccepted(_)
            | Self::Interface(_)
            | Self::SymbolAbsent { .. }
            | Self::UnboundBinding { .. }
            | Self::NoOutputBinding
            | Self::EmptyLaunch { .. }
            | Self::ForeignProgram
            | Self::Mismatch { .. } => None,
        }
    }
}

/// The whole vertical, in the order every stage must happen in.
#[allow(
    clippy::too_many_lines,
    reason = "the vertical is one linear narrative from a declared profile to compared bits, and splitting it would hide the ordering that is its point"
)]
pub fn run() -> Result<Report, VerticalError> {
    // ---- the declared target ---------------------------------------------
    let target = profile::scalar_cpu_profile().map_err(VerticalError::Profile)?;
    println!("declared target profile: {}", target.profile_key());
    println!(
        "  triple {}, {}, {}-bit addresses",
        profile::TARGET_TRIPLE,
        profile::DATA_LAYOUT,
        profile::ADDRESS_WIDTH.bits(),
    );
    println!(
        "  scalar execution model: {} thread per workgroup, {} byte(s) of staged local memory, \
         grid to {} invocation(s)",
        profile::WORKGROUP_THREADS,
        profile::LOCAL_MEMORY_BYTES,
        profile::GRID_AXIS_THREADS,
    );
    println!(
        "  vector width, mask/tail support, scalable-vector length, and thread count: undeclared, \
         and therefore Unknown"
    );

    // ---- verified physical work ------------------------------------------
    let program = scalar_program(ROWS, COLUMNS);
    let targets = TargetRequest::new([target]).expect("one target composes a request");
    let batch = compile(CompileRequest::new(
        &program,
        NumericalContract::STRICT_F32,
        targets,
    ))
    .map_err(|failure| VerticalError::Compile(Box::new(failure)))?;
    let (_, outcome) = batch
        .into_targets()
        .pop()
        .expect("one requested target yields one result")
        .into_parts();
    let compilation = outcome.map_err(|failure| {
        VerticalError::TargetRefused(format!("{:?} / {:?}", failure.class(), failure.refusal()))
    })?;
    let plan = compilation.selected().ok_or(VerticalError::NoSelection)?;
    println!(
        "selected plan {}: fused={}, {} kernel(s), {} deferred prepared-entry requirement(s)",
        plan.stable_id(),
        plan.is_fused(),
        plan.kernels().len(),
        plan.prepared_entry_target_requirements().count(),
    );

    // ---- the executable representation -----------------------------------
    let kernel_program = plan.abi().kernel_program();
    let mut entries = Vec::new();
    let mut source = Vec::new();
    for (position, stage) in kernel_program.stages().enumerate() {
        let entry = image::translate(stage.kernel()).map_err(VerticalError::Translate)?;
        source.extend_from_slice(stage.kernel().canonical_identity().as_bytes());
        entries.push((entry_symbol(position), entry));
    }
    let translated = ScalarImage { entries };
    let image_bytes = image::encode(&translated);
    println!(
        "translated {} stage(s) into {} byte(s) of {}",
        translated.entries.len(),
        image_bytes.len(),
        profile::REPRESENTATION_KEY,
    );

    // Every construct the bounded image refuses, observed refusing. Run before
    // the positive path, so an accepted translation below is a statement about
    // the kernel rather than about a translator that accepts anything.
    println!("translation refusals, against buffer parameters this backend cannot bind:");
    for outcome in refused_buffers()? {
        println!("  {outcome}");
    }

    // ---- the artifact ----------------------------------------------------
    let artifact = assemble(&program, &compilation, plan, image_bytes.clone(), source)?;
    // Stated as a recording rather than carried as the derivation it came from:
    // this vertical is one process, so the assertion is a tautology here, and
    // spelling it the way a cold consumer would is what keeps the spike a
    // measurement of the loader's boundary rather than of an in-process shortcut.
    let expected =
        RecordedArtifactProgramIdentity::from_bytes(artifact.canonical_identity().as_bytes())
            .map_err(VerticalError::RecordedIdentity)?;
    let envelope = artifact.encode().map_err(|_| VerticalError::Encode)?;
    println!(
        "packaged {} envelope byte(s), artifact identity {} byte(s)",
        envelope.len(),
        expected.as_bytes().len(),
    );

    // ---- device-free validation ------------------------------------------
    let mut decoded =
        DecodedProgram::decode(&envelope, SOLE_DELIVERY).map_err(VerticalError::Load)?;
    println!(
        "loaded as delivery position {} of the {} this artifact carries",
        decoded.delivery_position(),
        decoded.delivery_positions(),
    );
    let (rows, columns, abi) = bind_interface(&decoded)?;
    println!("the artifact declares a {rows} by {columns} input");
    let environment = host_environment(&compilation)?;

    println!("fail-closed probes against these exact envelope bytes:");
    let subject = ProbeSubject {
        bytes: &envelope,
        expected: &expected,
        environment: &environment,
        abi: &abi,
    };
    for outcome in probe_fail_closed(&subject)? {
        println!("  {outcome}");
    }

    println!("payload refusals, against these exact image bytes:");
    for outcome in probe_image(&image_bytes)? {
        println!("  {outcome}");
    }

    let preflight = decoded
        .preflight(&environment, &expected, &abi)
        .map_err(VerticalError::Load)?;
    // Checked before the commit: a route to a program this process did not
    // derive is a reason to abandon rather than to execute and compare.
    if preflight.kernel_program_identity() != kernel_program.canonical_identity().as_bytes() {
        return Err(VerticalError::ForeignProgram);
    }
    println!(
        "device-free preflight: {} entr(y/ies), {} shared allocation(s), kernel program identity \
         matched",
        preflight.entries().len(),
        preflight.shared_allocations().len(),
    );

    // ---- the live host execution context ---------------------------------
    // The payload's own bytes, decoded here and not before: this is the point at
    // which the loader hands over an object, and decoding it earlier would be
    // decoding something the route had not yet selected.
    let mut images = Vec::with_capacity(preflight.entries().len());
    for routed in preflight.entries() {
        images.push(image::decode(routed.object()).map_err(VerticalError::ImageDecode)?);
    }
    let prepared = prepare_route(&preflight, &images, &OPERANDS)?;
    let context = &prepared.context;
    println!(
        "bound host context: {} on {}, {}-bit pointers, {}-endian, subnormals {} in / {} out \
         (measured)",
        context.arch,
        context.os,
        context.pointer_width_bits,
        context.endianness,
        context
            .input_subnormals
            .map_or("unclassified", image::ImageSubnormals::as_str),
        context
            .result_subnormals
            .map_or("unclassified", image::ImageSubnormals::as_str),
    );

    println!("host-context refusals against this exact route:");
    for outcome in probe_host(context, &images)? {
        println!("  {outcome}");
    }

    // ---- the routing commit ----------------------------------------------
    let routed = preflight.commit();
    println!(
        "committed: {} entr(y/ies) in execution order",
        routed.entries().len(),
    );
    for (position, entry) in routed.entries().iter().enumerate() {
        println!(
            "  entry {position}: symbol {:?}, {} payload byte(s), {} invocation(s) in groups of {}",
            entry.entry_symbol(),
            entry.object().len(),
            entry.launch().grid_threads(),
            entry.launch().threads_per_workgroup(),
        );
        for binding in entry.bindings() {
            println!(
                "    abi slot {} -> transport {} at byte {}, {} byte(s), {:?}",
                binding.slot(),
                binding.transport_slot(),
                binding.accessible_offset(),
                binding.accessible_bytes(),
                binding.binding().target(),
            );
        }
    }

    // ---- execution -------------------------------------------------------
    let PreparedRoute {
        entries: prepared_entries,
        mut allocations,
        output,
        ..
    } = prepared;
    for prepared_entry in &prepared_entries {
        if prepared_entry.skipped {
            continue;
        }
        interpret::execute(
            prepared_entry.entry,
            &prepared_entry.placements,
            &mut allocations,
            prepared_entry.grid_threads,
        )
        .map_err(VerticalError::Execute)?;
    }

    let (allocation, elements) = output;
    let backend: Vec<u32> = allocations[allocation]
        .as_chunks::<4>()
        .0
        .iter()
        .take(elements)
        .map(|chunk| u32::from_le_bytes(*chunk))
        .collect();

    // ---- the comparison --------------------------------------------------
    let (reference, reference_identity) = reference_bits(&program, &OPERANDS, rows, columns);
    println!("backend   {backend:08x?}");
    println!("reference {reference:08x?}");
    if backend != reference {
        return Err(VerticalError::Mismatch { backend, reference });
    }
    println!(
        "bit-for-bit agreement on {} element(s) between {} and the reference registry",
        reference.len(),
        profile::BACKEND_KEY,
    );

    Ok(Report {
        profile_key: profile::PROFILE_KEY,
        profile_descriptor_bytes: compilation.target_profile_descriptor().len(),
        backend_key: profile::BACKEND_KEY,
        representation_key: profile::REPRESENTATION_KEY,
        plan_id: plan.stable_id().to_owned(),
        deferred_predicates: plan.prepared_entry_target_requirements().count(),
        image_bytes: image_bytes.len(),
        envelope_bytes: envelope.len(),
        artifact_identity_bytes: expected.as_bytes().len(),
        reference_identity_bytes: reference_identity.len(),
        elements: reference.len(),
        output_bits: reference,
        host: format!(
            "{}-{} {}-bit {}-endian",
            context.arch, context.os, context.pointer_width_bits, context.endianness
        ),
    })
}

/// What one completed run recorded.
pub struct Report {
    pub profile_key: &'static str,
    pub profile_descriptor_bytes: usize,
    pub backend_key: &'static str,
    pub representation_key: &'static str,
    pub plan_id: String,
    pub deferred_predicates: usize,
    pub image_bytes: usize,
    pub envelope_bytes: usize,
    pub artifact_identity_bytes: usize,
    pub reference_identity_bytes: usize,
    pub elements: usize,
    pub output_bits: Vec<u32>,
    pub host: String,
}

/// The exact inputs a fail-closed probe perturbs one element of.
#[derive(Clone, Copy)]
struct ProbeSubject<'a> {
    bytes: &'a [u8],
    expected: &'a RecordedArtifactProgramIdentity,
    environment: &'a ExecutionEnvironment,
    abi: &'a AbiFacts,
}

fn refused(probe: &'static str, outcome: String) -> VerticalError {
    VerticalError::NotFailedClosed { probe, outcome }
}

/// Returns the one host-relative exclusion a refusal reports, when it excluded
/// this spike's whole portfolio and nothing else.
///
/// Host-relative ineligibility is a *filter* applied before any applicability
/// guard rather than a terminal mismatch after one, so a probe that perturbs
/// what this host states does not get a rejection naming its own subject: it
/// gets [`LoadRejection::NoEligibleVariant`] carrying the reason each excluded
/// variant was filtered under. Pinning the class alone would therefore let all
/// four envelope probes below pass on each other's perturbation.
///
/// `packaged: 1` and the single-element slice are both asserted rather than one
/// standing in for the other. This spike packages exactly one variant, so a
/// probe that excluded the portfolio must report exactly one reason at rank 0;
/// reading only the first element would accept a longer list, and reading only
/// the length would depend on the loader's own `filtered.len() == packaged`
/// invariant rather than on what this artifact declares.
fn sole_ineligibility(rejection: &LoadRejection) -> Option<&VariantIneligibility> {
    let LoadRejection::NoEligibleVariant {
        packaged: 1,
        filtered,
    } = rejection
    else {
        return None;
    };
    let [FilteredVariant { variant: 0, reason }] = filtered.as_slice() else {
        return None;
    };
    Some(reason)
}

/// Proves the loader accepts the unperturbed subject, before anything is
/// perturbed.
fn probe_baseline(subject: &ProbeSubject<'_>) -> Result<String, VerticalError> {
    let mut decoded = DecodedProgram::decode(subject.bytes, SOLE_DELIVERY)
        .map_err(VerticalError::ProbeBaseline)?;
    let preflight = decoded
        .preflight(subject.environment, subject.expected, subject.abi)
        .map_err(VerticalError::ProbeBaseline)?;
    let threads: u64 = preflight
        .entries()
        .iter()
        .map(|entry| entry.launch().grid_threads())
        .sum();
    Ok(format!(
        "the unperturbed envelope routes: {} entr(y/ies), {threads} invocation(s)",
        preflight.entries().len(),
    ))
}

/// Every envelope-level perturbation, each pinned to the class it must refuse under.
#[allow(
    clippy::too_many_lines,
    reason = "each probe perturbs exactly one input and pins the class of the refusal; splitting them into helpers would separate a perturbation from the class it must produce"
)]
fn probe_fail_closed(subject: &ProbeSubject<'_>) -> Result<Vec<String>, VerticalError> {
    let mut outcomes = vec![probe_baseline(subject)?];

    // A flipped interior byte never survives into routing.
    let mut damaged = subject.bytes.to_vec();
    let midpoint = damaged.len() / 2;
    damaged[midpoint] ^= 0x01;
    outcomes.push(match DecodedProgram::decode(&damaged, SOLE_DELIVERY) {
        Err(rejection @ LoadRejection::Artifact(_)) => {
            format!("a flipped byte at offset {midpoint}: {rejection}")
        }
        Err(other) => return Err(refused("a flipped interior byte", other.to_string())),
        Ok(_) => {
            return Err(refused(
                "a flipped interior byte",
                "the envelope decoded as valid".to_owned(),
            ));
        }
    });

    // A truncated envelope is malformed.
    outcomes.push(
        match DecodedProgram::decode(&subject.bytes[..midpoint], SOLE_DELIVERY) {
            Err(rejection @ LoadRejection::Artifact(ArtifactCodecFailure::Malformed { .. })) => {
                format!("truncated to {midpoint} byte(s): {rejection}")
            }
            Err(other) => return Err(refused("a truncated envelope", other.to_string())),
            Ok(_) => {
                return Err(refused(
                    "a truncated envelope",
                    "the envelope decoded as valid".to_owned(),
                ));
            }
        },
    );

    // An artifact that is not the expected one is a program mismatch. The
    // trailing byte is perturbed deliberately: a recorded identity is
    // domain-checked when it is stated, so a leading-byte flip would be refused
    // at the assertion boundary and never reach the loader.
    let mut decoded = DecodedProgram::decode(subject.bytes, SOLE_DELIVERY)
        .map_err(VerticalError::ProbeBaseline)?;
    let mut bytes = subject.expected.as_bytes().to_vec();
    if let Some(last) = bytes.last_mut() {
        *last ^= 0x01;
    }
    let foreign = RecordedArtifactProgramIdentity::from_bytes(&bytes)
        .map_err(VerticalError::RecordedIdentity)?;
    outcomes.push(
        match decoded.preflight(subject.environment, &foreign, subject.abi) {
            Err(rejection @ LoadRejection::ProgramMismatch { .. }) => {
                format!("an expected identity that is not this artifact's: {rejection}")
            }
            Err(other) => return Err(refused("a foreign expected identity", other.to_string())),
            Ok(_) => {
                return Err(refused(
                    "a foreign expected identity",
                    "the route was accepted".to_owned(),
                ));
            }
        },
    );

    // A host offering another descriptor of the same profile family.
    let mut decoded = DecodedProgram::decode(subject.bytes, SOLE_DELIVERY)
        .map_err(VerticalError::ProbeBaseline)?;
    let mut descriptor = subject
        .environment
        .target_profile
        .descriptor
        .as_bytes()
        .to_vec();
    if let Some(last) = descriptor.last_mut() {
        *last ^= 0x01;
    }
    let other_descriptor = ExecutionEnvironment {
        target_profile: TargetProfileRef {
            key: subject.environment.target_profile.key.clone(),
            descriptor: TargetProfileDescriptorDigest::from_bytes(&descriptor)
                .map_err(|_| VerticalError::HostProfile)?,
        },
        backend: subject.environment.backend.clone(),
        representation: subject.environment.representation.clone(),
        dtype_dispatch: subject.environment.dtype_dispatch.clone(),
    };
    outcomes.push(
        match decoded.preflight(&other_descriptor, subject.expected, subject.abi) {
            Err(rejection)
                if sole_ineligibility(&rejection).is_some_and(|reason| {
                    matches!(
                        reason,
                        VariantIneligibility::AssessedProfile {
                            classification: TargetCompatibility::DescriptorMismatch { .. },
                        },
                    )
                }) =>
            {
                format!("a host offering another profile descriptor: {rejection}")
            }
            Err(other) => return Err(refused("another profile descriptor", other.to_string())),
            Ok(_) => {
                return Err(refused(
                    "another profile descriptor",
                    "the route was accepted".to_owned(),
                ));
            }
        },
    );

    // A host stating the *Metal* profile family. This is the perturbation the
    // CPU vertical exists to be able to make: the two profiles are different
    // families rather than two revisions of one, so the refusal must be a key
    // mismatch and not a descriptor mismatch.
    let mut decoded = DecodedProgram::decode(subject.bytes, SOLE_DELIVERY)
        .map_err(VerticalError::ProbeBaseline)?;
    let metal_family = ExecutionEnvironment {
        target_profile: TargetProfileRef {
            key: TargetProfileKey::new("tiler.target.governed-prototype")
                .map_err(|_| VerticalError::HostProfile)?,
            descriptor: subject.environment.target_profile.descriptor.clone(),
        },
        backend: subject.environment.backend.clone(),
        representation: subject.environment.representation.clone(),
        dtype_dispatch: subject.environment.dtype_dispatch.clone(),
    };
    outcomes.push(
        match decoded.preflight(&metal_family, subject.expected, subject.abi) {
            Err(rejection)
                if sole_ineligibility(&rejection).is_some_and(|reason| {
                    matches!(
                        reason,
                        VariantIneligibility::AssessedProfile {
                            classification: TargetCompatibility::ProfileKeyMismatch { .. },
                        },
                    )
                }) =>
            {
                format!("a host stating another target family: {rejection}")
            }
            Err(other) => return Err(refused("another target family", other.to_string())),
            Ok(_) => {
                return Err(refused(
                    "another target family",
                    "the route was accepted".to_owned(),
                ));
            }
        },
    );

    // A host that executes Metal. Filtered on the backend and representation
    // pair the entry's payload declares, with the profile still exactly the one
    // the variant was assessed against, so the exclusion cannot come from either
    // compatibility classification.
    let mut decoded = DecodedProgram::decode(subject.bytes, SOLE_DELIVERY)
        .map_err(VerticalError::ProbeBaseline)?;
    let metal_host = ExecutionEnvironment {
        target_profile: subject.environment.target_profile.clone(),
        backend: BackendKey::new("tiler.metal").map_err(|_| VerticalError::HostProfile)?,
        representation: RepresentationKey::new("metallib")
            .map_err(|_| VerticalError::HostProfile)?,
        dtype_dispatch: subject.environment.dtype_dispatch.clone(),
    };
    outcomes.push(
        match decoded.preflight(&metal_host, subject.expected, subject.abi) {
            Err(rejection)
                if sole_ineligibility(&rejection).is_some_and(|reason| {
                    matches!(
                        reason,
                        VariantIneligibility::UnsupportedRepresentation {
                            entry: 0,
                            host_backend,
                            host_representation,
                            ..
                        } if host_backend.as_str() == "tiler.metal"
                            && host_representation.as_str() == "metallib",
                    )
                }) =>
            {
                format!("a host that executes metallibs: {rejection}")
            }
            Err(other) => return Err(refused("a Metal host", other.to_string())),
            Ok(_) => {
                return Err(refused("a Metal host", "the route was accepted".to_owned()));
            }
        },
    );

    // A host stating this backend family and a later version of its own
    // representation. The pair is checked together rather than either half
    // alone, so a backend that widened its representation set would have to say
    // so — and the host half is pinned here precisely because this exclusion and
    // the Metal one above now report the same class.
    let mut decoded = DecodedProgram::decode(subject.bytes, SOLE_DELIVERY)
        .map_err(VerticalError::ProbeBaseline)?;
    let other_representation = ExecutionEnvironment {
        target_profile: subject.environment.target_profile.clone(),
        backend: subject.environment.backend.clone(),
        representation: RepresentationKey::new("tiler.cpu.scalar-image-v2")
            .map_err(|_| VerticalError::HostProfile)?,
        dtype_dispatch: subject.environment.dtype_dispatch.clone(),
    };
    outcomes.push(
        match decoded.preflight(&other_representation, subject.expected, subject.abi) {
            Err(rejection)
                if sole_ineligibility(&rejection).is_some_and(|reason| {
                    matches!(
                        reason,
                        VariantIneligibility::UnsupportedRepresentation {
                            entry: 0,
                            host_backend,
                            host_representation,
                            ..
                        } if host_backend.as_str() == profile::BACKEND_KEY
                            && host_representation.as_str() == "tiler.cpu.scalar-image-v2",
                    )
                }) =>
            {
                format!("a host consuming another representation version: {rejection}")
            }
            Err(other) => return Err(refused("another representation", other.to_string())),
            Ok(_) => {
                return Err(refused(
                    "another representation",
                    "the route was accepted".to_owned(),
                ));
            }
        },
    );

    Ok(outcomes)
}

/// Every payload-level perturbation, each pinned to the class it must refuse under.
///
/// These are the checks the *artifact* layer cannot make: a payload's object
/// bytes are opaque to every check `DecodedProgram` performs, so an image that
/// decoded correctly as an envelope and is garbage as an executable would reach
/// the dispatch. The backend owns that boundary, and this is where it is
/// established.
fn probe_image(bytes: &[u8]) -> Result<Vec<String>, VerticalError> {
    let mut outcomes = Vec::new();
    let baseline = image::decode(bytes).map_err(VerticalError::ImageDecode)?;
    outcomes.push(format!(
        "the unperturbed payload decodes: {} entr(y/ies)",
        baseline.entries.len(),
    ));

    let mut foreign = bytes.to_vec();
    foreign[0] ^= 0x01;
    outcomes.push(match image::decode(&foreign) {
        Err(error @ ImageDecodeError::NotThisRepresentation) => {
            format!("a payload whose domain separator differs: {error}")
        }
        Err(other) => return Err(refused("a foreign payload domain", other.to_string())),
        Ok(_) => {
            return Err(refused(
                "a foreign payload domain",
                "the payload decoded as valid".to_owned(),
            ));
        }
    });

    let midpoint = bytes.len() / 2;
    outcomes.push(match image::decode(&bytes[..midpoint]) {
        Err(
            error @ (ImageDecodeError::Truncated { .. } | ImageDecodeError::TrailingBytes { .. }),
        ) => {
            format!("a payload truncated to {midpoint} byte(s): {error}")
        }
        Err(other) => return Err(refused("a truncated payload", other.to_string())),
        Ok(_) => {
            return Err(refused(
                "a truncated payload",
                "the payload decoded as valid".to_owned(),
            ));
        }
    });

    let mut extended = bytes.to_vec();
    extended.push(0x00);
    outcomes.push(match image::decode(&extended) {
        Err(error @ ImageDecodeError::TrailingBytes { .. }) => {
            format!("a payload with one appended byte: {error}")
        }
        Err(other) => return Err(refused("an extended payload", other.to_string())),
        Ok(_) => {
            return Err(refused(
                "an extended payload",
                "the payload decoded as valid".to_owned(),
            ));
        }
    });

    // A slot reference past the entry's declared value space. Constructed by
    // re-encoding a *modified image* rather than by flipping a byte at a guessed
    // offset, so the perturbation is exactly the one being claimed.
    let mut widened = baseline.clone();
    let out_of_range = u32::try_from(widened.entries[0].1.slot_types.len())
        .expect("a bounded slot count fits a u32");
    match widened.entries[0].1.body.instructions.first_mut() {
        Some(Instruction::GlobalInvocationIndex { result }) => *result = out_of_range,
        _ => {
            return Err(VerticalError::ProbeAccepted(
                "an image with no first instruction",
            ));
        }
    }
    outcomes.push(match image::decode(&image::encode(&widened)) {
        Err(error @ ImageDecodeError::SlotOutOfRange { .. }) => {
            format!("an instruction naming slot {out_of_range}: {error}")
        }
        Err(other) => return Err(refused("an out-of-range slot", other.to_string())),
        Ok(_) => {
            return Err(refused(
                "an out-of-range slot",
                "the payload decoded as valid".to_owned(),
            ));
        }
    });

    // A store into the read-only input parameter.
    let mut violating = baseline.clone();
    let entry = &mut violating.entries[0].1;
    let store = find_store(&mut entry.body).ok_or(VerticalError::ProbeAccepted(
        "an image with no store to redirect",
    ))?;
    if let Instruction::Store { buffer, .. } = store {
        *buffer = 0;
    }
    outcomes.push(match image::decode(&image::encode(&violating)) {
        Err(error @ ImageDecodeError::BufferAccessViolation { .. }) => {
            format!("a store into the read-only parameter: {error}")
        }
        Err(other) => return Err(refused("an access-mode violation", other.to_string())),
        Ok(_) => {
            return Err(refused(
                "an access-mode violation",
                "the payload decoded as valid".to_owned(),
            ));
        }
    });

    // A well-formed image declaring a realization this host does not deliver is
    // deliberately *not* a decode refusal: those bytes are a valid image, and
    // what refuses them is the live host context in `probe_host` below. Keeping
    // the two apart is what lets a reader tell a corrupt payload from a payload
    // this machine cannot honour.
    Ok(outcomes)
}

/// Returns the first store instruction of a block, searching nested blocks.
fn find_store(block: &mut image::Block) -> Option<&mut Instruction> {
    // Two passes rather than one, because a single `iter_mut` pass that recursed
    // would hold a mutable borrow of the block across the recursive call.
    let position = block
        .instructions
        .iter()
        .position(|instruction| matches!(instruction, Instruction::Store { .. }));
    if let Some(position) = position {
        return block.instructions.get_mut(position);
    }
    for instruction in &mut block.instructions {
        match instruction {
            Instruction::Predicated { body, .. } | Instruction::SerialLoop { body, .. } => {
                let found = find_store(body);
                if found.is_some() {
                    return found;
                }
            }
            _ => {}
        }
    }
    None
}

/// Every host-context perturbation, each pinned to the refusal it must produce.
///
/// The host is the *fixed* half here and the declaration is what varies, which
/// is the only direction available: this process cannot be made to flush
/// subnormals or to use 32-bit pointers, and pretending otherwise would be a
/// probe that measures nothing. Each case therefore asks the real measured
/// context to admit a realization it does not deliver, and the refusal is the
/// same code path a genuinely flushing host would take.
fn probe_host(
    context: &HostExecutionContext,
    images: &[ScalarImage],
) -> Result<Vec<String>, VerticalError> {
    let entry = images
        .first()
        .and_then(|image| image.entries.first())
        .map(|(_, entry)| entry)
        .ok_or(VerticalError::ProbeAccepted("a route with no entries"))?;

    let mut outcomes = vec![format!(
        "the unperturbed declaration is admitted: {}",
        match context.admits(
            entry.numerics,
            profile::ADDRESS_WIDTH.bits(),
            profile::TARGET_TRIPLE,
        ) {
            Ok(()) => "the measured host honours it".to_owned(),
            Err(refusal) => return Err(VerticalError::Host(Box::new(refusal))),
        }
    )];

    let cases: [(&'static str, ImageNumerics, u8, &'static str); 4] = [
        (
            "an image declaring flushed input subnormals",
            ImageNumerics {
                input_subnormals: image::ImageSubnormals::FlushSignedZero,
                ..entry.numerics
            },
            profile::ADDRESS_WIDTH.bits(),
            profile::TARGET_TRIPLE,
        ),
        (
            "an image declaring flushed result subnormals",
            ImageNumerics {
                result_subnormals: image::ImageSubnormals::FlushPositiveZero,
                ..entry.numerics
            },
            profile::ADDRESS_WIDTH.bits(),
            profile::TARGET_TRIPLE,
        ),
        (
            "an artifact declaring a 32-bit address model",
            entry.numerics,
            32,
            profile::TARGET_TRIPLE,
        ),
        (
            "an artifact declaring another architecture",
            entry.numerics,
            profile::ADDRESS_WIDTH.bits(),
            "x86_64-apple-darwin",
        ),
    ];
    for (name, numerics, bits, triple) in cases {
        match context.admits(numerics, bits, triple) {
            Err(refusal) => outcomes.push(format!("{name}: {refusal}")),
            Ok(()) => return Err(refused(name, "the host admitted it".to_owned())),
        }
    }
    Ok(outcomes)
}

/// Observes the bounded image refusing buffer parameters it cannot realize.
///
/// Every case here is a **real call into the translator's own decision**, made
/// against a synthetic `BufferParameter` this function builds, so each line is a
/// refusal that happened rather than an error value formatted for display.
///
/// # What is deliberately not exercised, and why
///
/// The operation-level refusals — packed-nibble extraction, a barrier, the
/// dequantization conversions, `I32Subtract` — are not reachable from here.
/// Constructing an `OperationView` requires a `VerifiedValueId`, which only
/// `tiler-ir` can mint and which this crate holds none of, and building a
/// verified kernel containing those operations means assembling a scheduled
/// region the compiler does not produce for any program this profile admits.
/// They are guarded instead by the exhaustive match in
/// [`crate::image::translate`]: an operation kind added to KIR without a scalar
/// meaning is a build error there. That is a weaker guarantee than an observed
/// refusal and it is stated as such rather than papered over — the README
/// records it as the spike's measurement boundary.
///
/// # Errors
///
/// Returns [`VerticalError::ProbeAccepted`] when a parameter this backend must
/// refuse is accepted, which is as loud a failure as a wrong refusal.
fn refused_buffers() -> Result<Vec<String>, VerticalError> {
    let admitted = BufferParameter {
        tensor: TensorRole::Input,
        component_role: None,
        element_type: KernelType::F32,
        address_space: AddressSpace::Device,
        access: BufferAccess::Read,
        element_count: 12,
    };
    // The accepted neighbour, first and separately: without it every refusal
    // below could be produced by a translator that refuses everything.
    let mut outcomes = vec![match image::translate_buffer(0, admitted) {
        Ok(buffer) => format!(
            "an f32 device buffer is admitted: {} element(s), {:?}",
            buffer.element_count, buffer.access,
        ),
        Err(error) => return Err(VerticalError::Translate(error)),
    }];

    let cases: [(&str, BufferParameter); 4] = [
        (
            "a workgroup-space buffer",
            BufferParameter {
                address_space: AddressSpace::Workgroup,
                ..admitted
            },
        ),
        (
            "a constant-space buffer",
            BufferParameter {
                address_space: AddressSpace::Constant,
                ..admitted
            },
        ),
        (
            "a u8 buffer",
            BufferParameter {
                element_type: KernelType::U8,
                ..admitted
            },
        ),
        (
            "an invocation-private buffer",
            BufferParameter {
                address_space: AddressSpace::InvocationPrivate,
                ..admitted
            },
        ),
    ];
    for (name, parameter) in cases {
        match image::translate_buffer(0, parameter) {
            Err(error) => outcomes.push(format!("{name}: {error}")),
            Ok(_) => {
                return Err(VerticalError::ProbeAccepted(
                    "a buffer this backend cannot bind",
                ));
            }
        }
    }
    Ok(outcomes)
}

impl Report {
    /// Renders this run as the retained result fixture.
    ///
    /// Hand-written JSON rather than a serialization dependency: the record is
    /// six scalars and a bit vector, and a spike that pulled in a serializer to
    /// emit them would be adding a dependency to the evidence rather than to the
    /// thing measured.
    #[must_use]
    pub fn to_json(&self) -> String {
        let bits = self
            .output_bits
            .iter()
            .map(|value| format!("\"0x{value:08x}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{{\n  \"spike\": \"scalar-cpu-vertical\",\n  \"host\": \"{}\",\n  \
             \"target_profile_key\": \"{}\",\n  \"target_profile_descriptor_bytes\": {},\n  \
             \"backend_key\": \"{}\",\n  \"representation_key\": \"{}\",\n  \"plan\": \"{}\",\n  \
             \"deferred_prepared_entry_predicates\": {},\n  \"payload_bytes\": {},\n  \
             \"envelope_bytes\": {},\n  \"artifact_identity_bytes\": {},\n  \
             \"reference_registry_identity_bytes\": {},\n  \"elements\": {},\n  \
             \"output_bits\": [{}]\n}}\n",
            self.host,
            self.profile_key,
            self.profile_descriptor_bytes,
            self.backend_key,
            self.representation_key,
            self.plan_id,
            self.deferred_predicates,
            self.image_bytes,
            self.envelope_bytes,
            self.artifact_identity_bytes,
            self.reference_identity_bytes,
            self.elements,
            bits,
        )
    }
}
