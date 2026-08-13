use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::sync::{Mutex, OnceLock, PoisonError};

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::index::{
    FrozenIndexRealizationLawRegistry, FrozenScalarRegistry, IndexRealizationLaw,
};
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::schedule::{
    AxisDecode, InputOrdinal, LogicalAccess, PointwiseBf16Expression,
    PointwiseBf16ExpressionBuilder, PointwiseBf16Value, PointwiseF32Expression,
    PointwiseF32ExpressionBuilder, PointwiseF32Node, PointwiseF32Value, TensorRole,
};
use tiler_ir::semantic::{
    BF16_CONSTANT_BITS_ATTRIBUTE, BROADCAST_AXIS_MAPPING_ATTRIBUTE, Bf16, BroadcastAxisMapping,
    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CanonicalIntegerWidth, CanonicalValueView,
    ContractionIndex, ContractionIndexStructure, F32, F32_CONSTANT_BITS_ATTRIBUTE, InputKey, OpKey,
    OperationAttributes, OutputKey, ProviderIdentity, REDUCTION_AXES_ATTRIBUTE,
    REINDEX_MAPPING_ATTRIBUTE, ReindexForm, ReindexFormKind, ResolvedValueType, SemanticIdentity,
    SemanticProgram, TypeKey, ValueId, add_bf16_op, add_f32_op, broadcast_f32_op, constant_bf16_op,
    constant_f32_op, multiply_bf16_op, multiply_f32_op, reindex_f32_op, silu_f32_op,
    strict_serial_sum_f32_op, strict_tensor_contraction_f32_op,
};
use tiler_ir::shape::{Axis, Extent, ExtentSources, Shape, SourcedExtent};

// The numerical-realization vocabulary is target-neutral and owned by the shared
// IR (ADR 0070); the compiler contract references it rather than duplicating it.
pub(crate) use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, MaterializationRounding,
    NumericalPermission, SubnormalMode, ValueDomainProvenance,
};
use tiler_ir::schedule::{
    Bf16NumericalContractKey, F32NumericalContractKey, NumericalContractKeyError,
};

use crate::capability::{
    CanonicalLoweringRegistryIdentity, FrozenLoweringCapabilityRegistry, LoweringCapabilityRevision,
};
use crate::elementary::{PointwiseExpressionSink, silu_point_body};
use crate::governed::{governed_lowering_capabilities, governed_scalars};
use crate::policy::UnrepresentableDimension;
use crate::region::{SemanticMemberId, SemanticStage};
use crate::target::DTypeDispatchabilityResolution;
use crate::target::honourability::{
    DeferredDimension, DimensionBehaviour, NumericalDimension, NumericalRequirement,
    UndeclaredDimension, UnhonouredDimension,
};
pub(crate) use crate::target::{TargetProfile, TargetProfileKey};

const REQUEST_SCHEMA_VERSION: u32 = 2;

/// Maximum distinct numerical contracts admitted in one preference.
///
/// **Four, and now retained rather than derived.** It used to be
/// `NumericalPolicyPreset::ALL.len()`, on the argument that a preference admits
/// no duplicate and a caller could only name a registered preset, so the longest
/// well-formed list was one entry per preset. Composition removes that ceiling
/// outright: the statable space is the dimension space, and a bound derived from
/// it would be thousands.
///
/// So the number is now a deliberate request-shape bound rather than a table's
/// length. Every stated entry is resolved against every target in the caller's
/// order and every entry enters the request subject, so the ladder is bounded to
/// keep resolution and identity work small and to keep a stated fallback
/// readable. Four is retained because it is the value the accepted public
/// boundary already carries and nothing measured asks for more; moving it is a
/// public-boundary decision rather than an implementation detail.
pub(crate) const MAX_NUMERICAL_CONTRACT_PREFERENCES: usize = 4;

/// Returns the arithmetic type one key's own contract scheme states.
///
/// This performs the complete IR-owned parse and canonicality check in each
/// governed domain. A key under no governed domain, a malformed vector, or a
/// noncanonical spelling answers `None` rather than being inferred from a
/// textual prefix.
///
/// The two domains are mutually closed — `Bf16NumericalContractKey::try_from_str`
/// refuses an `f32` key and the converse holds — so the order of the two attempts
/// is presentation and not precedence, and no key can answer twice.
pub(crate) fn contract_key_arithmetic(key: &str) -> Option<ArithmeticType> {
    if F32NumericalContractKey::try_from_str(key).is_ok() {
        Some(ArithmeticType::F32)
    } else if Bf16NumericalContractKey::try_from_str(key).is_ok() {
        Some(ArithmeticType::Bf16)
    } else {
        None
    }
}

/// Returns the element width in bytes one governed contract key's width states.
///
/// **Derived from the registered scalar catalog, not written down here.** The
/// key names an [`ArithmeticType`], that names a registered value identity, and
/// that identity's descriptor states its width in bits — so a catalog row whose
/// width moved would move this answer instead of leaving a literal disagreeing
/// with it. A key under no governed domain, or a width the catalog describes with
/// no whole-byte size, answers `None`, which is the fail-closed direction: a
/// caller must report the quantity unknown rather than continue with a
/// neighbour's width.
pub(crate) fn contract_key_element_bytes(key: &str) -> Option<u64> {
    let arithmetic = contract_key_arithmetic(key)?;
    let facts = tiler_ir::numerics::registered_arithmetic_facts(arithmetic)?;
    let (_, width_bits) = tiler_ir::numerics::registered_scalar_format(&facts)?;
    (width_bits > 0 && width_bits.is_multiple_of(8)).then_some(width_bits / 8)
}

/// Returns whether one key was minted by the current `f32` contract scheme.
///
/// Expressed through [`contract_key_arithmetic`] rather than beside it, so the
/// two cannot come to disagree about which keys the `f32` domain admits.
#[cfg(test)]
pub(crate) fn is_f32_contract_key(key: &str) -> bool {
    matches!(contract_key_arithmetic(key), Some(ArithmeticType::F32))
}

/// Every distinct contract key this process has minted.
///
/// **Why an intern table exists at all.** A scheduled region's
/// [`tiler_ir::schedule::NumericalRealization`] carries its governing contract
/// key as a `&'static str`, which is what keeps that record `Copy` and
/// `const fn`-constructible across the schedule layer's value-semantic call
/// sites. While a caller could only name one of four presets, every key was a
/// literal and the lifetime was free. A composed contract's key is a function of
/// its dimension vector, so it cannot be a literal — and it must still be
/// `'static`, because the IR record's spelling is not this crate's to change.
///
/// Interning is what reconciles the two, and its cost is bounded rather than
/// open: the statable space is finite (the product of the governed dimensions'
/// resolutions), each key is minted at most once, and a key's *content* is a
/// pure function of the vector, so nothing observable depends on which
/// invocation minted it. Only content is ever encoded; the pointer is never an
/// identity input.
static CONTRACT_KEYS: OnceLock<Mutex<BTreeSet<&'static str>>> = OnceLock::new();

/// Returns the process-lifetime `&'static str` spelling one contract key.
fn intern_contract_key(key: String) -> &'static str {
    let table = CONTRACT_KEYS.get_or_init(|| Mutex::new(BTreeSet::new()));
    // Poisoning carries no information here: the guarded value is a set of
    // immutable strings that only grows, so a panic elsewhere cannot have left
    // it half-written. Recovering the inner set keeps a poisoned lock from
    // turning every later compilation into a panic.
    let mut table = table.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(existing) = table.get(key.as_str()) {
        return existing;
    }
    let leaked: &'static str = Box::leak(key.into_boxed_str());
    table.insert(leaked);
    leaked
}

/// Renders the canonical, injective key of one resolved contract.
///
/// **Injective because the bytes it renders are.** The preimage is the same
/// exhaustive per-dimension encoding [`encode_contract`] writes into a request
/// subject: the arithmetic type's tag, the canonical arithmetic NaN bits, and
/// then, in [`crate::target::honourability::CANONICAL_DIMENSIONS`] order, each
/// dimension's own tag followed by
/// [`crate::target::honourability::DimensionBehaviour::encode`]. Every one of
/// those matches is exhaustive, so a widened behaviour space is a build error at
/// the encoder rather than a silent key collision, and the per-dimension tag
/// prefix keeps two behaviours of different dimensions from framing alike.
///
/// The key omits the contract key itself, for the obvious reason: it is what is
/// being derived. It includes the arithmetic type and the NaN bits, which
/// [`StrictF32NumericalContract::behaviour`] deliberately does not project,
/// because two contracts resolving the same dimensions for different dtypes or
/// producing different NaN patterns are different contracts (ADR 0076 item 6).
fn canonical_contract_key(
    contract: &StrictF32NumericalContract,
) -> Result<String, NumericalContractKeyError> {
    // The width selects the key domain, and the NaN payload is checked against
    // the one that width canonically produces rather than carried through. A
    // contract naming `bf16` with an `f32` NaN pattern is not a `bf16` contract
    // with an unusual field; it is a record whose two halves disagree, and
    // minting a key for it would give that disagreement a canonical identity.
    match (contract.arithmetic, contract.canonical_arithmetic_nan_bits) {
        (ArithmeticType::F32, bits)
            if bits == tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS =>
        {
            F32NumericalContractKey::new(
                contract.input_subnormals,
                contract.result_subnormals,
                contract.contraction,
                contract.reassociation,
                contract.permutation,
                contract.signed_zero,
                contract.reciprocal_transform,
                contract.approximate_intrinsics,
                contract.nan_assumptions,
                contract.infinity_assumptions,
                contract.materialization_rounding,
            )
            .map(|key| key.as_str().to_owned())
        }
        (ArithmeticType::Bf16, bits)
            if bits == u32::from(tiler_ir::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS) =>
        {
            Bf16NumericalContractKey::new(
                contract.input_subnormals,
                contract.result_subnormals,
                contract.contraction,
                contract.reassociation,
                contract.permutation,
                contract.signed_zero,
                contract.reciprocal_transform,
                contract.approximate_intrinsics,
                contract.nan_assumptions,
                contract.infinity_assumptions,
                contract.materialization_rounding,
            )
            .map(|key| key.as_str().to_owned())
        }
        // `f16` and `f64` are named by the arithmetic vocabulary and have no key
        // domain, so a contract stating one is refused here and again at
        // admission rather than compiled under a placeholder key. Widening this
        // match is what admitting a further width means.
        _ => Err(NumericalContractKeyError::InvalidArithmetic),
    }
}

/// Returns whether the request carries the program's own environment.
///
/// Compared by the inner `ShapeEnv` pointer, not by a second constructed
/// wrapper: two `ExtentSources` over one `Arc<ShapeEnv>` are one environment,
/// and two independently built environments are two even when their identities
/// happen to encode alike. That is the ambiguity
/// [`tiler_ir::index::IndexRegionBuilder::new_with_shape_environment`] exists
/// to prevent.
fn carries_program_environment(carried: Option<&ExtentSources>, program: &SemanticProgram) -> bool {
    match (carried, program.extent_sources()) {
        (None, None) => true,
        (Some(carried), Some(owned)) => std::ptr::eq(carried.environment(), owned.environment()),
        _ => false,
    }
}

/// One caller-stated resolved numerical contract, complete over every dimension.
///
/// **Complete, and complete for one arithmetic type.** Every governed dimension
/// is resolved here, because a contract that omitted one would place no
/// requirement on it, and no requirement is trivially satisfiable rather than
/// `Unknown`; and every resolution here is stated for exactly one
/// [`ArithmeticType`], because subnormal behaviour is measurably per-dtype — one
/// Apple row flushes in `f32` and preserves in `f16` — so a dtype-free contract
/// would be stating something known to be false for one of them.
///
/// The name is historical and now narrower than the type: this is the general
/// contract, and only one of the presets that build it is strict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StrictF32NumericalContract {
    pub(crate) key: &'static str,
    /// The arithmetic type every resolution below is stated for.
    pub(crate) arithmetic: ArithmeticType,
    pub(crate) canonical_arithmetic_nan_bits: u32,
    pub(crate) input_subnormals: SubnormalMode,
    pub(crate) result_subnormals: SubnormalMode,
    pub(crate) contraction: NumericalPermission,
    pub(crate) reassociation: NumericalPermission,
    /// Whether changing a reduction's logical contributor order is permitted.
    pub(crate) permutation: NumericalPermission,
    /// Whether eliminating the two signed zeros' distinction is permitted.
    pub(crate) signed_zero: NumericalPermission,
    /// Whether replacing a division by a reciprocal multiplication is permitted.
    pub(crate) reciprocal_transform: NumericalPermission,
    /// The maximum accuracy envelope approximate intrinsics may consume.
    pub(crate) approximate_intrinsics: ApproximationEnvelope,
    /// Whether NaN operands may be assumed absent, and on what evidence.
    pub(crate) nan_assumptions: ExceptionalValueAssumption,
    /// Whether infinite operands may be assumed absent, and on what evidence.
    pub(crate) infinity_assumptions: ExceptionalValueAssumption,
    /// The rounding an observable materialization boundary applies.
    pub(crate) materialization_rounding: MaterializationRounding,
}

impl StrictF32NumericalContract {
    /// Rebinds this contract's key to the canonical encoding of its dimensions.
    ///
    /// **The one place a key is minted.** Every constructor below composes a
    /// dimension vector and ends here, so a widened dimension changes the key
    /// without anyone remembering to change a string — which is exactly the
    /// failure the four hand-written names could not prevent, and the reason
    /// [`Self::is_governed`] can now check a contract against its own key
    /// instead of against a table of four.
    pub(crate) fn keyed(mut self) -> Self {
        self.key = canonical_contract_key(&self)
            .map_or(crate::policy::UNKEYED_CONTRACT, intern_contract_key);
        self
    }

    /// The strict resolution of every governed dimension, for `f32`.
    ///
    /// **The base every other contract is composed from, and the fail-closed
    /// default.** An unstated dimension resolves here, so omission never widens
    /// a contract: a caller that says nothing about reassociation has forbidden
    /// it, and a dimension added to the vocabulary later arrives forbidden in
    /// every contract that predates it rather than silently permitted.
    pub(crate) fn governed() -> Self {
        crate::policy::strict_contract(
            ArithmeticType::F32,
            tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS,
        )
        .keyed()
    }

    /// The named contract that accepts sign-preserving subnormal flushing.
    ///
    /// The vector lives on
    /// [`crate::session::NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32`], which
    /// is the one place it is spelled, and this is the internal alias the
    /// compiler's own call sites and tests use. A **different contract, not a
    /// relaxation**: its key is derived from its dimensions, so a program
    /// compiled under it has a different identity from the strict one.
    ///
    /// `#[cfg(test)]`, like its three siblings and [`Self::named_profile`]: the
    /// compile path takes whatever contract the caller stated and never mints a
    /// named one, so these exist for the crate's own tests and for the doc link
    /// above. A named contract reaching the compile path would mean some
    /// authority below the request boundary had chosen a meaning for the caller.
    #[cfg(test)]
    pub(crate) fn governed_flush_to_zero() -> Self {
        crate::session::NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32.resolve()
    }

    /// The named contract that authorizes the reshaping freedoms this build can
    /// express.
    ///
    /// Contraction, reassociation, reciprocal replacement, and approximate
    /// intrinsics within a named envelope. Subnormals stay preserved, and operand
    /// permutation, signed-zero elimination, and both exceptional-value
    /// assumptions stay refused — see
    /// [`crate::session::NumericalContract::RELAXED_F32`] for the vector, and
    /// `crate::policy::unrepresentable_dimension` for the rule that enforces
    /// representability rather than leaving it to this comment.
    #[cfg(test)]
    pub(crate) fn governed_relaxed() -> Self {
        crate::session::NumericalContract::RELAXED_F32.resolve()
    }

    /// The governed contract that authorizes ordered regrouping and nothing
    /// else.
    ///
    /// **The claim it states.** This program's results may differ from the
    /// strict reading by ordered regrouping of one same-operation operand
    /// sequence — a reduction's contributor sequence being the instance that
    /// matters here — and by nothing else. It is what a caller states to make a
    /// split reduction a legal implementation of its program without also
    /// stating that a multiply feeding an add may round once.
    ///
    /// **Every dimension, derived rather than defaulted.** The vector on
    /// [`crate::session::NumericalContract::REASSOCIATE_F32`] resolves exactly
    /// one dimension away from [`crate::policy::strict_contract`], so "this
    /// contract widens exactly one dimension" is a readable property of the
    /// code; the derivation of the other ten is here because a reader must be
    /// able to refute it:
    ///
    /// - `reassociation` — **`Permitted`**, the whole point. `physical.rs`'s
    ///   multi-pass split is refused before any region is built unless this
    ///   resolves permitted, and `ReductionTopology::MultiPass` carries
    ///   `permits_reassociation` that the schedule verifier cross-checks against
    ///   this field.
    /// - `contraction` — `Forbidden`. ADR 0015 makes it independent of
    ///   reassociation: permission to regroup an operand sequence is not
    ///   permission to fuse a multiply into an add. Forbidding it is also what
    ///   keeps the delivered realization *pinned* rather than merely authorized:
    ///   `tiler_metal::emit::realization_requirements` names
    ///   `NoFloatingPointContraction` only in the forbidden arm, so a permitting
    ///   realization places no `-ffp-contract=off` obligation on the artifact at
    ///   all, and the measured Apple row fuses a written multiply/add pair under
    ///   `-ffp-contract=fast`.
    /// - `permutation` — `Forbidden`. ADR 0014 separates it from reassociation
    ///   precisely so that one does not carry the other; a split preserves the
    ///   contributor sequence and consumes no permutation.
    /// - `input_subnormals`, `result_subnormals` — `Preserve`. Regrouping a sum
    ///   makes no claim about gradual underflow; widening either here would
    ///   state a second, unrelated meaning under one key.
    /// - `signed_zero` — `Forbidden`. A regrouped sum still distinguishes the
    ///   two zeros; nothing about the split needs them collapsed.
    /// - `reciprocal_transform`, `approximate_intrinsics` — `Forbidden` and
    ///   `ApproximationEnvelope::Forbidden`. The relaxed contract authorizes both
    ///   because it is the "every reshaping freedom this build can express"
    ///   claim; this one is the narrow one, and an authorization no operation
    ///   here consumes would still be a different stated meaning.
    /// - `nan_assumptions`, `infinity_assumptions` — `MakeNoAssumption`. A split
    ///   still canonicalizes every arithmetic NaN and still evaluates every
    ///   contributor.
    /// - `materialization_rounding` — `NearestTiesToEven`, and load-bearing
    ///   rather than incidental: the split *adds* an observable materialization
    ///   boundary — the staged partial tensor — so this is the dimension that
    ///   says the partials are stored and reloaded without a rounding change.
    ///
    /// Its own key follows from that vector, because the resolution above is a
    /// different meaning and not a setting of one of its siblings.
    #[cfg(test)]
    pub(crate) fn governed_reassociating() -> Self {
        crate::session::NumericalContract::REASSOCIATE_F32.resolve()
    }

    /// The governed contract that flushes subnormals *and* permits ordered
    /// regrouping.
    ///
    /// **The combination that composition exists for, and it is not a fifth
    /// preset.** It is the strict resolution with two independent dimensions
    /// resolved away from it — sign-preserving subnormal flushing, which the
    /// measured Apple `f32` row delivers in every math mode, and permitted
    /// ordered regrouping of one same-operation operand sequence, which every
    /// parallel reduction strategy consumes. Under the four-preset enumeration
    /// neither of the two contracts granting regrouping could accept flushing,
    /// so a parallel reduction was unstatable on the one measured Apple row and
    /// no target fact was missing.
    ///
    /// Contraction, permutation, signed-zero elimination, reciprocal
    /// replacement, approximate intrinsics, and both exceptional-value
    /// assumptions stay at their strict resolution, because widening a dimension
    /// is a statement about meaning and this contract states exactly two.
    ///
    /// Named here beside its four siblings because it is a *retained named
    /// point*, not because the space has five points: a caller may resolve any
    /// coherent vector, and these five are the ones this build documents.
    #[cfg(test)]
    pub(crate) fn governed_flush_and_reassociate() -> Self {
        crate::session::NumericalContract::FLUSH_AND_REASSOCIATE_F32.resolve()
    }

    /// Returns the named contracts this build documents.
    ///
    /// **Documentation and test population, no longer an admission authority.**
    /// While a caller could only name one of four presets, admission was
    /// membership in this set. A caller now resolves the dimensions directly, so
    /// membership would refuse every contract nobody had thought to name — the
    /// exact failure that filed this work. [`Self::is_governed`] carries
    /// admission now, and this set is what the named points are enumerated from.
    #[cfg(test)]
    pub(crate) fn named_profile() -> [Self; 5] {
        [
            Self::governed(),
            Self::governed_flush_to_zero(),
            Self::governed_relaxed(),
            Self::governed_reassociating(),
            Self::governed_flush_and_reassociate(),
        ]
    }

    /// Returns whether this contract is one this build admits.
    ///
    /// **Three conditions, and each rules out a different way a contract can be
    /// wrong.** The key must be the canonical encoding of the very dimensions
    /// beside it, so a record whose key and vector disagree — a mutated field, a
    /// key carried across a widening — is refused rather than compiled under a
    /// name that no longer describes it. The vector must be coherent, so a
    /// self-contradictory contract cannot reach a plan through an internal
    /// construction site that bypassed the public builder. And every dimension
    /// must be one this build can realize, so a contract whose meaning no
    /// scheduled region can record is refused before it gives two meanings one
    /// identity.
    ///
    /// The first condition is also what admits a *width*: a key is minted only
    /// for an arithmetic type this build states contracts in, so a contract
    /// naming `f16` or `f64` fails here rather than reaching a target that would
    /// have to report a missing declaration for a width no caller may state.
    ///
    /// Membership in a table of four is deliberately *not* among them: that test
    /// is what made an unnamed corner unreachable.
    pub(crate) fn is_governed(&self) -> bool {
        canonical_contract_key(self).is_ok_and(|key| self.key == key)
            && coherence(self).is_ok()
            && crate::policy::unrepresentable_dimension(self).is_none()
    }

    /// This contract's resolution of one governed dimension.
    ///
    /// Exhaustive over the dimension vocabulary, so a new dimension is a build
    /// error here rather than a field the contract silently fails to project.
    /// `key`, `arithmetic`, and `canonical_arithmetic_nan_bits` are deliberately
    /// not reachable through it: the first names the governing contract, the
    /// second keys every resolution, and the third is a produced value, and none
    /// is a behaviour a target declares honourability for. Letting the key stand
    /// in for the dimensions it names is exactly the projection ADR 0076 item 6
    /// forbids.
    pub(crate) const fn behaviour(&self, dimension: NumericalDimension) -> DimensionBehaviour {
        match dimension {
            NumericalDimension::InputSubnormals => {
                DimensionBehaviour::Subnormals(self.input_subnormals)
            }
            NumericalDimension::ResultSubnormals => {
                DimensionBehaviour::Subnormals(self.result_subnormals)
            }
            NumericalDimension::Contraction => DimensionBehaviour::Transform(self.contraction),
            NumericalDimension::Reassociation => DimensionBehaviour::Transform(self.reassociation),
            NumericalDimension::Permutation => DimensionBehaviour::Transform(self.permutation),
            NumericalDimension::SignedZero => DimensionBehaviour::Transform(self.signed_zero),
            NumericalDimension::ReciprocalTransform => {
                DimensionBehaviour::Transform(self.reciprocal_transform)
            }
            NumericalDimension::ApproximateIntrinsics => {
                DimensionBehaviour::Approximation(self.approximate_intrinsics)
            }
            NumericalDimension::NanAssumptions => {
                DimensionBehaviour::ExceptionalValue(self.nan_assumptions)
            }
            NumericalDimension::InfinityAssumptions => {
                DimensionBehaviour::ExceptionalValue(self.infinity_assumptions)
            }
            NumericalDimension::MaterializationRounding => {
                DimensionBehaviour::Rounding(self.materialization_rounding)
            }
        }
    }

    /// Projects this contract into the per-dimension requirements a target
    /// profile's honourability declaration is assessed against.
    ///
    /// Delegated to [`crate::policy::dimension_requirements`], which owns the
    /// rule deciding which dimensions place an obligation on a target at all.
    pub(crate) fn dimension_requirements(&self) -> Vec<NumericalRequirement> {
        crate::policy::dimension_requirements(self)
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
            self.permutation,
            self.signed_zero,
            self.nan_assumptions,
            self.infinity_assumptions,
        )
    }

    /// Whether this contract lets two realizations of one occurrence differ.
    ///
    /// The partition search asks exactly this before it may compute one semantic
    /// occurrence in two regions: recomputation preserves the program only if
    /// every admitted realization of that occurrence computes the same function
    /// bit for bit. A contract that *permits* a transform does not say which
    /// realization takes it, so two regions may take it differently, and the two
    /// copies of one semantic value would then disagree — which is a change of
    /// meaning, not a cost.
    ///
    /// **The distinction is freedom, not strictness.** A fixed resolution binds
    /// every realization equally: both subnormal modes, the canonical NaN
    /// payload, and the materialization rounding are *decided* by the contract,
    /// so two realizations under one contract cannot differ on them however
    /// permissive the decision is. They are deliberately absent below. The
    /// dimensions listed are the ones whose resolution admits *alternatives* a
    /// realization chooses between.
    ///
    /// Written as an exhaustive destructuring so a dimension added to the
    /// vocabulary is a build error here rather than a freedom this predicate
    /// silently declares absent.
    pub(crate) const fn grants_realization_freedom(&self) -> bool {
        let Self {
            key: _,
            arithmetic: _,
            canonical_arithmetic_nan_bits: _,
            input_subnormals: _,
            result_subnormals: _,
            contraction,
            reassociation,
            permutation,
            signed_zero,
            reciprocal_transform,
            approximate_intrinsics,
            nan_assumptions,
            infinity_assumptions,
            materialization_rounding: _,
        } = *self;
        !matches!(contraction, NumericalPermission::Forbidden)
            || !matches!(reassociation, NumericalPermission::Forbidden)
            || !matches!(permutation, NumericalPermission::Forbidden)
            || !matches!(signed_zero, NumericalPermission::Forbidden)
            || !matches!(reciprocal_transform, NumericalPermission::Forbidden)
            || !matches!(approximate_intrinsics, ApproximationEnvelope::Forbidden)
            // An assumption is a freedom: a realization told NaN operands are
            // absent may emit an operation that differs from the general one on
            // an input the assumption excluded, and a realization that declines
            // the assumption may not. Two copies of one value would then be two
            // different computations.
            || !matches!(
                nan_assumptions,
                ExceptionalValueAssumption::MakeNoAssumption
            )
            || !matches!(
                infinity_assumptions,
                ExceptionalValueAssumption::MakeNoAssumption
            )
    }
}

/// Why one composed dimension vector is not a contract this build will hold.
///
/// **Enumerated, not discovered.** A composed contract can state combinations a
/// four-preset enumeration could not, so the combinations that are *not*
/// contracts have to be named before a caller finds one. The enumeration below
/// is deliberately small, and the eliminations matter as much as the survivor —
/// a reader must be able to refute the list rather than only read it.
///
/// **What survives.** Exactly one: a contract may not assert a value-domain
/// absence on evidence it is not the author of.
/// `docs/numerical-semantics.md`'s value-assumption section defines
/// compiler-proven as "derived soundly from verified producers, constants, or
/// analysis" and runtime-validated as "established by a guard or validation
/// computation before any plan that relies on it executes". Neither is a claim a
/// caller is in a position to make: the first is a conclusion this compiler
/// reaches, and the second names a guard this build neither emits nor checks. A
/// caller-stated `AssumeAbsent` therefore carries
/// [`ValueDomainProvenance::CallerDeclaredUnvalidated`] or it is asserting
/// somebody else's evidence — which is the same failure the same document's own
/// example names, that replacing `x / x` with `1` "requires more than a caller's
/// unchecked claim".
///
/// **What was eliminated, and why each elimination holds.**
///
/// - *NaN absence against the canonical arithmetic NaN pattern.* Not
///   contradictory: the pattern governs a NaN this build *produces*, the
///   assumption governs a NaN an *operand* may carry, and
///   [`ExceptionalValueAssumption`]'s own definition keeps the two apart.
/// - *NaN absence against infinity presence, in either direction.* ADR 0011
///   makes every permission independent; one never implies another, and refusing
///   the pair would re-couple two dimensions a decision separated.
/// - *Permitted signed-zero elimination against a sign-preserving flush.* The
///   flush's zero sign is carried on the behaviour precisely so that no
///   permission can leave it unspecified —
///   [`tiler_ir::schedule::FlushedZeroSign`] says so — so the two are
///   independent by construction rather than in tension.
/// - *A flush to always-positive zero against forbidden signed-zero
///   elimination.* A declared flush behaviour is a stated, checkable result, not
///   a rewrite; forbidding the *elimination* of a distinction does not forbid an
///   operation whose defined result happens to produce one zero.
/// - *Permitted contraction against forbidden reassociation, and permitted
///   permutation against forbidden reassociation.* ADR 0015 separates fusing a
///   multiply into an add from regrouping an operand sequence, and ADR 0014
///   separates permuting a reduction's contributors from regrouping them: a
///   permuted sequence folded strictly left to right is a well-defined sum that
///   consumes no regrouping at all.
///
/// **What this is not.** A contract this build cannot *realize* is a different
/// refusal with a different owner —
/// [`RequestError::UnrepresentableNumericalDimension`] names the dimension, the
/// behaviour the build realizes, and the operation that would consume it — and a
/// contract a *target* cannot honour is a third, named per dimension by
/// feasibility. Coherence is about the statement alone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IncoherentContract {
    /// A stated absence claims provenance the caller cannot be the author of.
    UnfoundedValueDomainProvenance {
        /// The exceptional-value dimension the absence was stated on.
        dimension: ExceptionalValueDimensionKind,
        /// The provenance class the contract asserted.
        provenance: ValueDomainProvenance,
    },
}

/// Which exceptional value an absence was stated about.
///
/// Narrower than [`NumericalDimension`] deliberately: only two dimensions carry
/// an [`ExceptionalValueAssumption`], so a refusal that could name any of the
/// eleven would force every consumer to handle nine cases it can never see.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExceptionalValueDimensionKind {
    /// NaN operands.
    Nan,
    /// Infinite operands.
    Infinity,
}

impl ExceptionalValueDimensionKind {
    /// The stable diagnostic key naming this dimension.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Nan => NumericalDimension::NanAssumptions.key(),
            Self::Infinity => NumericalDimension::InfinityAssumptions.key(),
        }
    }
}

impl fmt::Display for IncoherentContract {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnfoundedValueDomainProvenance {
                dimension,
                provenance,
            } => write!(
                formatter,
                "{} states an absence on {} provenance, which a caller is not the author of; \
                 a caller-stated absence carries caller-declared-unvalidated provenance",
                dimension.key(),
                match provenance {
                    ValueDomainProvenance::CompilerProven => "compiler-proven",
                    ValueDomainProvenance::RuntimeValidated => "runtime-validated",
                    ValueDomainProvenance::CallerDeclaredUnvalidated =>
                        "caller-declared-unvalidated",
                }
            ),
        }
    }
}

impl Error for IncoherentContract {}

/// Returns the first incoherence in one composed dimension vector.
///
/// The two exceptional-value dimensions are walked in canonical order, so the
/// reported cause is a function of the contract rather than of iteration order,
/// and the provenance match is exhaustive rather than a negated comparison, so a
/// widened [`ValueDomainProvenance`] is a build error here instead of arriving
/// classified with whichever arm a wildcard covered.
///
/// It reads the two fields directly rather than projecting through
/// [`StrictF32NumericalContract::behaviour`]: only those two carry an
/// [`ExceptionalValueAssumption`], and a walk over every dimension would have to
/// skip nine behaviours it can say nothing about.
pub(crate) fn coherence(contract: &StrictF32NumericalContract) -> Result<(), IncoherentContract> {
    for (dimension, assumption) in [
        (ExceptionalValueDimensionKind::Nan, contract.nan_assumptions),
        (
            ExceptionalValueDimensionKind::Infinity,
            contract.infinity_assumptions,
        ),
    ] {
        let ExceptionalValueAssumption::AssumeAbsent { provenance } = assumption else {
            continue;
        };
        match provenance {
            ValueDomainProvenance::CallerDeclaredUnvalidated => {}
            ValueDomainProvenance::CompilerProven | ValueDomainProvenance::RuntimeValidated => {
                return Err(IncoherentContract::UnfoundedValueDomainProvenance {
                    dimension,
                    provenance,
                });
            }
        }
    }
    Ok(())
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
    /// Returns a typed request error for an empty, duplicate, or oversized list.
    /// There is no default and no implicit strictest reading: a request that
    /// states no contract does not compile, and the diagnostic says the contract
    /// is unstated rather than naming a dimension.
    pub(crate) fn ordered(stated: Vec<StrictF32NumericalContract>) -> Result<Self, RequestError> {
        if stated.is_empty() {
            return Err(RequestError::UnstatedNumericalContract);
        }
        if stated.len() > MAX_NUMERICAL_CONTRACT_PREFERENCES {
            return Err(RequestError::TooManyNumericalContracts {
                actual: stated.len(),
                max: MAX_NUMERICAL_CONTRACT_PREFERENCES,
            });
        }
        if stated
            .iter()
            .enumerate()
            .any(|(index, contract)| stated[..index].contains(contract))
        {
            return Err(RequestError::DuplicateNumericalContract);
        }
        Ok(Self { stated })
    }

    /// The stated contracts, in the caller's order.
    pub(crate) fn stated(&self) -> &[StrictF32NumericalContract] {
        &self.stated
    }
}

/// What a budget refusal says about the space on the far side of the bound.
///
/// A caller acting on an exhausted budget needs to know whether the compiler
/// finished measuring the quantity it refused or stopped counting partway, and
/// the two imply different actions. The distinction is not a nicety: the demand
/// a truncating stop reports is a *lower bound* on what it did not explore, so
/// reading it as a required size would be wrong in the silent direction.
///
/// # Not `#[non_exhaustive]`
///
/// ADR 0074 convention 5a marks a public enum whose variant set is a
/// bounded-profile placeholder. This is not one: a bound either finished
/// measuring its subject or it did not, and there is no third answer for a
/// later budget to occupy. Marking it would oblige every out-of-crate consumer
/// to carry a wildcard arm over a closed two-way split, which is the cost 5a
/// exists to avoid paying for nothing.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BudgetRefusal {
    /// The budget bounds a quantity the compiler measured, and the limit
    /// refused that exact quantity.
    ///
    /// The reported demand is a number computed in full and then compared
    /// against the limit — a submitted program's own counts, or a refused
    /// region candidate's members, retained outputs, or live values. Where one
    /// resource refused several candidates, it is the largest of those exact
    /// counts.
    ///
    /// No additional search reaches a plan under this limit, so the caller's
    /// action is to widen the bound or to submit a smaller program.
    Bounding,
    /// The budget stopped a search before that search finished.
    ///
    /// The reported demand is the first demand the limit refused, which is a
    /// lower bound on the space left unexplored rather than that space's size.
    /// A wider limit may reach a plan this compilation never saw, and may
    /// equally find nothing.
    Truncated,
}

/// Which deterministic budget refused a compilation.
///
/// Four authorities raise a budget refusal and each owns its own stop record.
/// They are named here as plain text because every one of them is crate-private
/// and a public doc cannot link a private item: `request::check_program_budgets`
/// refuses a submitted program's own size before any target is consulted, and
/// `region::RegionBudgetResource`, `cover::CoverBudgetResource`, and
/// `selection::PlanBudgetResource` bound the three searches that run once one
/// has been. Those records stay distinct because their surrounding
/// data differs — a plan stop also names the cover whose enumeration it
/// stopped. This is the single vocabulary they all name a resource in, so a
/// caller reads one closed set rather than four, and each authority maps into
/// it through a total `const fn` that a new internal budget must extend before
/// it compiles.
///
/// [`Self::key`] is the stable diagnostic key, and it is the sole authority for
/// these strings: the per-authority accessors delegate here rather than
/// repeating a table that could drift.
///
/// # Which of these a public caller can actually observe
///
/// Only the five program-scoped resources today. The other eight are raised by
/// the three searches, which reach a caller only through an empty portfolio,
/// and `crate::session`'s reachability note records why that route is currently
/// unreachable from the public surface: the region-shape bounds are now the
/// same formulas as the program-scoped bounds they derive from, so a program
/// large enough to truncate a search is refused for its size first. Reaching it
/// needs a caller-stated budget set, which the public surface does not admit.
/// The vocabulary is nevertheless complete, because the mapping into it must be
/// total over what the compiler can raise and reachability is a property of the
/// budgets a request carries rather than of this type.
///
/// # Why `#[non_exhaustive]`
///
/// ADR 0074 convention 5's clause test asks what an out-of-crate wildcard arm
/// would have to do. No consumer outside this crate maps this vocabulary onto a
/// derived value it must get right per variant, and none matches it to decide
/// what it supports; a consumer renders it, forwards it, or classifies it
/// partially. That is clause 5a, so the attribute applies and a later budget
/// lands additively.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum BudgetResource {
    /// Values a submitted program may declare and produce.
    SemanticValues,
    /// Semantic occurrences a submitted program may declare.
    SemanticOperations,
    /// Dispatch regions the widest plan for a submitted program may assemble.
    Regions,
    /// Host expression nodes the widest plan for a submitted program may spell.
    HostExpressionNodes,
    /// Buffers the widest plan for a submitted program may bind.
    Buffers,
    /// Semantic occurrences admitted in one region candidate.
    RegionMembers,
    /// Retained boundary outputs admitted for one region candidate.
    RegionBoundaryOutputs,
    /// Boundary and member-result values live across one region candidate.
    RegionLiveValues,
    /// Grown candidates admitted for one seed occurrence.
    RegionCandidatesPerSeed,
    /// Candidate expansion attempts admitted for one compilation request.
    RegionExpansions,
    /// Distinct legal complete covers retained for one enumeration request.
    RegionCovers,
    /// Partition-search expansion attempts for one enumeration request.
    RegionCoverExpansions,
    /// Complete-plan combinations admitted for one cover source.
    PhysicalPlanCombinations,
}

impl BudgetResource {
    /// Every budget resource, sized from the type.
    ///
    /// `variant_count` is what makes a widened vocabulary a build error here
    /// rather than a census that silently shrinks while still reporting no
    /// duplicate key. A hand-written length would be satisfied by a list that
    /// had stopped covering its own enum.
    ///
    /// Test-only, so the nightly feature it needs stays out of a normal build.
    #[cfg(test)]
    pub(crate) const ALL: [Self; std::mem::variant_count::<Self>()] = [
        Self::SemanticValues,
        Self::SemanticOperations,
        Self::Regions,
        Self::HostExpressionNodes,
        Self::Buffers,
        Self::RegionMembers,
        Self::RegionBoundaryOutputs,
        Self::RegionLiveValues,
        Self::RegionCandidatesPerSeed,
        Self::RegionExpansions,
        Self::RegionCovers,
        Self::RegionCoverExpansions,
        Self::PhysicalPlanCombinations,
    ];

    /// Returns the stable diagnostic key of this budget.
    ///
    /// The key is meaning rather than presentation: it is the rule key a
    /// request refusal reports, the resource key an explain record carries, and
    /// part of the reason code a failure detail spells, so it is compared. ADR
    /// 0074 convention 2's correction is what decides the spelling — `key` is
    /// reserved for a stable semantic key and `label` for a presentation digest.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::SemanticValues => "semantic-values",
            Self::SemanticOperations => "semantic-operations",
            Self::Regions => "regions",
            Self::HostExpressionNodes => "host-expression-nodes",
            Self::Buffers => "buffers",
            Self::RegionMembers => "region-members",
            Self::RegionBoundaryOutputs => "region-boundary-outputs",
            Self::RegionLiveValues => "region-live-values",
            Self::RegionCandidatesPerSeed => "region-candidates-per-seed",
            Self::RegionExpansions => "region-expansions",
            Self::RegionCovers => "region-covers",
            Self::RegionCoverExpansions => "region-cover-expansions",
            Self::PhysicalPlanCombinations => "physical-plan-combinations",
        }
    }

    /// Returns what a refusal on this budget says about the space beyond it.
    ///
    /// The split is the one `DeterministicBudgets` already draws in prose and is
    /// derived from where each stop is recorded. The five program-scoped bounds
    /// and the three region-*shape* bounds compare a demand the compiler has
    /// finished computing — a program's counts, a candidate's members, retained
    /// outputs, or live values — so their demand is exact. The five search
    /// bounds stop an enumeration at the first demand they refuse, and all three
    /// stop records say so in their own documentation: the value is "a lower
    /// bound on the unexplored space rather than its size".
    ///
    /// This is the answer a `&'static str` resource could not give. A caller
    /// holding a key has no way to learn whether the number beside it is a size
    /// or a floor without reading compiler source, which is the reading this
    /// whole surface exists to remove.
    #[must_use]
    pub const fn refusal(self) -> BudgetRefusal {
        match self {
            Self::SemanticValues
            | Self::SemanticOperations
            | Self::Regions
            | Self::HostExpressionNodes
            | Self::Buffers
            | Self::RegionMembers
            | Self::RegionBoundaryOutputs
            | Self::RegionLiveValues => BudgetRefusal::Bounding,
            Self::RegionCandidatesPerSeed
            | Self::RegionExpansions
            | Self::RegionCovers
            | Self::RegionCoverExpansions
            | Self::PhysicalPlanCombinations => BudgetRefusal::Truncated,
        }
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
    ///
    /// **This and the two below bound a region's admissible *shape* rather
    /// than a search, and each of them can refuse a program.** They declare the
    /// largest region this profile will form at all, so a program whose only
    /// implementable cover needs a bigger one has no plan under them however
    /// long the search runs — a refusal reported as `BudgetExhausted` naming
    /// the bound, because the caller's action is to widen it. The two search
    /// bounds below carry the opposite guarantee.
    ///
    /// Because they can refuse, all three are **derivations over the
    /// declaration** rather than stated numbers: a region is a subset of the
    /// program it covers, so [`Self::governed`] derives this one from
    /// `semantic_operations`, `region_live_values` from `semantic_values`, and
    /// `region_boundary_outputs` from the declared output count.
    pub(crate) region_members: u32,
    /// Retained boundary outputs admitted for one region candidate.
    pub(crate) region_boundary_outputs: u32,
    /// Boundary and member-result values live across one region candidate.
    pub(crate) region_live_values: u32,
    /// Grown candidates admitted for one seed occurrence.
    ///
    /// Both coverage extremes — every singleton region and the whole-program
    /// region — are emitted before growth starts and neither is bounded by this
    /// budget, so exhausting it loses the partitions discovered between them
    /// rather than either end.
    pub(crate) region_candidates_per_seed: u32,
    /// Candidate expansion attempts admitted for one compilation request.
    ///
    /// Bounds the same discovered space as `region_candidates_per_seed` and
    /// carries the same guarantee, for the same reason: coverage precedes
    /// growth. It did not before
    /// `region-expansion-exhaustion-loses-the-only-feasible-plan`, and the
    /// consequence was not academic — growth reaches the whole-program
    /// candidate last, so a twelve-operation chain exhausted this bound before
    /// forming the one region the profile could implement and the compilation
    /// refused.
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
    /// The bounded profile's deterministic budgets.
    ///
    /// **The five program-scoped bounds are sized to the complete decoder-layer
    /// program, which is the largest program shape this profile may be asked to
    /// admit.** Each is derived from that program's own measured counts rather
    /// than from the smallest number that lets it through, which is the rule
    /// [`check_program_budgets`] states and the rule the split reduction's
    /// earlier widenings followed. The counts are the two rows the layer was
    /// verified and reference-evaluated at: eighteen declared inputs and three
    /// ordered named outputs at both, fifty-eight occurrences over seventy-six
    /// values at the C1 prefill row, and sixty-two over eighty at the C1 decode
    /// row. The decode row is the binding one, and it is larger for a reason
    /// that is not the cache: at `T = 1` six position-axis rank pads duplicate
    /// nothing, so the broadcast family refuses a many-to-one relation onto an
    /// extent-one result axis and the layer spells those widenings as further
    /// occurrences.
    ///
    /// - `semantic_values` is `80`: the decode row's eighteen declared inputs
    ///   plus one result per occurrence, because no occurrence in the layer
    ///   produces more than one value. The prefill row is `18 + 58 = 76` by the
    ///   same arithmetic, so eighty bounds both.
    /// - `semantic_operations` is `62`: the decode row's occurrence count.
    /// - `regions` is `12`: [`check_program_budgets`] derives the actual as four
    ///   dispatches per declared output — the widest producer chain one output
    ///   can reach, prologue, partial, final, and epilogue — so three outputs
    ///   reach `3 × 4`.
    /// - `host_expression_nodes` is `51`: the same function derives the actual
    ///   as two nodes per declared input, four per declared output, and three
    ///   program-scoped nodes, so `2 × 18 + 4 × 3 + 3`.
    /// - `buffers` is `30`: the actual is every declared input plus four per
    ///   declared output — the prologue's temporary, a split's staged partial
    ///   tensor, the fold's staged result an epilogue reads, and the output — so
    ///   `18 + 4 × 3`. It was `3`, then `4`, then `6`, then `21` — the one-input
    ///   materialized program's input, temporary and output; the split's staged
    ///   partial tensor; that split over the widest three-input prologue the
    ///   governed target's four buffer bindings admit; and the eighteen-input
    ///   layer under a one-output derivation — and every step, including this
    ///   one, is the same derivation over a wider admitted program.
    ///
    /// **The three region-shape bounds are derived from those five rather than
    /// picked, because a region is a subset of the program it covers.** Its
    /// members are a subset of the program's occurrences, its live values are
    /// disjoint subsets of the program's values, and the largest of them — the
    /// whole-program region — exports exactly what the program declares. So
    /// each is a formula over the same declaration, on exactly the ground
    /// `regions` was corrected on below: a quantity that belongs to a *plan* is
    /// still a function of the declaration the plan covers.
    ///
    /// - `region_members` is `62`: `semantic_operations`, because a region's
    ///   members are a subset of the program's own occurrences and a program
    ///   admitted at all holds no more than that many. It is the program-scoped
    ///   bound itself; see the collapse note below for why the field is still
    ///   encoded.
    /// - `region_boundary_outputs` is `3`: the declared output count, which is
    ///   the same count `regions` multiplies by four. The whole-program region
    ///   exports one value per *named* result and nothing else, because no
    ///   occurrence outside it consumes anything, so the largest region this
    ///   profile forms exports exactly the declaration's ordered named outputs.
    /// - `region_live_values` is `80`: `semantic_values`, because a region's
    ///   live values are its boundary inputs and its members' results, which
    ///   are disjoint subsets of the program's own values. It is tight at the
    ///   whole-program region, whose boundary inputs are the eighteen declared
    ///   inputs — every other value it reads it also produces — and whose
    ///   member results are one per occurrence: the same `18 + 62`
    ///   `semantic_values` is.
    ///
    /// The three program bounds derived through [`check_program_budgets`] —
    /// `regions`, `host_expression_nodes`, and `buffers` — are tight at exactly
    /// eighteen declared inputs and three declared outputs, so their thresholds
    /// coincide along each axis:
    /// a nineteen-input program exceeds two of them at once and the earlier
    /// check, `host-expression-nodes`, is the one that reports, while a
    /// four-output program exceeds all three and `regions` reports.
    ///
    /// **`regions` was `4` and was checked against a constant rather than
    /// derived**, on the ground that a region count is a property of a *plan*
    /// and this profile plans no decoder layer: [`select_supported_strategy`]
    /// refuses it under its own named rules, which is a separate refusal with a
    /// separate remedy. That ground survives and its conclusion did not. A plan
    /// covers every declared output, so the plan-scoped constant is still a
    /// function of the *declaration* — one widest chain per ordered named
    /// output — and while recognition could name only one output the two were
    /// indistinguishable. Since multi-output admission they are not: two
    /// independent chains assemble seven or eight dispatches, so the literal
    /// bounded nothing and the program it refuses is the one it was written to
    /// refuse.
    ///
    /// The four in the per-output derivation is the measured stage count of the
    /// widest chain, taken from
    /// `crate::pipeline::tests::the_widest_assembled_plan_is_the_split_reduction_with_its_epilogue`,
    /// whose reassociation-forbidding neighbour is what attributes the fourth
    /// stage to the split rather than to the epilogue alone.
    ///
    /// The consequence of every one of these moves is the one this comment
    /// already records: every budget is written into the request subject, so
    /// every governed compilation's qualifier moved with them. The one pinned
    /// literal is `explain`'s
    /// `deterministic_trace_is_sealed_and_rendered_separately` request qualifier
    /// — and its ledger comment records the recomputation. No encoding version
    /// moved: the field set, widths, and order are untouched, so a value change
    /// stays injective inside the current `tiler.compiler.request-subject.v6`
    /// domain.
    ///
    /// They move again when the decoder layer becomes plannable, and that is a
    /// second identity move this one cannot honestly absorb. The three
    /// region-shape bounds move *with* them and never on their own account,
    /// which is what the derivation buys: a ceiling somebody has to raise per
    /// program is replaced by a formula that tracks the declaration.
    ///
    /// `normalization_rewrites`, `region_candidates_per_seed`, and
    /// `region_expansions` are unchanged, and the ground for leaving them alone
    /// survives intact: none of the three admits or refuses a program, because
    /// each bounds a *search* whose alternatives sit between two coverage
    /// extremes region formation emits unconditionally, so exhausting one costs
    /// an alternative while the verified input and complete coverage survive.
    ///
    /// That ground was stated for all six `region_*` bounds and was **half
    /// right**: the three shape bounds declare the largest region this profile
    /// forms, so a program whose only implementable cover needs a bigger one is
    /// refused by them however long the search runs. While `region_members` was
    /// the bare constant `32`, that refusal was measurable and measured: a
    /// shared-constant `f32` multiply chain's recognized partition is its whole
    /// program and nothing smaller is implementable, so 33..=62 occurrences
    /// refused `BudgetExhausted` on `region_members` although every bound on the
    /// program's own *size* admitted them. The derivations above dissolve that:
    /// the stated admission envelope and the actual planning envelope are now
    /// the same formulas over one declaration rather than two disagreeing
    /// ceilings.
    ///
    /// **Two of the three collapse onto the program-scoped bound they derive
    /// from, and that is the derivation's answer rather than a defect in it.**
    /// `region_members` *is* `semantic_operations` and `region_live_values`
    /// *is* `semantic_values`, so for a program whose occurrences are each
    /// realized by one region neither can fire: `check_program_budgets` has
    /// already refused anything with more occurrences or values than the region
    /// bound would. They are still encoded, for two reasons. The first is that
    /// region formation's attribution atom is a realization *stage* rather than
    /// an occurrence and its live values include the intermediates a staged law
    /// hands between stages — neither is a value the program's own occurrence
    /// and value counts hold — so both bounds still bind on a program whose
    /// families realize region sequences. The second is that a budget set is a
    /// *request* field: these bound one region's shape for any budget policy,
    /// and the governed profile's coincidence is a property of its declaration
    /// rather than of the fields. Tom accepted on 2026-08-11 that both keep
    /// their slots in the canonical request subject. Omitting them would make
    /// distinct staged-region policies share one request/evidence subject,
    /// while the measured saving is eight bytes. `region_boundary_outputs`
    /// does not collapse: it is the declared output count rather than any
    /// program-scoped bound, and it still refuses a grown candidate that would
    /// export more values than the program names.
    ///
    /// Every value here is a *deliberate* decision and not a test-enabling
    /// edit, because every one of these numbers is inside the canonical request
    /// subject ([`VerifiedRequestSubject::canonical_explain_subject_bytes`]
    /// writes every budget). Every governed compilation's request/evidence
    /// subject moves with such a change — for programs nowhere near any bound
    /// as much as for ones at it — because a budget is a property of the
    /// compilation request rather than of the plan chosen for it. The request
    /// subject is not artifact or cache identity; those move only when the
    /// selected packaged content moves. The one checked-in literal derived from
    /// these bytes is `explain`'s
    /// `deterministic_trace_is_sealed_and_rendered_separately` request
    /// qualifier, whose ledger comment records the recomputation. No encoding
    /// version moved with it — the field set, widths, and order are untouched,
    /// so a value change stays injective inside the current
    /// `tiler.compiler.request-subject.v6` domain.
    ///
    /// A budget is an upper bound, so widening admits program shapes and never
    /// requires them: [`check_program_budgets`] still refuses a program one step
    /// past each of these, and `verify_host_contract` still refuses a built
    /// program whose expression, value, or stage count exceeds
    /// `host_expression_nodes`, `buffers`, or `regions`. The same holds of the
    /// three derived region bounds one layer down:
    /// [`crate::region::RegionBudgetResource`] still stops a candidate one step
    /// past each of `region_members`, `region_boundary_outputs`, and
    /// `region_live_values`, and the stop is still reported as a typed
    /// `BudgetStop` naming the resource rather than dropped. Nor does clearing
    /// the budget gate compile a decoder layer — the recognizer's refusal is
    /// untouched, and what these values remove is only the refusal that was
    /// about *size*.
    pub(crate) const fn governed() -> Self {
        Self {
            semantic_values: 80,
            semantic_operations: 62,
            regions: 12,
            host_expression_nodes: 51,
            buffers: 30,
            normalization_rewrites: 8,
            region_members: 62,
            region_boundary_outputs: 3,
            region_live_values: 80,
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
    pub(crate) fn new(
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
        .expect("the governed scalar registry retains its exact semantic authority")
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

#[derive(Clone, Debug)]
pub(crate) struct CompilationRequest<'a> {
    pub(crate) program: &'a SemanticProgram,
    /// The program's own environment, never a second caller-supplied one.
    ///
    /// `None` when the program has only literal extents. Two environments over
    /// one program is the ambiguity
    /// [`tiler_ir::index::IndexRegionBuilder::new_with_shape_environment`]
    /// exists to prevent; [`verify_request`] refuses a request that does not
    /// carry this exact environment.
    pub(crate) shape_environment: Option<&'a ExtentSources>,
    /// The caller's ordered numerical-contract preference. Required, with no
    /// `Default` and no ambient fallback (ADR 0076 item 2).
    pub(crate) numerical_contracts: NumericalContractPreference,
    pub(crate) budgets: DeterministicBudgets,
    pub(crate) target_profiles: Vec<TargetProfile>,
    pub(crate) capabilities: CompilerCapabilitySnapshot,
}

impl CompilationRequest<'_> {
    /// Builds the fixed governed compilation-request fixture.
    ///
    /// Conformance and unit tests use this exact combination of the program's
    /// own shape environment, strict-`f32` numerical contract, deterministic
    /// budgets, target profile, and installed lowering capabilities. Production
    /// resolves the public caller's stated preference through
    /// [`Self::governed_preferring`].
    #[allow(
        dead_code,
        reason = "crate-internal fixed governed fixture used by conformance and unit tests; the public CompileRequest path resolves caller preferences through governed_preferring until a production caller needs this exact fixture"
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
            shape_environment: program.extent_sources(),
            numerical_contracts,
            budgets: DeterministicBudgets::governed(),
            target_profiles: vec![TargetProfile::governed()],
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
/// identity may store the prologue's constants in either order, and the
/// recognized coverage must not depend on which spelling the caller authored. A
/// shared constant simply contributes one member instead of two.
///
/// **The prologue set is empty exactly when the fold has no prologue.** A
/// reduction whose contributor tensor is a declared input — `sum(x)` — claims one
/// occurrence and needs one region, so its partition has one part and the empty
/// part is not a member set any cover region may match. That is a fact about the
/// program rather than a degenerate case: [`NormalizedSerialSum::prologue`] is
/// `None` for it, and every derivation that would spell a prologue region reads
/// the option rather than the emptiness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecognizedSerialSumMembers {
    pointwise: Vec<SemanticStage>,
    reduction: Vec<SemanticStage>,
}

impl RecognizedSerialSumMembers {
    /// Binds the recognized prologue's occurrences and the reduction's own.
    fn new(pointwise: Vec<SemanticStage>, reduction: u32) -> Self {
        let mut pointwise = pointwise;
        pointwise.sort_unstable();
        pointwise.dedup();
        Self {
            pointwise,
            reduction: vec![SemanticStage::first(SemanticMemberId(reduction))],
        }
    }

    /// Returns the pointwise prologue members in ascending order.
    pub(crate) fn pointwise(&self) -> &[SemanticStage] {
        &self.pointwise
    }

    /// Returns the reduction members in ascending order.
    pub(crate) fn reduction(&self) -> &[SemanticStage] {
        &self.reduction
    }

    /// Returns every recognized member in ascending order.
    pub(crate) fn all(&self) -> Vec<SemanticStage> {
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

/// A verified N-input, one-output `f32` program whose output is a strict serial
/// reduction of a recognized elementwise contributor expression.
///
/// **The prologue is a general expression, not a template.** It is whatever
/// [`recognize_elementwise`] found between the declared inputs and the
/// reduction's operand — any depth, any mix of the recognized families, any
/// number of declared inputs, and shared reads. `input_keys` and `inputs` are
/// parallel and in declaration order, which is the order the expression's input
/// ordinals index and the order the assembled program binds its buffers in.
///
/// **And it is optional, because `sum(x)` has none.** A fold whose operand is a
/// declared input computes nothing before the fold, so there is no expression to
/// carry and no region to build for one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedSerialSum {
    pub(crate) input_keys: Vec<InputKey>,
    pub(crate) output_key: OutputKey,
    /// The contributor domain: the shape the prologue writes and the fold reads.
    pub(crate) input_shape: Shape,
    pub(crate) output_shape: Shape,
    pub(crate) reduction_axes: Vec<Axis>,
    /// The recognized elementwise prologue the fold's contributors come from.
    ///
    /// `None` when the fold's operand is a declared input tensor. That is the
    /// typed statement of "there is no prologue region here", and it is what every
    /// prologue-spelling derivation asks: an identity expression standing in for
    /// the absence would let a cover spell a copy kernel whose materialization —
    /// and whose rounding boundary — the caller's program never asked for.
    pub(crate) prologue: Option<PointwiseF32Expression>,
    /// The prologue region's reads, in access order, or empty when there is no
    /// prologue.
    ///
    /// Empty exactly when `prologue` is `None`, for the reason that field is
    /// `None`: a fold over a declared input has no prologue region, so there is
    /// no read list to state and an inhabited one would describe a region no
    /// cover places.
    pub(crate) prologue_reads: Vec<(u32, LogicalAccess)>,
    /// The declared input ordinal the fold reads directly, or `None` when a
    /// prologue region materializes its contributors.
    ///
    /// **`Some` exactly when [`Self::prologue`] is `None`, and it is the
    /// recognized ordinal rather than zero.** A prologue-less fold's own read is
    /// the one access no read list describes — `prologue_reads` belongs to a
    /// region this program does not have — so without this field the physical
    /// layer had nothing but the declared arity to derive the contributor buffer
    /// from, and derived `Input { ordinal: 0 }`. That was right while every
    /// elementwise walk read every declared input, because such a program
    /// declared exactly one; `sum(b)` beside an independent `a * a` declares two
    /// and folds the second.
    pub(crate) contributor_input: Option<u32>,
    pub(crate) members: RecognizedSerialSumMembers,
    pub(crate) inputs: Vec<ValueId>,
    pub(crate) pointwise_result: ValueId,
    pub(crate) output: ValueId,
    pub(crate) input_elements: u64,
    pub(crate) output_elements: u64,
}

impl NormalizedSerialSum {
    /// The occurrences a prologue region would cover, when this fold has one.
    ///
    /// `None` rather than an empty slice for a fold over a declared input, so a
    /// cover region covering no occurrence cannot match the part. The two answers
    /// are the same bytes and different facts: "the prologue claims nothing" is a
    /// state no recognized program is in, and treating it as one is how an empty
    /// member set would acquire a region.
    pub(crate) fn prologue_members(&self) -> Option<&[SemanticStage]> {
        self.prologue.as_ref().map(|_| self.members.pointwise())
    }
}

/// One recognized per-point expression, in the arithmetic its program states.
///
/// **The arithmetic is carried rather than assumed, and the two vocabularies are
/// separate types rather than one width-tagged one.** A per-point body is a
/// function on a *specific* format — `x * 3.0` rounds differently in binary32 and
/// in `bf16`, and a `bf16` constant is a sixteen-bit pattern that no
/// [`PointwiseF32Node::Constant`] payload can hold — so `tiler_ir::schedule`
/// gives each width its own expression type and its own scheduled-region
/// spelling. This enum is what lets one recognizer walk produce either.
///
/// Every consumer matches it exhaustively rather than projecting it to a tag, so
/// a third admitted width is a build error at each site instead of an expression
/// silently spelled as one of these two.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RecognizedPointwise {
    F32(PointwiseF32Expression),
    Bf16(PointwiseBf16Expression),
}

impl RecognizedPointwise {
    /// The `f32` expression this recognition holds, or a refusal naming the
    /// width it holds instead.
    ///
    /// **Defence in depth rather than a live gate.** The one caller is the fold's
    /// prologue walk, which is entered only from `tiler::strict-serial-sum-f32@1`
    /// and therefore states `f32` at the call site; the refusal exists so that a
    /// fold family admitted at another width fails loudly here instead of the
    /// prologue silently acquiring a spelling its region cannot carry.
    fn into_f32(self) -> Result<PointwiseF32Expression, RequestError> {
        match self {
            Self::F32(expression) => Ok(expression),
            Self::Bf16(_) => mismatch("prologue-arithmetic"),
        }
    }

    /// The `f32` expression a fixture asserted about, for the crate's own tests.
    ///
    /// Panics for the other width, like [`NormalizedOutput::serial_sum`] and its
    /// siblings: a fixture whose recognized width changed should fail loudly
    /// here rather than have its assertion quietly skipped.
    #[cfg(test)]
    fn f32(&self) -> &PointwiseF32Expression {
        match self {
            Self::F32(expression) => expression,
            Self::Bf16(_) => panic!("the fixture recognizes an f32 expression"),
        }
    }

    /// The `bf16` expression a fixture asserted about, for the crate's own tests.
    #[cfg(test)]
    fn bf16(&self) -> &PointwiseBf16Expression {
        match self {
            Self::Bf16(expression) => expression,
            Self::F32(_) => panic!("the fixture recognizes a bf16 expression"),
        }
    }
}

/// A verified N-input, one-output elementwise program.
///
/// `input_keys` and `inputs` are parallel and in the program's declaration
/// order, which is the order the expression's input ordinals index and the order
/// the assembled program binds its buffers in. One `shape` governs every input
/// and the output, so a single element count sizes the whole region.
///
/// **`expression` is the recognized program, not a projection of it.** It is the
/// general per-point expression vocabulary rather than a fixed leaf count and
/// association, so what the recognizer admits is bounded by what the physical
/// expression can spell rather than by a shape it was taught. It also carries
/// the arithmetic the program is stated in, because that is what decides which
/// scheduled-region scalar program realizes it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedPointwise {
    pub(crate) input_keys: Vec<InputKey>,
    pub(crate) output_key: OutputKey,
    pub(crate) shape: Shape,
    pub(crate) expression: RecognizedPointwise,
    pub(crate) members: Vec<SemanticStage>,
    pub(crate) inputs: Vec<ValueId>,
    pub(crate) output: ValueId,
    pub(crate) elements: u64,
    /// The region's reads, in access order.
    ///
    /// One entry per expression input leaf, naming the declared input ordinal it
    /// binds and the relation it addresses that tensor with. Ordinals do not
    /// descend and one may appear twice — once densely and once through a
    /// relation — which is how `a * permute(a)` is spelled: two leaves meaning
    /// two different tensors derived from one declared input.
    pub(crate) reads: Vec<(u32, LogicalAccess)>,
}

/// One declared-input read of a verified binary tensor contraction.
///
/// The declaration ordinal is the ABI binding, while `operand_position` is the
/// position in the contraction occurrence and its canonical index structure.
/// Keeping both beside the operand's value, shape, and count prevents a
/// region-local renumbering from silently changing which program tensor a
/// structure operand reads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedContractionRead {
    pub(crate) input_ordinal: u32,
    pub(crate) shape: Shape,
    pub(crate) elements: u64,
    pub(crate) value: ValueId,
    pub(crate) operand_position: usize,
}

/// A verified binary tensor-contraction `f32` shape over two distinct declared
/// inputs and one semantic result.
///
/// **The structure is carried whole, not projected.** ADR 0087 makes the
/// canonical index structure the operation's identity, so a normalization that
/// kept only the extents it happened to need would let two different structures
/// over the same shapes share a request subject. `reads` is ordered by strictly
/// ascending declared-input ordinal. Each entry names the structure operand it
/// supplies; the complete declaration remains in `input_keys` so those ordinals
/// keep their program-wide meaning when another output reads an input this
/// contraction does not.
///
/// `output_shape` and `contracted_shape` are derived from the structure and the
/// operand shapes rather than read from the graph, and the derived output shape
/// is required to equal the program's own: the semantic inferencer already
/// proved them equal at construction, so a disagreement here is invalid state
/// and is refused rather than resolved in favour of either side.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedContraction {
    /// The complete declared-input list, in program declaration order.
    pub(crate) input_keys: Vec<InputKey>,
    pub(crate) output_key: OutputKey,
    /// The two distinct operand reads, ordered by declared input ordinal.
    pub(crate) reads: [NormalizedContractionRead; 2],
    pub(crate) output_shape: Shape,
    /// Row-major shape of the contracted iteration space, ascending by
    /// canonical contracted index.
    pub(crate) contracted_shape: Shape,
    pub(crate) structure: ContractionIndexStructure,
    pub(crate) members: Vec<SemanticStage>,
    pub(crate) output: ValueId,
    pub(crate) output_elements: u64,
    /// Points of the contracted iteration space; the fold length per output.
    pub(crate) contracted_elements: u64,
}

/// Which boundary tensor one recognized read binds.
///
/// **The access position and the boundary role are separate facts here, and a
/// whole-program elementwise region never had to distinguish them.** That region
/// reads every declared input in declaration order, so leaf `i` and declared
/// input `i` coincide and one number serves as both. An epilogue reads the value
/// an earlier region staged plus whichever declared inputs its expression names,
/// so the *position* of a read — the leaf it serves — and the *tensor* it binds
/// are independent. `tiler_ir::schedule`'s `reads_bind_boundary_tensors_in_order`
/// states the same separation from the schedule side, and
/// `crate::program::CoverAssembly::from_plan` is what resolves the role against
/// the program's declared interface.
///
/// **Two recognized shapes carry it, and the separation is the same fact in
/// both.** An epilogue's read list names the tensor each expression leaf binds;
/// a staged family's operand run names the tensor each *occurrence operand*
/// binds, because
/// [`admit-a-staged-family-that-reads-a-materialized-intermediate`](../../../tickets/admit-a-staged-family-that-reads-a-materialized-intermediate.md)
/// made an operand's source a recognition-time property. One vocabulary rather
/// than two is what keeps `crate::program::CoverAssembly::from_plan` resolving
/// one kind of role against the cover, and what keeps
/// [`Self::tensor`] the single statement of the mapping onto
/// [`TensorRole`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoundaryRead {
    /// The value the producer region staged, bound to the materialization edge
    /// the cover hands this region.
    ///
    /// Carries no ordinal because [`TensorRole::Intermediate`] carries none: a
    /// region reads at most one staged value, which is exactly what makes the
    /// unordinalled role sufficient and a second staged read inadmissible.
    Staged,
    /// The declared program input at this ordinal.
    Input(u32),
}

impl BoundaryRead {
    /// Returns the boundary tensor this read binds.
    pub(crate) const fn tensor(self) -> TensorRole {
        match self {
            Self::Staged => TensorRole::Intermediate,
            Self::Input(ordinal) => TensorRole::Input {
                ordinal: InputOrdinal::new(ordinal),
            },
        }
    }

    /// Returns the declared input ordinal this read binds, or `None` for the
    /// staged one.
    pub(crate) const fn declared_ordinal(self) -> Option<u32> {
        match self {
            Self::Staged => None,
            Self::Input(ordinal) => Some(ordinal),
        }
    }
}

/// A verified `f32` program output that is an elementwise expression over a
/// value an earlier region produces.
///
/// **The chain is the recognized shape, not two shapes that happen to compose.**
/// `matmul(a, b) * 2.0` and `sum(x * x) * scale` are one declared output each,
/// and neither the contraction nor the fold publishes anything: their result is
/// a materialization edge some cover places, and the epilogue is the region that
/// consumes it. Carrying the producer *inside* this shape is what makes "which
/// recognized partition does this region belong to" answerable for both halves
/// from one place, and what lets every region builder, cost, and subject binding
/// the producing family already has apply to the producer unchanged.
///
/// **The producer is a folding family, and only ever those two.** A pointwise
/// producer is not a materialization boundary at all — its occurrences are part
/// of the epilogue's own walk, and fusing them is the whole point of the
/// expression vocabulary. [`recognize_epilogue_producer`] is where that is
/// enforced, and the `NormalizedOutput` typing here is a convenience for the
/// consumers rather than a claim that any variant may appear.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedEpilogue {
    /// The producer whose staged result this epilogue reads.
    ///
    /// Its `output_key` is *this* chain's published key. The producer names no
    /// key of its own — it publishes nothing — and the field means "the ordered
    /// named output the partition this shape belongs to publishes", which is the
    /// producer's own key exactly when the producer is the whole output.
    pub(crate) producer: Box<NormalizedOutput>,
    pub(crate) input_keys: Vec<InputKey>,
    pub(crate) output_key: OutputKey,
    /// The epilogue region's iteration domain, which is the published shape.
    pub(crate) shape: Shape,
    pub(crate) expression: PointwiseF32Expression,
    /// One entry per expression input leaf, in access order.
    ///
    /// Parallel to the region's reads: leaf `i` is served by entry `i`, which
    /// names the boundary tensor it binds and the relation it addresses that
    /// tensor with. Exactly one entry is [`BoundaryRead::Staged`].
    pub(crate) reads: Vec<(BoundaryRead, LogicalAccess)>,
    /// The occurrences the epilogue region itself covers.
    pub(crate) members: Vec<SemanticStage>,
    pub(crate) inputs: Vec<ValueId>,
    pub(crate) output: ValueId,
    pub(crate) elements: u64,
}

/// A verified `f32` value produced by one occurrence of a registered family
/// whose realization law realizes a region *sequence*.
///
/// **The classification is the law's, not a family list's.** What makes an
/// occurrence this shape is that
/// [`FrozenIndexRealizationLawRegistry::family_realizes_region_sequence`]
/// answers true for its operation key — so every family the registry carries a
/// multi-region law for is recognized by the same arm, and a family added to the
/// registry becomes recognizable without a line here. No operation key is named
/// in the recognizer, which is what keeps the capability the general one.
///
/// **It carries the occurrence and not the stage split, because the stage split
/// belongs to region formation.** Tom's Option A′ decision made
/// [`crate::region::RegionGraph::with_realizations`] the authority that reads
/// each law's realized sequence and enumerates one candidate per stage; a
/// recognizer that also enumerated them would be a second account of one fact,
/// and the two would have to agree about stage counts, sources, and handed
/// values for either to mean anything. So this shape claims
/// [`SemanticStage::first`] — the occurrence — and
/// [`NormalizedOutput::owns_region_members`] answers for whichever stage atoms
/// formation actually minted.
///
/// **One operand may be a value another region materializes, and the shape says
/// which.** `rms_norm(matmul(a, b), w)` reads its first operand across a
/// materialization edge rather than from a declared buffer, so
/// [`Self::operand_reads`] carries a per-operand [`BoundaryRead`] and
/// [`Self::producer`] carries the recognized shape whose regions write it. Both
/// are recognition-time facts and both live here, which is the resolution
/// [`admit-a-staged-family-that-reads-a-materialized-intermediate`](../../../tickets/admit-a-staged-family-that-reads-a-materialized-intermediate.md)
/// chose over deriving the operand's source from the cover's materialization
/// edges: an operand supplied by no declared input and no recognizable producer
/// is a property of the *program*, and a stage that discovered it later could
/// only report it as a cover it could not assemble.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedStaged {
    /// The recognized shape producing the value one operand reads, when an
    /// operand is a materialized intermediate.
    ///
    /// `None` exactly when every entry of [`Self::operand_reads`] is
    /// [`BoundaryRead::Input`]. It is carried for the reason
    /// [`NormalizedEpilogue::producer`] is carried: the producing occurrences
    /// belong to *this* output's walk — nothing else claims them, and
    /// [`check_output_cover`] refuses a program with an occurrence no walk
    /// claimed — so the region a cover places for them has to be spelled from a
    /// recognized shape this partition holds.
    ///
    /// Its `output_key` is this occurrence's own, for the reason
    /// [`NormalizedEpilogue::producer`] states: a producer publishes nothing and
    /// the field means the ordered named output the partition it belongs to
    /// publishes.
    pub(crate) producer: Option<Box<NormalizedOutput>>,
    /// The registered family this occurrence belongs to.
    pub(crate) operation: OpKey,
    /// The law the registry carries for that family.
    ///
    /// **Read once, here, from the same registry row the admission above reads.**
    /// A scheduled region for one of these stages has to know what the stage
    /// computes — which axes it folds, which payload its epilogue carries — and
    /// that is the law's content rather than the occurrence's shape: a `[2, 2]`
    /// operand reduced to `[2]` names two different reductions, so no derivation
    /// from shapes can recover it. Carrying the law is what lets the physical
    /// layer be written against the closed law vocabulary — one arm per law, a
    /// fail-closed wildcard for the rest — instead of against a family list.
    ///
    /// It is *not* a second account of the realization. The stage count, each
    /// stage's reads, and the handed values stay
    /// [`crate::region::RegionGraph::with_realizations`]'s, read off the law's own
    /// realized sequence; this field is the law itself, which is one value with
    /// one owner however many readers it has.
    pub(crate) law: IndexRealizationLaw,
    /// The occurrence's attribute record in canonical bytes.
    ///
    /// Carried whole rather than projected. `tiler::rms-norm-f32@1` declares its
    /// reduced axis and its exact `eps` payload here, both of which are part of
    /// what the occurrence computes, and a subject that dropped them would give
    /// two different normalizations one identity.
    pub(crate) attributes: Box<[u8]>,
    /// The same record, typed.
    ///
    /// Beside the canonical bytes rather than instead of them, because the two
    /// serve different readers and neither derives the other cheaply: identity
    /// binds the bytes, and the law names the *fields* it interprets by
    /// identifier, which only the typed record can answer. They cannot disagree —
    /// the bytes are this record's own canonical encoding.
    pub(crate) attribute_record: OperationAttributes,
    pub(crate) input_keys: Vec<InputKey>,
    pub(crate) output_key: OutputKey,
    /// Boundary tensor supplying each occurrence operand, in operand order.
    ///
    /// At most one entry is [`BoundaryRead::Staged`], and the bound is the
    /// unordinalled [`TensorRole::Intermediate`]'s rather than a simplification:
    /// a second staged operand — one value read twice, or two different
    /// materialized values — has nothing to say which edge each read binds.
    /// [`recognize_staged_family`]'s `staged-operand-conflict` refusal is that
    /// boundary and [`Self::producer`] is the one edge that survives it.
    pub(crate) operand_reads: Vec<BoundaryRead>,
    /// Operand shapes, in operand order.
    pub(crate) operand_shapes: Vec<Shape>,
    /// The published shape of the occurrence's one result.
    pub(crate) output_shape: Shape,
    /// The occurrence this walk claimed.
    pub(crate) member: SemanticMemberId,
    pub(crate) inputs: Vec<ValueId>,
    pub(crate) output: ValueId,
    /// Operand element counts, in operand order.
    pub(crate) operand_elements: Vec<u64>,
    pub(crate) output_elements: u64,
}

impl NormalizedStaged {
    /// Returns whether one region's members are exactly stages of this
    /// occurrence.
    ///
    /// **Narrower than [`NormalizedOutput::owns_region_members`] on purpose, and
    /// the difference is the producer.** That predicate answers for every region
    /// of this output's partition, which since a staged operand became
    /// recognizable includes the producer's own regions; this one answers only
    /// for the stages *this* occurrence realizes as. [`crate::physical`] asks
    /// this before it names a stage spelling, because a producer region resolved
    /// to this output belongs to the producer's family and not to the law's two
    /// stages.
    ///
    /// The stage list is deliberately not enumerated here: region formation
    /// decided how many stages there are and which atoms exist, so asking for a
    /// stage list would be a second account of that decision, free to disagree
    /// with the candidates actually enumerated. An empty member set is no region
    /// of this occurrence.
    pub(crate) fn owns_stage_members(&self, members: &[SemanticStage]) -> bool {
        !members.is_empty() && members.iter().all(|atom| atom.member() == self.member)
    }

    /// Returns each operand that binds a declared input, as `(ordinal, count)`.
    ///
    /// The staged operand is skipped rather than reported at some ordinal: its
    /// element count sizes a materialization edge, not a declared buffer, and a
    /// caller scaling work over a declared input must not receive it.
    fn declared_operands(&self) -> impl Iterator<Item = (u32, u64)> + '_ {
        self.operand_reads
            .iter()
            .zip(&self.operand_elements)
            .filter_map(|(read, elements)| Some((read.declared_ordinal()?, *elements)))
    }
}

/// One recognized ordered named program output, and the region partition that
/// implements it.
///
/// **A property of one output, not of the program.** Each variant carries the
/// occurrences its own walk claimed, partitioned into the parts a region can be
/// spelled from — one part for the two single-region shapes, the prologue and
/// the fold for a reduction. [`NormalizedProgram`] holds one of these per
/// declared output, in declaration order, so "which strategy implements this
/// cover region" is answered by the part whose members the region covers rather
/// than by asking which whole-program template matched.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NormalizedOutput {
    SerialSum(NormalizedSerialSum),
    Pointwise(NormalizedPointwise),
    /// Boxed because a contraction carries two operand shapes, an output shape,
    /// a contracted shape, and a validated index structure — roughly twice the
    /// serial sum's payload — and every value of this enum would otherwise pay
    /// for the widest variant.
    Contraction(Box<NormalizedContraction>),
    /// An elementwise expression over a value a folding region stages.
    ///
    /// Boxed because it carries a whole further recognized output inside it,
    /// which would otherwise make every value of this enum the size of two.
    Epilogue(Box<NormalizedEpilogue>),
    /// One occurrence of a registered family realized as a region sequence, and
    /// the shape producing the value one operand reads when that operand is a
    /// materialized intermediate.
    ///
    /// Boxed because it carries an operand-indexed shape list, an element-count
    /// list, the occurrence's canonical attribute bytes, and a whole further
    /// recognized output, none of which the other variants pay for.
    Staged(Box<NormalizedStaged>),
}

impl NormalizedOutput {
    pub(crate) const fn serial_sum(&self) -> &NormalizedSerialSum {
        match self {
            Self::SerialSum(normalized) => normalized,
            Self::Pointwise(_) | Self::Contraction(_) | Self::Epilogue(_) | Self::Staged(_) => {
                panic!("request is not a serial-sum program")
            }
        }
    }

    pub(crate) const fn try_serial_sum(&self) -> Option<&NormalizedSerialSum> {
        match self {
            Self::SerialSum(normalized) => Some(normalized),
            Self::Pointwise(_) | Self::Contraction(_) | Self::Epilogue(_) | Self::Staged(_) => None,
        }
    }

    pub(crate) const fn pointwise(&self) -> Option<&NormalizedPointwise> {
        match self {
            Self::SerialSum(_) | Self::Contraction(_) | Self::Epilogue(_) | Self::Staged(_) => None,
            Self::Pointwise(normalized) => Some(normalized),
        }
    }

    pub(crate) const fn contraction(&self) -> Option<&NormalizedContraction> {
        match self {
            Self::SerialSum(_) | Self::Pointwise(_) | Self::Epilogue(_) | Self::Staged(_) => None,
            Self::Contraction(normalized) => Some(normalized),
        }
    }

    pub(crate) const fn epilogue(&self) -> Option<&NormalizedEpilogue> {
        match self {
            Self::SerialSum(_) | Self::Pointwise(_) | Self::Contraction(_) | Self::Staged(_) => {
                None
            }
            Self::Epilogue(normalized) => Some(normalized),
        }
    }

    pub(crate) const fn staged(&self) -> Option<&NormalizedStaged> {
        match self {
            Self::SerialSum(_) | Self::Pointwise(_) | Self::Contraction(_) | Self::Epilogue(_) => {
                None
            }
            Self::Staged(normalized) => Some(normalized),
        }
    }

    /// Returns the recognized shape one *producer* region of this output is
    /// built from.
    ///
    /// A chain's producer regions — the fold, its prologue, its split passes,
    /// its cooperative tile, the contraction — are spelled from the producer's
    /// own recognized shape, so every derivation that would otherwise read the
    /// chain asks this instead and reaches the same value it reaches for a
    /// standalone output. The epilogue region is the one part that is not built
    /// from it, and [`crate::physical::RegionSpellingKind::Epilogue`] is what
    /// distinguishes it.
    ///
    /// **It takes the region's members, and it has to since a staged family may
    /// read a materialized intermediate.** Such an output holds two recognized
    /// shapes whose regions a cover both places — the occurrence's own two
    /// stages, and its producer's partition — so "the producer shape" is not a
    /// property of the output alone. The epilogue arm descends unconditionally
    /// because its own region is never spelled through here; the staged arm
    /// descends exactly when the members are not the occurrence's stages, which
    /// is the same question [`crate::physical::spell_region`] answered to reach
    /// this call.
    pub(crate) fn producer_shape_for(&self, members: &[SemanticStage]) -> &Self {
        match self {
            Self::SerialSum(_) | Self::Pointwise(_) | Self::Contraction(_) => self,
            Self::Epilogue(chain) => chain.producer.producer_shape_for(members),
            Self::Staged(normalized) => normalized
                .producer
                .as_deref()
                .filter(|_| !normalized.owns_stage_members(members))
                .map_or(self, |producer| producer.producer_shape_for(members)),
        }
    }

    /// Returns the element count of one declared input tensor this output
    /// reads.
    ///
    /// Per ordinal rather than one shared count, because a contraction's two
    /// operands generally have different extents. Every arm answers only for an
    /// ordinal *this* output reads and `None` for one it does not, so a caller
    /// that names an ordinal this output never loads gets a refusal instead of
    /// another tensor's size.
    ///
    /// **The count is the declared tensor's own, never the iteration domain of
    /// a region that reads it.** Its consumer binds
    /// `TensorRole::Input { ordinal }`, which names the buffer the ABI binds at
    /// that ordinal, so a count taken from the reading region would scale a call
    /// by an iteration space rather than by the tensor the caller passed. The
    /// two coincide for a dense read — [`plan_elementwise`] refuses a leaf at
    /// any shape but the region's — and diverge exactly for a widening
    /// structural read: `a * broadcast(w)` over a `[2]` weight iterates `[2, 2]`
    /// and must still answer `2` for the ordinal `w` occupies.
    ///
    /// So every arm derives from an *operand* shape. `Contraction` reads its
    /// per-ordinal operand counts and `Staged` its operand run, both of which
    /// are declared operands' own extents. The three arms holding elementwise
    /// read lists ask [`read_tensor_elements`] per read, which answers a
    /// structural read from the relation's own operand shape and declines a
    /// relation it cannot size.
    ///
    /// **The read lists are the gate, not the declared arity, and that is a
    /// per-read truth rather than a per-program one.** `input_keys` is the whole
    /// program's declaration list, so a bound on its length says nothing about
    /// which of those inputs this output's walk reached; the premise that every
    /// declared input of a reduced program is read at the contributor domain
    /// held only while every walk had to read every declared input. Since a walk
    /// may read a subset, an output iterating one domain would otherwise
    /// volunteer its own count for an ordinal only a sibling loads, and
    /// [`NormalizedProgram::agreed_input_elements_at`] would read that
    /// volunteered count as a disagreement and refuse a call the reading output
    /// sizes exactly. [`Self::reads_declared_input`] states the same fact for
    /// the callers that need only the predicate, and the two must keep agreeing
    /// about which ordinals a walk reached.
    ///
    /// **Agreement or nothing wherever one ordinal has several reads**, for the
    /// reason [`NormalizedProgram::agreed_input_elements_at`] states: a half
    /// that reads the ordinal and cannot size it must refuse rather than be
    /// overruled by a half that can. Whether a half *reads* the ordinal is asked
    /// separately from what it answers, so an unsizable read is a `Some(None)`
    /// the fold sees rather than an abstention it drops.
    ///
    /// The disagreement is unreachable for the relations recognized today: every
    /// route above resolves to the shape the program declared the ordinal at, so
    /// the answer is a function of the ordinal alone. That is exhaustive finite
    /// evidence over five arms and three admitted access maps rather than a
    /// proof about relations not yet spelled, which is why the fold stays.
    /// `every_arm_answers_the_declared_tensors_own_count` is what says so, and
    /// says no when an arm reintroduces a domain.
    pub(crate) fn input_elements_at(&self, ordinal: InputOrdinal) -> Option<u64> {
        let ordinal = ordinal.get();
        match self {
            // A prologue region's reads address declared tensors from the
            // contributor domain; a prologue-less fold's own contributor read
            // addresses the declared tensor directly, and `input_shape` is that
            // operand's own shape rather than a domain standing in for it. The
            // two sources are mutually exclusive — `prologue_reads` is inhabited
            // exactly when `contributor_input` is `None` — and are folded
            // together anyway so neither has to restate the exclusion.
            Self::SerialSum(normalized) => agreed(
                normalized
                    .prologue_reads
                    .iter()
                    .filter(|(read, _)| *read == ordinal)
                    .map(|(_, map)| read_tensor_elements(map, normalized.input_elements))
                    .chain(
                        (normalized.contributor_input == Some(ordinal))
                            .then_some(Some(normalized.input_elements)),
                    ),
            )
            .flatten(),
            Self::Pointwise(normalized) => agreed(
                normalized
                    .reads
                    .iter()
                    .filter(|(read, _)| *read == ordinal)
                    .map(|(_, map)| read_tensor_elements(map, normalized.elements)),
            )
            .flatten(),
            // A contraction's two explicit reads are a subset of the complete
            // declared interface, so the ordinal map — not declaration length
            // or read position — gates the count.
            Self::Contraction(normalized) => normalized
                .reads
                .iter()
                .find(|read| read.input_ordinal == ordinal)
                .map(|read| read.elements),
            // A chain reads a declared input from its producer, from its
            // epilogue, or from both. Whether each half reads it is asked before
            // what it answers, so a half that reads the ordinal without being
            // able to size it contributes a `None` the fold compares rather than
            // an absence it cannot tell from silence. Neither half reading is
            // the arm's own spelling of "this chain does not read that ordinal".
            Self::Epilogue(chain) => agreed(
                chain
                    .producer
                    .reads_declared_input(ordinal)
                    .then(|| chain.producer.input_elements_at(InputOrdinal::new(ordinal)))
                    .into_iter()
                    .chain(
                        chain
                            .reads
                            .iter()
                            .any(|(read, _)| *read == BoundaryRead::Input(ordinal))
                            .then(|| {
                                agreed(
                                    chain
                                        .reads
                                        .iter()
                                        .filter(|(read, _)| *read == BoundaryRead::Input(ordinal))
                                        .map(|(_, map)| read_tensor_elements(map, chain.elements)),
                                )
                                .flatten()
                            }),
                    ),
            )
            .flatten(),
            // The operand run is the occurrence's read list, so an operand
            // binding a declared input answers that operand tensor's own count
            // and the staged operand answers nothing — its count sizes a
            // materialization edge rather than a declared buffer.
            //
            // Agreement or nothing, over the occurrence's own operands *and* its
            // producer's answer, for the reason the chain arm above and
            // [`NormalizedProgram::agreed_input_elements_at`] both state: two
            // claimants that cannot name one extent for a tensor give a
            // work-scaling caller no single answer, and the producer is asked
            // only when it reads the ordinal so its silence costs nothing.
            Self::Staged(normalized) => agreed(
                normalized
                    .declared_operands()
                    .filter(|(operand, _)| *operand == ordinal)
                    .map(|(_, elements)| Some(elements))
                    .chain(
                        normalized
                            .producer
                            .as_deref()
                            .filter(|producer| producer.reads_declared_input(ordinal))
                            .map(|producer| producer.input_elements_at(InputOrdinal::new(ordinal))),
                    ),
            )
            .flatten(),
        }
    }

    /// Returns whether some region of this output's partition reads one
    /// declared input tensor.
    ///
    /// **Read from the recognized read lists, not from the declared arity.** An
    /// output's regions bind the inputs its own walk reached, so "this program
    /// declares three inputs" says nothing about which of them this output
    /// loads. It is exhaustive over the recognized shapes rather than projected
    /// through one of them, because each carries the fact differently: an
    /// elementwise region in its read list, a fold in its prologue's read list
    /// *or* its own contributor ordinal, a contraction in its operand count, and
    /// a chain in both halves of the chain.
    fn reads_declared_input(&self, ordinal: u32) -> bool {
        match self {
            Self::Pointwise(normalized) => {
                normalized.reads.iter().any(|(read, _)| *read == ordinal)
            }
            Self::SerialSum(normalized) => {
                normalized.contributor_input == Some(ordinal)
                    || normalized
                        .prologue_reads
                        .iter()
                        .any(|(read, _)| *read == ordinal)
            }
            Self::Contraction(normalized) => normalized
                .reads
                .iter()
                .any(|read| read.input_ordinal == ordinal),
            Self::Epilogue(chain) => {
                chain
                    .reads
                    .iter()
                    .any(|(read, _)| *read == BoundaryRead::Input(ordinal))
                    || chain.producer.reads_declared_input(ordinal)
            }
            // The recognized operand map, not the declared arity: a staged
            // occurrence binds one read per operand and a program may declare an
            // input it never names. Its producer's regions are part of this
            // output's partition too, so a declared input only the producer
            // reaches is one this output reads.
            Self::Staged(normalized) => {
                normalized
                    .declared_operands()
                    .any(|(operand, _)| operand == ordinal)
                    || normalized
                        .producer
                        .as_deref()
                        .is_some_and(|producer| producer.reads_declared_input(ordinal))
            }
        }
    }

    /// Returns the largest declared input element count this output reads.
    ///
    /// **Declared tensors' own counts, on the same basis
    /// [`Self::input_elements_at`] states**, so the two accessors cannot
    /// disagree about what a "declared input element count" is. A widening read
    /// used to make this report the reading region's domain, which is the
    /// iteration space rather than any tensor the ABI binds.
    ///
    /// **A read whose relation [`read_tensor_elements`] declines contributes the
    /// reading region's domain rather than refusing**, and the asymmetry with
    /// [`Self::input_elements_at`] is deliberate: this feeds structural cost
    /// estimates alone — [`NormalizedProgram::max_input_elements`] records the
    /// caller — and a maximum that refused would turn an estimate into a
    /// feasibility gate. It is unreachable for the three maps recognized today,
    /// and where a fourth reached it the domain would be an estimate rather than
    /// a bound: it happens to equal the operand count for a bijection and to
    /// exceed it for a replication, but a narrowing relation would sit the other
    /// side and nothing here would say so.
    pub(crate) fn max_input_elements(&self) -> u64 {
        match self {
            // Same two sources the reading arm folds, and for the same reason:
            // a prologue region's reads, or a prologue-less fold's own
            // contributor read of the declared tensor.
            Self::SerialSum(normalized) => normalized
                .prologue_reads
                .iter()
                .map(|(_, map)| {
                    read_tensor_elements(map, normalized.input_elements)
                        .unwrap_or(normalized.input_elements)
                })
                .chain(
                    normalized
                        .contributor_input
                        .map(|_| normalized.input_elements),
                )
                .max()
                .unwrap_or_default(),
            Self::Pointwise(normalized) => normalized
                .reads
                .iter()
                .map(|(_, map)| {
                    read_tensor_elements(map, normalized.elements).unwrap_or(normalized.elements)
                })
                .max()
                .unwrap_or_default(),
            Self::Contraction(normalized) => normalized
                .reads
                .iter()
                .map(|read| read.elements)
                .max()
                .unwrap_or_default(),
            // The epilogue's declared-input reads only: a chain whose epilogue
            // reads only the staged value reads no declared input there, and
            // reporting its domain would overstate what this output reads.
            Self::Epilogue(chain) => chain.producer.max_input_elements().max(
                chain
                    .reads
                    .iter()
                    .filter(|(read, _)| read.declared_ordinal().is_some())
                    .map(|(_, map)| {
                        read_tensor_elements(map, chain.elements).unwrap_or(chain.elements)
                    })
                    .max()
                    .unwrap_or_default(),
            ),
            // The declared-input operands only, and the producer's own answer
            // beside them: a staged operand's count is a materialization edge's
            // extent, and reporting it here would overstate the largest declared
            // input this output reads.
            Self::Staged(normalized) => normalized
                .declared_operands()
                .map(|(_, elements)| elements)
                .max()
                .unwrap_or_default()
                .max(
                    normalized
                        .producer
                        .as_deref()
                        .map_or(0, NormalizedOutput::max_input_elements),
                ),
        }
    }

    pub(crate) const fn output_elements(&self) -> u64 {
        match self {
            Self::SerialSum(normalized) => normalized.output_elements,
            Self::Pointwise(normalized) => normalized.elements,
            Self::Contraction(normalized) => normalized.output_elements,
            Self::Epilogue(chain) => chain.elements,
            Self::Staged(normalized) => normalized.output_elements,
        }
    }

    /// Returns every occurrence this output's walk claimed, in ascending order.
    pub(crate) fn members(&self) -> Vec<SemanticStage> {
        match self {
            Self::SerialSum(normalized) => normalized.members.all(),
            Self::Pointwise(normalized) => normalized.members.clone(),
            Self::Contraction(normalized) => normalized.members.clone(),
            Self::Epilogue(chain) => {
                let mut members = chain.producer.members();
                members.extend_from_slice(&chain.members);
                members.sort_unstable();
                members.dedup();
                members
            }
            // The occurrence, once, and its producer's own claim when an operand
            // is a materialized intermediate. A staged family's realization
            // stages are region formation's to enumerate — see
            // [`NormalizedStaged`] — and claiming them here would state the same
            // split twice and make [`check_output_cover`]'s per-occurrence
            // accounting count a realization choice as program work. The
            // producer's occurrences are program work and are claimed by this
            // walk alone, exactly as a chain's producer's are.
            Self::Staged(normalized) => {
                let mut members = normalized
                    .producer
                    .as_deref()
                    .map_or_else(Vec::new, NormalizedOutput::members);
                members.push(SemanticStage::first(normalized.member));
                members.sort_unstable();
                members.dedup();
                members
            }
        }
    }

    /// Returns whether one region's exact member set is a part of this output's
    /// partition, so that a region spelled from it covers this output's work and
    /// no other's.
    ///
    /// The reduction's *whole* partition is a part in its own right: the fused
    /// spelling realizes the prologue and the fold in one region, which is the
    /// one case where a part is the union of two others.
    ///
    /// A prologue-less fold's partition has one part, and the prologue part is
    /// asked for through [`NormalizedSerialSum::prologue_members`] rather than by
    /// comparing against an empty set: a region covering no occurrence would
    /// otherwise resolve to a prologue this program does not have, and every
    /// derivation downstream would build one. Like the same distinction in
    /// [`crate::physical::spell_region`], it is defence in depth rather than a
    /// live gate — no cover this search places carries an empty member set.
    ///
    /// **Crate-visible because the region vocabulary asks it rather than
    /// restating it.** *Which* region spells a member set is
    /// [`crate::physical::spell_region`]'s question, decided against the region
    /// vocabulary; whether the member set is this output's at all is this one,
    /// decided against the recognized partition. A physical arm answering the
    /// second for itself would be a second account of the partition, free to
    /// disagree with the account [`NormalizedProgram::output_for_region`] and
    /// [`check_output_cover`] read.
    pub(crate) fn owns_region_members(&self, members: &[SemanticStage]) -> bool {
        match self {
            Self::SerialSum(normalized) => {
                normalized
                    .prologue_members()
                    .is_some_and(|prologue| members == prologue)
                    || members == normalized.members.reduction()
                    || members == normalized.members.all()
            }
            Self::Pointwise(normalized) => members == normalized.members,
            Self::Contraction(normalized) => members == normalized.members,
            // The epilogue's own part, or any part of the producer's partition.
            // The chain as a whole is deliberately *not* a part: no scheduled
            // region computes a fold and an expression over its result, so a
            // cover grouping both has no spelling and must be declined rather
            // than resolved to this output.
            Self::Epilogue(chain) => {
                members == chain.members || chain.producer.owns_region_members(members)
            }
            // Every region whose atoms are all stages of this one occurrence —
            // which is [`NormalizedStaged::owns_stage_members`], and which states
            // why no stage list is enumerated here — or any part of the
            // producer's partition when an operand is a materialized
            // intermediate. The two are disjoint by construction: a producer's
            // atoms name a different occurrence, so no member set can be both.
            Self::Staged(normalized) => {
                normalized.owns_stage_members(members)
                    || normalized
                        .producer
                        .as_deref()
                        .is_some_and(|producer| producer.owns_region_members(members))
            }
        }
    }

    #[cfg(test)]
    fn serial_sum_mut(&mut self) -> &mut NormalizedSerialSum {
        match self {
            Self::SerialSum(normalized) => normalized,
            Self::Pointwise(_) | Self::Contraction(_) | Self::Epilogue(_) | Self::Staged(_) => {
                panic!("the fixture is a serial sum")
            }
        }
    }
}

/// The recognized program: one implementable region partition per ordered named
/// output, in the program's own declaration order.
///
/// **A list rather than one whole-program strategy, and the difference is what
/// makes several ordered outputs statable at all.** Recognition used to read
/// `outputs().next()`, classify that one occurrence, and require the resulting
/// walk to cover the program exactly, so a second declared output was either
/// outside the walk — leaving the program uncovered — or inside it, where one
/// region's owning write would have had to serve two publications. Each output
/// now carries its own walk, and the *program*-wide obligation moved to the
/// relation between them: the walks partition the occurrences, so every
/// occurrence is claimed exactly once and every published value has one region
/// that owns its write.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedProgram {
    outputs: Vec<NormalizedOutput>,
}

impl NormalizedProgram {
    /// Returns the recognized outputs in the program's declaration order.
    pub(crate) fn outputs(&self) -> &[NormalizedOutput] {
        &self.outputs
    }

    /// Returns the recognized output whose partition contains one region's exact
    /// member set, with its declaration position.
    ///
    /// This is the lookup every per-region authority below the boundary asks:
    /// the members are the region's own coverage, and the partition they belong
    /// to is what says which shape, domain, and expression the region realizes.
    /// A member set belonging to no output's partition is `None` — a region
    /// covering occurrences from two outputs' walks, or covering part of one
    /// part, has no recognized implementation and is refused by name rather than
    /// spelled against whichever partition happened to be first.
    ///
    /// **Two outputs may own one member set, and declaration order decides.**
    /// [`check_output_cover`] admits exactly one overlap — a walk that is one
    /// whole part of a longer walk's partition, publishing the value that part
    /// hands across the boundary — and both claimants are then recognitions of
    /// the same value over the same occurrences, so they resolve to the same
    /// region. [`crate::physical::spell_region`] carries the derivation and
    /// `both_claimants_of_a_published_and_consumed_part_spell_one_region` is the
    /// check that says no if that ever stops holding.
    pub(crate) fn output_for_region(
        &self,
        members: &[SemanticStage],
    ) -> Option<(usize, &NormalizedOutput)> {
        self.outputs
            .iter()
            .enumerate()
            .find(|(_, output)| output.owns_region_members(members))
    }

    /// Returns the recognized output at one declared position.
    pub(crate) fn output_at(&self, position: usize) -> Option<&NormalizedOutput> {
        self.outputs.get(position)
    }

    /// Returns the element count of one declared input tensor, when every
    /// recognized output that *reads* it agrees on the count.
    ///
    /// **Agreement or nothing, because the caller is sizing work.** The count
    /// scales a call over the tensor bound to that ordinal, and answering from
    /// whichever output claimed first would let one claimant's number stand for
    /// a tensor another claimant sizes differently — the confidently-wrong
    /// verdict a work-scaling resolution exists to prevent. A disagreement
    /// therefore yields `None` and the caller refuses, exactly as it does for an
    /// ordinal no input occupies.
    ///
    /// **Defence in depth rather than a live gate, and the distinction is worth
    /// stating.** Two outputs used to read one declared input at different
    /// domains — a reduction at its contributor shape, an elementwise sibling at
    /// its own — and this fold is what refused the pair. Since
    /// [`NormalizedOutput::input_elements_at`] answers the declared tensor's own
    /// count on every arm, that count is a function of the ordinal alone and no
    /// recognizable program reaches the refusal. The fold stays because the
    /// property it enforces is one an added arm or access relation could break,
    /// and a wrong work count is not a failure a later stage catches.
    /// `every_arm_answers_the_declared_tensors_own_count` records the reasoning
    /// and `a_bound_ordinal_resolves_from_the_output_that_reads_it` records the
    /// perturbation that makes this fold refuse again.
    ///
    /// **The fold ranges over the reading outputs, and it has to.** An output
    /// that never loads the ordinal has nothing to say about it, but [`agreed`]
    /// compares `Option`s, so a silent output's `None` is a *value* that
    /// disagrees with every count rather than an abstention — and a program
    /// whose two outputs iterate disjoint inputs would refuse every ordinal,
    /// each of which exactly one output sizes. Filtering them out before the
    /// fold is what makes silence cost nothing.
    ///
    /// The filter asks [`NormalizedOutput::reads_declared_input`] rather than
    /// whether the output produced a count, and the difference is load-bearing:
    /// an output that *does* read the ordinal and still cannot name one domain
    /// for it — an epilogue chain whose producer and epilogue read it at two
    /// domains — answers `None`, and that is a genuine disagreement the fold
    /// must keep. Filtering on the answer would drop exactly that refusal and
    /// let a sibling's count stand for a chain that has no single one.
    ///
    /// The two `None`s are flattened deliberately: "the reading outputs
    /// disagree" and "no output reads that ordinal" are different findings, and
    /// this accessor's caller acts identically on both — it refuses.
    pub(crate) fn agreed_input_elements_at(&self, ordinal: InputOrdinal) -> Option<u64> {
        agreed(
            self.outputs
                .iter()
                .filter(|output| output.reads_declared_input(ordinal.get()))
                .map(|output| output.input_elements_at(ordinal)),
        )
        .flatten()
    }

    /// Returns the published element count, when every recognized output agrees.
    ///
    /// Agreement for the reason [`Self::agreed_input_elements_at`] states: this
    /// count sizes work, and two outputs of different extents have no single
    /// answer to give.
    pub(crate) fn agreed_output_elements(&self) -> Option<u64> {
        agreed(self.outputs.iter().map(NormalizedOutput::output_elements))
    }

    /// Returns the largest declared input element count over every output.
    ///
    /// A structural proxy for the widest thing a plan for this request could
    /// stage, which is what `GovernedPhysicalProvider::propose`'s cost estimate
    /// wants. Deliberately a maximum rather than an agreement: a cost may be an
    /// upper bound over the whole request, and a cost that refused would turn an
    /// estimate into a feasibility gate. That one caller is the whole reason
    /// [`NormalizedOutput::max_input_elements`] substitutes a domain for a
    /// relation it cannot size instead of declining.
    pub(crate) fn max_input_elements(&self) -> u64 {
        self.outputs
            .iter()
            .map(NormalizedOutput::max_input_elements)
            .max()
            .unwrap_or_default()
    }

    /// Returns the largest published element count over every output.
    ///
    /// A maximum for the reason [`Self::max_input_elements`] is one: its callers
    /// are structural cost estimates, never feasibility.
    pub(crate) fn max_output_elements(&self) -> u64 {
        self.outputs
            .iter()
            .map(NormalizedOutput::output_elements)
            .max()
            .unwrap_or_default()
    }

    /// Returns every attribution atom any output's walk claimed, ascending.
    ///
    /// The walks partition the program's occurrences — [`check_output_cover`]
    /// proves it — so this is the program's whole operation set and the
    /// deduplication is the invariant being relied on rather than a repair.
    pub(crate) fn all_members(&self) -> Vec<SemanticStage> {
        let mut members: Vec<SemanticStage> = self
            .outputs
            .iter()
            .flat_map(NormalizedOutput::members)
            .collect();
        members.sort_unstable();
        members.dedup();
        members
    }

    /// Returns every *occurrence* any output's walk claimed, ascending.
    ///
    /// The projection of [`Self::all_members`] onto operations, for the
    /// authorities whose subject is the occurrence rather than a stage of it:
    /// an occurrence resolves one lowering capability and carries one refinement
    /// receipt however many regions realize it, so asking per atom would resolve
    /// one capability twice and mint two receipts for one proof obligation.
    pub(crate) fn all_occurrences(&self) -> Vec<SemanticMemberId> {
        let mut members: Vec<SemanticMemberId> = self
            .all_members()
            .into_iter()
            .map(SemanticStage::member)
            .collect();
        members.dedup();
        members
    }

    #[cfg(test)]
    fn serial_sum_mut(&mut self) -> &mut NormalizedSerialSum {
        let [output] = self.outputs.as_mut_slice() else {
            panic!("the fixture declares one output");
        };
        output.serial_sum_mut()
    }
}

/// Returns the element count of the tensor one recognized elementwise read
/// addresses, given the domain of the region performing it.
///
/// **The tensor's own count, which is what separates it from the domain.** An
/// elementwise region's read list carries a relation per read, and a widening
/// one addresses fewer elements than the region iterates: `broadcast(w)` over a
/// `[2]` weight into a `[2, 2]` domain reads two elements four times. Callers
/// sizing a call over the buffer the ABI binds want the two, so the count is
/// taken from the relation rather than from the region.
///
/// A dense read answers the domain because the recognizer made them equal:
/// [`plan_elementwise`] refuses an elementwise leaf whose shape is not the
/// region's, so the domain *is* that tensor's count rather than standing in for
/// it. A structural read answers its operand shape, and
/// [`recognize_structural_read`] admits a structural occurrence only over a
/// value this walk reads — a declared input or the staged value — so that
/// operand shape is the declared tensor's own.
///
/// **The wildcard declines rather than guesses.** [`LogicalAccess`] is
/// `#[non_exhaustive]`, so a relation added upstream reaches it; naming a count
/// for a relation whose operand extent is unknown is exactly the confidently
/// wrong work count a refusal exists to prevent. The named arms are the ones
/// that would be reached first and are listed so a reader can see which
/// relations are being declined on purpose.
fn read_tensor_elements(map: &LogicalAccess, domain_elements: u64) -> Option<u64> {
    match map {
        LogicalAccess::LinearIdentity => Some(domain_elements),
        // The overflow refusal is unreachable through a recognized program:
        // `recognize_structural_read` took this shape from the declared value,
        // and `element_count_u64` already multiplied the same extents when the
        // shape's own arm minted its count. It declines rather than saturating
        // because a saturated count is a work count nothing derived.
        LogicalAccess::ReindexBijection { operand_shape, .. }
        | LogicalAccess::BroadcastReplication { operand_shape, .. } => {
            tiler_ir::schedule::element_count(operand_shape).ok()
        }
        LogicalAccess::ScalarBroadcast
        | LogicalAccess::PackedU4LsbZeroTail { .. }
        | LogicalAccess::ReductionContributor { .. }
        | LogicalAccess::ContractionOperand { .. }
        | _ => None,
    }
}

/// Returns the one value every entry carries, or `None` when they disagree.
///
/// An empty sequence answers `None` rather than a vacuous value: a program with
/// no recognized output has nothing to report, and reporting a default would be
/// an answer nothing derived.
fn agreed<T: Eq>(values: impl IntoIterator<Item = T>) -> Option<T> {
    let mut values = values.into_iter();
    let first = values.next()?;
    values.all(|value| value == first).then_some(first)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedCompilationRequest {
    normalized: NormalizedProgram,
    semantic_identity: SemanticIdentity,
    numerical_contracts: NumericalContractPreference,
    budgets: DeterministicBudgets,
    /// Ordered target receipts minted at verification.
    ///
    /// Profile, resolved contract, and authority travel as one slot so no later
    /// stage can recover their association by comparing whole profile values or
    /// by indexing several parallel vectors.
    target_slots: Vec<VerifiedTargetSlot>,
    capabilities: CompilerCapabilitySnapshot,
    realization_laws: FrozenIndexRealizationLawRegistry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedTargetSlot {
    target_profile: TargetProfile,
    resolution: VerifiedTargetResolution,
}

/// Contract resolution retained for one structurally admitted target.
///
/// A target that cannot honour any stated contract is still a verified member
/// of the request. Keeping that outcome in its ordered slot lets later
/// orchestration report it beside successful companions instead of aborting the
/// batch before those companions are considered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VerifiedTargetResolution {
    Resolved {
        numerical_contract: StrictF32NumericalContract,
        authority: Box<VerifiedRequestSubject>,
    },
    Rejected(RequestError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedTargetRequest {
    normalized: NormalizedProgram,
    semantic_identity: SemanticIdentity,
    numerical_contracts: NumericalContractPreference,
    numerical_contract: StrictF32NumericalContract,
    budgets: DeterministicBudgets,
    target_profile: TargetProfile,
    capabilities: CompilerCapabilitySnapshot,
    realization_laws: FrozenIndexRealizationLawRegistry,
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
    normalized: NormalizedProgramSubject,
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
    target_profile: TargetProfile,
    capability_schema_version: u32,
    lowering_registry: CanonicalLoweringRegistryIdentity,
    realization_registry: Box<[u8]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedSerialSumSubject {
    input_keys: Vec<InputKey>,
    output_key: OutputKey,
    input_shape: Shape,
    output_shape: Shape,
    reduction_axes: Vec<Axis>,
    prologue: Option<PointwiseF32Expression>,
    prologue_reads: Vec<(u32, LogicalAccess)>,
    contributor_input: Option<u32>,
    members: RecognizedSerialSumMembers,
    input_elements: u64,
    output_elements: u64,
}

/// The subject projection of one recognized ordered named output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NormalizedOutputSubject {
    SerialSum(NormalizedSerialSumSubject),
    Pointwise(NormalizedPointwise),
    /// Boxed for the reason [`NormalizedOutput::Contraction`] is.
    Contraction(Box<NormalizedContraction>),
    /// Boxed for the reason [`NormalizedOutput::Epilogue`] is.
    Epilogue(Box<NormalizedEpilogueSubject>),
    /// Boxed for the reason [`NormalizedOutput::Staged`] is.
    ///
    /// The occurrence's recognized shape is carried whole rather than projected:
    /// it holds no graph handle a subject must not bind, because the occurrence
    /// coordinate it does carry is the graph-local member ordinal every other
    /// arm's member run already writes. Its producer is the one part that *is*
    /// projected, into the subject's own recursive slot, for the reason
    /// [`NormalizedEpilogueSubject`] projects a chain's.
    Staged(Box<NormalizedStagedSubject>),
}

/// The subject projection of one recognized staged family.
///
/// It carries the producer's own subject rather than a summary of it, for the
/// reason [`NormalizedEpilogueSubject`] does: a region of the producer's
/// partition binds against exactly the subject it would bind against if the
/// producer were the whole declared output, so [`crate::physical`]'s binding
/// recurses instead of restating each producing family's obligations again.
///
/// **The occurrence copy's own [`NormalizedStaged::producer`] slot is cleared,
/// and the clearing is what keeps one fact in one place.** Carrying the producer
/// both as a recognized shape and as a subject would be two accounts of one
/// value, free to disagree; the recognized side is [`NormalizedProgram`]'s and
/// the subject side is this one's.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedStagedSubject {
    occurrence: Box<NormalizedStaged>,
    producer: Option<Box<NormalizedOutputSubject>>,
}

impl NormalizedStagedSubject {
    /// Returns the occurrence's own recognized shape.
    ///
    /// Its producer slot is cleared; [`Self::producer`] is where the producer
    /// travels.
    pub(crate) const fn occurrence(&self) -> &NormalizedStaged {
        &self.occurrence
    }

    /// Returns the producer subject a region of the producing partition binds
    /// against, or `None` when every operand binds a declared input.
    pub(crate) fn producer(&self) -> Option<&NormalizedOutputSubject> {
        self.producer.as_deref()
    }
}

/// The subject projection of one recognized elementwise epilogue chain.
///
/// It carries the producer's own subject rather than a summary of it, so a
/// region of the producer's partition binds against exactly the subject it would
/// bind against if the producer were the whole declared output — which is what
/// lets [`crate::physical`]'s binding recurse instead of restating each
/// producing family's obligations a second time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedEpilogueSubject {
    producer: Box<NormalizedOutputSubject>,
    input_keys: Vec<InputKey>,
    output_key: OutputKey,
    shape: Shape,
    expression: PointwiseF32Expression,
    reads: Vec<(BoundaryRead, LogicalAccess)>,
    members: Vec<SemanticStage>,
    elements: u64,
}

impl NormalizedEpilogueSubject {
    /// Returns the producer subject a region of the producing partition binds
    /// against.
    pub(crate) fn producer(&self) -> &NormalizedOutputSubject {
        &self.producer
    }
    /// Returns the epilogue region's iteration domain.
    pub(crate) const fn shape(&self) -> &Shape {
        &self.shape
    }
    /// Returns the recognized epilogue expression.
    pub(crate) const fn expression(&self) -> &PointwiseF32Expression {
        &self.expression
    }
    /// Returns the epilogue region's reads, in access order.
    pub(crate) fn reads(&self) -> &[(BoundaryRead, LogicalAccess)] {
        &self.reads
    }
    /// Returns the occurrences the epilogue region itself covers.
    pub(crate) fn members(&self) -> &[SemanticStage] {
        &self.members
    }
    /// Returns the epilogue region's published element count.
    pub(crate) const fn elements(&self) -> u64 {
        self.elements
    }
}

/// The recognized program as the request subject records it: one per ordered
/// named output, in declaration order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedProgramSubject {
    outputs: Vec<NormalizedOutputSubject>,
}

impl NormalizedProgramSubject {
    /// Returns the recognized output subjects in declaration order.
    pub(crate) fn outputs(&self) -> &[NormalizedOutputSubject] {
        &self.outputs
    }
}

impl VerifiedTargetRequest {
    pub(crate) const fn normalized(&self) -> &NormalizedProgram {
        &self.normalized
    }

    /// Returns the recognized output implementing one cover region's members.
    ///
    /// Every per-region authority below this boundary asks this rather than
    /// asking the request for "the" strategy: with several declared outputs
    /// there is no such thing, and a region's members are exactly the fact that
    /// says which output's partition it belongs to.
    pub(crate) fn output_for_region(
        &self,
        members: &[SemanticStage],
    ) -> Option<(usize, &NormalizedOutput)> {
        self.normalized.output_for_region(members)
    }

    /// Returns the recognized output at one declared position.
    ///
    /// # Panics
    ///
    /// Panics when the position names no declared output, which is invalid
    /// compiler output rather than a caller error: every position handed to
    /// this accessor came from [`Self::output_for_region`] resolving a region
    /// this same request recognized.
    pub(crate) fn output_at(&self, position: usize) -> &NormalizedOutput {
        self.normalized
            .output_at(position)
            .expect("a resolved output position names a recognized output")
    }

    /// Returns the one recognized output of a single-output request.
    ///
    /// **No compile-path derivation reads this, and the `output-arity` guard
    /// that used to justify it is gone.** Relaxing that guard surfaced no caller
    /// to convert, which is the fact worth recording: every per-region authority
    /// on the compile path already resolves through
    /// [`crate::physical::spell_region`] and [`Self::output_at`], and the two
    /// whole-program constructors that still call this —
    /// [`crate::physical::build_scheduled_regions`] and
    /// [`crate::physical::build_fused_scheduled_region`] — are retained as the
    /// single definition of each canonical region and are reached only from
    /// tests. This accessor is what lets those, and the fixtures around them,
    /// name a one-output program's shape without repeating a destructuring.
    ///
    /// # Panics
    ///
    /// Panics for a request whose program declares other than one declared
    /// output. That is now a *reachable* state — the boundary admits ordered
    /// multi-output programs — so the panic is the guarantee: a fixture or
    /// constructor that grows a second output fails loudly here rather than
    /// silently asserting about the first.
    pub(crate) fn sole_output(&self) -> &NormalizedOutput {
        let [output] = self.normalized.outputs() else {
            panic!("this derivation is for a request declaring exactly one output");
        };
        output
    }

    /// The sole recognized output's serial-sum shape, for fixtures.
    ///
    /// `#[cfg(test)]`, and the three below with it. Compile-path code resolves
    /// the output a region belongs to through [`Self::output_for_region`]; these
    /// exist so a fixture that built a one-output program can name its shape
    /// without repeating `sole_output()` at every assertion. They carry the
    /// same panic as [`Self::sole_output`], which is what makes a fixture that
    /// grew a second output fail loudly rather than assert about the first.
    #[cfg(test)]
    pub(crate) fn serial_sum(&self) -> &NormalizedSerialSum {
        self.sole_output().serial_sum()
    }

    #[cfg(test)]
    pub(crate) fn pointwise(&self) -> Option<&NormalizedPointwise> {
        self.sole_output().pointwise()
    }

    #[cfg(test)]
    pub(crate) fn contraction(&self) -> Option<&NormalizedContraction> {
        self.sole_output().contraction()
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
            &self.target_profile,
            VerifiedRequestAuthorities {
                installed: &self.capabilities,
                realization_laws: &self.realization_laws,
            },
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

    pub(crate) const fn target_profile(&self) -> &TargetProfile {
        &self.target_profile
    }

    pub(crate) const fn capabilities(&self) -> &CompilerCapabilitySnapshot {
        &self.capabilities
    }

    pub(crate) const fn realization_laws(&self) -> &FrozenIndexRealizationLawRegistry {
        &self.realization_laws
    }

    pub(crate) const fn semantic_identity(&self) -> &SemanticIdentity {
        &self.semantic_identity
    }

    /// Rebinds only the profile for downstream tamper-check fixtures.
    #[cfg(test)]
    pub(crate) fn with_target_profile_for_test(mut self, target_profile: TargetProfile) -> Self {
        self.target_profile = target_profile;
        self
    }
}

impl VerifiedRequestSubject {
    pub(crate) const fn normalized(&self) -> &NormalizedProgramSubject {
        &self.normalized
    }

    pub(crate) const fn numerical_contract(&self) -> StrictF32NumericalContract {
        self.numerical_contract
    }

    pub(crate) fn canonical_explain_subject_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // The enclosing domain steps to `v6` because `SemanticIdentity` gained
        // its fifth subject, the shape environment, and this preimage
        // enumerates the subject set positionally: a `v5` reader would take the
        // environment identity's length frame for the output count. An
        // appends-only argument does not close for a field written *before* the
        // count, so this is a domain step.
        //
        // The earlier step to `v5` because the recognized program
        // became a *list* — one implementable region partition per ordered named
        // output — and the list is length-framed ahead of the arms. A `v4`
        // subject encoded exactly one arm with no count, so its first
        // post-identity byte is the arm sub-tag's own length frame while a `v5`
        // subject's is the output count. Nothing rules out a count that happens
        // to frame like a sub-tag length, so this is a domain step rather than
        // an appends-only re-tag: the per-tag injectivity argument that would
        // license the cheaper option does not close, and half a step is worse
        // than none.
        //
        // The earlier step to `v4` because the installed independent
        // semantic-realization authority now participates after lowering
        // authority. A v3 subject did not encode that field at all.
        //
        // The earlier step to `v3` rather than only the per-arm
        // sub-tags, because this recognizer moved two of the three arms' shapes
        // at once *and* gave the serial-sum arm its first sub-tag. A same-domain
        // re-tag would have to argue that a newly tagged arm cannot be read as
        // the untagged one it replaced — the old arm opened with a length-framed
        // input key, and a caller may name an input whatever it likes — and that
        // argument does not close. Stepping the domain makes the separation
        // structural instead.
        bytes.extend_from_slice(b"tiler.compiler.request-subject.v6\0");
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
        push_slice(
            &mut bytes,
            self.semantic_identity.shape_environment().as_bytes(),
        );
        // The ordered named outputs, counted then written in declaration order.
        // The count is what keeps a two-output subject from framing like a
        // one-output subject followed by the contract encoding, and the order is
        // identity rather than presentation: two programs differing only in
        // which output is declared first are different programs, and the
        // semantic graph identity above already says so.
        push_len(&mut bytes, self.normalized.outputs.len());
        for normalized in &self.normalized.outputs {
            encode_output_subject(&mut bytes, normalized);
        }
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
        bytes.extend_from_slice(self.target_profile.request_subject_bytes());
        bytes.extend_from_slice(&self.capability_schema_version.to_be_bytes());
        push_slice(&mut bytes, self.lowering_registry.as_bytes());
        push_slice(&mut bytes, &self.realization_registry);
        bytes
    }
}

/// Appends one recognized output's complete canonical subject encoding.
///
/// Recursive because an epilogue chain's producer is itself a recognized output;
/// every other arm is flat. The recursion is bounded by the recognizer, which
/// admits a folding family as a chain's producer and nothing else, so a chain of
/// chains is not a subject this function can be handed.
///
/// **Every member run below writes the occurrence ordinal of an attribution
/// atom and not its stage, and that is complete rather than lossy.** Every
/// recognized part in this module is minted at [`SemanticStage::first`] —
/// `plan_elementwise`, [`RecognizedSerialSumMembers::new`], the contraction
/// recognizer, and [`recognize_staged_family`] each mint one — so no two
/// subjects can differ in a stage ordinal and the ordinals alone separate them.
///
/// That holds for a family realized as a region *sequence* too, and the reason
/// is the recognizer's rather than this encoder's: a staged family's stage split
/// is region formation's to enumerate, so the recognized partition names the
/// occurrence and [`NormalizedOutput::owns_region_members`] answers for the
/// atoms formation minted. A recognizer that instead enumerated stages into a
/// partition would break the premise and would have to fold the stage into these
/// runs, which steps this domain's version and moves every pinned request
/// qualifier with it.
fn encode_output_subject(bytes: &mut Vec<u8>, normalized: &NormalizedOutputSubject) {
    match normalized {
        NormalizedOutputSubject::SerialSum(normalized) => {
            // **The sub-tag holds at `v3` although the arm gained an
            // absent prologue, and the forced-not-chosen standard is what
            // decides that.** A prologue is written below as its framed
            // node run, and
            // `tiler_ir::schedule::PointwiseF32ExpressionBuilder::build`
            // refuses an expression with no node — so every subject this
            // arm could encode before carries a node count of at least
            // one at that position. Writing the absent prologue as a
            // framed *zero* therefore occupies a byte string no
            // previously encodable subject can produce, and the run stays
            // self-delimiting: a count of zero ends the prologue and a
            // count of `n` is followed by exactly `n` nodes and the root.
            // Per-tag injectivity closes, no already-encodable subject's
            // bytes move, and a step would restate every pin for a
            // separation the encoding already has.
            //
            // The earlier step to `v3` was forced, and by the shape this
            // one is not: the access-relation run is written at the arm's
            // *end*, so a `v2` subject and a `v3` one carrying no maps
            // would have differed only by a trailing framed zero, and a
            // reader with the old framing would have consumed the
            // following output's tag as this arm's payload.
            push_slice(bytes, b"serial-sum-f32.v3");
            push_len(bytes, normalized.input_keys.len());
            for key in &normalized.input_keys {
                push_slice(bytes, key.as_str().as_bytes());
            }
            push_slice(bytes, normalized.output_key.as_str().as_bytes());
            encode_explain_shape(bytes, &normalized.input_shape);
            encode_explain_shape(bytes, &normalized.output_shape);
            push_len(bytes, normalized.reduction_axes.len());
            for axis in &normalized.reduction_axes {
                bytes.extend_from_slice(&axis.get().to_be_bytes());
            }
            match &normalized.prologue {
                Some(prologue) => encode_pointwise_expression(bytes, prologue),
                None => push_len(bytes, 0),
            }
            for members in [
                normalized.members.pointwise(),
                normalized.members.reduction(),
            ] {
                push_len(bytes, members.len());
                for atom in members {
                    bytes.extend_from_slice(&atom.member().0.to_be_bytes());
                }
            }
            bytes.extend_from_slice(&normalized.input_elements.to_be_bytes());
            bytes.extend_from_slice(&normalized.output_elements.to_be_bytes());
            // The declared inputs this fold's own regions read: the prologue
            // region's read list, or — when there is no prologue region — the
            // fold's own contributor read. One run rather than two fields
            // because what the arm must separate is *which* declared inputs
            // this output reads, and `sum(a)` and `sum(b)` over the same two
            // declarations agree on every other field here.
            //
            // The contributor read's relation is spelled `LinearIdentity`
            // rather than the `ReductionContributor` the region carries, and
            // that is not one fact encoded twice: the arm has already written
            // the contributor domain, the published domain, and the canonical
            // reduction axes, which is everything that relation is derived
            // from. What the entry contributes is the ordinal — and the run
            // omits a lone dense read, so what reaches the bytes is the marker
            // for each declared input the fold does not read.
            let contributor = normalized
                .contributor_input
                .map(|ordinal| [(ordinal, LogicalAccess::LinearIdentity)]);
            let reads = contributor
                .as_ref()
                .map_or(normalized.prologue_reads.as_slice(), <[_; 1]>::as_slice);
            encode_elementwise_reads(bytes, normalized.input_keys.len(), reads);
        }
        NormalizedOutputSubject::Pointwise(normalized) => {
            // The sub-tag steps to `v4` because the arm gained each read's
            // access relation, and that fact is *load-bearing for
            // identity*: `a * w` with both inputs declared at the region's
            // shape and `a * broadcast(w)` widening a smaller `w` encode
            // the same input keys, the same result shape, the same
            // expression, and the same element count. Only the access maps
            // separate them, so a subject that omitted them would give two
            // different programs one identity — and leaning on the member
            // list to separate them would be exactly the unstated invariant
            // an identity encoder must not rest on.
            //
            // `v3` stepped because a fixed root family, child family,
            // association, and three leaves became the general expression
            // the recognizer now admits. A `v2` pointwise subject can never
            // be read as a `v3` one, and a `v3` one can never be read as a
            // `v4`.
            //
            // **A `bf16` program takes its own sub-tag rather than stepping
            // this one**, on the contraction, epilogue, and staged arms'
            // argument: an `f32` pointwise subject still encodes to exactly
            // the bytes it did, byte for byte, so no pinned request
            // qualifier moves for a program this vocabulary could already
            // express, and a reader that reaches `pointwise-bf16.v1` is
            // reading a subject the earlier vocabulary could not state.
            // The arithmetic is *in the tag* rather than beside it because
            // the node run that follows is a different vocabulary — sixteen
            // bit constants and four node kinds against thirty-two and seven
            // — so the two runs are not two spellings of one encoding.
            push_slice(
                bytes,
                match &normalized.expression {
                    RecognizedPointwise::F32(_) => b"pointwise-f32.v4".as_slice(),
                    RecognizedPointwise::Bf16(_) => b"pointwise-bf16.v1".as_slice(),
                },
            );
            push_len(bytes, normalized.input_keys.len());
            for key in &normalized.input_keys {
                push_slice(bytes, key.as_str().as_bytes());
            }
            push_slice(bytes, normalized.output_key.as_str().as_bytes());
            encode_explain_shape(bytes, &normalized.shape);
            match &normalized.expression {
                RecognizedPointwise::F32(expression) => {
                    encode_pointwise_expression(bytes, expression);
                }
                RecognizedPointwise::Bf16(expression) => {
                    encode_pointwise_bf16_expression(bytes, expression);
                }
            }
            push_len(bytes, normalized.members.len());
            for atom in &normalized.members {
                bytes.extend_from_slice(&atom.member().0.to_be_bytes());
            }
            bytes.extend_from_slice(&normalized.elements.to_be_bytes());
            encode_elementwise_reads(bytes, normalized.input_keys.len(), &normalized.reads);
        }
        // A third sub-tag rather than a step of the enclosing
        // `request-subject.v2` domain: neither existing arm's bytes move, so
        // a subject encoded before this variant existed still encodes to
        // exactly what it did, and a reader that reaches this tag is reading
        // a subject the earlier vocabulary could not express.
        NormalizedOutputSubject::Contraction(normalized) => {
            push_slice(bytes, b"contraction-f32.v1");
            push_len(bytes, normalized.input_keys.len());
            for key in &normalized.input_keys {
                push_slice(bytes, key.as_str().as_bytes());
            }
            push_slice(bytes, normalized.output_key.as_str().as_bytes());
            // Exactly two declarations made every previously admitted subject's
            // distinct, ascending read ordinals recoverably `0, 1`. Keep that
            // branch byte-for-byte unchanged. A wider declaration was refused
            // before this change, and its earlier framed key count selects this
            // fixed two-ordinal run unambiguously; the run separates every
            // two-input subset without moving `contraction-f32.v1`'s old bytes.
            if normalized.input_keys.len() > normalized.reads.len() {
                for read in &normalized.reads {
                    bytes.extend_from_slice(&read.input_ordinal.to_be_bytes());
                }
            }
            for read in &normalized.reads {
                encode_explain_shape(bytes, &read.shape);
            }
            encode_explain_shape(bytes, &normalized.output_shape);
            encode_explain_shape(bytes, &normalized.contracted_shape);
            // The canonical structure encoding, not a projection of it: the
            // index tuples are what ADR 0087 makes the operation's identity,
            // and two structures over one set of shapes are two programs.
            push_slice(bytes, normalized.structure.canonical_encoding().as_bytes());
            for read in &normalized.reads {
                push_len(bytes, read.operand_position);
            }
            push_len(bytes, normalized.members.len());
            for atom in &normalized.members {
                bytes.extend_from_slice(&atom.member().0.to_be_bytes());
            }
            for read in &normalized.reads {
                bytes.extend_from_slice(&read.elements.to_be_bytes());
            }
            bytes.extend_from_slice(&normalized.output_elements.to_be_bytes());
            bytes.extend_from_slice(&normalized.contracted_elements.to_be_bytes());
        }
        // A fourth sub-tag rather than a step of the enclosing
        // `request-subject.v5` domain, and the argument is the contraction
        // arm's: no existing arm's bytes move, so a subject encoded before this
        // variant existed still encodes to exactly what it did, and a reader
        // that reaches this tag is reading a subject the earlier vocabulary
        // could not express. The nested producer is written through this same
        // function, so a chain's producer encodes exactly as the standalone
        // output of that family would — which is what keeps the two spellings of
        // one fold from acquiring two identities.
        NormalizedOutputSubject::Epilogue(normalized) => {
            push_slice(bytes, b"epilogue-f32.v1");
            push_len(bytes, normalized.input_keys.len());
            for key in &normalized.input_keys {
                push_slice(bytes, key.as_str().as_bytes());
            }
            push_slice(bytes, normalized.output_key.as_str().as_bytes());
            encode_explain_shape(bytes, &normalized.shape);
            encode_pointwise_expression(bytes, &normalized.expression);
            // The read list, in access order, each entry naming the boundary
            // tensor it binds and the relation it addresses it with. Both halves
            // are identity: two chains whose epilogues read the same leaves in a
            // different order are different regions, and a staged read and a
            // declared-input read at the same position bind different buffers.
            push_len(bytes, normalized.reads.len());
            for (read, map) in &normalized.reads {
                match read {
                    BoundaryRead::Staged => bytes.push(0x01),
                    BoundaryRead::Input(ordinal) => {
                        bytes.push(0x02);
                        bytes.extend_from_slice(&ordinal.to_be_bytes());
                    }
                }
                encode_access_relation(bytes, map);
            }
            push_len(bytes, normalized.members.len());
            for atom in &normalized.members {
                bytes.extend_from_slice(&atom.member().0.to_be_bytes());
            }
            bytes.extend_from_slice(&normalized.elements.to_be_bytes());
            encode_output_subject(bytes, &normalized.producer);
        }
        // A fifth sub-tag, on the contraction and epilogue arms' argument: no
        // existing arm's bytes move, so a subject encoded before this variant
        // existed still encodes to exactly what it did, and a reader that
        // reaches this tag is reading a subject the earlier vocabulary could not
        // express. The enclosing domain therefore does not step and no pinned
        // request qualifier moves.
        //
        // **The operation key and the attribute record are both identity, and
        // neither is redundant with the other.** Two families realized as region
        // sequences over the same shapes differ only in the key; two occurrences
        // of *one* family differ only in the record — `tiler::rms-norm-f32@1`'s
        // reduced axis and exact `eps` payload live there and are part of what
        // the occurrence computes. The record is written through
        // [`crate::region::encode_attributes`], the same canonical encoder
        // region content identity uses, so the two never disagree about what an
        // attribute value is.
        NormalizedOutputSubject::Staged(normalized) => {
            let occurrence = normalized.occurrence();
            // **The sub-tag steps to `v2`, and the step is forced.** An operand
            // entry used to open with its declared input ordinal and now opens
            // with the boundary-role tag that says whether there *is* one, so
            // every byte string this arm could already produce moves. The
            // per-tag injectivity argument that licenses a same-domain re-tag —
            // "no already-encodable subject's bytes move" — therefore does not
            // close, and half a step is worse than none.
            //
            // Only this arm's bytes move. The enclosing
            // `tiler.compiler.request-subject.v5` domain does not step, because
            // a program naming no staged family encodes exactly what it did, and
            // no pinned request qualifier encodes a staged subject:
            // `explain`'s `deterministic_trace_is_sealed_and_rendered_separately`
            // qualifies a multiply and `tiler-build`'s standard Metal goldens
            // qualify a reduction.
            push_slice(bytes, b"staged-family.v2");
            push_slice(bytes, occurrence.operation.namespace().as_bytes());
            push_slice(bytes, occurrence.operation.name().as_bytes());
            bytes.extend_from_slice(&occurrence.operation.semantic_version().to_be_bytes());
            push_slice(bytes, &occurrence.attributes);
            push_len(bytes, occurrence.input_keys.len());
            for key in &occurrence.input_keys {
                push_slice(bytes, key.as_str().as_bytes());
            }
            push_slice(bytes, occurrence.output_key.as_str().as_bytes());
            // The operand run: which boundary tensor supplies each operand, at
            // which shape and element count. Position is identity because the
            // family reads its operands by position — `rms_norm(x, w)` and
            // `rms_norm(w, x)` are different programs — the ordinals are the
            // program's own, which is what the ABI binds, and the role tag is
            // identity because `rms_norm(x, w)` and `rms_norm(matmul(a, b), w)`
            // agree on every other field of this entry.
            //
            // The two tags are the epilogue arm's, for the reason they are one
            // vocabulary rather than two: the same [`BoundaryRead`] is written,
            // and the arm's own sub-tag separates the two runs before either is
            // read.
            push_len(bytes, occurrence.operand_reads.len());
            for ((read, shape), elements) in occurrence
                .operand_reads
                .iter()
                .zip(&occurrence.operand_shapes)
                .zip(&occurrence.operand_elements)
            {
                match read {
                    BoundaryRead::Staged => bytes.push(0x01),
                    BoundaryRead::Input(ordinal) => {
                        bytes.push(0x02);
                        bytes.extend_from_slice(&ordinal.to_be_bytes());
                    }
                }
                encode_explain_shape(bytes, shape);
                bytes.extend_from_slice(&elements.to_be_bytes());
            }
            encode_explain_shape(bytes, &occurrence.output_shape);
            bytes.extend_from_slice(&occurrence.member.0.to_be_bytes());
            bytes.extend_from_slice(&occurrence.output_elements.to_be_bytes());
            // The producer, present exactly when some operand above is staged,
            // written through this same function so it encodes exactly as the
            // standalone output of its family would — the epilogue arm's own
            // property, and what keeps two spellings of one contraction from
            // acquiring two identities. The presence byte leads so the arm stays
            // self-delimiting.
            match normalized.producer() {
                Some(producer) => {
                    bytes.push(0x01);
                    encode_output_subject(bytes, producer);
                }
                None => bytes.push(0x00),
            }
        }
    }
}

/// Appends one numerical contract's complete canonical encoding.
///
/// Complete over every dimension and exhaustive per dimension: each dimension is
/// written through [`StrictF32NumericalContract::behaviour`] and
/// [`DimensionBehaviour::encode`], whose matches are exhaustive over every
/// behaviour space, and the dimensions are walked in
/// [`crate::target::honourability::CANONICAL_DIMENSIONS`] order. The contract key is
/// encoded beside the field values it names and never in place of them, and the
/// arithmetic type keying every resolution is encoded too — two contracts that
/// resolve the same dimensions for different dtypes are different contracts
/// (ADR 0076 item 6).
///
/// Walking the canonical order rather than listing fields is what makes adding a
/// dimension a build error at `behaviour` instead of a silent omission here.
fn encode_contract(bytes: &mut Vec<u8>, contract: StrictF32NumericalContract) {
    push_slice(bytes, contract.key.as_bytes());
    bytes.push(contract.arithmetic.tag());
    bytes.extend_from_slice(&contract.canonical_arithmetic_nan_bits.to_be_bytes());
    push_len(
        bytes,
        crate::target::honourability::CANONICAL_DIMENSIONS.len(),
    );
    for dimension in crate::target::honourability::CANONICAL_DIMENSIONS {
        bytes.push(dimension.tag());
        contract.behaviour(dimension).encode(bytes);
    }
}

// The transform-permission tag is `tiler_ir::numerics`'. It was duplicated
// here and in the artifact's own encoder; the request subject,
// the target-profile descriptor, and the delivered-realization record all
// encode the same behaviours, so one definition is what keeps them from
// drifting. Both remain exhaustive matches rather than discriminant casts, for
// the reason the relocated definitions record: a cast reads whatever ordinal a
// variant happens to occupy, so adding or reordering one would silently restate
// every encoded subject (ADR 0074 convention 5b).
pub(crate) use tiler_ir::numerics::permission_tag;

/// Appends one recognized elementwise expression's complete canonical encoding.
///
/// **Complete, and structural rather than summarized.** The node run is written
/// in the expression's own canonical order with each node's operand ordinals, so
/// two expressions that differ in association, in which leaf an operand reads,
/// or in the sharing of a subexpression encode differently — all three are
/// different binary32 functions, and a subject that could not tell them apart
/// would let one artifact stand for two programs.
///
/// The per-node tag is an exhaustive match rather than a discriminant cast, for
/// the reason the relocated tag encoders record: a node added to the
/// vocabulary must stop
/// the build here rather than silently encode under a neighbour's tag.
///
/// **The leading count is never zero**, because a `PointwiseF32Expression` is
/// constructible only through a builder that refuses an empty node run. The
/// serial-sum subject arm relies on that to spell an *absent* prologue as a
/// framed zero without a sub-tag step, so a vocabulary change admitting a
/// node-free expression would have to move that arm's tag.
fn encode_pointwise_expression(bytes: &mut Vec<u8>, expression: &PointwiseF32Expression) {
    push_len(bytes, expression.nodes().len());
    for node in expression.nodes() {
        match node {
            PointwiseF32Node::Input { ordinal } => {
                bytes.push(0x01);
                bytes.extend_from_slice(&ordinal.get().to_be_bytes());
            }
            PointwiseF32Node::Constant { bits } => {
                bytes.push(0x02);
                bytes.extend_from_slice(&bits.to_be_bytes());
            }
            PointwiseF32Node::Add { lhs, rhs } => encode_binary_node(bytes, 0x03, *lhs, *rhs),
            PointwiseF32Node::Multiply { lhs, rhs } => encode_binary_node(bytes, 0x04, *lhs, *rhs),
            PointwiseF32Node::Divide { lhs, rhs } => encode_binary_node(bytes, 0x05, *lhs, *rhs),
            PointwiseF32Node::Exp { argument } => {
                bytes.push(0x06);
                bytes.extend_from_slice(&argument.index().to_be_bytes());
            }
            PointwiseF32Node::Rsqrt { argument } => {
                bytes.push(0x07);
                bytes.extend_from_slice(&argument.index().to_be_bytes());
            }
        }
    }
    bytes.extend_from_slice(&expression.root().index().to_be_bytes());
}

/// Appends one recognized `bf16` elementwise expression's canonical encoding.
///
/// Structurally the same run [`encode_pointwise_expression`] writes and
/// deliberately not the same function: the node vocabularies are different types
/// with different payload widths, and a shared encoder would have to erase one of
/// them to a common shape. The tags overlap by design — the two runs never share
/// a byte string because the arm's own sub-tag separates them before either is
/// read — and a `bf16` constant is written as its sixteen bits rather than
/// widened, so two constants differing only above bit fifteen cannot exist to be
/// confused.
///
/// Written as an exhaustive match for the reason its `f32` sibling is: a node
/// added to the `bf16` vocabulary must stop the build here rather than encode
/// under a neighbour's tag.
fn encode_pointwise_bf16_expression(bytes: &mut Vec<u8>, expression: &PointwiseBf16Expression) {
    push_len(bytes, expression.nodes().len());
    for node in expression.nodes() {
        match node {
            tiler_ir::schedule::PointwiseBf16Node::Input { ordinal } => {
                bytes.push(0x01);
                bytes.extend_from_slice(&ordinal.get().to_be_bytes());
            }
            tiler_ir::schedule::PointwiseBf16Node::Constant { bits } => {
                bytes.push(0x02);
                bytes.extend_from_slice(&bits.to_be_bytes());
            }
            tiler_ir::schedule::PointwiseBf16Node::Add { lhs, rhs } => {
                bytes.push(0x03);
                bytes.extend_from_slice(&lhs.index().to_be_bytes());
                bytes.extend_from_slice(&rhs.index().to_be_bytes());
            }
            tiler_ir::schedule::PointwiseBf16Node::Multiply { lhs, rhs } => {
                bytes.push(0x04);
                bytes.extend_from_slice(&lhs.index().to_be_bytes());
                bytes.extend_from_slice(&rhs.index().to_be_bytes());
            }
        }
    }
    bytes.extend_from_slice(&expression.root().index().to_be_bytes());
}

/// Appends one ordered binary expression node under its canonical tag.
fn encode_binary_node(
    bytes: &mut Vec<u8>,
    tag: u8,
    lhs: tiler_ir::schedule::PointwiseF32NodeId,
    rhs: tiler_ir::schedule::PointwiseF32NodeId,
) {
    bytes.push(tag);
    bytes.extend_from_slice(&lhs.index().to_be_bytes());
    bytes.extend_from_slice(&rhs.index().to_be_bytes());
}

/// The run entry naming a declared input this region's read list does not read.
///
/// It occupies the relation slot, and the slot's tag space is
/// [`encode_access_relation`]'s: that function writes `0x01`, `0x02`, `0x03`, or
/// the wildcard `0x00`, and nothing else, so `0x04` is a byte no run could carry
/// before this entry existed. That disjointness is the whole argument holding
/// `pointwise-f32.v4` and `serial-sum-f32.v3` where they are — a relation added
/// to that encoder must take the wildcard or a tag above this one, never this
/// one.
const UNREAD_DECLARED_INPUT_TAG: u8 = 0x04;

/// Encodes which declared inputs one whole-program, prologue, or fold region
/// reads, and how.
///
/// The count leads, then each entry gives an input ordinal and either its
/// relation or [`UNREAD_DECLARED_INPUT_TAG`]. **One read of an ordinal,
/// addressing densely, is written as nothing**: the ordinal's absence from the
/// run is that read's canonical spelling, so the empty run means "every declared
/// input is read once, densely".
///
/// **A declared input read by no leaf is written explicitly, and that entry is
/// what keeps the projection injective across this widening.** The recovery rule
/// is "an ordinal absent from the run has one dense read", and its premise was
/// the `elementwise-reads` completeness rule
/// `admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs` lifted.
/// Without the marker, three declared inputs and two dense reads would encode
/// alike whether the pair read is `{0, 1}`, `{0, 2}`, or `{1, 2}` — one arm, one
/// byte string, three programs — and leaning on the enclosing subject's graph
/// identity to separate them is exactly the unstated invariant an identity
/// encoder must not rest on.
///
/// **The step is therefore not forced, and this is why.** Writing every read
/// positionally would separate them too, but it moves the bytes of every subject
/// already encodable — an all-dense complete read list writes an empty run today
/// and would write one entry per input — so it costs both sub-tags a version and
/// every governed compilation its request qualifier. The marker moves nothing:
/// a program reading every declared input emits no marker at all, so its bytes
/// are what they were, and a byte string carrying one is a subject the earlier
/// vocabulary could not express. Per-tag injectivity closes, so
/// `pointwise-f32.v4` and `serial-sum-f32.v3` hold rather than step.
///
/// **The recovery, stated so a reader can refute it.** The declared input count
/// is written earlier in the same arm. For each ordinal in `0..declared`: it is
/// read not at all when the run carries its marker; it has one dense read when
/// the run does not name it; and it has exactly its `k` run entries, in run
/// order, otherwise. The two byte strings that would be ambiguous — a lone entry
/// writing `LinearIdentity`, and a marker beside any other entry for one ordinal
/// — are the two this projection never emits.
///
/// The relation is written through a per-variant tag and its own framed payload,
/// so two reads differing in operand shape, result shape, or any decode differ
/// in these bytes. The two structural relations get distinct tags for the reason
/// they are distinct variants: a bijection and a replication are different facts
/// about what a read consumes.
///
/// `declared` is a count rather than an ordinal list because the arm writes the
/// declared input keys immediately before, in the same order. It saturates at
/// [`u32::MAX`] rather than truncating, which no request reaches:
/// [`check_program_budgets`] bounds a program's declared inputs far below it, so
/// the saturation is unreachable rather than a collision this encoder tolerates.
fn encode_elementwise_reads(output: &mut Vec<u8>, declared: usize, reads: &[(u32, LogicalAccess)]) {
    let written = || {
        reads
            .iter()
            .enumerate()
            .filter(|(position, (ordinal, map))| {
                *map != LogicalAccess::LinearIdentity
                    || reads
                        .iter()
                        .enumerate()
                        .any(|(other, (seen, _))| other != *position && seen == ordinal)
            })
    };
    let declared = u32::try_from(declared).unwrap_or(u32::MAX);
    let unread = || (0..declared).filter(|ordinal| !reads.iter().any(|(seen, _)| seen == ordinal));
    push_len(output, written().count() + unread().count());
    for (_, (ordinal, map)) in written() {
        output.extend_from_slice(&ordinal.to_be_bytes());
        encode_access_relation(output, map);
    }
    for ordinal in unread() {
        output.extend_from_slice(&ordinal.to_be_bytes());
        output.push(UNREAD_DECLARED_INPUT_TAG);
    }
}

/// Appends one read's access relation under its canonical per-variant tag.
///
/// Split out of [`encode_elementwise_reads`] because an epilogue's read list
/// writes every position unconditionally, so it needs the relation without that
/// run's canonical omission. One definition is what keeps the two spellings from
/// drifting into two tag vocabularies.
///
/// [`LogicalAccess::LinearIdentity`] carries its own tag rather than falling
/// through the wildcard, which is what keeps the dense read distinguishable from
/// a relation this encoder refuses. Both callers reach it: an epilogue read that
/// interposes no relation, and the dense half of a declared input read twice.
///
/// **The tag space stops at `0x03`, and that is load-bearing elsewhere.**
/// [`UNREAD_DECLARED_INPUT_TAG`] occupies this slot's `0x04` in
/// [`encode_elementwise_reads`]'s run precisely because no relation can write it
/// here; a relation added later takes the wildcard or a tag above `0x04`, never
/// `0x04` itself, or two arms of that run become one byte string.
fn encode_access_relation(output: &mut Vec<u8>, map: &LogicalAccess) {
    match map {
        LogicalAccess::ReindexBijection {
            operand_shape,
            result_shape,
            axes,
        } => {
            output.push(0x01);
            encode_explain_shape(output, operand_shape);
            encode_explain_shape(output, result_shape);
            encode_explain_axis_decodes(output, axes);
        }
        LogicalAccess::BroadcastReplication {
            operand_shape,
            result_shape,
            axes,
        } => {
            output.push(0x02);
            encode_explain_shape(output, operand_shape);
            encode_explain_shape(output, result_shape);
            encode_explain_axis_decodes(output, axes);
        }
        LogicalAccess::LinearIdentity => output.push(0x03),
        // No other relation can be recorded here: `recognize_structural_read`
        // is the only producer of a mapped read and it builds exactly the two
        // above. The arm is a refusal to encode rather than a wildcard tag, so a
        // relation added later cannot silently share one of these tags.
        _ => output.push(0x00),
    }
}

/// Encodes one framed run of operand-axis coordinate decodes.
fn encode_explain_axis_decodes(output: &mut Vec<u8>, axes: &[AxisDecode]) {
    push_len(output, axes.len());
    for decode in axes {
        output.extend_from_slice(&decode.divisor.to_be_bytes());
        output.extend_from_slice(&decode.modulus.to_be_bytes());
        output.push(u8::from(decode.mirrored));
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
    /// The recognized elementwise prologue the fold's contributors come from, or
    /// `None` when the fold's operand is a declared input tensor.
    pub(crate) const fn prologue(&self) -> Option<&PointwiseF32Expression> {
        self.prologue.as_ref()
    }
    /// The prologue region's reads, in access order; empty when there is no
    /// prologue.
    pub(crate) fn prologue_reads(&self) -> &[(u32, LogicalAccess)] {
        &self.prologue_reads
    }
    /// The declared input ordinal a prologue-less fold reads directly, or
    /// `None` when a prologue region materializes its contributors.
    pub(crate) const fn contributor_input(&self) -> Option<u32> {
        self.contributor_input
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
    pub(crate) fn target_slots(&self) -> &[VerifiedTargetSlot] {
        &self.target_slots
    }

    /// Returns the verified target indexes used by receipt-mutation fixtures.
    #[cfg(test)]
    pub(crate) fn target_profiles(&self) -> Vec<usize> {
        (0..self.target_slots.len()).collect()
    }

    /// Returns the verified deterministic budgets bound to this request.
    pub(crate) const fn budgets(&self) -> DeterministicBudgets {
        self.budgets
    }

    /// Re-admits one semantic candidate for an already verified target group.
    ///
    /// The outer request has already admitted the nonempty unique target set,
    /// contract vocabulary, capability pairing, and request schema. Candidate
    /// readmission therefore rechecks only the candidate program and remints
    /// target-local authorities for the named resolved slots. Repeating the
    /// outer admission here would let an unrelated target rejection erase this
    /// contract group.
    pub(crate) fn readmit_candidate(
        &self,
        program: &SemanticProgram,
        target_indexes: &[usize],
    ) -> Result<Self, RequestError> {
        // Authorities before recognition, for the reason [`verify_request`]
        // states at the same pair of statements.
        let Ok(realization_laws) = FrozenIndexRealizationLawRegistry::from_semantic(
            program.semantic_registry().clone(),
            self.capabilities.scalars.clone(),
        ) else {
            return unsupported("capability", "semantic-authority-pairing");
        };
        if self.capabilities.lowering.semantic_snapshot()
            != program.semantic_registry().snapshot_identity()
        {
            return unsupported("capability", "semantic-authority-pairing");
        }
        let (normalized, semantic_identity) =
            verify_program(program, self.budgets, &realization_laws)?;
        let mut target_slots = Vec::with_capacity(target_indexes.len());
        for target_index in target_indexes {
            let slot = self
                .target_slots
                .get(*target_index)
                .ok_or(RequestError::UnverifiedTargetSelection)?;
            let VerifiedTargetResolution::Resolved {
                numerical_contract, ..
            } = &slot.resolution
            else {
                return Err(RequestError::UnverifiedTargetSelection);
            };
            // Rechecked rather than inherited from the resolved slot. The
            // obligation is a property of the candidate's operation multiset,
            // and a rewrite that introduced a family this target cannot realize
            // would otherwise inherit an admission granted to a program that did
            // not contain it. Today's algebraic rules preserve the multiset, so
            // this cannot fire — which is exactly why it is a check rather than
            // a comment, and why its failure is invalid compiler output rather
            // than a candidate silently dropped.
            require_elementary_accuracy(program, &slot.target_profile)?;
            let authority = request_subject(
                &normalized,
                &semantic_identity,
                &self.numerical_contracts,
                *numerical_contract,
                self.budgets,
                &slot.target_profile,
                VerifiedRequestAuthorities {
                    installed: &self.capabilities,
                    realization_laws: &realization_laws,
                },
            );
            target_slots.push(VerifiedTargetSlot {
                target_profile: slot.target_profile.clone(),
                resolution: VerifiedTargetResolution::Resolved {
                    numerical_contract: *numerical_contract,
                    authority: Box::new(authority),
                },
            });
        }
        Ok(Self {
            normalized,
            semantic_identity,
            numerical_contracts: self.numerical_contracts.clone(),
            budgets: self.budgets,
            target_slots,
            capabilities: self.capabilities.clone(),
            realization_laws,
        })
    }

    pub(crate) fn for_target(
        &self,
        target_index: usize,
    ) -> Result<VerifiedTargetRequest, RequestError> {
        let Some(slot) = self.target_slots.get(target_index) else {
            return Err(RequestError::UnverifiedTargetSelection);
        };
        let (numerical_contract, authority) = match &slot.resolution {
            VerifiedTargetResolution::Resolved {
                numerical_contract,
                authority,
            } => (numerical_contract, authority),
            VerifiedTargetResolution::Rejected(error) => return Err(error.clone()),
        };
        let current_authority = request_subject(
            &self.normalized,
            &self.semantic_identity,
            &self.numerical_contracts,
            *numerical_contract,
            self.budgets,
            &slot.target_profile,
            VerifiedRequestAuthorities {
                installed: &self.capabilities,
                realization_laws: &self.realization_laws,
            },
        );
        if !numerical_contract.is_governed() || authority.as_ref() != &current_authority {
            return Err(RequestError::UnverifiedTargetSelection);
        }
        Ok(VerifiedTargetRequest {
            normalized: self.normalized.clone(),
            semantic_identity: self.semantic_identity.clone(),
            numerical_contracts: self.numerical_contracts.clone(),
            numerical_contract: *numerical_contract,
            budgets: self.budgets,
            target_profile: slot.target_profile.clone(),
            capabilities: self.capabilities.clone(),
            realization_laws: self.realization_laws.clone(),
            authority: current_authority,
        })
    }
}

impl VerifiedTargetSlot {
    pub(crate) const fn target_profile(&self) -> &TargetProfile {
        &self.target_profile
    }

    pub(crate) const fn resolution(&self) -> &VerifiedTargetResolution {
        &self.resolution
    }
}

#[derive(Clone, Copy)]
struct VerifiedRequestAuthorities<'a> {
    installed: &'a CompilerCapabilitySnapshot,
    realization_laws: &'a FrozenIndexRealizationLawRegistry,
}

fn request_subject(
    normalized: &NormalizedProgram,
    semantic_identity: &SemanticIdentity,
    numerical_contracts: &NumericalContractPreference,
    numerical_contract: StrictF32NumericalContract,
    budgets: DeterministicBudgets,
    target_profile: &TargetProfile,
    authorities: VerifiedRequestAuthorities<'_>,
) -> VerifiedRequestSubject {
    #[cfg(test)]
    crate::workcount::REQUEST_SUBJECT_REBUILDS.record();
    let normalized = NormalizedProgramSubject {
        outputs: normalized.outputs().iter().map(output_subject).collect(),
    };
    VerifiedRequestSubject {
        normalized,
        semantic_identity: semantic_identity.clone(),
        numerical_contracts: numerical_contracts.clone(),
        numerical_contract,
        budgets,
        target_profile: target_profile.clone(),
        capability_schema_version: authorities.installed.schema_version,
        lowering_registry: authorities.installed.registry_identity().clone(),
        realization_registry: authorities
            .realization_laws
            .identity()
            .as_bytes()
            .to_vec()
            .into_boxed_slice(),
    }
}

/// Projects one recognized output into the subject the request is bound to.
///
/// Recursive because an epilogue chain carries a whole further recognized output
/// inside it; every other arm is a flat projection.
fn output_subject(normalized: &NormalizedOutput) -> NormalizedOutputSubject {
    match normalized {
        NormalizedOutput::SerialSum(normalized) => {
            NormalizedOutputSubject::SerialSum(NormalizedSerialSumSubject {
                input_keys: normalized.input_keys.clone(),
                output_key: normalized.output_key.clone(),
                input_shape: normalized.input_shape.clone(),
                output_shape: normalized.output_shape.clone(),
                reduction_axes: normalized.reduction_axes.clone(),
                prologue: normalized.prologue.clone(),
                prologue_reads: normalized.prologue_reads.clone(),
                contributor_input: normalized.contributor_input,
                members: normalized.members.clone(),
                input_elements: normalized.input_elements,
                output_elements: normalized.output_elements,
            })
        }
        NormalizedOutput::Pointwise(normalized) => {
            NormalizedOutputSubject::Pointwise(normalized.clone())
        }
        NormalizedOutput::Contraction(normalized) => {
            NormalizedOutputSubject::Contraction(normalized.clone())
        }
        NormalizedOutput::Staged(normalized) => {
            let mut occurrence = normalized.clone();
            // Cleared here rather than left duplicated: the producer travels in
            // the subject's own recursive slot beside it, and two copies of one
            // recognized shape are two accounts of one fact.
            let producer = occurrence
                .producer
                .take()
                .map(|producer| Box::new(output_subject(&producer)));
            NormalizedOutputSubject::Staged(Box::new(NormalizedStagedSubject {
                occurrence,
                producer,
            }))
        }
        NormalizedOutput::Epilogue(chain) => {
            NormalizedOutputSubject::Epilogue(Box::new(NormalizedEpilogueSubject {
                producer: Box::new(output_subject(&chain.producer)),
                input_keys: chain.input_keys.clone(),
                output_key: chain.output_key.clone(),
                shape: chain.shape.clone(),
                expression: chain.expression.clone(),
                reads: chain.reads.clone(),
                members: chain.members.clone(),
                elements: chain.elements,
            }))
        }
    }
}

/// Why one stated numerical contract could not be resolved on one target.
///
/// The three arms are three different claims and are deliberately not collapsed:
/// a declared refusal, an absent declaration, and a declaration that has not yet
/// become available are not the same thing, and reporting the second or third as
/// a rejection would assert knowledge the profile never supplied.
///
/// A stated contract about *another* arithmetic type produces no arm here at
/// all, and deliberately: it was never asked of the target, because a contract's
/// arithmetic is part of its identity and a target's rows are keyed by subject,
/// so there is no declaration of this profile's that could answer for it.
/// [`RequestError::NoApplicableNumericalContract`] is that refusal, and it is
/// program-scoped rather than target-local for the same reason.
#[derive(Clone, Debug, Eq, PartialEq)]
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
    pub(crate) const fn contract_key(&self) -> &'static str {
        match self {
            Self::Unhonourable { contract_key, .. }
            | Self::Undeclared { contract_key, .. }
            | Self::Deferred { contract_key, .. } => contract_key,
        }
    }

    /// The dimension the resolution failed on.
    pub(crate) fn dimension(&self) -> NumericalDimension {
        match self {
            Self::Unhonourable { cause, .. } => cause.dimension(),
            Self::Undeclared { cause, .. } => cause.dimension(),
            Self::Deferred { cause, .. } => cause.dimension(),
        }
    }

    /// The arithmetic type the resolution failed for.
    ///
    /// Reported beside the dimension because one profile can honour a dimension
    /// in one arithmetic type and refuse it in another — the measured Apple row
    /// preserves subnormals in `f16` and flushes them in `f32` — so a rejection
    /// naming only the dimension would be false about the other type.
    pub(crate) fn arithmetic(&self) -> ArithmeticType {
        match self {
            Self::Unhonourable { cause, .. } => cause.arithmetic(),
            Self::Undeclared { cause, .. } => cause.arithmetic(),
            Self::Deferred { cause, .. } => cause.arithmetic(),
        }
    }

    /// The complete resolved semantic type the resolution failed for.
    pub(crate) fn resolved_type(&self) -> &ResolvedValueType {
        match self {
            Self::Unhonourable { cause, .. } => cause.resolved_type(),
            Self::Undeclared { cause, .. } => cause.resolved_type(),
            Self::Deferred { cause, .. } => cause.resolved_type(),
        }
    }

    /// The behaviour the contract required on that dimension.
    pub(crate) const fn required(&self) -> DimensionBehaviour {
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
            "{}: {} in {} requires {}",
            self.contract_key(),
            self.dimension().key(),
            self.arithmetic().canonical_type_key(),
            self.required().key()
        )?;
        match self {
            Self::Unhonourable { cause, .. } => {
                write!(formatter, ", target declares {}", cause.means().label())?;
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

/// Why one exact program dtype cannot be dispatched at compile profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DTypeDispatchRefusalDisposition {
    /// The profile explicitly refuses the exact type.
    Unsupported,
    /// The first exact fact becomes available only at a later phase.
    Deferred { available_at: AvailabilityPhase },
    /// No fact names the exact type at any phase.
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RequestError {
    UnsupportedRequestVersion,
    /// The request carried a shape environment that is not the program's own.
    ///
    /// Two environments over one program is the ambiguity
    /// [`tiler_ir::index::IndexRegionBuilder::new_with_shape_environment`]
    /// exists to prevent. Dropping the program's environment, or attaching a
    /// different one, is this refusal rather than
    /// [`Self::UnsupportedRequestVersion`]: the schema is current and the
    /// authority that is wrong is the environment pairing.
    MismatchedShapeEnvironment,
    EmptyTargetSet,
    DuplicateTargetProfile,
    UnverifiedTargetSelection,
    /// The caller stated no numerical contract at all.
    ///
    /// Distinct from every rejection that names a dimension: there is no default
    /// and no implicit strictest reading, so the diagnostic says the contract is
    /// unstated rather than reporting a dimension the caller never chose.
    UnstatedNumericalContract,
    /// The same exact numerical contract appeared more than once.
    DuplicateNumericalContract,
    /// The preference exceeded the number of distinct public contracts.
    TooManyNumericalContracts {
        actual: usize,
        max: usize,
    },
    /// No contract in the caller's stated order is about this program's
    /// arithmetic type.
    ///
    /// **Program-scoped, and checked before any target is consulted**, for the
    /// reason [`Self::UnrepresentableNumericalDimension`] is: it is a property of
    /// the request rather than of a profile. A contract's arithmetic is part of
    /// its identity (ADR 0076 item 6) and a target's honourability rows are keyed
    /// by subject, so a `bf16` program stated under an `f32` contract is not a
    /// question any profile can answer — the `f32` rows would answer honestly
    /// about a width the program does not use, and the program would compile
    /// under a meaning nobody stated for it.
    ///
    /// Distinct from [`Self::NoResolvableNumericalContract`], which reports that
    /// the target *was* asked and declined. Nothing here proposes a substitute:
    /// only the caller may state what its program means.
    NoApplicableNumericalContract {
        /// The arithmetic every value of the submitted program carries.
        program: ArithmeticType,
        /// Each stated contract's key and the arithmetic it resolves, in the
        /// caller's own order.
        stated: Vec<(&'static str, ArithmeticType)>,
    },
    /// No contract in the caller's stated order resolves on this target.
    ///
    /// Every stated entry's first canonical failure is retained, in the caller's
    /// order, so the diagnostic explains the whole preference rather than only
    /// its last entry. Nothing here proposes a substitute contract: only the
    /// caller may change what its program means.
    NoResolvableNumericalContract {
        target_profile: TargetProfileKey,
        rejections: Vec<ContractRejection>,
    },
    /// One exact program value type cannot be dispatched on this target at the
    /// compile-profile phase.
    DTypeNotDispatchable {
        target_profile: TargetProfileKey,
        resolved_type: Box<ResolvedValueType>,
        disposition: DTypeDispatchRefusalDisposition,
    },
    /// A stated contract resolves a dimension this build cannot realize.
    ///
    /// Distinct from every target rejection above, and deliberately so: those say
    /// *this target* cannot do what the caller asked, and this says *no target
    /// could*, because the scheduled-region IR has nowhere to record which
    /// resolution was chosen and two contracts differing only there would reach
    /// one region. Reporting it as an unhonourable dimension would attribute a
    /// build limitation to a profile that never claimed anything about it.
    UnrepresentableNumericalDimension {
        cause: UnrepresentableDimension,
    },
    /// A deterministic budget refused a demand.
    ///
    /// The sole carrier of every budget refusal the compiler raises, from all
    /// four authorities. `limit` and `actual` are `u64` because the internal
    /// stop records are: the two search budgets `DeterministicBudgets` declares
    /// as `u64` cannot be narrowed to `u32` without reporting a saturated
    /// number as though it were the declared bound, and a `usize` demand would
    /// make the width of a public refusal a property of the host.
    ///
    /// Whether `actual` is the exact demand or a lower bound is not uniform
    /// across the vocabulary and is read from [`BudgetResource::refusal`].
    BudgetExceeded {
        resource: BudgetResource,
        limit: u64,
        actual: u64,
    },
    UnsupportedCapability {
        phase: &'static str,
        rule: &'static str,
    },
    /// A strategy or later capability stated over fixed extents met a symbolic one.
    ///
    /// Named by the extent as written, never by a specialized value. A bound
    /// symbol is still this refusal: specializing it into the logical plan is a
    /// physical-planning decision this boundary must not make. Distinct from
    /// [`Self::UnsupportedCapability`]: that variant names a handle, signature,
    /// or other rule that happened to observe the shape, which is the
    /// mis-attribution this refusal exists to close.
    UnsupportedSymbolicExtent {
        phase: &'static str,
        /// The capability that cannot plan over the extent.
        rule: &'static str,
        /// The extent as written. Never a value the environment determines.
        extent: SourcedExtent,
    },
    /// The target declares no realization refining a registered elementary
    /// accuracy contract this program's operations carry.
    ///
    /// Distinct from every dimension rejection above. Those resolve a *generic*
    /// numerical freedom the caller stated; this one is an ADR 0042 accuracy
    /// contract the registered operation itself carries, which no contract a
    /// caller can state weakens or waives. It is a target-local hard rejection,
    /// so a companion profile that does declare a refining realization still
    /// compiles.
    UnrealizedElementaryAccuracy {
        /// The elementary family whose registered contract went unsatisfied.
        operation: OpKey,
        /// The profile that was asked.
        target_profile: TargetProfileKey,
        /// Stable diagnostic code of the refusing reason.
        ///
        /// Carried rather than re-derived so the public failure key and the
        /// refusal that produced it cannot disagree; the three reasons — no
        /// installed realization at all, an installed one that could not be
        /// proved to refine, and a refining one whose evidence cannot discharge —
        /// are different findings and keep different keys.
        reason: &'static str,
        /// The failing half, when `reason` is undischarged evidence.
        undischarged_half: Option<crate::target::accuracy::ElementaryEvidenceHalf>,
        /// The failing evidence class, when `reason` is undischarged evidence.
        undischarged_class: Option<tiler_ir::semantic::accuracy::ConformanceEvidenceClass>,
        /// Declared same-operation candidates in canonical order.
        ///
        /// Empty when nothing was installed. Several unrefined or undischarged
        /// rows appear here in the same order the profile stores them after
        /// canonicalization, so the public refusal cannot depend on insertion
        /// order.
        candidates: Box<[crate::target::accuracy::ElementaryAccuracyCandidate]>,
    },
    ShapeProductOverflow {
        role: &'static str,
    },
}

impl fmt::Display for RequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedRequestVersion => {
                formatter.write_str("compile.request.schema: unsupported request schema")
            }
            Self::MismatchedShapeEnvironment => formatter.write_str(
                "compile.request.shape-environment: request must carry the program's own environment",
            ),
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
            Self::DuplicateNumericalContract => formatter.write_str(
                "compile.request.numerics.duplicate: numerical contracts must be distinct",
            ),
            Self::TooManyNumericalContracts { actual, max } => write!(
                formatter,
                "compile.request.numerics.too-many: {actual} contracts exceeds maximum {max}"
            ),
            Self::NoApplicableNumericalContract { program, stated } => {
                write!(
                    formatter,
                    "compile.request.numerics.inapplicable: no stated contract resolves {}",
                    program.canonical_type_key()
                )?;
                for (key, arithmetic) in stated {
                    write!(
                        formatter,
                        "; {key} resolves {}",
                        arithmetic.canonical_type_key()
                    )?;
                }
                Ok(())
            }
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
            Self::DTypeNotDispatchable {
                target_profile,
                resolved_type,
                disposition,
            } => write!(
                formatter,
                "compile.request.dtype.dispatch: target {target_profile} cannot dispatch exact type {:?} at compile profile: {disposition:?}",
                resolved_type.canonical_encoding().as_bytes(),
            ),
            Self::UnrepresentableNumericalDimension { cause } => write!(
                formatter,
                "compile.request.numerics.unrepresentable: {} in {} requires {}, this build realizes only {} and {} can consume it",
                cause.dimension().key(),
                cause.arithmetic().canonical_type_key(),
                cause.required().key(),
                cause.realized().key(),
                cause.consumed_by(),
            ),
            Self::BudgetExceeded {
                resource,
                limit,
                actual,
            } => write!(
                formatter,
                "compile.budget.{}: {actual} exceeds deterministic limit {limit}",
                resource.key()
            ),
            Self::UnsupportedCapability { phase, rule } => {
                write!(
                    formatter,
                    "compile.unsupported.{phase}.{rule}: no installed capability can compile this valid semantic program"
                )
            }
            Self::UnsupportedSymbolicExtent {
                phase,
                rule,
                extent,
            } => write!(
                formatter,
                "compile.{phase}.{rule}: {extent} is a symbolic extent this capability cannot plan over"
            ),
            Self::UnrealizedElementaryAccuracy {
                operation,
                target_profile,
                reason,
                undischarged_half,
                undischarged_class,
                candidates: _,
            } => {
                write!(
                    formatter,
                    "{reason}: target {target_profile} declares no realization that both refines and discharges the registered accuracy contract of {operation}"
                )?;
                match (undischarged_half, undischarged_class) {
                    (Some(half), Some(class)) => write!(
                        formatter,
                        "; {half} evidence class {class} cannot discharge a hard requirement"
                    ),
                    _ => Ok(()),
                }
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
) -> Result<VerifiedRequest, RequestError> {
    if !carries_program_environment(request.shape_environment, request.program) {
        return Err(RequestError::MismatchedShapeEnvironment);
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
    // Representability is checked before admission and before any target is
    // consulted. Before a target, because it is a property of this build rather
    // than of a profile: a dimension an admitted operation can consume and no
    // scheduled region can record would give two meanings one identity on every
    // target at once, and reporting that as an unhonourable dimension would
    // attribute a build limitation to a declaration that never spoke about it.
    // Before admission, because it is the more specific of two true statements —
    // an unrealizable contract is also unregistered, and "this build cannot
    // realize a permitted signed-zero dimension" names the reason while "this
    // contract is not one this build registers" only names the consequence.
    for contract in request.numerical_contracts.stated() {
        if let Some(cause) = crate::policy::unrepresentable_dimension(contract) {
            return Err(RequestError::UnrepresentableNumericalDimension { cause });
        }
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
    let mut target_keys: Vec<_> = request
        .target_profiles
        .iter()
        .map(TargetProfile::profile_key)
        .collect();
    target_keys.sort_unstable();
    if target_keys.windows(2).any(|keys| keys[0] == keys[1]) {
        return Err(RequestError::DuplicateTargetProfile);
    }
    // Budgets before targets, because exceeding one is a property of the
    // submitted program that no target outcome can make admissible. Recognition
    // is deliberately *not* here — see the phase comment below.
    check_program_budgets(request.program, request.budgets)?;
    let dispatch_types = canonical_program_value_types(request.program);
    // The arithmetic every stated contract is measured against, and `None` for a
    // program whose value types this build states no contract vocabulary for.
    //
    // **`ok()` rather than `?`, and the discarded refusal is deliberate.** The
    // recognizer reports the same finding under `dtype-recognized` in its own
    // phase, *after* every target has answered, and hoisting it here would move
    // a build limitation ahead of the target's own dtype-dispatch and
    // honourability refusals — the exact ordering the phase split below exists to
    // prevent. What is hoisted is only the *applicability* narrowing, which needs
    // an arithmetic to compare against and simply does not apply without one.
    let program_arithmetic = recognized_program_arithmetic(request.program).ok();
    // Applicability before targets, for the reason representability is checked
    // before them: a contract stated for another width is not a question this
    // profile — or any profile — can answer, because a contract's arithmetic is
    // part of its identity and a target's honourability rows are keyed by
    // subject. Only the *complete* absence of an applicable entry refuses here;
    // a preference naming this program's width alongside another's resolves
    // against the applicable entries and reports their own causes.
    if let Some(program) = program_arithmetic
        && !request
            .numerical_contracts
            .stated()
            .iter()
            .any(|contract| contract.arithmetic == program)
    {
        return Err(RequestError::NoApplicableNumericalContract {
            program,
            stated: request
                .numerical_contracts
                .stated()
                .iter()
                .map(|contract| (contract.key, contract.arithmetic))
                .collect(),
        });
    }

    // Resolve every structurally admitted target independently. A profile that
    // honours no stated contract is a target-local outcome, not a reason to
    // discard the other ordered slots. Intrinsic profile/authority failures
    // remain outer request errors because no target outcome can make malformed
    // input valid.
    //
    // **This runs before the program is recognized, and the order is the whole
    // point of the phase split.** Honourability is a property of the stated
    // contract and the target's own declaration; it does not depend on which
    // physical strategy this build happens to be able to spell. Recognition
    // answers a different question — what this build can *plan* — so asking it
    // first attributes a build limitation to a request whose stated meaning the
    // target already cannot deliver. That was not hypothetical: while the
    // recognizer refused every non-`f32` program under one `dtype-f32` rule, a
    // profile's measured `bf16` subnormal row could never produce the refusal it
    // exists to produce, and the missing answer read as a missing target fact
    // rather than as a boundary in the wrong order. The rule is gone and the
    // order is what keeps its lesson: a width this build cannot *spell* is still
    // reported after the target has answered for the width it cannot *dispatch*.
    //
    // Each of the three checks below keeps its former relative order, so nothing
    // about which refusal a rejected target reports has moved.
    let target_resolutions = request
        .target_profiles
        .iter()
        .map(|target| {
            let structural = require_compile_profile_dispatch(target, &dispatch_types)
                .and_then(|()| require_elementary_accuracy(request.program, target));
            match structural {
                Ok(()) => match resolve_numerical_contract(
                    &request.numerical_contracts,
                    target,
                    program_arithmetic,
                ) {
                    Ok(numerical_contract) => Ok(Ok(numerical_contract)),
                    Err(error @ RequestError::NoResolvableNumericalContract { .. }) => {
                        Ok(Err(error))
                    }
                    Err(error) => Err(error),
                },
                // Both structural refusals are target-local: another requested
                // profile may dispatch the dtype, or declare the elementary
                // realization, that this one does not.
                Err(
                    error @ (RequestError::DTypeNotDispatchable { .. }
                    | RequestError::UnrealizedElementaryAccuracy { .. }),
                ) => Ok(Err(error)),
                Err(error) => Err(error),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Nothing left to plan: no requested target admitted the request, so there
    // is no program for a strategy to be chosen for. Returning the ordered
    // refusals here is what keeps the more specific statement — *this target
    // declares it cannot honour this dimension* — from being replaced by the
    // recognizer's, which would be true and would answer a question the caller
    // cannot act on while every target still refuses.
    if target_resolutions.iter().all(Result::is_err) {
        return Ok(VerifiedRequest::Refused(
            request
                .target_profiles
                .iter()
                .zip(target_resolutions)
                .map(|(target_profile, resolution)| VerifiedTargetSlot {
                    target_profile: target_profile.clone(),
                    resolution: VerifiedTargetResolution::Rejected(
                        resolution.expect_err("every resolution is an error in this branch"),
                    ),
                })
                .collect(),
        ));
    }

    // The authorities the recognized program's subject is bound to, then
    // recognition.
    //
    // **The realization-law authority now precedes recognition, and it has to.**
    // Recognition asks it whether an occurrence's registered law realizes a
    // region *sequence*, which is what admits a staged family as a program
    // stage, so an authority that does not cohere is not one recognition may
    // consult. The order change is confined to a program that fails both: it
    // used to report the recognizer's rule and now reports the pairing, which is
    // the more specific of the two statements — recognition's answer under an
    // incoherent authority would not be evidence about the program at all. The
    // semantic-snapshot pairing keeps its own position between them, and both
    // report the same rule, so a program failing only one is unmoved.
    let Ok(realization_laws) = FrozenIndexRealizationLawRegistry::from_semantic(
        request.program.semantic_registry().clone(),
        request.capabilities.scalars.clone(),
    ) else {
        return unsupported("capability", "semantic-authority-pairing");
    };
    if request.capabilities.lowering.semantic_snapshot()
        != request.program.semantic_registry().snapshot_identity()
    {
        return unsupported("capability", "semantic-authority-pairing");
    }
    let normalized = select_supported_strategy(request.program, &realization_laws)?;
    let semantic_identity = request.program.semantic_identity().clone();
    let target_slots = request
        .target_profiles
        .iter()
        .zip(target_resolutions)
        .map(|(target, resolution)| VerifiedTargetSlot {
            target_profile: target.clone(),
            resolution: match resolution {
                Ok(numerical_contract) => VerifiedTargetResolution::Resolved {
                    numerical_contract,
                    authority: Box::new(request_subject(
                        &normalized,
                        &semantic_identity,
                        &request.numerical_contracts,
                        numerical_contract,
                        request.budgets,
                        target,
                        VerifiedRequestAuthorities {
                            installed: &request.capabilities,
                            realization_laws: &realization_laws,
                        },
                    )),
                },
                Err(error) => VerifiedTargetResolution::Rejected(error),
            },
        })
        .collect();
    Ok(VerifiedRequest::Planned(Box::new(
        VerifiedCompilationRequest {
            normalized,
            semantic_identity,
            numerical_contracts: request.numerical_contracts,
            budgets: request.budgets,
            target_slots,
            capabilities: request.capabilities,
            realization_laws,
        },
    )))
}

/// The outcome of admitting one compilation request.
///
/// Two variants because a request can be completely refused *before* a strategy
/// is chosen, and the alternative shapes are both worse: an optional recognized
/// program inside [`VerifiedCompilationRequest`] would make every later stage
/// carry a case that cannot occur once a target resolved, and forcing
/// recognition to run anyway would report a recognizer limitation for a request
/// no target admitted.
pub(crate) enum VerifiedRequest {
    /// At least one target admitted the request, so the program was recognized.
    Planned(Box<VerifiedCompilationRequest>),
    /// Every requested target refused, in the caller's order.
    Refused(Vec<VerifiedTargetSlot>),
}

/// Admits a request whose fixture profile is expected to resolve.
///
/// The crate's own fixtures state a contract their governed profile honours, so
/// the planned outcome is the one under test at every one of these call sites
/// and an unexpected complete refusal should fail loudly rather than be pattern
/// matched away. Tests that assert a *refused* request call [`verify_request`]
/// directly and match the variant.
#[cfg(test)]
pub(crate) fn verify_planned_request(
    request: CompilationRequest<'_>,
) -> Result<VerifiedCompilationRequest, RequestError> {
    verify_request(request).map(|verified| {
        verified
            .planned()
            .expect("the fixture profile admits the stated contract")
    })
}

impl VerifiedRequest {
    /// The ordered target slots, whichever way the request was admitted.
    pub(crate) fn target_slots(&self) -> &[VerifiedTargetSlot] {
        match self {
            Self::Planned(request) => request.target_slots(),
            Self::Refused(slots) => slots,
        }
    }

    /// The planned request, or `None` when every target refused.
    pub(crate) fn planned(self) -> Option<VerifiedCompilationRequest> {
        match self {
            Self::Planned(request) => Some(*request),
            Self::Refused(_) => None,
        }
    }
}

/// Returns every exact value type in canonical byte order, without duplicates.
fn canonical_program_value_types(program: &SemanticProgram) -> Vec<ResolvedValueType> {
    let mut resolved_types = program
        .values()
        .map(|value| value.resolved_type().clone())
        .collect::<Vec<_>>();
    resolved_types.sort_by(|left, right| {
        left.canonical_encoding()
            .as_bytes()
            .cmp(right.canonical_encoding().as_bytes())
    });
    resolved_types.dedup();
    resolved_types
}

/// Requires the target to realize every registered elementary accuracy contract
/// this program's operations carry.
///
/// **Asked of the whole program rather than of the recognized members, and the
/// two are the same set here.** Recognition already requires the members it
/// matched to cover the program exactly — that is what `operation-set` refuses —
/// so walking the program's operations reaches every recognized occurrence and
/// nothing else, without threading the recognizer's member vector through a
/// question that is about operations rather than about regions.
///
/// **Asked per target, before any numerical contract is resolved.** The
/// obligation is the registered operation's and is fixed; no contract a caller
/// can state widens or waives it, so resolving one first would order two
/// independent rejections without making either more specific.
fn require_elementary_accuracy(
    program: &SemanticProgram,
    target: &TargetProfile,
) -> Result<(), RequestError> {
    let operations: Vec<OpKey> = program
        .operations()
        .map(|operation| operation.key().clone())
        .collect();
    crate::target::accuracy::assess_program_elementary_accuracy(operations.iter(), target).map_err(
        |refusal| RequestError::UnrealizedElementaryAccuracy {
            operation: refusal.operation().clone(),
            target_profile: target.profile_key().clone(),
            reason: refusal.diagnostic_code(),
            undischarged_half: refusal.undischarged_half(),
            undischarged_class: refusal.undischarged_class(),
            candidates: refusal.candidates().to_vec().into_boxed_slice(),
        },
    )
}

/// Requires an exact compile-profile dispatch fact for every program value type.
fn require_compile_profile_dispatch(
    target: &TargetProfile,
    resolved_types: &[ResolvedValueType],
) -> Result<(), RequestError> {
    for resolved_type in resolved_types {
        let disposition =
            match target.dtype_dispatchability(resolved_type, AvailabilityPhase::CompileProfile) {
                DTypeDispatchabilityResolution::Dispatchable => continue,
                DTypeDispatchabilityResolution::Unsupported => {
                    DTypeDispatchRefusalDisposition::Unsupported
                }
                DTypeDispatchabilityResolution::Deferred { available_at } => {
                    DTypeDispatchRefusalDisposition::Deferred { available_at }
                }
                DTypeDispatchabilityResolution::Unknown => DTypeDispatchRefusalDisposition::Unknown,
            };
        return Err(RequestError::DTypeNotDispatchable {
            target_profile: target.profile_key().clone(),
            resolved_type: Box::new(resolved_type.clone()),
            disposition,
        });
    }
    Ok(())
}

/// Verifies the program-scoped portion shared by outer admission and semantic
/// candidate readmission.
fn verify_program(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    laws: &FrozenIndexRealizationLawRegistry,
) -> Result<(NormalizedProgram, SemanticIdentity), RequestError> {
    check_program_budgets(program, budgets)?;
    Ok((
        select_supported_strategy(program, laws)?,
        program.semantic_identity().clone(),
    ))
}

/// Checks every deterministic budget this request's program must fit.
///
/// Separated from recognition so outer admission can run it in its own phase:
/// exceeding a budget is a property of the submitted program that no target
/// outcome can excuse, while recognition is a statement about what this build
/// can plan and is asked only of a request some target admitted. Readmission
/// keeps both together, because a rewritten candidate has to clear each again.
fn check_program_budgets(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
) -> Result<(), RequestError> {
    check_budget(
        BudgetResource::SemanticValues,
        budgets.semantic_values,
        program.value_count(),
    )?;
    check_budget(
        BudgetResource::SemanticOperations,
        budgets.semantic_operations,
        program.operation_count(),
    )?;
    // The largest shape this profile may assemble, not the smallest it might:
    // the request is admitted before any plan is chosen, so a budget that only
    // admitted the two-region materialized program would let a request through
    // and then refuse the split at assembly, reporting a caller's request as a
    // compiler-output defect.
    //
    // Four dispatches per *declared output*, because a region count belongs to a
    // plan and each ordered named output carries its own producer chain:
    // prologue, partial, final, and the elementwise epilogue that reads the
    // fold's staged result. Four is that chain's measured stage count
    // (`crate::pipeline::tests::the_widest_assembled_plan_is_the_split_reduction_with_its_epilogue`,
    // whose reassociation-forbidding neighbour attributes the fourth stage to
    // the split rather than to the epilogue), and it is the widest chain the
    // recognizer can spell for one output. The outputs' walks partition the
    // program's occurrences — `check_output_cover` proves it — so their chains
    // are disjoint region sets and the assembled plan's stage count is their
    // sum.
    //
    // It was the bare literal `4` while recognition could name only one output,
    // and the sentence that justified spelling it — that the widest plan this
    // profile assembles is that chain whatever the program declares — stopped
    // being true when multi-output admission landed: two independent chains pass
    // every other budget and assemble seven or eight stages against a bound of
    // four, which is exactly the request the boundary admitted and assembly then
    // refused.
    check_budget(
        BudgetResource::Regions,
        budgets.regions,
        program.output_count().saturating_mul(4),
    )?;
    // Derived from the declared arity rather than spelled, because it is an
    // upper bound over every plan the request could reach and the widest of
    // those grows with *both* arities: three program-scoped nodes — the element
    // width, the workgroup width, and the applicability guard — one element
    // count and one byte count per declared input, and per declared output its
    // own pair together with its chain's staged partial tensor's pair.
    //
    // One input and one output reach nine, which is what the split program
    // declared when this was a literal. The bound is deliberately loose: the
    // two-input contraction's own demand is nine by a different route — it
    // declares no partial tensor and one further input — and the widest
    // one-input chain declares seven, because an upper bound over every
    // reachable plan cannot also be each plan's exact count.
    check_budget(
        BudgetResource::HostExpressionNodes,
        budgets.host_expression_nodes,
        program
            .input_count()
            .saturating_mul(2)
            .saturating_add(program.output_count().saturating_mul(4))
            .saturating_add(3),
    )?;
    // The widest buffer count any plan for this request could reach: every
    // declared input, and four per declared output — the prologue's materialized
    // temporary, a split's staged partial tensor, the fold's staged result that
    // an elementwise epilogue reads, and the output itself. A standalone
    // elementwise output binds only the last of the four and a contraction only
    // its output, so this bounds them too, which is what lets it be checked
    // before a strategy has been chosen.
    //
    // The per-output four is measured rather than enumerated from the
    // vocabulary, by
    // `crate::pipeline::tests::the_widest_assembled_plan_binds_four_buffers_per_declared_output`:
    // it was three while the enumeration stopped at the split's partial tensor,
    // which under-reported the epilogue's staged read by one for every output —
    // one declared input reaches five values, not four.
    check_budget(
        BudgetResource::Buffers,
        budgets.buffers,
        program
            .input_count()
            .saturating_add(program.output_count().saturating_mul(4)),
    )?;
    Ok(())
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
    target: &TargetProfile,
    program_arithmetic: Option<ArithmeticType>,
) -> Result<StrictF32NumericalContract, RequestError> {
    let mut rejections = Vec::new();
    for contract in preference.stated() {
        // A contract about another width is skipped rather than rejected,
        // because it was never asked: `verify_request` has already refused a
        // preference in which *every* entry is inapplicable, so reaching here
        // means some applicable entry exists and this one simply is not it.
        // Pushing a rejection would report a profile declining a question no
        // profile was put.
        if program_arithmetic.is_some_and(|program| contract.arithmetic != program) {
            continue;
        }
        let outcome = crate::physical::assess_contract(target, *contract).map_err(|_| {
            RequestError::UnsupportedCapability {
                phase: "numerics",
                rule: "target-profile-malformed",
            }
        })?;
        match outcome {
            crate::target::feasibility::FeasibilityOutcome::Proven(_) => return Ok(*contract),
            crate::target::feasibility::FeasibilityOutcome::Rejected(rejection) => {
                // The representative is the canonical-first unhonourable
                // dimension; a contract-only proposal has no capability
                // requirements, so it is always a numerical cause.
                if let crate::target::feasibility::RejectionCause::Numerical(cause) =
                    rejection.representative()
                {
                    rejections.push(ContractRejection::Unhonourable {
                        contract_key: contract.key,
                        cause,
                    });
                }
            }
            crate::target::feasibility::FeasibilityOutcome::Unknown(unknown) => {
                rejections.extend(unknown.dimensions().first().map(|cause| {
                    ContractRejection::Undeclared {
                        contract_key: contract.key,
                        cause: cause.clone(),
                    }
                }));
            }
            crate::target::feasibility::FeasibilityOutcome::Deferred(deferred) => {
                rejections.extend(deferred.dimensions().first().map(|cause| {
                    ContractRejection::Deferred {
                        contract_key: contract.key,
                        cause: cause.clone(),
                    }
                }));
            }
        }
    }
    Err(RequestError::NoResolvableNumericalContract {
        target_profile: target.profile_key().clone(),
        rejections,
    })
}

/// Recognizes one verified semantic program, or explains what it could not
/// recognize.
///
/// # What generalized, and what the generalization rests on
///
/// This is **not** a match against whole-program templates. The program-wide
/// properties every recognized program shares — at least one declared input, and
/// one recognized arithmetic type throughout — are checked once and each names
/// its own rule, and the
/// program's shape is then decided per declared output by *the occurrence that
/// produces it*, walked outward through the occurrences that feed it. A program
/// whose exact shape nothing here was taught is admitted when every occurrence
/// it contains is one the physical layer can realize and they compose into a
/// region chain it can assemble; nothing asks whether the whole graph matches a
/// spelling.
///
/// Concretely, the elementwise dimension is now the general
/// [`PointwiseF32Expression`] vocabulary rather than a leaf count. A reduction
/// over `(a * b) + c` with three declared inputs, a whole-program
/// `((a * 2.0) + b) * c` over two, and a chain sharing one subexpression at
/// several places are all admitted by the same walk that admits the scale-bias
/// program the old template spelled, and none of them was a shape this boundary
/// had been taught.
///
/// # What is still refused, and where the wall actually is
///
/// Recognition may only admit what the physical layer can express, so the walls
/// below this boundary are refused *at* it, each under its own rule:
///
/// - **An operation the region vocabulary cannot spell** (`operation-set`). A
///   family whose per-point body no [`PointwiseF32Expression`] node composes and
///   whose access relation no [`tiler_ir::schedule::LogicalAccess`] denotes has
///   no region to be built into, and a region for it is a `tiler-ir` widening
///   rather than a projection this boundary could make.
///
///   **A family realized as a region *sequence* is the one shape that leaves
///   this rule without being expressible per point, and the admitting fact is a
///   registry row.** An occurrence whose registered
///   [`tiler_ir::index::IndexRealizationLaw`] realizes a sequence computes
///   several regions' worth of work, so no single per-point body was ever going
///   to spell it; what makes it recognizable is that the law says how many
///   regions there are and region formation enumerates one candidate per stage.
///   [`recognize_staged_family`] is that arm and it names no operation key, so
///   `tiler::rms-norm-f32@1` and any family registered after it are recognized
///   by the same statement. `tiler::softmax-f32@1` still refuses here because it
///   carries no law at all — `crates/tiler-compiler/tests/softmax_recognizer_boundary.rs`
///   measures that — and a recognized staged family still stops one layer down,
///   where no [`tiler_ir::schedule::ScalarProgram`] spells a stage's work
///   ([`crate::physical::RegionVocabularyWall::StagedFamilyUnspellable`]).
///
///   **Two families used to be named here and no longer are, for two different
///   reasons, and both distinctions are worth keeping.** `tiler::silu-f32@1` has
///   no node of its own, but its per-point body is expressible in the node
///   vocabulary, so the boundary *projects* it — admissibly, because the
///   projection is not written here:
///   [`crate::elementary::silu_point_body`] is the one statement of the
///   composition and the governed index-access lowering drives the same
///   function, so occurrence refinement's proof that the emitted region realizes
///   the occurrence covers the projection. `tiler::reindex-f32@1` and
///   `tiler::broadcast-f32@1` project no body at all; they were refused because
///   `LogicalAccess` spelled neither access relation, and
///   `admit-the-structural-families-into-the-scheduled-region-vocabulary` landed
///   `LogicalAccess::ReindexBijection` and
///   `LogicalAccess::BroadcastReplication`, so each is now recognized by
///   [`recognize_structural_read`] as a *mapped read* contributing addressing and
///   no arithmetic.
/// - **An elementwise stage reading a materialized intermediate**
///   (`operation-set` from the contraction cover, `elementwise-shape` or
///   `operation-set` from the elementwise walk). Every elementwise region this
///   profile builds reads declared input tensors and nothing else, so a
///   contraction or a reduction feeding an elementwise epilogue is a *chain*
///   rather than a refusal.
///
///   **The wall was in the schedule vocabulary rather than in this crate, and it
///   is gone in all three of its rows.** The paragraph that used to stand here
///   reasoned from `tiler_ir::schedule::TensorRole::Intermediate` being a
///   per-region role to the conclusion that nothing in `tiler-ir` forbade the
///   chain. The role is indeed per-region; what forbade the chain was the
///   *access contract* each scalar-program family declares around it.
///   `verify_pointwise_region` required read access `i` to be
///   `TensorRole::Input { ordinal: i }` at every position, so no
///   `ScalarProgram::PointwiseF32` region could read an intermediate at all —
///   `admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary`
///   separated the access position from the declared input the role names, and a
///   pointwise region may now read one materialized intermediate alongside
///   non-descending declared inputs. `verify_access_and_semantics` then
///   admitted a fold only when its owning write targeted `TensorRole::Output`,
///   and `admit-a-strict-serial-fold-that-writes-a-materialized-intermediate`
///   replaced that with a cover-assigned obligation at every committing pass. A
///   contraction could already write one.
///   `crates/tiler-compiler/tests/materialized_intermediate_epilogue_wall.rs`
///   measures all three rows.
///
///   [`recognize_epilogue`] is what builds the chain from a recognized program:
///   the elementwise walk *names* the value a folding family produced instead of
///   stopping at it, the producer is recognized as its own shape, and the cover
///   search places the materialization edge between them. What is still refused
///   under `operation-set` is a walk that reaches a *second, different* folded
///   value, and that is a rule about chain **width** rather than depth: the two
///   folds would feed one region two `TensorRole::Intermediate` reads, which is
///   the same unordinalled-role fact [`record_leaf`] refuses for one staged value
///   read twice.
///   [`admit-a-scheduled-region-that-reads-two-materialization-edges`](../../../tickets/admit-a-scheduled-region-that-reads-two-materialization-edges.md)
///   owns the region vocabulary underneath both. The chain-*depth* rule is
///   [`recognize_staged_family`]'s `staged-operand-depth`, stated once at
///   [`StagedOperandAdmission`], which also names the third folded-value wall —
///   a fold whose prologue is itself a chain — and its separate owner.
/// - **A staged family reading a materialized intermediate is admitted**, which
///   is where the last of this rule's rows moved.
///   [`admit-a-staged-family-that-reads-a-materialized-intermediate`](../../../tickets/admit-a-staged-family-that-reads-a-materialized-intermediate.md)
///   gave the recognized staged shape a per-operand [`BoundaryRead`] and the
///   producer whose regions write the edge, so `rms_norm(matmul(a, b), w)` is one
///   output's partition rather than a `staged-operand` refusal. It stops one
///   layer down instead: the consuming stage would read that edge *and* the value
///   the producing stage handed it, which is two `TensorRole::Intermediate`
///   accesses, so [`crate::physical::staged_plan`] declines the occurrence and
///   `crates/tiler-compiler/tests/staged_family_over_a_materialized_intermediate.rs`
///   measures where that leaves it.
///
/// **A reduction reading a declared input directly was the third wall here, and
/// it is gone.** `sum(x)` was refused under `reduction-prologue` because
/// `verify_access_and_semantics` required a `ScalarProgram::StrictSerialSum`
/// region's contributor access to read `TensorRole::Intermediate`;
/// `admit-a-reduction-over-a-declared-input-tensor` widened that arm to the fold's
/// *declared contributor domain*, which is the input tensor the program folds
/// directly or an intermediate when a prologue region wrote it.
/// [`recognize_reduction`] therefore admits the shape with no prologue at all, and
/// the rule name no longer exists.
///
/// Which refusal a rejected program reports is settled by the occurrence it
/// actually ends in rather than by enumeration order: a program whose output is
/// a reduction gets the reduction's reason, one whose output is a contraction
/// gets the contraction's, and any other gets the elementwise walk's. With
/// several declared outputs the walks run in declaration order and the first
/// one that cannot be recognized reports, so the rule names a property of the
/// caller's own interface rather than of a traversal it cannot see.
///
/// # Ordered multi-output programs are admitted, and the arity guard is gone
///
/// This function used to open with `output_count() != 1`, refusing every
/// multi-output program under `output-arity` before any occurrence was
/// classified. That refusal is gone in both of the places it stood — here and
/// `verify_artifact_refinements`'s `semantic-output-coverage` arity check — and
/// nothing replaced it with a narrower cardinality rule: a program declaring
/// several ordered named outputs is now recognized, covered, planned, and
/// assembled like any other.
///
/// What made it removable is that every layer it was standing in for now answers
/// for itself. [`recognize_program_outputs`] walks each declared output and
/// [`check_output_cover`] requires the walks to *partition* the occurrences, so
/// every occurrence is claimed exactly once and every published value has one
/// region that owns its write. The cover carries which named result each region
/// retains, so `CoverAssembly::from_plan` attributes each declared output to its
/// publishing region *by value* rather than by execution order — the pairing
/// that made the guard load-bearing after recognition had already been widened.
///
/// **What a multi-output program is still refused for is the shape of its
/// outputs, never their number.** Two outputs whose walks share an occurrence
/// refuse under `output-partition-overlap`, which is the branch where one
/// region's owning write would have to serve both a materialization edge and a
/// publication: two keys naming one value, and a published intermediate that is
/// also consumed. [`tiler_ir::program::ValueRole`] is exclusive and a region
/// writes one owning tensor, so a cover this boundary let through would die
/// mid-pipeline instead.
///
/// **One spelling of the overlap is now admitted, and the four walls that
/// blocked it came down together.** A published-and-consumed intermediate is
/// realized as two dispatches of the region that computed it — one staging the
/// value the fold reads across, one publishing a copy of it — so the shape is
/// no longer refused here. [`check_output_cover`] owns that rule and states the
/// measured order the four walls fell in; it is not restated here, because two
/// derivations of one measurement are what drift.
/// [`crate::pipeline::conformance`]'s
/// `a_published_and_consumed_intermediate_compiles_and_agrees` is the compiling
/// assertion, `an_output_key_pair_naming_one_value_still_refuses_by_name` is the
/// neighbour that must keep refusing, and
/// `crates/tiler-compiler/tests/multi_output_boundary.rs` holds the evidence for
/// where the boundary now is.
fn select_supported_strategy(
    program: &SemanticProgram,
    laws: &FrozenIndexRealizationLawRegistry,
) -> Result<NormalizedProgram, RequestError> {
    // Program-wide properties first, each under the rule that names it. A
    // program failing one of these fails it for every shape below, so reporting
    // it here is both the more specific statement and the only one that does not
    // depend on which occurrence happens to produce the output.
    if program.input_count() == 0 {
        return mismatch("input-arity");
    }
    let arithmetic = recognized_program_arithmetic(program)?;
    recognize_program_outputs(program, laws, arithmetic)
}

/// The one arithmetic type every value of a recognizable program is stated in.
///
/// **This replaced a `dtype-f32` gate, and the two refusals it splits into are
/// different findings.** The gate refused every program carrying a non-`f32`
/// value, which conflated "this build states no per-point vocabulary for that
/// width" with "this program mixes two widths and therefore has no single scalar
/// program at all". Both still refuse, and each now names the property it found:
///
/// - `dtype-recognized` for a value whose resolved type is neither of the two
///   widths [`RecognizedPointwise`] can spell. Every conversion family is in this
///   arm, which is correct rather than incidental — a program that converts
///   between widths has no *one* arithmetic and a region carrying one realization
///   record cannot realize it.
/// - `dtype-uniform` for a program whose values are two recognized widths at
///   once. A scheduled region carries one [`ArithmeticType`] worth of numerical
///   realization and one scalar-program vocabulary, so a mixed-width program is
///   refused here rather than compiled under whichever width happened to be
///   first.
///
/// **What it deliberately does not decide is whether the width can be
/// dispatched or its contract honoured.** Those are the target profile's and the
/// numerical contract's, they run before this function, and each reports its own
/// typed refusal: [`require_compile_profile_dispatch`] for a width the profile
/// names no dispatch fact for, [`resolve_numerical_contract`] for a contract no
/// stated entry resolves, and
/// [`RequestError::NoApplicableNumericalContract`] for a preference no entry of
/// which is about this program's arithmetic at all.
fn recognized_program_arithmetic(
    program: &SemanticProgram,
) -> Result<ArithmeticType, RequestError> {
    let mut recognized: Option<ArithmeticType> = None;
    for value in program.values() {
        let Some(arithmetic) = recognized_arithmetic(value.resolved_type()) else {
            return mismatch("dtype-recognized");
        };
        match recognized {
            Some(seen) if seen != arithmetic => return mismatch("dtype-uniform"),
            Some(_) => {}
            None => recognized = Some(arithmetic),
        }
    }
    // Unreachable through the caller, which has already refused a program
    // declaring no input, and refused by name rather than defaulted: a width
    // nothing derived is not a width this build may compile under.
    recognized.ok_or(RequestError::UnsupportedCapability {
        phase: "strategy",
        rule: "dtype-recognized",
    })
}

/// The arithmetic type one resolved value type names, when this build states a
/// per-point vocabulary for it.
///
/// **The single statement of which widths recognition admits.** Every authority
/// that needs the set asks this rather than restating it —
/// [`recognized_program_arithmetic`] derives a program's width from it, and
/// [`crate::program::verify_semantic_output_type`] checks a declared output
/// against it — because two lists would be free to disagree about which
/// programs the compiler claims it can plan, and the disagreement's shape is a
/// program admitted by one and refused by the other after a plan exists.
pub(crate) fn recognized_arithmetic(resolved: &ResolvedValueType) -> Option<ArithmeticType> {
    if resolved == &F32::resolved_type() {
        Some(ArithmeticType::F32)
    } else if resolved == &Bf16::resolved_type() {
        Some(ArithmeticType::Bf16)
    } else {
        None
    }
}

/// Recognizes every ordered named output of one verified program.
///
/// **One walk per declared output, and the outputs are recognized in declaration
/// order** — the order is identity rather than presentation, and the recognized
/// list preserves it so that the request subject, the cover's named-output
/// attribution, and the assembled program's interface all speak about the same
/// ordering the caller declared.
///
/// The per-output walk is exactly the one a single-output program has always
/// taken: the occurrence producing the output decides the shape, and the
/// occurrences feeding it are walked outward. What changed is where the
/// whole-program obligation lives. Each recognizer used to end by demanding that
/// its own walk cover the program exactly; that demand is now
/// [`check_output_cover`]'s, stated over the walks together, and it is strictly
/// the same requirement when there is one output.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `missing-output` for a
/// program declaring none, every rule the per-output recognizers report, and the
/// two [`check_output_cover`] states: `operation-set` for an occurrence no
/// walk claimed, and `output-partition-overlap` for one claimed twice.
fn recognize_program_outputs(
    program: &SemanticProgram,
    laws: &FrozenIndexRealizationLawRegistry,
    arithmetic: ArithmeticType,
) -> Result<NormalizedProgram, RequestError> {
    if program.output_count() == 0 {
        return unsupported("strategy", "missing-output");
    }
    let mut outputs = Vec::with_capacity(program.output_count());
    for output in program.outputs() {
        outputs.push(recognize_output(program, &output, laws, arithmetic)?);
    }
    check_output_cover(program, &outputs)?;
    Ok(NormalizedProgram { outputs })
}

/// Recognizes the region partition implementing one ordered named output.
fn recognize_output(
    program: &SemanticProgram,
    output: &tiler_ir::semantic::ProgramOutputRef<'_>,
    laws: &FrozenIndexRealizationLawRegistry,
    arithmetic: ArithmeticType,
) -> Result<NormalizedOutput, RequestError> {
    // An output that *is* a declared input computes nothing: it names no
    // operation for any region to realize. The property that was not recognized
    // is its operation set, so it is reported under that rule rather than as the
    // missing producer a bare graph walk would report.
    if program
        .inputs()
        .any(|input| input.value() == output.value())
    {
        return mismatch("operation-set");
    }
    let (member, root) = producer_for_value(program, output.value())?;
    if root.key() == &strict_serial_sum_f32_op() {
        recognize_reduction(
            program,
            output.value(),
            output.key().clone(),
            member,
            &root,
            laws,
        )
        .map(NormalizedOutput::SerialSum)
    } else if root.key() == &strict_tensor_contraction_f32_op() {
        normalize_contraction(program, output.value(), output.key().clone())
            .map(|normalized| NormalizedOutput::Contraction(Box::new(normalized)))
    } else if laws.family_realizes_region_sequence(root.key()) {
        recognize_staged_family(
            program,
            laws,
            output.value(),
            output.key().clone(),
            member,
            &root,
            // The declared output's own occurrence is at the near side of every
            // materialization edge this walk may place, so it is the one that
            // may read one. See [`recognize_staged_family`]'s
            // `staged-operand-depth` refusal for the far side.
            StagedOperandAdmission::OneEdge,
        )
        .map(|normalized| NormalizedOutput::Staged(Box::new(normalized)))
    } else {
        recognize_elementwise_output(program, output, laws, arithmetic)
    }
}

/// Recognizes an output whose producing occurrence is elementwise.
///
/// **Two shapes share this entry, and which one a program is depends on a fact
/// only the walk can report.** An elementwise expression over declared inputs is
/// one region; the same expression over a *folded* value is two, because no
/// per-point body spells a fold. The walk is run once and its answer decides:
/// a completed plan is the whole-program shape, and a plan that stopped at a
/// folding family names the value the chain materializes.
///
/// Deciding it this way rather than by pre-scanning the graph is deliberate. A
/// pre-scan would be a second classifier of the same operand DAG, and the two
/// would have to agree about constants, structural occurrences, shapes, and
/// arity for the answer to mean anything — which is exactly the drift a single
/// authority exists to prevent.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `output-handle` for an
/// output the program holds no shape for, `elementwise-rank` for a rank-zero
/// domain no region iterates, and every rule [`plan_elementwise`],
/// [`mint_elementwise`], and [`recognize_epilogue`] report.
fn recognize_elementwise_output(
    program: &SemanticProgram,
    output: &tiler_ir::semantic::ProgramOutputRef<'_>,
    laws: &FrozenIndexRealizationLawRegistry,
    arithmetic: ArithmeticType,
) -> Result<NormalizedOutput, RequestError> {
    let declared: Vec<ValueId> = program.inputs().map(|input| input.value()).collect();
    let shape = static_shape(program, output.value(), "output-handle")?;
    if shape.rank() == 0 {
        return mismatch("elementwise-rank");
    }
    let leaves = ElementwiseLeaves {
        declared: &declared,
        staged: None,
    };
    match plan_elementwise(program, output.value(), &leaves, &shape, laws, arithmetic) {
        Ok(plan) => recognize_pointwise(program, output, &declared, shape, plan, arithmetic)
            .map(NormalizedOutput::Pointwise),
        Err(ElementwiseRefusal::Folded(staged)) => {
            recognize_epilogue(program, output, &declared, shape, staged, laws, arithmetic)
                .map(|chain| NormalizedOutput::Epilogue(Box::new(chain)))
        }
        Err(ElementwiseRefusal::Refused(error)) => Err(error),
    }
}

/// Requires the recognized walks to partition the program's occurrences and to
/// read every declared input between them.
///
/// **Three obligations, and they are separate claims about different failures.**
///
/// *Every occurrence is claimed by some walk* (`operation-set`). A built program
/// retains only output-reachable operations, so an unclaimed one is work no
/// region would compute and the assembled program would silently drop. This is
/// the widened form of the check each recognizer used to make alone — with one
/// declared output the union is that output's own member set and the rule is
/// unchanged, which is why widening it rather than removing it is what keeps the
/// uncovered case refused.
///
/// *Every occurrence claimed twice is claimed by the one admitted overlap*
/// (`output-partition-overlap`). Two outputs whose walks share an occurrence are
/// the shape where one value is both published and consumed, and exactly one
/// spelling of it is admitted — [`published_and_consumed_overlap`] is the
/// predicate and states what it proves. Everything else still refuses here,
/// including two output keys naming one value, because
/// [`tiler_ir::program::ValueRole`] is exclusive and a dispatch owns one write,
/// so a cover the boundary let through would die mid-pipeline instead.
///
/// **What lifted the admitted case was four walls, and only the last was a crate
/// down.** Measured by disabling each in turn against a governed spelling of the
/// fixture: this rule; then [`crate::program`]'s `cover-named-output-attribution`
/// and its `internal-unwritten`, both widenings *here*, because the cover legally
/// places one region as both the edge's producer and the publication's retainer
/// and that region needs a *second dispatch* to write the publication; and only
/// then `tiler_ir::program`'s `UncoveringStage`, which its publishing-copy
/// declaration now accounts for. [`crate::pipeline::conformance`]'s
/// `a_published_and_consumed_intermediate_compiles_and_agrees` is the compiling
/// assertion and `an_output_key_pair_naming_one_value_still_refuses_by_name` is
/// the neighbour that must keep refusing.
///
/// *Every declared input is read by some walk* (`input-set`). This is the
/// obligation [`canonical_input_reads`] used to state per walk, under
/// `elementwise-reads`, and it was the same requirement while a program had one
/// declared output: that walk's read set was the program's. With several
/// outputs the walks split the declared inputs between them, so the per-walk
/// form refused a program whose *union* was complete —
/// `admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs` is
/// what moved it here rather than deleting it. What it protects is unchanged:
/// a declared input no region reads is a buffer the caller binds, the ABI
/// declares, and no kernel loads.
///
/// **It is defence in depth here, and the derivation says so rather than the
/// check pretending otherwise.** `SemanticProgramBuilder` freezes only
/// output-reachable values, so a retained declared input is an operand of some
/// retained occurrence; the `operation-set` obligation above claims every
/// retained occurrence for some walk; and every way a walk consumes an operand
/// — an elementwise node, a structural occurrence, a fold's contributor, a
/// contraction's operand — records a read of it. So no program the public
/// builder can construct reaches this refusal, and `tiler_ir::program`'s
/// `verify_usage` refuses the same shape a layer down under `unused-value`.
/// Stating it here is what makes the boundary report the program property
/// instead of letting an assembled program die naming a different authority.
///
/// Claimed counts are taken over the deduplicated per-output member sets, so one
/// constant shared by two operands of the *same* walk contributes one member
/// rather than two — the normalized spelling of one program, not a duplicate.
fn check_output_cover(
    program: &SemanticProgram,
    outputs: &[NormalizedOutput],
) -> Result<(), RequestError> {
    let claimed: Vec<Vec<SemanticStage>> = outputs.iter().map(NormalizedOutput::members).collect();
    let total: usize = claimed.iter().map(Vec::len).sum();
    let mut distinct: Vec<SemanticStage> = claimed.iter().flatten().copied().collect();
    distinct.sort_unstable();
    distinct.dedup();
    if total != distinct.len()
        && published_and_consumed_overlap(program, outputs, &claimed).is_none()
    {
        return mismatch("output-partition-overlap");
    }
    if program.operation_count() != distinct.len() {
        return mismatch("operation-set");
    }
    for position in 0..program.input_count() {
        let Ok(ordinal) = u32::try_from(position) else {
            return mismatch("input-ordinal");
        };
        if !outputs
            .iter()
            .any(|output| output.reads_declared_input(ordinal))
        {
            return mismatch("input-set");
        }
    }
    Ok(())
}

/// Recognizes the one overlap between two recognized walks this boundary admits,
/// as `(published output, consuming output)` declaration positions.
///
/// **The predicate is not "any overlap", and each conjunct is load-bearing.**
///
/// *Exactly one pair of walks overlaps.* A value published and consumed by two
/// different downstream outputs, or two independent published-and-consumed
/// values, would need a cover shape nothing below here expresses — the first
/// because `cover-region-multiple-materializations` refuses a region producing
/// two edges, the second because both would have to be the same region's second
/// dispatch. They are unsupported cases that reject explicitly rather than being
/// approximated.
///
/// *One walk's member set is a strict subset of the other's.* Two walks that
/// merely intersect share work neither wholly owns, which is the shape where one
/// region's write would have to serve two publications.
///
/// *The shorter walk is one whole **part** of the longer walk's recognized
/// partition*, asked through [`NormalizedOutput::owns_region_members`] — the same
/// authority [`crate::physical::spell_region`] resolves a region against. A
/// subset that is not a part has no scheduled region of its own, so nothing could
/// publish it without splitting a region the recognizer did not split.
///
/// *The published value is the one crossing the part boundary*: some occurrence
/// of the longer walk **outside** the part reads it. That is what makes the
/// publication and the materialization edge the same value, and it is the
/// conjunct that distinguishes this from a subset walk publishing some *other*
/// value the part happens to compute.
///
/// What it does not prove is that a cover placing the part as one region exists;
/// that is the cover search's answer, and a program admitted here whose cover
/// cannot be assembled is refused by name at the assembler.
fn published_and_consumed_overlap(
    program: &SemanticProgram,
    outputs: &[NormalizedOutput],
    claimed: &[Vec<SemanticStage>],
) -> Option<(usize, usize)> {
    let mut overlapping = None;
    for short in 0..claimed.len() {
        for long in (short + 1)..claimed.len() {
            if !claimed[short]
                .iter()
                .any(|member| claimed[long].contains(member))
            {
                continue;
            }
            if overlapping.is_some() {
                return None;
            }
            overlapping = Some((short, long));
        }
    }
    let (first, second) = overlapping?;
    // Orient the pair by containment rather than by declaration order: which
    // output publishes the consumed value is a fact about the walks, and the
    // caller may declare them either way round.
    let (published, consuming) = if claimed[first].len() < claimed[second].len() {
        (first, second)
    } else {
        (second, first)
    };
    if claimed[published].len() >= claimed[consuming].len()
        || !claimed[published]
            .iter()
            .all(|member| claimed[consuming].contains(member))
    {
        return None;
    }
    if !outputs[consuming].owns_region_members(&claimed[published]) {
        return None;
    }
    let staged = program.outputs().nth(published)?.value();
    let crosses = program
        .operations()
        .enumerate()
        .filter(|(ordinal, _)| {
            u32::try_from(*ordinal).is_ok_and(|ordinal| {
                !claimed[published]
                    .iter()
                    .any(|atom| atom.member().0 == ordinal)
            })
        })
        .any(|(_, operation)| operation.operands().any(|operand| operand == staged));
    crosses.then_some((published, consuming))
}

/// One recognized elementwise expression and the occurrences it covers.
struct RecognizedElementwise {
    expression: RecognizedPointwise,
    members: Vec<SemanticStage>,
    /// One entry per expression input leaf, in access order.
    ///
    /// Parallel to the region's reads: leaf `i` is served by entry `i`, which
    /// names the declared input ordinal it binds and the relation it addresses
    /// that tensor with. An ordinal appears twice when one declared input is
    /// read both densely and through a relation.
    reads: Vec<(u32, LogicalAccess)>,
}

/// Which values one elementwise walk *reads* rather than computes.
///
/// **The leaf set and the leaf *order* are separate facts, and separating them
/// is what makes an epilogue expressible.** A whole-program or prologue walk
/// reads exactly the declared program inputs and numbers its expression leaves
/// by declaration position, because its region binds one buffer per declared
/// input in that order. An epilogue additionally reads the value an earlier
/// region staged, reads only *some* of the declared inputs, and numbers its
/// leaves by the position of the read that serves them — which is not the
/// declaration ordinal. [`plan_elementwise`] decides the set and the validation;
/// [`mint_elementwise`] is handed the order.
struct ElementwiseLeaves<'a> {
    /// The program's declared input values, in declaration order.
    declared: &'a [ValueId],
    /// The producer result an epilogue reads as a materialized value.
    ///
    /// `None` for every walk that reads only declared inputs, which keeps the
    /// classification below one rule rather than two.
    staged: Option<ValueId>,
}

impl ElementwiseLeaves<'_> {
    /// Returns whether one value is read rather than computed by this walk.
    fn is_leaf(&self, value: ValueId) -> bool {
        self.staged == Some(value) || self.declared.contains(&value)
    }
}

/// One tensor a walk reads, and the relation it addresses it with.
///
/// **The relation is part of the leaf's identity, not an annotation on it.** One
/// expression may name a tensor twice meaning two different things — `a *
/// permute(a)` reads declared input `0` densely *and* through a transposition —
/// and those two leaves need two reads with two relations. Keying leaves by
/// value alone made them one leaf, so the region bound one access for both and
/// `a * permute(a)` compiled as `permute(a) * permute(a)`. Two reads of one
/// tensor under the *same* relation address identically and stay one leaf, which
/// is what keeps `(a * a) + b` one read of `a`.
#[derive(Clone, Debug, Eq, PartialEq)]
struct LeafRead {
    value: ValueId,
    map: LogicalAccess,
}

/// What one step of a planned elementwise walk mints.
///
/// The steps are in mint order, so a node's operands are always already minted
/// when it is reached — which is the property that lets [`mint_elementwise`]
/// replay a plan against any leaf ordering without re-deciding anything.
enum ElementwiseMint {
    /// A read of one leaf tensor value under one relation.
    ///
    /// The leaf's value is the step's own for a direct read, and the structural
    /// occurrence's *operand* when one interposed: a structural occurrence
    /// computes nothing, so the value it produces is minted as the leaf that
    /// reads the tensor behind it under the relation the family denotes.
    Read { leaf: LeafRead },
    /// An exact `f32` constant leaf.
    Constant(u32),
    /// One node of the recognized vocabulary over already-minted operands.
    Node(ElementwiseFamily, Vec<ValueId>),
}

/// Why one elementwise walk did not complete.
///
/// **The second variant is the epilogue's discovery, and it is a variant rather
/// than a rule code because the caller acts on it.** A walk that reaches a value
/// produced by a folding family has not found an unrecognizable program; it has
/// found the *boundary* between two regions, and the value it names is the one a
/// cover materializes. Reporting it as `operation-set` — which is what a caller
/// that only wants a whole-program expression still does — would throw away the
/// one fact the epilogue recognizer needs.
enum ElementwiseRefusal {
    /// The typed refusal to report.
    Refused(RequestError),
    /// The walk reached a value a folding family produces.
    ///
    /// Raised only for a walk that has no staged value yet: a walk that already
    /// reads one and reaches a second has nothing to attribute the second read
    /// to, so it is refused rather than reported as another boundary. That is a
    /// bound on how many edges reach *one* region and not on how deep the chain
    /// is; [`StagedOperandAdmission`] states the depth rule and separates the
    /// two.
    Folded(ValueId),
}

impl From<ElementwiseRefusal> for RequestError {
    /// Flattens a discovered materialization boundary into the rule a caller
    /// with no epilogue to build reports for it.
    ///
    /// **This is where a fold's chained prologue is refused, and it is a third
    /// wall rather than either of the two above.** [`recognize_reduction`]'s
    /// contributor walk is the only caller that reaches it with a `Folded`
    /// finding, and it discards the finding because [`NormalizedSerialSum`]
    /// carries no producer field to hang the boundary on — so `sum(sum(x) * 2.0)`
    /// reports `reduction-contributor-materialization` here rather than reaching
    /// [`StagedOperandAdmission`]'s guard, which never runs for it.
    /// [`name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set`](../../../tickets/name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set.md)
    /// owns the rule name.
    fn from(refusal: ElementwiseRefusal) -> Self {
        match refusal {
            ElementwiseRefusal::Refused(error) => error,
            ElementwiseRefusal::Folded(_) => Self::UnsupportedCapability {
                phase: "strategy",
                rule: "reduction-contributor-materialization",
            },
        }
    }
}

/// A validated elementwise expression, linearized in mint order.
///
/// **Every rule the recognizer states is discharged here**, so the only thing
/// left for minting is the arithmetic of node identifiers. That split exists
/// because two callers need the same validation under different leaf numbering,
/// and a second walk written for the second numbering would be a second
/// classifier that could drift from this one.
struct ElementwisePlan {
    /// Each value the walk mints, in mint order.
    steps: Vec<(ValueId, ElementwiseMint)>,
    /// The distinct leaf reads, in first-mint order.
    leaves: Vec<LeafRead>,
    members: Vec<SemanticStage>,
    root: ValueId,
}

/// The elementwise operation families this recognizer projects.
///
/// Exactly the families whose per-point body the physical expression vocabulary
/// can express. Two are single nodes of that vocabulary; the third is a
/// composition, and the distinction between "one node" and "expressible" is
/// where this set used to stop.
///
/// **`tiler::silu-f32@1` is projected rather than restated, and the difference
/// is the whole reason it is admissible here.** No `PointwiseF32Node` spells a
/// sigmoid-weighted linear unit, so the projection is a subtree — but the
/// subtree is not written in this module. [`crate::elementary::silu_point_body`]
/// is the one statement of the composition in this crate, and the governed
/// index-access lowering emits the *same* function into the scalar vocabulary
/// its regions are built from. So the boundary is not re-deriving a provider's
/// lowering; both realizations are driven from one authority, and occurrence
/// refinement independently proves that the resolved provider's emitted region
/// realizes the occurrence.
#[derive(Clone, Copy)]
enum ElementwiseFamily {
    Add,
    Multiply,
    /// The activation, projected through [`crate::elementary::silu_point_body`].
    Silu,
}

impl ElementwiseFamily {
    /// The operand count this family's occurrences declare.
    ///
    /// Read from the family rather than from the occurrence, so an occurrence
    /// whose arity disagrees with its registered family is refused under
    /// `elementwise-arity` instead of being projected against whichever operands
    /// happened to be present.
    const fn operand_count(self) -> usize {
        match self {
            Self::Add | Self::Multiply => 2,
            Self::Silu => 1,
        }
    }
}

/// Classifies one operation as a recognized elementwise family, or declines.
///
/// **Keyed by the program's arithmetic rather than by trying both vocabularies.**
/// A family's key already names its width, so the two lists are disjoint and a
/// union would classify the same operations; keying on the arithmetic the caller
/// derived is what keeps a `bf16` program from ever being offered an `f32`
/// projection to fail on later. The exhaustive match is what makes a third
/// admitted width a build error here rather than a program silently declining
/// every family.
///
/// The `bf16` row is deliberately shorter. There is no `tiler::silu-bf16@1`
/// registered to classify, and [`PointwiseBf16Node`] has no division or
/// exponential for a projection to land in, so the activation is absent because
/// the vocabulary cannot state it rather than because this list forgot it.
///
/// [`PointwiseBf16Node`]: tiler_ir::schedule::PointwiseBf16Node
fn elementwise_family(
    operation: &tiler_ir::semantic::OperationRef<'_>,
    arithmetic: ArithmeticType,
) -> Option<ElementwiseFamily> {
    match arithmetic {
        ArithmeticType::F32 => {
            if operation.key() == &add_f32_op() {
                Some(ElementwiseFamily::Add)
            } else if operation.key() == &multiply_f32_op() {
                Some(ElementwiseFamily::Multiply)
            } else if operation.key() == &silu_f32_op() {
                Some(ElementwiseFamily::Silu)
            } else {
                None
            }
        }
        ArithmeticType::Bf16 => {
            if operation.key() == &add_bf16_op() {
                Some(ElementwiseFamily::Add)
            } else if operation.key() == &multiply_bf16_op() {
                Some(ElementwiseFamily::Multiply)
            } else {
                None
            }
        }
        // No program of either width reaches this function:
        // [`recognized_program_arithmetic`] refuses every value type that is not
        // one of the two above under `dtype-recognized`. Declining is the
        // fail-closed answer rather than a wildcard that would silently offer one
        // width's families to another.
        ArithmeticType::F16 | ArithmeticType::F64 => None,
    }
}

/// The nullary constant family of one recognized arithmetic type.
///
/// `None` for a width this recognizer states no constant family for, which is
/// the same fail-closed answer [`elementwise_family`] gives and for the same
/// reason.
fn constant_family(arithmetic: ArithmeticType) -> Option<OpKey> {
    match arithmetic {
        ArithmeticType::F32 => Some(constant_f32_op()),
        ArithmeticType::Bf16 => Some(constant_bf16_op()),
        ArithmeticType::F16 | ArithmeticType::F64 => None,
    }
}

/// Recognizes the elementwise expression rooted at one value.
///
/// **General over the graph rather than over a taught shape.** Each operand is
/// classified independently — a declared input tensor becomes the leaf that
/// reads it, a `tiler.constant-f32` occurrence becomes an exact constant leaf,
/// and a recognized elementwise occurrence is walked in turn — so depth, arity,
/// family mixing, and shared subexpressions are properties of the caller's
/// program rather than of a template. Two operands naming one value share the
/// node already minted, which is what makes `(a * a) + b` one read of `a`.
///
/// `shape` is the region's iteration domain, and every tensor read must carry
/// it: the region binds one linear-identity access per read, so an operand at a
/// different shape would be sized by a domain it does not have. A constant is
/// rank-zero and is a literal node rather than a read, so it is deliberately not
/// held to it.
///
/// The walk is iterative over an explicit worklist rather than recursive. The
/// depth is the caller's own longest elementwise chain, and a recognizer that
/// consumed host stack proportional to it would turn an input property into a
/// crash rather than a refusal.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the exact property
/// that was not recognized: `operation-set` for a family the expression
/// vocabulary cannot spell, `elementwise-shape` for a read at another domain,
/// `elementwise-attributes` for an attribute this projection would drop,
/// `elementwise-arity` for an operand count the vocabulary has no node for, and
/// every rule [`resolve_elementwise`] reports for the numbering that follows —
/// which is where `elementwise-expression` comes from, along with
/// `elementwise-reads`, `input-ordinal`, `elementwise-operand`, and
/// `elementwise-node-limit`.
fn recognize_elementwise(
    program: &SemanticProgram,
    root: ValueId,
    declared: &[ValueId],
    shape: &Shape,
    laws: &FrozenIndexRealizationLawRegistry,
    arithmetic: ArithmeticType,
) -> Result<RecognizedElementwise, RequestError> {
    let plan = plan_elementwise(
        program,
        root,
        &ElementwiseLeaves {
            declared,
            staged: None,
        },
        shape,
        laws,
        arithmetic,
    )
    .map_err(RequestError::from)?;
    resolve_elementwise(plan, declared, arithmetic)
}

/// Resolves one planned whole-program or prologue expression against the
/// declared inputs.
///
/// Declaration order is the *group* order here: the region's reads walk the
/// declared inputs in the order the ABI binds them. It is not a one-to-one
/// correspondence with the leaves in either direction — one declared input may
/// be read twice, and one this walk does not reach is read not at all, so the
/// list is a *map* from the expression's dense leaf ordinals to the program's
/// input ordinals. [`canonical_input_reads`] states both orders.
///
/// An epilogue additionally reads a staged value, and [`recognize_epilogue`]
/// states its own order rather than relaxing this one.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `elementwise-reads` for
/// a leaf that is not a declared input, `input-ordinal` for a declaration
/// position no expression ordinal can hold, and every rule
/// [`mint_elementwise`] reports.
fn resolve_elementwise(
    plan: ElementwisePlan,
    declared: &[ValueId],
    arithmetic: ArithmeticType,
) -> Result<RecognizedElementwise, RequestError> {
    let order = canonical_input_reads(&plan.leaves, declared)?;
    let expression = mint_elementwise(&plan, &order, arithmetic)?;
    let reads = order
        .iter()
        .map(|leaf| Ok((declared_ordinal(declared, leaf.value)?, leaf.map.clone())))
        .collect::<Result<Vec<_>, RequestError>>()?;
    Ok(RecognizedElementwise {
        expression,
        members: plan.members,
        reads,
    })
}

/// Orders one walk's leaf reads into the read list a whole-program or prologue
/// region binds.
///
/// **Declared inputs in declaration order, and each input's reads dense-first.**
/// The group order is the ABI's. The order *within* a group is the canonical
/// spelling `tiler_ir::schedule`'s `reads_bind_boundary_tensors_in_order`
/// admits, and it has to be decided here rather than left to the walk: the two
/// reads of `a` in `a * permute(a)` are popped in whichever order the operands
/// happened to be visited, and a read list in walk order would give one program
/// two spellings — and one of them would be refused by the region verifier for
/// no property of the program.
///
/// **A declared input this walk never reads contributes no read, and the
/// ordinals stay the program's.** An output whose expression names two of three
/// declared inputs binds those two, carrying the ordinals the program declared
/// them at rather than a region-local renumbering — which is what
/// `crate::program::CoverAssembly::from_plan` resolves against the declared
/// interface and what `reads_bind_boundary_tensors_in_order` admits, its rule
/// being that declared input ordinals ascend strictly with a gap allowed. This
/// walk therefore skips an unread group instead of refusing it: the obligation
/// that no declared input goes unread by *every* output is a program-scoped
/// property and lives in [`check_output_cover`], where the other program-scoped
/// obligations moved when several ordered outputs became statable.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `elementwise-reads`
/// when some leaf reads a value that is not a declared input. That is
/// unreachable for these walks, whose leaf set is the declared inputs by
/// construction, and is refused rather than assumed away.
fn canonical_input_reads(
    leaves: &[LeafRead],
    declared: &[ValueId],
) -> Result<Vec<LeafRead>, RequestError> {
    let mut order: Vec<LeafRead> = Vec::with_capacity(leaves.len());
    for input in declared {
        for dense in [true, false] {
            order.extend(
                leaves
                    .iter()
                    .filter(|leaf| {
                        leaf.value == *input && (leaf.map == LogicalAccess::LinearIdentity) == dense
                    })
                    .cloned(),
            );
        }
    }
    if order.len() != leaves.len() {
        return mismatch("elementwise-reads");
    }
    Ok(order)
}

/// Returns the expression input ordinal one declared input value occupies.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `elementwise-reads` for
/// a value the declaration does not name and `input-ordinal` for a declaration
/// position no expression ordinal can hold.
fn declared_ordinal(declared: &[ValueId], value: ValueId) -> Result<u32, RequestError> {
    let position = declared.iter().position(|input| *input == value).ok_or(
        RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "elementwise-reads",
        },
    )?;
    u32::try_from(position).map_err(|_| RequestError::UnsupportedCapability {
        phase: "strategy",
        rule: "input-ordinal",
    })
}

/// Records one leaf read at its first sighting, or refuses a second read of one
/// tensor that nothing could attribute.
///
/// **Two reads of one tensor are admitted exactly when a region can tell them
/// apart and order them.** A dense read and a mapped read of one declared input
/// are two different tensors as far as the expression is concerned, and
/// `tiler_ir::schedule`'s `reads_bind_boundary_tensors_in_order` binds the pair
/// in one canonical order — dense first — so the region has one spelling.
///
/// The two refusals are that admission's own boundary rather than separate
/// rules. A *staged* value read twice would need two `TensorRole::Intermediate`
/// accesses, and that role carries no ordinal, so nothing says which
/// materialization edge each binds — the very attribution that makes the input
/// pair unambiguous is what it lacks. Two *structural* relations on one tensor
/// have no canonical order between them, so the pair would have two encodings
/// and the region two identities.
fn record_leaf(
    leaves: &mut Vec<LeafRead>,
    staged: Option<ValueId>,
    read: LeafRead,
) -> Result<(), ElementwiseRefusal> {
    if leaves.contains(&read) {
        return Ok(());
    }
    let already_read = |mapped_only: bool| {
        leaves.iter().any(|leaf| {
            leaf.value == read.value && (!mapped_only || leaf.map != LogicalAccess::LinearIdentity)
        })
    };
    let unattributable = staged == Some(read.value) && already_read(false);
    let unordered = read.map != LogicalAccess::LinearIdentity && already_read(true);
    if unattributable || unordered {
        return refused("structural-access-conflict");
    }
    leaves.push(read);
    Ok(())
}

/// Validates and linearizes the elementwise expression rooted at one value.
///
/// This is the whole of the recognition stated in [`recognize_elementwise`]'s
/// documentation; what it deliberately does not do is choose expression input
/// ordinals, because two callers number their leaves differently and a walk that
/// decided the numbering would have to be written twice.
///
/// # Errors
///
/// Returns every [`RequestError::UnsupportedCapability`]
/// [`recognize_elementwise`] documents except `elementwise-reads`,
/// `elementwise-node-limit`, and `elementwise-expression`, which are properties
/// of a *numbering* and are reported by [`mint_elementwise`], each wrapped in
/// [`ElementwiseRefusal::Refused`] — or [`ElementwiseRefusal::Folded`] naming
/// the value a folding family produced.
fn plan_elementwise(
    program: &SemanticProgram,
    root: ValueId,
    leaves: &ElementwiseLeaves<'_>,
    shape: &Shape,
    laws: &FrozenIndexRealizationLawRegistry,
    arithmetic: ArithmeticType,
) -> Result<ElementwisePlan, ElementwiseRefusal> {
    let mut steps: Vec<(ValueId, ElementwiseMint)> = Vec::new();
    let mut minted: Vec<ValueId> = Vec::new();
    let mut members: Vec<SemanticStage> = Vec::new();
    let mut leaf_reads: Vec<LeafRead> = Vec::new();
    let mut pending = vec![(root, false)];
    while let Some((value, operands_visited)) = pending.pop() {
        if minted.contains(&value) {
            continue;
        }
        if leaves.is_leaf(value) {
            if static_shape_ref(program, value) != Some(shape) {
                return refused("elementwise-shape");
            }
            let leaf = LeafRead {
                value,
                map: LogicalAccess::LinearIdentity,
            };
            record_leaf(&mut leaf_reads, leaves.staged, leaf.clone())?;
            steps.push((value, ElementwiseMint::Read { leaf }));
            minted.push(value);
            continue;
        }
        let (member, operation) =
            producer_for_value(program, value).map_err(ElementwiseRefusal::Refused)?;
        if operation.results().collect::<Vec<_>>() != [value] {
            return refused("elementwise-result-arity");
        }
        if constant_family(arithmetic).is_some_and(|constant| operation.key() == &constant) {
            let (bits, _) =
                constant_bits(program, value, arithmetic).map_err(ElementwiseRefusal::Refused)?;
            members.push(SemanticStage::first(SemanticMemberId(member)));
            steps.push((value, ElementwiseMint::Constant(bits)));
            minted.push(value);
            continue;
        }
        // A structural occurrence contributes an *access relation*, not a node.
        // It computes nothing — the value it produces is the value it read — so
        // it becomes the leaf that reads its operand, carrying the coordinate
        // map the family denotes. That is what makes a fused region the
        // deliverable rather than a materializing copy kernel: the arithmetic
        // still comes from the neighbour, and only the addressing changes.
        if let Some((operand, map)) = recognize_structural_read(program, &operation, leaves, shape)
            .map_err(ElementwiseRefusal::Refused)?
        {
            let leaf = LeafRead {
                value: operand,
                map,
            };
            record_leaf(&mut leaf_reads, leaves.staged, leaf.clone())?;
            members.push(SemanticStage::first(SemanticMemberId(member)));
            steps.push((value, ElementwiseMint::Read { leaf }));
            minted.push(value);
            continue;
        }
        let Some(family) = elementwise_family(&operation, arithmetic) else {
            // A folding family is the *boundary* between two regions rather than
            // an unrecognizable operation: no `PointwiseF32Node` spells a sum
            // over a contributor sequence, and none ever will, because the
            // expression is a per-point body. Naming the value lets the epilogue
            // recognizer read it as the tensor an earlier region staged.
            //
            // **A walk that already reads one staged value reports the ordinary
            // rule instead, and that is a rule about chain *width* rather than
            // depth.** Naming a second, *different* folded value —
            // `sum(a, 1) * sum(b, 1)` — would give this one region two
            // `TensorRole::Intermediate` reads, and that role carries no ordinal,
            // so nothing would say which edge each access binds. The walk is
            // still one materialization boundary deep. It is the same
            // unordinalled-role fact `record_leaf` refuses for one staged value
            // read *twice*, and its region-vocabulary owner is
            // `admit-a-scheduled-region-that-reads-two-materialization-edges`.
            // The depth rule is `StagedOperandAdmission`'s, which states where it
            // sits and what it is not.
            if leaves.staged.is_none() && materializes_its_result(&operation, laws) {
                return Err(ElementwiseRefusal::Folded(value));
            }
            return refused("operation-set");
        };
        // A recognized elementwise operation of this profile is attribute-free.
        // An attribute is a semantic fact the expression does not carry forward,
        // so admitting one would silently drop it.
        if !operation.attributes().fields().is_empty() {
            return refused("elementwise-attributes");
        }
        // The region's domain, or rank zero. The second arm is not a relaxation:
        // the expression's nodes are *per-point values*, so a subexpression over
        // constants alone — which the semantic inferencer types as rank zero —
        // is evaluated exactly like a constant leaf and reads no tensor. It is
        // also reachable rather than defensive: reassociating `(a * 2.0) * 3.0`
        // into `a * (2.0 * 3.0)` is an alternative the algebraic exploration
        // proposes under a contract that permits it, and its inner product is
        // rank zero. Refusing every rank would have lost that alternative to a
        // check with no correctness content behind it.
        let value_shape = static_shape_ref(program, value);
        if value_shape != Some(shape) && value_shape.map(Shape::rank) != Some(0) {
            return refused("elementwise-shape");
        }
        let operands: Vec<ValueId> = operation.operands().collect();
        if operands.len() != family.operand_count() {
            return refused("elementwise-arity");
        }
        if !operands_visited {
            pending.push((value, true));
            // Pushed in reverse so the first operand is popped first, which is
            // what keeps a deterministic walk order across arities.
            for operand in operands.iter().rev() {
                pending.push((*operand, false));
            }
            continue;
        }
        // Every operand is already minted, so the node's own step records the
        // operand *values* and the numbering is left to the mint pass.
        if !operands.iter().all(|operand| minted.contains(operand)) {
            return refused("elementwise-operand");
        }
        members.push(SemanticStage::first(SemanticMemberId(member)));
        steps.push((value, ElementwiseMint::Node(family, operands)));
        minted.push(value);
    }
    members.sort_unstable();
    members.dedup();
    Ok(ElementwisePlan {
        steps,
        leaves: leaf_reads,
        members,
        root,
    })
}

/// One per-point expression vocabulary a planned walk can be minted into.
///
/// **One walk, one mint loop, two vocabularies.** The plan
/// [`plan_elementwise`] produces is arithmetic-neutral — it is a linearized run
/// of reads, constants, and classified families — and the only thing that
/// differs between the two widths is which builder the run is replayed against.
/// Writing the replay twice would be two accounts of one numbering, free to
/// disagree about which leaf serves which read, which is the drift a single
/// authority exists to prevent.
///
/// The error is a rule name rather than a unit, because the two vocabularies
/// refuse for different reasons and a shared `()` would report a node-count bound
/// for a family the width has no node for at all.
trait PointwiseMintSink {
    /// The sink's handle to one minted per-point value.
    type Value: Clone;
    /// The verified expression this sink builds.
    type Expression;

    /// Mints a read of the expression input at one dense leaf ordinal.
    fn input(&mut self, ordinal: InputOrdinal) -> Result<Self::Value, &'static str>;
    /// Mints an exact constant leaf from its canonical bit pattern.
    fn constant(&mut self, bits: u32) -> Result<Self::Value, &'static str>;
    /// Mints one ordered addition.
    fn add(&mut self, lhs: Self::Value, rhs: Self::Value) -> Result<Self::Value, &'static str>;
    /// Mints one ordered multiplication.
    fn multiply(&mut self, lhs: Self::Value, rhs: Self::Value)
    -> Result<Self::Value, &'static str>;
    /// Mints the sigmoid-weighted linear unit's projected body.
    fn silu(&mut self, argument: &Self::Value) -> Result<Self::Value, &'static str>;
    /// Builds the verified expression rooted at one minted value.
    fn build(self, root: Self::Value) -> Result<Self::Expression, &'static str>;
}

/// The `f32` per-point vocabulary.
struct F32Mint(PointwiseF32ExpressionBuilder);

impl PointwiseMintSink for F32Mint {
    type Value = PointwiseF32Value;
    type Expression = PointwiseF32Expression;

    fn input(&mut self, ordinal: InputOrdinal) -> Result<Self::Value, &'static str> {
        self.0.input(ordinal).map_err(|_| "elementwise-node-limit")
    }

    fn constant(&mut self, bits: u32) -> Result<Self::Value, &'static str> {
        self.0.constant(bits).map_err(|_| "elementwise-node-limit")
    }

    fn add(&mut self, lhs: Self::Value, rhs: Self::Value) -> Result<Self::Value, &'static str> {
        self.0.add(lhs, rhs).map_err(|_| "elementwise-node-limit")
    }

    fn multiply(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
    ) -> Result<Self::Value, &'static str> {
        self.0
            .multiply(lhs, rhs)
            .map_err(|_| "elementwise-node-limit")
    }

    // The composition is emitted by the shared authority rather than spelled
    // here; see [`ElementwiseFamily::Silu`].
    fn silu(&mut self, argument: &Self::Value) -> Result<Self::Value, &'static str> {
        let mut sink = PointwiseExpressionSink::new(&mut self.0);
        silu_point_body(&mut sink, argument).map_err(|_| "elementwise-node-limit")
    }

    fn build(self, root: Self::Value) -> Result<Self::Expression, &'static str> {
        self.0.build(root).map_err(|_| "elementwise-expression")
    }
}

/// The `bf16` per-point vocabulary.
struct Bf16Mint(PointwiseBf16ExpressionBuilder);

impl PointwiseMintSink for Bf16Mint {
    type Value = PointwiseBf16Value;
    type Expression = PointwiseBf16Expression;

    fn input(&mut self, ordinal: InputOrdinal) -> Result<Self::Value, &'static str> {
        self.0.input(ordinal).map_err(|_| "elementwise-node-limit")
    }

    /// The payload is narrowed rather than truncated.
    ///
    /// [`constant_bits`] reads a `bf16` constant's exactly two declared payload
    /// bytes, so every value reaching here fits; a wider one is a mismatch
    /// between the two and is refused by name instead of silently losing the
    /// upper half of a pattern that would then be a different number.
    fn constant(&mut self, bits: u32) -> Result<Self::Value, &'static str> {
        let bits = u16::try_from(bits).map_err(|_| "constant-bits")?;
        self.0.constant(bits).map_err(|_| "elementwise-node-limit")
    }

    fn add(&mut self, lhs: Self::Value, rhs: Self::Value) -> Result<Self::Value, &'static str> {
        self.0.add(lhs, rhs).map_err(|_| "elementwise-node-limit")
    }

    fn multiply(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
    ) -> Result<Self::Value, &'static str> {
        self.0
            .multiply(lhs, rhs)
            .map_err(|_| "elementwise-node-limit")
    }

    /// Unreachable, and refused by its own name rather than by a bound it did
    /// not exceed: [`elementwise_family`] classifies no activation for this
    /// width, because no `bf16` activation family is registered and the `bf16`
    /// node vocabulary has neither the division nor the exponential its body
    /// composes.
    fn silu(&mut self, _argument: &Self::Value) -> Result<Self::Value, &'static str> {
        Err("elementwise-family-arithmetic")
    }

    fn build(self, root: Self::Value) -> Result<Self::Expression, &'static str> {
        self.0.build(root).map_err(|_| "elementwise-expression")
    }
}

/// Replays one planned walk into one per-point vocabulary.
///
/// `order` is the read list the region will bind, in access order: leaf `i` of
/// the built expression is served by read `i`, which is the correspondence
/// `emit_pointwise` relies on and the one
/// `tiler_ir::schedule::reads_bind_boundary_tensors_in_order` states the
/// boundary-role rules against.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `elementwise-reads` for
/// a leaf the order does not name, `input-ordinal` for a position no expression
/// ordinal can hold, `elementwise-operand` for an operand no earlier step of the
/// plan minted, `elementwise-arity` for a family and operand count this
/// projection has no node for, and every rule the sink itself reports —
/// `elementwise-node-limit` for an expression exceeding its vocabulary's node
/// bound, `elementwise-expression` for an assembled expression no region can
/// bind, and the sink's own refusal for a family its width cannot state.
fn mint_into<S: PointwiseMintSink>(
    plan: &ElementwisePlan,
    order: &[LeafRead],
    mut sink: S,
) -> Result<S::Expression, RequestError> {
    let mut minted: Vec<(ValueId, S::Value)> = Vec::new();
    for (value, mint) in &plan.steps {
        let node = match mint {
            ElementwiseMint::Read { leaf } => {
                let position = order.iter().position(|named| named == leaf).ok_or(
                    RequestError::UnsupportedCapability {
                        phase: "strategy",
                        rule: "elementwise-reads",
                    },
                )?;
                let ordinal =
                    u32::try_from(position).map_err(|_| RequestError::UnsupportedCapability {
                        phase: "strategy",
                        rule: "input-ordinal",
                    })?;
                sink.input(InputOrdinal::new(ordinal))
            }
            ElementwiseMint::Constant(bits) => sink.constant(*bits),
            ElementwiseMint::Node(family, operands) => {
                let projected: Vec<S::Value> = operands
                    .iter()
                    .map(|operand| minted_value(&minted, *operand))
                    .collect::<Result<_, _>>()?;
                match (family, projected.as_slice()) {
                    (ElementwiseFamily::Add, [lhs, rhs]) => sink.add(lhs.clone(), rhs.clone()),
                    (ElementwiseFamily::Multiply, [lhs, rhs]) => {
                        sink.multiply(lhs.clone(), rhs.clone())
                    }
                    (ElementwiseFamily::Silu, [argument]) => sink.silu(argument),
                    // Unreachable through the planner's arity check, and refused
                    // rather than assumed away: an arity this projection has no
                    // case for is a vocabulary gap, not a node to invent.
                    _ => return mismatch("elementwise-arity"),
                }
            }
        }
        .map_err(mismatch_rule)?;
        minted.push((*value, node));
    }
    let root = minted_value(&minted, plan.root)?;
    sink.build(root).map_err(mismatch_rule)
}

/// Mints one planned elementwise expression in the program's own arithmetic.
///
/// # Errors
///
/// Returns every rule [`mint_into`] reports, and `dtype-recognized` for an
/// arithmetic type this recognizer states no per-point vocabulary for — the same
/// rule [`recognized_program_arithmetic`] refuses that width under, because it is
/// the same finding reached from the other end.
fn mint_elementwise(
    plan: &ElementwisePlan,
    order: &[LeafRead],
    arithmetic: ArithmeticType,
) -> Result<RecognizedPointwise, RequestError> {
    match arithmetic {
        ArithmeticType::F32 => {
            mint_into(plan, order, F32Mint(PointwiseF32ExpressionBuilder::new()))
                .map(RecognizedPointwise::F32)
        }
        ArithmeticType::Bf16 => {
            mint_into(plan, order, Bf16Mint(PointwiseBf16ExpressionBuilder::new()))
                .map(RecognizedPointwise::Bf16)
        }
        ArithmeticType::F16 | ArithmeticType::F64 => mismatch("dtype-recognized"),
    }
}

/// Mints one planned expression into the `f32` vocabulary specifically.
///
/// The fold's prologue and the elementwise epilogue call this rather than
/// [`mint_elementwise`], because both shapes are reachable only for an `f32`
/// program: each is entered from a folding family, and the three families that
/// discover one — the strict serial sum, the strict tensor contraction, and any
/// registered family whose realization law spans a region sequence — are `f32`
/// throughout. Asking for the `f32` vocabulary directly is what keeps
/// [`NormalizedSerialSum::prologue`] and [`NormalizedEpilogue::expression`] typed
/// as the one vocabulary they can hold, instead of carrying a width neither
/// shape can reach.
fn mint_elementwise_f32(
    plan: &ElementwisePlan,
    order: &[LeafRead],
) -> Result<PointwiseF32Expression, RequestError> {
    mint_into(plan, order, F32Mint(PointwiseF32ExpressionBuilder::new()))
}

/// Wraps one sink's rule name in the recognizer's typed refusal.
const fn mismatch_rule(rule: &'static str) -> RequestError {
    RequestError::UnsupportedCapability {
        phase: "strategy",
        rule,
    }
}

/// Recognizes one structural occurrence as a mapped read of a leaf tensor.
///
/// Returns the leaf value and the access relation the occurrence denotes, or
/// `None` when the operation is not a structural family at all — which is the
/// caller's signal to try the elementwise projection instead. An operation that
/// *is* structural but cannot be admitted returns a typed refusal rather than
/// `None`, so a reindex this profile cannot bind never falls through to be
/// reported as an unrecognized operation set.
///
/// **The operand must already be a value this walk reads rather than computes.**
/// A direct mapped-only occurrence over a value another region would materialize
/// does not discover that boundary: on the first walk the producer result is not
/// yet a staged leaf, so it refuses under `structural-operand`. If another dense
/// occurrence first discovers the boundary, replay does make the producer result
/// a staged leaf and this function recognizes the mapped read, but
/// [`record_leaf`] then refuses it as a second read of the unordinalled
/// [`TensorRole::Intermediate`] under `structural-access-conflict`. Thus neither
/// path currently admits a structural read of a staged operand; materializing a
/// same-region computed value would additionally introduce an observable rounding
/// boundary the structural family's admission deliberately excludes.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the property that was
/// not recognized: `structural-arity`, `structural-operand` for an operand this
/// walk does not read, `structural-attributes` for a malformed or missing
/// form record, `structural-shape` for a result at another domain, and
/// `structural-relation` when the derived map is not one the region vocabulary
/// admits.
fn recognize_structural_read(
    program: &SemanticProgram,
    operation: &tiler_ir::semantic::OperationRef<'_>,
    leaves: &ElementwiseLeaves<'_>,
    shape: &Shape,
) -> Result<Option<(ValueId, LogicalAccess)>, RequestError> {
    let reindex = operation.key() == &reindex_f32_op();
    if !reindex && operation.key() != &broadcast_f32_op() {
        return Ok(None);
    }
    let operands: Vec<ValueId> = operation.operands().collect();
    let [operand] = operands.as_slice() else {
        return mismatch("structural-arity");
    };
    if !leaves.is_leaf(*operand) {
        return mismatch("structural-operand");
    }
    let Some(operand_shape) = static_shape_ref(program, *operand) else {
        return mismatch("structural-operand");
    };
    // The occurrence's result is what the region iterates, so a result at any
    // other domain would make every derived divisor address the wrong window.
    let results: Vec<ValueId> = operation.results().collect();
    let [result] = results.as_slice() else {
        return mismatch("structural-arity");
    };
    if static_shape_ref(program, *result) != Some(shape) {
        return mismatch("structural-shape");
    }
    let operand_shape = operand_shape.clone();
    let map = if reindex {
        let Some(value) = operation.attributes().get(REINDEX_MAPPING_ATTRIBUTE) else {
            return mismatch("structural-attributes");
        };
        let Ok(form) = ReindexForm::from_canonical_value(value) else {
            return mismatch("structural-attributes");
        };
        // Re-derived rather than trusted: the form must produce exactly this
        // result from exactly this operand, or the region would realize a
        // different occurrence than the one requested — the same check the
        // governed index-access lowering makes for the same reason.
        if form.result_shape(&operand_shape).ok().as_ref() != Some(shape) {
            return mismatch("structural-shape");
        }
        let Some(axes) = reindex_axis_decodes(&form, &operand_shape, shape) else {
            return mismatch("structural-relation");
        };
        LogicalAccess::ReindexBijection {
            operand_shape,
            result_shape: shape.clone(),
            axes,
        }
    } else {
        let Some(value) = operation.attributes().get(BROADCAST_AXIS_MAPPING_ATTRIBUTE) else {
            return mismatch("structural-attributes");
        };
        let Ok(mapping) = BroadcastAxisMapping::from_canonical_value(value) else {
            return mismatch("structural-attributes");
        };
        if mapping.result_shape(&operand_shape).ok().as_ref() != Some(shape) {
            return mismatch("structural-shape");
        }
        let Some(axes) = broadcast_axis_decodes(&mapping, &operand_shape, shape) else {
            return mismatch("structural-relation");
        };
        LogicalAccess::BroadcastReplication {
            operand_shape,
            result_shape: shape.clone(),
            axes,
        }
    };
    // The region verifier will refuse a map that fails its admission rule, but
    // refusing here reports the *program* property rather than letting a region
    // be assembled that cannot be built. A broadcast that widens nothing lands
    // here, which is the one case a well-formed semantic mapping can reach.
    let admissible = match &map {
        LogicalAccess::ReindexBijection {
            operand_shape,
            result_shape,
            axes,
        } => tiler_ir::schedule::reindex_decodes_are_bijective(operand_shape, result_shape, axes),
        LogicalAccess::BroadcastReplication {
            operand_shape,
            result_shape,
            axes,
        } => {
            tiler_ir::schedule::broadcast_decodes_are_replicating(operand_shape, result_shape, axes)
        }
        _ => false,
    };
    if !admissible {
        return mismatch("structural-relation");
    }
    Ok(Some((*operand, map)))
}

/// Returns the row-major suffix products of `shape`, one per axis.
///
/// Entry `k` is the product of every extent after axis `k`, which is the divisor
/// that extracts axis `k`'s coordinate from a row-major linear index. `None` on
/// overflow, so a derived divisor is never a wrapped one.
fn shape_suffix_products(shape: &Shape) -> Option<Vec<u64>> {
    let extents = shape.extents();
    let mut products = vec![1_u64; extents.len()];
    let mut running = 1_u64;
    for (position, extent) in extents.iter().enumerate().rev() {
        products[position] = running;
        running = running.checked_mul(extent.get())?;
    }
    Some(products)
}

/// Builds one operand axis's decode, canonicalizing an extent-one axis.
///
/// An extent-one axis has exactly one coordinate, so its divisor and mirroring
/// are unobservable — and [`AxisDecode::is_canonical`] requires them to be the
/// canonical pair, because admitting any other spelling would give one access
/// relation many identities. Routing every construction through here is what
/// makes that a property of the derivation rather than a rule each form has to
/// remember.
fn axis_decode(divisor: u64, extent: u64, mirrored: bool) -> AxisDecode {
    if extent == 1 {
        return AxisDecode::fixed();
    }
    AxisDecode {
        divisor,
        modulus: extent,
        mirrored,
    }
}

/// Derives the per-operand-axis decodes one reindex form realizes.
///
/// The physical restatement of the same coordinate relation
/// `reindex_operand_coordinates` emits into the index vocabulary, and it is a
/// restatement rather than a second derivation for a reason worth stating: the
/// index-region half is what occurrence refinement proves realizes the
/// occurrence, and this half is what the region's identity and its kernel offset
/// are built from. They are checked against each other by the compiled result
/// being bit-compared with the reference evaluator, which is the only place the
/// two can disagree and be caught.
///
/// Every form reduces to one decode per operand axis. Returns `None` when the
/// form and the shapes disagree, which the caller turns into a typed refusal
/// rather than a nearest-fit map.
fn reindex_axis_decodes(
    form: &ReindexForm,
    operand: &Shape,
    result: &Shape,
) -> Option<Vec<AxisDecode>> {
    use std::cmp::Ordering;

    let suffix = shape_suffix_products(result)?;
    let extents: Vec<u64> = operand
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    let rank = extents.len();
    let position_of = |axis: &Axis| {
        usize::try_from(axis.get())
            .ok()
            .filter(|index| *index < rank)
    };
    let mut decodes = Vec::with_capacity(rank);
    match form.kind() {
        // Result axis `k` reads operand axis `order[k]`, so operand axis
        // `order[k]` takes the window of result axis `k`. Written as a scatter
        // for the reason the index-region half is: that is the direction the
        // attribute states.
        ReindexFormKind::PermuteAxes => {
            let order = form.axes();
            if order.len() != rank {
                return None;
            }
            let mut slots: Vec<Option<AxisDecode>> = vec![None; rank];
            for (position, axis) in order.iter().enumerate() {
                let index = position_of(axis)?;
                let decode = axis_decode(*suffix.get(position)?, extents[index], false);
                if slots.get_mut(index)?.replace(decode).is_some() {
                    return None;
                }
            }
            slots.into_iter().collect()
        }
        // The split's result axes are a *contiguous* run, and contiguous
        // row-major axes linearize as one window: the run's combined coordinate
        // is the window ending at its last axis, whose extent is the operand
        // axis's own. That is why a split needs no multi-term sum here.
        ReindexFormKind::SplitAxis => {
            let axis = position_of(form.axes().first()?)?;
            let factors = form.factors().len();
            let last = axis.checked_add(factors)?.checked_sub(1)?;
            for (position, extent) in extents.iter().enumerate() {
                let divisor = match position.cmp(&axis) {
                    Ordering::Less => *suffix.get(position)?,
                    Ordering::Equal => *suffix.get(last)?,
                    Ordering::Greater => {
                        *suffix.get(position.checked_add(factors)?.checked_sub(1)?)?
                    }
                };
                decodes.push(axis_decode(divisor, *extent, false));
            }
            Some(decodes)
        }
        // The merge decodes one result coordinate back into the merged run. The
        // two-level decode collapses into one window per operand axis: the outer
        // wrap is redundant because the merged result axis's extent is the
        // product of the run, so the part the outer wrap would discard is
        // already a multiple of each inner modulus.
        ReindexFormKind::MergeAxes => {
            let merged = form.axes();
            let first = position_of(merged.first()?)?;
            let count = merged.len();
            let base = *suffix.get(first)?;
            let mut inner = vec![1_u64; count];
            let mut running = 1_u64;
            for offset in (0..count).rev() {
                inner[offset] = running;
                running = running.checked_mul(*extents.get(first.checked_add(offset)?)?)?;
            }
            for (position, extent) in extents.iter().enumerate() {
                let divisor = if position < first {
                    *suffix.get(position)?
                } else if position < first.checked_add(count)? {
                    base.checked_mul(inner[position.checked_sub(first)?])?
                } else {
                    *suffix.get(position.checked_sub(count)?.checked_add(1)?)?
                };
                decodes.push(axis_decode(divisor, *extent, false));
            }
            Some(decodes)
        }
        // The inserted result axis has extent one and no operand axis behind it,
        // so every operand axis reads the result axis one position later from
        // the insertion point onward.
        ReindexFormKind::InsertUnitAxis => {
            let inserted = usize::try_from(form.axes().first()?.get()).ok()?;
            for (position, extent) in extents.iter().enumerate() {
                let source = if position < inserted {
                    position
                } else {
                    position.checked_add(1)?
                };
                decodes.push(axis_decode(*suffix.get(source)?, *extent, false));
            }
            Some(decodes)
        }
        // The removed operand axis has extent one, so its only coordinate is
        // zero and it reads no result axis at all.
        ReindexFormKind::RemoveUnitAxis => {
            let removed = position_of(form.axes().first()?)?;
            for (position, extent) in extents.iter().enumerate() {
                let decode = match position.cmp(&removed) {
                    Ordering::Equal => AxisDecode::fixed(),
                    Ordering::Less => axis_decode(*suffix.get(position)?, *extent, false),
                    Ordering::Greater => {
                        axis_decode(*suffix.get(position.checked_sub(1)?)?, *extent, false)
                    }
                };
                decodes.push(decode);
            }
            Some(decodes)
        }
        // The shape is preserved, so every operand axis reads its own result
        // axis; the reversed one reads it mirrored. This is the one form the
        // mirror flag exists for.
        ReindexFormKind::ReverseAxis => {
            let reversed = position_of(form.axes().first()?)?;
            for (position, extent) in extents.iter().enumerate() {
                decodes.push(axis_decode(
                    *suffix.get(position)?,
                    *extent,
                    position == reversed,
                ));
            }
            Some(decodes)
        }
    }
}

/// Derives the per-operand-axis decodes one broadcast mapping realizes.
///
/// Each entry of the mapping names a *result* axis. A `FromOperand` entry gives
/// its operand axis that result axis's window; a `StretchUnit` entry names an
/// extent-one operand axis, whose decode is the canonical fixed one; and a
/// `Replicate` entry names no operand axis, which is exactly what leaves the
/// read invariant in that result axis.
fn broadcast_axis_decodes(
    mapping: &BroadcastAxisMapping,
    operand: &Shape,
    result: &Shape,
) -> Option<Vec<AxisDecode>> {
    let suffix = shape_suffix_products(result)?;
    let extents: Vec<u64> = operand
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    let sources = mapping.sources();
    if sources.len() != result.rank() {
        return None;
    }
    let mut slots: Vec<Option<AxisDecode>> = vec![None; extents.len()];
    for (position, source) in sources.iter().enumerate() {
        let Some(axis) = source.operand_axis() else {
            continue;
        };
        let index = usize::try_from(axis.get())
            .ok()
            .filter(|index| *index < extents.len())?;
        let decode = axis_decode(*suffix.get(position)?, extents[index], false);
        if slots.get_mut(index)?.replace(decode).is_some() {
            return None;
        }
    }
    slots.into_iter().collect()
}

/// Returns the expression node already minted for one recognized value.
///
/// Generic over the sink's handle rather than over one vocabulary's value type,
/// because the lookup is by planned `ValueId` and says nothing about what the
/// handle denotes; a second copy per width would be one rule stated twice.
fn minted_value<V: Clone>(minted: &[(ValueId, V)], value: ValueId) -> Result<V, RequestError> {
    minted
        .iter()
        .find(|(seen, _)| *seen == value)
        .map(|(_, node)| node.clone())
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "elementwise-operand",
        })
}

/// Recognizes a whole-program elementwise expression.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the unrecognized
/// property — every rule [`resolve_elementwise`] reports, the planning half
/// having already run in the caller — or [`RequestError::ShapeProductOverflow`]
/// under `input` for a domain whose extents do not multiply into a `u64`. The
/// rank and occurrence-coverage obligations are the caller's:
/// [`recognize_elementwise_output`] reports `elementwise-rank` and
/// [`check_output_cover`] reports `operation-set`.
fn recognize_pointwise(
    program: &SemanticProgram,
    output: &tiler_ir::semantic::ProgramOutputRef<'_>,
    declared: &[ValueId],
    shape: Shape,
    plan: ElementwisePlan,
    arithmetic: ArithmeticType,
) -> Result<NormalizedPointwise, RequestError> {
    let recognized = resolve_elementwise(plan, declared, arithmetic)?;
    let elements = element_count_u64(&shape, "input")?;
    Ok(NormalizedPointwise {
        input_keys: program.inputs().map(|input| input.key().clone()).collect(),
        output_key: output.key().clone(),
        shape,
        expression: recognized.expression,
        members: recognized.members,
        inputs: declared.to_vec(),
        output: output.value(),
        elements,
        reads: recognized.reads,
    })
}

/// Recognizes an elementwise epilogue over one staged producer result.
///
/// **The read order is canonical rather than the order the walk minted leaves
/// in, and that is a correctness requirement rather than tidiness.**
/// `tiler_ir::schedule`'s pointwise access contract requires a region's declared
/// input ordinals not to descend across its read list, so a read list in walk
/// order would make `staged * (b + a)` admissible and `staged * (a + b)` not —
/// the same computation refused for the order its operands happened to be popped
/// in. The staged read leads because exactly one read binds it and it carries no
/// ordinal to interleave with; the declared inputs follow in declaration order,
/// each input's own reads in the dense-first order that contract states.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the property that was
/// not recognized: every rule [`plan_elementwise`], [`mint_elementwise`], and
/// [`declared_ordinal`] report for the epilogue's own walk, and every rule
/// [`recognize_epilogue_producer`] reports for the staged half — `operation-set`
/// for a folded family it has no producer recognizer for,
/// [`producer_for_value`]'s `missing-producer` and `operation-ordinal`, and the
/// producing family's own rules. Returns [`RequestError::ShapeProductOverflow`]
/// under `output` for a domain whose extents do not multiply into a `u64`.
fn recognize_epilogue(
    program: &SemanticProgram,
    output: &tiler_ir::semantic::ProgramOutputRef<'_>,
    declared: &[ValueId],
    shape: Shape,
    staged: ValueId,
    laws: &FrozenIndexRealizationLawRegistry,
    arithmetic: ArithmeticType,
) -> Result<NormalizedEpilogue, RequestError> {
    let leaves = ElementwiseLeaves {
        declared,
        staged: Some(staged),
    };
    let plan = plan_elementwise(program, output.value(), &leaves, &shape, laws, arithmetic)
        .map_err(RequestError::from)?;
    // The staged read, then whichever declared inputs the expression names. The
    // *declared* half is now the same rule `canonical_input_reads` states —
    // groups in declaration order, dense before mapped, an unread input
    // contributing nothing — and what keeps the walk spelled here is the staged
    // read: it leads, it binds no declared input, and that function refuses a
    // leaf which is not one. Reusing it would mean handing it a leaf set it is
    // defined to reject.
    let mut order: Vec<LeafRead> = plan
        .leaves
        .iter()
        .filter(|leaf| leaf.value == staged)
        .cloned()
        .collect();
    for input in declared {
        for dense in [true, false] {
            order.extend(
                plan.leaves
                    .iter()
                    .filter(|leaf| {
                        leaf.value == *input && (leaf.map == LogicalAccess::LinearIdentity) == dense
                    })
                    .cloned(),
            );
        }
    }
    let expression = mint_elementwise_f32(&plan, &order)?;
    let reads = order
        .iter()
        .map(|leaf| {
            let read = if leaf.value == staged {
                BoundaryRead::Staged
            } else {
                BoundaryRead::Input(declared_ordinal(declared, leaf.value)?)
            };
            Ok((read, leaf.map.clone()))
        })
        .collect::<Result<Vec<_>, RequestError>>()?;
    let producer = recognize_epilogue_producer(program, staged, output.key().clone(), laws)?;
    let elements = element_count_u64(&shape, "output")?;
    Ok(NormalizedEpilogue {
        producer: Box::new(producer),
        input_keys: program.inputs().map(|input| input.key().clone()).collect(),
        output_key: output.key().clone(),
        shape,
        expression,
        reads,
        members: plan.members,
        inputs: declared.to_vec(),
        output: output.value(),
        elements,
    })
}

/// Returns whether one occurrence's result is a value some region *materializes*
/// rather than a value a consumer's own per-point body can recompute.
///
/// **The single statement of where a materialization edge may sit**, asked by
/// [`plan_elementwise`]'s folding discovery and by
/// [`recognize_staged_family`]'s operand walk. The two used to be one disjunct
/// written once; they are one function now because a second copy would be free
/// to disagree about which programs contain an edge at all, and the shape of
/// that disagreement is a walk naming a boundary the producer recognizer refuses
/// to build for.
///
/// A recognized *elementwise* family is deliberately not here. Its result is an
/// expression its consumer evaluates per point, which is the whole reason the
/// expression vocabulary exists; treating it as an edge would materialize a
/// value — and add the observable rounding boundary — the caller's program never
/// asked for.
fn materializes_its_result(
    operation: &tiler_ir::semantic::OperationRef<'_>,
    laws: &FrozenIndexRealizationLawRegistry,
) -> bool {
    operation.key() == &strict_serial_sum_f32_op()
        || operation.key() == &strict_tensor_contraction_f32_op()
        || laws.family_realizes_region_sequence(operation.key())
}

/// Recognizes the shape producing one materialized value.
///
/// The folding families and nothing else, which is exactly
/// [`materializes_its_result`]'s set. The refusal is not dead code standing in
/// for an impossible state: both callers gate on that predicate, and a family
/// added to it without a producer region here must refuse rather than acquire
/// one.
///
/// **The producer is at the far side of an edge, so it places none of its own**
/// — [`StagedOperandAdmission::NoEdge`] below. This is the only site that hands
/// that value, and of the three arms only the staged one can place an edge at
/// all, so the whole depth rule is reachable from here.
/// [`StagedOperandAdmission`] is where it is stated, including the measured
/// reason it stays and the two neighbouring folded-value walls it is not.
fn recognize_epilogue_producer(
    program: &SemanticProgram,
    staged: ValueId,
    output_key: OutputKey,
    laws: &FrozenIndexRealizationLawRegistry,
) -> Result<NormalizedOutput, RequestError> {
    let (member, root) = producer_for_value(program, staged)?;
    if root.key() == &strict_serial_sum_f32_op() {
        recognize_reduction(program, staged, output_key, member, &root, laws)
            .map(NormalizedOutput::SerialSum)
    } else if root.key() == &strict_tensor_contraction_f32_op() {
        normalize_contraction(program, staged, output_key)
            .map(|normalized| NormalizedOutput::Contraction(Box::new(normalized)))
    } else if laws.family_realizes_region_sequence(root.key()) {
        recognize_staged_family(
            program,
            laws,
            staged,
            output_key,
            member,
            &root,
            StagedOperandAdmission::NoEdge,
        )
        .map(|normalized| NormalizedOutput::Staged(Box::new(normalized)))
    } else {
        mismatch("operation-set")
    }
}

/// Whether one staged occurrence may place a materialization edge of its own.
///
/// **This is the single statement of the recognized chain's depth rule**, and
/// every other site that mentions depth points here rather than restating it.
///
/// **A depth counter would be the wrong shape.** What bounds the recognized
/// chain is not a number of levels but a rule about *sides*: a recognized shape
/// admits at most one edge of its own, and a shape reached across an edge admits
/// none. Two variants say exactly that, and a reader can refute the rule by
/// checking the two call sites rather than by reasoning about arithmetic.
///
/// # The rule has one guard, and two neighbours that are not it
///
/// The `NoEdge` arm of [`recognize_staged_family`]'s operand walk —
/// `staged-operand-depth` — is the whole of it. [`recognize_epilogue_producer`]
/// is the one function reached across an edge and the only site that ever passes
/// `NoEdge`, and of the three shapes it recognizes only the staged one can place
/// an edge at all: [`normalize_contraction`] refuses a non-declared operand under
/// `contraction-operands`, and [`recognize_reduction`]'s contributor walk reads
/// declared inputs by construction.
///
/// Two neighbouring refusals also fire on a folded value and state *different*
/// rules. Reading either as this one is what would make a widener delete the
/// wrong guard, so each is named with the shape that separates it:
///
/// - [`plan_elementwise`]'s `leaves.staged.is_none()` guard refuses one walk
///   that reaches a *second, different* folded value — `sum(a, 1) * sum(b, 1)`.
///   That is one region reading two materialization edges, which is a rule about
///   chain *width*: the walk is still one boundary deep, and what it lacks is the
///   ordinal [`TensorRole::Intermediate`] does not carry.
///   [`admit-a-scheduled-region-that-reads-two-materialization-edges`](../../../tickets/admit-a-scheduled-region-that-reads-two-materialization-edges.md)
///   owns the region vocabulary and
///   [`admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region`](../../../tickets/admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region.md)
///   owns the one-value-twice spelling of it.
/// - `From<ElementwiseRefusal>`'s flattening refuses a fold whose *prologue*
///   reaches a folded value — `sum(sum(x) * 2.0)`. That one is about depth, but
///   the wall is structural rather than this guard's: [`NormalizedSerialSum`]
///   carries no producer field for the boundary to hang on, so the discovery is
///   discarded before any admission is consulted.
///   [`name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set`](../../../tickets/name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set.md)
///   owns its `reduction-contributor-materialization` rule.
///
/// # Why `NoEdge` stays, measured rather than argued
///
/// Widening is
/// [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`](../../../tickets/admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md)'s,
/// and it measured that the widening buys no program today. Every program this
/// guard refuses contains a staged occurrence whose operand is an edge, and
/// [`crate::physical::staged_plan`] has no region for one: its only law arm
/// destructures two [`BoundaryRead::Input`] operands, so such an occurrence is
/// [`crate::physical::RegionVocabularyWall::StagedFamilyUnspellable`] however
/// deep the chain around it is. Handing `OneEdge` here therefore recognizes the
/// chain — the nested shape is well formed, and only the assertion of the
/// refusal itself moves — and then refuses it as a target rejection instead of a
/// named program property. `crates/tiler-compiler/tests/recognized_chain_depth_boundary.rs`
/// holds the measurement and the trigger that reopens it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StagedOperandAdmission {
    /// One operand may be a value another region materializes.
    OneEdge,
    /// Every operand must be a declared input, because this occurrence is
    /// already at the far side of an edge.
    NoEdge,
}

/// Recognizes one occurrence of a registered family realized as a region
/// sequence.
///
/// **The admitting fact is the registered law, and no operation key appears
/// here.** The caller has already asked
/// [`FrozenIndexRealizationLawRegistry::family_realizes_region_sequence`], so
/// every family the law authority carries a multi-region law for reaches this
/// function and a family registered tomorrow reaches it unchanged. What this
/// function decides is whether *this occurrence* of such a family is one the
/// boundary can describe, and it names each property it cannot.
///
/// **What it does not do is describe the realization.** The stage count, each
/// stage's reads, and the handed values are the law's, read by
/// [`crate::region::RegionGraph::with_realizations`] when it enumerates one
/// region candidate per stage. Re-deriving them here would put a second account
/// of one law in the boundary, which is the drift
/// [`recognize_elementwise_output`]'s own doc argues against for the same
/// reason.
///
/// **One operand may be a value another region materializes, and that is this
/// function's own admission rather than a later stage's derivation.** The
/// recognized shape carries a [`BoundaryRead`] per operand and the producer's
/// recognized shape beside them, so `rms_norm(matmul(a, b), w)` is a partition
/// this output owns end to end — the producer's occurrence included, which is
/// what [`check_output_cover`] requires and what makes the *producing* region
/// spellable from a shape this partition holds. The alternative considered and
/// rejected was deriving each operand's source from the cover's materialization
/// edges: it keeps one authority for the stage split and moves a recognition-time
/// property to a stage that can only report it as a cover it could not assemble.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the exact property
/// that was not recognized:
///
/// - `staged-result-arity` for an occurrence whose results are not exactly the
///   recognized value. A staged realization's final stage writes the
///   occurrence's results and every earlier stage publishes one handed value, so
///   a second result would need a second write this boundary cannot attribute.
/// - `staged-operand` for an operand that is neither a declared program input
///   nor a value some region materializes — an elementwise expression feeding
///   the family directly, whose result [`materializes_its_result`] says is no
///   materialization edge at all. Admitting it here would be a second account of
///   where an edge may sit, disagreeing with the one
///   [`plan_elementwise`]'s folding discovery reads.
/// - `staged-operand-conflict` for a second operand supplied by a
///   materialization edge, whether that is one staged value read twice or two
///   different ones. [`TensorRole::Intermediate`] carries no ordinal, so nothing
///   says which edge each read binds; it is the same unattributable pair
///   [`record_leaf`] refuses for an epilogue's leaves.
/// - `staged-operand-depth` for a staged operand of an occurrence that is
///   *itself* at the far side of an edge. That is a recognized chain more than
///   one materialization boundary deep, and it is the one guard the depth rule
///   has; [`StagedOperandAdmission`] states the rule, the measured reason it
///   stays, and the neighbouring refusals that are about chain width and about a
///   fold's chained prologue instead.
/// - `staged-attributes` for an attribute record the canonical encoder cannot
///   write. The record is part of the occurrence's meaning, so a subject that
///   could not carry it whole must refuse rather than bind a partial one.
/// - `input-handle`/`output-handle` for a value the program holds no shape for,
///   and every rule [`declared_ordinal`] and [`recognize_epilogue_producer`]
///   report.
///
/// Returns [`RequestError::ShapeProductOverflow`] for a domain whose extents do
/// not multiply into a `u64`.
fn recognize_staged_family(
    program: &SemanticProgram,
    laws: &FrozenIndexRealizationLawRegistry,
    result: ValueId,
    output_key: OutputKey,
    member: u32,
    operation: &tiler_ir::semantic::OperationRef<'_>,
    admission: StagedOperandAdmission,
) -> Result<NormalizedStaged, RequestError> {
    if operation.results().collect::<Vec<_>>() != [result] {
        return mismatch("staged-result-arity");
    }
    let declared: Vec<ValueId> = program.inputs().map(|input| input.value()).collect();
    let operands: Vec<ValueId> = operation.operands().collect();
    let mut operand_reads = Vec::with_capacity(operands.len());
    let mut operand_shapes = Vec::with_capacity(operands.len());
    let mut operand_elements = Vec::with_capacity(operands.len());
    let mut producer = None;
    for operand in &operands {
        let read = if declared.contains(operand) {
            BoundaryRead::Input(declared_ordinal(&declared, *operand)?)
        } else {
            // The operand is computed. Whether that makes it a *materialization
            // edge* is `materializes_its_result`'s answer and not this walk's,
            // which is what keeps one statement of where an edge may sit; the
            // producer's own recognition is `recognize_epilogue_producer`'s, so
            // this arm decides only that there is one edge and that this
            // occurrence is allowed to place it.
            let (_, root) = producer_for_value(program, *operand)?;
            if !materializes_its_result(&root, laws) {
                return mismatch("staged-operand");
            }
            if producer.is_some() {
                return mismatch("staged-operand-conflict");
            }
            if admission == StagedOperandAdmission::NoEdge {
                return mismatch("staged-operand-depth");
            }
            producer = Some(Box::new(recognize_epilogue_producer(
                program,
                *operand,
                output_key.clone(),
                laws,
            )?));
            BoundaryRead::Staged
        };
        operand_reads.push(read);
        let shape = static_shape(program, *operand, "input-handle")?;
        operand_elements.push(element_count_u64(&shape, "input")?);
        operand_shapes.push(shape);
    }
    let output_shape = static_shape(program, result, "output-handle")?;
    let output_elements = element_count_u64(&output_shape, "output")?;
    let mut attributes = Vec::new();
    crate::region::encode_attributes(&mut attributes, operation.attributes()).map_err(|_| {
        RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "staged-attributes",
        }
    })?;
    // The same registry row the caller's admission read. `None` is unreachable
    // through that admission — a family with no law realizes no region sequence —
    // and is refused by name rather than unwrapped, because this function is the
    // one that would otherwise carry an invented law into every later stage.
    let law = laws
        .family_realization_law(operation.key())
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "staged-law",
        })?
        .clone();
    Ok(NormalizedStaged {
        producer,
        operation: operation.key().clone(),
        law,
        attribute_record: operation.attributes().clone(),
        attributes: attributes.into_boxed_slice(),
        input_keys: program.inputs().map(|input| input.key().clone()).collect(),
        output_key,
        operand_reads,
        operand_shapes,
        output_shape,
        member: SemanticMemberId(member),
        inputs: declared,
        output: result,
        operand_elements,
        output_elements,
    })
}

/// Recognizes a strict serial reduction and whatever elementwise expression
/// feeds it.
///
/// The prologue is recognized by the same general walk a whole-program
/// elementwise expression is, so what composes with the reduction is bounded by
/// the expression vocabulary rather than by the scale-then-bias shape the
/// superseded template spelled.
///
/// **A fold whose operand is a declared input has no prologue at all, and that is
/// recognized rather than refused.** The walk below is run for it too, so the
/// obligations it states — every declared input read, every read at the
/// contributor domain — are discharged for this shape by the same authority and
/// under the same rules. What differs is what the walk returns: a bare input leaf
/// claiming no occurrence, which is the fold's own contributor read and not an
/// expression a region computes. Recording `None` is therefore the fact, and it is
/// what makes the fold's contributor access bind the input tensor directly instead
/// of an intermediate a synthesized identity prologue would have had to
/// materialize.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the unrecognized
/// property: `sum-signature`, `sum-output`, `sum-shape`, `sum-axes*`, and
/// `input-rank` for the reduction itself, `operation-set` when the recognized
/// occurrences do not cover the program, and every rule
/// [`recognize_elementwise`] reports for the contributor walk, including
/// `reduction-contributor-materialization` when that walk reaches a value a
/// recognized folding or staged family materializes.
fn recognize_reduction(
    program: &SemanticProgram,
    result: ValueId,
    output_key: OutputKey,
    sum_member: u32,
    sum: &tiler_ir::semantic::OperationRef<'_>,
    laws: &FrozenIndexRealizationLawRegistry,
) -> Result<NormalizedSerialSum, RequestError> {
    let declared: Vec<ValueId> = program.inputs().map(|input| input.value()).collect();
    let sum_operands: Vec<_> = sum.operands().collect();
    let [contributor] = sum_operands.as_slice() else {
        return mismatch("sum-signature");
    };
    if sum.results().collect::<Vec<_>>() != [result] {
        return mismatch("sum-output");
    }
    let axes = reduction_axes(sum.attributes())?;
    let input_shape = static_shape(program, *contributor, "input-handle")?;
    if input_shape.rank() == 0 {
        return mismatch("input-rank");
    }
    check_canonical_reduction_axes(&axes, input_shape.rank())?;
    let output_shape = input_shape.without_axes(&axes);
    if static_shape_ref(program, result) != Some(&output_shape) {
        return mismatch("sum-shape");
    }

    // `f32` is the fold family's own, not the enclosing program's: the caller
    // reached this function by matching `tiler::strict-serial-sum-f32@1`, so the
    // contributor tensor and every occurrence feeding it are binary32 or the
    // program is mixed-width and `recognized_program_arithmetic` already refused
    // it. Stating the width at the call site is what lets
    // [`NormalizedSerialSum::prologue`] stay typed as the one vocabulary a fold's
    // prologue region can carry.
    let RecognizedElementwise {
        expression: recognized_expression,
        members: recognized_members,
        reads: recognized_reads,
    } = recognize_elementwise(
        program,
        *contributor,
        &declared,
        &input_shape,
        laws,
        ArithmeticType::F32,
    )?;
    // The walk claims an occurrence for every leaf and node it mints except one:
    // a declared input contributes the leaf that reads it and nothing else. So a
    // fold straight over a declared input arrives here with an empty member set
    // and a bare input leaf, and that leaf is the fold's own contributor read
    // rather than a prologue any region computes — which is why the condition
    // tested is the operand itself and not the emptiness that follows from it.
    let prologue = if declared.contains(contributor) {
        None
    } else {
        Some(recognized_expression.into_f32()?)
    };
    // The read list belongs to the prologue *region*, so a fold that has no
    // prologue states none. The walk still returns the fold's own contributor
    // read, and recording it here would describe a region no cover places.
    let prologue_reads = if prologue.is_some() {
        recognized_reads
    } else {
        Vec::new()
    };
    // The fold's own contributor read, recorded exactly when there is no
    // prologue region to describe it. It is resolved here because this is the
    // only place that holds both the contributor value and the declaration
    // order; every physical spelling of the fold asks for the answer.
    let contributor_input = if prologue.is_some() {
        None
    } else {
        Some(declared_ordinal(&declared, *contributor)?)
    };
    let members = RecognizedSerialSumMembers::new(recognized_members, sum_member);

    let input_elements = element_count_u64(&input_shape, "input")?;
    let output_elements = element_count_u64(&output_shape, "output")?;
    Ok(NormalizedSerialSum {
        input_keys: program.inputs().map(|input| input.key().clone()).collect(),
        output_key,
        input_shape,
        output_shape,
        reduction_axes: axes,
        prologue,
        prologue_reads,
        contributor_input,
        members,
        inputs: declared,
        pointwise_result: *contributor,
        output: result,
        input_elements,
        output_elements,
    })
}

/// Recognizes a two-input binary tensor contraction over `f32`.
///
/// The admitted set is *every* well-formed binary index structure the semantic
/// registry validates, not one hard-coded matmul spelling. That is not a
/// widening for its own sake: the physical realization addresses each operand
/// axis by whichever output or contracted coordinate the structure binds it to,
/// so a structure whose contracted index sits at a different axis of each
/// operand costs this recognizer nothing extra and refusing it would be a check
/// with no correctness content behind it. What stays narrow is everything else —
/// exactly two operands, exactly one contraction operation reachable, `f32`
/// throughout, and no attribute beyond the index structure.
fn normalize_contraction(
    program: &SemanticProgram,
    result: ValueId,
    output_key: OutputKey,
) -> Result<NormalizedContraction, RequestError> {
    // An elementwise epilogue over a contraction result is a two-region chain
    // this profile assembles as a two-region chain, and this normalization is
    // the producer half of it: [`recognize_epilogue`] reaches here with the
    // contraction's own result value rather than a declared program output, and
    // `contraction_region` writes whichever tensor the cover assigns.
    let (ordinal, operation) = producer(program, result, &strict_tensor_contraction_f32_op())?;
    if operation.results().collect::<Vec<_>>() != [result] {
        return mismatch("contraction-output");
    }
    // Exactly the index structure. An attribute this normalization does not
    // carry forward is a semantic fact it would silently drop.
    let [field] = operation.attributes().fields() else {
        return mismatch("contraction-attributes");
    };
    if field.id() != CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE {
        return mismatch("contraction-attributes");
    }
    let structure =
        ContractionIndexStructure::from_canonical_value(field.value()).map_err(|_| {
            RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "contraction-structure",
            }
        })?;
    if structure.operand_count() != 2 {
        return mismatch("contraction-operand-count");
    }

    // Each structure operand must be one distinct declared input. The complete
    // declaration may be wider: a sibling output or a later stage can read an
    // input this contraction does not. Each read therefore carries both the
    // program ordinal the ABI binds and the structure operand position it
    // supplies, then the pair is canonicalized by ascending program ordinal.
    let operands: [ValueId; 2] = operation
        .operands()
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "contraction-operand-count",
        })?;
    let declared: Vec<ValueId> = program.inputs().map(|input| input.value()).collect();
    let shape_of = |value: ValueId| static_shape(program, value, "input-handle");
    let mut reads = Vec::with_capacity(2);
    for (position, operand) in operands.into_iter().enumerate() {
        let Some(declaration) = declared.iter().position(|declared| *declared == operand) else {
            return mismatch("contraction-operands");
        };
        let input_ordinal =
            u32::try_from(declaration).map_err(|_| RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "input-ordinal",
            })?;
        let shape = shape_of(operand)?;
        let elements = element_count_u64(&shape, "input")?;
        reads.push(NormalizedContractionRead {
            input_ordinal,
            shape,
            elements,
            value: operand,
            operand_position: position,
        });
    }
    reads.sort_by_key(|read| read.input_ordinal);
    let reads: [NormalizedContractionRead; 2] =
        reads
            .try_into()
            .map_err(|_| RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "contraction-operand-count",
            })?;
    if reads[0].input_ordinal >= reads[1].input_ordinal {
        return mismatch("contraction-operands");
    }

    // One extent per index, bound by the first operand axis naming it. The
    // semantic inferencer already proved agreement at construction, so a
    // disagreement here is invalid state and is refused rather than preferred
    // one way.
    let mut extents: Vec<(ContractionIndex, Extent)> = Vec::new();
    for read in &reads {
        let tuple = structure.operand(read.operand_position).ok_or(
            RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "contraction-structure",
            },
        )?;
        if read.shape.rank() != tuple.len() {
            return mismatch("contraction-rank");
        }
        for (axis, index) in tuple.iter().enumerate() {
            let extent = read.shape.extents()[axis];
            match extents.iter().find(|(bound, _)| bound == index) {
                Some((_, bound)) if *bound != extent => return mismatch("contraction-extent"),
                Some(_) => {}
                None => extents.push((*index, extent)),
            }
        }
    }
    let extent_of = |index: &ContractionIndex| -> Result<Extent, RequestError> {
        extents
            .iter()
            .find(|(bound, _)| bound == index)
            .map(|(_, extent)| *extent)
            .ok_or(RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "contraction-extent",
            })
    };
    let shape_over = |indices: &[ContractionIndex]| -> Result<Shape, RequestError> {
        Shape::try_new(
            indices
                .iter()
                .map(&extent_of)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "contraction-shape",
        })
    };
    let output_shape = shape_over(structure.output())?;
    let contracted_shape = shape_over(structure.contracted())?;
    if static_shape_ref(program, result) != Some(&output_shape) {
        return mismatch("contraction-output-shape");
    }

    let output_elements = element_count_u64(&output_shape, "output")?;
    let contracted_elements = element_count_u64(&contracted_shape, "input")?;
    // `direct`'s one precondition, and its only one. The semantic inferencer
    // refuses a zero contracted extent at construction, so this is unreachable
    // through a built program; it is kept because it is the *stated* precondition
    // of this realization and a reader must be able to find it here rather than
    // infer it from an inferencer three crates away.
    if contracted_elements == 0 {
        return mismatch("contraction-empty-domain");
    }

    Ok(NormalizedContraction {
        input_keys: program.inputs().map(|input| input.key().clone()).collect(),
        output_key,
        reads,
        output_shape,
        contracted_shape,
        structure,
        members: vec![SemanticStage::first(SemanticMemberId(ordinal))],
        output: result,
        output_elements,
        contracted_elements,
    })
}

fn check_budget(resource: BudgetResource, limit: u32, actual: usize) -> Result<(), RequestError> {
    let limit = u64::from(limit);
    // Saturating, on the same ground as the four `count` helpers this crate
    // already carries: no supported target has a `usize` wider than `u64`, and a
    // count that did not fit would exceed every budget this profile declares.
    let actual = u64::try_from(actual).unwrap_or(u64::MAX);
    if actual > limit {
        return Err(RequestError::BudgetExceeded {
            resource,
            limit,
            actual,
        });
    }
    Ok(())
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

fn producer<'a>(
    program: &'a SemanticProgram,
    value: ValueId,
    expected: &OpKey,
) -> Result<(u32, tiler_ir::semantic::OperationRef<'a>), RequestError> {
    let (ordinal, operation) = producer_for_value(program, value)?;
    if operation.key() != expected {
        return mismatch("operation-family");
    }
    Ok((ordinal, operation))
}

fn producer_for_value(
    program: &SemanticProgram,
    value: ValueId,
) -> Result<(u32, tiler_ir::semantic::OperationRef<'_>), RequestError> {
    let (ordinal, operation) = program
        .operations()
        .enumerate()
        .find(|(_, operation)| operation.results().any(|result| result == value))
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "missing-producer",
        })?;
    let ordinal = u32::try_from(ordinal).map_err(|_| RequestError::UnsupportedCapability {
        phase: "strategy",
        rule: "operation-ordinal",
    })?;
    Ok((ordinal, operation))
}

/// Reads one exact constant occurrence's declared payload, in its own width.
///
/// **The payload is returned in a `u32` for both widths, and that is a carrier
/// rather than a widening.** The declared byte run is read at the exact length
/// the family declares — four for binary32, two for `bf16` — and a run of any
/// other length is refused rather than zero-extended, so a `bf16` payload that
/// arrived four bytes wide is a malformed record here instead of a number whose
/// upper half nobody stated. [`Bf16Mint::constant`] narrows back before minting.
///
/// The format key is checked against the *family's own* type key rather than
/// against binary32's: a record naming one family and carrying another's format
/// is a disagreement between its two halves, and admitting it would let a
/// `bf16` occurrence carry a binary32 pattern into a region whose identity
/// claims `bf16`.
fn constant_bits(
    program: &SemanticProgram,
    value: ValueId,
    arithmetic: ArithmeticType,
) -> Result<(u32, u32), RequestError> {
    let Some(family) = constant_family(arithmetic) else {
        return mismatch("dtype-recognized");
    };
    let (ordinal, operation) = producer(program, value, &family)?;
    if operation.operands().len() != 0 || operation.results().len() != 1 {
        return mismatch("constant-signature");
    }
    let (attribute, name) = match arithmetic {
        ArithmeticType::F32 => (F32_CONSTANT_BITS_ATTRIBUTE, "f32"),
        ArithmeticType::Bf16 => (BF16_CONSTANT_BITS_ATTRIBUTE, "bf16"),
        // Unreachable through the family lookup above, and refused rather than
        // defaulted to either row: a payload field guessed for a width this
        // recognizer states no constant family for would read some other
        // family's bytes.
        ArithmeticType::F16 | ArithmeticType::F64 => return mismatch("dtype-recognized"),
    };
    let Some(CanonicalValueView::FloatBits(bits)) = operation
        .attributes()
        .get(attribute)
        .map(tiler_ir::semantic::CanonicalValue::view)
    else {
        return mismatch("constant-bits");
    };
    let governed =
        TypeKey::new("tiler", name, 1).map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "governed-constant-key",
        })?;
    if bits.format() != &governed {
        return mismatch("constant-bits-format");
    }
    let packed = match arithmetic {
        ArithmeticType::F32 => <[u8; 4]>::try_from(bits.bits())
            .map(u32::from_be_bytes)
            .ok(),
        ArithmeticType::Bf16 => <[u8; 2]>::try_from(bits.bits())
            .map(|bytes| u32::from(u16::from_be_bytes(bytes)))
            .ok(),
        ArithmeticType::F16 | ArithmeticType::F64 => None,
    };
    packed
        .map(|packed| (packed, ordinal))
        .ok_or(RequestError::UnsupportedCapability {
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

/// Returns one semantic value's fixed shape, refusing a symbolic one by name.
///
/// Recognition matches a program against a physical strategy, and every
/// strategy below is stated over fixed extents: a domain a launch geometry is
/// derived from, an element count, a reindex or broadcast axis decode. A
/// symbolic extent is refused here rather than resolved through the
/// environment, which would make the recognized region name extents nobody
/// wrote.
///
/// The refusal names the extent as written, not the handle lookup that
/// observed it. A bound symbol is still this refusal: specializing it into the
/// logical plan is a physical-planning decision this boundary must not make.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] for a foreign handle, and
/// [`RequestError::UnsupportedSymbolicExtent`] naming the first non-static
/// extent when the value's shape is symbolic.
fn static_shape(
    program: &SemanticProgram,
    value: ValueId,
    rule: &'static str,
) -> Result<Shape, RequestError> {
    let sourced = program
        .shape(value)
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule,
        })?;
    if let Some(shape) = sourced.as_static() {
        return Ok(shape.clone());
    }
    let extent = sourced
        .extents()
        .find(|extent| extent.as_static().is_none())
        .expect("a non-static SourcedShape holds at least one symbol");
    Err(RequestError::UnsupportedSymbolicExtent {
        phase: "strategy",
        rule: "symbolic-extent",
        extent,
    })
}

/// Returns one value's fixed shape, or `None` for a foreign or symbolic one.
///
/// The borrowing form, for the comparisons that already treat an unreadable
/// shape as a mismatch. A symbolic shape compares unequal to every [`Shape`],
/// which is the answer those sites want: the strategy is not recognized.
fn static_shape_ref(program: &SemanticProgram, value: ValueId) -> Option<&Shape> {
    program.shape(value).ok()?.as_static()
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

/// The elementwise planner's spelling of [`mismatch`].
///
/// Separate because the planner's error type additionally carries a discovered
/// materialization boundary, which is a finding rather than a rule.
fn refused<T>(rule: &'static str) -> Result<T, ElementwiseRefusal> {
    Err(ElementwiseRefusal::Refused(
        RequestError::UnsupportedCapability {
            phase: "strategy",
            rule,
        },
    ))
}

fn unsupported<T>(phase: &'static str, rule: &'static str) -> Result<T, RequestError> {
    Err(RequestError::UnsupportedCapability { phase, rule })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use tiler_ir::schedule::FlushedZeroSign;
    use tiler_ir::semantic::{
        Bf16Add, Bf16Constant, Bf16Multiply, CanonicalValue, CanonicalValueKind, F32Add,
        F32Constant, F32Gather, F32Multiply, F32RmsNorm, NormativeDefinitionRef, OperationArity,
        OperationAttributeSchema, OperationAttributes, OperationConformance, OperationDefinition,
        OperationDefinitionFacts, OperationEffect, OperationInferenceError, OperationInferencer,
        OperationSchema, ProviderDiagnosticCode, ProviderIdentity, RegistryError,
        ResolvedValueType, SemanticProgramBuilder, SemanticRegistryBuilder,
        SemanticRegistryProvider, SemanticRegistryRegistrar, StrictSerialF32Sum,
        TypeDefinitionFacts, ValueFact, ValueTypeDefinition, ValueTypeDefinitionKey,
        gather_index_resolved_type,
    };
    use tiler_ir::shape::{
        BindingSource, ExtentRelation, ExtentTerm, FactProvenance, RootBinding,
        SemanticInputConstraint, ShapeEnv, ShapeEnvBuilder, ShapeSymbol, SymbolScope,
    };

    fn diagnostic_code(value: &str) -> ProviderDiagnosticCode {
        ProviderDiagnosticCode::new(value).unwrap()
    }

    /// Every resolution of every governed dimension, in canonical order.
    ///
    /// The population is counted rather than described: an enumeration that
    /// silently lost a resolution would make every claim below pass over a
    /// smaller space than it names, which is the failure mode a uniform pass
    /// hides. Each row's length is asserted where it is consumed.
    fn statable_contracts() -> Vec<StrictF32NumericalContract> {
        let subnormals = [
            SubnormalMode::Preserve,
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            },
        ];
        let permissions = [
            NumericalPermission::Forbidden,
            NumericalPermission::Permitted,
        ];
        let envelopes = [
            ApproximationEnvelope::Forbidden,
            ApproximationEnvelope::BackendElementary,
        ];
        let assumptions = [
            ExceptionalValueAssumption::MakeNoAssumption,
            ExceptionalValueAssumption::AssumeAbsent {
                provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
            },
        ];
        let roundings = [MaterializationRounding::NearestTiesToEven];
        let mut contracts = Vec::new();
        for input in subnormals {
            for result in subnormals {
                for contraction in permissions {
                    for reassociation in permissions {
                        for permutation in permissions {
                            for signed_zero in permissions {
                                for reciprocal_transform in permissions {
                                    for approximate_intrinsics in envelopes {
                                        for nan_assumptions in assumptions {
                                            for infinity_assumptions in assumptions {
                                                for materialization_rounding in roundings {
                                                    contracts.push(
                                                        StrictF32NumericalContract {
                                                            input_subnormals: input,
                                                            result_subnormals: result,
                                                            contraction,
                                                            reassociation,
                                                            permutation,
                                                            signed_zero,
                                                            reciprocal_transform,
                                                            approximate_intrinsics,
                                                            nan_assumptions,
                                                            infinity_assumptions,
                                                            materialization_rounding,
                                                            ..StrictF32NumericalContract::governed()
                                                        }
                                                        .keyed(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        contracts
    }

    /// The size of the statable space, spelled as its factors.
    ///
    /// Written as the product rather than as `2304` so a widened behaviour space
    /// changes the expected count at the factor that moved, and a reader can
    /// check the arithmetic against the vocabulary instead of trusting a
    /// literal. The factors, in canonical dimension order: three subnormal
    /// resolutions twice, two transform permissions five times, two
    /// approximation envelopes, and two caller-statable exceptional-value
    /// assumptions twice. Compiler-proven and runtime-validated provenance are
    /// derived evidence, not caller-statements, and therefore are not keys in
    /// this population.
    /// Materialization rounding contributes no factor because it has exactly one
    /// resolution, so its absence here is the note rather than a `* 1` term.
    const STATABLE_CONTRACTS: usize = 3 * 3 * 2 * 2 * 2 * 2 * 2 * 2 * 2 * 2;

    /// The canonical key separates every statable contract from every other.
    ///
    /// **Exhaustive finite evidence, not a sample.** The key is the contract's
    /// standing identity: [`tiler_ir::index::NumericalContractIdentity`], the fusion
    /// legality content identity, and the scheduled region's `profile_key` each
    /// carry it *alone*, with no dimension beside it, so two contracts sharing a
    /// key would give two stated meanings one artifact and one cache entry. The
    /// space is finite and small enough to walk, so it is walked.
    #[test]
    fn the_canonical_key_is_injective_over_the_statable_space() {
        let contracts = statable_contracts();
        assert_eq!(
            contracts.len(),
            STATABLE_CONTRACTS,
            "the enumeration does not cover the space it names",
        );
        let mut keys: Vec<&str> = contracts.iter().map(|contract| contract.key).collect();
        let mut lengths: Vec<usize> = keys.iter().map(|key| key.len()).collect();
        lengths.sort_unstable();
        lengths.dedup();
        assert_eq!(
            lengths,
            [98, 100, 102],
            "the statable grammar reached an unexpected rendered length",
        );
        for contract in &contracts {
            let parsed = F32NumericalContractKey::try_from_str(contract.key)
                .expect("every statable compiler key is admitted by IR");
            assert_eq!(
                canonical_contract_key(contract).unwrap(),
                parsed.as_str(),
                "compiler and IR canonical encoders disagree"
            );
        }
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(
            keys.len(),
            STATABLE_CONTRACTS,
            "two statable contracts share one key",
        );
    }

    /// Every minted key is spelled in the governed key alphabet.
    ///
    /// A key is compared byte for byte against one minted by a build that never
    /// met this one and is printed in rejections a reader copies back out, so the
    /// alphabet is part of what a key is: ASCII lowercase, digits, and `.`, with
    /// no case, whitespace, or control byte. It is also carried into an explain
    /// `SubjectKey`, which refuses anything longer than 255 bytes.
    #[test]
    fn every_minted_key_is_spelled_in_the_governed_alphabet() {
        let contracts = statable_contracts();
        assert_eq!(contracts.len(), STATABLE_CONTRACTS);
        for contract in contracts {
            let key = contract.key;
            assert!(
                crate::explain::SubjectKey::new(key).is_ok(),
                "{key} is not admissible as an explain subject",
            );
            assert!(key.len() <= 255, "{key} exceeds the explain key bound");
            assert!(
                key.bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'.'),
                "{key} leaves the governed key alphabet",
            );
            assert!(
                is_f32_contract_key(key),
                "{key} is not recognized as an f32 contract key",
            );
        }
    }

    /// The domain prefix test says no, and says it for the right reason.
    ///
    /// Driven against cases that must fail, because a predicate that only ever
    /// sees keys it accepts is indistinguishable from one that returns `true`.
    #[test]
    fn the_contract_key_domain_test_refuses_a_key_from_another_domain() {
        assert!(is_f32_contract_key(
            StrictF32NumericalContract::governed().key
        ));
        for refused in [
            "",
            tiler_ir::schedule::F32_NUMERICAL_CONTRACT_KEY_DOMAIN,
            "tiler.contract.f32.v20.0011",
            "tiler.contract.f16.v2.0011",
            "tiler.strict-f32.v1",
            crate::policy::UNKEYED_CONTRACT,
        ] {
            assert!(!is_f32_contract_key(refused), "{refused} was admitted");
        }
    }

    /// A key names its own width, and its width names its own element size.
    ///
    /// **The `f32` answer is the pair's control.** Reporting two bytes for a BF16
    /// key means nothing unless the binary32 key still reports four, because a
    /// derivation that had simply stopped resolving would answer `None` for both
    /// and pass a one-sided assertion by failing to be asked. And the refused
    /// cases are what stop the width from being read off a textual prefix: each
    /// is rejected by the IR-owned parse rather than admitted at some default.
    #[test]
    fn a_governed_contract_key_derives_its_own_width_and_element_size() {
        let f32_key = StrictF32NumericalContract::governed().key;
        let bf16_key = crate::session::NumericalContract::STRICT_BF16.key();
        assert_ne!(f32_key, bf16_key);

        assert_eq!(contract_key_arithmetic(f32_key), Some(ArithmeticType::F32));
        assert_eq!(
            contract_key_arithmetic(bf16_key),
            Some(ArithmeticType::Bf16)
        );
        assert_eq!(contract_key_element_bytes(f32_key), Some(4));
        assert_eq!(contract_key_element_bytes(bf16_key), Some(2));

        // A key under no governed domain answers `None` on both, which is what
        // makes an unregistered width report `Unknown` rather than continue at a
        // neighbour's size.
        for refused in [
            "",
            tiler_ir::schedule::F32_NUMERICAL_CONTRACT_KEY_DOMAIN,
            tiler_ir::schedule::BF16_NUMERICAL_CONTRACT_KEY_DOMAIN,
            "tiler.contract.f16.v2.0011",
            "tiler.contract.bf16.v10.0011",
            crate::policy::UNKEYED_CONTRACT,
        ] {
            assert_eq!(
                contract_key_arithmetic(refused),
                None,
                "{refused} was admitted"
            );
            assert_eq!(
                contract_key_element_bytes(refused),
                None,
                "{refused} was sized"
            );
        }
    }

    /// Omission resolves strict on every dimension, so it can never widen.
    ///
    /// Checked against the *canonical dimension walk* rather than field by
    /// field, so a dimension added to the vocabulary is covered by this claim the
    /// moment it exists rather than when someone remembers to add a line.
    #[test]
    fn an_unstated_dimension_resolves_strict() {
        let strict = StrictF32NumericalContract::governed();
        for dimension in crate::target::honourability::CANONICAL_DIMENSIONS {
            let behaviour = strict.behaviour(dimension);
            let is_strict = match behaviour {
                DimensionBehaviour::Subnormals(mode) => mode == SubnormalMode::Preserve,
                DimensionBehaviour::Transform(permission) => {
                    permission == NumericalPermission::Forbidden
                }
                DimensionBehaviour::Approximation(envelope) => {
                    envelope == ApproximationEnvelope::Forbidden
                }
                DimensionBehaviour::ExceptionalValue(assumption) => {
                    assumption == ExceptionalValueAssumption::MakeNoAssumption
                }
                DimensionBehaviour::Rounding(rounding) => {
                    rounding == MaterializationRounding::NearestTiesToEven
                }
            };
            assert!(
                is_strict,
                "{} does not resolve strict when unstated",
                dimension.key()
            );
        }
    }

    /// A caller-stated absence on evidence it is not the author of is refused.
    ///
    /// Both dimensions and both refused provenance classes, and the accepted
    /// class beside them, so the check is shown saying yes and no rather than
    /// only yes.
    #[test]
    fn an_absence_on_unfounded_provenance_is_incoherent() {
        for (dimension, apply) in [
            (
                ExceptionalValueDimensionKind::Nan,
                (|contract: &mut StrictF32NumericalContract,
                  assumption: ExceptionalValueAssumption| {
                    contract.nan_assumptions = assumption;
                })
                    as fn(&mut StrictF32NumericalContract, ExceptionalValueAssumption),
            ),
            (
                ExceptionalValueDimensionKind::Infinity,
                |contract, assumption| {
                    contract.infinity_assumptions = assumption;
                },
            ),
        ] {
            for provenance in [
                ValueDomainProvenance::CompilerProven,
                ValueDomainProvenance::RuntimeValidated,
            ] {
                let mut contract = StrictF32NumericalContract::governed();
                apply(
                    &mut contract,
                    ExceptionalValueAssumption::AssumeAbsent { provenance },
                );
                assert_eq!(
                    coherence(&contract),
                    Err(IncoherentContract::UnfoundedValueDomainProvenance {
                        dimension,
                        provenance,
                    }),
                );
                assert!(!contract.keyed().is_governed());
            }
            let mut declared = StrictF32NumericalContract::governed();
            apply(
                &mut declared,
                ExceptionalValueAssumption::AssumeAbsent {
                    provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
                },
            );
            assert_eq!(coherence(&declared), Ok(()));
        }
    }

    /// The eliminated combinations are coherent, and each is named.
    ///
    /// The enumeration on [`IncoherentContract`] is only refutable if the
    /// combinations it *rejected as candidates* are driven: a later change that
    /// quietly started refusing one of these would be narrowing what a caller may
    /// state without anyone deciding to.
    #[test]
    fn the_eliminated_combinations_are_coherent() {
        let flush_preserving = SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        };
        let flush_positive = SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        };
        let declared = ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
        };
        let cases: [(&str, StrictF32NumericalContract); 6] = [
            (
                "assumed-absent NaNs beside a canonical arithmetic NaN pattern",
                StrictF32NumericalContract {
                    nan_assumptions: declared,
                    ..StrictF32NumericalContract::governed()
                },
            ),
            (
                "one exceptional value assumed absent and the other not",
                StrictF32NumericalContract {
                    infinity_assumptions: declared,
                    ..StrictF32NumericalContract::governed()
                },
            ),
            (
                "permitted signed-zero elimination beside a sign-preserving flush",
                StrictF32NumericalContract {
                    input_subnormals: flush_preserving,
                    signed_zero: NumericalPermission::Permitted,
                    ..StrictF32NumericalContract::governed()
                },
            ),
            (
                "forbidden signed-zero elimination beside an always-positive flush",
                StrictF32NumericalContract {
                    result_subnormals: flush_positive,
                    ..StrictF32NumericalContract::governed()
                },
            ),
            (
                "permitted contraction with forbidden reassociation",
                StrictF32NumericalContract {
                    contraction: NumericalPermission::Permitted,
                    ..StrictF32NumericalContract::governed()
                },
            ),
            (
                "permitted permutation with forbidden reassociation",
                StrictF32NumericalContract {
                    permutation: NumericalPermission::Permitted,
                    ..StrictF32NumericalContract::governed()
                },
            ),
        ];
        for (name, contract) in cases {
            assert_eq!(coherence(&contract), Ok(()), "{name} was refused");
        }
    }

    /// A key that does not describe the vector beside it is not admitted.
    ///
    /// The direction that matters: a contract carrying a name from before a
    /// dimension moved would otherwise reach a plan under a key that describes a
    /// different meaning.
    #[test]
    fn a_contract_whose_key_does_not_describe_it_is_refused() {
        let strict = StrictF32NumericalContract::governed();
        assert!(strict.is_governed());
        let mutated = StrictF32NumericalContract {
            reassociation: NumericalPermission::Permitted,
            ..strict
        };
        assert!(
            !mutated.is_governed(),
            "a widened contract kept the strict key and was admitted",
        );
        assert!(mutated.keyed().is_governed());
        let unkeyed = crate::policy::strict_contract(
            ArithmeticType::F32,
            tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS,
        );
        assert!(!unkeyed.is_governed(), "an unkeyed contract was admitted");
    }

    pub(super) fn program() -> SemanticProgram {
        program_with_builder(SemanticProgramBuilder::try_standard().unwrap())
    }

    /// One ordinary governed gather occurrence over the admitted F32/U32 signature.
    fn gather_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let source = builder
            .input::<F32>(InputKey::new("source").unwrap(), Shape::from_dims([4, 2]))
            .unwrap();
        let index = builder
            .input_resolved(
                InputKey::new("index").unwrap(),
                Shape::from_dims([3]),
                gather_index_resolved_type(),
            )
            .unwrap();
        let gathered = F32Gather::apply(&mut builder, source, index, Axis::new(0)).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), gathered)
            .unwrap();
        builder.build().unwrap()
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

    /// Builds one whole-program elementwise fixture and its expected nodes.
    ///
    /// `(first * second) + third` over three declared inputs. It is deliberately
    /// *not* a shape the superseded template could spell: two of its leaves are
    /// distinct input tensors rather than constants, and the old recognizer
    /// demanded exactly one input.
    fn three_input_elementwise() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let inputs: Vec<_> = ["a", "b", "c"]
            .into_iter()
            .map(|key| {
                builder
                    .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([2, 3]))
                    .unwrap()
            })
            .collect();
        let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
        let root = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    }

    /// A normalization over `[2, 2]` reduced on axis one, optionally scaled.
    ///
    /// `weighted` decides which of the two shapes the ticket names is built: the
    /// family as the whole declared output, and the family as a program stage a
    /// later elementwise pass consumes.
    fn normalization_program(weighted: bool, eps_bits: u32) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let shape = Shape::from_dims([2, 2]);
        let value = builder
            .input::<F32>(InputKey::new("value").unwrap(), shape.clone())
            .unwrap();
        let weight = builder
            .input::<F32>(InputKey::new("weight").unwrap(), shape)
            .unwrap();
        let normalized = tiler_ir::semantic::F32RmsNorm::apply(
            &mut builder,
            value,
            weight,
            Axis::new(1),
            eps_bits,
        )
        .unwrap();
        let root = if weighted {
            F32Multiply::apply(&mut builder, normalized, value).unwrap()
        } else {
            normalized
        };
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    }

    /// A registered family whose law realizes a region sequence is a program
    /// stage, both as the declared output and as a chain's producer.
    ///
    /// **The recognition is the law's and the partition is the occurrence's, and
    /// both halves are asserted.** `tiler::rms-norm-f32@1` reaches this arm
    /// because its registered `IndexRealizationLaw` realizes a region *sequence*
    /// — no operation key appears in the recognizer — and what the recognized
    /// part claims is the occurrence, once, because region formation is the
    /// authority that enumerates the stages. `owns_region_members` therefore
    /// answers for whichever stage atoms formation minted, which is what lets a
    /// cover region covering one stage resolve to this output at all.
    ///
    /// Watched failing under a deliberate perturbation: removing the
    /// `laws.family_realizes_region_sequence(operation.key())` disjunct from
    /// `plan_elementwise`'s folding discovery refuses the weighted program under
    /// `operation-set`, which is the wall this ticket moved.
    #[test]
    fn a_registered_staged_family_is_recognized_as_a_program_stage() {
        let eps = 1.0e-6_f32.to_bits();

        // The family as the whole declared output.
        let whole = normalization_program(false, eps);
        let NormalizedOutput::Staged(staged) = recognize(&whole).unwrap() else {
            panic!("a family whose law realizes a region sequence is a staged stage")
        };
        assert_eq!(staged.operation, tiler_ir::semantic::rms_norm_f32_op());
        assert_eq!(staged.member, SemanticMemberId(0));
        assert_eq!(
            staged.operand_reads,
            [BoundaryRead::Input(0), BoundaryRead::Input(1)]
        );
        assert_eq!(staged.producer, None);
        assert_eq!(staged.output_shape, Shape::from_dims([2, 2]));
        assert_eq!(staged.output_elements, 4);
        assert!(
            !staged.attributes.is_empty(),
            "the occurrence's axis and eps record reaches the recognized shape"
        );

        // The family as a program stage a later pass consumes: the walk names
        // the value the chain materializes and the producer is this shape.
        let weighted = normalization_program(true, eps);
        let NormalizedOutput::Epilogue(chain) = recognize(&weighted).unwrap() else {
            panic!("an elementwise pass over a staged family's result is a chain")
        };
        let NormalizedOutput::Staged(producer) = chain.producer.as_ref() else {
            panic!("the chain's producer is the staged family")
        };
        assert_eq!(producer.member, SemanticMemberId(0));
        assert_eq!(chain.members, [SemanticStage::first(SemanticMemberId(1))]);

        // The partition: the occurrence once, and every region whose atoms are
        // stages of it.
        let output = NormalizedOutput::Staged(producer.clone());
        assert_eq!(
            output.members(),
            [SemanticStage::first(SemanticMemberId(0))]
        );
        let fold = SemanticStage::first(SemanticMemberId(0));
        let pass = fold.next_stage();
        assert!(output.owns_region_members(&[fold]));
        assert!(output.owns_region_members(&[pass]));
        assert!(output.owns_region_members(&[fold, pass]));
        assert!(
            !output.owns_region_members(&[]),
            "an empty member set is no region of this occurrence"
        );
        assert!(
            !output.owns_region_members(&[fold, SemanticStage::first(SemanticMemberId(1))]),
            "a region straddling the consumer belongs to no single part"
        );
    }

    /// A staged family reading a value another region *computes* refuses by name.
    ///
    /// **This is the neighbour that keeps the widening below attributable, and
    /// its rule survives the widening with a narrower meaning.** A multiply's
    /// result is no materialization edge — [`materializes_its_result`] is the one
    /// statement of where an edge may sit, and it says the expression vocabulary
    /// evaluates a multiply per point — so admitting it here would be a second
    /// account of that fact, and materializing it would add exactly the
    /// observable rounding boundary the caller's program never asked for. Only
    /// the operand differs between this program and
    /// [`a_staged_family_reading_a_materialized_intermediate_is_recognized`].
    ///
    /// Watched failing under a deliberate perturbation: replacing
    /// `materializes_its_result(&root, laws)` with `true` admits the walk to
    /// [`recognize_epilogue_producer`], which refuses the same program under
    /// `operation-set` — a true statement about the producing family and not
    /// about this occurrence's operand, and the reason the guard states the
    /// operand rule itself.
    #[test]
    fn a_staged_family_reading_a_computed_value_refuses_by_name() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let shape = Shape::from_dims([2, 2]);
        let value = builder
            .input::<F32>(InputKey::new("value").unwrap(), shape.clone())
            .unwrap();
        let weight = builder
            .input::<F32>(InputKey::new("weight").unwrap(), shape)
            .unwrap();
        let doubled = F32Multiply::apply(&mut builder, value, value).unwrap();
        let normalized = tiler_ir::semantic::F32RmsNorm::apply(
            &mut builder,
            doubled,
            weight,
            Axis::new(1),
            1.0e-6_f32.to_bits(),
        )
        .unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), normalized)
            .unwrap();
        let program = builder.build().unwrap();
        assert_eq!(recognize(&program).unwrap_err(), "staged-operand");
    }

    /// A normalization over a materialized contraction result, optionally with
    /// a trailing elementwise pass and optionally normalizing that result twice.
    ///
    /// `ab,bc->ac` over `a` and `b`, with an independent third `[2, 2]` input
    /// `w` serving as the normalization weight. The contraction's two reads are
    /// therefore a strict subset of the complete interface in the ordinary
    /// `rms_norm(matmul(a, b), w)` spelling.
    fn contraction_fed_normalization(passed: bool, doubly_staged: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let shape = Shape::from_dims([2, 2]);
        let left = builder
            .input::<F32>(InputKey::new("a").unwrap(), shape.clone())
            .unwrap();
        let right = builder
            .input::<F32>(InputKey::new("b").unwrap(), shape.clone())
            .unwrap();
        let independent_weight = builder
            .input::<F32>(InputKey::new("w").unwrap(), shape)
            .unwrap();
        let structure = ContractionIndexStructure::new(
            [
                vec![ContractionIndex::new(0), ContractionIndex::new(1)],
                vec![ContractionIndex::new(1), ContractionIndex::new(2)],
            ],
            [ContractionIndex::new(0), ContractionIndex::new(2)],
        )
        .expect("ab,bc->ac is an admitted structure");
        let product =
            tiler_ir::semantic::F32TensorContraction::apply(&mut builder, &structure, left, right)
                .unwrap();
        let weight = if doubly_staged {
            product
        } else {
            independent_weight
        };
        let normalized = tiler_ir::semantic::F32RmsNorm::apply(
            &mut builder,
            product,
            weight,
            Axis::new(1),
            1.0e-6_f32.to_bits(),
        )
        .unwrap();
        let root = if passed {
            F32Multiply::apply(&mut builder, normalized, independent_weight).unwrap()
        } else {
            normalized
        };
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    }

    /// A staged family reading a materialized intermediate is recognized, and
    /// the operand's boundary role is the recognized shape's.
    ///
    /// **The admission this ticket exists for.** `rms_norm(matmul(a, b), w)`
    /// reads its first operand across a materialization edge, which used to be
    /// refused under `staged-operand` because nothing in the recognized staged
    /// shape could record that operand zero is served by an edge rather than by a
    /// declared buffer. Both halves are asserted, because either alone would be
    /// consistent with a defect: the operand run names the boundary tensor per
    /// operand, and the producer is carried so that the contraction's occurrence
    /// is claimed by this output's walk — without which [`check_output_cover`]
    /// refuses the program under `operation-set` for an occurrence no walk owns.
    ///
    /// The partition is asserted too, on both sides of the edge, because it is
    /// what lets a cover place two regions here: the occurrence's own stages and
    /// the contraction's part are all this output's, and a set mixing them is
    /// nobody's.
    ///
    /// Watched failing under a deliberate perturbation: dropping the
    /// `producer` field from [`NormalizedOutput::members`]'s staged arm — so the
    /// walk claims only its own occurrence — refuses this program under
    /// `operation-set`, which is exactly the coverage obligation the producer is
    /// carried to discharge.
    #[test]
    fn a_staged_family_reading_a_materialized_intermediate_is_recognized() {
        let program = contraction_fed_normalization(false, false);
        assert_eq!(program.operation_count(), 2);
        let recognized = recognize(&program).expect("the staged operand is recognized");
        let NormalizedOutput::Staged(staged) = &recognized else {
            panic!("a normalization output recognizes as a staged family")
        };
        // The operand's source, carried by the recognized shape: operand zero is
        // the edge and operand one is the independent third declared input.
        assert_eq!(
            staged.operand_reads,
            [BoundaryRead::Staged, BoundaryRead::Input(2)]
        );
        assert_eq!(staged.member, SemanticMemberId(1));
        // The producer, recognized as the shape a standalone contraction output
        // would be, so every region builder the contraction already has applies
        // to it unchanged.
        let producer = staged
            .producer
            .as_deref()
            .expect("a staged operand carries the shape producing it");
        assert!(producer.contraction().is_some());
        assert_eq!(
            producer.members(),
            [SemanticStage::first(SemanticMemberId(0))]
        );

        // The whole partition: the contraction's part, and the occurrence's own
        // stages. The population is counted, so an assertion about the parts is
        // an assertion about the whole program's occurrences.
        assert_eq!(recognized.members().len(), program.operation_count());
        let fold = SemanticStage::first(SemanticMemberId(1));
        for part in [
            vec![SemanticStage::first(SemanticMemberId(0))],
            vec![fold],
            vec![fold.next_stage()],
        ] {
            assert!(
                recognized.owns_region_members(&part),
                "{part:?} is a part of this output's partition",
            );
        }
        assert!(
            !recognized.owns_region_members(&[SemanticStage::first(SemanticMemberId(0)), fold]),
            "a region straddling the materialization edge is no part",
        );

        // Which declared input each side reads, and at which count. Both are
        // read by the occurrence's own operand run *and* by the producer, and
        // the two agree at `[2, 2]`, so the accessor answers rather than
        // refusing.
        for ordinal in [0, 1, 2] {
            assert!(recognized.reads_declared_input(ordinal));
            assert_eq!(
                recognized.input_elements_at(InputOrdinal::new(ordinal)),
                Some(4),
            );
        }
        assert!(!recognized.reads_declared_input(3));
        assert_eq!(recognized.max_input_elements(), 4);

        // **The boundary this widening does not move, asserted rather than
        // implied.** The consuming stage would read the occurrence's operand
        // edge *and* the value the producing stage handed it, and
        // `TensorRole::Intermediate` carries no ordinal, so
        // [`crate::physical::staged_plan`] declines the occurrence outright. Its
        // control is the same law over two declared operands, whose plan exists
        // — without which this `None` would be evidence that the plan derivation
        // had stopped working rather than evidence about the edge.
        assert_eq!(crate::physical::staged_plan(staged), None);
        let declared = normalization_program(false, 1.0e-6_f32.to_bits());
        let NormalizedOutput::Staged(control) = recognize(&declared).unwrap() else {
            panic!("a normalization output recognizes as a staged family")
        };
        assert!(crate::physical::staged_plan(&control).is_some());
    }

    /// One fixture of [`every_arm_answers_the_declared_tensors_own_count`] and
    /// everything asserted about it.
    ///
    /// Named rather than a tuple so each column reads as the claim it is: the
    /// rows carry six columns each, and in a positional literal an exchanged
    /// pair of `u64`s looks like a passing row.
    struct CountRow {
        label: &'static str,
        /// The arm the fixture must reach, so a row whose recognition moved
        /// stops standing for the arm it names.
        arm: &'static str,
        output: NormalizedOutput,
        /// The iteration domain the widening read is *not* answered at, or
        /// `None` where the row has no widening read — for the two arms that
        /// hold no elementwise read list, and for the bare fold whose one read
        /// is dense.
        domain: Option<u64>,
        /// The count each declared ordinal must resolve to, in declaration
        /// order. Its length is the declared arity.
        counts: &'static [Option<u64>],
        max: u64,
    }

    /// Every arm of [`NormalizedOutput::input_elements_at`] answers the declared
    /// tensor's own element count, and none answers a reading region's domain.
    ///
    /// **The two numbers coincide unless a read widens, so most rows carry a
    /// widening one.** A `[2]` weight broadcast into a `[2, 2]` region iterates
    /// four points and holds two elements; an arm answering `4` would scale an
    /// opaque call by the iteration space rather than by the buffer
    /// [`TensorRole::Input`] binds at that ordinal, which is the confidently
    /// wrong work count [`crate::call_declaration::WorkScaling`] exists to
    /// prevent. Each row therefore states the domain beside the counts and
    /// refuses to run if they are equal, so a row that had no widening to get
    /// wrong cannot pass for one that did.
    ///
    /// **The rows are counted against the arms.** "Every arm" is the claim, so
    /// the population is asserted to reach all five rather than described as
    /// doing so; a variant added without a row fails here rather than shipping
    /// unexamined. [`NormalizedOutput::reads_declared_input`] is asserted beside
    /// every count because the two are separate statements of which ordinals a
    /// walk reached, and
    /// [`NormalizedProgram::agreed_input_elements_at`] refuses when they drift.
    ///
    /// **Watched failing once each, every perturbation on the subject rather
    /// than on an assertion, and each quoted by the row that caught it:**
    ///
    /// - Restoring [`NormalizedOutput::input_elements_at`]'s pointwise arm to
    ///   `normalized.elements`, the reading region's domain it answered before:
    ///   *a sole widened pointwise read: ordinal 0 is not the declared tensor's
    ///   own count — left `Some(4)`, right `Some(2)`*.
    /// - Restoring [`NormalizedOutput::max_input_elements`]'s pointwise arm to
    ///   the same domain, perturbed alone so the count rows still pass: *a sole
    ///   widened pointwise read: the largest declared input count this output
    ///   reads — left `4`, right `2`*. The two arms are perturbed separately
    ///   because together the first fires and hides the second.
    /// - Restoring the serial-sum arm to `normalized.input_elements`: *a widened
    ///   read in a fold's prologue: ordinal 0 — left `Some(4)`, right
    ///   `Some(2)`*.
    /// - Restoring the epilogue arm's consumed half to `chain.elements`: *a
    ///   widened read in a chain's epilogue: ordinal 1 — left `Some(4)`, right
    ///   `Some(2)`*.
    /// - Dropping the serial-sum arm's `contributor_input` term, which is the
    ///   one read no read list describes: *a prologue-less fold's own
    ///   contributor read: ordinal 0 — left `None`, right `Some(6)`*.
    /// - Answering [`read_tensor_elements`]'s structural arms with
    ///   `domain_elements` instead of the operand shape, which is the single
    ///   statement the three widening rows share: the first of them fires, *a
    ///   sole widened pointwise read: ordinal 0 — left `Some(4)`, right
    ///   `Some(2)`*, and each later row fires in turn once its predecessor is
    ///   admitted.
    #[test]
    fn every_arm_answers_the_declared_tensors_own_count() {
        // A `[2]` operand replicated across a leading axis into `[2, 2]`: the
        // read addresses two elements over a domain of four, which is the whole
        // difference these rows are about.
        let widen = |builder: &mut SemanticProgramBuilder,
                     operand: tiler_ir::semantic::Value<F32>| {
            let mapping = tiler_ir::semantic::BroadcastAxisMapping::new(
                [Extent::new(2), Extent::new(2)],
                [
                    tiler_ir::semantic::BroadcastAxisSource::Replicate,
                    tiler_ir::semantic::BroadcastAxisSource::FromOperand(Axis::new(0)),
                ],
            )
            .expect("one replicated axis over a rank-one operand is an admitted relation");
            tiler_ir::semantic::F32Broadcast::apply(builder, &mapping, operand)
                .expect("the standard registry admits the broadcast family")
        };
        let weight = |builder: &mut SemanticProgramBuilder| {
            builder
                .input::<F32>(InputKey::new("w").unwrap(), Shape::from_dims([2]))
                .unwrap()
        };

        // `w + w` over the widened read alone: one declared input, read only
        // through the relation, so this is the row where the maximum moves too.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let w = weight(&mut builder);
        let widened = widen(&mut builder, w);
        let root = F32Add::apply(&mut builder, widened, widened).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        let sole_widened_read = builder.build().unwrap();

        // `a * broadcast(w)`: the widened read beside a dense one, so the two
        // ordinals must answer different counts from one region.
        let mixed_program = |folded: bool| {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let w = weight(&mut builder);
            let a = builder
                .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
                .unwrap();
            let widened = widen(&mut builder, w);
            let scaled = F32Multiply::apply(&mut builder, a, widened).unwrap();
            let root = if folded {
                StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap()
            } else {
                scaled
            };
            builder
                .output(OutputKey::new("result").unwrap(), root)
                .unwrap();
            builder.build().unwrap()
        };

        // `sum(a, axis 1)`: no prologue, so the fold's own contributor read is
        // the one access no read list describes. Nothing widens here, and the
        // row is what keeps that term live.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let a = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let root = StrictSerialF32Sum::apply(&mut builder, a, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        let bare_fold = builder.build().unwrap();

        // `sum(a, axis 2) * broadcast(w)`: the producer folds ordinal `0` at its
        // own twelve-element shape and the epilogue widens ordinal `1` over a
        // four-point domain, so one chain carries both halves.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let a = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2, 3]))
            .unwrap();
        let w = weight(&mut builder);
        let reduced = StrictSerialF32Sum::apply(&mut builder, a, [Axis::new(2)]).unwrap();
        let widened = widen(&mut builder, w);
        let root = F32Multiply::apply(&mut builder, reduced, widened).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        let widened_epilogue = builder.build().unwrap();

        let arm = |output: &NormalizedOutput| match output {
            NormalizedOutput::SerialSum(_) => "serial-sum",
            NormalizedOutput::Pointwise(_) => "pointwise",
            NormalizedOutput::Contraction(_) => "contraction",
            NormalizedOutput::Epilogue(_) => "epilogue",
            NormalizedOutput::Staged(_) => "staged",
        };
        let rows: [CountRow; 7] = [
            CountRow {
                label: "a sole widened pointwise read",
                arm: "pointwise",
                output: recognize(&sole_widened_read)
                    .expect("a widened read is an elementwise region"),
                domain: Some(4),
                counts: &[Some(2)],
                max: 2,
            },
            CountRow {
                label: "a widened pointwise read beside a dense one",
                arm: "pointwise",
                output: recognize(&mixed_program(false))
                    .expect("a widened read is an elementwise region"),
                domain: Some(4),
                counts: &[Some(2), Some(4)],
                max: 4,
            },
            CountRow {
                label: "a widened read in a fold's prologue",
                arm: "serial-sum",
                output: recognize(&mixed_program(true))
                    .expect("a widened prologue read is recognized"),
                domain: Some(4),
                counts: &[Some(2), Some(4)],
                max: 4,
            },
            CountRow {
                label: "a prologue-less fold's own contributor read",
                arm: "serial-sum",
                output: recognize(&bare_fold).expect("a fold over a declared input is recognized"),
                domain: None,
                counts: &[Some(6)],
                max: 6,
            },
            CountRow {
                label: "a widened read in a chain's epilogue",
                arm: "epilogue",
                output: recognize(&widened_epilogue)
                    .expect("a widened epilogue read is recognized"),
                domain: Some(4),
                counts: &[Some(12), Some(2)],
                max: 12,
            },
            CountRow {
                label: "a contraction's two operands",
                arm: "contraction",
                output: recognize(&contraction_program(false))
                    .expect("a binary contraction is recognized"),
                domain: None,
                counts: &[Some(6), Some(12)],
                max: 12,
            },
            CountRow {
                label: "a staged family's operand run",
                arm: "staged",
                output: recognize(&normalization_program(false, 1.0e-6_f32.to_bits()))
                    .expect("a normalization is recognized"),
                domain: None,
                counts: &[Some(4), Some(4)],
                max: 4,
            },
        ];
        let reached: BTreeSet<&str> = rows.iter().map(|row| row.arm).collect();
        assert_eq!(
            reached.len(),
            5,
            "the rows reach {reached:?}, which is not every arm of the accessor",
        );

        for CountRow {
            label,
            arm: expected_arm,
            output,
            domain,
            counts,
            max,
        } in rows
        {
            assert_eq!(
                arm(&output),
                expected_arm,
                "{label}: the fixture recognized as another arm, so the row proves nothing about \
                 the one it names",
            );
            if let Some(domain) = domain {
                assert!(
                    counts.iter().any(|count| *count != Some(domain)),
                    "{label}: every count equals the domain of {domain}, so this row cannot \
                     observe the difference it exists for",
                );
            }
            for (ordinal, expected) in counts.iter().enumerate() {
                let ordinal = u32::try_from(ordinal).expect("the fixtures declare few inputs");
                assert_eq!(
                    output.input_elements_at(InputOrdinal::new(ordinal)),
                    *expected,
                    "{label}: ordinal {ordinal} is not the declared tensor's own count",
                );
                assert_eq!(
                    output.reads_declared_input(ordinal),
                    expected.is_some(),
                    "{label}: ordinal {ordinal} — the predicate and the count disagree about what \
                     this walk reads",
                );
            }
            let past = u32::try_from(counts.len()).expect("the fixtures declare few inputs");
            assert_eq!(
                output.input_elements_at(InputOrdinal::new(past)),
                None,
                "{label}: an ordinal past the declaration produced a count",
            );
            assert_eq!(
                output.max_input_elements(),
                max,
                "{label}: the largest declared input count this output reads",
            );
        }
    }

    /// The two shapes a staged operand still refuses, each by its own name.
    ///
    /// **Both are asserted rather than left implicit, because one admitted shape
    /// reads as general support unless its boundary is stated.** Their admitted
    /// neighbour is
    /// [`a_staged_family_reading_a_materialized_intermediate_is_recognized`]'s
    /// program, which differs from each by exactly the property named:
    ///
    /// - *A second operand supplied by a materialization edge.*
    ///   `rms_norm(m, m)` gives one occurrence two `TensorRole::Intermediate`
    ///   reads, and that role carries no ordinal, so nothing says which edge each
    ///   binds. `staged-operand-conflict`.
    /// - *An occurrence already at the far side of an edge reading its own.*
    ///   `rms_norm(matmul(a, b), w) * w` makes the normalization an epilogue
    ///   chain's producer, so admitting its operand edge would be a recognized
    ///   chain two materialization boundaries deep. `staged-operand-depth`, the
    ///   depth rule's one guard, stated at [`StagedOperandAdmission`].
    ///
    /// Each was watched failing before it was restored: with the
    /// `producer.is_some()` guard deleted the first program is recognized with
    /// two `BoundaryRead::Staged` operands and one producer, and with the
    /// `StagedOperandAdmission::NoEdge` guard deleted the second is recognized as
    /// a two-boundary chain — both admissions no region vocabulary here can
    /// spell.
    ///
    /// **The second perturbation was rerun on 2026-08-08 and its cost measured**,
    /// because "no region vocabulary can spell it" is a claim about a stage this
    /// assertion cannot see. Handing `recognize_epilogue_producer`'s call site
    /// `OneEdge` recognizes the program as
    /// `Epilogue { producer: Staged { producer: Some(Contraction), operand_reads:
    /// [Staged, Input(2)] } }` — a well-formed nesting — and this row is the
    /// *only* one of the crate's 784 tests that moves. End to end the program
    /// then refuses `NoFeasiblePlan` rather than compiling.
    /// `crates/tiler-compiler/tests/recognized_chain_depth_boundary.rs` holds
    /// that measurement and the trigger that reopens it.
    #[test]
    fn a_staged_operand_still_refuses_a_second_edge_and_a_deeper_chain() {
        assert_eq!(
            recognize(&contraction_fed_normalization(false, true)).unwrap_err(),
            "staged-operand-conflict",
        );
        assert_eq!(
            recognize(&contraction_fed_normalization(true, false)).unwrap_err(),
            "staged-operand-depth",
        );
    }

    /// The staged subject separates an edge-fed operand from a declared one, and
    /// separates a carried producer from an absent one.
    ///
    /// **Two claims, each isolated, because either alone would pass on the
    /// other's evidence.** The occurrence's own operand run and the producer are
    /// two facts the `staged-family.v2` arm writes, and a forgery that moved both
    /// at once would be separated by whichever the encoder still carried — the
    /// exact way a check stops exercising its shape while staying green.
    ///
    /// Each forgery therefore moves exactly one field of the *same* recognized
    /// value, leaving every operand shape, element count, key, member ordinal and
    /// published shape identical. Neither forgery is a value the recognizer
    /// produces; that is what makes them drivable at all, and it is the same
    /// device the request-subject mutation tests above use.
    ///
    /// Watched failing under two deliberate perturbations, one per claim:
    /// dropping the role tag from `encode_output_subject`'s staged arm makes the
    /// first pair equal, and dropping its producer run makes the second pair
    /// equal.
    #[test]
    fn a_staged_subject_separates_an_edge_fed_operand_from_a_declared_one() {
        let program = contraction_fed_normalization(false, false);
        let normalized = select_supported_strategy(&program, &laws_of(&program)).unwrap();
        let [recognized] = normalized.outputs() else {
            panic!("the fixture declares one output");
        };
        let encoded = |output: &NormalizedOutput| {
            let mut bytes = Vec::new();
            encode_output_subject(&mut bytes, &output_subject(output));
            bytes
        };
        let forge = |edit: fn(&mut NormalizedStaged)| {
            let mut forged = recognized.clone();
            let NormalizedOutput::Staged(staged) = &mut forged else {
                panic!("a normalization output recognizes as a staged family")
            };
            edit(staged);
            encoded(&forged)
        };
        assert_ne!(
            encoded(recognized),
            forge(|staged| staged.operand_reads[0] = BoundaryRead::Input(0)),
            "the operand's boundary role is part of what the occurrence reads",
        );
        assert_ne!(
            encoded(recognized),
            forge(|staged| staged.producer = None),
            "the shape writing the edge is part of what this partition computes",
        );
    }

    /// Two occurrences differing only in `eps` bind different request subjects.
    ///
    /// The attribute record is what separates them: both programs declare the
    /// same keys, the same shapes, the same operand map, the same member, and
    /// the same element counts, so a staged subject arm that omitted the record
    /// would give two different normalizations one identity. Watched failing
    /// under a deliberate perturbation: dropping the attribute run from
    /// `encode_output_subject`'s staged arm makes the two subjects equal.
    #[test]
    fn a_staged_subject_separates_two_occurrences_differing_only_in_eps() {
        let subject_bytes = |eps_bits: u32| {
            let program = normalization_program(false, eps_bits);
            let normalized = select_supported_strategy(&program, &laws_of(&program)).unwrap();
            let mut bytes = Vec::new();
            for output in normalized.outputs() {
                encode_output_subject(&mut bytes, &output_subject(output));
            }
            bytes
        };
        let first = subject_bytes(1.0e-6_f32.to_bits());
        let second = subject_bytes(1.0e-5_f32.to_bits());
        assert_ne!(
            first, second,
            "the occurrence's eps payload is part of what the staged stage computes"
        );
    }

    /// Builds the five-node `input * scale + bias` expression a forgery swaps in.
    fn affine_expression(scale_bits: u32, bias_bits: u32) -> PointwiseF32Expression {
        let mut expression = PointwiseF32ExpressionBuilder::new();
        let input = expression.input(InputOrdinal::FIRST).unwrap();
        let scale = expression.constant(scale_bits).unwrap();
        let product = expression.multiply(input, scale).unwrap();
        let bias = expression.constant(bias_bits).unwrap();
        let root = expression.add(product, bias).unwrap();
        expression.build(root).unwrap()
    }

    /// The realization-law authority recognition consults, for one fixture.
    ///
    /// Paired with the governed scalar profile, which is what the compile path
    /// pairs. A fixture that registers its own operations has a semantic
    /// authority the governed scalars were never frozen over, so it is paired
    /// with the empty scalar registry built over *its* semantic authority
    /// instead — recognition asks this registry one question, whether a family's
    /// registered law realizes a region sequence, and that reads the semantic
    /// law rows alone.
    fn laws_of(program: &SemanticProgram) -> FrozenIndexRealizationLawRegistry {
        let semantic = program.semantic_registry().clone();
        FrozenIndexRealizationLawRegistry::from_semantic(
            semantic.clone(),
            governed_scalars().expect("the governed scalar profile is coherent"),
        )
        .or_else(|_| {
            FrozenIndexRealizationLawRegistry::from_semantic(
                semantic.clone(),
                tiler_ir::index::ScalarRegistryBuilder::new(semantic).freeze(),
            )
        })
        .expect("a law authority over the fixture's own semantic authority coheres")
    }

    /// A gather stops first at exact target dispatch, then at arithmetic recognition.
    ///
    /// The second compile changes only the target's exact U32 dispatch fact. It
    /// keeps the semantic program byte-for-byte identical, so advancing from the
    /// target-local `DTypeNotDispatchable` refusal to `dtype-recognized` pins the
    /// request boundary's ordered diagnostic layers without granting Gather a
    /// production target claim or a planning route.
    ///
    /// Watched failing under a deliberate subject perturbation: removing the
    /// U32 row from `governed_with_gather_index_dispatch_for_test` makes the
    /// second compile return the same target-local refusal as the first.
    #[test]
    fn a_governed_gather_refuses_at_dispatch_before_arithmetic_recognition() {
        let program = gather_program();
        let product = crate::pipeline::compile(CompilationRequest::governed(&program))
            .expect("a target-local refusal is an ordinary compilation product");
        let [outcome] = product.targets.as_slice() else {
            panic!("the governed request carries one target outcome");
        };
        assert_eq!(
            outcome.failure(),
            Some(&crate::pipeline::CompileError::NoFeasiblePlan(
                crate::pipeline::NoFeasiblePlanError::Request(RequestError::DTypeNotDispatchable {
                    target_profile: TargetProfile::governed().profile_key().clone(),
                    resolved_type: Box::new(gather_index_resolved_type()),
                    disposition: DTypeDispatchRefusalDisposition::Unknown,
                })
            )),
            "the governed target answers for the exact U32 index type before recognition",
        );

        let mut widened = CompilationRequest::governed(&program);
        widened.target_profiles =
            vec![TargetProfile::governed_with_gather_index_dispatch_for_test()];
        match crate::pipeline::compile(widened) {
            Err(error) => assert_eq!(
                error,
                crate::pipeline::CompileError::UnsupportedCapability(
                    RequestError::UnsupportedCapability {
                        phase: "strategy",
                        rule: "dtype-recognized",
                    }
                ),
                "an exact U32 dispatch fact advances the same program to arithmetic recognition",
            ),
            Ok(product) => {
                let [outcome] = product.targets.as_slice() else {
                    panic!("the widened request carries one target outcome");
                };
                panic!(
                    "the exact U32 dispatch row did not advance recognition: {:?}",
                    outcome.failure()
                );
            }
        }
    }

    /// The real output recognizer independently refuses Gather under `operation-set`.
    ///
    /// Supplying F32 deliberately bypasses whole-program arithmetic recognition;
    /// the real realization-law authority and output walk then reach the later
    /// operation-family boundary rather than an arbitrary test stub.
    ///
    /// Watched failing under a deliberate subject perturbation: classifying the
    /// Gather key in `elementwise_family` advances this walk to its attribute
    /// rule, so this exact `operation-set` expectation changes.
    #[test]
    fn gather_is_absent_from_the_real_request_recognition_operation_set() {
        let program = gather_program();
        assert_eq!(
            recognize_program_outputs(&program, &laws_of(&program), ArithmeticType::F32),
            Err(RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "operation-set",
            }),
        );
    }

    /// Recognizes one program through the whole boundary, or reports the rule.
    ///
    /// Answers with the sole recognized output, because every fixture reaching
    /// it declares one; [`recognize_outputs`] is the multi-output form.
    fn recognize(program: &SemanticProgram) -> Result<NormalizedOutput, &'static str> {
        strategy_rule(select_supported_strategy(program, &laws_of(program))).map(|recognized| {
            let [output] = recognized.outputs() else {
                panic!("the fixture declares one output");
            };
            output.clone()
        })
    }

    /// Recognizes one program's ordered named outputs, or reports the rule.
    ///
    /// Drives [`recognize_program_outputs`] directly rather than through
    /// [`select_supported_strategy`], so a refusal this helper returns is one
    /// the walks themselves produced. The two program-wide properties the
    /// boundary checks before them are asserted rather than reported, which is
    /// what makes that attribution exact.
    fn recognize_outputs(program: &SemanticProgram) -> Result<NormalizedProgram, &'static str> {
        assert_ne!(program.input_count(), 0, "the fixture declares an input");
        assert!(
            program
                .values()
                .all(|value| value.resolved_type() == &F32::resolved_type()),
            "the fixture is f32 throughout",
        );
        strategy_rule(recognize_program_outputs(
            program,
            &laws_of(program),
            ArithmeticType::F32,
        ))
    }

    /// Reduces one recognition outcome to the strategy rule it refused under.
    fn strategy_rule(
        outcome: Result<NormalizedProgram, RequestError>,
    ) -> Result<NormalizedProgram, &'static str> {
        outcome.map_err(|error| match error {
            RequestError::UnsupportedCapability {
                phase: "strategy",
                rule,
            } => rule,
            other => panic!("recognition refuses under the strategy phase, got {other:?}"),
        })
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
            let attributes = request.attributes();
            match self {
                Self::Constant => {
                    outputs.try_push(ValueFact::new(F32::resolved_type(), Shape::new([])))
                }
                Self::Binary => {
                    let left = request.static_operand_shape(0)?;
                    let right = request.static_operand_shape(1)?;
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
                        request.static_operand_shape(0)?.without_axes(&axes),
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

    fn request_with_matching_empty_capabilities(
        program: &SemanticProgram,
    ) -> CompilationRequest<'_> {
        let scalars =
            tiler_ir::index::ScalarRegistryBuilder::new(program.semantic_registry().clone())
                .freeze();
        let lowering = crate::capability::LoweringCapabilityRegistryBuilder::new(
            program.semantic_registry().clone(),
            scalars.clone(),
        )
        .unwrap()
        .freeze();
        let mut request = CompilationRequest::governed(program);
        request.capabilities = CompilerCapabilitySnapshot::new(lowering, scalars);
        request
    }

    #[test]
    fn governed_request_selects_the_supported_serial_sum_strategy() {
        let program = program();
        let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
        let [recognized] = verified.normalized.outputs() else {
            panic!("the fixture declares one output");
        };
        let normalized = recognized.serial_sum();
        assert_eq!(normalized.input_shape, Shape::from_dims([2, 3]));
        assert_eq!(normalized.output_shape, Shape::from_dims([2]));
        assert_eq!(normalized.reduction_axes, [Axis::new(1)]);
        assert_eq!(normalized.input_elements, 6);
        assert_eq!(normalized.output_elements, 2);
        assert_eq!(normalized.input_keys, [InputKey::new("input").unwrap()]);
        // The prologue is the recognized expression, not two constants: it is
        // `input * 2.0 + 1.0` in the physical node vocabulary, and the affine
        // pair the fused region needs is recovered from it rather than stored
        // beside it.
        let prologue = normalized
            .prologue
            .as_ref()
            .expect("a fold over a computed contributor has a prologue");
        assert_eq!(prologue.input_count(), 1);
        assert!(matches!(
            prologue.nodes(),
            [
                PointwiseF32Node::Input { .. },
                PointwiseF32Node::Constant { bits: scale },
                PointwiseF32Node::Multiply { .. },
                PointwiseF32Node::Constant { bits: bias },
                PointwiseF32Node::Add { .. },
            ] if *scale == 2.0_f32.to_bits() && *bias == 1.0_f32.to_bits()
        ));
        assert_eq!(
            verified
                .target_slots
                .iter()
                .map(|slot| &slot.target_profile)
                .collect::<Vec<_>>(),
            [&TargetProfile::governed()]
        );
    }

    /// The composed program: a multi-input elementwise expression feeding a
    /// strict serial reduction.
    ///
    /// **This is the shape no normalization matched.** The superseded serial-sum
    /// template demanded exactly one declared input and the exact four- or
    /// five-operation `x * scale + bias` prologue; the superseded pointwise
    /// template refused anything containing a reduction. `sum((a * b) + c)` over
    /// three declared inputs is neither, and it is admitted here on the strength
    /// of its occurrences: two recognized elementwise families composing into one
    /// expression, feeding one recognized reduction.
    #[test]
    fn a_multi_input_elementwise_expression_feeding_a_reduction_is_recognized() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let inputs: Vec<_> = ["a", "b", "c"]
            .into_iter()
            .map(|key| {
                builder
                    .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([2, 3]))
                    .unwrap()
            })
            .collect();
        let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
        let biased = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, biased, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        let program = builder.build().unwrap();

        let NormalizedOutput::SerialSum(recognized) =
            recognize(&program).expect("the composed program is recognized")
        else {
            panic!("a program whose output is a reduction recognizes as one");
        };
        assert_eq!(recognized.input_keys.len(), 3);
        assert_eq!(recognized.input_shape, Shape::from_dims([2, 3]));
        assert_eq!(recognized.output_shape, Shape::from_dims([2]));
        assert_eq!(
            recognized
                .prologue
                .as_ref()
                .expect("a fold over a computed contributor has a prologue")
                .input_count(),
            3,
            "one leaf per declared input tensor",
        );
        // Three elementwise occurrences in the prologue is exactly two — the
        // multiply and the add — with no constant, and the reduction is the
        // third occurrence of the program.
        assert_eq!(recognized.members.pointwise().len(), 2);
        assert_eq!(recognized.members.all().len(), program.operation_count());
        // No fused spelling exists: `FusedMultiplyAddSerialSum` applies one
        // scalar constant and one scalar bias, and this prologue applies neither.
        let verified = verify_planned_request(CompilationRequest::governed(&program))
            .unwrap()
            .for_target(0)
            .unwrap();
        assert_eq!(
            crate::physical::fused_prologue_constants(verified.sole_output()),
            None
        );
    }

    /// A reduction over a declared input is recognized with no prologue.
    ///
    /// `sum(x)` is the simplest fold there is, and it used to be the one shape
    /// this recognizer refused for a wall *below* it: `verify_access_and_semantics`
    /// required a `ScalarProgram::StrictSerialSum` region's contributor access to
    /// read `TensorRole::Intermediate`, so a region folding the input directly was
    /// rejected as malformed. That arm now admits the fold's declared contributor
    /// domain, and the absence of a prologue is recorded as `None` rather than as
    /// an identity expression — which is what keeps a cover from spelling the copy
    /// kernel the refusal existed to avoid.
    ///
    /// Its neighbour is the same fold with one elementwise occurrence between the
    /// input and the sum, asserted beside it so the `None` is attributable to the
    /// missing prologue rather than to the fold.
    #[test]
    fn a_reduction_over_a_declared_input_is_recognized_with_no_prologue() {
        let fold = |prologue: bool| {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let input = builder
                .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
                .unwrap();
            let contributor = if prologue {
                let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
                F32Multiply::apply(&mut builder, input, scale).unwrap()
            } else {
                input
            };
            let sum = StrictSerialF32Sum::apply(&mut builder, contributor, [Axis::new(1)]).unwrap();
            builder
                .output(OutputKey::new("result").unwrap(), sum)
                .unwrap();
            builder.build().unwrap()
        };
        let bare = fold(false);
        assert_eq!(bare.operation_count(), 1);
        let Ok(NormalizedOutput::SerialSum(recognized)) = recognize(&bare) else {
            panic!("a fold over a declared input is recognized as a serial sum");
        };
        assert_eq!(recognized.prologue, None);
        assert_eq!(recognized.prologue_reads, []);
        // One part, not two: the empty prologue part is not a member set a cover
        // region may match, which is what `prologue_members` states.
        assert_eq!(recognized.prologue_members(), None);
        assert_eq!(recognized.members.reduction().len(), 1);

        let neighbour = fold(true);
        let Ok(NormalizedOutput::SerialSum(recognized)) = recognize(&neighbour) else {
            panic!("a fold over a computed contributor is recognized as a serial sum");
        };
        assert!(recognized.prologue.is_some());
        assert_eq!(recognized.prologue_members().map(<[_]>::len), Some(2));
    }

    /// Elementwise recognition follows the graph, not a taught depth or arity.
    ///
    /// Each shape below was refused by the superseded template, and each was
    /// refused for the *leaf count* rather than for anything about what it
    /// computes: the old recognizer admitted exactly two operations over exactly
    /// three leaves in one of two associations.
    #[test]
    fn elementwise_recognition_admits_depth_sharing_and_multiple_inputs() {
        // Three declared inputs and a mixed multiply-then-add chain.
        let three = three_input_elementwise();
        let NormalizedOutput::Pointwise(recognized) =
            recognize(&three).expect("a three-input expression is recognized")
        else {
            panic!("an elementwise output recognizes as an elementwise program");
        };
        assert_eq!(
            recognized.input_keys,
            [
                InputKey::new("a").unwrap(),
                InputKey::new("b").unwrap(),
                InputKey::new("c").unwrap(),
            ],
        );
        assert_eq!(recognized.expression.f32().input_count(), 3);
        assert_eq!(recognized.members.len(), three.operation_count());

        // A four-deep chain: `((a * 2.0) + b) * ((a * 2.0) + b)`, whose shared
        // subexpression is one node rather than two. Depth and sharing are both
        // beyond what a three-leaf template could spell.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let first = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let second = builder
            .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let scaled = F32Multiply::apply(&mut builder, first, scale).unwrap();
        let shifted = F32Add::apply(&mut builder, scaled, second).unwrap();
        let root = F32Multiply::apply(&mut builder, shifted, shifted).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        let deep = builder.build().unwrap();
        let NormalizedOutput::Pointwise(recognized) =
            recognize(&deep).expect("a deep shared expression is recognized")
        else {
            panic!("an elementwise output recognizes as an elementwise program");
        };
        assert_eq!(recognized.expression.f32().input_count(), 2);
        assert_eq!(recognized.members.len(), deep.operation_count());
        assert_eq!(
            recognized.expression.f32().nodes().len(),
            6,
            "the shared `(a * 2.0) + b` is one node, not two",
        );

        // One input read at two leaves, which binds one read access.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let constant = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let doubled = F32Add::apply(&mut builder, input, input).unwrap();
        let root = F32Add::apply(&mut builder, doubled, constant).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        let repeated = builder.build().unwrap();
        let NormalizedOutput::Pointwise(recognized) =
            recognize(&repeated).expect("a repeated read is recognized")
        else {
            panic!("an elementwise output recognizes as an elementwise program");
        };
        assert_eq!(recognized.expression.f32().input_count(), 1);
        assert_eq!(recognized.input_keys.len(), 1);
    }

    /// The recognizer admits a `bf16` program and mints its own vocabulary.
    ///
    /// **The wall this replaces refused every program carrying a non-`f32`
    /// value under `dtype-f32`, before a subject was normalized**, so no
    /// `NormalizedProgram` for one could exist and nothing downstream could be
    /// asked about it. Recognition now derives the program's one arithmetic type
    /// and walks it with the same authority the `f32` walk uses — the same
    /// classification, the same shape checks, the same leaf ordering — and only
    /// the minting differs.
    ///
    /// The expression is asserted whole rather than by node count alone: the
    /// constant leaf carries the *sixteen* declared payload bits, which is the
    /// one place a widened `f32` reading would show up as a number no `bf16`
    /// program stated.
    #[test]
    fn a_bf16_program_is_recognized_in_its_own_expression_vocabulary() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        // `3.0` in bf16, whose sixteen bits are not the low half of any binary32
        // pattern this walk could have read instead.
        let scale = Bf16Constant::apply(&mut builder, 0x4040).unwrap();
        let scaled = Bf16Multiply::apply(&mut builder, input, scale).unwrap();
        let bias = Bf16Constant::apply(&mut builder, 0x8000).unwrap();
        let root = Bf16Add::apply(&mut builder, scaled, bias).unwrap();
        builder
            .output(OutputKey::new("out").unwrap(), root)
            .unwrap();
        let program = builder.build().unwrap();

        let NormalizedOutput::Pointwise(recognized) =
            recognize(&program).expect("a bf16 elementwise program is recognized")
        else {
            panic!("an elementwise output recognizes as an elementwise program");
        };
        let expression = recognized.expression.bf16();
        assert_eq!(expression.input_count(), 1);
        // The population, counted: every occurrence the program declares is
        // claimed, so an assertion about the expression is an assertion about
        // the whole program rather than about a prefix of it.
        assert_eq!(recognized.members.len(), program.operation_count());
        assert_eq!(
            expression.nodes().len(),
            5,
            "one input leaf, two constants, the multiply, and the add",
        );
        let constants: Vec<u16> = expression
            .nodes()
            .iter()
            .filter_map(|node| match node {
                tiler_ir::schedule::PointwiseBf16Node::Constant { bits } => Some(*bits),
                _ => None,
            })
            .collect();
        assert_eq!(
            constants,
            [0x4040, 0x8000],
            "the constants are the declared bf16 payloads, not a widened reading",
        );
        assert_eq!(
            recognized.reads,
            vec![(0, LogicalAccess::LinearIdentity)],
            "one dense read of the one declared input",
        );
    }

    /// Constant occurrence identity reaches the initial recognizer and mint.
    ///
    /// Each pair computes `x * 2 + 2` in its own arithmetic. The only authored
    /// difference is whether the add reuses the exact constant value consumed by
    /// the multiply or consumes a second constant occurrence with the same
    /// payload. Semantic construction, elementwise planning, and minting all
    /// preserve that difference for both arithmetic widths the compiler
    /// currently recognizes. This drives `recognize` directly: ordinary
    /// compilation normalizes equal pure constants before candidate readmission,
    /// as the normalization and pipeline regressions assert separately.
    ///
    #[test]
    fn equal_constant_occurrences_remain_distinct_through_initial_recognition() {
        fn f32_program(repeat_occurrence: bool) -> SemanticProgram {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let input = builder
                .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
                .unwrap();
            let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
            let scaled = F32Multiply::apply(&mut builder, input, two).unwrap();
            let addend = if repeat_occurrence {
                F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap()
            } else {
                two
            };
            let root = F32Add::apply(&mut builder, scaled, addend).unwrap();
            builder
                .output(OutputKey::new("out").unwrap(), root)
                .unwrap();
            builder.build().unwrap()
        }

        fn bf16_program(repeat_occurrence: bool) -> SemanticProgram {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let input = builder
                .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
                .unwrap();
            let two = Bf16Constant::apply(&mut builder, 0x4000).unwrap();
            let scaled = Bf16Multiply::apply(&mut builder, input, two).unwrap();
            let addend = if repeat_occurrence {
                Bf16Constant::apply(&mut builder, 0x4000).unwrap()
            } else {
                two
            };
            let root = Bf16Add::apply(&mut builder, scaled, addend).unwrap();
            builder
                .output(OutputKey::new("out").unwrap(), root)
                .unwrap();
            builder.build().unwrap()
        }

        fn recognized_pointwise(program: &SemanticProgram) -> RecognizedPointwise {
            let NormalizedOutput::Pointwise(recognized) =
                recognize(program).expect("the compiler recognizes the elementwise program")
            else {
                panic!("an elementwise output recognizes as an elementwise program");
            };
            assert_eq!(
                recognized.members.len(),
                program.operation_count(),
                "the expression must cover every semantic occurrence",
            );
            recognized.expression
        }

        let shared_f32 = f32_program(false);
        let repeated_f32 = f32_program(true);
        assert_eq!(shared_f32.operation_count(), 3);
        assert_eq!(repeated_f32.operation_count(), 4);
        let RecognizedPointwise::F32(shared_f32_expression) = recognized_pointwise(&shared_f32)
        else {
            panic!("an f32 program must mint the f32 pointwise vocabulary");
        };
        let RecognizedPointwise::F32(repeated_f32_expression) = recognized_pointwise(&repeated_f32)
        else {
            panic!("an f32 program must mint the f32 pointwise vocabulary");
        };
        assert_eq!(shared_f32_expression.nodes().len(), 4);
        assert_eq!(repeated_f32_expression.nodes().len(), 5);
        assert_eq!(
            shared_f32_expression
                .nodes()
                .iter()
                .filter_map(|node| match node {
                    PointwiseF32Node::Constant { bits } => Some(*bits),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [2.0_f32.to_bits()],
        );
        assert_eq!(
            repeated_f32_expression
                .nodes()
                .iter()
                .filter_map(|node| match node {
                    PointwiseF32Node::Constant { bits } => Some(*bits),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [2.0_f32.to_bits(), 2.0_f32.to_bits()],
            "the extra node is a second equal-payload constant occurrence",
        );
        assert_ne!(shared_f32_expression, repeated_f32_expression);

        let shared_bf16 = bf16_program(false);
        let repeated_bf16 = bf16_program(true);
        assert_eq!(shared_bf16.operation_count(), 3);
        assert_eq!(repeated_bf16.operation_count(), 4);
        let RecognizedPointwise::Bf16(shared_bf16_expression) = recognized_pointwise(&shared_bf16)
        else {
            panic!("a bf16 program must mint the bf16 pointwise vocabulary");
        };
        let RecognizedPointwise::Bf16(repeated_bf16_expression) =
            recognized_pointwise(&repeated_bf16)
        else {
            panic!("a bf16 program must mint the bf16 pointwise vocabulary");
        };
        assert_eq!(shared_bf16_expression.nodes().len(), 4);
        assert_eq!(repeated_bf16_expression.nodes().len(), 5);
        assert_eq!(
            shared_bf16_expression
                .nodes()
                .iter()
                .filter_map(|node| match node {
                    tiler_ir::schedule::PointwiseBf16Node::Constant { bits } => Some(*bits),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [0x4000],
        );
        assert_eq!(
            repeated_bf16_expression
                .nodes()
                .iter()
                .filter_map(|node| match node {
                    tiler_ir::schedule::PointwiseBf16Node::Constant { bits } => Some(*bits),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            [0x4000, 0x4000],
            "the extra node is a second equal-payload constant occurrence",
        );
        assert_ne!(shared_bf16_expression, repeated_bf16_expression);

        let VerifiedRequest::Refused(refusals) =
            verify_request(CompilationRequest::governed_under(
                &repeated_bf16,
                crate::session::NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16.resolve(),
            ))
            .expect("the governed target refusal is a target-local outcome")
        else {
            panic!("the governed target declares no bf16 dispatch row");
        };
        let [refusal] = refusals.as_slice() else {
            panic!("the governed request carries one target and one refusal");
        };
        let VerifiedTargetResolution::Rejected(refusal) = &refusal.resolution else {
            panic!("the governed target slot is refused");
        };
        assert_eq!(
            *refusal,
            RequestError::DTypeNotDispatchable {
                target_profile: TargetProfile::governed().profile_key().clone(),
                resolved_type: Box::new(Bf16::resolved_type()),
                disposition: DTypeDispatchRefusalDisposition::Unknown,
            },
            "the governed request stops at dtype dispatch before target-specific recognition",
        );
    }

    /// The two refusals the `dtype-f32` rule split into name different findings.
    ///
    /// **`dtype-recognized` and `dtype-uniform` are not one rule renamed.** The
    /// first says this build states no per-point vocabulary for a width the
    /// program uses; the second says the program uses two widths at once, which
    /// no single scheduled region can carry however well each width is
    /// supported. Each is exercised by a program that fails only it, and the
    /// admitted neighbours above are what keep the pair from passing for a
    /// recognizer that refused everything.
    #[test]
    fn a_mixed_width_program_and_an_unspelled_width_refuse_by_different_names() {
        // Two recognized widths in one program: the quantized carrier is `bf16`
        // and its declared sibling is `f32`, so no one arithmetic governs it.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let narrow = builder
            .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let wide = builder
            .input::<F32>(InputKey::new("y").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let narrow_sum = Bf16Add::apply(&mut builder, narrow, narrow).unwrap();
        let wide_sum = F32Add::apply(&mut builder, wide, wide).unwrap();
        builder
            .output(OutputKey::new("narrow").unwrap(), narrow_sum)
            .unwrap();
        builder
            .output(OutputKey::new("wide").unwrap(), wide_sum)
            .unwrap();
        let mixed = builder.build().unwrap();
        assert_eq!(
            recognize(&mixed),
            Err("dtype-uniform"),
            "a program of two widths has no single scalar program",
        );

        // One width this build spells no per-point body in: the strict-affine
        // encoded carrier, a registered value type that names no arithmetic type
        // at all.
        let published = |program: &SemanticProgram| recognize(program);
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let codes = builder
            .input::<tiler_ir::semantic::StrictAffineU4>(
                InputKey::new("codes").unwrap(),
                Shape::from_dims([2, 3]),
            )
            .unwrap();
        builder
            .output(OutputKey::new("out").unwrap(), codes)
            .unwrap();
        let encoded = builder.build().unwrap();
        assert_eq!(
            published(&encoded),
            Err("dtype-recognized"),
            "a value type this build states no per-point vocabulary for is named as such",
        );

        // The neighbour that attributes that refusal to the *width* rather than
        // to the shape: the same program in a recognized width publishes a
        // declared input, which is refused one rule later under `operation-set`.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let value = builder
            .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        builder
            .output(OutputKey::new("out").unwrap(), value)
            .unwrap();
        let published_input = builder.build().unwrap();
        assert_eq!(
            published(&published_input),
            Err("operation-set"),
            "the shape alone refuses under its own rule, so the width is what the U4 program \
             was refused for",
        );
    }

    /// Every refusal names the exact property that was not recognized.
    ///
    /// The table is the ticket's contract: recognition generalizes, admission
    /// does not become silent. Each row is a program the boundary refuses, the
    /// rule it refuses under, and — through the accepted neighbour built beside
    /// it — a demonstration that the rule can say yes as well as no.
    #[test]
    fn every_refusal_names_its_unrecognized_property() {
        let shape = || Shape::from_dims([2, 3]);

        // `input-arity`: an all-constant graph has no output-reachable input,
        // and a frozen program drops the unused declaration. The neighbour is
        // the same expression with one leaf replaced by the declared tensor.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let _input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape())
            .unwrap();
        let first = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let second = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let root = F32Add::apply(&mut builder, first, second).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        let all_constant = builder.build().unwrap();
        assert_eq!(all_constant.input_count(), 0);
        assert_eq!(recognize(&all_constant).unwrap_err(), "input-arity");

        // `output-partition-overlap`: two named outputs one walk would have to
        // publish, because the second names a value the first's walk consumes.
        // The neighbour is the same graph naming only the root, which recognizes
        // — so the rule reads the *sharing* rather than the second output. This
        // row replaced an `output-arity` row: the arity guard is gone, and what
        // refuses this program is the partition obligation it actually violates.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape())
            .unwrap();
        let constant = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let scaled = F32Multiply::apply(&mut builder, input, constant).unwrap();
        let root = F32Add::apply(&mut builder, scaled, constant).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder
            .output(OutputKey::new("partial").unwrap(), scaled)
            .unwrap();
        let two_outputs = builder.build().unwrap();
        assert_eq!(two_outputs.output_count(), 2);
        assert_eq!(
            recognize(&two_outputs).unwrap_err(),
            "output-partition-overlap",
        );

        // **Admitted, and this row is the one the structural widening flipped.**
        // A transposition over a declared input becomes the *read map* of the
        // region that consumes it, so `tiler::reindex-f32@1` is recognized
        // rather than refused. The derived relation is asserted rather than
        // merely the admission: a recognizer that admitted the family and bound
        // a dense read would compile the wrong tensor, which is precisely the
        // failure a bare `is_ok()` here would not see.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape())
            .unwrap();
        let permuted = tiler_ir::semantic::F32Reindex::apply(
            &mut builder,
            &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
                .expect("a two-axis transposition is an admitted form"),
            input,
        )
        .expect("the standard registry admits the reindex family");
        builder
            .output(OutputKey::new("result").unwrap(), permuted)
            .unwrap();
        let structural = builder.build().unwrap();
        let NormalizedOutput::Pointwise(recognized) =
            recognize(&structural).expect("a transposition of a declared input is a mapped read")
        else {
            panic!("a reindex over a declared input is an elementwise region");
        };
        // `shape()` is `[2, 3]`, so the transposed result is `[3, 2]` with
        // suffix products `[2, 1]`. Operand axis 1 takes result axis 0's window
        // and operand axis 0 takes result axis 1's, which is the transposition
        // written as a decode per operand axis.
        assert_eq!(
            recognized.reads,
            vec![(
                0,
                LogicalAccess::ReindexBijection {
                    operand_shape: Shape::from_dims([2, 3]),
                    result_shape: Shape::from_dims([3, 2]),
                    axes: vec![AxisDecode::read(1, 2), AxisDecode::read(2, 3)],
                },
            )],
        );

        // `structural-operand`: the family is admitted, and what is refused is a
        // structural occurrence over a *computed* value. The region binds one
        // read per declared input and has no access to bind an intermediate it
        // also produces, so this refuses by name rather than materializing the
        // intermediate — which would add exactly the observable rounding
        // boundary the family's admission excludes. It is the neighbour that
        // keeps the row above attributable: both are reindexes, and only the
        // operand differs.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape())
            .unwrap();
        let doubled = tiler_ir::semantic::F32Silu::apply(&mut builder, input)
            .expect("the standard registry admits the silu family");
        let permuted = tiler_ir::semantic::F32Reindex::apply(
            &mut builder,
            &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
                .expect("a two-axis transposition is an admitted form"),
            doubled,
        )
        .expect("the standard registry admits the reindex family");
        builder
            .output(OutputKey::new("result").unwrap(), permuted)
            .unwrap();
        let computed = builder.build().unwrap();
        assert_eq!(recognize(&computed).unwrap_err(), "structural-operand");

        // **Admitted, and this row moved here from the refusal inventory.** One
        // declared input read *both* densely and through a relation was refused
        // under `structural-access-conflict`, because the region bound one read
        // per declared input and the expression's two `Input { ordinal: 0 }`
        // nodes shared it — so the mapped relation served both leaves and
        // `a * permute(a)` over `[[1, 2], [4, 8]]` compiled to `[1, 16, 4, 64]`,
        // which is `permute(a) * permute(a)`, where the reference evaluator
        // gives `[1, 8, 8, 64]`. The region now binds two reads of ordinal `0`,
        // and the read list is asserted rather than the admission: a recognizer
        // that admitted the program and bound one read would compile exactly the
        // wrong tensor that a bare `is_ok()` would not see.
        //
        // What still refuses is the pair with no canonical order between its two
        // members — two *structural* relations on one input — which is the
        // neighbour that keeps the admission attributable.
        let mixed = |second_dense: bool| {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let a = builder
                .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
                .unwrap();
            let reindex = |builder: &mut SemanticProgramBuilder,
                           form: &tiler_ir::semantic::ReindexForm| {
                tiler_ir::semantic::F32Reindex::apply(builder, form, a)
                    .expect("the standard registry admits the reindex family")
            };
            let transposed = reindex(
                &mut builder,
                &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
                    .expect("a two-axis transposition is an admitted form"),
            );
            let second = if second_dense {
                a
            } else {
                reindex(
                    &mut builder,
                    &tiler_ir::semantic::ReindexForm::reverse_axis(Axis::new(0))
                        .expect("an axis reversal is an admitted form"),
                )
            };
            let root = F32Multiply::apply(&mut builder, second, transposed).unwrap();
            builder
                .output(OutputKey::new("result").unwrap(), root)
                .unwrap();
            builder.build().unwrap()
        };
        let NormalizedOutput::Pointwise(recognized) = recognize(&mixed(true))
            .expect("one declared input may be read densely and through a relation")
        else {
            panic!("an elementwise output recognizes as an elementwise program");
        };
        // The dense read leads and the mapped one follows, which is the pair's
        // canonical order and the only one the region verifier admits.
        assert_eq!(
            recognized.reads,
            vec![
                (0, LogicalAccess::LinearIdentity),
                (
                    0,
                    LogicalAccess::ReindexBijection {
                        operand_shape: Shape::from_dims([2, 2]),
                        result_shape: Shape::from_dims([2, 2]),
                        axes: vec![AxisDecode::read(1, 2), AxisDecode::read(2, 2)],
                    },
                ),
            ],
        );
        assert_eq!(recognized.expression.f32().input_count(), 2);
        assert_eq!(
            recognize(&mixed(false)).unwrap_err(),
            "structural-access-conflict",
        );

        // `structural-access-conflict` again, and this is the *other* half of the
        // widening's boundary: the twice-read tensor is the value an earlier
        // region staged rather than a declared input. What admits the pair above
        // is the ordinal saying which tensor each read binds, and
        // `TensorRole::Intermediate` carries none — so a second staged read has
        // nothing to attribute it to a second materialization edge. Its accepted
        // neighbour is `s * s`, which reads the staged value once and differs by
        // exactly the read that would have no attribution.
        let staged = |mapped: bool| {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let a = builder
                .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
                .unwrap();
            let folded = StrictSerialF32Sum::apply(&mut builder, a, [Axis::new(1)]).unwrap();
            let second = if mapped {
                tiler_ir::semantic::F32Reindex::apply(
                    &mut builder,
                    &tiler_ir::semantic::ReindexForm::reverse_axis(Axis::new(0))
                        .expect("an axis reversal is an admitted form"),
                    folded,
                )
                .expect("the standard registry admits the reindex family")
            } else {
                folded
            };
            let root = F32Multiply::apply(&mut builder, folded, second).unwrap();
            builder
                .output(OutputKey::new("result").unwrap(), root)
                .unwrap();
            builder.build().unwrap()
        };
        assert!(matches!(
            recognize(&staged(false)),
            Ok(NormalizedOutput::Epilogue(_)),
        ));
        assert_eq!(
            recognize(&staged(true)).unwrap_err(),
            "structural-access-conflict",
        );

        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape())
            .unwrap();
        let activated = tiler_ir::semantic::F32Silu::apply(&mut builder, input)
            .expect("the standard registry admits the silu family");
        builder
            .output(OutputKey::new("result").unwrap(), activated)
            .unwrap();
        let unary = builder.build().unwrap();
        let NormalizedOutput::Pointwise(recognized) =
            recognize(&unary).expect("the activation projects into the expression vocabulary")
        else {
            panic!("an elementwise output recognizes as an elementwise program");
        };
        // One occurrence, one declared input read once, and the composition's
        // seven nodes: the projection is the shared body's, not a per-shape one.
        assert_eq!(
            recognized.members,
            vec![SemanticStage::first(SemanticMemberId(0))]
        );
        assert_eq!(recognized.expression.f32().input_count(), 1);
        assert_eq!(recognized.expression.f32().nodes().len(), 7);

        // A contraction with a reachable elementwise epilogue is a *chain*, not
        // a refusal, and the bare contraction beside it is what makes the
        // difference attributable: the two programs differ by exactly the
        // epilogue, and the recognized shape differs by exactly the consumer
        // region.
        let contraction = contraction_program(false);
        assert!(matches!(
            recognize(&contraction),
            Ok(NormalizedOutput::Contraction(_))
        ));
        let with_epilogue = contraction_program(true);
        let Ok(NormalizedOutput::Epilogue(chain)) = recognize(&with_epilogue) else {
            panic!("an elementwise expression over a contraction result is a chain");
        };
        assert!(matches!(*chain.producer, NormalizedOutput::Contraction(_)));
        assert_eq!(
            chain.reads.len(),
            1,
            "the epilogue reads only the staged value"
        );
        assert_eq!(chain.reads[0].0, BoundaryRead::Staged);

        // `reduction-contributor-materialization`, from the one side the
        // discovery deliberately does not open: a fold whose *contributors*
        // cross a materialization boundary. The producer is recognized; what
        // is missing is a place on `NormalizedSerialSum` to retain it.
        //
        // **Which of the three folded-value walls refuses it was measured, not
        // read off the shape.** It is `From<ElementwiseRefusal>`'s flattening:
        // `NormalizedSerialSum` carries no producer field, so the discovery is
        // discarded before any admission runs.
        // `StagedOperandAdmission`'s `staged-operand-depth` guard is never
        // consulted for this program, and `plan_elementwise`'s
        // `leaves.staged.is_none()` condition is *true* here, so that guard does
        // not fire either. Watched on 2026-08-08: renaming this arm's rule to a
        // probe string made this row report the probe, while renaming the
        // `leaves.staged.is_none()` arm's left it reporting the contributor
        // materialization rule.
        // `name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set`
        // owns that stable rule name.
        //
        // The accepted neighbour is the same fold over the same scaling of the
        // *declared input*, so the difference between them is exactly where the
        // scaled value comes from.
        let folded_prologue = |nested: bool| {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let input = builder
                .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 4]))
                .unwrap();
            let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
            let (contributors, axis) = if nested {
                let inner = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
                (
                    F32Multiply::apply(&mut builder, inner, scale).unwrap(),
                    Axis::new(0),
                )
            } else {
                (
                    F32Multiply::apply(&mut builder, input, scale).unwrap(),
                    Axis::new(1),
                )
            };
            let outer = StrictSerialF32Sum::apply(&mut builder, contributors, [axis]).unwrap();
            builder
                .output(OutputKey::new("result").unwrap(), outer)
                .unwrap();
            builder.build().unwrap()
        };
        assert!(matches!(
            recognize(&folded_prologue(false)),
            Ok(NormalizedOutput::SerialSum(_)),
        ));
        assert_eq!(
            recognize(&folded_prologue(true)).unwrap_err(),
            "reduction-contributor-materialization"
        );

        // The key names the failed contributor relation rather than the
        // producer family. A staged family reaches the same retained fact as a
        // nested reduction: it is recognized as a materializing producer, and
        // the serial-sum normal form has nowhere to bind that producer.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let value = builder
            .input::<F32>(InputKey::new("value").unwrap(), Shape::from_dims([2, 4]))
            .unwrap();
        let weight = builder
            .input::<F32>(InputKey::new("weight").unwrap(), Shape::from_dims([2, 4]))
            .unwrap();
        let normalized = F32RmsNorm::apply(
            &mut builder,
            value,
            weight,
            Axis::new(1),
            1.0e-6_f32.to_bits(),
        )
        .unwrap();
        let reduced = StrictSerialF32Sum::apply(&mut builder, normalized, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), reduced)
            .unwrap();
        let staged_contributor = builder.build().unwrap();
        assert_eq!(
            recognize(&staged_contributor).unwrap_err(),
            "reduction-contributor-materialization"
        );
    }

    /// Two ordered named outputs whose producers share no occurrence.
    ///
    /// `product = a * b` and `sum = a + b` over the same two declared inputs.
    /// The independence is the point: neither output's walk reaches the other's
    /// producer, which is exactly the branch the superseded single-output
    /// recognition refused under `operation-set` — one walk covered one of the
    /// two operations and the program had two.
    fn independent_two_output_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let first = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let second = builder
            .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let product = F32Multiply::apply(&mut builder, first, second).unwrap();
        let sum = F32Add::apply(&mut builder, first, second).unwrap();
        builder
            .output(OutputKey::new("product").unwrap(), product)
            .unwrap();
        builder.output(OutputKey::new("sum").unwrap(), sum).unwrap();
        builder.build().unwrap()
    }

    /// Recognition names one implementable region partition per ordered output.
    ///
    /// **The wall this ticket was filed for, observed gone.** The recognition
    /// used to read one output, classify it, and require that one walk to cover
    /// the program; a second declared output outside the walk therefore refused
    /// under `operation-set`, which is what the measurement at `3adc0689`
    /// recorded when both arity guards were relaxed. The same program now
    /// recognizes into two partitions, each naming its own output key,
    /// expression, and members — and the members are disjoint, which is what
    /// makes each one a region a cover can place without two regions claiming
    /// one occurrence.
    ///
    /// The whole boundary is asserted beside the walk, because the two together
    /// are what the claim needs: the same program recognizes into two partitions
    /// *and* clears [`select_supported_strategy`], which used to refuse it under
    /// `output-arity` before any occurrence was classified. That guard is gone,
    /// so the two derivations now agree rather than contradicting each other.
    #[test]
    fn recognizing_several_ordered_named_outputs_names_one_partition_each() {
        let program = independent_two_output_program();
        assert_eq!(program.output_count(), 2);
        assert_eq!(program.operation_count(), 2);

        let recognized = recognize_outputs(&program).expect("both outputs are recognized");
        let [product, sum] = recognized.outputs() else {
            panic!("one recognized partition per declared output, in declaration order");
        };
        let product = product
            .pointwise()
            .expect("a multiply is an elementwise output");
        let sum = sum.pointwise().expect("an add is an elementwise output");
        assert_eq!(product.output_key, OutputKey::new("product").unwrap());
        assert_eq!(sum.output_key, OutputKey::new("sum").unwrap());
        // Each walk claims exactly its own producer, and the two sets are
        // disjoint: together they partition the program's occurrences.
        assert_eq!(product.members.len(), 1);
        assert_eq!(sum.members.len(), 1);
        assert_ne!(product.members, sum.members);
        assert_eq!(recognized.all_members().len(), program.operation_count());
        // Two different binary32 functions over the same two reads, so the
        // partitions are distinguished by what they compute and not only by
        // which occurrence they name.
        assert_ne!(product.expression, sum.expression);

        // The same recognition reached through the ordinary boundary, which is
        // where the arity guard stood. Compared by the same fields rather than
        // by whole-value equality, for the reason
        // `two_programs_differing_only_in_output_order_recognize_differently`
        // gives about `ValueId` carrying its graph.
        let admitted = select_supported_strategy(&program, &laws_of(&program))
            .expect("the boundary admits it");
        assert_eq!(
            admitted
                .outputs()
                .iter()
                .map(NormalizedOutput::members)
                .collect::<Vec<_>>(),
            vec![product.members.clone(), sum.members.clone()],
        );
    }

    /// A cover region resolves to the output whose partition owns it.
    ///
    /// This is the lookup `crate::physical::spell_region` performs, exercised
    /// on the one shape that can distinguish it from the whole-program question
    /// it replaced: with two declared outputs, "which expression does this
    /// region compute" has two answers and the members are what choose between
    /// them. The straddling case is the one that must say no — a region covering
    /// both outputs' occurrences computes two published results from one owning
    /// write, and no scheduled region does that.
    #[test]
    fn a_region_resolves_to_the_output_whose_partition_owns_it() {
        let program = independent_two_output_program();
        let recognized = recognize_outputs(&program).expect("both outputs are recognized");
        let [first, second] = recognized.outputs() else {
            panic!("one recognized partition per declared output");
        };
        let first_members = first.members();
        let second_members = second.members();

        assert_eq!(
            recognized
                .output_for_region(&first_members)
                .map(|(at, _)| at),
            Some(0),
        );
        assert_eq!(
            recognized
                .output_for_region(&second_members)
                .map(|(at, _)| at),
            Some(1),
        );
        // The check can say no, in both of the ways a cover can get it wrong: a
        // region straddling the two partitions, and a region covering neither.
        let straddling = recognized.all_members();
        assert_eq!(straddling.len(), 2);
        assert!(recognized.output_for_region(&straddling).is_none());
        assert!(recognized.output_for_region(&[]).is_none());
    }

    /// The whole-program cover check was widened, not removed, and says no.
    ///
    /// **Both arms are driven against a case that must fail.** The accepted
    /// neighbour is the recognized two-output partition itself; each perturbation
    /// takes exactly one property away from it.
    ///
    /// *Removal-shaped.* Dropping one occurrence from a walk leaves an
    /// occurrence no output claims, which is work the assembled program would
    /// silently not compute. Removing the check rather than widening it is
    /// exactly what would admit this, so the perturbation is the removal.
    ///
    /// *Overlap-shaped.* Adding one walk's occurrence to another's makes the two
    /// partitions claim it twice, which is the shape where one region's owning
    /// write would have to serve both a materialization edge and a publication.
    #[test]
    fn the_output_partition_check_can_say_no_in_both_directions() {
        let program = independent_two_output_program();
        let recognized = recognize_outputs(&program).expect("both outputs are recognized");
        let outputs = recognized.outputs().to_vec();
        // The control: unperturbed, the walks partition the occurrences.
        assert_eq!(check_output_cover(&program, &outputs), Ok(()));

        let mut uncovered = outputs.clone();
        let NormalizedOutput::Pointwise(dropped) = &mut uncovered[1] else {
            panic!("the fixture's second output is elementwise");
        };
        dropped.members.clear();
        assert_eq!(
            check_output_cover(&program, &uncovered),
            mismatch("operation-set"),
            "an occurrence covered by no walk was admitted",
        );

        let mut overlapping = outputs.clone();
        let claimed = outputs[0].members();
        let NormalizedOutput::Pointwise(widened) = &mut overlapping[1] else {
            panic!("the fixture's second output is elementwise");
        };
        widened.members.extend_from_slice(&claimed);
        widened.members.sort_unstable();
        assert_eq!(
            check_output_cover(&program, &overlapping),
            mismatch("output-partition-overlap"),
            "one occurrence claimed by two walks was admitted",
        );
    }

    /// Two output keys naming one value still refuse under the partition rule.
    ///
    /// **This is the neighbour of the admitted overlap, and it differs from it
    /// by exactly the property [`published_and_consumed_overlap`] requires.**
    /// Three shapes, each observed refusing under the partition rule rather than
    /// being admitted and dropped a layer down:
    ///
    /// - Two output keys naming *one* value. The two walks are equal rather than
    ///   one being a strict subset of the other, so there is no shorter walk to
    ///   publish and no boundary to publish at. Whichever region owns that
    ///   value's write publishes once, and
    ///   `tiler_ir::program::KernelProgramBuilder` refuses a second publication
    ///   of one buffer.
    /// - A publication *inside* one recognized part. `product` is consumed by
    ///   the add that `biased` names, and a pointwise walk fusing the multiply
    ///   and the add has no region boundary between them — the subset is not a
    ///   *part*, which is the conjunct `owns_region_members` decides.
    /// - A published value nothing outside the part reads. This one is stated
    ///   against [`published_and_consumed_overlap`] directly rather than as a
    ///   program, and that is a fact worth recording rather than a convenience:
    ///   for every program the recognizer admits, the value a part publishes
    ///   *is* the value crossing its boundary, so the conjunct is defence in
    ///   depth against a future recognizer rather than a live gate. Stating the
    ///   member sets is what makes it drivable at all.
    ///
    /// Their admitted neighbour is the published-and-consumed program that
    /// `crate::pipeline::conformance`'s
    /// `a_published_and_consumed_intermediate_compiles_and_agrees` compiles,
    /// which differs from each by exactly one of those conjuncts.
    #[test]
    fn an_output_key_pair_naming_one_value_still_refuses_by_name() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let first = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let second = builder
            .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let product = F32Multiply::apply(&mut builder, first, second).unwrap();
        builder
            .output(OutputKey::new("product").unwrap(), product)
            .unwrap();
        builder
            .output(OutputKey::new("alias").unwrap(), product)
            .unwrap();
        let colliding = builder.build().unwrap();
        assert_eq!(colliding.output_count(), 2);
        assert_eq!(colliding.operation_count(), 1);
        assert_eq!(
            recognize_outputs(&colliding).unwrap_err(),
            "output-partition-overlap",
        );

        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let other = builder
            .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let product = F32Multiply::apply(&mut builder, input, other).unwrap();
        let biased = F32Add::apply(&mut builder, product, other).unwrap();
        builder
            .output(OutputKey::new("biased").unwrap(), biased)
            .unwrap();
        builder
            .output(OutputKey::new("product").unwrap(), product)
            .unwrap();
        let mid_walk = builder.build().unwrap();
        assert_eq!(
            recognize_outputs(&mid_walk).unwrap_err(),
            "output-partition-overlap",
        );

        // The admitted neighbour, at this same boundary: `scaled` is a strict
        // subset of the fold's walk, is exactly its recognized prologue part,
        // and is the value the fold reads across the boundary.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let scaled = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let reduced = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("reduced").unwrap(), reduced)
            .unwrap();
        builder
            .output(OutputKey::new("scaled").unwrap(), scaled)
            .unwrap();
        let published_and_consumed = builder.build().unwrap();
        let recognized =
            recognize_outputs(&published_and_consumed).expect("the overlap is admitted");
        let claimed: Vec<Vec<SemanticStage>> = recognized
            .outputs()
            .iter()
            .map(NormalizedOutput::members)
            .collect();
        assert_eq!(
            published_and_consumed_overlap(&published_and_consumed, recognized.outputs(), &claimed),
            Some((1, 0)),
        );

        // The crossing conjunct, driven against a stated member set: the shorter
        // walk is the fold's *reduction* part rather than its prologue part — a
        // part in its own right, and still a strict subset — but the value the
        // second output publishes is the multiply's, which no occurrence outside
        // that part reads. Every other conjunct is unchanged.
        let reduction_part = vec![claimed[0].last().copied().expect("the fold claims members")];
        assert_eq!(
            published_and_consumed_overlap(
                &published_and_consumed,
                recognized.outputs(),
                &[claimed[0].clone(), reduction_part],
            ),
            None,
        );
    }

    /// Two declared inputs and one expression naming both of the outer ones.
    ///
    /// `product = a * c` and `doubled = b + b` over three declared `[2, 3]`
    /// inputs. The first walk reads ordinals `0` and `2`, which is deliberately
    /// not a prefix and not contiguous: a region-local renumbering would give
    /// its two leaves reads `0` and `1` and the assembled program would multiply
    /// `a * b`, and every other recognized fact would agree.
    fn non_contiguous_subset_program(outer: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let inputs: Vec<_> = ["a", "b", "c"]
            .into_iter()
            .map(|key| {
                builder
                    .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([2, 3]))
                    .unwrap()
            })
            .collect();
        let (paired, doubled) = if outer { (2, 1) } else { (1, 2) };
        let product = F32Multiply::apply(&mut builder, inputs[0], inputs[paired]).unwrap();
        let sum = F32Add::apply(&mut builder, inputs[doubled], inputs[doubled]).unwrap();
        builder
            .output(OutputKey::new("product").unwrap(), product)
            .unwrap();
        builder
            .output(OutputKey::new("doubled").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    /// A walk reading a subset carries the program's ordinals, not its own.
    ///
    /// **The read list is the map this ticket asked for.** `mint_elementwise`
    /// numbers the expression's leaves by position in the canonical read order,
    /// and the read at that position names the declared input ordinal it binds,
    /// so `reads` *is* the leaf-ordinal-to-input-ordinal correspondence and
    /// nothing further had to be carried. What changed is that it is no longer
    /// the identity on `0..declared`.
    ///
    /// The neighbour swaps which of the two later inputs each output reads, so
    /// the recognized ordinals move with the program while the expression, the
    /// declared keys, the domain, and the member sets all stay put — which is
    /// what makes the assertion about the read list rather than about the
    /// program being recognized at all.
    #[test]
    fn a_walk_reading_a_subset_carries_the_program_input_ordinals_it_reached() {
        for (outer, expected) in [(true, 2_u32), (false, 1)] {
            let program = non_contiguous_subset_program(outer);
            assert_eq!(program.input_count(), 3);
            let recognized = recognize_outputs(&program).expect("a subset walk is recognized");
            let [product, doubled] = recognized.outputs() else {
                panic!("the fixture declares two outputs");
            };
            let NormalizedOutput::Pointwise(product) = product else {
                panic!("an elementwise output recognizes as an elementwise program");
            };
            let NormalizedOutput::Pointwise(doubled) = doubled else {
                panic!("an elementwise output recognizes as an elementwise program");
            };
            // The declared interface stays whole: the ordinals index it, so a
            // region reading two of three inputs still resolves against all
            // three at assembly.
            assert_eq!(product.input_keys.len(), 3);
            assert_eq!(
                product.reads,
                vec![
                    (0, LogicalAccess::LinearIdentity),
                    (expected, LogicalAccess::LinearIdentity),
                ],
            );
            assert_eq!(product.expression.f32().input_count(), 2);
            // The other output reads the remaining input at one leaf, twice.
            let other = if outer { 1 } else { 2 };
            assert_eq!(doubled.reads, vec![(other, LogicalAccess::LinearIdentity)]);
            assert_eq!(doubled.expression.f32().input_count(), 1);
        }
    }

    /// A declared input no output reads is refused at program scope.
    ///
    /// **The removal-shaped perturbation, and it has to be forged.** The
    /// obligation `canonical_input_reads` used to state per walk moved to
    /// [`check_output_cover`], and no program the public builder can construct
    /// reaches it: a frozen program retains only output-reachable values, the
    /// `operation-set` rule claims every retained occurrence for some walk, and
    /// every way a walk consumes an operand records a read of it. So the check
    /// is driven against a recognized program whose read list has had one entry
    /// removed — which is exactly the state deleting the check would admit —
    /// and its unforged neighbour is asserted to pass, so a check that refused
    /// everything would fail here too.
    #[test]
    fn a_declared_input_no_output_reads_is_refused_at_program_scope() {
        let program = non_contiguous_subset_program(true);
        let recognized = recognize_outputs(&program).expect("a subset walk is recognized");
        assert_eq!(check_output_cover(&program, recognized.outputs()), Ok(()));

        let mut forged = recognized.clone();
        let NormalizedOutput::Pointwise(product) = &mut forged.outputs[0] else {
            panic!("the first declared output is elementwise");
        };
        product.reads.retain(|(ordinal, _)| *ordinal != 2);
        assert_eq!(
            check_output_cover(&program, &forged.outputs),
            mismatch("input-set"),
        );
    }

    /// A fold retains whichever declared input its contributor names.
    ///
    /// The two programs have the same declaration, output families, shapes, and
    /// operation order. The contributor ordinal is the relevant difference, and
    /// it reaches both normalization and the output-subject bytes rather than
    /// being renumbered to the fold region's only read.
    #[test]
    fn a_fold_over_a_later_declared_input_retains_its_ordinal() {
        let folded = |first: bool| {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let inputs: Vec<_> = ["a", "b"]
                .into_iter()
                .map(|key| {
                    builder
                        .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([2, 3]))
                        .unwrap()
                })
                .collect();
            let (folded, doubled) = if first { (0, 1) } else { (1, 0) };
            let sum =
                StrictSerialF32Sum::apply(&mut builder, inputs[folded], [Axis::new(1)]).unwrap();
            let pair = F32Add::apply(&mut builder, inputs[doubled], inputs[doubled]).unwrap();
            builder
                .output(OutputKey::new("folded").unwrap(), sum)
                .unwrap();
            builder
                .output(OutputKey::new("doubled").unwrap(), pair)
                .unwrap();
            builder.build().unwrap()
        };
        let recognized = [
            recognize_outputs(&folded(true)).expect("a fold over input zero"),
            recognize_outputs(&folded(false)).expect("a fold over input one"),
        ];
        let mut encoded = Vec::new();
        for (ordinal, outputs) in recognized.iter().enumerate() {
            let [normalized, _] = outputs.outputs() else {
                panic!("the fixture declares two outputs");
            };
            let NormalizedOutput::SerialSum(fold) = normalized else {
                panic!("a reduction output recognizes as a serial sum");
            };
            assert_eq!(fold.prologue, None);
            assert_eq!(
                fold.contributor_input,
                Some(u32::try_from(ordinal).unwrap())
            );

            let mut bytes = Vec::new();
            encode_output_subject(&mut bytes, &output_subject(normalized));
            encoded.push(bytes);
        }
        assert_ne!(encoded[0], encoded[1]);
    }

    /// The read run separates two subsets and leaves a complete one empty.
    ///
    /// **Both halves of the sub-tag determination, driven at the encoder.** The
    /// complete read list writes the framed zero it has always written, which is
    /// the "no already-encodable subject's bytes move" half; the three
    /// two-element subsets of three declared inputs write three different runs,
    /// which is the injectivity half the marker exists for. Without the marker
    /// all three would be that same framed zero, and one arm would encode three
    /// programs.
    #[test]
    fn the_read_run_marks_unread_declared_inputs_and_leaves_a_complete_list_empty() {
        let dense = |ordinal| (ordinal, LogicalAccess::LinearIdentity);
        let run = |reads: &[(u32, LogicalAccess)]| {
            let mut bytes = Vec::new();
            encode_elementwise_reads(&mut bytes, 3, reads);
            bytes
        };
        // The framed zero every already-encodable subject wrote, byte for byte.
        assert_eq!(run(&[dense(0), dense(1), dense(2)]), vec![0_u8; 8]);
        // One marker, naming the ordinal no leaf read.
        let mut expected = vec![0_u8; 7];
        expected.push(1);
        expected.extend_from_slice(&1_u32.to_be_bytes());
        expected.push(UNREAD_DECLARED_INPUT_TAG);
        assert_eq!(run(&[dense(0), dense(2)]), expected);
        // The three subsets of the same size are three distinct runs, which is
        // the collision the marker closes.
        let subsets = [
            run(&[dense(0), dense(1)]),
            run(&[dense(0), dense(2)]),
            run(&[dense(1), dense(2)]),
        ];
        for (position, first) in subsets.iter().enumerate() {
            for second in &subsets[position + 1..] {
                assert_ne!(first, second);
            }
        }
    }

    /// Both claimants of a published-and-consumed part resolve to one region.
    ///
    /// **This is the check behind the decided tie-break.**
    /// [`NormalizedProgram::output_for_region`] scans in declaration order and
    /// takes the first match, and the admitted overlap makes two outputs own one
    /// member set — so "first" is only correct because the two claimants are
    /// recognitions of one value over one occurrence set and therefore spell the
    /// same region. That argument is worth less than a check that says no when
    /// it stops holding, which is what this is: the same member set is resolved
    /// against each claimant in turn, and the two regions the physical layer
    /// builds from those resolutions are compared whole.
    ///
    /// The two spellings are reached through different arms — the fold's
    /// prologue part and the pointwise output's own walk — so an agreement here
    /// is about the recognitions rather than about one code path being called
    /// twice.
    #[test]
    fn both_claimants_of_a_published_and_consumed_part_spell_one_region() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let scaled = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let reduced = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("reduced").unwrap(), reduced)
            .unwrap();
        builder
            .output(OutputKey::new("scaled").unwrap(), scaled)
            .unwrap();
        let program = builder.build().unwrap();
        let recognized = recognize_outputs(&program).expect("the overlap is admitted");
        let [fold, publication] = recognized.outputs() else {
            panic!("one recognized partition per declared output");
        };
        let shared = publication.members();

        // Both own it, which is the state the tie-break exists for.
        assert!(fold.owns_region_members(&shared));
        assert!(publication.owns_region_members(&shared));
        assert_eq!(
            recognized.output_for_region(&shared).map(|(at, _)| at),
            Some(0),
            "the first declared claimant is the one the scan returns",
        );

        // And they spell one region. Compared through the request the physical
        // layer actually reads, at the write the cover assigns a published-and-
        // consumed region.
        let request = verify_planned_request(CompilationRequest::governed(&program))
            .unwrap()
            .for_target(0)
            .unwrap();
        let staging = crate::physical::RegionWrite::MaterializedAndPublished;
        let (from_fold, fold_members) = crate::physical::pointwise_region(&request, fold, staging);
        let (from_publication, publication_members) =
            crate::physical::pointwise_region(&request, publication, staging);
        assert_eq!(from_fold, from_publication);
        assert_eq!(fold_members, publication_members);
        assert_eq!(fold_members, shared);
    }

    /// Output order reaches the recognized program, not only the semantic graph.
    ///
    /// Two programs holding the same operations and the same two output keys,
    /// differing only in which `output()` call came first, recognize into lists
    /// that are unequal *and* unequal in order — the first entry of one is the
    /// second entry of the other. The request subject encodes that list
    /// length-framed in this order, so a permuted declaration cannot reach one
    /// subject; the semantic half of the same claim is pinned in
    /// `crates/tiler-compiler/tests/multi_output_boundary.rs`.
    ///
    /// **The subject half is asserted here too, and it was not reachable until
    /// `output-arity` was relaxed:** a subject is minted only for a request the
    /// boundary admitted, and that guard admitted no two-output program at all.
    /// Both orders now mint one, and the two subjects name their outputs in the
    /// order their programs declared them.
    ///
    /// **Measurement boundary, and it is a limit on what any test here can
    /// claim.** The subject's *output list* is compared against the program's
    /// declared keys, not its canonical bytes. The previous version of this
    /// comment predicted the encoded form would become checkable once the guard
    /// moved, and it does not: the subject folds the semantic graph identity,
    /// output order is already part of that identity, and no two programs can
    /// differ *only* in the recognized list — so two subjects' bytes differ
    /// whatever the list order, observed by sorting the arms in
    /// [`VerifiedRequestSubject::canonical_explain_subject_bytes`] and watching
    /// the inequality still hold. A check that cannot say no is not evidence.
    /// The list comparison is anchored to the declared keys for the same reason:
    /// comparing the two subjects only to each other survives a list reversed
    /// for both, which was also observed.
    ///
    /// The recognized entries are compared by the fields the subject encodes
    /// rather than by the whole recognized value, because a [`ValueId`] carries
    /// the graph it was built in: two separately built programs never share one,
    /// so whole-value equality would report a difference this test is not about
    /// and would hold whatever the order.
    #[test]
    fn two_programs_differing_only_in_output_order_recognize_differently() {
        fn ordered(product_first: bool) -> SemanticProgram {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let first = builder
                .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
                .unwrap();
            let second = builder
                .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
                .unwrap();
            let product = F32Multiply::apply(&mut builder, first, second).unwrap();
            let sum = F32Add::apply(&mut builder, first, second).unwrap();
            let product_key = OutputKey::new("product").unwrap();
            let sum_key = OutputKey::new("sum").unwrap();
            if product_first {
                builder.output(product_key, product).unwrap();
                builder.output(sum_key, sum).unwrap();
            } else {
                builder.output(sum_key, sum).unwrap();
                builder.output(product_key, product).unwrap();
            }
            builder.build().unwrap()
        }

        /// The per-output facts the request subject encodes, in list order.
        fn encoded(recognized: &NormalizedProgram) -> Vec<(OutputKey, Vec<SemanticStage>)> {
            recognized
                .outputs()
                .iter()
                .map(|output| {
                    let pointwise = output.pointwise().expect("an elementwise output");
                    (pointwise.output_key.clone(), pointwise.members.clone())
                })
                .collect()
        }

        let product_first = encoded(&recognize_outputs(&ordered(true)).expect("recognized"));
        let sum_first = encoded(&recognize_outputs(&ordered(false)).expect("recognized"));
        assert_ne!(
            product_first, sum_first,
            "output order must reach the recognized program, not only presentation",
        );
        assert_eq!(product_first[0], sum_first[1]);
        assert_eq!(product_first[1], sum_first[0]);
        // The check can say no: re-declaring the same order reproduces the
        // recognition, so the inequality above is about the order and not about
        // rebuilding the program.
        assert_eq!(
            product_first,
            encoded(&recognize_outputs(&ordered(true)).expect("recognized")),
        );

        // The same claim about the *subject*, minted through the ordinary
        // boundary rather than from the walk alone, and anchored to the
        // program's own declared order rather than only to the other subject.
        // Comparing the two subjects to each other is not enough: a subject list
        // reversed for *both* programs still swaps entry for entry, so that
        // relation holds while the interface is backwards. The declared keys are
        // the fixed point a reversal moves away from.
        for product_first in [true, false] {
            let program = ordered(product_first);
            let declared: Vec<OutputKey> = program
                .outputs()
                .map(|output| output.key().clone())
                .collect();
            let request = verify_planned_request(CompilationRequest::governed(&program))
                .expect("the boundary admits an ordered two-output program");
            let request = request.for_target(0).expect("one governed target");
            let subject: Vec<OutputKey> = request
                .subject()
                .normalized()
                .outputs()
                .iter()
                .map(|output| match output {
                    NormalizedOutputSubject::Pointwise(normalized) => normalized.output_key.clone(),
                    _ => panic!("both outputs of the fixture are elementwise"),
                })
                .collect();
            assert_eq!(
                subject, declared,
                "the request subject does not name the outputs in declaration order",
            );
        }
    }

    /// Builds a binary contraction, optionally with an elementwise epilogue.
    fn contraction_program(epilogue: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let left = builder
            .input::<F32>(InputKey::new("left").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let right = builder
            .input::<F32>(InputKey::new("right").unwrap(), Shape::from_dims([3, 4]))
            .unwrap();
        // `ab,bc->ac`: the ordinary matrix product, stated as the index
        // structure the operation's identity is.
        let structure = ContractionIndexStructure::new(
            [
                vec![ContractionIndex::new(0), ContractionIndex::new(1)],
                vec![ContractionIndex::new(1), ContractionIndex::new(2)],
            ],
            [ContractionIndex::new(0), ContractionIndex::new(2)],
        )
        .expect("ab,bc->ac is an admitted structure");
        let product =
            tiler_ir::semantic::F32TensorContraction::apply(&mut builder, &structure, left, right)
                .unwrap();
        let root = if epilogue {
            let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
            F32Multiply::apply(&mut builder, product, scale).unwrap()
        } else {
            product
        };
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    }

    /// A contraction over one of the three two-input subsets of one declaration.
    ///
    /// The independent output retains the skipped input without entering the
    /// contraction walk. All input shapes and occurrence positions are equal
    /// across fixtures, so the read ordinals are the only contraction-subject
    /// field that changes.
    fn contraction_subset_program(pair: [usize; 2]) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let shape = Shape::from_dims([2, 2]);
        let inputs: Vec<_> = ["a", "b", "c"]
            .into_iter()
            .map(|key| {
                builder
                    .input::<F32>(InputKey::new(key).unwrap(), shape.clone())
                    .unwrap()
            })
            .collect();
        let structure = ContractionIndexStructure::new(
            [
                vec![ContractionIndex::new(0), ContractionIndex::new(1)],
                vec![ContractionIndex::new(1), ContractionIndex::new(2)],
            ],
            [ContractionIndex::new(0), ContractionIndex::new(2)],
        )
        .expect("ab,bc->ac is an admitted structure");
        let product = tiler_ir::semantic::F32TensorContraction::apply(
            &mut builder,
            &structure,
            inputs[pair[0]],
            inputs[pair[1]],
        )
        .unwrap();
        let skipped = (0..3)
            .find(|ordinal| !pair.contains(ordinal))
            .expect("two of three inputs leave one skipped");
        let retained = F32Add::apply(&mut builder, inputs[skipped], inputs[skipped]).unwrap();
        builder
            .output(OutputKey::new("product").unwrap(), product)
            .unwrap();
        builder
            .output(OutputKey::new("retained").unwrap(), retained)
            .unwrap();
        builder.build().unwrap()
    }

    /// The three subsets are distinguished by the contraction arm itself.
    ///
    /// This drives [`encode_output_subject`] directly, excluding the enclosing
    /// semantic graph identity that would distinguish separately built programs
    /// whatever this arm encoded. It also pins both read predicates for the
    /// skipped ordinal, so restoring dense indexing or a declaration-length
    /// predicate makes the first non-prefix subset fail independently.
    #[test]
    fn contraction_subjects_separate_all_two_input_subsets_of_three_declarations() {
        let pairs = [[0_u32, 1_u32], [0, 2], [1, 2]];
        let mut subjects = Vec::new();
        for pair in pairs {
            let program = contraction_subset_program([
                usize::try_from(pair[0]).unwrap(),
                usize::try_from(pair[1]).unwrap(),
            ]);
            let recognized = recognize_outputs(&program).expect("both outputs are recognized");
            let NormalizedOutput::Contraction(contraction) = &recognized.outputs()[0] else {
                panic!("the first output is the contraction");
            };
            assert_eq!(contraction.input_keys.len(), 3);
            assert_eq!(
                contraction
                    .reads
                    .iter()
                    .map(|read| read.input_ordinal)
                    .collect::<Vec<_>>(),
                pair,
            );
            let skipped = (0..3).find(|ordinal| !pair.contains(ordinal)).unwrap();
            for ordinal in pair {
                assert!(recognized.outputs()[0].reads_declared_input(ordinal));
                assert_eq!(
                    recognized.outputs()[0].input_elements_at(InputOrdinal::new(ordinal)),
                    Some(4),
                );
            }
            assert!(!recognized.outputs()[0].reads_declared_input(skipped));
            assert_eq!(
                recognized.outputs()[0].input_elements_at(InputOrdinal::new(skipped)),
                None,
            );

            let mut bytes = Vec::new();
            encode_output_subject(&mut bytes, &output_subject(&recognized.outputs()[0]));
            subjects.push(bytes);
        }
        for (position, first) in subjects.iter().enumerate() {
            for second in &subjects[position + 1..] {
                assert!(first != second, "two declared-input subsets collided");
            }
        }
    }

    /// The conditional ordinal run does not move an old contraction subject.
    ///
    /// The helper is the exact pre-widening `contraction-f32.v1` arm, projected
    /// through the new read records. Equality therefore checks every byte of an
    /// already-admitted two-declaration subject, not merely its tag or digest.
    #[test]
    fn a_two_declaration_contraction_keeps_its_v1_subject_bytes() {
        let program = contraction_program(false);
        let recognized = recognize(&program).expect("the contraction is recognized");
        let NormalizedOutput::Contraction(normalized) = &recognized else {
            panic!("the output is a contraction");
        };
        assert_eq!(
            normalized
                .reads
                .iter()
                .map(|read| read.input_ordinal)
                .collect::<Vec<_>>(),
            [0, 1],
        );

        let mut legacy = Vec::new();
        push_slice(&mut legacy, b"contraction-f32.v1");
        push_len(&mut legacy, normalized.input_keys.len());
        for key in &normalized.input_keys {
            push_slice(&mut legacy, key.as_str().as_bytes());
        }
        push_slice(&mut legacy, normalized.output_key.as_str().as_bytes());
        for read in &normalized.reads {
            encode_explain_shape(&mut legacy, &read.shape);
        }
        encode_explain_shape(&mut legacy, &normalized.output_shape);
        encode_explain_shape(&mut legacy, &normalized.contracted_shape);
        push_slice(
            &mut legacy,
            normalized.structure.canonical_encoding().as_bytes(),
        );
        for read in &normalized.reads {
            push_len(&mut legacy, read.operand_position);
        }
        push_len(&mut legacy, normalized.members.len());
        for atom in &normalized.members {
            legacy.extend_from_slice(&atom.member().0.to_be_bytes());
        }
        for read in &normalized.reads {
            legacy.extend_from_slice(&read.elements.to_be_bytes());
        }
        legacy.extend_from_slice(&normalized.output_elements.to_be_bytes());
        legacy.extend_from_slice(&normalized.contracted_elements.to_be_bytes());

        let mut current = Vec::new();
        encode_output_subject(&mut current, &output_subject(&recognized));
        assert_eq!(current, legacy, "an existing v1 subject moved bytes");
    }

    #[test]
    fn invalid_pointwise_arity_shape_and_dtype_fail_at_semantic_admission() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let tensor = builder
            .input::<F32>(InputKey::new("tensor").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        assert!(
            builder
                .apply(
                    add_f32_op(),
                    OperationAttributes::empty(),
                    &[tensor.erase()],
                )
                .is_err(),
            "the semantic schema refuses invalid builtin arity before normalization",
        );

        let other_shape = builder
            .input::<F32>(InputKey::new("other").unwrap(), Shape::from_dims([3, 2]))
            .unwrap();
        assert!(
            F32Add::apply(&mut builder, tensor, other_shape).is_err(),
            "the semantic inferencer refuses incompatible shapes before normalization",
        );

        let mut registry = SemanticRegistryBuilder::standard().unwrap();
        registry
            .register_provider(&UnusedSemantics { revision: 1 })
            .unwrap();
        let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
        let foreign = builder
            .input_resolved(
                InputKey::new("foreign").unwrap(),
                Shape::from_dims([2, 3]),
                ResolvedValueType::nominal(TypeKey::new("tiler-test", "unused", 1).unwrap()),
            )
            .unwrap();
        let scalar = F32Constant::apply(&mut builder, 1.0_f32.to_bits())
            .unwrap()
            .erase();
        assert!(
            builder
                .apply(
                    add_f32_op(),
                    OperationAttributes::empty(),
                    &[foreign, scalar],
                )
                .is_err(),
            "the semantic authority refuses a non-f32 builtin operand before normalization",
        );
    }

    #[test]
    fn program_dispatch_types_are_exact_canonical_and_unique() {
        let mut registry = SemanticRegistryBuilder::standard().unwrap();
        registry
            .register_provider(&UnusedSemantics { revision: 1 })
            .unwrap();
        let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
        let f32 = builder
            .input::<F32>(InputKey::new("f32").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scalar = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let foreign_type =
            ResolvedValueType::nominal(TypeKey::new("tiler-test", "unused", 1).unwrap());
        let foreign = builder
            .input_resolved(
                InputKey::new("foreign").unwrap(),
                Shape::from_dims([2, 3]),
                foreign_type.clone(),
            )
            .unwrap();
        builder
            .output(OutputKey::new("f32-output").unwrap(), f32)
            .unwrap();
        builder
            .output(OutputKey::new("scalar-output").unwrap(), scalar)
            .unwrap();
        builder
            .output_resolved(OutputKey::new("foreign-output").unwrap(), foreign)
            .unwrap();
        let program = builder.build().unwrap();

        let actual = canonical_program_value_types(&program);
        assert_eq!(actual.len(), 2, "repeated F32 values are deduplicated");
        assert!(actual.contains(&F32::resolved_type()));
        assert!(actual.contains(&foreign_type));
        assert!(actual.windows(2).all(|pair| {
            pair[0].canonical_encoding().as_bytes() < pair[1].canonical_encoding().as_bytes()
        }));
    }

    #[test]
    fn request_rejects_profile_and_budget_mismatches_stably() {
        let program = program();
        let mut request = CompilationRequest::governed(&program);
        request.budgets.semantic_operations = 4;
        assert_eq!(
            verify_planned_request(request),
            Err(RequestError::BudgetExceeded {
                resource: BudgetResource::SemanticOperations,
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
            verify_planned_request(CompilationRequest::governed(&invalid)),
            Err(RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "operation-set",
            })
        );
    }

    /// Builds a program declaring exactly `inputs` inputs and `outputs` ordered
    /// named outputs over `operations` occurrences, so a budget's `actual` can be
    /// placed on either side of its bound.
    ///
    /// Every occurrence is one `f32` add producing one value, so
    /// `value_count() == inputs + operations`. That is the same identity the
    /// decoder layer has — no occurrence in it produces more than one value —
    /// and it is the identity `semantic_values` is sized against. The chain
    /// consumes every declared input before it starts re-reading the last, so no
    /// declared input is left unreached.
    ///
    /// The outputs are the chain's last `outputs` accumulator values, so the
    /// output arity moves without moving any of the other three counts: that
    /// independence is what lets a probe exceed exactly one of the five bounds.
    fn budget_probe(inputs: usize, operations: usize, outputs: usize) -> SemanticProgram {
        assert!(inputs >= 2, "the chain's first add needs two operands");
        assert!(
            operations >= inputs - 1,
            "fewer adds than inputs would leave a declared input unreached",
        );
        assert!(
            (1..=operations).contains(&outputs),
            "each declared output publishes one of the chain's own results",
        );
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let declared: Vec<_> = (0..inputs)
            .map(|index| {
                builder
                    .input::<F32>(
                        InputKey::new(format!("input{index}")).unwrap(),
                        Shape::from_dims([2, 3]),
                    )
                    .unwrap()
            })
            .collect();
        let mut accumulator = declared[0];
        let mut results = Vec::with_capacity(operations);
        for step in 0..operations {
            let operand = declared[(step + 1).min(inputs - 1)];
            accumulator = F32Add::apply(&mut builder, accumulator, operand).unwrap();
            results.push(accumulator);
        }
        for (ordinal, result) in results[operations - outputs..].iter().enumerate() {
            builder
                .output(OutputKey::new(format!("result{ordinal}")).unwrap(), *result)
                .unwrap();
        }
        let program = builder.build().unwrap();
        assert_eq!(program.input_count(), inputs);
        assert_eq!(program.operation_count(), operations);
        assert_eq!(program.output_count(), outputs);
        assert_eq!(program.value_count(), inputs + operations);
        program
    }

    /// Each widened budget refuses the program one step past it, and the
    /// decoder layer's own measured counts are admitted.
    ///
    /// The five program-scoped bounds are sized to that layer, so the admitted
    /// neighbours are its two measured rows exactly — eighteen declared inputs
    /// and three ordered named outputs over sixty-two occurrences and eighty
    /// values at the decode row, and over fifty-eight and seventy-six at the
    /// prefill row — and the decode row sits *on* all five bounds rather than
    /// under them.
    ///
    /// Refusals are observed through [`verify_program`], which is the entry the
    /// budgets guard; admission is observed at [`check_program_budgets`],
    /// because clearing the budget gate is the whole of what a budget can
    /// promise. `verify_program` still refuses the layer's *shape* at the
    /// recognizer under a rule this widening deliberately does not touch, so an
    /// admitted probe here is evidence about size and about nothing else.
    /// Every budget resource carries its own stable key.
    ///
    /// A duplicate would make two budgets indistinguishable everywhere the key
    /// is what travels — the rule key of a request refusal, the resource key of
    /// an explain record, the reason code of a failure detail — so a caller told
    /// which budget refused would be told the wrong one, silently.
    ///
    /// The population is sized by `variant_count` rather than written out, so a
    /// budget added to the vocabulary and not to `ALL` fails the build here
    /// rather than shrinking the set this test checks while it still reports no
    /// duplicate. The census is printed for the same reason: "nothing ran" must
    /// not be able to look green.
    #[test]
    fn every_budget_resource_key_is_distinct() {
        let keys: BTreeSet<&'static str> = BudgetResource::ALL
            .iter()
            .map(|resource| resource.key())
            .collect();
        assert_eq!(
            keys.len(),
            BudgetResource::ALL.len(),
            "two budget resources share a stable key: {keys:?}",
        );
        assert_eq!(
            BudgetResource::ALL.len(),
            13,
            "the vocabulary changed size; every dependent claim about it needs re-reading",
        );
    }

    /// The three internal stop vocabularies map onto the shared one injectively.
    ///
    /// Each `resource()` is exhaustive, so `rustc` already proves it total. What
    /// it cannot prove is that two internal budgets do not land on one public
    /// row, which would report a region stop as a cover stop or the reverse.
    ///
    /// [`crate::cover::CoverBudgetResource::Refusals`] is deliberately absent
    /// from the image: it refuses no compilation, and its `None` is what keeps
    /// that exclusion typed rather than an inequality at the consuming site.
    #[test]
    fn the_stop_vocabularies_map_onto_distinct_shared_resources() {
        let region = [
            crate::region::RegionBudgetResource::Members,
            crate::region::RegionBudgetResource::BoundaryOutputs,
            crate::region::RegionBudgetResource::LiveValues,
            crate::region::RegionBudgetResource::CandidatesPerSeed,
            crate::region::RegionBudgetResource::Expansions,
        ];
        let mut image: Vec<BudgetResource> = region.iter().map(|stop| stop.resource()).collect();
        image.extend(
            [
                crate::cover::CoverBudgetResource::Covers,
                crate::cover::CoverBudgetResource::Expansions,
                crate::cover::CoverBudgetResource::Refusals,
            ]
            .iter()
            .filter_map(|stop| stop.truncating_resource()),
        );
        image.push(crate::selection::PlanBudgetResource::Combinations.resource());

        let distinct: BTreeSet<BudgetResource> = image.iter().copied().collect();
        assert_eq!(
            distinct.len(),
            image.len(),
            "two stops share one row: {image:?}"
        );
        assert_eq!(
            image.len(),
            8,
            "five region stops, two cover stops, one plan stop"
        );
        assert!(
            crate::cover::CoverBudgetResource::Refusals
                .truncating_resource()
                .is_none(),
            "the explanation budget refuses no compilation and holds no row",
        );

        // Every one of the eight is a search or shape stop reached after a
        // target is consulted, and the five program-scoped rows are exactly the
        // ones no stop vocabulary maps onto.
        for resource in BudgetResource::ALL {
            let program_scoped = matches!(
                resource,
                BudgetResource::SemanticValues
                    | BudgetResource::SemanticOperations
                    | BudgetResource::Regions
                    | BudgetResource::HostExpressionNodes
                    | BudgetResource::Buffers
            );
            assert_eq!(
                program_scoped,
                !distinct.contains(&resource),
                "{resource:?} is claimed by both a stop vocabulary and the request boundary",
            );
        }
    }

    /// A refusal's demand is exact exactly when the bound is not a search bound.
    ///
    /// The split is not decorative: `actual` is a size for one half and a lower
    /// bound for the other, and a caller reading a floor as a requirement would
    /// shrink a program that was never too large. Pinning both directions is
    /// what stops a budget added later from defaulting into whichever answer
    /// happens to be listed first.
    #[test]
    fn only_the_search_bounds_report_a_truncated_demand() {
        for resource in BudgetResource::ALL {
            let searching = matches!(
                resource,
                BudgetResource::RegionCandidatesPerSeed
                    | BudgetResource::RegionExpansions
                    | BudgetResource::RegionCovers
                    | BudgetResource::RegionCoverExpansions
                    | BudgetResource::PhysicalPlanCombinations
            );
            let expected = if searching {
                BudgetRefusal::Truncated
            } else {
                BudgetRefusal::Bounding
            };
            assert_eq!(
                resource.refusal(),
                expected,
                "{resource:?} reports the wrong kind of demand",
            );
        }
    }

    #[test]
    fn each_widened_budget_refuses_the_program_one_step_past_it() {
        let governed = DeterministicBudgets::governed();

        for (inputs, operations) in [(18, 62), (18, 58)] {
            assert_eq!(
                check_program_budgets(&budget_probe(inputs, operations, 3), governed),
                Ok(()),
                "the decoder layer's measured row {inputs}/{operations} is admitted",
            );
        }

        // Exceeding `semantic_values` alone is not expressible: the bound is
        // exactly the eighteen inputs plus the sixty-two occurrences, so one
        // more value is one more input or one more occurrence. Which resource is
        // reported is therefore the check order's guarantee rather than an
        // accident, and it is the first one.
        assert_eq!(
            verify_program(
                &budget_probe(19, 62, 3),
                governed,
                &laws_of(&budget_probe(19, 62, 3))
            )
            .err(),
            Some(RequestError::BudgetExceeded {
                resource: BudgetResource::SemanticValues,
                limit: 80,
                actual: 81,
            }),
        );

        assert_eq!(
            verify_program(
                &budget_probe(17, 63, 3),
                governed,
                &laws_of(&budget_probe(17, 63, 3))
            )
            .err(),
            Some(RequestError::BudgetExceeded {
                resource: BudgetResource::SemanticOperations,
                limit: 62,
                actual: 63,
            }),
        );

        // One further declared output is four further dispatches, and it is the
        // *only* one of these five probes that moves along the output axis. It
        // exceeds all three derived bounds at once — sixteen dispatches,
        // fifty-five expression nodes, and thirty-four buffers — and `regions`
        // is the one that reports, which is the check order's guarantee again.
        assert_eq!(
            verify_program(
                &budget_probe(18, 62, 4),
                governed,
                &laws_of(&budget_probe(18, 62, 4))
            )
            .err(),
            Some(RequestError::BudgetExceeded {
                resource: BudgetResource::Regions,
                limit: 12,
                actual: 16,
            }),
        );

        assert_eq!(
            verify_program(
                &budget_probe(19, 18, 3),
                governed,
                &laws_of(&budget_probe(19, 18, 3))
            )
            .err(),
            Some(RequestError::BudgetExceeded {
                resource: BudgetResource::HostExpressionNodes,
                limit: 51,
                actual: 53,
            }),
        );

        // `buffers` is reached only once the bound that shadows it moves, and
        // the shadowing is a property of the two bounds rather than of this
        // test: both are derived from the declared input count and both are
        // tight at eighteen, so a nineteen-input program exceeds them together
        // and the earlier check reports. The perturbation widens
        // `host_expression_nodes` to exactly what nineteen inputs and three
        // outputs need and leaves `buffers` at its governed value, so what is
        // observed refusing is the governed bound.
        let unshadowed = DeterministicBudgets {
            host_expression_nodes: 53,
            ..governed
        };
        assert_eq!(
            verify_program(
                &budget_probe(19, 18, 3),
                unshadowed,
                &laws_of(&budget_probe(19, 18, 3))
            )
            .err(),
            Some(RequestError::BudgetExceeded {
                resource: BudgetResource::Buffers,
                limit: 30,
                actual: 31,
            }),
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
        let bare = verify_planned_request(CompilationRequest::governed_under(
            &program,
            StrictF32NumericalContract::governed(),
        ))
        .unwrap();
        let listed = verify_planned_request(CompilationRequest::governed_preferring(
            &program,
            NumericalContractPreference::ordered(vec![StrictF32NumericalContract::governed()])
                .unwrap(),
        ))
        .unwrap();
        assert_eq!(bare, listed);
        assert_eq!(
            bare.for_target(0).unwrap().subject(),
            listed.for_target(0).unwrap().subject(),
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
            let verified = verify_planned_request(CompilationRequest::governed_preferring(
                &program,
                NumericalContractPreference::ordered(vec![first, second]).unwrap(),
            ))
            .unwrap();
            assert!(matches!(
                verified.target_slots[0].resolution,
                VerifiedTargetResolution::Resolved {
                    numerical_contract,
                    ..
                } if numerical_contract == first
            ));
            let target = verified.for_target(0).unwrap();
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
        let alone = verify_planned_request(CompilationRequest::governed_preferring(
            &program,
            NumericalContractPreference::ordered(vec![StrictF32NumericalContract::governed()])
                .unwrap(),
        ))
        .unwrap();
        let with_fallback = verify_planned_request(CompilationRequest::governed_preferring(
            &program,
            NumericalContractPreference::ordered(vec![
                StrictF32NumericalContract::governed(),
                StrictF32NumericalContract::governed_flush_to_zero(),
            ])
            .unwrap(),
        ))
        .unwrap();
        let alone = alone.for_target(0).unwrap();
        let with_fallback = with_fallback.for_target(0).unwrap();
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
            verify_planned_request(request),
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
            verify_planned_request(CompilationRequest::governed_under(&program, positive_flush)),
            Err(RequestError::UnsupportedCapability {
                phase: "numerics",
                rule: "governed-contract-profile",
            })
        );

        let target = TargetProfile::governed();
        let error = resolve_numerical_contract(
            &NumericalContractPreference::ordered(vec![
                positive_flush,
                StrictF32NumericalContract::governed(),
            ])
            .unwrap(),
            // A profile that declares nothing at all: every dimension of every
            // entry is undeclared, so nothing may be admitted.
            &TargetProfile::governed_without_numerical_declarations(),
            // The fixture program is `f32`, which both stated entries resolve,
            // so applicability narrows nothing and every entry is asked.
            Some(ArithmeticType::F32),
        )
        .unwrap_err();
        let RequestError::NoResolvableNumericalContract {
            target_profile,
            rejections,
        } = error
        else {
            panic!("an unhonourable preference rejects by name");
        };
        assert_eq!(target_profile, *target.profile_key());
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
                crate::target::honourability::NumericalDimension::InputSubnormals
            );
        }
    }

    /// The governed baseline resolves every registered contract, and its
    /// declaration is what admits them.
    #[test]
    fn the_governed_baseline_honours_every_registered_contract() {
        let target = TargetProfile::governed();
        let expected = crate::target::honourability::CANONICAL_DIMENSIONS
            .into_iter()
            .filter(|dimension| crate::policy::is_consumable(*dimension))
            .count();
        for contract in StrictF32NumericalContract::named_profile() {
            let outcome = crate::physical::assess_contract(&target, contract).unwrap();
            let crate::target::feasibility::FeasibilityOutcome::Proven(evidence) = outcome else {
                panic!("the baseline honours {}", contract.key);
            };
            assert_eq!(
                evidence.honoured().len(),
                expected,
                "one per dimension an admitted operation can consume"
            );
            for honoured in evidence.honoured() {
                assert_eq!(
                    honoured.means(),
                    crate::target::honourability::HonouringMeans::SupportedExactly
                );
                assert_eq!(honoured.arithmetic(), contract.arithmetic);
                assert_eq!(honoured.profile().key(), target.profile_key().as_str());
            }
        }
    }

    /// A contract stating an arithmetic type the profile is silent about is
    /// `Unknown`, never honoured by inheritance from a neighbouring type.
    ///
    /// This is the measured case in miniature. One Apple profile flushes
    /// subnormals in `f32` and preserves them in `f16`, so a declaration for one
    /// width says nothing about the other; a resolver that fell back to a
    /// neighbouring type's fact would report a conformance claim the hardware
    /// contradicts.
    #[test]
    fn a_contract_for_an_undeclared_arithmetic_type_is_unknown() {
        let target = TargetProfile::governed();
        let mut contract = StrictF32NumericalContract::governed();
        contract.arithmetic = ArithmeticType::F16;
        let outcome = crate::physical::assess_contract(&target, contract).unwrap();
        let crate::target::feasibility::FeasibilityOutcome::Unknown(unknown) = outcome else {
            panic!("a profile silent about f16 cannot prove an f16 contract");
        };
        let first = unknown.dimensions().first().expect("a cause is reported");
        assert_eq!(first.arithmetic(), ArithmeticType::F16);
        assert_eq!(first.dimension(), NumericalDimension::InputSubnormals);
    }

    /// Every consumable contract dimension reaches the scheduled realization.
    #[test]
    fn realization_carries_every_consumable_contract_dimension() {
        let mut contract = StrictF32NumericalContract::governed();
        contract.permutation = NumericalPermission::Permitted;
        contract.signed_zero = NumericalPermission::Permitted;
        contract.nan_assumptions = ExceptionalValueAssumption::AssumeAbsent {
            provenance: tiler_ir::schedule::ValueDomainProvenance::CompilerProven,
        };
        contract.infinity_assumptions = ExceptionalValueAssumption::AssumeAbsent {
            provenance: tiler_ir::schedule::ValueDomainProvenance::RuntimeValidated,
        };
        let realization = contract.realization();
        assert_eq!(realization.permutation, contract.permutation);
        assert_eq!(realization.signed_zero, contract.signed_zero);
        assert_eq!(realization.nan_assumptions, contract.nan_assumptions);
        assert_eq!(
            realization.infinity_assumptions,
            contract.infinity_assumptions
        );
    }

    #[test]
    fn request_requires_a_nonempty_unique_target_set() {
        let program = program();
        let mut empty = CompilationRequest::governed(&program);
        empty.target_profiles.clear();
        assert_eq!(
            verify_planned_request(empty),
            Err(RequestError::EmptyTargetSet)
        );

        let mut duplicate = CompilationRequest::governed(&program);
        duplicate.target_profiles.push(TargetProfile::governed());
        assert_eq!(
            verify_planned_request(duplicate),
            Err(RequestError::DuplicateTargetProfile)
        );
    }

    #[test]
    fn verified_request_receipts_reject_post_verification_mutation() {
        let program = program();
        let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
        let mut forged = verified.clone();
        forged.budgets.buffers += 1;
        assert_eq!(
            forged.for_target(0),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified.clone();
        forged.capabilities = CompilerCapabilitySnapshot::without_capabilities();
        assert_eq!(
            forged.for_target(0),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified.clone();
        forged.target_slots[0].target_profile =
            TargetProfile::governed_without_numerical_declarations();
        assert_eq!(
            forged.for_target(0),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified.clone();
        forged.semantic_identity = program_with_unused_provider(7).semantic_identity().clone();
        assert_eq!(
            forged.for_target(0),
            Err(RequestError::UnverifiedTargetSelection)
        );

        // The recognized prologue's scale changed. It is the mutation that used
        // to be a `scale_bits` edit: the subject now carries the whole
        // expression, so a forged prologue is a forged expression.
        let mut forged = verified.clone();
        forged.normalized.serial_sum_mut().prologue =
            Some(affine_expression(3.0_f32.to_bits(), 1.0_f32.to_bits()));
        assert_eq!(
            forged.for_target(0),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified;
        forged.normalized.serial_sum_mut().output_key = OutputKey::new("forged").unwrap();
        assert_eq!(
            forged.for_target(0),
            Err(RequestError::UnverifiedTargetSelection)
        );
    }

    #[test]
    fn verified_target_receipt_detects_every_governed_subject_mutation_class() {
        let program = program();
        let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
        let target = verified.for_target(0).unwrap();

        let mut forged = target.clone();
        forged.target_profile = TargetProfile::governed_without_numerical_declarations();
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

        // One constant of the recognized prologue flipped. The expression is
        // rebuilt rather than edited in place, because it is opaque by
        // construction — which is exactly what makes the subject bind it whole.
        let mut forged = target.clone();
        forged.normalized.serial_sum_mut().prologue =
            Some(affine_expression(2.0_f32.to_bits(), 1.0_f32.to_bits() ^ 1));
        assert!(!forged.reconstructs_its_authority());

        let mut forged = target;
        forged.normalized.serial_sum_mut().input_keys = vec![InputKey::new("forged").unwrap()];
        assert!(!forged.reconstructs_its_authority());
    }

    #[test]
    fn used_provider_revision_changes_admission_and_snapshot_subjects() {
        let first = governed_test_program(1);
        let second = governed_test_program(2);
        let first =
            verify_planned_request(request_with_matching_empty_capabilities(&first)).unwrap();
        let second =
            verify_planned_request(request_with_matching_empty_capabilities(&second)).unwrap();

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
        let first =
            verify_planned_request(request_with_matching_empty_capabilities(&first)).unwrap();
        let second =
            verify_planned_request(request_with_matching_empty_capabilities(&second)).unwrap();

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

    fn request_symbol(name: &str) -> ShapeSymbol {
        ShapeSymbol::new(SymbolScope::new("program/0").unwrap(), name).unwrap()
    }

    fn request_axis_binding(input: &str, axis: u32) -> RootBinding {
        RootBinding::new(
            BindingSource::InputDimension {
                input: InputKey::new(input).unwrap(),
                axis: Axis::new(axis),
            },
            AvailabilityPhase::LiveDevicePreflight,
            FactProvenance::RuntimeValidated,
        )
        .unwrap()
    }

    fn request_environment(bound_to: Option<u64>) -> Arc<ShapeEnv> {
        let mut draft = ShapeEnvBuilder::new();
        let declared = request_symbol("n");
        draft.declare(declared.clone()).unwrap();
        draft.bind(&declared, request_axis_binding("a", 0)).unwrap();
        if let Some(value) = bound_to {
            draft
                .require(SemanticInputConstraint::new(
                    ExtentRelation::equal(
                        ExtentTerm::Symbol(declared),
                        ExtentTerm::Constant(value),
                    ),
                    FactProvenance::FrontendRequired,
                ))
                .unwrap();
        }
        Arc::new(draft.build().unwrap())
    }

    /// `(a * b) + c` over three rank-one `f32` inputs of one sourced extent.
    fn three_input_elementwise_with(
        environment: Option<Arc<ShapeEnv>>,
        extents: &[SourcedExtent],
    ) -> SemanticProgram {
        let mut builder = match environment {
            Some(environment) => {
                SemanticProgramBuilder::try_standard_with_shape_environment(environment).unwrap()
            }
            None => SemanticProgramBuilder::try_standard().unwrap(),
        };
        let inputs: Vec<_> = ["a", "b", "c"]
            .into_iter()
            .map(|key| {
                builder
                    .input_sourced::<F32>(InputKey::new(key).unwrap(), extents.to_vec())
                    .unwrap()
            })
            .collect();
        let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
        let root = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    }

    fn symbolic_three_input_elementwise(bound_to: Option<u64>) -> SemanticProgram {
        three_input_elementwise_with(
            Some(request_environment(bound_to)),
            &[SourcedExtent::Symbol(request_symbol("n"))],
        )
    }

    fn literal_three_input_elementwise(extent: u64) -> SemanticProgram {
        three_input_elementwise_with(None, &[SourcedExtent::Static(Extent::new(extent))])
    }

    fn first_symbolic_extent(program: &SemanticProgram) -> SourcedExtent {
        program
            .inputs()
            .next()
            .and_then(|input| program.shape(input.value()).ok())
            .and_then(|shape| shape.extents().find(|extent| extent.as_static().is_none()))
            .expect("the symbolic fixture names at least one symbol")
    }

    /// A symbolic program is admitted as far as strategy selection.
    ///
    /// The leftover version gate is gone: the request carries the program's own
    /// environment and the refusal is the strategy's, not
    /// `UnsupportedRequestVersion`. Watched failing under a deliberate
    /// perturbation: dropping `shape_environment` from `governed_preferring`
    /// makes the same program refuse as `MismatchedShapeEnvironment` before
    /// recognition runs.
    #[test]
    fn a_symbolic_program_reaches_strategy_selection() {
        let program = symbolic_three_input_elementwise(None);
        let request = CompilationRequest::governed(&program);
        assert!(
            std::ptr::eq(
                request
                    .shape_environment
                    .expect("a symbolic program carries its environment")
                    .environment(),
                program
                    .extent_sources()
                    .expect("the constructed program owns its environment")
                    .environment(),
            ),
            "the request must carry the program's own environment, not a second one",
        );
        match verify_request(request) {
            Err(RequestError::UnsupportedSymbolicExtent {
                phase: "strategy",
                rule: "symbolic-extent",
                extent,
            }) => {
                assert_eq!(extent, SourcedExtent::Symbol(request_symbol("n")));
            }
            other => match other {
                Ok(_) => panic!(
                    "a well-formed symbolic request must reach strategy selection, got Planned/Refused"
                ),
                Err(error) => panic!(
                    "a well-formed symbolic request must reach strategy selection, got {error}"
                ),
            },
        }
    }

    /// An unsupported symbolic case names the extent; the literal neighbour compiles.
    ///
    /// Watched failing under a deliberate perturbation: restoring
    /// `static_shape`'s handle-rule attribution makes the symbolic program
    /// refuse as `UnsupportedCapability { rule: "output-handle" }` and the
    /// extent is no longer in the diagnostic.
    #[test]
    fn an_unsupported_symbolic_case_names_the_extent_and_the_literal_neighbour_compiles() {
        let symbolic = symbolic_three_input_elementwise(None);
        let extent = first_symbolic_extent(&symbolic);
        assert_eq!(extent, SourcedExtent::Symbol(request_symbol("n")));
        assert_eq!(
            verify_planned_request(CompilationRequest::governed(&symbolic)),
            Err(RequestError::UnsupportedSymbolicExtent {
                phase: "strategy",
                rule: "symbolic-extent",
                extent,
            }),
        );

        let literal = literal_three_input_elementwise(4);
        crate::pipeline::compile(CompilationRequest::governed(&literal))
            .expect("the literal neighbour of the symbolic elementwise program still compiles");
    }

    /// A bound symbol is not folded into the logical plan.
    ///
    /// The environment pins `n` to 4. The program still names the symbol, the
    /// request still carries that environment, and compilation still refuses
    /// the symbol rather than emitting a `[4]` plan. Watched failing under a
    /// deliberate perturbation: resolving the symbol through the environment
    /// inside `static_shape` replaces the named-extent refusal with a
    /// mis-attributed `elementwise-shape` capability refusal.
    #[test]
    fn a_compiled_plan_does_not_fold_a_bound_extent_value() {
        let bound = symbolic_three_input_elementwise(Some(4));
        let extent = first_symbolic_extent(&bound);
        assert_eq!(
            extent,
            SourcedExtent::Symbol(request_symbol("n")),
            "a constraint that pins n to 4 must not rewrite the authored shape",
        );
        for value in bound.values() {
            assert_eq!(
                value.shape().as_static(),
                None,
                "no authored boundary may collapse to the bound value",
            );
        }
        assert_eq!(
            verify_planned_request(CompilationRequest::governed(&bound)),
            Err(RequestError::UnsupportedSymbolicExtent {
                phase: "strategy",
                rule: "symbolic-extent",
                extent,
            }),
            "a bound symbol must still be refused as the symbol, not compiled as 4",
        );

        let literal = literal_three_input_elementwise(4);
        crate::pipeline::compile(CompilationRequest::governed(&literal))
            .expect("the literal [4] neighbour still compiles");
    }

    /// Dropping the program's environment is a pairing refusal, not a schema one.
    #[test]
    fn dropping_the_program_environment_is_a_pairing_refusal() {
        let program = symbolic_three_input_elementwise(None);
        let mut request = CompilationRequest::governed(&program);
        request.shape_environment = None;
        match verify_request(request) {
            Err(RequestError::MismatchedShapeEnvironment) => {}
            Ok(_) => panic!("dropping the environment must refuse, got a verified request"),
            Err(error) => panic!("dropping the environment must be a pairing refusal, got {error}"),
        }
        assert_eq!(
            RequestError::MismatchedShapeEnvironment.to_string(),
            "compile.request.shape-environment: request must carry the program's own environment",
        );
    }
}

#[cfg(test)]
mod subject_budget {
    use super::*;
    use crate::target::honourability::encode_declared_behaviours;

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
        let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
        let target = verified.for_target(0).unwrap();
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
        let declared = TargetProfile::governed_declared_behaviours();
        let mut numerical_bytes = Vec::new();
        encode_declared_behaviours(&mut numerical_bytes, &declared);
        let numerical = numerical_bytes.len();
        let declaration_lines = declared.len();
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
