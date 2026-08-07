//! Builds one real artifact envelope carrying an exact number of object bytes.
//!
//! Lifted from `spikes/cache/hot-path-efficiency/harness/src/envelope.rs`, whose
//! module documentation gives the reason a real envelope is required rather than
//! a byte run: the thing being measured is what the *production* validator does,
//! and that validator is [`tiler_artifact::program::decode_artifact`]. Two
//! differences are deliberate. This harness is parameterized by object length
//! directly rather than by a target envelope length, because the quantity it
//! sweeps is section bytes and the length solver that spike needs would only add
//! a failure mode. And it hands back the [`VerifiedArtifactProgram`] — or, for
//! the `build` row, the unbuilt [`ArtifactProgramBuilder`] — rather than its
//! bytes, because the producer's own steps are among the paths measured.
//!
//! The carried object is synthetic, and what that does and does not bound is
//! unchanged: artifact identity folds the payload *metadata* and excludes every
//! object byte, so the artifact layer performs identical work on `n` synthetic
//! bytes and `n` bytes of `metallib`. Nothing here is evidence about a real
//! Metal compilation.

use tiler_artifact::program::{
    AbiBinaryOp, AbiExprId, AbiRoot, ApproximationEnvelope, ArtifactExecutionPolicy,
    ArtifactProgramBuilder, BackendEntryKey, BackendEntryRef, BackendKey, BindingKind, BindingSpec,
    CANONICAL_DIMENSIONS, CapabilityKey, CompilationEnvironment, DIMENSION_COUNT,
    DeliveredRealizationBuilder, DeliveredRealizationRecord, DimensionBehaviour, EntryRealization,
    EntrySpec, FactSourceProvenance, FeasibilityRuleSetKey, FeasibilityRuleSetRef, HonouringMeans,
    LaunchSpec, MaterializationRounding, NumericalDimension, NumericalObligationKey,
    NumericalPermission, PayloadContent, PayloadEntryMapping, PayloadId, PayloadMetadata,
    PayloadPlatform, PayloadProvenance, PolicyLocus, ProvenanceIdentity, RepresentationKey,
    ScalarArithmeticSubject, SchemaVersion, SelectedProvider, SemanticOccurrence,
    TargetEvidenceDeclaration, TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
    ToolComponent, VariantSpec, VerifiedArtifactProgram, overlapping_behaviour,
};
use tiler_compiler::session::{Compilation, NumericalContract, PlanAlternative, compile_governed};
use tiler_ir::program::VerifiedKernelProgram;
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// Rows of the exercised program's input.
const ROWS: u64 = 4;
/// Columns of the exercised program's input.
const COLUMNS: u64 = 3;

/// One compiled program, reusable as the source of envelopes of any size.
///
/// Compiling is the expensive step and it is invariant across the object lengths
/// this spike sweeps, so it happens once. Nothing below that point is memoized:
/// each call re-assembles the artifact, because a measured encode must encode.
pub struct EnvelopeFactory {
    semantic: SemanticProgram,
    compilation: Compilation,
}

impl EnvelopeFactory {
    /// Compiles the governed program once.
    ///
    /// # Panics
    ///
    /// Panics when the governed program does not compile. That is a defect in
    /// this spike or in the workspace it builds against, never a codec outcome,
    /// so it must not be confusable with the decoder falling open.
    #[must_use]
    pub fn new() -> Self {
        let semantic = serial_sum_program(ROWS, COLUMNS);
        let compilation =
            compile_governed(&semantic, NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32)
                .expect("the governed program compiles");
        Self {
            semantic,
            compilation,
        }
    }

    /// Assembles one verified artifact carrying `object_bytes` object bytes and
    /// `arena_chain` additional ABI expression nodes.
    ///
    /// The two parameters are the envelope's two independent size dimensions.
    /// Object bytes grow the framed **sections**; arena nodes grow the
    /// **manifest** and, far faster, the canonical content keys a decoder
    /// derives from it.
    ///
    /// # Panics
    ///
    /// Panics when the assembled artifact does not verify, for the reason above.
    #[must_use]
    pub fn artifact(&self, object_bytes: usize, arena_chain: usize) -> VerifiedArtifactProgram {
        self.draft(object_bytes, arena_chain)
            .build()
            .expect("the assembled artifact verifies")
    }

    /// Assembles the same artifact's draft, stopping short of
    /// [`ArtifactProgramBuilder::build`].
    ///
    /// Separate from [`Self::artifact`] because `build` is itself a measured
    /// path and it consumes its builder: a row that reports what `build`
    /// allocates has to be handed a draft that already exists, or it would be
    /// reporting the declaration of a payload rather than the verification of
    /// one. `artifact` is this call and that one, so the two cannot describe
    /// different artifacts.
    #[must_use]
    pub fn draft(&self, object_bytes: usize, arena_chain: usize) -> ArtifactProgramBuilder {
        let plan = self
            .compilation
            .selected()
            .expect("a selected plan alternative");
        assemble(
            &self.semantic,
            &self.compilation,
            plan,
            object_bytes,
            arena_chain,
        )
    }
}

impl Default for EnvelopeFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds `sum((input * 1.0) + 0.0)` over the reduced axis.
///
/// The smallest governed program that produces a real plan. The compilation is
/// not what this spike measures.
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

/// Packages one plan alternative and a carried payload as an artifact draft.
fn assemble(
    semantic: &SemanticProgram,
    compilation: &Compilation,
    plan: PlanAlternative<'_>,
    object_bytes: usize,
    arena_chain: usize,
) -> ArtifactProgramBuilder {
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
    )
    .expect("the offered providers compose an environment");
    let mut builder =
        ArtifactProgramBuilder::new(semantic, environment).expect("a builder identity remains");
    for selected in plan.selected_capabilities() {
        builder
            .select_provider(SelectedProvider {
                provider: selected.provider().clone(),
                capability: CapabilityKey::new(selected.capability_key())
                    .expect("the compiler mints a governed capability key"),
                capability_revision: selected.capability_revision(),
            })
            .expect("a selected provider was offered");
    }

    let program = plan.abi().kernel_program();
    let payload = push_object(&mut builder, program, &profile, object_bytes);
    let precondition = deep_arena(&mut builder, arena_chain);

    let entries: Vec<EntrySpec> = program
        .stages()
        .enumerate()
        .map(|(position, stage)| EntrySpec {
            bindings: stage
                .accesses()
                .map(|_| BindingSpec {
                    kind: BindingKind::Buffer,
                })
                .collect(),
            launch: LaunchSpec {
                // Not a choice: every verified scheduled region carries it.
                zero_work_skips_dispatch: true,
                // Only the first entry carries it: a precondition is
                // per-entry-distinct, and one reachable use site is all the
                // chain needs, since expression closure follows operands.
                preconditions: precondition.filter(|_| position == 0).into_iter().collect(),
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
                deferred_predicates: Vec::new(),
                entries,
            },
        )
        .expect("the variant packages the plan it was built from");
    builder
        .declare_realization(realization_record(&profile, program))
        .expect("the record agrees with the packaged portfolio");
    builder
}

/// Pushes one carried payload of `object_bytes` synthetic object bytes.
///
/// One mapping per distinct kernel identity, in canonical key order, with as
/// many transport slots as the entry has bindings. `check_entry_mappings` proves
/// both on every decode, so a mapping that drifted from the entries the caller
/// declares is a decode failure rather than a silently weaker artifact.
fn push_object(
    builder: &mut ArtifactProgramBuilder,
    program: &VerifiedKernelProgram,
    profile: &TargetProfileRef,
    object_bytes: usize,
) -> PayloadId {
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

    builder
        .push_carried_payload(
            BackendKey::new("tiler.spike.decoder-allocation").expect("a governed backend key"),
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
                        family: "tiler.spike.decoder-allocation".to_owned(),
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
                code: object_of(object_bytes),
            },
        )
        .expect("the carried payload is accepted")
}

/// Mints a chain of `nodes` additional arena expressions and returns its head.
///
/// A chain rather than a wide fan, because the quantity this dimension exists to
/// exercise is arena **depth**. At manifest schema `13.0` the codec derived a
/// canonical content key per node with `tiler_ir::program::abi::expr_key`, which
/// frames each operand's whole key inside its node's key, so a chain of depth `d`
/// carried a key linear in `d` and an arena of `d` such nodes carried key bytes
/// quadratic in `d`. A wide fan of shallow nodes would grow the manifest at the
/// same rate and that table only linearly, so it would not separate the two.
///
/// `14.0` orders the arena through `compare_expr_nodes` and derives no table at
/// all, and the shape is retained unchanged because that is exactly what makes
/// the two recorded result files comparable: the same depth, measured twice.
///
/// The head is boolean, so it can be a launch precondition — the one use site an
/// out-of-crate producer still supplies, and therefore the only way to make an
/// arena of a chosen size reachable.
fn deep_arena(builder: &mut ArtifactProgramBuilder, nodes: usize) -> Option<AbiExprId> {
    if nodes == 0 {
        return None;
    }
    let step = builder
        .push_root(AbiRoot::UnsignedLiteral(1))
        .expect("a literal root");
    let mut head = builder
        .push_root(AbiRoot::UnsignedLiteral(2))
        .expect("a literal root");
    for _ in 0..nodes {
        head = builder
            .push_binary(AbiBinaryOp::CheckedAdd, head, step)
            .expect("the arena admits the chain");
    }
    let limit = builder
        .push_root(AbiRoot::UnsignedLiteral(u64::MAX))
        .expect("a literal root");
    Some(
        builder
            .push_binary(AbiBinaryOp::LessOrEqual, head, limit)
            .expect("the chain heads a predicate"),
    )
}

/// Builds the delivered-realization record every executable artifact carries.
///
/// The eleven resolutions are derived from the packaged program's own scheduled
/// realization rather than restated here, so this harness cannot describe a
/// contract its plan does not schedule. Every packaged entry is bound, in the
/// flat declared space; one variant means that is the program's stage count.
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
fn object_of(bytes: usize) -> Vec<u8> {
    (0..bytes)
        .map(|index| u8::try_from(index % 251).expect("a remainder below 251 fits in a byte"))
        .collect()
}
