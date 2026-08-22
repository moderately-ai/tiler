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
//!
//! [`PackagedPlan::FusedInapplicable`] is the fused plan under a guard that is
//! statically false. It exists so a portfolio can hold a variant this host *can*
//! execute and the producer excludes anyway, which is the one state that
//! separates "no eligible variant" from "no applicable variant" — two refusals
//! with opposite repairs.
//!
//! # A portfolio, and why its members need distinct plans
//!
//! [`assemble_portfolio`] packages one variant per member, at the routing rank
//! its position gives it, each with its own carried payload. That is what makes
//! a multi-backend-family artifact expressible here: the members declare
//! different backend families and representations, and the loader's eligibility
//! filter is what decides which of them this host may route to.
//!
//! Two constraints of the artifact layer shape what a portfolio may contain, and
//! neither is this file's invention. `push_variant` refuses a second variant
//! that packages the same kernel program under the same applicability guard, so
//! two members must differ in *plan* rather than only in backend — which is also
//! the realistic case, since two backends do not produce one physical plan. And
//! `check_subject` requires every variant of one artifact to declare the same
//! *variant* target profile, so a member varies the profile its **payload** was
//! built for, which is per-payload by design.
//!
//! # This module is path-shared, and what that costs
//!
//! This file and `image.rs` beside it are compiled into roots other than
//! `tests/adapter_route/main.rs` through `#[path]`, so that one assembly
//! authority exists rather than a copy. `tests/identity_join/main.rs` takes
//! `image.rs` that way, and
//! `spikes/target-profiles/metal-subgroup-width-route-gate/src/main.rs` takes
//! both — a root outside this workspace entirely.
//!
//! The cost is that a `crate::`-rooted path here resolves in the owning test
//! binary and nowhere else. That is not hypothetical: commit `2cb7c83c` added
//! four `crate::adapter::ScalarEnvironmentSchema` references to this file, the
//! owning suite stayed green because it *does* have an `adapter` module, and
//! the out-of-workspace consumer was left failing to compile with four
//! `error[E0433]: cannot find adapter in crate` — discovered months later by
//! hand, not by any check. `ScalarEnvironmentSchema` is declared above as a
//! result, and this module now reaches only `crate::image`.
//!
//! **The owner is this directory**, `crates/tiler-runtime/tests/adapter_route`:
//! a change here is the shared authority moving, and the sharing consumers do
//! not get a vote. **The check is
//! `crates/tiler-runtime/tests/adapter_route_portability.rs`**, a test target
//! whose only job is to compile this closed module set from a second root, the
//! way `prototypes/serial-sum-run/tests/lint_table.rs` compiles the
//! conformance crate's lint reader from a second root. It runs in the ordinary
//! package gate, so a back-edge added here is a compile error at the same
//! moment it is written. It also enumerates every `#[path]` consumer in the
//! repository and fails when one shares a module it does not cover, because a
//! hand-written module list that has stopped covering its domain reports no
//! drift exactly as loudly as one that has.

use std::collections::BTreeMap;

use tiler_runtime::load::DTypeDispatch;

use tiler_artifact::program::{
    ApproximationEnvelope, ArithmeticType, ArtifactBuildError, ArtifactExecutionPolicy,
    ArtifactProgramBuilder, AvailabilityPhase, BackendEntryKey, BackendEntryRef,
    BackendFeatureRequirement, BackendKey, BindingKind, BindingSpec, CANONICAL_DIMENSIONS,
    CapabilityFamilyKey, CompilationEnvironment, DIMENSION_COUNT, DeferredPredicateSpec,
    DeliveredRealizationBuilder, DimensionBehaviour, EntryRealization, EntrySpec,
    ExceptionalValueAssumption, FactSourceProvenance, FeasibilityRuleSetKey, FeasibilityRuleSetRef,
    HonouringMeans, LaunchSpec, LoweringCapabilitySubject, MaterializationRounding,
    NumericalDimension, NumericalObligationKey, NumericalPermission, PayloadContent,
    PayloadMetadata, PayloadPlanDeterminismRefusal, PayloadPlanDeterminismVerifier,
    PayloadPlatform, PayloadProvenance, PhysicalImplementationProposalIdentity,
    PhysicalProposalKind, PhysicalRegionOccurrenceIdentity, PolicyLocus, ProvenanceIdentity,
    RecordedArtifactProgramIdentity, RepresentationKey, RouteFeatureKey, RouteRequirement,
    ScalarArithmeticSubject, ScalarArithmeticSubjectIdentity, SchemaVersion,
    SelectedLoweringProvider, SelectedPhysicalImplementation, SemanticOccurrence, SubnormalMode,
    TargetEnvironmentDeclaration, TargetEnvironmentDescriptor, TargetEnvironmentDescriptorSchema,
    TargetEnvironmentReasonCode, TargetEvidenceDeclaration, TargetProfileDescriptorDigest,
    TargetProfileKey, TargetProfileRef, TargetPropertyKey, ToolComponent, VariantSpec,
    overlapping_behaviour,
};
use tiler_artifact::program::{BackendPayloadDescriptor, ValidatedTargetEnvironmentDeclaration};
use tiler_ir::index::{
    DomainRole, FrozenIndexRealizationLawRegistry, FrozenScalarRegistry, IndexInteger,
    IndexRealizationAuthority, IndexRefinementSubject, IndexRefinementVerificationOutcome,
    IndexRegionBuilder, NumericalContractIdentity, ScalarAttributes, ScalarOpKey,
    TensorRole as IndexTensorRole, VerifiedIndexRegion, add_f32_scalar_op, constant_f32_scalar_op,
    multiply_f32_scalar_op,
};
use tiler_ir::kernel::{
    KernelType, PlanDeterminismWitness, VerifiedKernel, lower_scheduled_region,
    verify_plan_determinism,
};
use tiler_ir::program::abi::{
    AbiBinaryOp, AbiRoot, PreparedEntryTargetRequirement, TargetPropertyProviderIdentity,
    TargetPropertyQuery, TargetPropertyRequirementRelation,
};
use tiler_ir::program::{
    AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec, ByteWindow,
    CoveredOccurrence, KernelProgramBuilder, MaterializedOrigin, MaterializedValueSpec,
    MemorySpace, RoutingCommitState, RoutingCommitTransition, StageAccess, StageAccessMode,
    StageLaunch, StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram,
};
// The five behaviour vocabularies `DimensionBehaviour` ranges over are named
// through `tiler_artifact::program` above rather than here, even where this half
// of the fixture uses them to schedule a region: they are one type reached by two
// paths, and naming them through the boundary a consumer at ADR 0081 item 2's
// closure actually has is what keeps this file evidence that the closure suffices.
use tiler_ir::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContractionAxisSource, ContributorOrder, ExecutionBinding, F32NumericalContractKey,
    KernelSchedule, LaunchPlan, LogicalAccess, NumericalRealization, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, PointwiseF32Expression, PointwiseF32ExpressionBuilder,
    ReductionTopology, RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder, TailPolicy,
    TensorRole,
};
use tiler_ir::semantic::{
    Bf16, CanonicalField, CanonicalValue, F32, F32_CONSTANT_BITS_ATTRIBUTE, F32Add, F32Constant,
    F32Multiply, InputKey, OpKey, OutputKey, ProviderIdentity, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum, add_f32_op, constant_f32_op, multiply_f32_op,
};

/// The provider this fixture grants **physical** authority to.
///
/// A different identity from the lowering provider on purpose: the two grants
/// are separate, and a fixture that reused one identity could satisfy a
/// cross-role check by accident.
fn physical_implementer() -> ProviderIdentity {
    ProviderIdentity::new("tiler-test", "scalar-host-implementer", 1).expect("a provider")
}

/// The physical role every artifact this fixture assembles is offered.
fn offered_physical() -> [ProviderIdentity; 1] {
    [physical_implementer()]
}

/// A canonical run of `rows` selections over distinct ascending occurrences.
///
/// The occurrence spelling is fixed-width so that byte order and ordinal order
/// agree; the canonical rule is over *bytes*, so a variable-width spelling would
/// order `10` before `9` and fail the rule this run exists to satisfy.
fn physical_run(rows: usize) -> Vec<SelectedPhysicalImplementation> {
    (0..u16::try_from(rows).expect("a bounded entry table fits u16"))
        .map(|ordinal| {
            let mut occurrence = b"tiler-test.occurrence.".to_vec();
            occurrence.extend_from_slice(&ordinal.to_be_bytes());
            let mut proposal = b"tiler-test.proposal.".to_vec();
            proposal.extend_from_slice(&ordinal.to_be_bytes());
            SelectedPhysicalImplementation {
                region_occurrence: PhysicalRegionOccurrenceIdentity::from_bytes(occurrence)
                    .expect("a nonempty occurrence identity"),
                implementation_proposal: PhysicalImplementationProposalIdentity::from_bytes(
                    proposal,
                )
                .expect("a nonempty proposal identity"),
                provider: physical_implementer(),
                proposal_kind: PhysicalProposalKind::ScheduledKernel,
            }
        })
        .collect()
}
use tiler_ir::shape::{Axis, Extent, Shape};
use tiler_runtime::load::ExecutionEnvironment;

use crate::image::{IDENTITY_BIAS_BITS, IDENTITY_SCALE_BITS, ScalarEntry, ScalarImage, encode};

/// Governed backend family of the fixture's own backend.
pub const BACKEND_KEY: &str = "tiler.test.scalar-host";
/// Governed executable representation the fixture's backend consumes.
pub const REPRESENTATION_KEY: &str = "tiler.test.scalar-host-image-v1";
/// Governed representation of the source the backend retained.
pub const SOURCE_REPRESENTATION_KEY: &str = "tiler.test.scalar-host-source-v1";
/// Governed backend family of the second family a portfolio declares.
///
/// Metal's real key. See [`FixtureSpec::metal`] for why a member of this family
/// carries an object no Metal toolchain produced and why that is sound.
pub const METAL_BACKEND_KEY: &str = "tiler.metal";
/// Governed executable representation that family consumes.
pub const METAL_REPRESENTATION_KEY: &str = "metallib";
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
/// The backend's own entry-point symbol for the live-extent direct variant.
pub const LIVE_EXTENT_SYMBOL: &str = "live_row_major";
/// Entry symbol of the live strict-contraction routing fixture.
pub const LIVE_CONTRACTION_SYMBOL: &str = "live_contraction";

/// Governed key of the route requirement this backend owns.
pub const HOST_ARITHMETIC_FEATURE: &str = "tiler.test.scalar-host.route-requirement.strict-f32";
/// Governed version of that requirement's meaning, matched exactly.
pub const HOST_ARITHMETIC_VERSION: u32 = 1;
/// Canonical payload of that requirement.
pub const HOST_ARITHMETIC_PAYLOAD: &[u8] = b"subnormals-preserved";

/// Prepared-entry property the fixture's deferred predicate queries.
pub const PREPARED_PROPERTY_KEY: &str = "tiler.target.prepared-entry.max-invocations";
/// Provider namespace that answers [`PREPARED_PROPERTY_KEY`].
pub const PREPARED_PROPERTY_PROVIDER_NAMESPACE: &str = "tiler-test";
/// Provider name that answers [`PREPARED_PROPERTY_KEY`].
pub const PREPARED_PROPERTY_PROVIDER_NAME: &str = "scalar-host-prepared-entry";
/// Provider revision that answers [`PREPARED_PROPERTY_KEY`].
pub const PREPARED_PROPERTY_PROVIDER_REVISION: u32 = 1;
/// A second legal prepared-entry key this adapter does not own.
///
/// Used to prove that an unrelated quantity equal to the required value cannot
/// admit a property nothing evaluated.
pub const FOREIGN_PREPARED_PROPERTY_KEY: &str =
    "tiler.target.prepared-entry.thread-execution-width";
/// Threshold that property must reach.
pub const PREPARED_PROPERTY_MINIMUM: u64 = 2;
/// Prepared-entry key of this adapter's per-entry subgroup execution width.
///
/// Owned by the same provider tuple as [`PREPARED_PROPERTY_KEY`] and compared
/// by **equality**: a subgroup route's combine steps are its content, so a
/// wider prepared width satisfies a floor while running lane arithmetic
/// nothing verified. The row exists to drive the loader's retained
/// `ObservedEqualsRequired` relation from an artifact's own bytes.
pub const SUBGROUP_WIDTH_PROPERTY_KEY: &str = "tiler.target.prepared-entry.subgroup-width";
/// The per-entry subgroup width this host's prepared state reports.
///
/// Distinct per entry so answering one entry's row from another's prepared
/// state is an observable substitution rather than a coincidence.
pub const SCALAR_SUBGROUP_WIDTHS: [u64; 2] = [4, 8];

/// Live-device property the property-guarded fused member's guard reads.
///
/// A route-time selection key the caller binds beside the interface facts. It
/// exists so a portfolio over the fixed interface can still select different
/// ranks on different routes without pretending a fixed input axis varies.
pub const SELECTION_PROPERTY_KEY: &str = "tiler.target.test.selection@1";

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

/// Returns Metal's governed backend family key.
#[must_use]
pub fn metal_backend() -> BackendKey {
    BackendKey::new(METAL_BACKEND_KEY).expect("a governed backend key")
}

/// Returns Metal's governed executable representation key.
#[must_use]
pub fn metal_representation() -> RepresentationKey {
    RepresentationKey::new(METAL_REPRESENTATION_KEY).expect("a governed representation key")
}

/// Returns the dtype row a family dispatching both packaged widths declares.
///
/// Both, and not only the width a given fixture packages: a host declares what
/// its *family* can dispatch, which is a fact about the machine rather than
/// about the artifact in front of it. A helper that declared only the width
/// under test would make every positive case pass for the wrong reason.
#[must_use]
pub fn dispatches_f32_and_bf16() -> BTreeMap<ArithmeticType, DTypeDispatch> {
    BTreeMap::from([
        (ArithmeticType::F32, DTypeDispatch::Dispatchable),
        (ArithmeticType::Bf16, DTypeDispatch::Dispatchable),
    ])
}

/// Returns the execution environment a host of the fixture's own family states.
#[must_use]
pub fn scalar_host() -> ExecutionEnvironment {
    ExecutionEnvironment {
        target_profile: profile(),
        backend: backend(),
        representation: representation(),
        dtype_dispatch: dispatches_f32_and_bf16(),
    }
}

/// Returns the execution environment a Metal host states, over the same profile.
///
/// Deliberately the *same* target profile as [`scalar_host`]. Every variant of
/// one artifact declares one variant profile, so a Metal host that also differed
/// in profile would be filtered on the profile and never reach the pair
/// comparison this environment exists to exercise.
#[must_use]
pub fn metal_host() -> ExecutionEnvironment {
    ExecutionEnvironment {
        target_profile: profile(),
        backend: metal_backend(),
        representation: metal_representation(),
        dtype_dispatch: dispatches_f32_and_bf16(),
    }
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
    prepared_predicate_owned(
        entry,
        PREPARED_PROPERTY_KEY,
        PREPARED_PROPERTY_PROVIDER_NAMESPACE,
        PREPARED_PROPERTY_PROVIDER_NAME,
        PREPARED_PROPERTY_PROVIDER_REVISION,
    )
}

/// Returns one deferred prepared-entry predicate with caller-chosen ownership.
///
/// The adapter exact-matches namespace, name, revision, and key before reading
/// a quantity. A fixture that names any other ownership is how an unknown
/// property is shown to refuse rather than compare equal.
#[must_use]
pub fn prepared_predicate_owned(
    entry: u32,
    key: &str,
    namespace: &str,
    name: &str,
    revision: u32,
) -> DeferredPredicateSpec {
    let query = TargetPropertyQuery::new(
        TargetPropertyKey::new(key).expect("a governed property key"),
        AvailabilityPhase::PreparedKernelPreflight,
        TargetPropertyProviderIdentity::new(namespace, name, revision)
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

/// Returns one subgroup-width confirmation bound to a variant entry.
///
/// `ObservedEqualsRequired`, never a floor, and one row per requiring entry
/// with its own required width: two prepared pipelines may report different
/// widths, so deduplicating across entries would compare one pipeline's width
/// against another entry's requirement.
#[must_use]
pub fn subgroup_width_predicate(entry: u32, required: u64) -> DeferredPredicateSpec {
    let query = TargetPropertyQuery::new(
        TargetPropertyKey::new(SUBGROUP_WIDTH_PROPERTY_KEY).expect("a governed property key"),
        AvailabilityPhase::PreparedKernelPreflight,
        TargetPropertyProviderIdentity::new(
            PREPARED_PROPERTY_PROVIDER_NAMESPACE,
            PREPARED_PROPERTY_PROVIDER_NAME,
            PREPARED_PROPERTY_PROVIDER_REVISION,
        )
        .expect("a property provider identity"),
    )
    .expect("a well-formed target property query");
    DeferredPredicateSpec {
        requirement: PreparedEntryTargetRequirement::new(
            query,
            required,
            TargetPropertyRequirementRelation::ObservedEqualsRequired,
        )
        .expect("a well-formed prepared-entry requirement"),
        entry,
    }
}

/// Returns the scalar image the live-extent member carries.
///
/// `columns` is the baked neighbour the interpreter must not use: the bound
/// `RoutedExtentParameter` is the contributor-loop width.
#[must_use]
pub fn live_extent_image(symbol: &str) -> ScalarImage {
    ScalarImage {
        entries: vec![ScalarEntry {
            symbol: symbol.to_owned(),
            read_transport: 0,
            write_transport: 1,
            rows: u32::try_from(ROWS).expect("a small fixture extent"),
            columns: u32::try_from(COLUMNS).expect("a baked neighbour the live route must ignore"),
            scale_bits: SCALE_BITS,
            bias_bits: BIAS_BITS,
        }],
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
    /// The same one stage, under a guard that never holds.
    ///
    /// A separate plan rather than a flag on [`Self::Fused`] because it *is* a
    /// separate kernel program: the applicability guard is part of what the
    /// producer packages, and the artifact layer refuses two variants that agree
    /// on both the program and the guard.
    FusedInapplicable,
    /// The same one stage, under a guard that reads a fact the caller binds.
    ///
    /// True whenever the input is bound at all, so it changes nothing about a
    /// route in the ordinary case. What it makes reachable is the case where the
    /// caller binds *nothing*: the guard is then unanswerable rather than false,
    /// which is a different thing for the loader to do — and the distinction is
    /// only observable against a guard that is not a constant.
    FusedExtentGuarded,
    /// The same one stage, under `SELECTION_PROPERTY_KEY ≡ 0 (mod 16)`.
    ///
    /// Ranked ahead of [`Self::Fused`] in a portfolio, it is selected exactly
    /// when the caller binds an aligned selection property, so one artifact
    /// identity selects different routing ranks on different routes.
    FusedPropertyGuarded,
    /// A pointwise stage and a reduction stage over an explicit intermediate.
    Materialized,
    /// One `LiveRowMajor` stage whose inner extent is a payload operand.
    ///
    /// Retained as the exact former wrong-positive subject: over this fixture's
    /// fixed `[2, 3]` semantic graph it now refuses at artifact construction,
    /// which is what the association test asserts through [`try_assemble`].
    LiveExtent,
    /// One strict unseeded contraction whose contributor extent is live.
    ///
    /// Retained for the same refusal evidence in the contraction spelling.
    LiveContraction,
}

impl PackagedPlan {
    /// Returns the compilation subject a payload for this plan describes.
    ///
    /// Deliberately independent of the emitted object, so a perturbation of the
    /// carried bytes does not change artifact identity — and distinct per plan,
    /// so two members of one portfolio declaring the same backend family still
    /// declare two payloads rather than colliding on one descriptor.
    fn source(self) -> Vec<u8> {
        match self {
            Self::Fused => b"fused-multiply-add-strict-serial-sum rows=2 columns=3".to_vec(),
            Self::FusedInapplicable => {
                b"fused-multiply-add-strict-serial-sum rows=2 columns=3 inapplicable".to_vec()
            }
            Self::FusedExtentGuarded => {
                b"fused-multiply-add-strict-serial-sum rows=2 columns=3 extent-guarded".to_vec()
            }
            Self::FusedPropertyGuarded => {
                b"fused-multiply-add-strict-serial-sum rows=2 columns=3 property-guarded".to_vec()
            }
            Self::Materialized => b"multiply-add then strict-serial-sum rows=2 columns=3".to_vec(),
            Self::LiveExtent => b"live-row-major e0; N is not in this subject".to_vec(),
            Self::LiveContraction => b"live-contraction e0; S is not in this subject".to_vec(),
        }
    }
}

/// Which applicability guard a fused plan packages.
///
/// The guard is part of the kernel program rather than something a variant
/// restates, so varying it is how this fixture produces two fused members the
/// artifact layer will accept side by side — and how it reaches the three
/// answers a guard can give: yes, no, and "the caller did not bind that".
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FusedGuard {
    /// A constant that always holds.
    AlwaysHolds,
    /// A constant that never holds.
    NeverHolds,
    /// `1 <= extent(input, 0)`, which needs the input's shape to be bound.
    NeedsBoundInput,
    /// The bound [`SELECTION_PROPERTY_KEY`] quantity is a multiple of 16.
    ///
    /// A fact-driven guard over a *target property* rather than an input
    /// extent: the fixture's interface is fixed, so the one per-route fact a
    /// caller may legitimately vary is a live-device property, and this is
    /// what lets one portfolio select different ranks on different routes.
    PropertyMultipleOfSixteen,
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
    /// The arithmetic type the delivered-realization record binds this entry to.
    ///
    /// # This varies what the artifact *records*, not what its kernels compute
    ///
    /// The delivered-realization record's entry bindings are the only place an
    /// artifact says which arithmetic governs an entry — an entry's own
    /// numerical realization carries behaviour dimensions and no dtype — and
    /// `validate_against_artifact` deliberately never reads the subject's
    /// arithmetic type, which `tiler-artifact`'s own fixture documents as the
    /// reason the subject is a producer parameter there too. So an entry
    /// packaging this file's `f32` kernel under a `bf16` subject builds,
    /// encodes, and decodes.
    ///
    /// That is exactly the right fixture for what this suite tests and exactly
    /// the wrong one for what it does not. The loader reads the recorded
    /// association and has no second dtype authority, so varying it varies the
    /// whole of the loader's input. It does **not** make the carried payload a
    /// BF16 program: the object is still this fixture's `f32` scalar image, and
    /// no case here claims BF16 *executes*. `docs/dtype-support.md` records BF16
    /// backend execution as absent, and a fixture implying otherwise would
    /// contradict it.
    ///
    /// **Per entry rather than per variant**, because that is what makes a check
    /// walking only the first entry visible: a two-entry variant whose entries
    /// record two widths refuses on the second, and a loop that stopped at the
    /// first would route.
    pub arithmetic: ArithmeticType,
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
    /// Additional live-device requirements of the route.
    pub route_requirements: Vec<RouteRequirement>,
    /// Deferred prepared-entry predicates of the variant.
    pub deferred_predicates: Vec<DeferredPredicateSpec>,
    /// The packaged entries, in the plan's own stage order.
    ///
    /// Each carries the arithmetic the delivered-realization record binds it to;
    /// see [`FixtureEntry::arithmetic`].
    pub entries: Vec<FixtureEntry>,
    /// The provider-versioned target environment the payload declares, if any.
    pub environment: Option<TargetEnvironmentDeclaration>,
    /// Whether the member publishes the ADR 0013 `Plan` claim at delivery 0.
    ///
    /// Publishing requires [`Self::environment`] and a carried object; the
    /// fixture derives the IR witness from the packaged program and mints the
    /// receipt through [`ScalarPayloadDeterminismVerifier`], so the claim is
    /// proof-bound exactly as a production producer's would be.
    pub claim_plan: bool,
}

/// Returns the governed entry key of one packaged backend entry.
#[must_use]
pub fn entry_key(name: &[u8]) -> BackendEntryKey {
    BackendEntryKey::from_bytes(name).expect("a governed entry key")
}

impl FixtureSpec {
    /// Returns a member of the **Metal** backend family, for selection only.
    ///
    /// The second backend family a portfolio needs, spelled with Metal's real
    /// governed keys rather than a second invented one, because the case the
    /// loader has to get right is a genuinely heterogeneous artifact rather than
    /// two flavours of this fixture.
    ///
    /// **The carried object is not a metallib and does not claim to be.** It is
    /// this fixture's own scalar image, and nothing ever decodes it: a variant of
    /// another family is filtered before its guard, and a route that selects it
    /// is asserted at the loader rather than dispatched. That is exactly the
    /// division ADR 0090 item 8 draws — payload bytes are the backend's to
    /// validate, and the loader is not permitted to interpret them — so a
    /// selection fixture that needed a real Metal toolchain would be testing the
    /// toolchain.
    ///
    /// It declares no route requirement, so a host that *does* state Metal
    /// reaches selection rather than the foreign-owner refusal, which would
    /// answer a different question first.
    #[must_use]
    pub fn metal(plan: PackagedPlan) -> Self {
        Self {
            plan,
            backend: metal_backend(),
            representation: metal_representation(),
            payload_profile: profile(),
            route_requirements: Vec::new(),
            ..Self::for_plan(plan)
        }
    }

    /// Returns the scalar-host member of one packaged plan.
    ///
    /// The entry keys, symbols, transports, deferred predicates, and carried
    /// image a plan implies, in one place, so a portfolio member states only
    /// what it varies.
    /// Returns the same member with every entry recorded at one arithmetic.
    ///
    /// See [`FixtureEntry::arithmetic`] for what this does and does not vary.
    #[must_use]
    pub fn recording(mut self, arithmetic: ArithmeticType) -> Self {
        for entry in &mut self.entries {
            entry.arithmetic = arithmetic;
        }
        self
    }

    /// Returns the same member with each entry recorded at its own arithmetic.
    ///
    /// # Panics
    ///
    /// Panics when the widths do not match the member's entry count. A caller
    /// stating fewer than a plan packages would leave an entry at whatever it
    /// had, and the case would then pass for a reason it did not choose.
    #[must_use]
    pub fn recording_each(mut self, arithmetic: &[ArithmeticType]) -> Self {
        assert_eq!(
            arithmetic.len(),
            self.entries.len(),
            "state one width per packaged entry",
        );
        for (entry, width) in self.entries.iter_mut().zip(arithmetic) {
            entry.arithmetic = *width;
        }
        self
    }

    #[must_use]
    pub fn for_plan(plan: PackagedPlan) -> Self {
        match plan {
            PackagedPlan::Fused => Self::default(),
            // The same entries and the same carried image as the fused member:
            // only the packaged guard differs, which is what makes a portfolio
            // holding both a test of selection rather than of two backends.
            PackagedPlan::FusedInapplicable
            | PackagedPlan::FusedExtentGuarded
            | PackagedPlan::FusedPropertyGuarded => Self {
                plan,
                ..Self::default()
            },
            PackagedPlan::Materialized => Self::materialized(),
            PackagedPlan::LiveExtent => Self::live_extent(),
            PackagedPlan::LiveContraction => Self::live_contraction(),
        }
    }

    /// Returns the live-extent member: one payload operand, no baked N.
    #[must_use]
    pub fn live_extent() -> Self {
        Self {
            code: encode(&live_extent_image(LIVE_EXTENT_SYMBOL)),
            plan: PackagedPlan::LiveExtent,
            route_requirements: Vec::new(),
            deferred_predicates: Vec::new(),
            entries: vec![FixtureEntry {
                key: entry_key(b"live-row-major"),
                symbol: LIVE_EXTENT_SYMBOL.to_owned(),
                transports: vec![0, 1, 2],
                arithmetic: ArithmeticType::F32,
            }],
            ..Self::default()
        }
    }

    /// Returns the live strict-contraction member used for preflight routing.
    #[must_use]
    pub fn live_contraction() -> Self {
        let mut image = live_extent_image(LIVE_CONTRACTION_SYMBOL);
        image.entries[0].rows = 1;
        image.entries[0].write_transport = 2;
        Self {
            code: encode(&image),
            plan: PackagedPlan::LiveContraction,
            route_requirements: Vec::new(),
            deferred_predicates: Vec::new(),
            entries: vec![FixtureEntry {
                key: entry_key(b"live-contraction"),
                symbol: LIVE_CONTRACTION_SYMBOL.to_owned(),
                // Three buffers followed by the one live extent operand.
                transports: vec![0, 1, 2, 3],
                arithmetic: ArithmeticType::F32,
            }],
            ..Self::default()
        }
    }

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
                    arithmetic: ArithmeticType::F32,
                },
                FixtureEntry {
                    key: entry_key(b"scalar-host-reduction"),
                    symbol: REDUCTION_SYMBOL.to_owned(),
                    transports: vec![0, 1],
                    arithmetic: ArithmeticType::F32,
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
            route_requirements: vec![host_arithmetic_requirement(backend())],
            deferred_predicates: vec![prepared_predicate(0)],
            entries: vec![FixtureEntry {
                key: entry_key(b"scalar-host-fused"),
                symbol: ENTRY_SYMBOL.to_owned(),
                transports: vec![1, 0],
                arithmetic: ArithmeticType::F32,
            }],
            environment: None,
            claim_plan: false,
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

/// Assembles, verifies, and encodes one single-variant fixture artifact.
///
/// The one-member case of [`assemble_portfolio`], delegating rather than
/// repeating it: a portfolio that assembled its members differently from the way
/// this suite's other cases are built would make every comparison between them
/// an argument about the fixture.
///
/// # Panics
///
/// Panics when the artifact does not verify or does not encode. A fixture that
/// cannot be built is a defect in this file rather than a case under test.
#[must_use]
pub fn assemble(spec: &FixtureSpec) -> Fixture {
    assemble_portfolio(std::slice::from_ref(spec))
}

/// Assembles one artifact packaging every member as its own variant and payload.
///
/// Routing rank is position: `members[0]` is the producer's first choice under
/// [`RoutingPolicy::StablePriority`](tiler_artifact::program::RoutingPolicy).
/// Each member carries its own payload, so a portfolio can declare more than one
/// backend family, more than one executable representation, and more than one
/// payload compatibility profile.
///
/// # Panics
///
/// Panics when the artifact does not verify or does not encode, and when the
/// member list is empty — a portfolio with no variants is refused by
/// whole-artifact verification, and asserting it here names the fixture defect
/// rather than the diagnostic it produces.
#[must_use]
pub fn assemble_portfolio(members: &[FixtureSpec]) -> Fixture {
    assert!(
        !members.is_empty(),
        "a portfolio packages at least one variant",
    );
    assemble_portfolio_over(members, &semantic_program())
}

/// Assembles one portfolio over a caller-supplied semantic program.
///
/// The ordinary fixture graph is enough for route tests. Retained-shape
/// preflight needs a program that carries a non-empty environment with a
/// fixed interface, which this path admits without changing every other case.
#[must_use]
pub fn assemble_portfolio_over(members: &[FixtureSpec], semantic: &SemanticProgram) -> Fixture {
    try_assemble_portfolio_over(members, semantic)
        .expect("the fixture variant packages the bound plan")
}

/// [`assemble`], surfacing the artifact layer's variant refusal.
///
/// # Errors
///
/// Returns the exact [`ArtifactBuildError`] `push_variant` refused with — the
/// association fail-close's runtime-side evidence path.
pub fn try_assemble(spec: &FixtureSpec) -> Result<Fixture, ArtifactBuildError> {
    try_assemble_portfolio_over(std::slice::from_ref(spec), &semantic_program())
}

/// [`assemble_portfolio_over`], surfacing the artifact layer's variant refusal.
///
/// Every *other* fixture obligation still panics: a payload, claim, or record
/// this file cannot even declare is a fixture defect, while a refused variant
/// is the artifact layer's own judgment about the packaged plan — the thing a
/// refusal test asserts.
///
/// # Errors
///
/// Returns the exact [`ArtifactBuildError`] `push_variant` refused with.
pub fn try_assemble_portfolio_over(
    members: &[FixtureSpec],
    semantic: &SemanticProgram,
) -> Result<Fixture, ArtifactBuildError> {
    assert!(
        !members.is_empty(),
        "a portfolio packages at least one variant",
    );
    // One provider offering one capability, realized by any packaged plan. The
    // capability is what the semantic graph asks for; how many stages implement
    // it, and which backend emits them, are physical choices below it.
    let provider =
        ProviderIdentity::new("tiler-test", "scalar-host-serial-sum", 1).expect("a provider");
    let environment = CompilationEnvironment::new([provider.clone()], offered_physical())
        .expect("an environment");
    let mut draft = ArtifactProgramBuilder::new(semantic, environment).expect("an artifact draft");
    draft
        .select_lowering_provider(SelectedLoweringProvider {
            provider,
            capability: LoweringCapabilitySubject {
                family: CapabilityFamilyKey::new("index-access").expect("a capability family"),
                operation: OpKey::new("tiler", "strict-serial-sum-f32", 1)
                    .expect("an operation key"),
            },
            capability_revision: 1,
        })
        .expect("the selected provider was offered");

    for spec in members {
        push_member(&mut draft, semantic, spec)?;
    }

    declare_realization(&mut draft, members);

    let artifact = draft.build().expect("the fixture artifact verifies");
    let bytes = artifact.encode().expect("the fixture artifact encodes");
    let expected =
        RecordedArtifactProgramIdentity::from_bytes(artifact.canonical_identity().as_bytes())
            .expect("the producing side records its own identity");
    Ok(Fixture { bytes, expected })
}

/// Declares one member's payload and its variant on a portfolio draft.
/// Declares the numerical realization the fixture artifact delivers.
///
/// Every executable artifact carries one, so a consumer-side fixture that could
/// not build one would be evidence the boundary is unusable from outside the
/// producing crates. It is assembled here entirely through
/// `tiler_artifact::program` re-exports — no `tiler_ir` path appears in it —
/// which is what proves the record is reachable from a consumer whose dependency
/// closure ADR 0081 item 2 fixes at `[tiler-artifact]`.
///
/// The eleven resolutions are derived from the packaged kernels' own scheduled
/// realization rather than restated, so a fixture that changed its contract
/// cannot leave a record describing the old one.
///
/// One subject per distinct arithmetic the members record, and every packaged
/// entry bound to its own member's — so a portfolio mixing widths states which
/// entries each subject governs rather than one answer for all of them.
///
/// # Panics
///
/// Panics when the record does not build. Every member's subject is governed and
/// every packaged entry is bound to exactly one, so a refusal is a defect in
/// this file rather than a case under test.
fn declare_realization(draft: &mut ArtifactProgramBuilder, members: &[FixtureSpec]) {
    let entry = EntryRealization::of(strict());
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
                // Reciprocal transform, the third dimension the scheduled
                // realization does not carry. Named as the remaining arm rather
                // than a wildcard over all eleven.
                _ => DimensionBehaviour::Transform(NumericalPermission::Forbidden),
            });
    }
    // The portfolio's single profile: `push_variant` refuses a second variant
    // declaring a different one, so the first member's is every member's.
    let profile = members[0].variant_profile.clone();
    let mut record = DeliveredRealizationBuilder::new(profile.clone());
    // One declaration per *distinct* arithmetic. `declare_scalar_arithmetic`
    // refuses a redeclared subject, so a portfolio whose members share a width
    // must declare it once and bind both members' entries to it.
    let mut declared: Vec<ArithmeticType> = Vec::new();
    for arithmetic in members
        .iter()
        .flat_map(|spec| spec.entries.iter().map(|entry| entry.arithmetic))
    {
        if declared.contains(&arithmetic) {
            continue;
        }
        declared.push(arithmetic);
        let subject = subject_identity(arithmetic);
        record
            .declare_scalar_arithmetic(subject.clone(), resolutions)
            .expect("the fixture contract");
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
                        ProvenanceIdentity::new(PROFILE_KEY, 1),
                        ProvenanceIdentity::new("tiler.test.scalar-host.guarantee", 1),
                    ),
                },
            )
            .expect("the fixture obligation");
    }
    // The flat declared packaged-entry space: every member's entries, in the
    // order the members are pushed as variants.
    let mut entry = 0_u32;
    for spec in members {
        for packaged in &spec.entries {
            record
                .bind_entry(entry, &subject_identity(packaged.arithmetic))
                .expect("a packaged entry");
            entry += 1;
        }
    }
    draft
        .declare_realization(record.build().expect("the fixture record"))
        .expect("the fixture record agrees with the packaged portfolio");
}

/// Returns the governed scalar-arithmetic subject identity of one width.
///
/// The one place in this file that reaches `tiler_ir` to build a subject, and it
/// is kept out of [`declare_realization`] on purpose: that function's claim is
/// that a delivered-realization record is assemblable from
/// `tiler_artifact::program` re-exports alone, which is what proves the record is
/// reachable from a consumer whose dependency closure ADR 0081 item 2 fixes at
/// `[tiler-artifact]`. A subject *identity* is one of those re-exports; only
/// minting a `bf16` one from the semantic catalog is not, because
/// `ResolvedValueType` deliberately does not travel with the artifact surface.
///
/// # Panics
///
/// Panics for an arithmetic type the governed catalog does not register, and for
/// `f16`/`f64`, which this fixture packages no program at.
fn subject_identity(arithmetic: ArithmeticType) -> ScalarArithmeticSubjectIdentity {
    match arithmetic {
        ArithmeticType::F32 => ScalarArithmeticSubject::f32().identity(),
        ArithmeticType::Bf16 => {
            ScalarArithmeticSubject::new(ArithmeticType::Bf16, Bf16::resolved_type())
                .expect("the governed bf16 arithmetic subject is registered")
                .identity()
        }
        // Named rather than a wildcard: a width this fixture grows a program for
        // must state its subject here instead of falling into another's.
        ArithmeticType::F16 | ArithmeticType::F64 => {
            panic!(
                "this fixture packages no {} program",
                arithmetic.canonical_type_key()
            )
        }
    }
}

fn push_member(
    draft: &mut ArtifactProgramBuilder,
    semantic: &SemanticProgram,
    spec: &FixtureSpec,
) -> Result<(), ArtifactBuildError> {
    let program = match spec.plan {
        PackagedPlan::Fused => fused_program(semantic, FusedGuard::AlwaysHolds),
        PackagedPlan::FusedInapplicable => fused_program(semantic, FusedGuard::NeverHolds),
        PackagedPlan::FusedExtentGuarded => fused_program(semantic, FusedGuard::NeedsBoundInput),
        PackagedPlan::FusedPropertyGuarded => {
            fused_program(semantic, FusedGuard::PropertyMultipleOfSixteen)
        }
        PackagedPlan::Materialized => materialized_program(semantic),
        PackagedPlan::LiveExtent => live_extent_program(semantic),
        PackagedPlan::LiveContraction => live_contraction_program(semantic),
    };
    let metadata = member_metadata(spec, "aarch64-apple-darwin");
    let payload = draft
        .push_carried_payload(
            spec.backend.clone(),
            spec.representation.clone(),
            SchemaVersion::new(1, 0),
            spec.payload_profile.clone(),
            // The only policy the vocabulary defines, so the fixture states it
            // rather than offering a knob with one position.
            ArtifactExecutionPolicy::NativeImage,
            spec.environment.clone(),
            PayloadContent {
                metadata: metadata.clone(),
                code: spec.code.clone(),
            },
        )
        .expect("the fixture payload was accepted");

    let buffer_bindings = match spec.plan {
        PackagedPlan::LiveContraction => 3,
        PackagedPlan::Fused
        | PackagedPlan::FusedInapplicable
        | PackagedPlan::FusedExtentGuarded
        | PackagedPlan::FusedPropertyGuarded
        | PackagedPlan::Materialized
        | PackagedPlan::LiveExtent => 2,
    };
    let variant = draft.push_variant(
        &program,
        VariantSpec {
            target_profile: spec.variant_profile.clone(),
            feasibility_rules: FeasibilityRuleSetRef {
                key: FeasibilityRuleSetKey::new("tiler.test.scalar-host.feasibility")
                    .expect("a governed rule-set key"),
                revision: 1,
            },
            // One selected region per packaged stage: the ordinary shape, and
            // the one that exercises multiplicity for the multi-stage fixtures
            // rather than only for the single-stage ones.
            selected_physical_implementations: physical_run(spec.entries.len()),
            deferred_predicates: spec.deferred_predicates.clone(),
            entries: spec
                .entries
                .iter()
                .map(|entry| EntrySpec {
                    bindings: vec![
                        BindingSpec {
                            kind: BindingKind::Buffer,
                        };
                        buffer_bindings
                    ],
                    launch: LaunchSpec {
                        zero_work_skips_dispatch: true,
                        preconditions: Vec::new(),
                    },
                    implementation: BackendEntryRef {
                        payloads: vec![payload],
                        entry_key: entry.key.clone(),
                    },
                })
                .collect(),
        },
    )?;

    for requirement in &spec.route_requirements {
        draft
            .require_route(variant, requirement.clone())
            .expect("the fixture route requirement was accepted");
    }

    if spec.claim_plan {
        // The proof-bound ADR 0013 join, exactly as a production producer would
        // run it: the IR witness over the packaged program, a receipt minted by
        // the backend's installed verifier against the exact descriptor,
        // object bytes, and validated declaration, then the transactional
        // publication of the one claimed cell.
        let declaration = spec
            .environment
            .as_ref()
            .expect("a claiming member declares a target environment");
        let witness =
            verify_plan_determinism(&program).expect("the fixture plan is plan deterministic");
        let descriptor = BackendPayloadDescriptor {
            backend: spec.backend.clone(),
            representation: spec.representation.clone(),
            payload_schema: SchemaVersion::new(1, 0),
            digest: metadata
                .identity()
                .expect("the fixture metadata has an identity"),
            compatibility: spec.payload_profile.clone(),
            execution_policy: ArtifactExecutionPolicy::NativeImage,
            environment: Some(declaration.clone()),
        };
        // Validated against the *producer's* registration, derived from its own
        // declaration: a producer that declares a revision or class the
        // consumer's adapter does not register still builds a coherent claimed
        // artifact — the mismatch is then the runtime filter's to name.
        let validated = declaration
            .validate(&producer_schema(declaration))
            .expect("the fixture declaration is its own schema's canonical spelling");
        let receipt = ScalarPayloadDeterminismVerifier
            .verify(&witness, &descriptor, &spec.code, &validated)
            .expect("the scalar host's translation makes no run-dependent choice");
        draft
            .publish_plan(variant, 0, &witness, &[receipt])
            .expect("the proof-bound claim publishes");
    }
    Ok(())
}

/// The scalar host's installed payload plan-determinism verifier.
///
/// Its whole translation is a byte-for-byte image decode on the calling
/// thread, so "no run-dependent translation choice" is this backend's honest
/// judgment rather than a fixture shortcut; the receipt's bindings are minted
/// by the artifact layer from the exact inputs either way.
pub struct ScalarPayloadDeterminismVerifier;

impl PayloadPlanDeterminismVerifier for ScalarPayloadDeterminismVerifier {
    fn assess(
        &self,
        _witness: &PlanDeterminismWitness<'_>,
        _descriptor: &BackendPayloadDescriptor,
        _object_bytes: &[u8],
        _declaration: &ValidatedTargetEnvironmentDeclaration,
    ) -> Result<(), PayloadPlanDeterminismRefusal> {
        Ok(())
    }
}

/// The scalar host's one governed environment provider identity.
#[must_use]
pub fn environment_provider() -> ProviderIdentity {
    ProviderIdentity::new("tiler.test", "scalar-host-environment", 1)
        .expect("a governed provider identity")
}

/// The canonical spelling of the class this process actually is.
pub const ENVIRONMENT_DESCRIPTOR: &[u8] = b"process-arithmetic-v1";

/// The canonical spelling of a second admitted class this process is not.
pub const ALTERED_ENVIRONMENT_DESCRIPTOR: &[u8] = b"process-arithmetic-v1-altered";

/// The scalar host's declared target environment.
#[must_use]
pub fn environment_declaration() -> TargetEnvironmentDeclaration {
    declaration_of(ENVIRONMENT_DESCRIPTOR)
}

/// The same declaration over the second admitted class.
///
/// The accepted provider-descriptor perturbation: one descriptor field moved,
/// everything else held.
#[must_use]
pub fn altered_environment_declaration() -> TargetEnvironmentDeclaration {
    declaration_of(ALTERED_ENVIRONMENT_DESCRIPTOR)
}

fn declaration_of(descriptor: &[u8]) -> TargetEnvironmentDeclaration {
    TargetEnvironmentDeclaration::new(
        environment_provider(),
        SchemaVersion::new(1, 0),
        TargetEnvironmentDescriptor::new(descriptor).expect("a bounded fixture descriptor"),
    )
    .expect("a nonzero schema major")
}

/// The same declaration under the next provider revision, which nothing registers.
#[must_use]
pub fn revised_provider_declaration() -> TargetEnvironmentDeclaration {
    TargetEnvironmentDeclaration::new(
        ProviderIdentity::new("tiler.test", "scalar-host-environment", 2)
            .expect("a governed provider identity"),
        SchemaVersion::new(1, 0),
        TargetEnvironmentDescriptor::new(ENVIRONMENT_DESCRIPTOR)
            .expect("a bounded fixture descriptor"),
    )
    .expect("a nonzero schema major")
}

/// One test provider's registered target-environment descriptor schema.
///
/// The scalar host's whole authority claim is byte equality with one exact
/// canonical spelling: the process *is* the execution-environment class its
/// schema names, which is the strongest claim a single-process interpreter can
/// honestly make and exactly the shape the ADR 0013 contract requires a
/// provider to prove before registering positive support.
///
/// It is declared here rather than beside the adapter because both of its
/// constructors are here — [`producer_schema`] below is private to this module
/// and cannot reach out of it, so a declaration in `adapter.rs` is a back-edge
/// this module's non-owning consumers cannot resolve. See this file's
/// "path-shared" module note for what that costs and what now checks it.
#[derive(Clone, Debug)]
pub struct ScalarEnvironmentSchema {
    /// Provider identity, with its exact nonzero revision.
    pub provider: ProviderIdentity,
    /// Exact schema version.
    pub schema: SchemaVersion,
    /// The canonical descriptor spellings this schema admits, one per class.
    ///
    /// Each admitted value is the exactly-one canonical spelling of its own
    /// environment class; a second class is a second member of this set, never
    /// a second spelling of the first. That is what makes a declared-versus-
    /// observed class mismatch representable while validation still accepts
    /// exactly one byte spelling per class.
    pub admitted: Vec<Vec<u8>>,
}

impl TargetEnvironmentDescriptorSchema for ScalarEnvironmentSchema {
    fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    fn schema_version(&self) -> SchemaVersion {
        self.schema
    }

    fn validate_canonical_descriptor(
        &self,
        descriptor: &[u8],
    ) -> Result<(), TargetEnvironmentReasonCode> {
        if self.admitted.iter().any(|value| value == descriptor) {
            Ok(())
        } else {
            Err(
                TargetEnvironmentReasonCode::new("scalar-host.not-the-canonical-spelling")
                    .expect("a literal governed reason code"),
            )
        }
    }
}

/// The producer-side registration one declaration was validated under.
///
/// Derived from the declaration itself: the producer's authority is its own
/// registration, and whether the *consumer's* independently selected adapter
/// registers the same provider, revision, schema, and class is exactly the
/// question the runtime filter answers.
fn producer_schema(declaration: &TargetEnvironmentDeclaration) -> ScalarEnvironmentSchema {
    ScalarEnvironmentSchema {
        provider: declaration.provider().clone(),
        schema: declaration.descriptor_schema(),
        admitted: vec![declaration.descriptor().as_bytes().to_vec()],
    }
}

/// The scalar host's registered descriptor schema, for the adapter to expose.
///
/// Two admitted classes, each with exactly one canonical spelling, so a
/// declared-versus-observed class mismatch is representable.
#[must_use]
pub fn environment_schema() -> ScalarEnvironmentSchema {
    ScalarEnvironmentSchema {
        provider: environment_provider(),
        schema: SchemaVersion::new(1, 0),
        admitted: vec![
            ENVIRONMENT_DESCRIPTOR.to_vec(),
            ALTERED_ENVIRONMENT_DESCRIPTOR.to_vec(),
        ],
    }
}

/// Builds one member's payload compilation subject.
///
/// The retained source describes what was translated, and it is a function of
/// the packaged plan alone — deliberately independent of `code`. Every payload
/// perturbation therefore changes the carried object and *not* the compilation
/// subject, which is what makes two artifacts with different bytes share one
/// canonical identity — the asymmetry ADR 0090 item 8 rests on. `target` is
/// the one provenance field the two-delivery fixture varies, because two
/// delivery positions are one plan compiled twice for two consumer targets.
fn member_metadata(spec: &FixtureSpec, target: &str) -> PayloadMetadata {
    PayloadMetadata {
        source_representation: RepresentationKey::new(SOURCE_REPRESENTATION_KEY)
            .expect("a governed source representation"),
        source: spec.plan.source(),
        provenance: PayloadProvenance {
            toolchain: "tiler.test.scalar-image-translator".to_owned(),
            target: target.to_owned(),
            family: spec.backend.as_str().to_owned(),
            language: "tiler.kernel-ir.v4".to_owned(),
            // No SDK and no platform deployment minimum on this backend's
            // target, stated rather than approximated.
            platform: PayloadPlatform::Unversioned,
            components: vec![ToolComponent {
                role: "translator".to_owned(),
                version: "1".to_owned(),
            }],
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
    }
}

/// Assembles one claimed member realized at **two** delivery positions.
///
/// One plan, one kernel program, two compiled objects: the second payload's
/// compilation subject differs only in its provenance target, which is exactly
/// what a second consumer build target is. Both payloads declare the same
/// target environment and the member publishes the proof-bound `Plan` claim at
/// both positions, so the accepted delivery-position perturbation can hold the
/// envelope and rank fixed and move only the selected coordinate.
///
/// # Panics
///
/// Panics when the artifact does not verify, claim, or encode: a fixture that
/// cannot be built is a defect in this file rather than a case under test.
#[must_use]
pub fn assemble_two_delivery_claimed() -> Fixture {
    let semantic = semantic_program();
    let provider =
        ProviderIdentity::new("tiler-test", "scalar-host-serial-sum", 1).expect("a provider");
    let compilation = CompilationEnvironment::new([provider.clone()], offered_physical())
        .expect("an environment");
    let mut draft = ArtifactProgramBuilder::new(&semantic, compilation).expect("an artifact draft");
    draft
        .select_lowering_provider(SelectedLoweringProvider {
            provider,
            capability: LoweringCapabilitySubject {
                family: CapabilityFamilyKey::new("index-access").expect("a capability family"),
                operation: OpKey::new("tiler", "strict-serial-sum-f32", 1)
                    .expect("an operation key"),
            },
            capability_revision: 1,
        })
        .expect("the selected provider was offered");

    let spec = FixtureSpec {
        environment: Some(environment_declaration()),
        claim_plan: true,
        ..FixtureSpec::for_plan(PackagedPlan::Fused)
    };
    let program = fused_program(&semantic, FusedGuard::AlwaysHolds);
    let metadata = [
        member_metadata(&spec, "aarch64-apple-darwin"),
        member_metadata(&spec, "aarch64-apple-ios"),
    ];
    let mut payloads = Vec::new();
    for subject in &metadata {
        payloads.push(
            draft
                .push_carried_payload(
                    spec.backend.clone(),
                    spec.representation.clone(),
                    SchemaVersion::new(1, 0),
                    spec.payload_profile.clone(),
                    ArtifactExecutionPolicy::NativeImage,
                    spec.environment.clone(),
                    PayloadContent {
                        metadata: subject.clone(),
                        code: spec.code.clone(),
                    },
                )
                .expect("the delivery payload was accepted"),
        );
    }
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
                selected_physical_implementations: physical_run(1),
                deferred_predicates: Vec::new(),
                entries: vec![EntrySpec {
                    bindings: vec![
                        BindingSpec {
                            kind: BindingKind::Buffer,
                        };
                        2
                    ],
                    launch: LaunchSpec {
                        zero_work_skips_dispatch: true,
                        preconditions: Vec::new(),
                    },
                    implementation: BackendEntryRef {
                        payloads: payloads.clone(),
                        entry_key: spec.entries[0].key.clone(),
                    },
                }],
            },
        )
        .expect("the two-delivery variant packages the bound plan");

    let witness =
        verify_plan_determinism(&program).expect("the fixture plan is plan deterministic");
    let declaration = spec.environment.as_ref().expect("the fixture declares");
    let validated = declaration
        .validate(&producer_schema(declaration))
        .expect("the fixture declaration is its own schema's canonical spelling");
    for (delivery, subject) in metadata.iter().enumerate() {
        let descriptor = BackendPayloadDescriptor {
            backend: spec.backend.clone(),
            representation: spec.representation.clone(),
            payload_schema: SchemaVersion::new(1, 0),
            digest: subject
                .identity()
                .expect("the fixture metadata has an identity"),
            compatibility: spec.payload_profile.clone(),
            execution_policy: ArtifactExecutionPolicy::NativeImage,
            environment: Some(declaration.clone()),
        };
        let receipt = ScalarPayloadDeterminismVerifier
            .verify(&witness, &descriptor, &spec.code, &validated)
            .expect("the scalar host's translation makes no run-dependent choice");
        draft
            .publish_plan(variant, delivery, &witness, &[receipt])
            .expect("the proof-bound claim publishes at each delivery");
    }

    declare_realization(&mut draft, std::slice::from_ref(&spec));
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
        NumericalPermission::Forbidden,
        ApproximationEnvelope::Forbidden,
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

/// Builds the two-input graph bound by the live-contraction routing fixture.
#[must_use]
pub fn live_contraction_semantic_program() -> SemanticProgram {
    let shape = Shape::from_dims([1, 1]);
    let mut draft = SemanticProgramBuilder::try_standard().expect("the standard registry composes");
    let left = draft
        .input::<F32>(InputKey::new("left").expect("left key"), shape.clone())
        .expect("left input");
    let right = draft
        .input::<F32>(InputKey::new("right").expect("right key"), shape)
        .expect("right input");
    let result = F32Add::apply(&mut draft, left, right).expect("fixture occurrence");
    draft
        .output(OutputKey::new("result").expect("output key"), result)
        .expect("output");
    draft.build().expect("the program verifies")
}

/// Obtains proof-derived coverage for a range of canonical occurrences.
///
/// This fixture assembles its own physical programs, but it does not get to
/// invent the logical evidence those programs claim. Each record below comes
/// from the sealed IR path: derive the occurrence's subject, admit a lowering
/// authority, build a *candidate* index region here rather than asking the law
/// for its own answer, and submit the pair to the verifier — which mints a
/// receipt only when the candidate's canonical identity equals the law's.
///
/// Building the candidate independently is the point, and is also forced: the
/// law's realization is deliberately not public, because a caller that could
/// ask for the expected region and hand it straight back would turn the
/// verifier into a rubber stamp. This crate reaches `tiler-ir` and never
/// `tiler-compiler`, so the region below is this fixture's own claim about what
/// the operation means, checked against an authority it cannot influence.
fn checked_coverage(
    semantic: &SemanticProgram,
    occurrences: std::ops::Range<u32>,
) -> Vec<CoveredOccurrence> {
    let scalars = FrozenScalarRegistry::standard().expect("the standard scalar authority freezes");
    let laws = FrozenIndexRealizationLawRegistry::from_semantic(
        semantic.semantic_registry().clone(),
        scalars.clone(),
    )
    .expect("the standard scalar and semantic authorities cohere");
    let mut coverage: Vec<CoveredOccurrence> = semantic
        .operations()
        .map(|operation| checked_occurrence(semantic, &scalars, &laws, operation.id()))
        .collect();
    coverage.sort_unstable_by_key(CoveredOccurrence::occurrence);
    coverage.retain(|covered| occurrences.contains(&covered.occurrence().get()));
    coverage
}

fn checked_occurrence(
    semantic: &SemanticProgram,
    scalars: &FrozenScalarRegistry,
    laws: &FrozenIndexRealizationLawRegistry,
    operation: tiler_ir::semantic::OperationId,
) -> CoveredOccurrence {
    let subject = IndexRefinementSubject::derive(semantic, operation, strict_contract())
        .expect("every fixture operation derives a refinement subject");
    let (emitted, region): (Vec<ScalarOpKey>, VerifiedIndexRegion) =
        if subject.operation() == &constant_f32_op() {
            (
                vec![constant_f32_scalar_op()],
                constant_region(&subject, scalars),
            )
        } else if subject.operation() == &multiply_f32_op() {
            (
                vec![multiply_f32_scalar_op()],
                pointwise_region(&subject, scalars, multiply_f32_scalar_op()),
            )
        } else if subject.operation() == &add_f32_op() {
            (
                vec![add_f32_scalar_op()],
                pointwise_region(&subject, scalars, add_f32_scalar_op()),
            )
        } else {
            (
                vec![add_f32_scalar_op()],
                serial_sum_region(&subject, scalars),
            )
        };
    let authority = IndexRealizationAuthority::admit(
        semantic.semantic_registry(),
        scalars,
        subject.operation().clone(),
        subject.signature().clone(),
        &emitted,
    )
    .expect("the fixture's emission ceiling is admissible");
    let resolution = laws
        .resolve(&subject)
        .expect("the registered law resolves for this subject");
    match resolution
        .verify(&authority, &region)
        .expect("the fixture's candidate region realizes its operation")
    {
        IndexRefinementVerificationOutcome::Verified(receipt) => {
            CoveredOccurrence::from_receipt(&receipt)
        }
        IndexRefinementVerificationOutcome::Pending(_) => {
            panic!("the fixture's static regions retain no residual index-domain obligation")
        }
    }
}

/// The governed strict F32 contract the fixture's kernels realize.
fn strict_contract() -> NumericalContractIdentity {
    F32NumericalContractKey::new(
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
    .expect("the fixture contract vector is coherent")
    .into()
}

fn constant_region(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
) -> VerifiedIndexRegion {
    let [result] = subject.results() else {
        panic!("a constant has one result")
    };
    let bits = subject
        .attributes()
        .get(F32_CONSTANT_BITS_ATTRIBUTE)
        .expect("a constant carries its bits attribute")
        .clone();
    let attributes = ScalarAttributes::new(
        CanonicalValue::record([CanonicalField::new(F32_CONSTANT_BITS_ATTRIBUTE, bits)])
            .expect("the scalar attribute record composes"),
    )
    .expect("scalar attributes are a record");
    let mut region = IndexRegionBuilder::new(scalars.clone()).expect("an index region builder");
    let output = region
        .tensor(
            IndexTensorRole::Output,
            result.value_type().clone(),
            result.shape().clone(),
        )
        .expect("the constant's output tensor");
    let value = region
        .apply(constant_f32_scalar_op(), attributes, &[])
        .expect("the constant scalar applies")
        .get(0)
        .expect("one constant result");
    let write = region.write(output, &[], &[]).expect("the constant write");
    region.output(write, value).expect("the output root");
    region.build().expect("a verified constant region")
}

fn pointwise_region(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
    operation: ScalarOpKey,
) -> VerifiedIndexRegion {
    let [result] = subject.results() else {
        panic!("a binary pointwise operation has one result")
    };
    let mut region = IndexRegionBuilder::new(scalars.clone()).expect("an index region builder");
    let dimensions = result
        .shape()
        .extents()
        .iter()
        .copied()
        .map(|extent| {
            region
                .dimension(DomainRole::Parallel, extent)
                .expect("a parallel dimension")
        })
        .collect::<Vec<_>>();
    let coordinates = dimensions
        .iter()
        .copied()
        .map(|dimension| {
            region
                .dimension_expr(dimension)
                .expect("a dimension coordinate")
        })
        .collect::<Vec<_>>();
    let tensors = subject
        .inputs()
        .iter()
        .map(|input| {
            region
                .tensor(
                    IndexTensorRole::Input,
                    input.value_type().clone(),
                    input.shape().clone(),
                )
                .expect("a pointwise input tensor")
        })
        .collect::<Vec<_>>();
    let operands = subject
        .operands()
        .iter()
        .map(|position| {
            let input = &subject.inputs()[*position];
            if input.shape() == result.shape() {
                region
                    .read(tensors[*position], &dimensions, &coordinates)
                    .expect("an elementwise read")
            } else {
                region
                    .read(tensors[*position], &[], &[])
                    .expect("a rank-zero broadcast read")
            }
        })
        .collect::<Vec<_>>();
    let value = region
        .apply(operation, ScalarAttributes::empty(), &operands)
        .expect("the pointwise scalar applies")
        .get(0)
        .expect("one pointwise result");
    let output = region
        .tensor(
            IndexTensorRole::Output,
            result.value_type().clone(),
            result.shape().clone(),
        )
        .expect("the pointwise output tensor");
    let write = region
        .write(output, &dimensions, &coordinates)
        .expect("the pointwise write");
    region.output(write, value).expect("the output root");
    region.build().expect("a verified pointwise region")
}

fn serial_sum_region(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
) -> VerifiedIndexRegion {
    let ([input], [result]) = (subject.inputs(), subject.results()) else {
        panic!("a serial sum has one input and one result")
    };
    assert_eq!(input.shape(), &Shape::from_dims([ROWS, COLUMNS]));
    assert_eq!(result.shape(), &Shape::from_dims([ROWS]));
    let mut region = IndexRegionBuilder::new(scalars.clone()).expect("an index region builder");
    let row = region
        .dimension(DomainRole::Parallel, Extent::new(ROWS))
        .expect("the row dimension");
    let row_coordinate = region.dimension_expr(row).expect("the row coordinate");
    let zero = region
        .constant(IndexInteger::from_u64(0))
        .expect("the seed column");
    let input_tensor = region
        .tensor(
            IndexTensorRole::Input,
            input.value_type().clone(),
            input.shape().clone(),
        )
        .expect("the reduction input tensor");
    let seed = region
        .read(input_tensor, &[row], &[row_coordinate, zero])
        .expect("the first contributor");
    let tail = region
        .dimension(DomainRole::Reduction, Extent::new(COLUMNS - 1))
        .expect("the tail dimension");
    let tail_coordinate = region.dimension_expr(tail).expect("the tail coordinate");
    let one = IndexInteger::from_u64(1);
    let contributor_column = region
        .linear_combination(one.clone(), &[(one, tail_coordinate)])
        .expect("the tail contributor coordinate");
    let contributor = region
        .read(
            input_tensor,
            &[row, tail],
            &[row_coordinate, contributor_column],
        )
        .expect("a tail contributor");
    let total = region
        .reduce(&[tail], &[seed], &[contributor], |body| {
            let state = body.state(0).expect("one reduction state");
            let value = body.contributor(0).expect("one contributor");
            let accumulated = body
                .apply(
                    add_f32_scalar_op(),
                    ScalarAttributes::empty(),
                    &[state, value],
                )?
                .get(0)
                .expect("one accumulated result");
            body.yield_values(&[accumulated])
        })
        .expect("the serial reduction")
        .get(0)
        .expect("one reduction result");
    let output = region
        .tensor(
            IndexTensorRole::Output,
            result.value_type().clone(),
            result.shape().clone(),
        )
        .expect("the reduction output tensor");
    let write = region
        .write(output, &[row], &[row_coordinate])
        .expect("the reduction write");
    region.output(write, total).expect("the output root");
    region.build().expect("a verified serial-sum region")
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
            tensor: TensorRole::Input,
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
            tensor: TensorRole::Input,
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
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::FusedMultiplyAddSerialSum {
                scale_bits: SCALE_BITS,
                bias_bits: BIAS_BITS,
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_NAN,
                empty_identity_bits: 0,
                contraction: false,
            },
            numerical: strict(),
        })
        .expect("the scalar program");
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
///
/// `guard` is the applicability guard it packages. A guard that never holds is
/// what a producer packages for a plan it wants ranked but not chosen under the
/// bound facts, and it is the only way this fixture can put an *eligible*
/// variant in a portfolio that selection must nevertheless pass over.
fn live_row_major_kernel() -> VerifiedKernel {
    let inner = Axis::new(1);
    let mut region = ScheduledRegionBuilder::new(RegionId::new(40));
    region
        .iteration_shape(Shape::from_dims([ROWS]))
        .expect("rows");
    region
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LiveRowMajorSource { inner_axis: inner },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .expect("read");
    region
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LiveRowMajor,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .expect("write");
    for (witness, tensor) in [(0, TensorRole::Input), (1, TensorRole::Output)] {
        region
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 0 },
            })
            .expect("bounds");
    }
    region
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: ROWS },
        })
        .expect("ownership");
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).expect("input");
    let scale = expression.constant(SCALE_BITS).expect("scale");
    let product = expression.multiply(input, scale).expect("product");
    let bias = expression.constant(BIAS_BITS).expect("bias");
    let root = expression.add(product, bias).expect("root");
    region
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(expression.build(root).expect("expression")),
            numerical: strict(),
        })
        .expect("scalar");
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: ROWS,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: ROWS,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .expect("schedule");
    lower_scheduled_region(&region.build().expect("region")).expect("lowers")
}

fn live_extent_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    let kernel = live_row_major_kernel();
    let mut plan = KernelProgramBuilder::new(semantic).expect("a plan draft");
    let device = |capacity_bytes, ownership| AllocationSpec {
        capacity_bytes,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership,
    };
    let external = plan
        .push_allocation(device(ROWS * COLUMNS * 4, AllocationOwnership::External))
        .expect("external");
    let owned = plan
        .push_allocation(device(ROWS * COLUMNS * 4, AllocationOwnership::Program))
        .expect("owned");
    let value = |origin, role, shape| MaterializedValueSpec {
        origin,
        role,
        shape,
        storage_scalar: StorageScalar::F32,
        element_type: KernelType::F32,
        encoding: StorageEncoding::Unpacked,
        alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
    };
    let source = plan
        .push_value(
            value(
                MaterializedOrigin::ProgramInput { key: input_key() },
                ValueRole::Input,
                input_shape(),
            ),
            external,
        )
        .expect("source");
    let result = plan
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            owned,
        )
        .expect("result");
    let read = plan
        .push_view(
            source,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .expect("read");
    let write = plan
        .push_view(
            result,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .expect("write");
    let zero = plan
        .push_abi_root(AbiRoot::UnsignedLiteral(0))
        .expect("zero");
    let two = plan
        .push_abi_root(AbiRoot::UnsignedLiteral(2))
        .expect("two");
    let one = plan
        .push_abi_root(AbiRoot::UnsignedLiteral(1))
        .expect("one");
    let live_n = plan
        .push_abi_root(AbiRoot::InputExtent {
            key: input_key(),
            axis: Axis::new(1),
        })
        .expect("live N");
    let accessible = plan
        .push_abi_binary(AbiBinaryOp::CheckedMultiply, zero, live_n)
        .expect("accessible");
    let guard = plan
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("guard");
    plan.applicability_guard(guard).expect("applicability");
    declare_routing_commit(&mut plan);
    plan.push_stage(
        &kernel,
        &checked_coverage(semantic, 0..5),
        &[
            StageAccess {
                view: read,
                mode: StageAccessMode::Read,
                accessible_bytes: accessible,
            },
            StageAccess {
                view: write,
                mode: StageAccessMode::Write,
                accessible_bytes: accessible,
            },
        ],
        StageLaunch {
            grid_threads: two,
            threads_per_workgroup: one,
        },
    )
    .expect("the live-extent stage binds");
    plan.push_output(
        OutputKey::new("result").expect("a valid output key"),
        result,
    )
    .expect("the program output");
    plan.build().expect("the live-extent plan verifies")
}

fn live_contraction_kernel() -> VerifiedKernel {
    let operand = Shape::from_dims([1]);
    let output = Shape::from_dims([1, 1]);
    let contracted = Shape::from_dims([]);
    let owner = OwnershipWitnessId::new(0);
    let mut region = ScheduledRegionBuilder::new(RegionId::new(41));
    region
        .iteration_shape(output.clone())
        .expect("iteration shape");
    for (ordinal, free) in [0_u32, 1].into_iter().enumerate() {
        let witness = u32::try_from(ordinal).expect("two inputs");
        let tensor = TensorRole::Input;
        region
            .push_access(Access {
                tensor,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ContractionOperand {
                    operand_shape: operand.clone(),
                    output_shape: output.clone(),
                    contracted_shape: contracted.clone(),
                    sources: vec![ContractionAxisSource::Output { position: free }],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
            })
            .expect("operand access");
        region
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 0 },
            })
            .expect("live bounds");
    }
    region
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(owner),
        })
        .expect("output access");
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 1 },
        })
        .expect("output bounds");
    region
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 1 },
        })
        .expect("ownership");
    region
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted,
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_NAN,
            },
            numerical: strict(),
        })
        .expect("strict contraction");
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: 1,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: owner,
            reduction: ReductionTopology::LiveContraction {
                live_access: AccessOrdinal::FIRST,
                live_axis: Axis::new(1),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            launch: LaunchPlan {
                grid_threads: 1,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .expect("schedule");
    lower_scheduled_region(&region.build().expect("region")).expect("kernel")
}

fn live_contraction_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    let kernel = live_contraction_kernel();
    let mut plan = KernelProgramBuilder::new(semantic).expect("program");
    let allocation = |ownership| AllocationSpec {
        capacity_bytes: 4,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership,
    };
    let left_allocation = plan
        .push_allocation(allocation(AllocationOwnership::External))
        .expect("left allocation");
    let right_allocation = plan
        .push_allocation(allocation(AllocationOwnership::External))
        .expect("right allocation");
    let output_allocation = plan
        .push_allocation(allocation(AllocationOwnership::Program))
        .expect("output allocation");
    let value = |origin, role| MaterializedValueSpec {
        origin,
        role,
        shape: Shape::from_dims([1, 1]),
        storage_scalar: StorageScalar::F32,
        element_type: KernelType::F32,
        encoding: StorageEncoding::Unpacked,
        alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
    };
    let left = plan
        .push_value(
            value(
                MaterializedOrigin::ProgramInput {
                    key: InputKey::new("left").expect("left key"),
                },
                ValueRole::Input,
            ),
            left_allocation,
        )
        .expect("left");
    let right = plan
        .push_value(
            value(
                MaterializedOrigin::ProgramInput {
                    key: InputKey::new("right").expect("right key"),
                },
                ValueRole::Input,
            ),
            right_allocation,
        )
        .expect("right");
    let output = plan
        .push_value(
            value(MaterializedOrigin::Internal, ValueRole::Output),
            output_allocation,
        )
        .expect("output");
    let left_view = plan
        .push_view(
            left,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .expect("left view");
    let right_view = plan
        .push_view(
            right,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .expect("right view");
    let output_view = plan.push_whole_view(output).expect("output view");
    let zero = plan
        .push_abi_root(AbiRoot::UnsignedLiteral(0))
        .expect("zero");
    let four = plan
        .push_abi_root(AbiRoot::UnsignedLiteral(4))
        .expect("four");
    let one = plan
        .push_abi_root(AbiRoot::UnsignedLiteral(1))
        .expect("one");
    let guard = plan
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("authored guard");
    plan.applicability_guard(guard).expect("guard");
    declare_routing_commit(&mut plan);
    plan.push_stage(
        &kernel,
        &checked_coverage(semantic, 0..1),
        &[
            StageAccess {
                view: left_view,
                mode: StageAccessMode::Read,
                accessible_bytes: zero,
            },
            StageAccess {
                view: right_view,
                mode: StageAccessMode::Read,
                accessible_bytes: zero,
            },
            StageAccess {
                view: output_view,
                mode: StageAccessMode::Write,
                accessible_bytes: four,
            },
        ],
        StageLaunch {
            grid_threads: one,
            threads_per_workgroup: one,
        },
    )
    .expect("stage");
    plan.push_output(OutputKey::new("result").expect("key"), output)
        .expect("output");
    plan.build().expect("verified live contraction")
}

#[must_use]
pub fn fused_program(semantic: &SemanticProgram, guard: FusedGuard) -> VerifiedKernelProgram {
    let kernel = fused_kernel();
    let mut plan = KernelProgramBuilder::new(semantic).expect("a plan draft");
    let external = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: ROWS * COLUMNS * 4,
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .expect("the external allocation");
    let owned = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: ROWS * 4,
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
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
                alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
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
                alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
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
    let guard = match guard {
        FusedGuard::AlwaysHolds => plan
            .push_abi_root(AbiRoot::BooleanLiteral(true))
            .expect("the guard predicate"),
        FusedGuard::NeverHolds => plan
            .push_abi_root(AbiRoot::BooleanLiteral(false))
            .expect("the guard predicate"),
        // `1 <= extent(input, 0)`. True for every shape this fixture declares,
        // so it selects exactly as a constant `true` does — and *unanswerable*
        // when the caller binds no input, which a constant can never be.
        FusedGuard::NeedsBoundInput => {
            let one = plan
                .push_abi_root(AbiRoot::UnsignedLiteral(1))
                .expect("an abi literal");
            let rows = plan
                .push_abi_root(AbiRoot::InputExtent {
                    key: input_key(),
                    axis: Axis::new(0),
                })
                .expect("the input extent");
            plan.push_abi_binary(AbiBinaryOp::LessOrEqual, one, rows)
                .expect("the guard predicate")
        }
        FusedGuard::PropertyMultipleOfSixteen => {
            let sixteen = plan
                .push_abi_root(AbiRoot::UnsignedLiteral(16))
                .expect("an abi literal");
            let property = plan
                .push_abi_root(AbiRoot::TargetProperty {
                    key: TargetPropertyKey::new(SELECTION_PROPERTY_KEY)
                        .expect("a governed property key"),
                    phase: AvailabilityPhase::LiveDevicePreflight,
                })
                .expect("the selection property");
            plan.push_abi_binary(AbiBinaryOp::IsMultipleOf, property, sixteen)
                .expect("the guard predicate")
        }
    };
    plan.applicability_guard(guard)
        .expect("the applicability guard");
    declare_routing_commit(&mut plan);

    plan.push_stage(
        &kernel,
        &checked_coverage(semantic, 0..5),
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
        .input(AccessOrdinal::FIRST)
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
            tensor: TensorRole::Input,
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
            tensor: TensorRole::Input,
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
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(scale_bias_expression()),
            numerical: strict(),
        })
        .expect("the scalar program");
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
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_NAN,
                empty_identity_bits: 0,
            },
            numerical: strict(),
        })
        .expect("the scalar program");
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
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .expect("the external allocation");
    let scratch = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: ROWS * COLUMNS * 4,
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::Program,
        })
        .expect("the scratch allocation");
    let owned = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: ROWS * 4,
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
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
                alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
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
                alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
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
                alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
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
            &checked_coverage(semantic, 0..4),
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
            &checked_coverage(semantic, 4..5),
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
