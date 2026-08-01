//! The artifact this suite routes, assembled the way a second backend has to.
//!
//! # Why the fixture builds a real artifact rather than hand-writing bytes
//!
//! The claim under test is that a consumer's own adapter executes a carried
//! payload **through the ordinary loader and route path**. An envelope written
//! by hand would prove that a decoder accepts hand-written bytes. So the program
//! is a real verified semantic graph, the plan is a real verified kernel program
//! over a real scheduled region, and the artifact is assembled through
//! `ArtifactProgramBuilder`, encoded, and decoded back — every check the
//! artifact layer performs runs on the way in.
//!
//! Every declaration here is the fixture's own: a governed profile key that is
//! not Apple's, a backend family that is not `tiler.metal`, an executable
//! representation that is not `metallib`, and a payload whose bytes no Metal
//! toolchain produced. Nothing in `crates/` was changed to admit them, which is
//! the same property the bounded scalar CPU vertical measured.
//!
//! # The transports are deliberately not the identity
//!
//! ABI slot 0 is the read binding and slot 1 the write binding, and the fused
//! payload declares transports `[1, 0]`. A backend that assumed a slot occupies
//! the transport of the same number would bind the input where the output goes,
//! and nothing else in the stack would notice. Metal's mapping is not the
//! identity in general; making the fixture's non-identity is what turns
//! `RoutedBinding::transport_slot` from a field into a checked fact.
//!
//! The materialized member goes one further and gives its two entries
//! *opposite* mappings — `[1, 0]` then `[0, 1]` — so a backend that resolved one
//! entry's transports and reused them for the next binds the shared scratch
//! where the result goes. A single mapping shared by every entry could not
//! distinguish a per-entry resolution from a per-route one.
//!
//! # Two packaged plans over one semantic program
//!
//! [`PackagedPlan::Fused`] packages one stage computing the whole graph.
//! [`PackagedPlan::Materialized`] packages the same graph as two stages with an
//! explicit intermediate: a pointwise map writing entry-internal scratch, and a
//! strict serial reduction reading it. They are alternative implementations of
//! one meaning, so both must agree with the same reference evaluation — and the
//! materialized one is what reaches the loader's multi-entry and
//! shared-allocation paths, which a single-entry route can only exercise as the
//! empty case.

use tiler_artifact::program::{
    ArtifactExecutionPolicy, ArtifactProgramBuilder, AvailabilityPhase, BackendEntryKey,
    BackendEntryRef, BackendFeatureRequirement, BackendKey, BindingKind, BindingSpec,
    CapabilityKey, CompilationEnvironment, DeferredPredicateSpec, EntrySpec, FeasibilityRuleSetKey,
    FeasibilityRuleSetRef, LaunchSpec, PayloadContent, PayloadMetadata, PayloadProvenance,
    PayloadSdkIdentity, RecordedArtifactProgramIdentity, RepresentationKey, RouteFeatureKey,
    RouteRequirement, SchemaVersion, SelectedProvider, TargetProfileDescriptorDigest,
    TargetProfileKey, TargetProfileRef, TargetPropertyKey, ToolComponent, VariantSpec,
};
use tiler_ir::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use tiler_ir::program::abi::{
    AbiRoot, PreparedEntryTargetRequirement, TargetPropertyProviderIdentity, TargetPropertyQuery,
    TargetPropertyRequirementRelation,
};
use tiler_ir::program::{
    AllocationOwnership, AllocationSpec, KernelProgramBuilder, MaterializedOrigin,
    MaterializedValueSpec, MemorySpace, RoutingCommitState, RoutingCommitTransition,
    SemanticOccurrence, StageAccess, StageAccessMode, StageLaunch, StorageEncoding, StorageScalar,
    ValueRole, VerifiedKernelProgram,
};
use tiler_ir::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContributorOrder,
    ExceptionalValueAssumption, ExecutionBinding, InputOrdinal, KernelSchedule, LaunchPlan,
    LogicalAccess, NumericalPermission, NumericalRealization, OwnershipProof, OwnershipProofKind,
    OwnershipWitnessId, PointwiseF32Expression, PointwiseF32ExpressionBuilder, ReductionTopology,
    RegionId, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, ProviderIdentity, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

use crate::image::{IDENTITY_BIAS_BITS, IDENTITY_SCALE_BITS, ScalarEntry, ScalarImage, encode};

/// Governed backend family of the fixture's own backend.
pub const BACKEND_KEY: &str = "tiler.test.scalar-host";
/// Governed executable representation the fixture's backend consumes.
pub const REPRESENTATION_KEY: &str = "tiler.test.scalar-host-image-v1";
/// Governed representation of the source the backend retained.
pub const SOURCE_REPRESENTATION_KEY: &str = "tiler.test.scalar-host-source-v1";
/// Governed target-profile key of the fixture's host.
pub const PROFILE_KEY: &str = "tiler.test.scalar-host-profile";
/// Exact descriptor identity of that profile.
pub const PROFILE_DESCRIPTOR: &[u8] = b"scalar-host-descriptor-a";
/// The backend's own entry-point symbol for the fused member's one entry.
pub const ENTRY_SYMBOL: &str = "scalar_fused_serial_sum";
/// The backend's own entry-point symbol for the materialized member's first entry.
pub const POINTWISE_SYMBOL: &str = "scalar_pointwise_scale_bias";
/// The backend's own entry-point symbol for the materialized member's second entry.
pub const REDUCTION_SYMBOL: &str = "scalar_strict_serial_sum";

/// Governed key of the route requirement this backend owns.
pub const HOST_ARITHMETIC_FEATURE: &str = "tiler.test.scalar-host.route-requirement.strict-f32";
/// Governed version of that requirement's meaning, matched exactly.
pub const HOST_ARITHMETIC_VERSION: u32 = 1;
/// Canonical payload of that requirement.
pub const HOST_ARITHMETIC_PAYLOAD: &[u8] = b"subnormals-preserved";

/// Prepared-entry property the fixture's deferred predicate queries.
pub const PREPARED_PROPERTY_KEY: &str = "tiler.target.prepared-entry.max-invocations";
/// Threshold that property must reach.
pub const PREPARED_PROPERTY_MINIMUM: u64 = 2;

/// Rows of the packaged input, which is also the output element count.
pub const ROWS: u64 = 2;
/// Columns of the packaged input, which is the reduction extent.
pub const COLUMNS: u64 = 3;
/// Bit pattern of the pointwise scale constant, `2.0f32`.
pub const SCALE_BITS: u32 = 0x4000_0000;
/// Bit pattern of the pointwise bias constant, `1.0f32`.
pub const BIAS_BITS: u32 = 0x3f80_0000;
/// Bit pattern of the canonical quiet NaN the realization declares.
pub const CANONICAL_NAN: u32 = 0x7fc0_0000;

/// Returns the fixture's declared target profile.
#[must_use]
pub fn profile() -> TargetProfileRef {
    profile_named(PROFILE_KEY, PROFILE_DESCRIPTOR)
}

/// Returns one declared target profile by key and descriptor.
#[must_use]
pub fn profile_named(key: &str, descriptor: &[u8]) -> TargetProfileRef {
    TargetProfileRef {
        key: TargetProfileKey::new(key).expect("a governed profile key"),
        descriptor: TargetProfileDescriptorDigest::from_bytes(descriptor)
            .expect("a descriptor identity"),
    }
}

/// Returns the fixture's governed backend family key.
#[must_use]
pub fn backend() -> BackendKey {
    BackendKey::new(BACKEND_KEY).expect("a governed backend key")
}

/// Returns the fixture's governed representation key.
#[must_use]
pub fn representation() -> RepresentationKey {
    RepresentationKey::new(REPRESENTATION_KEY).expect("a governed representation key")
}

/// Returns the backend-scoped route requirement the fixture's adapter owns.
#[must_use]
pub fn host_arithmetic_requirement(owner: BackendKey) -> RouteRequirement {
    RouteRequirement::BackendFeature(
        BackendFeatureRequirement::new(
            owner,
            RouteFeatureKey::new(HOST_ARITHMETIC_FEATURE).expect("a governed feature key"),
            HOST_ARITHMETIC_VERSION,
            HOST_ARITHMETIC_PAYLOAD,
        )
        .expect("a well-formed backend feature requirement"),
    )
}

/// Returns one deferred prepared-entry predicate bound to a variant entry.
///
/// The index names a position in the variant's entry table. A materialized
/// member carries one per entry, so the loader asks about each prepared entry
/// separately and an adapter answering from a route-wide property rather than
/// from the named entry's own prepared state answers at least one of them wrong.
#[must_use]
pub fn prepared_predicate(entry: u32) -> DeferredPredicateSpec {
    let query = TargetPropertyQuery::new(
        TargetPropertyKey::new(PREPARED_PROPERTY_KEY).expect("a governed property key"),
        AvailabilityPhase::PreparedKernelPreflight,
        TargetPropertyProviderIdentity::new("tiler-test", "scalar-host-prepared-entry", 1)
            .expect("a property provider identity"),
    )
    .expect("a well-formed target property query");
    DeferredPredicateSpec {
        requirement: PreparedEntryTargetRequirement::new(
            query,
            PREPARED_PROPERTY_MINIMUM,
            TargetPropertyRequirementRelation::ObservedAtLeastRequired,
        )
        .expect("a well-formed prepared-entry requirement"),
        entry,
    }
}

/// Returns the scalar image the fused member's payload carries.
#[must_use]
pub fn sound_image() -> ScalarImage {
    ScalarImage {
        entries: vec![ScalarEntry {
            symbol: ENTRY_SYMBOL.to_owned(),
            // Not the identity: ABI slot 0 (read) occupies transport 1 and ABI
            // slot 1 (write) occupies transport 0. See the module documentation.
            read_transport: 1,
            write_transport: 0,
            rows: u32::try_from(ROWS).expect("a small fixture extent"),
            columns: u32::try_from(COLUMNS).expect("a small fixture extent"),
            scale_bits: SCALE_BITS,
            bias_bits: BIAS_BITS,
        }],
    }
}

/// Returns the two-entry scalar image the materialized member's payload carries.
///
/// One object carrying both entries, which is the ordinary case for a backend
/// that emits a library rather than a translation unit per kernel. The loader
/// still validates and routes each entry separately, and the two entries'
/// opposite transport maps are what make that visible.
#[must_use]
pub fn sound_materialized_image() -> ScalarImage {
    ScalarImage {
        entries: vec![
            ScalarEntry {
                symbol: POINTWISE_SYMBOL.to_owned(),
                read_transport: 1,
                write_transport: 0,
                // One contributor per output element: the pointwise stage maps
                // every element of the input to the same position of the
                // scratch, so its "reduction" is over a single column.
                rows: u32::try_from(ROWS * COLUMNS).expect("a small fixture extent"),
                columns: 1,
                scale_bits: SCALE_BITS,
                bias_bits: BIAS_BITS,
            },
            ScalarEntry {
                symbol: REDUCTION_SYMBOL.to_owned(),
                // The opposite of the entry above, so a resolution cached across
                // entries binds the scratch where the result goes.
                read_transport: 0,
                write_transport: 1,
                rows: u32::try_from(ROWS).expect("a small fixture extent"),
                columns: u32::try_from(COLUMNS).expect("a small fixture extent"),
                // The exact identity contributor map: the pointwise stage
                // already applied the scale and the bias, and applying either
                // again would compute a different function.
                scale_bits: IDENTITY_SCALE_BITS,
                bias_bits: IDENTITY_BIAS_BITS,
            },
        ],
    }
}

/// Which packaged plan an assembled fixture carries.
///
/// Not derived from the entry count, deliberately. The plan decides how many
/// stages the packaged program has and the entry list decides what the payload
/// says about each; keeping them separate statements is what lets the artifact
/// layer's own cardinality check be the thing that catches a disagreement,
/// rather than this file arranging for one never to be expressible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackagedPlan {
    /// One stage computing the whole semantic graph.
    Fused,
    /// A pointwise stage and a reduction stage over an explicit intermediate.
    Materialized,
}

/// What one packaged entry's payload declares about itself.
#[derive(Clone, Debug)]
pub struct FixtureEntry {
    /// The backend entry this payload mapping realizes.
    pub key: BackendEntryKey,
    /// The backend's own entry-point symbol inside the carried object.
    pub symbol: String,
    /// Backend transport slot each ABI binding occupies, in slot order.
    pub transports: Vec<u32>,
}

/// What an assembled fixture artifact varies.
///
/// A plain input record with public fields: a caller writes the literal, and
/// growing it is a construction change either way (ADR 0074 convention 5a's
/// stated asymmetry).
#[derive(Clone, Debug)]
pub struct FixtureSpec {
    /// Exact carried object bytes.
    ///
    /// Varied on its own by most perturbations here, and that is the point:
    /// artifact identity excludes the emitted object, so a damaged image
    /// produces an artifact with the *same* canonical identity as a sound one.
    pub code: Vec<u8>,
    /// Which packaged plan the variant carries.
    pub plan: PackagedPlan,
    /// Governed backend family the payload declares.
    pub backend: BackendKey,
    /// Governed representation the payload declares.
    pub representation: RepresentationKey,
    /// Profile the variant was assessed against.
    pub variant_profile: TargetProfileRef,
    /// Profile the carried payload's own bytes were built for.
    pub payload_profile: TargetProfileRef,
    /// How the payload reaches an executable state.
    pub execution_policy: ArtifactExecutionPolicy,
    /// Additional live-device requirements of the route.
    pub route_requirements: Vec<RouteRequirement>,
    /// Deferred prepared-entry predicates of the variant.
    pub deferred_predicates: Vec<DeferredPredicateSpec>,
    /// The packaged entries, in the plan's own stage order.
    pub entries: Vec<FixtureEntry>,
}

/// Returns the governed entry key of one packaged backend entry.
#[must_use]
pub fn entry_key(name: &[u8]) -> BackendEntryKey {
    BackendEntryKey::from_bytes(name).expect("a governed entry key")
}

impl FixtureSpec {
    /// Returns the materialized member: two entries over one shared scratch.
    ///
    /// The shape the Metal proof's materialized route runs on hardware, made
    /// gate-resident here: two dispatches, one entry-internal allocation written
    /// by the first and read by the second, and every stage of the seam driven
    /// twice.
    #[must_use]
    pub fn materialized() -> Self {
        Self {
            code: encode(&sound_materialized_image()),
            plan: PackagedPlan::Materialized,
            // One per entry rather than one for the route. The loader asks about
            // each prepared entry by its own position, and a single predicate
            // would leave the second entry's prepared state unexamined.
            deferred_predicates: vec![prepared_predicate(0), prepared_predicate(1)],
            entries: vec![
                FixtureEntry {
                    key: entry_key(b"scalar-host-pointwise"),
                    symbol: POINTWISE_SYMBOL.to_owned(),
                    transports: vec![1, 0],
                },
                FixtureEntry {
                    key: entry_key(b"scalar-host-reduction"),
                    symbol: REDUCTION_SYMBOL.to_owned(),
                    transports: vec![0, 1],
                },
            ],
            ..Self::default()
        }
    }
}

impl Default for FixtureSpec {
    fn default() -> Self {
        Self {
            code: encode(&sound_image()),
            plan: PackagedPlan::Fused,
            backend: backend(),
            representation: representation(),
            variant_profile: profile(),
            payload_profile: profile(),
            execution_policy: ArtifactExecutionPolicy::NativeImage,
            route_requirements: vec![host_arithmetic_requirement(backend())],
            deferred_predicates: vec![prepared_predicate(0)],
            entries: vec![FixtureEntry {
                key: entry_key(b"scalar-host-fused"),
                symbol: ENTRY_SYMBOL.to_owned(),
                transports: vec![1, 0],
            }],
        }
    }
}

/// One assembled fixture: the envelope bytes and the identity recorded beside them.
#[derive(Clone, Debug)]
pub struct Fixture {
    /// The canonical envelope bytes a consumer loads.
    pub bytes: Vec<u8>,
    /// The identity the producing side recorded, as a caller would state it.
    pub expected: RecordedArtifactProgramIdentity,
}

/// Assembles, verifies, and encodes one fixture artifact.
///
/// # Panics
///
/// Panics when the artifact does not verify or does not encode. A fixture that
/// cannot be built is a defect in this file rather than a case under test.
#[must_use]
pub fn assemble(spec: &FixtureSpec) -> Fixture {
    let semantic = semantic_program();
    let program = match spec.plan {
        PackagedPlan::Fused => fused_program(&semantic),
        PackagedPlan::Materialized => materialized_program(&semantic),
    };
    // One provider offering one capability, realized by either packaged plan.
    // The capability is what the semantic graph asks for; how many stages
    // implement it is a physical choice below it.
    let provider =
        ProviderIdentity::new("tiler-test", "scalar-host-serial-sum", 1).expect("a provider");
    let environment = CompilationEnvironment::new([provider.clone()]).expect("an environment");
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).expect("an artifact draft");
    draft
        .select_provider(SelectedProvider {
            provider,
            capability: CapabilityKey::new("tiler.capability.serial-sum")
                .expect("a capability key"),
            capability_revision: 1,
        })
        .expect("the selected provider was offered");

    let payload = draft
        .push_carried_payload(
            spec.backend.clone(),
            spec.representation.clone(),
            SchemaVersion::new(1, 0),
            spec.payload_profile.clone(),
            spec.execution_policy,
            PayloadContent {
                metadata: PayloadMetadata {
                    source_representation: RepresentationKey::new(SOURCE_REPRESENTATION_KEY)
                        .expect("a governed source representation"),
                    // The retained source describes what was translated, and it
                    // is a function of the packaged plan alone — deliberately
                    // independent of `code`. Every payload perturbation below
                    // therefore changes the carried object and *not* the
                    // compilation subject, which is what makes two artifacts
                    // with different bytes share one canonical identity — the
                    // asymmetry ADR 0090 item 8 rests on.
                    source: match spec.plan {
                        PackagedPlan::Fused => {
                            b"fused-multiply-add-strict-serial-sum rows=2 columns=3".to_vec()
                        }
                        PackagedPlan::Materialized => {
                            b"multiply-add then strict-serial-sum rows=2 columns=3".to_vec()
                        }
                    },
                    provenance: PayloadProvenance {
                        toolchain: "tiler.test.scalar-image-translator".to_owned(),
                        target: "aarch64-apple-darwin".to_owned(),
                        family: BACKEND_KEY.to_owned(),
                        language: "tiler.kernel-ir.v4".to_owned(),
                        // Apple-shaped required fields with no meaning for this
                        // backend. ADR 0090 item 14 names that gap; stating this
                        // representation's own version here rather than a
                        // platform claim is the same compromise the CPU vertical
                        // recorded, not a new one.
                        deployment_major: 1,
                        deployment_minor: 0,
                        components: vec![ToolComponent {
                            role: "translator".to_owned(),
                            version: "1".to_owned(),
                        }],
                        sdk: PayloadSdkIdentity {
                            name: "tiler.test.scalar-host-image".to_owned(),
                            version: "1".to_owned(),
                            build: "0".to_owned(),
                        },
                        compile_flags: Vec::new(),
                        link_flags: Vec::new(),
                    },
                    entries: spec
                        .entries
                        .iter()
                        .map(|entry| tiler_artifact::program::PayloadEntryMapping {
                            entry_key: entry.key.clone(),
                            symbol: entry.symbol.clone(),
                            transports: entry.transports.clone(),
                        })
                        .collect(),
                    obligations: Vec::new(),
                },
                code: spec.code.clone(),
            },
        )
        .expect("the fixture payload was accepted");

    let variant = draft
        .push_variant(
            &program,
            VariantSpec {
                target_profile: spec.variant_profile.clone(),
                feasibility_rules: FeasibilityRuleSetRef {
                    key: FeasibilityRuleSetKey::new("tiler.test.scalar-host.feasibility")
                        .expect("a governed rule-set key"),
                    revision: 1,
                },
                deferred_predicates: spec.deferred_predicates.clone(),
                entries: spec
                    .entries
                    .iter()
                    .map(|entry| EntrySpec {
                        // Every kernel this fixture's profile verifies
                        // destructures to one read buffer and one write buffer.
                        bindings: vec![
                            BindingSpec {
                                kind: BindingKind::Buffer,
                            },
                            BindingSpec {
                                kind: BindingKind::Buffer,
                            },
                        ],
                        launch: LaunchSpec {
                            zero_work_skips_dispatch: true,
                            preconditions: Vec::new(),
                        },
                        implementation: BackendEntryRef {
                            payload,
                            entry_key: entry.key.clone(),
                        },
                    })
                    .collect(),
            },
        )
        .expect("the fixture variant packages the bound plan");

    for requirement in &spec.route_requirements {
        draft
            .require_route(variant, requirement.clone())
            .expect("the fixture route requirement was accepted");
    }

    let artifact = draft.build().expect("the fixture artifact verifies");
    let bytes = artifact.encode().expect("the fixture artifact encodes");
    let expected =
        RecordedArtifactProgramIdentity::from_bytes(artifact.canonical_identity().as_bytes())
            .expect("the producing side records its own identity");
    Fixture { bytes, expected }
}

// -------------------------------------------------------------------------
// The shared-IR half
// -------------------------------------------------------------------------

/// Returns the numerical realization the fixture's kernel declares.
fn strict() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-f32",
        CANONICAL_NAN,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    )
}

fn input_shape() -> Shape {
    Shape::from_dims([ROWS, COLUMNS])
}

fn output_shape() -> Shape {
    Shape::from_dims([ROWS])
}

/// Returns the interface key of the one named program input.
#[must_use]
pub fn input_key() -> InputKey {
    InputKey::new("input").expect("a valid interface key")
}

/// Builds the pointwise prefix of [`semantic_program`], as an oracle only.
///
/// **Never packaged.** No artifact this fixture assembles carries a plan for
/// this graph; it exists so a test can evaluate the materialized member's
/// *intermediate* through `tiler-reference` rather than by restating the
/// interpreter's arithmetic. Its operations are exactly the four the
/// materialized member's pointwise stage claims coverage of, which is what makes
/// its result the value that stage is obliged to write.
#[must_use]
pub fn pointwise_semantic_program() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("the standard registry composes");
    let input = draft
        .input::<F32>(input_key(), input_shape())
        .expect("the input binds");
    let scale = F32Constant::apply(&mut draft, SCALE_BITS).expect("the scale constant");
    let bias = F32Constant::apply(&mut draft, BIAS_BITS).expect("the bias constant");
    let product = F32Multiply::apply(&mut draft, input, scale).expect("the pointwise product");
    let mapped = F32Add::apply(&mut draft, product, bias).expect("the pointwise sum");
    draft
        .output(
            OutputKey::new("result").expect("a valid output key"),
            mapped,
        )
        .expect("the output binds");
    draft.build().expect("the program verifies")
}

/// Builds the verified semantic graph the artifact packages a plan for.
#[must_use]
pub fn semantic_program() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("the standard registry composes");
    let input = draft
        .input::<F32>(input_key(), input_shape())
        .expect("the input binds");
    let scale = F32Constant::apply(&mut draft, SCALE_BITS).expect("the scale constant");
    let bias = F32Constant::apply(&mut draft, BIAS_BITS).expect("the bias constant");
    let product = F32Multiply::apply(&mut draft, input, scale).expect("the pointwise product");
    let mapped = F32Add::apply(&mut draft, product, bias).expect("the pointwise sum");
    let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)])
        .expect("the strict reduction");
    draft
        .output(OutputKey::new("result").expect("a valid output key"), sum)
        .expect("the output binds");
    draft.build().expect("the program verifies")
}

/// Builds the one fused reduction kernel the packaged plan dispatches.
fn fused_kernel() -> VerifiedKernel {
    let axes = vec![Axis::new(1)];
    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region
        .iteration_shape(output_shape())
        .expect("the iteration shape");
    region
        .push_access(Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .expect("the read access");
    region
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .expect("the write access");
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .expect("the read bounds proof");
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: ROWS,
            },
        })
        .expect("the write bounds proof");
    region
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: ROWS },
        })
        .expect("the ownership proof");
    region
        .scalar_program(ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits: SCALE_BITS,
            bias_bits: BIAS_BITS,
            axes: axes.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: CANONICAL_NAN,
            empty_identity_bits: 0,
            contraction: false,
        })
        .expect("the scalar program");
    region.numerical(strict()).expect("the realization");
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: ROWS,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::Serial {
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            launch: LaunchPlan {
                grid_threads: ROWS,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .expect("the schedule");
    lower_scheduled_region(&region.build().expect("the region verifies"))
        .expect("the kernel lowers")
}

/// Builds the single-stage kernel program the artifact packages.
#[must_use]
pub fn fused_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    let kernel = fused_kernel();
    let mut plan = KernelProgramBuilder::new(semantic).expect("a plan draft");
    let external = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: ROWS * COLUMNS * 4,
            alignment: 4,
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .expect("the external allocation");
    let owned = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: ROWS * 4,
            alignment: 4,
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::Program,
        })
        .expect("the program allocation");
    let source = plan
        .push_value(
            MaterializedValueSpec {
                origin: MaterializedOrigin::ProgramInput { key: input_key() },
                role: ValueRole::Input,
                shape: input_shape(),
                storage_scalar: StorageScalar::F32,
                element_type: KernelType::F32,
                encoding: StorageEncoding::Unpacked,
                alignment: 4,
                memory_space: MemorySpace::Device,
            },
            external,
        )
        .expect("the input value");
    let result = plan
        .push_value(
            MaterializedValueSpec {
                origin: MaterializedOrigin::Internal,
                role: ValueRole::Output,
                shape: output_shape(),
                storage_scalar: StorageScalar::F32,
                element_type: KernelType::F32,
                encoding: StorageEncoding::Unpacked,
                alignment: 4,
                memory_space: MemorySpace::Device,
            },
            owned,
        )
        .expect("the output value");
    let read = plan.push_whole_view(source).expect("the read view");
    let write = plan.push_whole_view(result).expect("the write view");

    let mut literal = |value: u64| {
        plan.push_abi_root(AbiRoot::UnsignedLiteral(value))
            .expect("an abi literal")
    };
    let read_bytes = literal(ROWS * COLUMNS * 4);
    let write_bytes = literal(ROWS * 4);
    let grid_threads = literal(ROWS);
    let threads_per_workgroup = literal(1);
    let guard = plan
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("the guard predicate");
    plan.applicability_guard(guard)
        .expect("the applicability guard");
    declare_routing_commit(&mut plan);

    plan.push_stage(
        &kernel,
        &(0..5).map(SemanticOccurrence::new).collect::<Vec<_>>(),
        &[
            StageAccess {
                view: read,
                mode: StageAccessMode::Read,
                accessible_bytes: read_bytes,
            },
            StageAccess {
                view: write,
                mode: StageAccessMode::Write,
                accessible_bytes: write_bytes,
            },
        ],
        StageLaunch {
            grid_threads,
            threads_per_workgroup,
        },
    )
    .expect("the stage binds");
    plan.push_output(
        OutputKey::new("result").expect("a valid output key"),
        result,
    )
    .expect("the program output");
    plan.build().expect("the plan verifies")
}

/// Declares the complete routing-commit lifecycle every packaged plan spans.
///
/// Shared by both members rather than restated, because the lifecycle is a
/// property of ADR 0051 and not of either plan's shape: two members that stated
/// it separately could drift, and a member whose commit permitted a fallback
/// would be the one thing this suite's post-commit cases could not detect.
fn declare_routing_commit(plan: &mut KernelProgramBuilder) {
    for (from, to, fallback_permitted) in [
        (
            RoutingCommitState::Preflight,
            RoutingCommitState::Committed,
            true,
        ),
        (
            RoutingCommitState::Committed,
            RoutingCommitState::Executing,
            false,
        ),
        (
            RoutingCommitState::Executing,
            RoutingCommitState::Published,
            false,
        ),
    ] {
        plan.push_routing_commit_transition(RoutingCommitTransition {
            from,
            to,
            fallback_permitted,
        })
        .expect("a routing-commit transition");
    }
}

/// Returns the pointwise `input * scale + bias` expression both members apply.
fn scale_bias_expression() -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression
        .input(InputOrdinal::FIRST)
        .expect("the pointwise input");
    let scale = expression.constant(SCALE_BITS).expect("the scale constant");
    let product = expression
        .multiply(input, scale)
        .expect("the pointwise product");
    let bias = expression.constant(BIAS_BITS).expect("the bias constant");
    let root = expression.add(product, bias).expect("the pointwise sum");
    expression.build(root).expect("the pointwise expression")
}

/// Builds the materialized member's first kernel: one input element to one temporary.
fn pointwise_kernel() -> VerifiedKernel {
    let count = ROWS * COLUMNS;
    let mut region = ScheduledRegionBuilder::new(RegionId::new(1));
    region
        .iteration_shape(input_shape())
        .expect("the iteration shape");
    region
        .push_access(Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .expect("the read access");
    // The written tensor is `Intermediate`, not `Output`. That is what makes the
    // value this stage writes an entry-internal one at the artifact layer, and
    // therefore what makes the pairing below a shared allocation rather than two
    // named program buffers a loader could tell apart by name.
    region
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .expect("the write access");
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: count,
            },
        })
        .expect("the read bounds proof");
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: count,
            },
        })
        .expect("the write bounds proof");
    region
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: count,
            },
        })
        .expect("the ownership proof");
    region
        .scalar_program(ScalarProgram::PointwiseF32(scale_bias_expression()))
        .expect("the scalar program");
    region.numerical(strict()).expect("the realization");
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: count,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: count,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .expect("the schedule");
    lower_scheduled_region(&region.build().expect("the region verifies"))
        .expect("the kernel lowers")
}

/// Builds the materialized member's second kernel: one temporary to the output.
fn reduction_kernel() -> VerifiedKernel {
    let axes = vec![Axis::new(1)];
    let mut region = ScheduledRegionBuilder::new(RegionId::new(2));
    region
        .iteration_shape(output_shape())
        .expect("the iteration shape");
    region
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .expect("the read access");
    region
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .expect("the write access");
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .expect("the read bounds proof");
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: ROWS,
            },
        })
        .expect("the write bounds proof");
    region
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: ROWS },
        })
        .expect("the ownership proof");
    region
        .scalar_program(ScalarProgram::StrictSerialSum {
            axes: axes.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: CANONICAL_NAN,
            empty_identity_bits: 0,
        })
        .expect("the scalar program");
    region.numerical(strict()).expect("the realization");
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: ROWS,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::Serial {
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            launch: LaunchPlan {
                grid_threads: ROWS,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .expect("the schedule");
    lower_scheduled_region(&region.build().expect("the region verifies"))
        .expect("the kernel lowers")
}

/// Builds the two-stage kernel program the materialized member packages.
///
/// The scratch value is `Internal`/`Temporary`, so both stages address it
/// through bindings the artifact layer can only describe as entry-internal — it
/// has no durable name for a program value. That is precisely the condition
/// under which a loader allocating per binding would hand the reduction a fresh
/// buffer and read uninitialised storage, so the pairing has to come from the
/// declared data dependency below. Nothing weaker than a real intermediate
/// reaches that path.
#[must_use]
pub fn materialized_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    let pointwise = pointwise_kernel();
    let reduction = reduction_kernel();
    let mut plan = KernelProgramBuilder::new(semantic).expect("a plan draft");
    let external = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: ROWS * COLUMNS * 4,
            alignment: 4,
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .expect("the external allocation");
    let scratch = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: ROWS * COLUMNS * 4,
            alignment: 4,
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::Program,
        })
        .expect("the scratch allocation");
    let owned = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: ROWS * 4,
            alignment: 4,
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::Program,
        })
        .expect("the program allocation");

    let source = plan
        .push_value(
            MaterializedValueSpec {
                origin: MaterializedOrigin::ProgramInput { key: input_key() },
                role: ValueRole::Input,
                shape: input_shape(),
                storage_scalar: StorageScalar::F32,
                element_type: KernelType::F32,
                encoding: StorageEncoding::Unpacked,
                alignment: 4,
                memory_space: MemorySpace::Device,
            },
            external,
        )
        .expect("the input value");
    let intermediate = plan
        .push_value(
            MaterializedValueSpec {
                origin: MaterializedOrigin::Internal,
                role: ValueRole::Temporary,
                shape: input_shape(),
                storage_scalar: StorageScalar::F32,
                element_type: KernelType::F32,
                encoding: StorageEncoding::Unpacked,
                alignment: 4,
                memory_space: MemorySpace::Device,
            },
            scratch,
        )
        .expect("the scratch value");
    let result = plan
        .push_value(
            MaterializedValueSpec {
                origin: MaterializedOrigin::Internal,
                role: ValueRole::Output,
                shape: output_shape(),
                storage_scalar: StorageScalar::F32,
                element_type: KernelType::F32,
                encoding: StorageEncoding::Unpacked,
                alignment: 4,
                memory_space: MemorySpace::Device,
            },
            owned,
        )
        .expect("the output value");
    let read = plan.push_whole_view(source).expect("the read view");
    let scratch_view = plan
        .push_whole_view(intermediate)
        .expect("the scratch view");
    let write = plan.push_whole_view(result).expect("the write view");

    let mut literal = |value: u64| {
        plan.push_abi_root(AbiRoot::UnsignedLiteral(value))
            .expect("an abi literal")
    };
    let element_bytes = literal(ROWS * COLUMNS * 4);
    let output_bytes = literal(ROWS * 4);
    let pointwise_threads = literal(ROWS * COLUMNS);
    let reduction_threads = literal(ROWS);
    let threads_per_workgroup = literal(1);
    let guard = plan
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("the guard predicate");
    plan.applicability_guard(guard)
        .expect("the applicability guard");
    declare_routing_commit(&mut plan);

    // The occurrence split is the decomposition: the pointwise stage claims the
    // two constants, the multiply, and the add; the reduction claims the sum.
    // Together they cover the graph exactly once, which is what makes these two
    // stages an implementation of the same meaning the fused member's one stage
    // implements.
    let map = plan
        .push_stage(
            &pointwise,
            &(0..4).map(SemanticOccurrence::new).collect::<Vec<_>>(),
            &[
                StageAccess {
                    view: read,
                    mode: StageAccessMode::Read,
                    accessible_bytes: element_bytes,
                },
                StageAccess {
                    view: scratch_view,
                    mode: StageAccessMode::Write,
                    accessible_bytes: element_bytes,
                },
            ],
            StageLaunch {
                grid_threads: pointwise_threads,
                threads_per_workgroup,
            },
        )
        .expect("the pointwise stage binds");
    let reduce = plan
        .push_stage(
            &reduction,
            &(4..5).map(SemanticOccurrence::new).collect::<Vec<_>>(),
            &[
                StageAccess {
                    view: scratch_view,
                    mode: StageAccessMode::Read,
                    accessible_bytes: element_bytes,
                },
                StageAccess {
                    view: write,
                    mode: StageAccessMode::Write,
                    accessible_bytes: output_bytes,
                },
            ],
            StageLaunch {
                grid_threads: reduction_threads,
                threads_per_workgroup,
            },
        )
        .expect("the reduction stage binds");
    // The typed read-after-write the loader derives its one shared allocation
    // from. Without it the program is still well formed and the two stages still
    // run in order — and the reduction reads a buffer nothing wrote.
    plan.push_data_dependency(map, reduce, intermediate)
        .expect("the data dependency");
    plan.push_output(
        OutputKey::new("result").expect("a valid output key"),
        result,
    )
    .expect("the program output");
    plan.build().expect("the plan verifies")
}
