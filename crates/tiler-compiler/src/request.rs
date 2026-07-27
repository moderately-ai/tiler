use std::error::Error;
use std::fmt;
use std::sync::OnceLock;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::index::FrozenScalarRegistry;
use tiler_ir::semantic::{
    CanonicalIntegerWidth, CanonicalValueView, F32, F32_CONSTANT_BITS_ATTRIBUTE, InputKey, OpKey,
    OutputKey, ProviderIdentity, REDUCTION_AXES_ATTRIBUTE, SemanticIdentity, SemanticProgram,
    TypeKey, ValueId, add_f32_op, constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
};
use tiler_ir::shape::{Axis, Shape};

// The numerical-realization vocabulary is target-neutral and owned by the shared
// IR (ADR 0070); the compiler contract references it rather than duplicating it.
pub(crate) use tiler_ir::schedule::{FlushedZeroSign, NumericalPermission, SubnormalMode};

use crate::capability::{
    CanonicalLoweringRegistryIdentity, FrozenLoweringCapabilityRegistry, LoweringCapabilityRevision,
};
use crate::governed::{governed_lowering_capabilities, governed_scalars};
use crate::honourability::{
    DeclaredBehaviour, DeferredDimension, DimensionBehaviour, HonouringMeans, NumericalDimension,
    NumericalRequirement, UndeclaredDimension, UnhonouredDimension,
};
use crate::region::SemanticMemberId;

const REQUEST_SCHEMA_VERSION: u32 = 1;
const NUMERICAL_CONTRACT_KEY: &str = "tiler.strict-f32.v1";
/// Versioned key of the governed contract that accepts sign-preserving flushing.
///
/// A distinct key rather than a flag on the strict one: the two contracts give
/// the same program different observable results, so they must give it
/// different canonical identities, artifacts, and cache entries.
const FLUSH_CONTRACT_KEY: &str = "tiler.flush-f32.v1";
const TARGET_PROFILE_KEY: &str = "tiler.prototype-target-neutral-baseline.v1";

/// Maximum byte length of one target-profile key.
///
/// The key enters the request subject and therefore artifact identity, so it is
/// bounded where it is minted rather than wherever it is later encoded.
const MAX_TARGET_PROFILE_KEY_BYTES: usize = 128;

/// The governed key of one declared target profile.
///
/// **Opaque with a fallible constructor**, per ADR 0074 convention 2: a key
/// names a profile that was declared, and a caller assembling one from a bare
/// string could name a profile that never was. The bytes it encodes to are
/// exactly the bytes the `&'static str` it replaces encoded to — `push_slice`
/// of the same run — which is why `the_governed_descriptor_bytes_do_not_move`
/// keeps passing across this change.
///
/// It holds a `Cow` rather than a `&'static str` because
/// `admit-a-caller-declared-target-profile` needs an owned key, and moving the
/// applicability vocabulary onto a type that already admits one is what makes
/// that refactor tractable instead of a single 57-error commit. Nothing
/// constructs an owned key yet; the seam exists so the next step is additive.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct TargetProfileKey(std::borrow::Cow<'static, str>);

impl TargetProfileKey {
    /// Names a key this build governs, with no validation and no allocation.
    ///
    /// Reserved for keys compiled into this crate. A key arriving from outside
    /// goes through [`Self::declared`], which is where the checks are.
    pub(crate) const fn governed(key: &'static str) -> Self {
        Self(std::borrow::Cow::Borrowed(key))
    }

    /// Validates one caller-supplied key.
    ///
    /// # Errors
    ///
    /// [`RequestError::UnsupportedCapability`] when the key is empty, exceeds
    /// [`MAX_TARGET_PROFILE_KEY_BYTES`], or carries a byte outside the governed
    /// spelling. The spelling is restricted so a key cannot carry framing or
    /// display characters into an identity encoding that frames by length.
    #[allow(
        dead_code,
        reason = "the declaration path that consumes it is admit-a-caller-declared-target-profile; the seam lands with the vocabulary that has to accept it"
    )]
    pub(crate) fn declared(key: String) -> Result<Self, RequestError> {
        let admitted = |byte: u8| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        };
        if key.is_empty() || key.len() > MAX_TARGET_PROFILE_KEY_BYTES || !key.bytes().all(admitted)
        {
            return Err(RequestError::UnsupportedCapability {
                phase: "target",
                rule: "target-profile-key-spelling",
            });
        }
        Ok(Self(std::borrow::Cow::Owned(key)))
    }

    /// Returns the key's exact bytes as encoded into identity.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TargetProfileKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
/// Recognized operation count when both pointwise constants are one shared value.
const RECOGNIZED_OPERATIONS_MIN: usize = 4;
/// Recognized operation count when each pointwise constant is a distinct value.
const RECOGNIZED_OPERATIONS_MAX: usize = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StaticShapeEnvironment {
    schema_version: u32,
}

impl StaticShapeEnvironment {
    pub(crate) const fn governed() -> Self {
        Self {
            schema_version: REQUEST_SCHEMA_VERSION,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StrictF32NumericalContract {
    pub(crate) key: &'static str,
    pub(crate) canonical_arithmetic_nan_bits: u32,
    pub(crate) input_subnormals: SubnormalMode,
    pub(crate) result_subnormals: SubnormalMode,
    pub(crate) contraction: NumericalPermission,
    pub(crate) reassociation: NumericalPermission,
}

impl StrictF32NumericalContract {
    pub(crate) const fn governed() -> Self {
        Self {
            key: NUMERICAL_CONTRACT_KEY,
            canonical_arithmetic_nan_bits: tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS,
            input_subnormals: SubnormalMode::Preserve,
            result_subnormals: SubnormalMode::Preserve,
            contraction: NumericalPermission::Forbidden,
            reassociation: NumericalPermission::Forbidden,
        }
    }

    /// The governed contract that accepts sign-preserving subnormal flushing.
    ///
    /// A **different contract, not a relaxation**: its own versioned key, so a
    /// program compiled under it has a different identity. A caller states it to
    /// say that flushing subnormals to the sign-preserving zero is part of what
    /// its program means — which is what makes running on Apple hardware a
    /// choice the caller made rather than a compromise a planner made for it.
    ///
    /// `PreservesSign` because that is what the hardware measurably does
    /// (`0x80400000 * 2.0f` returns `0x80000000`), and a contract must name
    /// which zero it accepts: the two zeros are observably different results.
    ///
    /// Contraction and reassociation stay `Forbidden`. This widens exactly one
    /// dimension, so accepting flushing does not silently accept reassociation.
    pub(crate) const fn governed_flush_to_zero() -> Self {
        Self {
            key: FLUSH_CONTRACT_KEY,
            canonical_arithmetic_nan_bits: tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS,
            input_subnormals: SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            result_subnormals: SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            contraction: NumericalPermission::Forbidden,
            reassociation: NumericalPermission::Forbidden,
        }
    }

    /// Returns every contract this build registers.
    ///
    /// Admission is membership in this set rather than equality with one
    /// constant. Three separate sites previously compared against `governed()`
    /// directly — the request boundary, the per-target verification, and the
    /// physical schedule verifier — so registering a second contract meant
    /// finding all three. This is the single authority they now share.
    pub(crate) const fn governed_profile() -> [Self; 2] {
        [Self::governed(), Self::governed_flush_to_zero()]
    }

    /// Returns whether this contract is one this build registers.
    pub(crate) fn is_governed(&self) -> bool {
        Self::governed_profile()
            .iter()
            .any(|admitted| admitted == self)
    }

    /// Projects this contract into the per-dimension requirements a target
    /// profile's honourability declaration is assessed against.
    ///
    /// One requirement per governed dimension, complete and in canonical order.
    /// Completeness is what makes an unenumerated dimension fail closed: a
    /// contract that simply omitted a dimension would place no requirement on
    /// it, and no requirement is trivially satisfiable rather than `Unknown`.
    ///
    /// `key` and `canonical_arithmetic_nan_bits` are deliberately not projected.
    /// The first names the governing contract and the second is a produced
    /// value; neither is a behaviour a target declares honourability for, and
    /// letting the key stand in for the dimensions it names is exactly the
    /// projection ADR 0076 item 6 forbids.
    pub(crate) fn dimension_requirements(&self) -> Vec<NumericalRequirement> {
        vec![
            NumericalRequirement::new(
                NumericalDimension::InputSubnormals,
                DimensionBehaviour::Subnormals(self.input_subnormals),
            ),
            NumericalRequirement::new(
                NumericalDimension::ResultSubnormals,
                DimensionBehaviour::Subnormals(self.result_subnormals),
            ),
            NumericalRequirement::new(
                NumericalDimension::Contraction,
                DimensionBehaviour::Transform(self.contraction),
            ),
            NumericalRequirement::new(
                NumericalDimension::Reassociation,
                DimensionBehaviour::Transform(self.reassociation),
            ),
        ]
    }

    /// Projects this contract into the target-neutral numerical realization the
    /// scheduled-region IR preserves.
    pub(crate) const fn realization(&self) -> tiler_ir::schedule::NumericalRealization {
        tiler_ir::schedule::NumericalRealization::new(
            self.key,
            self.canonical_arithmetic_nan_bits,
            self.input_subnormals,
            self.result_subnormals,
            self.contraction,
            self.reassociation,
        )
    }
}

/// An ordered, nonempty caller preference over numerical contracts.
///
/// ADR 0076 item 2. A caller states one resolved contract, or an explicitly
/// ordered list of contracts it declares equally acceptable. Resolution is by
/// the caller's stated order, the first honourable entry wins, and it is
/// **never** cost-ranked: cost may rank implementations of one contract and may
/// never rank contracts against each other, because that would price meaning.
///
/// A single-entry list and a bare contract behave identically, so the list is an
/// additive generalization rather than a second mechanism.
///
/// The stated order participates in the request subject even though only the
/// resolved entry drives compilation, because the caller's fallback intent is
/// the thing the list exists to record: two requests whose lists differ but
/// resolve alike are different requests, and an explain trace that could not
/// tell them apart would attribute a resolution to a preference it never saw.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NumericalContractPreference {
    stated: Vec<StrictF32NumericalContract>,
}

impl NumericalContractPreference {
    /// States exactly one acceptable contract.
    pub(crate) fn exactly(contract: StrictF32NumericalContract) -> Self {
        Self {
            stated: vec![contract],
        }
    }

    /// States an ordered preference over several acceptable contracts.
    ///
    /// # Errors
    ///
    /// Returns [`RequestError::UnstatedNumericalContract`] for an empty list.
    /// There is no default and no implicit strictest reading: a request that
    /// states no contract does not compile, and the diagnostic says the contract
    /// is unstated rather than naming a dimension.
    pub(crate) fn ordered(stated: Vec<StrictF32NumericalContract>) -> Result<Self, RequestError> {
        if stated.is_empty() {
            return Err(RequestError::UnstatedNumericalContract);
        }
        Ok(Self { stated })
    }

    /// The stated contracts, in the caller's order.
    pub(crate) fn stated(&self) -> &[StrictF32NumericalContract] {
        &self.stated
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DeterministicBudgets {
    pub(crate) semantic_values: u32,
    pub(crate) semantic_operations: u32,
    pub(crate) regions: u32,
    pub(crate) host_expression_nodes: u32,
    pub(crate) buffers: u32,
    /// Rewrites the deterministic normalization stage may commit.
    ///
    /// Normalization visits each verified operation exactly once, so its
    /// traversal is already bounded by `semantic_operations`. This is the
    /// stage's own explicit budget over committed rewrites.
    pub(crate) normalization_rewrites: u32,
    /// Semantic occurrences admitted in one region candidate.
    pub(crate) region_members: u32,
    /// Retained boundary outputs admitted for one region candidate.
    pub(crate) region_boundary_outputs: u32,
    /// Boundary and member-result values live across one region candidate.
    pub(crate) region_live_values: u32,
    /// Grown candidates admitted for one seed occurrence.
    ///
    /// Singleton coverage is emitted before growth starts and is never bounded
    /// by this budget, so exhausting it loses fused alternatives rather than the
    /// unfused plan.
    pub(crate) region_candidates_per_seed: u32,
    /// Candidate expansion attempts admitted for one compilation request.
    pub(crate) region_expansions: u32,
    /// Distinct legal complete covers retained for one enumeration request.
    ///
    /// The fully-materialized and fused covers are retained unconditionally, so
    /// exhausting this bound loses additional discovered partitions rather than
    /// either extreme.
    pub(crate) region_covers: u32,
    /// Partition-search expansion attempts admitted for one cover enumeration.
    pub(crate) region_cover_expansions: u64,
    /// Complete-plan combinations admitted for one cover source.
    pub(crate) physical_plan_combinations: u64,
}

impl DeterministicBudgets {
    pub(crate) const fn governed() -> Self {
        Self {
            semantic_values: 16,
            semantic_operations: 8,
            regions: 2,
            host_expression_nodes: 32,
            buffers: 3,
            normalization_rewrites: 8,
            region_members: 32,
            region_boundary_outputs: 8,
            region_live_values: 64,
            region_candidates_per_seed: 32,
            region_expansions: 10_000,
            region_covers: 1_024,
            region_cover_expansions: 100_000,
            physical_plan_combinations: 4_096,
        }
    }
}

/// The exact lowering capability whose provider realized one occurrence.
///
/// Both halves are retained because ADR 0072 keeps them separate: the
/// [`ProviderIdentity`] revision is the admitting provider's own
/// output-affecting revision, and the [`LoweringCapabilityRevision`] covers the
/// exact lowering that provider registered for this family and signature. One
/// provider may own several capabilities at independent revisions.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct LoweringProviderIdentity {
    provider: ProviderIdentity,
    capability_key: String,
    capability_revision: LoweringCapabilityRevision,
}

impl LoweringProviderIdentity {
    /// Binds one resolved capability's provider, governed key, and revision.
    pub(crate) const fn new(
        provider: ProviderIdentity,
        capability_key: String,
        capability_revision: LoweringCapabilityRevision,
    ) -> Self {
        Self {
            provider,
            capability_key,
            capability_revision,
        }
    }

    /// Returns the governed key of the capability that lowered the occurrence.
    ///
    /// Minted here rather than derived by a consumer. The key enters artifact
    /// identity through the selected providers ADR 0072 folds in, and a
    /// consumer assembling it from exposed parts would be a second derivation
    /// of one identity — the drift this vocabulary exists to prevent. A
    /// consumer wraps this string in its own key type; it does not compose one.
    pub(crate) fn capability_key(&self) -> &str {
        &self.capability_key
    }

    /// Returns the admitting provider identity.
    pub(crate) const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the resolved capability's output-affecting revision.
    pub(crate) const fn capability_revision(&self) -> LoweringCapabilityRevision {
        self.capability_revision
    }
}

/// The installed lowering authority one compilation request is bound to.
///
/// The snapshot carries the frozen lowering-capability registry the compile path
/// resolves every recognized occurrence through, together with the exact frozen
/// scalar authority that registry was registered against. Neither is a
/// compile-time constant: an out-of-crate provider registered into the registry
/// drives compilation the same way the governed profile does.
#[derive(Clone, Debug)]
pub(crate) struct CompilerCapabilitySnapshot {
    schema_version: u32,
    lowering: FrozenLoweringCapabilityRegistry,
    scalars: FrozenScalarRegistry,
}

impl CompilerCapabilitySnapshot {
    /// Binds one installed lowering registry and the scalar authority it was
    /// registered against.
    pub(crate) const fn new(
        lowering: FrozenLoweringCapabilityRegistry,
        scalars: FrozenScalarRegistry,
    ) -> Self {
        Self {
            schema_version: REQUEST_SCHEMA_VERSION,
            lowering,
            scalars,
        }
    }

    /// Returns the lowering capabilities the bounded profile ships with.
    ///
    /// The snapshot is assembled once and shared. Assembly is deterministic and
    /// depends on nothing outside this crate and `tiler-ir`.
    ///
    /// # Panics
    ///
    /// Panics when Tiler's own governed profile violates the public capability
    /// contract, which is a defect in this crate rather than a caller error.
    pub(crate) fn governed() -> Self {
        static GOVERNED: OnceLock<CompilerCapabilitySnapshot> = OnceLock::new();
        GOVERNED
            .get_or_init(|| {
                let scalars =
                    governed_scalars().expect("the governed scalar authority is well formed");
                let lowering = governed_lowering_capabilities(&scalars)
                    .expect("the governed lowering capabilities are well formed");
                Self::new(lowering, scalars)
            })
            .clone()
    }

    /// Returns the installed lowering-capability registry.
    pub(crate) const fn lowering(&self) -> &FrozenLoweringCapabilityRegistry {
        &self.lowering
    }

    /// Returns the scalar authority every resolved provider emits against.
    pub(crate) const fn scalars(&self) -> &FrozenScalarRegistry {
        &self.scalars
    }

    /// Returns the registry's canonical provenance.
    pub(crate) fn registry_identity(&self) -> &CanonicalLoweringRegistryIdentity {
        self.lowering.canonical_identity()
    }

    /// Returns a snapshot whose registry admits no lowering capability at all.
    ///
    /// It is the smallest installed authority that still pairs correctly with
    /// the governed scalar profile, so a fixture can distinguish "the registry
    /// resolved nothing" from "the request was malformed".
    #[cfg(test)]
    pub(crate) fn without_capabilities() -> Self {
        let scalars = governed_scalars().expect("the governed scalar authority is well formed");
        let lowering = crate::capability::LoweringCapabilityRegistryBuilder::new(
            scalars.semantic_authority().clone(),
            scalars.clone(),
        )
        .freeze();
        Self::new(lowering, scalars)
    }
}

/// Two snapshots are equal exactly when their declared authority is.
///
/// The canonical registry identity binds every registered capability's family,
/// operation, signature, provider, capability revision, and reached authority,
/// together with the composed semantic and scalar snapshots. Provider
/// implementations are deliberately outside it: a provider whose emitted
/// lowering changes must raise its capability revision, which is inside it.
impl PartialEq for CompilerCapabilitySnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.registry_identity() == other.registry_identity()
            && self.scalars.snapshot_identity() == other.scalars.snapshot_identity()
    }
}

impl Eq for CompilerCapabilitySnapshot {}

/// The target-neutral baseline's per-dimension numerical honourability.
///
/// It replaces the retired `supports_strict_f32` boolean (ADR 0076 item 3),
/// which could say only *whether* one summary obligation was met and never which
/// dimension failed or by what means a target would honour it.
///
/// **What is declared and why.** This is a target-*neutral* prototype profile,
/// so it declares exactly the behaviours the contracts this build registers ask
/// for, and no more. Preservation and sign-preserving flushing are both honoured
/// exactly on both subnormal dimensions, which is what makes both registered
/// contracts compile here. Both transform dimensions honour `Forbidden` and
/// `Permitted` exactly: forbidding a transform is an obligation a neutral
/// profile meets by not performing it, and permitting one places no obligation
/// at all, so a target honours it whatever it does.
///
/// **What is deliberately absent.** `FlushToZero { AlwaysPositive }` is not
/// declared on either subnormal dimension. Nothing has measured a target that
/// produces a positive zero for a negative subnormal, and a neutral baseline
/// must not claim a behaviour on no evidence. A contract requiring it therefore
/// resolves to [`crate::feasibility::FeasibilityOutcome::Unknown`] rather than
/// being admitted — the fail-closed direction, and the case that shows an
/// unenumerated behaviour does not default to honoured.
const GOVERNED_TARGET_HONOURABILITY: &[DeclaredBehaviour] = &[
    DeclaredBehaviour::compile_profile(
        NumericalDimension::InputSubnormals,
        DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
        HonouringMeans::SupportedExactly,
    ),
    DeclaredBehaviour::compile_profile(
        NumericalDimension::InputSubnormals,
        DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        }),
        HonouringMeans::SupportedExactly,
    ),
    DeclaredBehaviour::compile_profile(
        NumericalDimension::ResultSubnormals,
        DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
        HonouringMeans::SupportedExactly,
    ),
    DeclaredBehaviour::compile_profile(
        NumericalDimension::ResultSubnormals,
        DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        }),
        HonouringMeans::SupportedExactly,
    ),
    DeclaredBehaviour::compile_profile(
        NumericalDimension::Contraction,
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        HonouringMeans::SupportedExactly,
    ),
    DeclaredBehaviour::compile_profile(
        NumericalDimension::Contraction,
        DimensionBehaviour::Transform(NumericalPermission::Permitted),
        HonouringMeans::SupportedExactly,
    ),
    DeclaredBehaviour::compile_profile(
        NumericalDimension::Reassociation,
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        HonouringMeans::SupportedExactly,
    ),
    DeclaredBehaviour::compile_profile(
        NumericalDimension::Reassociation,
        DimensionBehaviour::Transform(NumericalPermission::Permitted),
        HonouringMeans::SupportedExactly,
    ),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PrototypeTargetProfile {
    pub(crate) key: &'static str,
    pub(crate) max_threads_per_grid_axis: u64,
    pub(crate) max_threads_per_workgroup: u32,
    pub(crate) max_buffer_bindings_per_entry: u32,
    pub(crate) index_bits: u8,
    pub(crate) supports_device_memory: bool,
    /// What this target declares it honours on each numerical dimension.
    ///
    /// A `&'static` slice rather than an owned collection, so the profile stays
    /// a `Copy` value that participates in the request subject and in equality
    /// exactly as its quantitative bounds do.
    pub(crate) numerical: &'static [DeclaredBehaviour],
}

impl PrototypeTargetProfile {
    pub(crate) const fn governed() -> Self {
        Self {
            key: TARGET_PROFILE_KEY,
            max_threads_per_grid_axis: 65_535,
            max_threads_per_workgroup: 1,
            max_buffer_bindings_per_entry: 2,
            index_bits: 64,
            supports_device_memory: true,
            numerical: GOVERNED_TARGET_HONOURABILITY,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CompilationRequest<'a> {
    pub(crate) program: &'a SemanticProgram,
    pub(crate) shape_environment: StaticShapeEnvironment,
    /// The caller's ordered numerical-contract preference. Required, with no
    /// `Default` and no ambient fallback (ADR 0076 item 2).
    pub(crate) numerical_contracts: NumericalContractPreference,
    pub(crate) budgets: DeterministicBudgets,
    pub(crate) target_profiles: Vec<PrototypeTargetProfile>,
    pub(crate) capabilities: CompilerCapabilitySnapshot,
}

impl CompilationRequest<'_> {
    /// Builds the governed compilation request for the bounded prototype profile.
    ///
    /// This is the exact profile the request boundary admits: the governed static
    /// shape environment, strict-`f32` numerical contract, deterministic budgets,
    /// target profile, and installed lowering capabilities. It is the ordinary
    /// entry point every in-crate caller uses, not a test-only shortcut.
    #[allow(
        dead_code,
        reason = "the crate-internal governed request profile; its only in-crate callers are the compile path's own conformance and unit tests until a reviewed public facade exposes it"
    )]
    pub(crate) fn governed(program: &SemanticProgram) -> CompilationRequest<'_> {
        Self::governed_under(program, StrictF32NumericalContract::governed())
    }

    /// Builds the governed request under one caller-stated numerical contract.
    ///
    /// The contract is a parameter with no default. On the measured Apple row
    /// the strictest reading is unhonourable, so a strict default would make
    /// every Apple compilation fail with a rejection the caller never asked for
    /// and leave the knob reachable only by reading that rejection.
    pub(crate) fn governed_under(
        program: &SemanticProgram,
        numerical_contract: StrictF32NumericalContract,
    ) -> CompilationRequest<'_> {
        Self::governed_preferring(
            program,
            NumericalContractPreference::exactly(numerical_contract),
        )
    }

    /// Builds the governed request under a caller-stated ordered preference.
    ///
    /// The list is resolved by the caller's stated order against each target's
    /// declared honourability; the first honourable entry wins. No authority
    /// below this boundary may reorder it, and none may rank the entries by cost.
    pub(crate) fn governed_preferring(
        program: &SemanticProgram,
        numerical_contracts: NumericalContractPreference,
    ) -> CompilationRequest<'_> {
        CompilationRequest {
            program,
            shape_environment: StaticShapeEnvironment::governed(),
            numerical_contracts,
            budgets: DeterministicBudgets::governed(),
            target_profiles: vec![PrototypeTargetProfile::governed()],
            capabilities: CompilerCapabilitySnapshot::governed(),
        }
    }
}

/// The recognized serial-sum occurrences as canonical region member sets.
///
/// The strategy recognizer already walks the verified program to identify these
/// operations, so the exact occurrences it matched are retained instead of being
/// re-encoded as a fixed role vocabulary downstream. Only the ascending member
/// sets are retained: two programs that `tiler-ir` gives one canonical graph
/// identity may store the pointwise constants in either order, and the recognized
/// coverage must not depend on which spelling the caller authored. A shared
/// pointwise constant simply contributes one member instead of two.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecognizedSerialSumMembers {
    pointwise: Vec<SemanticMemberId>,
    reduction: Vec<SemanticMemberId>,
}

impl RecognizedSerialSumMembers {
    fn new(scale_constant: u32, multiply: u32, bias_constant: u32, add: u32, sum: u32) -> Self {
        Self {
            pointwise: ascending([scale_constant, multiply, bias_constant, add]),
            reduction: ascending([sum]),
        }
    }

    /// Returns the pointwise prologue members in ascending order.
    pub(crate) fn pointwise(&self) -> &[SemanticMemberId] {
        &self.pointwise
    }

    /// Returns the reduction members in ascending order.
    pub(crate) fn reduction(&self) -> &[SemanticMemberId] {
        &self.reduction
    }

    /// Returns every recognized member in ascending order.
    pub(crate) fn all(&self) -> Vec<SemanticMemberId> {
        let mut members: Vec<_> = self
            .pointwise
            .iter()
            .chain(&self.reduction)
            .copied()
            .collect();
        members.sort_unstable();
        members.dedup();
        members
    }
}

fn ascending<const N: usize>(ordinals: [u32; N]) -> Vec<SemanticMemberId> {
    let mut ordinals = ordinals;
    ordinals.sort_unstable();
    let mut members: Vec<_> = ordinals.into_iter().map(SemanticMemberId).collect();
    members.dedup();
    members
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedSerialSum {
    pub(crate) input_key: InputKey,
    pub(crate) output_key: OutputKey,
    pub(crate) input_shape: Shape,
    pub(crate) output_shape: Shape,
    pub(crate) reduction_axes: Vec<Axis>,
    pub(crate) scale_bits: u32,
    pub(crate) bias_bits: u32,
    pub(crate) members: RecognizedSerialSumMembers,
    pub(crate) input: ValueId,
    pub(crate) pointwise_result: ValueId,
    pub(crate) output: ValueId,
    pub(crate) input_elements: u64,
    pub(crate) output_elements: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NormalizedProgram {
    SerialSum(NormalizedSerialSum),
}

impl NormalizedProgram {
    pub(crate) const fn serial_sum(&self) -> &NormalizedSerialSum {
        match self {
            Self::SerialSum(normalized) => normalized,
        }
    }

    #[cfg(test)]
    fn serial_sum_mut(&mut self) -> &mut NormalizedSerialSum {
        match self {
            Self::SerialSum(normalized) => normalized,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedCompilationRequest {
    normalized: NormalizedProgram,
    semantic_identity: SemanticIdentity,
    numerical_contracts: NumericalContractPreference,
    /// The contract resolved for each target profile, positionally aligned with
    /// `target_profiles`. Resolution happens once, here, before any planning.
    resolved_contracts: Vec<StrictF32NumericalContract>,
    budgets: DeterministicBudgets,
    target_profiles: Vec<PrototypeTargetProfile>,
    capabilities: CompilerCapabilitySnapshot,
    authorities: Vec<VerifiedRequestSubject>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedTargetRequest {
    normalized: NormalizedProgram,
    semantic_identity: SemanticIdentity,
    numerical_contracts: NumericalContractPreference,
    numerical_contract: StrictF32NumericalContract,
    budgets: DeterministicBudgets,
    target_profile: PrototypeTargetProfile,
    capabilities: CompilerCapabilitySnapshot,
    authority: VerifiedRequestSubject,
}

/// The exact request facts every explain record and receipt is bound to.
///
/// The installed lowering authority participates through its canonical registry
/// identity rather than the registry itself: the identity is comparable and
/// orderable while a registry holding provider implementations is neither, and
/// the identity already binds every authority the registry was frozen over.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedRequestSubject {
    normalized: NormalizedSerialSumSubject,
    semantic_identity: SemanticIdentity,
    /// The caller's stated preference, retained beside the resolved contract.
    ///
    /// Both are bound because they answer different questions: the list is what
    /// the caller declared acceptable, and the resolved entry is what this
    /// target compiles under. Binding only the second would let two requests
    /// with different fallback intents share one subject.
    numerical_contracts: NumericalContractPreference,
    numerical_contract: StrictF32NumericalContract,
    budgets: DeterministicBudgets,
    target_profile: PrototypeTargetProfile,
    capability_schema_version: u32,
    lowering_registry: CanonicalLoweringRegistryIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedSerialSumSubject {
    input_key: InputKey,
    output_key: OutputKey,
    input_shape: Shape,
    output_shape: Shape,
    reduction_axes: Vec<Axis>,
    scale_bits: u32,
    bias_bits: u32,
    members: RecognizedSerialSumMembers,
    input_elements: u64,
    output_elements: u64,
}

impl VerifiedTargetRequest {
    pub(crate) const fn serial_sum(&self) -> &NormalizedSerialSum {
        self.normalized.serial_sum()
    }

    /// The request subject this target compiles under.
    ///
    /// **A borrow of the stored authority, not a reconstruction.** The subject
    /// is a pure function of fields that are private and never mutated after
    /// `for_target` verified them, so rebuilding it per call reproduced a value
    /// this type already holds — and it was called once per proposal, per
    /// region, per cover.
    ///
    /// [`Self::reconstructs_its_authority`] is the separate operation that
    /// re-derives and compares; the two were one method, so every reader paid
    /// the verifier's cost.
    pub(crate) const fn subject(&self) -> &VerifiedRequestSubject {
        &self.authority
    }

    /// Re-derives the subject from this request's fields and compares it to the
    /// stored authority.
    ///
    /// Deliberately **not** what [`Self::subject`] does. This is the tamper
    /// check, it costs a full reconstruction, and it is named so a caller
    /// choosing it is choosing the cost. A reader that only wants the subject
    /// wants the borrow.
    pub(crate) fn reconstructs_its_authority(&self) -> bool {
        request_subject(
            &self.normalized,
            &self.semantic_identity,
            &self.numerical_contracts,
            self.numerical_contract,
            self.budgets,
            self.target_profile,
            &self.capabilities,
        ) == self.authority
    }

    /// The one contract this target compiles under, resolved from the caller's
    /// stated preference before any planning began.
    pub(crate) const fn numerical_contract(&self) -> StrictF32NumericalContract {
        self.numerical_contract
    }

    /// The caller's stated preference, in the caller's order.
    ///
    /// It is bound into the request subject, and therefore into every explain
    /// record and receipt, already; this accessor exists so a consumer can *read*
    /// the fallback intent rather than only distinguish two requests by it.
    pub(crate) fn numerical_contracts(&self) -> &NumericalContractPreference {
        &self.numerical_contracts
    }

    pub(crate) const fn budgets(&self) -> DeterministicBudgets {
        self.budgets
    }

    pub(crate) const fn target_profile(&self) -> PrototypeTargetProfile {
        self.target_profile
    }

    pub(crate) const fn capabilities(&self) -> &CompilerCapabilitySnapshot {
        &self.capabilities
    }

    pub(crate) const fn semantic_identity(&self) -> &SemanticIdentity {
        &self.semantic_identity
    }
}

impl VerifiedRequestSubject {
    pub(crate) const fn normalized(&self) -> &NormalizedSerialSumSubject {
        &self.normalized
    }

    pub(crate) const fn numerical_contract(&self) -> StrictF32NumericalContract {
        self.numerical_contract
    }

    pub(crate) fn canonical_explain_subject_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"tiler.compiler.request-subject.v1\0");
        push_slice(&mut bytes, self.semantic_identity.graph().as_bytes());
        push_slice(
            &mut bytes,
            self.semantic_identity.reached_definitions().as_bytes(),
        );
        push_slice(
            &mut bytes,
            self.semantic_identity.admission_provenance().as_bytes(),
        );
        push_slice(
            &mut bytes,
            self.semantic_identity.registry_snapshot().as_bytes(),
        );
        push_slice(&mut bytes, self.normalized.input_key.as_str().as_bytes());
        push_slice(&mut bytes, self.normalized.output_key.as_str().as_bytes());
        encode_explain_shape(&mut bytes, &self.normalized.input_shape);
        encode_explain_shape(&mut bytes, &self.normalized.output_shape);
        push_len(&mut bytes, self.normalized.reduction_axes.len());
        for axis in &self.normalized.reduction_axes {
            bytes.extend_from_slice(&axis.get().to_be_bytes());
        }
        bytes.extend_from_slice(&self.normalized.scale_bits.to_be_bytes());
        bytes.extend_from_slice(&self.normalized.bias_bits.to_be_bytes());
        for members in [
            self.normalized.members.pointwise(),
            self.normalized.members.reduction(),
        ] {
            push_len(&mut bytes, members.len());
            for member in members {
                bytes.extend_from_slice(&member.0.to_be_bytes());
            }
        }
        bytes.extend_from_slice(&self.normalized.input_elements.to_be_bytes());
        bytes.extend_from_slice(&self.normalized.output_elements.to_be_bytes());
        encode_contract(&mut bytes, self.numerical_contract);
        // The stated preference follows the resolved contract, length-framed and
        // in the caller's order, so a reordered list is a different subject.
        push_len(&mut bytes, self.numerical_contracts.stated().len());
        for contract in self.numerical_contracts.stated() {
            encode_contract(&mut bytes, *contract);
        }
        for budget in [
            self.budgets.semantic_values,
            self.budgets.semantic_operations,
            self.budgets.regions,
            self.budgets.host_expression_nodes,
            self.budgets.buffers,
            self.budgets.normalization_rewrites,
            self.budgets.region_members,
            self.budgets.region_boundary_outputs,
            self.budgets.region_live_values,
            self.budgets.region_candidates_per_seed,
            self.budgets.region_expansions,
            self.budgets.region_covers,
        ] {
            bytes.extend_from_slice(&budget.to_be_bytes());
        }
        for budget in [
            self.budgets.region_cover_expansions,
            self.budgets.physical_plan_combinations,
        ] {
            bytes.extend_from_slice(&budget.to_be_bytes());
        }
        push_slice(&mut bytes, self.target_profile.key.as_bytes());
        bytes.extend_from_slice(&self.target_profile.max_threads_per_grid_axis.to_be_bytes());
        bytes.extend_from_slice(&self.target_profile.max_threads_per_workgroup.to_be_bytes());
        bytes.extend_from_slice(
            &self
                .target_profile
                .max_buffer_bindings_per_entry
                .to_be_bytes(),
        );
        bytes.push(self.target_profile.index_bits);
        bytes.push(u8::from(self.target_profile.supports_device_memory));
        // The honourability declaration replaces the retired `supports_strict_f32`
        // byte. It is encoded per line rather than summarized, because that is
        // exactly what the boolean could not say: which dimension, which
        // behaviour, and by what means.
        push_len(&mut bytes, self.target_profile.numerical.len());
        for declared in self.target_profile.numerical {
            declared.encode_declaration(&mut bytes);
        }
        bytes.extend_from_slice(&self.capability_schema_version.to_be_bytes());
        push_slice(&mut bytes, self.lowering_registry.as_bytes());
        bytes
    }
}

/// Appends one numerical contract's complete canonical encoding.
///
/// Complete over every dimension and exhaustive per dimension through
/// [`subnormal_tag`] and [`permission_tag`]: the contract key is encoded beside
/// the field values it names and never in place of them (ADR 0076 item 6).
fn encode_contract(bytes: &mut Vec<u8>, contract: StrictF32NumericalContract) {
    push_slice(bytes, contract.key.as_bytes());
    bytes.extend_from_slice(&contract.canonical_arithmetic_nan_bits.to_be_bytes());
    bytes.push(subnormal_tag(contract.input_subnormals));
    bytes.push(subnormal_tag(contract.result_subnormals));
    bytes.push(permission_tag(contract.contraction));
    bytes.push(permission_tag(contract.reassociation));
}

/// Returns the canonical tag of one subnormal dimension.
///
/// The mapping is an exhaustive match rather than an `as` discriminant cast.
/// A cast reads whatever ordinal position a variant happens to occupy, so
/// adding or reordering a variant would silently change every encoded request
/// subject; a match stops the build instead (ADR 0074 convention 5b). It also
/// gives the two flush behaviours distinct tags, which a cast over a struct
/// variant cannot express at all.
pub(crate) const fn subnormal_tag(mode: SubnormalMode) -> u8 {
    match mode {
        SubnormalMode::Preserve => 0x01,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        } => 0x02,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        } => 0x03,
    }
}

/// Returns the canonical tag of one transform permission.
pub(crate) const fn permission_tag(permission: NumericalPermission) -> u8 {
    match permission {
        NumericalPermission::Forbidden => 0x01,
        NumericalPermission::Permitted => 0x02,
    }
}

fn encode_explain_shape(output: &mut Vec<u8>, shape: &Shape) {
    push_len(output, shape.rank());
    for extent in shape.extents() {
        output.extend_from_slice(&extent.get().to_be_bytes());
    }
}

impl NormalizedSerialSumSubject {
    pub(crate) const fn input_shape(&self) -> &Shape {
        &self.input_shape
    }
    pub(crate) const fn output_shape(&self) -> &Shape {
        &self.output_shape
    }
    pub(crate) fn reduction_axes(&self) -> &[Axis] {
        &self.reduction_axes
    }
    pub(crate) const fn scale_bits(&self) -> u32 {
        self.scale_bits
    }
    pub(crate) const fn bias_bits(&self) -> u32 {
        self.bias_bits
    }
    pub(crate) const fn members(&self) -> &RecognizedSerialSumMembers {
        &self.members
    }
    pub(crate) const fn input_elements(&self) -> u64 {
        self.input_elements
    }
    pub(crate) const fn output_elements(&self) -> u64 {
        self.output_elements
    }
}

impl VerifiedCompilationRequest {
    pub(crate) fn target_profiles(&self) -> &[PrototypeTargetProfile] {
        &self.target_profiles
    }

    /// Returns the verified deterministic budgets bound to this request.
    pub(crate) const fn budgets(&self) -> DeterministicBudgets {
        self.budgets
    }

    /// Returns the caller's stated numerical-contract preference.
    pub(crate) fn numerical_contracts(&self) -> &NumericalContractPreference {
        &self.numerical_contracts
    }

    /// Returns the one contract every target resolved to, when they agree.
    ///
    /// A program-scoped stage that runs before per-target compilation — semantic
    /// normalization is the only one — needs exactly one contract, and there is
    /// no defensible way to pick among several: a rewrite legal under one
    /// contract may be illegal under another, so normalizing under either would
    /// apply to one target a licence the other never granted. Returning [`None`]
    /// when the targets disagree makes the caller fail closed instead.
    pub(crate) fn uniform_resolved_contract(&self) -> Option<StrictF32NumericalContract> {
        let (first, rest) = self.resolved_contracts.split_first()?;
        rest.iter()
            .all(|resolved| resolved == first)
            .then_some(*first)
    }

    pub(crate) fn for_target(
        &self,
        target_profile: PrototypeTargetProfile,
    ) -> Result<VerifiedTargetRequest, RequestError> {
        let Some(index) = self
            .target_profiles
            .iter()
            .position(|profile| *profile == target_profile)
        else {
            return Err(RequestError::UnverifiedTargetSelection);
        };
        let Some(numerical_contract) = self.resolved_contracts.get(index).copied() else {
            return Err(RequestError::UnverifiedTargetSelection);
        };
        let current_authority = request_subject(
            &self.normalized,
            &self.semantic_identity,
            &self.numerical_contracts,
            numerical_contract,
            self.budgets,
            target_profile,
            &self.capabilities,
        );
        if target_profile != PrototypeTargetProfile::governed()
            || self
                .target_profiles
                .iter()
                .any(|profile| *profile != PrototypeTargetProfile::governed())
            || !numerical_contract.is_governed()
            || self.authorities.get(index) != Some(&current_authority)
        {
            return Err(RequestError::UnverifiedTargetSelection);
        }
        Ok(VerifiedTargetRequest {
            normalized: self.normalized.clone(),
            semantic_identity: self.semantic_identity.clone(),
            numerical_contracts: self.numerical_contracts.clone(),
            numerical_contract,
            budgets: self.budgets,
            target_profile,
            capabilities: self.capabilities.clone(),
            authority: current_authority,
        })
    }
}

fn request_subject(
    normalized: &NormalizedProgram,
    semantic_identity: &SemanticIdentity,
    numerical_contracts: &NumericalContractPreference,
    numerical_contract: StrictF32NumericalContract,
    budgets: DeterministicBudgets,
    target_profile: PrototypeTargetProfile,
    capabilities: &CompilerCapabilitySnapshot,
) -> VerifiedRequestSubject {
    #[cfg(test)]
    crate::workcount::REQUEST_SUBJECT_REBUILDS.record();
    let normalized = normalized.serial_sum();
    VerifiedRequestSubject {
        normalized: NormalizedSerialSumSubject {
            input_key: normalized.input_key.clone(),
            output_key: normalized.output_key.clone(),
            input_shape: normalized.input_shape.clone(),
            output_shape: normalized.output_shape.clone(),
            reduction_axes: normalized.reduction_axes.clone(),
            scale_bits: normalized.scale_bits,
            bias_bits: normalized.bias_bits,
            members: normalized.members.clone(),
            input_elements: normalized.input_elements,
            output_elements: normalized.output_elements,
        },
        semantic_identity: semantic_identity.clone(),
        numerical_contracts: numerical_contracts.clone(),
        numerical_contract,
        budgets,
        target_profile,
        capability_schema_version: capabilities.schema_version,
        lowering_registry: capabilities.registry_identity().clone(),
    }
}

/// Why one stated numerical contract could not be resolved on one target.
///
/// The three arms are three different claims and are deliberately not collapsed:
/// a declared refusal, an absent declaration, and a declaration that has not yet
/// become available are not the same thing, and reporting the second or third as
/// a rejection would assert knowledge the profile never supplied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContractRejection {
    /// The target declares it cannot honour a required behaviour.
    Unhonourable {
        contract_key: &'static str,
        cause: UnhonouredDimension,
    },
    /// Nothing the profile declares speaks to a required behaviour, so the
    /// dimension is `Unknown` in ADR 0043's sense and fails closed.
    Undeclared {
        contract_key: &'static str,
        cause: UndeclaredDimension,
    },
    /// The declaration exists only from a later availability phase, so it cannot
    /// resolve the contract at the compile profile.
    Deferred {
        contract_key: &'static str,
        cause: DeferredDimension,
    },
}

impl ContractRejection {
    /// The contract whose resolution this rejection explains.
    pub(crate) const fn contract_key(self) -> &'static str {
        match self {
            Self::Unhonourable { contract_key, .. }
            | Self::Undeclared { contract_key, .. }
            | Self::Deferred { contract_key, .. } => contract_key,
        }
    }

    /// The dimension the resolution failed on.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        match self {
            Self::Unhonourable { cause, .. } => cause.dimension(),
            Self::Undeclared { cause, .. } => cause.dimension(),
            Self::Deferred { cause, .. } => cause.dimension(),
        }
    }

    /// The behaviour the contract required on that dimension.
    pub(crate) const fn required(self) -> DimensionBehaviour {
        match self {
            Self::Unhonourable { cause, .. } => cause.required(),
            Self::Undeclared { cause, .. } => cause.required(),
            Self::Deferred { cause, .. } => cause.required(),
        }
    }
}

impl fmt::Display for ContractRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {} requires {}",
            self.contract_key(),
            self.dimension().key(),
            self.required().key()
        )?;
        match self {
            Self::Unhonourable { cause, .. } => {
                write!(formatter, ", target declares {}", cause.means().key())?;
                if let Some(honoured) = cause.honoured() {
                    write!(formatter, " and honours {}", honoured.key())?;
                }
                write!(formatter, " (profile {})", cause.profile().key())
            }
            Self::Undeclared { .. } => formatter.write_str(", target declares nothing"),
            Self::Deferred { cause, .. } => write!(
                formatter,
                ", target declares it only from a later phase ({:?})",
                cause.phase()
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RequestError {
    UnsupportedRequestVersion,
    EmptyTargetSet,
    DuplicateTargetProfile,
    UnverifiedTargetSelection,
    /// The caller stated no numerical contract at all.
    ///
    /// Distinct from every rejection that names a dimension: there is no default
    /// and no implicit strictest reading, so the diagnostic says the contract is
    /// unstated rather than reporting a dimension the caller never chose.
    UnstatedNumericalContract,
    /// No contract in the caller's stated order resolves on this target.
    ///
    /// Every stated entry's first canonical failure is retained, in the caller's
    /// order, so the diagnostic explains the whole preference rather than only
    /// its last entry. Nothing here proposes a substitute contract: only the
    /// caller may change what its program means.
    NoResolvableNumericalContract {
        target_profile: &'static str,
        rejections: Vec<ContractRejection>,
    },
    BudgetExceeded {
        resource: &'static str,
        limit: u32,
        actual: usize,
    },
    UnsupportedCapability {
        phase: &'static str,
        rule: &'static str,
    },
    ShapeProductOverflow {
        role: &'static str,
    },
}

impl fmt::Display for RequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedRequestVersion => {
                formatter.write_str("compile.request.schema: unsupported static shape environment")
            }
            Self::EmptyTargetSet => formatter
                .write_str("compile.request.targets.empty: at least one target is required"),
            Self::DuplicateTargetProfile => formatter
                .write_str("compile.request.targets.duplicate: target profile keys must be unique"),
            Self::UnverifiedTargetSelection => formatter.write_str(
                "compile.request.targets.selection: target was not verified by the request",
            ),
            Self::UnstatedNumericalContract => formatter.write_str(
                "compile.request.numerics.unstated: a resolved numerical contract is required",
            ),
            Self::NoResolvableNumericalContract {
                target_profile,
                rejections,
            } => {
                write!(
                    formatter,
                    "compile.request.numerics.unhonourable: target {target_profile} honours no stated contract"
                )?;
                for rejection in rejections {
                    write!(formatter, "; {rejection}")?;
                }
                Ok(())
            }
            Self::BudgetExceeded {
                resource,
                limit,
                actual,
            } => write!(
                formatter,
                "compile.budget.{resource}: {actual} exceeds deterministic limit {limit}"
            ),
            Self::UnsupportedCapability { phase, rule } => {
                write!(
                    formatter,
                    "compile.unsupported.{phase}.{rule}: no installed capability can compile this valid semantic program"
                )
            }
            Self::ShapeProductOverflow { role } => write!(
                formatter,
                "compile.shape.{role}.element-count: static element count exceeds u64"
            ),
        }
    }
}

impl Error for RequestError {}

pub(crate) fn verify_request(
    request: CompilationRequest<'_>,
) -> Result<VerifiedCompilationRequest, RequestError> {
    if request.shape_environment != StaticShapeEnvironment::governed() {
        return Err(RequestError::UnsupportedRequestVersion);
    }
    if request.capabilities.schema_version != REQUEST_SCHEMA_VERSION {
        return Err(RequestError::UnsupportedRequestVersion);
    }
    // The registry itself is deliberately unconstrained: an externally
    // registered lowering provider is exactly what this boundary admits. What is
    // constrained is that the request pairs the registry with the same scalar
    // authority its capabilities were admitted against, because every resolved
    // provider is driven through — and revalidated under — that snapshot.
    if request.capabilities.lowering.scalar_snapshot()
        != request.capabilities.scalars.snapshot_identity()
    {
        return unsupported("capability", "scalar-authority-pairing");
    }
    if request.target_profiles.is_empty() {
        return Err(RequestError::EmptyTargetSet);
    }
    if request.numerical_contracts.stated().is_empty() {
        return Err(RequestError::UnstatedNumericalContract);
    }
    if request
        .numerical_contracts
        .stated()
        .iter()
        .any(|contract| !contract.is_governed())
    {
        // Names the profile rather than one contract: a caller stating an
        // unadmitted contract has not violated the strict one, it has named a
        // contract this build does not register.
        return unsupported("numerics", "governed-contract-profile");
    }
    if request
        .target_profiles
        .iter()
        .any(|target| *target != PrototypeTargetProfile::governed())
    {
        return unsupported("target", "prototype-target-neutral-baseline-v1");
    }
    let mut target_keys: Vec<_> = request
        .target_profiles
        .iter()
        .map(|target| target.key)
        .collect();
    target_keys.sort_unstable();
    if target_keys.windows(2).any(|keys| keys[0] == keys[1]) {
        return Err(RequestError::DuplicateTargetProfile);
    }
    check_budget(
        "semantic-values",
        request.budgets.semantic_values,
        request.program.value_count(),
    )?;
    check_budget(
        "semantic-operations",
        request.budgets.semantic_operations,
        request.program.operation_count(),
    )?;
    check_budget("regions", request.budgets.regions, 2)?;
    check_budget(
        "host-expression-nodes",
        request.budgets.host_expression_nodes,
        9,
    )?;
    check_budget("buffers", request.budgets.buffers, 3)?;

    let normalized = select_supported_strategy(request.program)?;
    let semantic_identity = request.program.semantic_identity().clone();
    // Resolve the caller's preference once, per target, before any planning
    // begins. Resolution is the honourability authority applied to the contract
    // alone — no region, no schedule, and no cost participates, because the
    // contract is not a search dimension (ADR 0076 item 5).
    let resolved_contracts = request
        .target_profiles
        .iter()
        .map(|target| resolve_numerical_contract(&request.numerical_contracts, target))
        .collect::<Result<Vec<_>, _>>()?;
    let authorities = request
        .target_profiles
        .iter()
        .zip(&resolved_contracts)
        .map(|(target, resolved)| {
            request_subject(
                &normalized,
                &semantic_identity,
                &request.numerical_contracts,
                *resolved,
                request.budgets,
                *target,
                &request.capabilities,
            )
        })
        .collect();
    Ok(VerifiedCompilationRequest {
        normalized,
        semantic_identity,
        numerical_contracts: request.numerical_contracts,
        resolved_contracts,
        budgets: request.budgets,
        target_profiles: request.target_profiles,
        capabilities: request.capabilities,
        authorities,
    })
}

/// Resolves a caller's ordered preference against one target's declaration.
///
/// The first stated entry every one of whose dimensions the target honours wins.
/// The order is the caller's; nothing here reorders, scores, or blends the
/// entries, and no entry is admitted on a weakened reading of itself.
///
/// # Errors
///
/// Returns [`RequestError::NoResolvableNumericalContract`] carrying one
/// canonical-first cause per stated entry, in the caller's order, when no entry
/// resolves. A malformed profile is an intrinsic contract violation rather than
/// a resolution outcome and surfaces as
/// [`RequestError::UnsupportedCapability`].
fn resolve_numerical_contract(
    preference: &NumericalContractPreference,
    target: &PrototypeTargetProfile,
) -> Result<StrictF32NumericalContract, RequestError> {
    let mut rejections = Vec::new();
    for contract in preference.stated() {
        let outcome = crate::physical::assess_contract(target, *contract).map_err(|_| {
            RequestError::UnsupportedCapability {
                phase: "numerics",
                rule: "target-profile-malformed",
            }
        })?;
        match outcome {
            crate::feasibility::FeasibilityOutcome::Proven(_) => return Ok(*contract),
            crate::feasibility::FeasibilityOutcome::Rejected(rejection) => {
                // The representative is the canonical-first unhonourable
                // dimension; a contract-only proposal has no capability
                // requirements, so it is always a numerical cause.
                if let crate::feasibility::RejectionCause::Numerical(cause) =
                    rejection.representative()
                {
                    rejections.push(ContractRejection::Unhonourable {
                        contract_key: contract.key,
                        cause,
                    });
                }
            }
            crate::feasibility::FeasibilityOutcome::Unknown(unknown) => {
                rejections.extend(unknown.dimensions().first().map(|cause| {
                    ContractRejection::Undeclared {
                        contract_key: contract.key,
                        cause: *cause,
                    }
                }));
            }
            crate::feasibility::FeasibilityOutcome::Deferred(deferred) => {
                rejections.extend(deferred.dimensions().first().map(|cause| {
                    ContractRejection::Deferred {
                        contract_key: contract.key,
                        cause: *cause,
                    }
                }));
            }
        }
    }
    Err(RequestError::NoResolvableNumericalContract {
        target_profile: target.key,
        rejections,
    })
}

fn select_supported_strategy(program: &SemanticProgram) -> Result<NormalizedProgram, RequestError> {
    normalize_serial_sum(program).map(NormalizedProgram::SerialSum)
}

fn check_budget(resource: &'static str, limit: u32, actual: usize) -> Result<(), RequestError> {
    if u64::try_from(actual).map_or(true, |actual| actual > u64::from(limit)) {
        return Err(RequestError::BudgetExceeded {
            resource,
            limit,
            actual,
        });
    }
    Ok(())
}

fn normalize_serial_sum(program: &SemanticProgram) -> Result<NormalizedSerialSum, RequestError> {
    // The recognized structure is exactly one reduction, two pointwise
    // operations, and one or two constants; a shared constant is the normalized
    // spelling of the same program. The exact count is pinned against the
    // distinct recognized set once the structural walk has identified it.
    if program.input_count() != 1
        || program.output_count() != 1
        || !(RECOGNIZED_OPERATIONS_MIN..=RECOGNIZED_OPERATIONS_MAX)
            .contains(&program.operation_count())
    {
        return mismatch("signature");
    }
    if program
        .values()
        .any(|value| value.resolved_type() != &F32::resolved_type())
    {
        return mismatch("dtype-f32");
    }

    let input = program
        .inputs()
        .next()
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "missing-input",
        })?;
    let output = program
        .outputs()
        .next()
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "missing-output",
        })?;
    let (sum_operation, sum) = producer(program, output.value(), &strict_serial_sum_f32_op())?;
    let sum_operands: Vec<_> = sum.operands().collect();
    let sum_results: Vec<_> = sum.results().collect();
    let [pointwise_result] = sum_operands.as_slice() else {
        return mismatch("sum-signature");
    };
    if sum_results.as_slice() != [output.value()] {
        return mismatch("sum-output");
    }

    let (add_operation, add) = producer(program, *pointwise_result, &add_f32_op())?;
    let (multiply_result, bias) = split_tensor_and_scalar(program, &add)?;
    let (multiply_operation, multiply) = producer(program, multiply_result, &multiply_f32_op())?;
    let (tensor_input, scale) = split_tensor_and_scalar(program, &multiply)?;
    if tensor_input != input.value() {
        return mismatch("pointwise-input");
    }
    let (scale, scale_operation) = constant_bits(program, scale)?;
    let (bias, bias_operation) = constant_bits(program, bias)?;
    let members = RecognizedSerialSumMembers::new(
        scale_operation,
        multiply_operation,
        bias_operation,
        add_operation,
        sum_operation,
    );

    check_recognized_operation_cover(program, &members)?;
    let axes = reduction_axes(sum.attributes())?;

    let input_shape = program
        .shape(input.value())
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "input-handle",
        })?
        .clone();
    if input_shape.rank() == 0 {
        return mismatch("input-rank");
    }
    check_canonical_reduction_axes(&axes, input_shape.rank())?;
    if program.shape(*pointwise_result).ok() != Some(&input_shape) {
        return mismatch("pointwise-shape");
    }
    let output_shape = input_shape.without_axes(&axes);
    if program.shape(output.value()).ok() != Some(&output_shape) {
        return mismatch("sum-shape");
    }
    let input_elements = element_count_u64(&input_shape, "input")?;
    let output_elements = element_count_u64(&output_shape, "output")?;

    Ok(NormalizedSerialSum {
        input_key: input.key().clone(),
        output_key: output.key().clone(),
        input_shape,
        output_shape,
        reduction_axes: axes,
        scale_bits: scale,
        bias_bits: bias,
        members,
        input: input.value(),
        pointwise_result: *pointwise_result,
        output: output.value(),
        input_elements,
        output_elements,
    })
}

/// Requires reduction axes to be in range and in strictly ascending order.
fn check_canonical_reduction_axes(axes: &[Axis], rank: usize) -> Result<(), RequestError> {
    let mut previous = None;
    for axis in axes {
        let index =
            usize::try_from(axis.get()).map_err(|_| RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "sum-axis-range",
            })?;
        if index >= rank {
            return mismatch("sum-axis-range");
        }
        if previous.is_some_and(|previous| previous >= axis.get()) {
            return mismatch("sum-axes-canonical");
        }
        previous = Some(axis.get());
    }
    Ok(())
}

/// Requires the recognized operations to cover the whole program exactly.
///
/// A built program retains only output-reachable operations, so demanding that
/// the reachable count equal the distinct recognized set rejects any operation
/// outside this exact structure. One constant shared by both pointwise operands
/// is the normalized spelling of the same program and covers four distinct
/// operations instead of five.
fn check_recognized_operation_cover(
    program: &SemanticProgram,
    recognized: &RecognizedSerialSumMembers,
) -> Result<(), RequestError> {
    if program.operation_count() != recognized.all().len() {
        return mismatch("signature");
    }
    Ok(())
}

fn producer<'a>(
    program: &'a SemanticProgram,
    value: ValueId,
    expected: &OpKey,
) -> Result<(u32, tiler_ir::semantic::OperationRef<'a>), RequestError> {
    let (ordinal, operation) = program
        .operations()
        .enumerate()
        .find(|(_, operation)| operation.results().any(|result| result == value))
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "missing-producer",
        })?;
    if operation.key() != expected {
        return mismatch("operation-family");
    }
    let ordinal = u32::try_from(ordinal).map_err(|_| RequestError::UnsupportedCapability {
        phase: "strategy",
        rule: "operation-ordinal",
    })?;
    Ok((ordinal, operation))
}

fn split_tensor_and_scalar(
    program: &SemanticProgram,
    operation: &tiler_ir::semantic::OperationRef<'_>,
) -> Result<(ValueId, ValueId), RequestError> {
    let operands: Vec<_> = operation.operands().collect();
    let [left, right] = operands.as_slice() else {
        return mismatch("pointwise-arity");
    };
    match (
        program.shape(*left).map(Shape::rank),
        program.shape(*right).map(Shape::rank),
    ) {
        (Ok(left_rank), Ok(0)) if left_rank > 0 => Ok((*left, *right)),
        (Ok(0), Ok(right_rank)) if right_rank > 0 => Ok((*right, *left)),
        _ => mismatch("scalar-broadcast"),
    }
}

fn constant_bits(program: &SemanticProgram, value: ValueId) -> Result<(u32, u32), RequestError> {
    let (ordinal, operation) = producer(program, value, &constant_f32_op())?;
    if operation.operands().len() != 0 || operation.results().len() != 1 {
        return mismatch("constant-signature");
    }
    let Some(CanonicalValueView::FloatBits(bits)) = operation
        .attributes()
        .get(F32_CONSTANT_BITS_ATTRIBUTE)
        .map(tiler_ir::semantic::CanonicalValue::view)
    else {
        return mismatch("constant-bits");
    };
    let governed_f32 =
        TypeKey::new("tiler", "f32", 1).map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "governed-f32-key",
        })?;
    if bits.format() != &governed_f32 {
        return mismatch("constant-bits-format");
    }
    <[u8; 4]>::try_from(bits.bits())
        .map(|bytes| (u32::from_be_bytes(bytes), ordinal))
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "constant-bits",
        })
}

fn reduction_axes(
    attributes: &tiler_ir::semantic::OperationAttributes,
) -> Result<Vec<Axis>, RequestError> {
    let Some(CanonicalValueView::Sequence(values)) = attributes
        .get(REDUCTION_AXES_ATTRIBUTE)
        .map(tiler_ir::semantic::CanonicalValue::view)
    else {
        return mismatch("sum-axes");
    };
    values
        .iter()
        .map(|value| {
            let CanonicalValueView::Unsigned { width, bits } = value.view() else {
                return mismatch("sum-axes");
            };
            if width != CanonicalIntegerWidth::Bits32 {
                return mismatch("sum-axes-width");
            }
            u32::try_from(bits)
                .map(Axis::new)
                .map_err(|_| RequestError::UnsupportedCapability {
                    phase: "strategy",
                    rule: "sum-axes",
                })
        })
        .collect()
}

fn element_count_u64(shape: &Shape, role: &'static str) -> Result<u64, RequestError> {
    if shape.extents().iter().any(|extent| extent.get() == 0) {
        return Ok(0);
    }
    shape.extents().iter().try_fold(1_u64, |count, extent| {
        count
            .checked_mul(extent.get())
            .ok_or(RequestError::ShapeProductOverflow { role })
    })
}

fn mismatch<T>(rule: &'static str) -> Result<T, RequestError> {
    unsupported("strategy", rule)
}

fn unsupported<T>(phase: &'static str, rule: &'static str) -> Result<T, RequestError> {
    Err(RequestError::UnsupportedCapability { phase, rule })
}

#[cfg(test)]
mod tests {
    /// A declared key is validated, and the governed one encodes unchanged.
    ///
    /// The spelling rule exists because the key is framed by length into an
    /// identity encoding: a key carrying whitespace or framing bytes would be
    /// encodable but unreadable in a trace, and one carrying arbitrary bytes
    /// would make two profiles distinguishable only by something no reader can
    /// print. The bound is checked at the same place for the same reason.
    #[test]
    fn a_declared_target_profile_key_is_validated() {
        assert!(TargetProfileKey::declared("tiler.some-target.v1".to_owned()).is_ok());
        assert!(TargetProfileKey::declared("with_underscore-1.0".to_owned()).is_ok());

        for refused in [
            String::new(),
            "Tiler.Capital.v1".to_owned(),
            "has space".to_owned(),
            "has\u{0}nul".to_owned(),
            "x".repeat(super::MAX_TARGET_PROFILE_KEY_BYTES + 1),
        ] {
            assert!(
                TargetProfileKey::declared(refused.clone()).is_err(),
                "an unadmitted key was accepted: {refused:?}",
            );
        }

        // The governed key round-trips to exactly the bytes it always encoded,
        // which is what `the_governed_descriptor_bytes_do_not_move` rests on.
        assert_eq!(
            TargetProfileKey::governed(TARGET_PROFILE_KEY).as_str(),
            TARGET_PROFILE_KEY,
        );
    }

    use std::sync::Arc;

    use super::*;
    use tiler_ir::semantic::{
        CanonicalValue, CanonicalValueKind, F32Add, F32Constant, F32Multiply,
        NormativeDefinitionRef, OperationArity, OperationAttributeSchema, OperationConformance,
        OperationDefinition, OperationDefinitionFacts, OperationEffect, OperationInferenceError,
        OperationInferencer, OperationSchema, ProviderDiagnosticCode, ProviderIdentity,
        RegistryError, SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
        SemanticRegistryRegistrar, StrictSerialF32Sum, TypeDefinitionFacts, ValueFact,
        ValueTypeDefinition, ValueTypeDefinitionKey,
    };

    fn diagnostic_code(value: &str) -> ProviderDiagnosticCode {
        ProviderDiagnosticCode::new(value).unwrap()
    }

    pub(super) fn program() -> SemanticProgram {
        program_with_builder(SemanticProgramBuilder::try_standard().unwrap())
    }

    fn program_with_builder(mut builder: SemanticProgramBuilder) -> SemanticProgram {
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let pointwise = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, pointwise, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
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
            request: tiler_ir::semantic::OperationInferenceRequest<'_>,
            outputs: &mut tiler_ir::semantic::OperationInferenceOutputs<'_>,
        ) -> Result<(), OperationInferenceError> {
            let operands = request.operands();
            let attributes = request.attributes();
            match self {
                Self::Constant => {
                    outputs.try_push(ValueFact::new(F32::resolved_type(), Shape::new([])))
                }
                Self::Binary => {
                    let left = operands[0].shape();
                    let right = operands[1].shape();
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
                        operands[0].shape().without_axes(&axes),
                    ))
                }
            }
        }
    }

    struct GovernedTestSemantics {
        revision: u32,
    }

    impl SemanticRegistryProvider for GovernedTestSemantics {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("tiler-test", "governed-semantics", self.revision).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            registrar.register_marked_value_type::<F32>(
                ValueTypeDefinition::structurally_valid(
                    ValueTypeDefinitionKey::Nominal(
                        TypeKey::new("tiler", "f32", 1).expect("the test F32 key is valid"),
                    ),
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
            )?;
            register_test_operation(registrar, multiply_f32_op(), 2, [], TestOperation::Binary)?;
            register_test_operation(registrar, add_f32_op(), 2, [], TestOperation::Binary)?;
            register_test_operation(
                registrar,
                strict_serial_sum_f32_op(),
                1,
                [OperationAttributeSchema::required(
                    REDUCTION_AXES_ATTRIBUTE,
                    CanonicalValueKind::Sequence,
                )],
                TestOperation::Sum,
            )
        }
    }

    fn register_test_operation<const N: usize>(
        registrar: &mut SemanticRegistryRegistrar<'_>,
        key: OpKey,
        operands: u32,
        attributes: [OperationAttributeSchema; N],
        inferencer: TestOperation,
    ) -> Result<(), RegistryError> {
        registrar.register_operation(OperationDefinition::new(
            key,
            OperationSchema::new(
                OperationArity::exact(operands),
                OperationArity::exact(1),
                attributes,
            )
            .expect("the test operation schema is valid"),
            NormativeDefinitionRef::new("test governed operation semantics")?,
            OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
            OperationConformance::new(CanonicalValue::boolean(true)),
            OperationEffect::Pure,
            Arc::new(inferencer),
        ))
    }

    fn governed_test_program(revision: u32) -> SemanticProgram {
        let mut registry = SemanticRegistryBuilder::new();
        registry
            .register_provider(&GovernedTestSemantics { revision })
            .unwrap();
        program_with_builder(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
    }

    struct UnusedSemantics {
        revision: u32,
    }

    impl SemanticRegistryProvider for UnusedSemantics {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("tiler-test", "unused-semantics", self.revision).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(
                    TypeKey::new("tiler-test", "unused", 1).expect("the test key is valid"),
                ),
                NormativeDefinitionRef::new("unused test semantics")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ))
        }
    }

    fn program_with_unused_provider(revision: u32) -> SemanticProgram {
        let mut registry = SemanticRegistryBuilder::standard().unwrap();
        registry
            .register_provider(&UnusedSemantics { revision })
            .unwrap();
        program_with_builder(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
    }

    #[test]
    fn governed_request_selects_the_supported_serial_sum_strategy() {
        let program = program();
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
        let normalized = verified.normalized.serial_sum();
        assert_eq!(normalized.input_shape, Shape::from_dims([2, 3]));
        assert_eq!(normalized.output_shape, Shape::from_dims([2]));
        assert_eq!(normalized.reduction_axes, [Axis::new(1)]);
        assert_eq!(normalized.scale_bits, 2.0_f32.to_bits());
        assert_eq!(normalized.bias_bits, 1.0_f32.to_bits());
        assert_eq!(normalized.input_elements, 6);
        assert_eq!(normalized.output_elements, 2);
        assert_eq!(
            verified.target_profiles,
            [PrototypeTargetProfile::governed()]
        );
    }

    #[test]
    fn request_rejects_profile_and_budget_mismatches_stably() {
        let program = program();
        let mut request = CompilationRequest::governed(&program);
        request.budgets.semantic_operations = 4;
        assert_eq!(
            verify_request(request),
            Err(RequestError::BudgetExceeded {
                resource: "semantic-operations",
                limit: 4,
                actual: 5,
            })
        );

        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), input)
            .unwrap();
        let invalid = builder.build().unwrap();
        assert_eq!(
            verify_request(CompilationRequest::governed(&invalid)),
            Err(RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "signature",
            })
        );
    }

    /// A single-entry list and a bare contract behave identically.
    ///
    /// The list is an additive generalization, not a second mechanism, so the
    /// two spellings must produce the same verified request — including the same
    /// request subject, which is what binds the caller's stated intent into
    /// every explain record and receipt.
    #[test]
    fn a_single_entry_preference_and_a_bare_contract_are_the_same_request() {
        let program = program();
        let bare = verify_request(CompilationRequest::governed_under(
            &program,
            StrictF32NumericalContract::governed(),
        ))
        .unwrap();
        let listed = verify_request(CompilationRequest::governed_preferring(
            &program,
            NumericalContractPreference::ordered(vec![StrictF32NumericalContract::governed()])
                .unwrap(),
        ))
        .unwrap();
        assert_eq!(bare, listed);
        let target = PrototypeTargetProfile::governed();
        assert_eq!(
            bare.for_target(target).unwrap().subject(),
            listed.for_target(target).unwrap().subject(),
        );
    }

    /// Resolution follows the caller's stated order, never a ranking of its own.
    ///
    /// The governed baseline honours both registered contracts, so whichever
    /// entry the caller put first is the one that wins. That is the whole
    /// property: nothing here prefers the strict entry because it is stricter or
    /// the flushing entry because it is cheaper, because a cost may never rank
    /// contracts against each other.
    #[test]
    fn a_preference_list_resolves_by_the_callers_order_and_never_by_rank() {
        let program = program();
        for (first, second) in [
            (
                StrictF32NumericalContract::governed(),
                StrictF32NumericalContract::governed_flush_to_zero(),
            ),
            (
                StrictF32NumericalContract::governed_flush_to_zero(),
                StrictF32NumericalContract::governed(),
            ),
        ] {
            let verified = verify_request(CompilationRequest::governed_preferring(
                &program,
                NumericalContractPreference::ordered(vec![first, second]).unwrap(),
            ))
            .unwrap();
            assert_eq!(verified.uniform_resolved_contract(), Some(first));
            let target = verified
                .for_target(PrototypeTargetProfile::governed())
                .unwrap();
            assert_eq!(target.numerical_contract(), first);
            // The whole stated list is retained, not only the winner: the
            // caller's fallback intent is what the list exists to record.
            assert_eq!(
                target.numerical_contracts().stated(),
                [first, second].as_slice()
            );
        }
    }

    /// Two lists that resolve alike but state different fallbacks are different
    /// requests.
    ///
    /// If the subject bound only the resolved contract, an explain trace and an
    /// artifact would attribute one resolution to a preference they never saw.
    #[test]
    fn the_stated_preference_separates_requests_that_resolve_alike() {
        let program = program();
        let target = PrototypeTargetProfile::governed();
        let alone = verify_request(CompilationRequest::governed_preferring(
            &program,
            NumericalContractPreference::ordered(vec![StrictF32NumericalContract::governed()])
                .unwrap(),
        ))
        .unwrap();
        let with_fallback = verify_request(CompilationRequest::governed_preferring(
            &program,
            NumericalContractPreference::ordered(vec![
                StrictF32NumericalContract::governed(),
                StrictF32NumericalContract::governed_flush_to_zero(),
            ])
            .unwrap(),
        ))
        .unwrap();
        let alone = alone.for_target(target).unwrap();
        let with_fallback = with_fallback.for_target(target).unwrap();
        assert_eq!(
            alone.numerical_contract(),
            with_fallback.numerical_contract()
        );
        assert_ne!(
            alone.subject().canonical_explain_subject_bytes(),
            with_fallback.subject().canonical_explain_subject_bytes(),
        );
    }

    /// A request that states no contract does not compile, and says so.
    ///
    /// The diagnostic names the contract as unstated rather than naming a
    /// dimension: there is no default and no implicit strictest reading, so
    /// there is no dimension the caller chose to report against.
    #[test]
    fn an_unstated_numerical_contract_is_refused_by_name() {
        let program = program();
        assert_eq!(
            NumericalContractPreference::ordered(Vec::new()),
            Err(RequestError::UnstatedNumericalContract)
        );
        let mut request = CompilationRequest::governed(&program);
        request.numerical_contracts.stated.clear();
        assert_eq!(
            verify_request(request),
            Err(RequestError::UnstatedNumericalContract)
        );
    }

    /// A target that honours no stated entry rejects, naming every entry's cause.
    ///
    /// The governed baseline deliberately declares nothing about the
    /// always-positive flush, so a contract requiring it is `Undeclared` — the
    /// fail-closed direction — rather than admitted. The rejection retains one
    /// cause per stated entry, in the caller's order, so a two-entry preference
    /// explains both entries rather than only the last.
    #[test]
    fn a_target_that_honours_no_stated_contract_rejects_with_a_cause_per_entry() {
        let program = program();
        let mut positive_flush = StrictF32NumericalContract::governed_flush_to_zero();
        positive_flush.input_subnormals = SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        };
        positive_flush.result_subnormals = SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        };
        // The contract must still be one this build registers, or the request
        // would be refused earlier for a different reason. It is not, so this
        // asserts the earlier refusal and then drives resolution directly.
        assert!(!positive_flush.is_governed());
        assert_eq!(
            verify_request(CompilationRequest::governed_under(&program, positive_flush)),
            Err(RequestError::UnsupportedCapability {
                phase: "numerics",
                rule: "governed-contract-profile",
            })
        );

        let target = PrototypeTargetProfile::governed();
        let error = resolve_numerical_contract(
            &NumericalContractPreference::ordered(vec![
                positive_flush,
                StrictF32NumericalContract::governed(),
            ])
            .unwrap(),
            &PrototypeTargetProfile {
                // A profile that declares nothing at all: every dimension of
                // every entry is undeclared, so nothing may be admitted.
                numerical: &[],
                ..target
            },
        )
        .unwrap_err();
        let RequestError::NoResolvableNumericalContract {
            target_profile,
            rejections,
        } = error
        else {
            panic!("an unhonourable preference rejects by name");
        };
        assert_eq!(target_profile, target.key);
        assert_eq!(rejections.len(), 2, "one cause per stated entry");
        assert_eq!(rejections[0].contract_key(), positive_flush.key);
        assert_eq!(
            rejections[1].contract_key(),
            StrictF32NumericalContract::governed().key
        );
        for rejection in &rejections {
            assert!(matches!(rejection, ContractRejection::Undeclared { .. }));
            assert_eq!(
                rejection.dimension(),
                crate::honourability::NumericalDimension::InputSubnormals
            );
        }
    }

    /// The governed baseline resolves both registered contracts, and its
    /// declaration is what admits them.
    #[test]
    fn the_governed_baseline_honours_both_registered_contracts() {
        let target = PrototypeTargetProfile::governed();
        for contract in StrictF32NumericalContract::governed_profile() {
            let outcome = crate::physical::assess_contract(&target, contract).unwrap();
            let crate::feasibility::FeasibilityOutcome::Proven(evidence) = outcome else {
                panic!("the baseline honours {}", contract.key);
            };
            assert_eq!(evidence.honoured().len(), 4, "one per governed dimension");
            for honoured in evidence.honoured() {
                assert_eq!(
                    honoured.means(),
                    crate::honourability::HonouringMeans::SupportedExactly
                );
                assert_eq!(honoured.profile().key(), target.key);
            }
        }
    }

    #[test]
    fn request_requires_a_nonempty_unique_target_set() {
        let program = program();
        let mut empty = CompilationRequest::governed(&program);
        empty.target_profiles.clear();
        assert_eq!(verify_request(empty), Err(RequestError::EmptyTargetSet));

        let mut duplicate = CompilationRequest::governed(&program);
        duplicate
            .target_profiles
            .push(PrototypeTargetProfile::governed());
        assert_eq!(
            verify_request(duplicate),
            Err(RequestError::DuplicateTargetProfile)
        );
    }

    #[test]
    fn verified_request_receipts_reject_post_verification_mutation() {
        let program = program();
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
        let governed_target = PrototypeTargetProfile::governed();

        let mut forged = verified.clone();
        forged.budgets.buffers += 1;
        assert_eq!(
            forged.for_target(governed_target),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified.clone();
        forged.capabilities = CompilerCapabilitySnapshot::without_capabilities();
        assert_eq!(
            forged.for_target(governed_target),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified.clone();
        forged.target_profiles[0].max_threads_per_grid_axis -= 1;
        assert_eq!(
            forged.for_target(governed_target),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified.clone();
        forged.semantic_identity = program_with_unused_provider(7).semantic_identity().clone();
        assert_eq!(
            forged.for_target(governed_target),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified.clone();
        forged.normalized.serial_sum_mut().scale_bits = 3.0_f32.to_bits();
        assert_eq!(
            forged.for_target(governed_target),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified;
        forged.normalized.serial_sum_mut().output_key = OutputKey::new("forged").unwrap();
        assert_eq!(
            forged.for_target(governed_target),
            Err(RequestError::UnverifiedTargetSelection)
        );
    }

    #[test]
    fn verified_target_receipt_detects_every_governed_subject_mutation_class() {
        let program = program();
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
        let target = verified.for_target(verified.target_profiles[0]).unwrap();

        let mut forged = target.clone();
        forged.target_profile.max_buffer_bindings_per_entry -= 1;
        assert!(!forged.reconstructs_its_authority());

        let mut forged = target.clone();
        forged.capabilities = CompilerCapabilitySnapshot::without_capabilities();
        assert!(!forged.reconstructs_its_authority());

        let mut forged = target.clone();
        forged.budgets.regions += 1;
        assert!(!forged.reconstructs_its_authority());

        let mut forged = target.clone();
        forged.semantic_identity = program_with_unused_provider(11).semantic_identity().clone();
        assert!(!forged.reconstructs_its_authority());

        let mut forged = target.clone();
        forged.normalized.serial_sum_mut().bias_bits ^= 1;
        assert!(!forged.reconstructs_its_authority());

        let mut forged = target;
        forged.normalized.serial_sum_mut().input_key = InputKey::new("forged").unwrap();
        assert!(!forged.reconstructs_its_authority());
    }

    #[test]
    fn used_provider_revision_changes_admission_and_snapshot_subjects() {
        let first = governed_test_program(1);
        let second = governed_test_program(2);
        let first = verify_request(CompilationRequest::governed(&first)).unwrap();
        let second = verify_request(CompilationRequest::governed(&second)).unwrap();

        assert_eq!(
            first.semantic_identity.graph(),
            second.semantic_identity.graph()
        );
        assert_eq!(
            first.semantic_identity.reached_definitions(),
            second.semantic_identity.reached_definitions()
        );
        assert_ne!(
            first.semantic_identity.admission_provenance(),
            second.semantic_identity.admission_provenance()
        );
        assert_ne!(
            first.semantic_identity.registry_snapshot(),
            second.semantic_identity.registry_snapshot()
        );
    }

    #[test]
    fn unused_provider_revision_changes_only_the_snapshot_subject() {
        let first = program_with_unused_provider(1);
        let second = program_with_unused_provider(2);
        let first = verify_request(CompilationRequest::governed(&first)).unwrap();
        let second = verify_request(CompilationRequest::governed(&second)).unwrap();

        assert_eq!(
            first.semantic_identity.graph(),
            second.semantic_identity.graph()
        );
        assert_eq!(
            first.semantic_identity.reached_definitions(),
            second.semantic_identity.reached_definitions()
        );
        assert_eq!(
            first.semantic_identity.admission_provenance(),
            second.semantic_identity.admission_provenance()
        );
        assert_ne!(
            first.semantic_identity.registry_snapshot(),
            second.semantic_identity.registry_snapshot()
        );
    }
}

#[cfg(test)]
mod subject_budget {
    use super::*;

    /// Reports what the canonical explain subject is made of, byte by byte.
    ///
    /// **The decomposition is the point, not the total.** The subject is hashed
    /// once per compilation to derive the explain writer's request qualifier,
    /// byte at a time, and it is compared whenever a record's evidence is bound
    /// to its compilation — so its size is paid on the compile path rather than
    /// only when a trace is rendered. A component that is large because it
    /// *expands* something already identified by a shorter injective identity is
    /// redundant work; one that is large because it carries irreducible content
    /// is not. Only the breakdown distinguishes those two.
    #[test]
    fn the_explain_subject_byte_budget() {
        let program = super::tests::program();
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
        let target = verified
            .for_target(PrototypeTargetProfile::governed())
            .unwrap();
        let subject = target.subject();
        let identity = &subject.semantic_identity;

        let components: [(&str, usize); 4] = [
            ("semantic graph", identity.graph().as_bytes().len()),
            (
                "reached definitions",
                identity.reached_definitions().as_bytes().len(),
            ),
            (
                "admission provenance",
                identity.admission_provenance().as_bytes().len(),
            ),
            (
                "registry snapshot",
                identity.registry_snapshot().as_bytes().len(),
            ),
        ];
        let lowering = subject.lowering_registry.as_bytes().len();
        let mut numerical = 0_usize;
        for declared in subject.target_profile.numerical {
            let mut line = Vec::new();
            declared.encode_declaration(&mut line);
            numerical += line.len();
        }
        let declaration_lines = subject.target_profile.numerical.len();
        let total = subject.canonical_explain_subject_bytes().len();
        let embedded: usize = components.iter().map(|(_, size)| size).sum();

        println!("MEASURE explain subject total: {total} bytes");
        let tenths = |size: usize| size.saturating_mul(1000) / total;
        for (name, size) in components {
            let share = tenths(size);
            println!(
                "MEASURE   {name}: {size} bytes ({}.{}%)",
                share / 10,
                share % 10
            );
        }
        println!(
            "MEASURE   lowering registry identity: {lowering} bytes ({}.{}%)",
            tenths(lowering) / 10,
            tenths(lowering) % 10
        );
        {
            // Counted in the encoded bytes rather than through the registry API,
            // because the question is exactly how many times a shared value was
            // *written*, and the written form is the only place that shows.
            let registry = subject.lowering_registry.as_bytes();
            for (name, needle) in [
                ("registry snapshot", identity.registry_snapshot().as_bytes()),
                (
                    "reached definitions",
                    identity.reached_definitions().as_bytes(),
                ),
                (
                    "admission provenance",
                    identity.admission_provenance().as_bytes(),
                ),
            ] {
                let mut occurrences = 0_usize;
                let mut at = 0_usize;
                while at + needle.len() <= registry.len() {
                    if &registry[at..at + needle.len()] == needle {
                        occurrences += 1;
                        at += needle.len();
                    } else {
                        at += 1;
                    }
                }
                println!(
                    "MEASURE     {name} appears {occurrences}x in the registry identity = {} bytes",
                    occurrences * needle.len(),
                );
            }
        }
        println!(
            "MEASURE   target honourability declarations: {numerical} bytes ({}.{}%) over \
             {declaration_lines} lines",
            tenths(numerical) / 10,
            tenths(numerical) % 10
        );
        println!(
            "MEASURE   everything else (keys, shapes, budgets, contracts, framing): {} bytes",
            total - embedded - lowering - numerical,
        );
    }
}
