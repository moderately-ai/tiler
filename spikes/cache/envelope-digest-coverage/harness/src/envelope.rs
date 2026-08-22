//! Builds real artifact envelopes of an exact byte length.
//!
//! # Why two programs rather than one
//!
//! This spike asks what a corrupted envelope run is caught by, and one of the
//! corruptions it must be able to express is *substitution*: the span holding
//! envelope `A` now holds envelope `B`, with every other byte of the cache
//! bundle exactly as its publisher wrote it. Expressing that without disturbing
//! the bundle's own framing needs two envelopes of the **same length** that are
//! genuinely different artifacts, so this factory is parameterized by the
//! program it compiles and solves for an exact envelope length by varying the
//! carried object's length.
//!
//! # Why the envelopes are real
//!
//! The verdict being recorded is
//! [`tiler_artifact::program::decode_artifact`]'s, and that decoder verifies a
//! manifest digest, every artifact section digest, a re-derived canonical
//! identity, and a re-encode byte-comparison. A stand-in payload would answer a
//! different question entirely. This harness is therefore an orchestrator
//! holding `tiler-ir`, `tiler-compiler`, `tiler-artifact`, and `tiler-cache`
//! together, which `tiler-cache` deliberately cannot — the same arrangement
//! `spikes/cache/hot-path-efficiency/` uses, and this file is adapted from its
//! `envelope.rs`.
//!
//! # What the synthetic object bytes do and do not bound
//!
//! The carried object travels opaquely: artifact identity folds the payload
//! *metadata* and excludes every object byte, and the object section carries its
//! own content digest as integrity. So the artifact layer performs the same work
//! on `n` synthetic bytes as on `n` bytes of `metallib`, and synthetic bytes are
//! what let an envelope be produced at an exact length. Nothing here is evidence
//! about a real Metal compilation.

use tiler_artifact::program::{
    PhysicalImplementationProposalIdentity, PhysicalProposalKind,
    PhysicalRegionOccurrenceIdentity, SelectedPhysicalImplementation,
    ApproximationEnvelope, ArtifactBuildError, ArtifactExecutionPolicy, ArtifactProgramBuilder,
    BackendEntryKey, BackendEntryRef, BackendKey, BindingKind, BindingSpec, CANONICAL_DIMENSIONS,
    CapabilityFamilyKey, CompilationEnvironment, DIMENSION_COUNT, DeliveredRealizationBuilder,
    DeliveredRealizationRecord, DimensionBehaviour, EntryRealization, EntrySpec,
    FactSourceProvenance, FeasibilityRuleSetKey, FeasibilityRuleSetRef, HonouringMeans, LaunchSpec,
    LoweringCapabilitySubject, MaterializationRounding, NumericalDimension, NumericalObligationKey,
    NumericalPermission, PayloadContent, PayloadEntryMapping, PayloadMetadata, PayloadPlatform,
    PayloadProvenance, PolicyLocus, ProvenanceIdentity, RepresentationKey, ScalarArithmeticSubject,
    SchemaVersion, SelectedLoweringProvider, SemanticOccurrence, TargetEvidenceDeclaration,
    TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef, ToolComponent, VariantSpec,
    VerifiedArtifactProgram, overlapping_behaviour,
};
use tiler_compiler::session::{Compilation, NumericalContract, PlanAlternative, compile_governed};
use tiler_ir::program::VerifiedKernelProgram;
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// How many corrections the length solver is allowed before it gives up.
///
/// The envelope length is affine in the object length — every variable-length
/// run carries a fixed-width length prefix — so one correction is enough and the
/// rest of the budget exists to fail loudly rather than to loop.
const LENGTH_ATTEMPTS: usize = 4;

/// One compiled program, reusable as the source of envelopes of any length.
pub struct EnvelopeFactory {
    semantic: SemanticProgram,
    compilation: Compilation,
    /// The byte this factory's object bytes are offset by, so two factories
    /// producing envelopes of one length still produce different bytes even
    /// where their structure agrees.
    tint: u8,
    base_bytes: usize,
}

impl EnvelopeFactory {
    /// Compiles one program and measures the envelope's fixed overhead.
    ///
    /// # Panics
    ///
    /// Panics when the governed program does not compile or the assembled
    /// artifact does not verify. Either is a defect in this spike or in the
    /// workspace it builds against, never a cache or codec outcome, so it must
    /// not be confusable with a corruption verdict.
    #[must_use]
    pub fn new(rows: u64, columns: u64, tint: u8) -> Self {
        let semantic = serial_sum_program(rows, columns);
        let compilation =
            compile_governed(&semantic, NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32)
                .expect("the governed program compiles");
        let mut factory = Self {
            semantic,
            compilation,
            tint,
            base_bytes: 0,
        };
        factory.base_bytes = factory.with_object(0).len();
        factory
    }

    /// Bytes an envelope occupies before any object byte is carried.
    #[must_use]
    pub const fn base_bytes(&self) -> usize {
        self.base_bytes
    }

    /// Encodes one envelope of exactly `target` bytes.
    ///
    /// # Panics
    ///
    /// Panics when `target` is below [`Self::base_bytes`], and when the solver
    /// does not land on the exact length within [`LENGTH_ATTEMPTS`]. Both are
    /// loud on purpose: a substitution class whose two envelopes differ in
    /// length is not the class this spike says it is.
    #[must_use]
    pub fn exactly(&self, target: usize) -> Vec<u8> {
        assert!(
            target >= self.base_bytes,
            "an envelope cannot be smaller than its {} byte fixed overhead; asked for {target}",
            self.base_bytes,
        );
        let mut object = target - self.base_bytes;
        for _ in 0..LENGTH_ATTEMPTS {
            let bytes = self.with_object(object);
            if bytes.len() == target {
                return bytes;
            }
            object = object
                .checked_add(target)
                .and_then(|grown| grown.checked_sub(bytes.len()))
                .expect("the length correction stays inside the address space");
        }
        panic!("no object length produced an envelope of exactly {target} bytes");
    }

    /// Assembles and encodes one envelope carrying `object_bytes` object bytes.
    fn with_object(&self, object_bytes: usize) -> Vec<u8> {
        let plan = self
            .compilation
            .selected()
            .expect("a selected plan alternative");
        assemble(
            &self.semantic,
            &self.compilation,
            plan,
            object_bytes,
            self.tint,
        )
        .expect("the compiler capability packages without narrowing")
        .encode()
        .expect("the envelope encodes")
    }
}

/// Builds `sum((input * 1.0) + 0.0)` over the reduced axis.
///
/// The same program `spikes/cache/hot-path-efficiency` compiles, for the same
/// reason: it is the smallest governed program that produces a real plan, and
/// the compilation is not what this spike is about. The shape is a parameter so
/// two factories package genuinely different semantic programs — different
/// reached semantic subjects, and therefore a different canonical artifact
/// identity — rather than two spellings of one.
fn serial_sum_program(rows: u64, columns: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([rows, columns]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");
    let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).expect("the bias applies");
    let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
    let mapped = F32Add::apply(&mut builder, product, bias).expect("the bias applies");
    let sum =
        StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).expect("the sum applies");
    builder
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            sum,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Packages one plan alternative and a carried payload as an artifact.
#[expect(
    clippy::too_many_lines,
    reason = "one artifact is assembled field by field; splitting it would scatter one fixture across helpers that are each used once"
)]
fn assemble(
    semantic: &SemanticProgram,
    compilation: &Compilation,
    plan: PlanAlternative<'_>,
    object_bytes: usize,
    tint: u8,
) -> Result<VerifiedArtifactProgram, ArtifactBuildError> {
    let profile = TargetProfileRef {
        key: TargetProfileKey::new(compilation.target_profile_key())
            .expect("the compiler mints a governed profile key"),
        descriptor: TargetProfileDescriptorDigest::from_bytes(
            compilation.target_profile_descriptor(),
        )
        .expect("the compiler mints a profile descriptor"),
    };
    let rules = FeasibilityRuleSetRef {
        key: FeasibilityRuleSetKey::new(compilation.feasibility_rule_set_key())
            .expect("the compiler mints a governed rule-set key"),
        revision: compilation.feasibility_rule_set_revision(),
    };

    let environment = CompilationEnvironment::new(
        plan.selected_capabilities()
            .map(|selected| selected.provider().clone()),
        compilation.offered_physical_providers().iter().cloned(),
    )
    .expect("the offered providers compose an environment");
    let mut builder =
        ArtifactProgramBuilder::new(semantic, environment).expect("a builder identity remains");
    for selected in plan.selected_capabilities() {
        let subject = selected.subject();
        builder
            .select_lowering_provider(SelectedLoweringProvider {
                provider: selected.provider().clone(),
                capability: LoweringCapabilitySubject {
                    family: CapabilityFamilyKey::new(subject.family().key_token())?,
                    operation: subject.operation().clone(),
                },
                capability_revision: selected.capability_revision(),
            })
            .expect("a selected provider was offered");
    }

    let program = plan.abi().kernel_program();

    // One mapping per distinct kernel identity, in canonical key order, with as
    // many transport slots as the entry has bindings. `check_entry_mappings`
    // proves both on every decode, so a mapping that drifted from the entries
    // below is a decode failure rather than a silently weaker artifact.
    let mut mappings: Vec<PayloadEntryMapping> = Vec::new();
    for stage in program.stages() {
        let entry_key = BackendEntryKey::from_bytes(stage.kernel().canonical_identity().as_bytes())
            .expect("the packaged kernel identity fits a backend entry key");
        if mappings
            .iter()
            .any(|mapping| mapping.entry_key == entry_key)
        {
            continue;
        }
        let transports = (0..u32::try_from(stage.accesses().len())
            .expect("a stage declares fewer accesses than a u32 counts"))
            .collect();
        mappings.push(PayloadEntryMapping {
            entry_key,
            symbol: format!("tiler_spike_entry_{}", mappings.len()),
            transports,
        });
    }
    mappings.sort_by(|left, right| left.entry_key.as_bytes().cmp(right.entry_key.as_bytes()));

    let mut source = Vec::new();
    for stage in program.stages() {
        source.extend_from_slice(stage.kernel().canonical_identity().as_bytes());
    }

    let payload = builder
        .push_carried_payload(
            BackendKey::new("tiler.spike.envelope-digest-coverage")
                .expect("a governed backend key"),
            RepresentationKey::new("opaque-object").expect("a governed representation key"),
            SchemaVersion::new(1, 0),
            profile.clone(),
            ArtifactExecutionPolicy::NativeImage,
            PayloadContent {
                metadata: PayloadMetadata {
                    source_representation: RepresentationKey::new("tiler.spike.kernel-identities")
                        .expect("a governed representation key"),
                    source,
                    provenance: PayloadProvenance {
                        toolchain: "tiler.spike.no-toolchain".to_owned(),
                        target: "tiler-spike-host".to_owned(),
                        family: "tiler.spike.envelope-digest-coverage".to_owned(),
                        language: "tiler.spike.opaque-object".to_owned(),
                        // This spike runs no external compiler and resolves no
                        // SDK. Stating the absence is what ADR 0090 item 14
                        // requires; minting a deployment minimum with no
                        // referent would put an approximated field into durable
                        // identity.
                        platform: PayloadPlatform::Unversioned,
                        components: vec![ToolComponent {
                            role: "synthesizer".to_owned(),
                            version: "1".to_owned(),
                        }],
                        compile_flags: Vec::new(),
                        link_flags: Vec::new(),
                    },
                    entries: mappings,
                    obligations: Vec::new(),
                },
                code: object_of(object_bytes, tint),
            },
        )
        .expect("the carried payload is accepted");

    let entries: Vec<EntrySpec> = program
        .stages()
        .map(|stage| EntrySpec {
            bindings: stage
                .accesses()
                .map(|_| BindingSpec {
                    kind: BindingKind::Buffer,
                })
                .collect(),
            launch: LaunchSpec {
                // Not a choice: every verified scheduled region carries it.
                zero_work_skips_dispatch: true,
                preconditions: Vec::new(),
            },
            implementation: BackendEntryRef {
                payloads: vec![payload],
                entry_key: BackendEntryKey::from_bytes(
                    stage.kernel().canonical_identity().as_bytes(),
                )
                .expect("the packaged kernel identity fits a backend entry key"),
            },
        })
        .collect();

    builder
        .push_variant(
            program,
            VariantSpec {
                target_profile: profile.clone(),
                feasibility_rules: rules,
                // Forwarded from the compiler's own iterator, in the order
                // it states: the selections are already sorted by whole
                // occurrence bytes before a public plan exists.
                selected_physical_implementations: plan
                    .selected_physical_providers()
                    .map(|selected| SelectedPhysicalImplementation {
                        region_occurrence: PhysicalRegionOccurrenceIdentity::from_bytes(
                            selected.region_occurrence_identity(),
                        )
                        .expect("the compiler mints a bounded occurrence identity"),
                        implementation_proposal:
                            PhysicalImplementationProposalIdentity::from_bytes(
                                selected.implementation_proposal_identity(),
                            )
                            .expect("the compiler mints a bounded proposal identity"),
                        provider: selected.provider().clone(),
                        proposal_kind: match selected.proposal_kind() {
                            "scheduled-kernel" => PhysicalProposalKind::ScheduledKernel,
                            "kernel-subprogram" => PhysicalProposalKind::KernelSubprogram,
                            "opaque-call" => PhysicalProposalKind::OpaqueCall,
                            kind => panic!("this harness packages no proposal of kind `{kind}`"),
                        },
                    })
                    .collect(),
                deferred_predicates: Vec::new(),
                entries,
            },
        )
        .expect("the variant packages the plan it was built from");
    builder
        .declare_realization(realization_record(&profile, program))
        .expect("the record agrees with the packaged portfolio");
    Ok(builder.build().expect("the assembled artifact verifies"))
}

/// Builds the delivered-realization record every executable artifact carries.
///
/// The eleven resolutions are derived from the packaged program's own scheduled
/// realization rather than restated here, so this harness cannot describe a
/// contract its plan does not schedule. Every packaged entry is bound, in the
/// flat declared space; one variant means that is the program's stage count.
///
/// The obligation is stated at the computation locus of occurrence 0 with
/// `SupportedExactly` means, which is what the strict contract this spike
/// compiles under actually rests on. A harness that invented a relaxation it did
/// not need would be writing a fact rather than recording one.
fn realization_record(
    profile: &TargetProfileRef,
    program: &VerifiedKernelProgram,
) -> DeliveredRealizationRecord {
    let entry = EntryRealization::of(
        program
            .stages()
            .next()
            .expect("a packaged program has a stage")
            .kernel()
            .numerical(),
    );
    let entries = u32::try_from(program.stages().len()).expect("a bounded stage table fits u32");
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

/// Object bytes that vary, so no digest ever sees a constant run.
fn object_of(bytes: usize, tint: u8) -> Vec<u8> {
    (0..bytes)
        .map(|index| {
            u8::try_from(index % 251)
                .expect("a remainder below 251 fits in a byte")
                .wrapping_add(tint)
        })
        .collect()
}
