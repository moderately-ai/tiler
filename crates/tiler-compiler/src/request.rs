use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::sync::{Mutex, OnceLock, PoisonError};

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::index::{FrozenIndexRealizationLawRegistry, FrozenScalarRegistry};
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::schedule::{
    AxisDecode, InputOrdinal, LogicalAccess, PointwiseF32Expression, PointwiseF32ExpressionBuilder,
    PointwiseF32Node, PointwiseF32Value, TensorRole,
};
use tiler_ir::semantic::{
    BROADCAST_AXIS_MAPPING_ATTRIBUTE, BroadcastAxisMapping, CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE,
    CanonicalIntegerWidth, CanonicalValueView, ContractionIndex, ContractionIndexStructure, F32,
    F32_CONSTANT_BITS_ATTRIBUTE, InputKey, OpKey, OutputKey, ProviderIdentity,
    REDUCTION_AXES_ATTRIBUTE, REINDEX_MAPPING_ATTRIBUTE, ReindexForm, ReindexFormKind,
    ResolvedValueType, SemanticIdentity, SemanticProgram, TypeKey, ValueId, add_f32_op,
    broadcast_f32_op, constant_f32_op, multiply_f32_op, reindex_f32_op, silu_f32_op,
    strict_serial_sum_f32_op, strict_tensor_contraction_f32_op,
};
use tiler_ir::shape::{Axis, Extent, Shape};

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
use crate::region::SemanticMemberId;
use crate::target::DTypeDispatchabilityResolution;
use crate::target::honourability::{
    DeferredDimension, DimensionBehaviour, NumericalDimension, NumericalRequirement,
    UndeclaredDimension, UnhonouredDimension,
};
pub(crate) use crate::target::{TargetProfile, TargetProfileKey};

const REQUEST_SCHEMA_VERSION: u32 = 2;

/// The versioned domain of the canonical numerical-contract key scheme.
///
/// **A scheme rather than a name, and that is the change this domain records.**
/// Four hand-written strings — `tiler.strict-f32.v1` and its three siblings —
/// used to be the whole contract vocabulary, because a caller could only name
/// one of four presets. A caller now resolves the dimensions directly, so the
/// number of statable contracts is the size of the dimension space and no
/// hand-written name can cover it. The key is therefore *derived* from the
/// dimension vector by [`canonical_contract_key`], and this domain is the
/// version of that derivation.
///
/// The version is `v2` because a `v1` key named a preset and a `v2` key spells a
/// vector: a reader holding either can tell which it has from the prefix, and no
/// `v1` key is reachable any more. The domain moves whenever the rendering
/// moves, which is what keeps a key minted by one build comparable to a key
/// minted by another — the whole reason a contract has a key at all.
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

/// Returns whether one key was minted by the current `f32` contract scheme.
///
/// This performs the complete IR-owned parse and canonicality check. A key
/// under another domain, a malformed vector, or a noncanonical spelling is
/// rejected rather than inferred from a textual prefix.
pub(crate) fn is_f32_contract_key(key: &str) -> bool {
    F32NumericalContractKey::try_from_str(key).is_ok()
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
    /// The bounded profile's deterministic budgets.
    ///
    /// **The four program-scoped bounds are sized to the complete decoder-layer
    /// program, which is the largest program shape this profile may be asked to
    /// admit.** Each is derived from that program's own measured counts rather
    /// than from the smallest number that lets it through, which is the rule
    /// [`check_program_budgets`] states and the rule the split reduction's
    /// earlier widenings followed. The counts are the two rows the layer was
    /// verified and reference-evaluated at: eighteen declared inputs at both,
    /// fifty-eight occurrences over seventy-six values at the C1 prefill row,
    /// and sixty-two over eighty at the C1 decode row. The decode row is the
    /// binding one, and it is larger for a reason that is not the cache: at
    /// `T = 1` six position-axis rank pads duplicate nothing, so the broadcast
    /// family refuses a many-to-one relation onto an extent-one result axis and
    /// the layer spells those widenings as further occurrences.
    ///
    /// - `semantic_values` is `80`: the decode row's eighteen declared inputs
    ///   plus one result per occurrence, because no occurrence in the layer
    ///   produces more than one value. The prefill row is `18 + 58 = 76` by the
    ///   same arithmetic, so eighty bounds both.
    /// - `semantic_operations` is `62`: the decode row's occurrence count.
    /// - `host_expression_nodes` is `43`: [`check_program_budgets`] derives the
    ///   actual as two nodes per declared input plus seven, so eighteen inputs
    ///   reach `2 × 18 + 7`.
    /// - `buffers` is `21`: the same function derives the actual as every
    ///   declared input plus the prologue's temporary, a split's staged partial
    ///   tensor, and the output, so eighteen inputs reach `18 + 3`. It was `3`,
    ///   then `4`, then `6` — the one-input materialized program's input,
    ///   temporary and output; the split's staged partial tensor; and that split
    ///   over the widest three-input prologue the governed target's four buffer
    ///   bindings admit — and every step, including this one, is the same
    ///   derivation over a wider admitted program.
    ///
    /// Both input-derived bounds are now tight at exactly eighteen declared
    /// inputs, so their thresholds coincide: a nineteen-input program exceeds
    /// both at once and the earlier check, `host-expression-nodes`, is the one
    /// that reports.
    ///
    /// **`regions` is `4`, and it is derived from a measurement rather than from
    /// the decoder layer.** The actual it is checked against is a constant,
    /// because a region count is a property of a *plan* and this profile plans no
    /// decoder layer: [`select_supported_strategy`] refuses it under its own
    /// named rules, which is a separate refusal with a separate remedy. What the
    /// constant states is the widest plan the profile assembles, and that moved
    /// when `admit-elementwise-epilogues-over-a-materialized-intermediate`
    /// landed: a fold may now stage its result for an elementwise epilogue, so
    /// the split program's three stages — prologue, partial, final — gain a
    /// fourth. The number is the measured stage count of that plan, taken from
    /// `crate::pipeline::tests::the_widest_assembled_plan_is_the_split_reduction_with_its_epilogue`,
    /// whose reassociation-forbidding neighbour is what attributes the fourth
    /// stage to the split rather than to the epilogue alone.
    ///
    /// It moved from `3`, and the consequence is the one this comment already
    /// records for the other four: every budget is written into the request
    /// subject, so every governed compilation's qualifier moved with it. The
    /// pinned identity is the same single one — `explain`'s
    /// `deterministic_trace_is_sealed_and_rendered_separately` request qualifier
    /// — and its ledger comment records the recomputation. No encoding version
    /// moved: the field set, widths, and order are untouched, so a value change
    /// stays injective inside `tiler.compiler.request-subject.v5`.
    ///
    /// It moves again when the decoder layer becomes plannable, and that is a
    /// second identity move this one cannot honestly absorb.
    ///
    /// `normalization_rewrites` and every `region_*` bound are unchanged because
    /// none of them admits or refuses a program: each bounds a *search*, and
    /// exhausting one costs an alternative while the verified input and complete
    /// coverage survive.
    ///
    /// The widening is a *deliberate* decision and not a test-enabling edit,
    /// because every one of these numbers is inside the canonical request
    /// subject ([`VerifiedRequestSubject::canonical_explain_subject_bytes`]
    /// writes every budget), which is carried into artifact identity. Every
    /// governed compilation's request subject, and therefore every artifact
    /// identity and cache entry derived from it, moves with this change — for
    /// programs nowhere near any of these bounds as much as for ones at them,
    /// because a budget is a property of the *request* rather than of the plan
    /// chosen for it. Exactly one pinned identity encodes these bytes and it
    /// moved with them: `explain`'s
    /// `deterministic_trace_is_sealed_and_rendered_separately` request
    /// qualifier, whose ledger comment records the recomputation. No encoding
    /// version moved with it — the field set, widths, and order are untouched,
    /// so a value change stays injective inside
    /// `tiler.compiler.request-subject.v5`.
    ///
    /// A budget is an upper bound, so widening admits program shapes and never
    /// requires them: [`check_program_budgets`] still refuses a program one step
    /// past each of these, and `verify_host_contract` still refuses a built
    /// program whose value count exceeds `buffers`. Nor does clearing the budget
    /// gate compile a decoder layer — the recognizer's refusal is untouched, and
    /// what this widening removes is only the refusal that was about *size*.
    pub(crate) const fn governed() -> Self {
        Self {
            semantic_values: 80,
            semantic_operations: 62,
            regions: 4,
            host_expression_nodes: 43,
            buffers: 21,
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
    pub(crate) shape_environment: StaticShapeEnvironment,
    /// The caller's ordered numerical-contract preference. Required, with no
    /// `Default` and no ambient fallback (ADR 0076 item 2).
    pub(crate) numerical_contracts: NumericalContractPreference,
    pub(crate) budgets: DeterministicBudgets,
    pub(crate) target_profiles: Vec<TargetProfile>,
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
    pointwise: Vec<SemanticMemberId>,
    reduction: Vec<SemanticMemberId>,
}

impl RecognizedSerialSumMembers {
    /// Binds the recognized prologue's occurrences and the reduction's own.
    fn new(pointwise: Vec<SemanticMemberId>, reduction: u32) -> Self {
        let mut pointwise = pointwise;
        pointwise.sort_unstable();
        pointwise.dedup();
        Self {
            pointwise,
            reduction: vec![SemanticMemberId(reduction)],
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
    pub(crate) fn prologue_members(&self) -> Option<&[SemanticMemberId]> {
        self.prologue.as_ref().map(|_| self.members.pointwise())
    }
}

/// A verified N-input, one-output elementwise `f32` program.
///
/// `input_keys` and `inputs` are parallel and in the program's declaration
/// order, which is the order the expression's input ordinals index and the order
/// the assembled program binds its buffers in. One `shape` governs every input
/// and the output, so a single element count sizes the whole region.
///
/// **`expression` is the recognized program, not a projection of it.** It is the
/// general [`PointwiseF32Expression`] vocabulary rather than a fixed leaf count
/// and association, so what the recognizer admits is bounded by what the
/// physical expression can spell rather than by a shape it was taught.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedPointwise {
    pub(crate) input_keys: Vec<InputKey>,
    pub(crate) output_key: OutputKey,
    pub(crate) shape: Shape,
    pub(crate) expression: PointwiseF32Expression,
    pub(crate) members: Vec<SemanticMemberId>,
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

/// A verified two-input, one-output binary tensor-contraction `f32` program.
///
/// **The structure is carried whole, not projected.** ADR 0087 makes the
/// canonical index structure the operation's identity, so a normalization that
/// kept only the extents it happened to need would let two different structures
/// over the same shapes share a request subject. `operand_positions` maps each
/// *declared input ordinal* to the structure operand it supplies, so a caller
/// whose declaration order differs from its operand order is admitted rather
/// than refused for a spelling — and every downstream binding indexes by
/// declaration order, which is what the ABI binds in.
///
/// `output_shape` and `contracted_shape` are derived from the structure and the
/// operand shapes rather than read from the graph, and the derived output shape
/// is required to equal the program's own: the semantic inferencer already
/// proved them equal at construction, so a disagreement here is invalid state
/// and is refused rather than resolved in favour of either side.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedContraction {
    pub(crate) input_keys: [InputKey; 2],
    pub(crate) output_key: OutputKey,
    /// Operand shapes, indexed by declared input ordinal.
    pub(crate) input_shapes: [Shape; 2],
    pub(crate) output_shape: Shape,
    /// Row-major shape of the contracted iteration space, ascending by
    /// canonical contracted index.
    pub(crate) contracted_shape: Shape,
    pub(crate) structure: ContractionIndexStructure,
    /// Structure operand position supplying each declared input ordinal.
    pub(crate) operand_positions: [usize; 2],
    pub(crate) members: Vec<SemanticMemberId>,
    /// Operand values, indexed by declared input ordinal.
    pub(crate) inputs: [ValueId; 2],
    pub(crate) output: ValueId,
    /// Operand element counts, indexed by declared input ordinal.
    pub(crate) input_elements: [u64; 2],
    pub(crate) output_elements: u64,
    /// Points of the contracted iteration space; the fold length per output.
    pub(crate) contracted_elements: u64,
}

/// One read an elementwise epilogue's expression leaf binds.
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EpilogueRead {
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

impl EpilogueRead {
    /// Returns the boundary tensor this read binds.
    pub(crate) const fn tensor(self) -> TensorRole {
        match self {
            Self::Staged => TensorRole::Intermediate,
            Self::Input(ordinal) => TensorRole::Input {
                ordinal: InputOrdinal::new(ordinal),
            },
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
    /// tensor with. Exactly one entry is [`EpilogueRead::Staged`].
    pub(crate) reads: Vec<(EpilogueRead, LogicalAccess)>,
    /// The occurrences the epilogue region itself covers.
    pub(crate) members: Vec<SemanticMemberId>,
    pub(crate) inputs: Vec<ValueId>,
    pub(crate) output: ValueId,
    pub(crate) elements: u64,
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
}

impl NormalizedOutput {
    pub(crate) const fn serial_sum(&self) -> &NormalizedSerialSum {
        match self {
            Self::SerialSum(normalized) => normalized,
            Self::Pointwise(_) | Self::Contraction(_) | Self::Epilogue(_) => {
                panic!("request is not a serial-sum program")
            }
        }
    }

    pub(crate) const fn try_serial_sum(&self) -> Option<&NormalizedSerialSum> {
        match self {
            Self::SerialSum(normalized) => Some(normalized),
            Self::Pointwise(_) | Self::Contraction(_) | Self::Epilogue(_) => None,
        }
    }

    pub(crate) const fn pointwise(&self) -> Option<&NormalizedPointwise> {
        match self {
            Self::SerialSum(_) | Self::Contraction(_) | Self::Epilogue(_) => None,
            Self::Pointwise(normalized) => Some(normalized),
        }
    }

    pub(crate) const fn contraction(&self) -> Option<&NormalizedContraction> {
        match self {
            Self::SerialSum(_) | Self::Pointwise(_) | Self::Epilogue(_) => None,
            Self::Contraction(normalized) => Some(normalized),
        }
    }

    pub(crate) const fn epilogue(&self) -> Option<&NormalizedEpilogue> {
        match self {
            Self::SerialSum(_) | Self::Pointwise(_) | Self::Contraction(_) => None,
            Self::Epilogue(normalized) => Some(normalized),
        }
    }

    /// Returns the recognized shape every *producer* region of this output is
    /// built from.
    ///
    /// A chain's producer regions — the fold, its prologue, its split passes,
    /// its cooperative tile, the contraction — are spelled from the producer's
    /// own recognized shape, so every derivation that would otherwise read the
    /// chain asks this instead and reaches the same value it reaches for a
    /// standalone output. The epilogue region is the one part that is not built
    /// from it, and [`crate::physical::RegionSpellingKind::Epilogue`] is what
    /// distinguishes it.
    pub(crate) fn producer_shape(&self) -> &Self {
        match self {
            Self::SerialSum(_) | Self::Pointwise(_) | Self::Contraction(_) => self,
            Self::Epilogue(chain) => &chain.producer,
        }
    }

    /// Returns the element count of one declared input tensor.
    ///
    /// Per ordinal rather than one shared count, because a contraction's two
    /// operands generally have different extents. The two single-shape
    /// strategies answer the same for every ordinal they declare and `None` for
    /// one they do not, so a caller that names an ordinal no input occupies gets
    /// a refusal instead of another tensor's size.
    pub(crate) fn input_elements_at(&self, ordinal: InputOrdinal) -> Option<u64> {
        let ordinal = usize::try_from(ordinal.get()).ok()?;
        match self {
            // Every declared input of a reduced program is read at the
            // contributor domain, which is the one shape its prologue governs.
            Self::SerialSum(normalized) => {
                (ordinal < normalized.input_keys.len()).then_some(normalized.input_elements)
            }
            Self::Pointwise(normalized) => {
                (ordinal < normalized.input_keys.len()).then_some(normalized.elements)
            }
            Self::Contraction(normalized) => normalized.input_elements.get(ordinal).copied(),
            // A chain reads a declared input from its producer, from its
            // epilogue, or from both — and the two read it at different domains
            // whenever the producer folds. Agreement or nothing, for the reason
            // [`NormalizedProgram::agreed_input_elements_at`] states: this count
            // scales a call over the tensor bound to the ordinal, and answering
            // with either side would size a call against a domain one of the two
            // regions does not iterate.
            Self::Epilogue(chain) => {
                let produced = chain
                    .producer
                    .input_elements_at(InputOrdinal::new(u32::try_from(ordinal).ok()?));
                let consumed = chain
                    .reads
                    .iter()
                    .any(|(read, _)| {
                        u32::try_from(ordinal)
                            .is_ok_and(|ordinal| *read == EpilogueRead::Input(ordinal))
                    })
                    .then_some(chain.elements);
                match (produced, consumed) {
                    (Some(produced), Some(consumed)) if produced != consumed => None,
                    (Some(elements), _) | (None, Some(elements)) => Some(elements),
                    (None, None) => None,
                }
            }
        }
    }

    /// Returns the largest declared input element count this output reads.
    pub(crate) fn max_input_elements(&self) -> u64 {
        match self {
            Self::SerialSum(normalized) => normalized.input_elements,
            Self::Pointwise(normalized) => normalized.elements,
            Self::Contraction(normalized) => normalized
                .input_elements
                .iter()
                .copied()
                .max()
                .unwrap_or_default(),
            // The epilogue's own domain counts only when it actually reads a
            // declared input: a chain whose epilogue reads only the staged value
            // reads no declared input at that domain, and reporting one would
            // overstate what this output reads.
            Self::Epilogue(chain) => chain.producer.max_input_elements().max(
                if chain
                    .reads
                    .iter()
                    .any(|(read, _)| matches!(read, EpilogueRead::Input(_)))
                {
                    chain.elements
                } else {
                    0
                },
            ),
        }
    }

    pub(crate) const fn output_elements(&self) -> u64 {
        match self {
            Self::SerialSum(normalized) => normalized.output_elements,
            Self::Pointwise(normalized) => normalized.elements,
            Self::Contraction(normalized) => normalized.output_elements,
            Self::Epilogue(chain) => chain.elements,
        }
    }

    /// Returns every occurrence this output's walk claimed, in ascending order.
    pub(crate) fn members(&self) -> Vec<SemanticMemberId> {
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
    fn owns_region_members(&self, members: &[SemanticMemberId]) -> bool {
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
        }
    }

    #[cfg(test)]
    fn serial_sum_mut(&mut self) -> &mut NormalizedSerialSum {
        match self {
            Self::SerialSum(normalized) => normalized,
            Self::Pointwise(_) | Self::Contraction(_) | Self::Epilogue(_) => {
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
        members: &[SemanticMemberId],
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
    /// recognized output agrees on it.
    ///
    /// **Agreement or nothing, because the caller is sizing work.** The count
    /// scales a call over the tensor bound to that ordinal, and two outputs may
    /// read one declared input at different domains — a reduction reads it at
    /// its contributor shape while an elementwise sibling reads it at its own.
    /// Answering with either one would size a call against a tensor the region
    /// does not iterate, which is the confidently-wrong verdict a work-scaling
    /// resolution exists to prevent. A disagreement therefore yields `None` and
    /// the caller refuses, exactly as it does for an ordinal no input occupies.
    ///
    /// The two `None`s are flattened deliberately: "the outputs disagree" and
    /// "no output declares that ordinal" are different findings, and this
    /// accessor's caller acts identically on both — it refuses.
    pub(crate) fn agreed_input_elements_at(&self, ordinal: InputOrdinal) -> Option<u64> {
        agreed(
            self.outputs
                .iter()
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
    /// The size of the widest thing a plan for this request could stage, which
    /// is what a structural cost estimate wants. Deliberately a maximum rather
    /// than an agreement: a cost may be an upper bound over the whole request,
    /// and a cost that refused would turn an estimate into a feasibility gate.
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

    /// Returns every occurrence any output's walk claimed, in ascending order.
    ///
    /// The walks partition the program's occurrences — [`check_output_cover`]
    /// proves it — so this is the program's whole operation set and the
    /// deduplication is the invariant being relied on rather than a repair.
    pub(crate) fn all_members(&self) -> Vec<SemanticMemberId> {
        let mut members: Vec<SemanticMemberId> = self
            .outputs
            .iter()
            .flat_map(NormalizedOutput::members)
            .collect();
        members.sort_unstable();
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
    reads: Vec<(EpilogueRead, LogicalAccess)>,
    members: Vec<SemanticMemberId>,
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
    pub(crate) fn reads(&self) -> &[(EpilogueRead, LogicalAccess)] {
        &self.reads
    }
    /// Returns the occurrences the epilogue region itself covers.
    pub(crate) fn members(&self) -> &[SemanticMemberId] {
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
        members: &[SemanticMemberId],
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
        // The enclosing domain steps to `v5` because the recognized program
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
        bytes.extend_from_slice(b"tiler.compiler.request-subject.v5\0");
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
                for member in members {
                    bytes.extend_from_slice(&member.0.to_be_bytes());
                }
            }
            bytes.extend_from_slice(&normalized.input_elements.to_be_bytes());
            bytes.extend_from_slice(&normalized.output_elements.to_be_bytes());
            encode_elementwise_reads(bytes, &normalized.prologue_reads);
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
            push_slice(bytes, b"pointwise-f32.v4");
            push_len(bytes, normalized.input_keys.len());
            for key in &normalized.input_keys {
                push_slice(bytes, key.as_str().as_bytes());
            }
            push_slice(bytes, normalized.output_key.as_str().as_bytes());
            encode_explain_shape(bytes, &normalized.shape);
            encode_pointwise_expression(bytes, &normalized.expression);
            push_len(bytes, normalized.members.len());
            for member in &normalized.members {
                bytes.extend_from_slice(&member.0.to_be_bytes());
            }
            bytes.extend_from_slice(&normalized.elements.to_be_bytes());
            encode_elementwise_reads(bytes, &normalized.reads);
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
            for shape in &normalized.input_shapes {
                encode_explain_shape(bytes, shape);
            }
            encode_explain_shape(bytes, &normalized.output_shape);
            encode_explain_shape(bytes, &normalized.contracted_shape);
            // The canonical structure encoding, not a projection of it: the
            // index tuples are what ADR 0087 makes the operation's identity,
            // and two structures over one set of shapes are two programs.
            push_slice(bytes, normalized.structure.canonical_encoding().as_bytes());
            for position in normalized.operand_positions {
                push_len(bytes, position);
            }
            push_len(bytes, normalized.members.len());
            for member in &normalized.members {
                bytes.extend_from_slice(&member.0.to_be_bytes());
            }
            for elements in normalized.input_elements {
                bytes.extend_from_slice(&elements.to_be_bytes());
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
                    EpilogueRead::Staged => bytes.push(0x01),
                    EpilogueRead::Input(ordinal) => {
                        bytes.push(0x02);
                        bytes.extend_from_slice(&ordinal.to_be_bytes());
                    }
                }
                encode_access_relation(bytes, map);
            }
            push_len(bytes, normalized.members.len());
            for member in &normalized.members {
                bytes.extend_from_slice(&member.0.to_be_bytes());
            }
            bytes.extend_from_slice(&normalized.elements.to_be_bytes());
            encode_output_subject(bytes, &normalized.producer);
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

/// Encodes one whole-program or prologue region's read list.
///
/// The count leads, then each written entry gives its input ordinal and its
/// relation. **One read of an ordinal, addressing densely, is written as
/// nothing**: the ordinal's absence from the run is that read's canonical
/// spelling, so the empty run means "every declared input is read once, densely"
/// and every already-recognized program encodes one.
///
/// **The projection is injective, and stating why is what holds the sub-tags
/// where they are.** The declared input count is written earlier in the same
/// arm, and the read list is recovered from the run against it: an ordinal
/// absent from the run has one dense read, and an ordinal present `k` times has
/// exactly those `k` reads in run order. The one byte string that would be
/// ambiguous — a lone entry writing `LinearIdentity` — is the one this
/// projection never emits.
///
/// **And it moves no already-encodable subject's bytes.** Before a region could
/// read one declared input twice, a run held exactly the ordinals a structural
/// occurrence interposed, which is what this projection still writes for such a
/// program. A run reaches [`LogicalAccess::LinearIdentity`]'s tag only for a
/// repeated ordinal, and no subject encodable before could produce that tag
/// here at all — so per-tag injectivity closes over the widened domain, and
/// `pointwise-f32.v4` and `serial-sum-f32.v3` hold rather than step.
///
/// The relation is written through a per-variant tag and its own framed payload,
/// so two reads differing in operand shape, result shape, or any decode differ
/// in these bytes. The two structural relations get distinct tags for the reason
/// they are distinct variants: a bijection and a replication are different facts
/// about what a read consumes.
fn encode_elementwise_reads(output: &mut Vec<u8>, reads: &[(u32, LogicalAccess)]) {
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
    push_len(output, written().count());
    for (_, (ordinal, map)) in written() {
        output.extend_from_slice(&ordinal.to_be_bytes());
        encode_access_relation(output, map);
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
        let (normalized, semantic_identity) = verify_program(program, self.budgets)?;
        if self.capabilities.lowering.semantic_snapshot()
            != program.semantic_registry().snapshot_identity()
        {
            return unsupported("capability", "semantic-authority-pairing");
        }
        let Ok(realization_laws) = FrozenIndexRealizationLawRegistry::from_semantic(
            program.semantic_registry().clone(),
            self.capabilities.scalars.clone(),
        ) else {
            return unsupported("capability", "semantic-authority-pairing");
        };
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
    BudgetExceeded {
        resource: &'static str,
        limit: u32,
        actual: usize,
    },
    UnsupportedCapability {
        phase: &'static str,
        rule: &'static str,
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
        /// refusal that produced it cannot disagree; the two reasons — no
        /// installed realization at all, and an installed one that could not be
        /// proved to refine — are different findings and keep different keys.
        reason: &'static str,
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
            Self::DuplicateNumericalContract => formatter.write_str(
                "compile.request.numerics.duplicate: numerical contracts must be distinct",
            ),
            Self::TooManyNumericalContracts { actual, max } => write!(
                formatter,
                "compile.request.numerics.too-many: {actual} contracts exceeds maximum {max}"
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
                "compile.budget.{resource}: {actual} exceeds deterministic limit {limit}"
            ),
            Self::UnsupportedCapability { phase, rule } => {
                write!(
                    formatter,
                    "compile.unsupported.{phase}.{rule}: no installed capability can compile this valid semantic program"
                )
            }
            Self::UnrealizedElementaryAccuracy {
                operation,
                target_profile,
                reason,
            } => write!(
                formatter,
                "{reason}: target {target_profile} declares no realization refining the registered accuracy contract of {operation}"
            ),
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
    // target already cannot deliver. That is not hypothetical: a pure-`bf16`
    // program was refused by the recognizer's `dtype-f32` rule before any target
    // was consulted, so a profile's measured `bf16` subnormal row could never
    // produce the refusal it exists to produce, and the missing answer read as a
    // missing target fact rather than as a boundary in the wrong order.
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
                Ok(()) => match resolve_numerical_contract(&request.numerical_contracts, target) {
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

    // Recognition, then the authorities the recognized program's subject is
    // bound to. Both keep the order they had relative to each other before the
    // target phase was hoisted above them, so which refusal a program that fails
    // more than one of these reports has not moved.
    let normalized = select_supported_strategy(request.program)?;
    let semantic_identity = request.program.semantic_identity().clone();
    if request.capabilities.lowering.semantic_snapshot()
        != request.program.semantic_registry().snapshot_identity()
    {
        return unsupported("capability", "semantic-authority-pairing");
    }
    let Ok(realization_laws) = FrozenIndexRealizationLawRegistry::from_semantic(
        request.program.semantic_registry().clone(),
        request.capabilities.scalars.clone(),
    ) else {
        return unsupported("capability", "semantic-authority-pairing");
    };
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
) -> Result<(NormalizedProgram, SemanticIdentity), RequestError> {
    check_program_budgets(program, budgets)?;
    Ok((
        select_supported_strategy(program)?,
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
        "semantic-values",
        budgets.semantic_values,
        program.value_count(),
    )?;
    check_budget(
        "semantic-operations",
        budgets.semantic_operations,
        program.operation_count(),
    )?;
    // The largest shape this profile may assemble, not the smallest it might:
    // the request is admitted before any plan is chosen, so a budget that only
    // admitted the two-region materialized program would let a request through
    // and then refuse the split at assembly, reporting a caller's request as a
    // compiler-output defect. Four is the split program's own stage count with
    // its consumer — prologue, partial, final, and the elementwise epilogue that
    // reads the fold's staged result — and it is spelled rather than derived
    // because a region count belongs to a plan, and the widest plan this profile
    // assembles is that chain whatever the submitted program declares. It was
    // three while a fold's result could only be a declared program output;
    // `crate::pipeline::tests::the_widest_assembled_plan_is_the_split_reduction_with_its_epilogue`
    // is the measurement that moved it, and the neighbour in that test is what
    // attributes the fourth stage to the split rather than to the epilogue.
    check_budget("regions", budgets.regions, 4)?;
    // Derived from the declared input arity rather than spelled, because it is
    // an upper bound over every plan the request could reach and the widest of
    // those grows with the arity: the element width, one element count and one
    // byte count per declared input, the output's pair, the staged partial
    // tensor's pair, the workgroup width, and the applicability guard. One
    // input reaches nine, which is what the split program declared when this was
    // a literal, and the two-input contraction reaches nine as well by a
    // different route — it declares no partial tensor and one further input.
    check_budget(
        "host-expression-nodes",
        budgets.host_expression_nodes,
        program.input_count().saturating_mul(2).saturating_add(7),
    )?;
    // The widest buffer count any plan for this request could reach: every
    // declared input, the prologue's materialized temporary, a split's staged
    // partial tensor, and the output. A standalone elementwise program binds
    // only the first and last of those and a contraction only two inputs and an
    // output, so this bounds them too — which is what lets it be checked before
    // a strategy has been chosen. One input reaches four, the split program's
    // own number before this was derived.
    check_budget(
        "buffers",
        budgets.buffers,
        program.input_count().saturating_add(3),
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
/// properties every recognized program shares — at least one declared input,
/// `f32` throughout — are checked once and each names its own rule, and the
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
///   under `operation-set` is a chain one boundary deeper — a walk that reaches a
///   second folded value has nothing to attribute a second staged read to,
///   because `TensorRole::Intermediate` carries no ordinal.
///
/// **A reduction reading a declared input directly was the third wall here, and
/// it is gone.** `sum(x)` was refused under `reduction-prologue` because
/// `verify_access_and_semantics` required a `ScalarProgram::StrictSerialSum`
/// region's contributor access to read `TensorRole::Intermediate`;
/// `admit-a-reduction-over-a-declared-input-tensor` widened that arm to the fold's
/// *declared contributor domain*, which is the first input tensor when the program
/// folds it directly and an intermediate when a prologue region wrote it.
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
/// writes one owning tensor, so both are refused a layer down.
/// The copy stage that would lift the second is blocked in four places, and only
/// the last of them is in `tiler-ir`. A stage publishing a value another region
/// computed claims no occurrence of its own, and
/// `tiler_ir::program`'s `verify_partial_reductions` admits an uncovering stage
/// only as the declared combiner of a split — but that refusal is never reached
/// today. Measured by disabling each wall in turn against a governed spelling of
/// the published-and-consumed fixture, the order is: this rule; then
/// `crate::program`'s `cover-named-output-attribution`, because the cover places
/// the producing region as both the edge's producer and the publication's
/// retainer; then `crate::program`'s `internal-unwritten`, because that region's
/// one owning write goes to the edge and nothing writes the published value; and
/// only then `UncoveringStage`. The third is the copy stage's real absence and it
/// is a *physical and frontier* widening in this crate — the region needs a
/// second dispatch, exactly as a split reduction has one — so each individual
/// region being expressible, which `materialized_intermediate_epilogue_wall.rs`
/// measures, is necessary rather than sufficient.
/// `crates/tiler-compiler/tests/multi_output_boundary.rs` holds the evidence for
/// where that boundary now is, and
/// [`crate::pipeline::conformance`]'s
/// `a_published_and_consumed_intermediate_refuses_by_name` records the measured
/// wall order.
fn select_supported_strategy(program: &SemanticProgram) -> Result<NormalizedProgram, RequestError> {
    // Program-wide properties first, each under the rule that names it. A
    // program failing one of these fails it for every shape below, so reporting
    // it here is both the more specific statement and the only one that does not
    // depend on which occurrence happens to produce the output.
    if program.input_count() == 0 {
        return mismatch("input-arity");
    }
    if program
        .values()
        .any(|value| value.resolved_type() != &F32::resolved_type())
    {
        return mismatch("dtype-f32");
    }
    recognize_program_outputs(program)
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
fn recognize_program_outputs(program: &SemanticProgram) -> Result<NormalizedProgram, RequestError> {
    if program.output_count() == 0 {
        return unsupported("strategy", "missing-output");
    }
    let mut outputs = Vec::with_capacity(program.output_count());
    for output in program.outputs() {
        outputs.push(recognize_output(program, &output)?);
    }
    check_output_cover(program, &outputs)?;
    Ok(NormalizedProgram { outputs })
}

/// Recognizes the region partition implementing one ordered named output.
fn recognize_output(
    program: &SemanticProgram,
    output: &tiler_ir::semantic::ProgramOutputRef<'_>,
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
        recognize_reduction(program, output.value(), output.key().clone(), member, &root)
            .map(NormalizedOutput::SerialSum)
    } else if root.key() == &strict_tensor_contraction_f32_op() {
        normalize_contraction(program, output.value(), output.key().clone())
            .map(|normalized| NormalizedOutput::Contraction(Box::new(normalized)))
    } else {
        recognize_elementwise_output(program, output)
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
) -> Result<NormalizedOutput, RequestError> {
    let declared: Vec<ValueId> = program.inputs().map(|input| input.value()).collect();
    let shape = program
        .shape(output.value())
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "output-handle",
        })?
        .clone();
    if shape.rank() == 0 {
        return mismatch("elementwise-rank");
    }
    let leaves = ElementwiseLeaves {
        declared: &declared,
        staged: None,
    };
    match plan_elementwise(program, output.value(), &leaves, &shape) {
        Ok(plan) => recognize_pointwise(program, output, &declared, shape, plan)
            .map(NormalizedOutput::Pointwise),
        Err(ElementwiseRefusal::Folded(staged)) => {
            recognize_epilogue(program, output, &declared, shape, staged)
                .map(|chain| NormalizedOutput::Epilogue(Box::new(chain)))
        }
        Err(ElementwiseRefusal::Refused(error)) => Err(error),
    }
}

/// Requires the recognized walks to partition the program's occurrences.
///
/// **Two obligations, and they are separate claims about different failures.**
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
/// Claimed counts are taken over the deduplicated per-output member sets, so one
/// constant shared by two operands of the *same* walk contributes one member
/// rather than two — the normalized spelling of one program, not a duplicate.
fn check_output_cover(
    program: &SemanticProgram,
    outputs: &[NormalizedOutput],
) -> Result<(), RequestError> {
    let claimed: Vec<Vec<SemanticMemberId>> =
        outputs.iter().map(NormalizedOutput::members).collect();
    let total: usize = claimed.iter().map(Vec::len).sum();
    let mut distinct: Vec<SemanticMemberId> = claimed.iter().flatten().copied().collect();
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
    claimed: &[Vec<SemanticMemberId>],
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
            u32::try_from(*ordinal)
                .is_ok_and(|ordinal| !claimed[published].contains(&SemanticMemberId(ordinal)))
        })
        .any(|(_, operation)| operation.operands().any(|operand| operand == staged));
    crosses.then_some((published, consuming))
}

/// One recognized elementwise expression and the occurrences it covers.
struct RecognizedElementwise {
    expression: PointwiseF32Expression,
    members: Vec<SemanticMemberId>,
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
    /// to, so it is refused rather than reported as another boundary.
    Folded(ValueId),
}

impl From<ElementwiseRefusal> for RequestError {
    /// Flattens a discovered materialization boundary into the rule a caller
    /// with no epilogue to build reports for it.
    fn from(refusal: ElementwiseRefusal) -> Self {
        match refusal {
            ElementwiseRefusal::Refused(error) => error,
            ElementwiseRefusal::Folded(_) => Self::UnsupportedCapability {
                phase: "strategy",
                rule: "operation-set",
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
    members: Vec<SemanticMemberId>,
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
fn elementwise_family(
    operation: &tiler_ir::semantic::OperationRef<'_>,
) -> Option<ElementwiseFamily> {
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
/// `elementwise-expression` when the assembled expression is not one a region
/// can bind.
fn recognize_elementwise(
    program: &SemanticProgram,
    root: ValueId,
    declared: &[ValueId],
    shape: &Shape,
) -> Result<RecognizedElementwise, RequestError> {
    let plan = plan_elementwise(
        program,
        root,
        &ElementwiseLeaves {
            declared,
            staged: None,
        },
        shape,
    )
    .map_err(RequestError::from)?;
    resolve_elementwise(plan, declared)
}

/// Resolves one planned whole-program or prologue expression against the
/// declared inputs.
///
/// Declaration order is the *group* order here: the region's reads walk the
/// declared inputs in the order the ABI binds them. It is no longer a
/// one-to-one correspondence with the leaves, because one declared input may be
/// read twice — [`canonical_input_reads`] states the order the pair takes. An
/// epilogue additionally reads a staged value, and [`recognize_epilogue`] states
/// its own order rather than relaxing this one.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `elementwise-reads` for
/// a walk that did not read every declared input, `input-ordinal` for a
/// declaration position no expression ordinal can hold, and every rule
/// [`mint_elementwise`] reports.
fn resolve_elementwise(
    plan: ElementwisePlan,
    declared: &[ValueId],
) -> Result<RecognizedElementwise, RequestError> {
    let order = canonical_input_reads(&plan.leaves, declared)?;
    let expression = mint_elementwise(&plan, &order)?;
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
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] under `elementwise-reads`
/// when some declared input is read by no leaf, or some leaf reads a value that
/// is not a declared input. The first would bind a buffer the kernel never
/// loads; the second is unreachable for these walks, whose leaf set is the
/// declared inputs by construction, and is refused rather than assumed away.
fn canonical_input_reads(
    leaves: &[LeafRead],
    declared: &[ValueId],
) -> Result<Vec<LeafRead>, RequestError> {
    let mut order: Vec<LeafRead> = Vec::with_capacity(leaves.len());
    for input in declared {
        let group = order.len();
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
        if order.len() == group {
            return mismatch("elementwise-reads");
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
) -> Result<ElementwisePlan, ElementwiseRefusal> {
    let mut steps: Vec<(ValueId, ElementwiseMint)> = Vec::new();
    let mut minted: Vec<ValueId> = Vec::new();
    let mut members: Vec<SemanticMemberId> = Vec::new();
    let mut leaf_reads: Vec<LeafRead> = Vec::new();
    let mut pending = vec![(root, false)];
    while let Some((value, operands_visited)) = pending.pop() {
        if minted.contains(&value) {
            continue;
        }
        if leaves.is_leaf(value) {
            if program.shape(value).ok() != Some(shape) {
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
        if operation.key() == &constant_f32_op() {
            let (bits, _) = constant_bits(program, value).map_err(ElementwiseRefusal::Refused)?;
            members.push(SemanticMemberId(member));
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
            members.push(SemanticMemberId(member));
            steps.push((value, ElementwiseMint::Read { leaf }));
            minted.push(value);
            continue;
        }
        let Some(family) = elementwise_family(&operation) else {
            // A folding family is the *boundary* between two regions rather than
            // an unrecognizable operation: no `PointwiseF32Node` spells a sum
            // over a contributor sequence, and none ever will, because the
            // expression is a per-point body. Naming the value lets the epilogue
            // recognizer read it as the tensor an earlier region staged. A walk
            // that already reads one staged value reports the ordinary rule
            // instead: `TensorRole::Intermediate` carries no ordinal, so a second
            // staged read has nothing to attribute it to a second edge.
            if leaves.staged.is_none()
                && (operation.key() == &strict_serial_sum_f32_op()
                    || operation.key() == &strict_tensor_contraction_f32_op())
            {
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
        let value_shape = program.shape(value).ok();
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
        members.push(SemanticMemberId(member));
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

/// Mints one planned elementwise expression under a stated leaf ordering.
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
/// ordinal can hold, `elementwise-node-limit` for an expression exceeding
/// [`tiler_ir::schedule::MAX_POINTWISE_F32_EXPRESSION_NODES`], and
/// `elementwise-expression` for an assembled expression no region can bind.
fn mint_elementwise(
    plan: &ElementwisePlan,
    order: &[LeafRead],
) -> Result<PointwiseF32Expression, RequestError> {
    let mut builder = PointwiseF32ExpressionBuilder::new();
    let mut minted: Vec<(ValueId, PointwiseF32Value)> = Vec::new();
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
                builder
                    .input(InputOrdinal::new(ordinal))
                    .map_err(|_| expression_bound())?
            }
            ElementwiseMint::Constant(bits) => {
                builder.constant(*bits).map_err(|_| expression_bound())?
            }
            ElementwiseMint::Node(family, operands) => {
                let projected: Vec<PointwiseF32Value> = operands
                    .iter()
                    .map(|operand| minted_value(&minted, *operand))
                    .collect::<Result<_, _>>()?;
                match (family, projected.as_slice()) {
                    (ElementwiseFamily::Add, [lhs, rhs]) => {
                        builder.add(lhs.clone(), rhs.clone()).map_err(|_| ())
                    }
                    (ElementwiseFamily::Multiply, [lhs, rhs]) => {
                        builder.multiply(lhs.clone(), rhs.clone()).map_err(|_| ())
                    }
                    // The composition is emitted by the shared authority rather
                    // than spelled here; see [`ElementwiseFamily::Silu`].
                    (ElementwiseFamily::Silu, [argument]) => {
                        let mut sink = PointwiseExpressionSink::new(&mut builder);
                        silu_point_body(&mut sink, argument).map_err(|_| ())
                    }
                    // Unreachable through the planner's arity check, and refused
                    // rather than assumed away: an arity this projection has no
                    // case for is a vocabulary gap, not a node to invent.
                    _ => return mismatch("elementwise-arity"),
                }
                .map_err(|()| expression_bound())?
            }
        };
        minted.push((*value, node));
    }
    let root = minted_value(&minted, plan.root)?;
    builder
        .build(root)
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "elementwise-expression",
        })
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
/// **The operand must be a value this walk reads rather than computes.** A
/// structural occurrence over a value the *same region* computes would need the
/// region to address an intermediate it also produces, which this region shape
/// has no access to bind — and admitting it by materializing the intermediate
/// would add the observable rounding boundary the family's admission
/// deliberately excludes. It is refused by name. An epilogue's staged operand is
/// a different case and is admitted: another region already materialized it, so
/// the rounding boundary is the cover's rather than one this occurrence
/// introduced, and the read binds the materialization edge the cover hands the
/// region.
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
    let Ok(operand_shape) = program.shape(*operand) else {
        return mismatch("structural-operand");
    };
    // The occurrence's result is what the region iterates, so a result at any
    // other domain would make every derived divisor address the wrong window.
    let results: Vec<ValueId> = operation.results().collect();
    let [result] = results.as_slice() else {
        return mismatch("structural-arity");
    };
    if program.shape(*result).ok() != Some(shape) {
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
fn minted_value(
    minted: &[(ValueId, PointwiseF32Value)],
    value: ValueId,
) -> Result<PointwiseF32Value, RequestError> {
    minted
        .iter()
        .find(|(seen, _)| *seen == value)
        .map(|(_, node)| node.clone())
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "elementwise-operand",
        })
}

/// The refusal for an expression that exceeds the physical vocabulary's bound.
///
/// Distinct from every structural rule above: the program is elementwise and
/// well formed, and what it exceeds is
/// [`tiler_ir::schedule::MAX_POINTWISE_F32_EXPRESSION_NODES`]. Reporting it as an
/// unrecognized operation set would name the wrong property.
const fn expression_bound() -> RequestError {
    RequestError::UnsupportedCapability {
        phase: "strategy",
        rule: "elementwise-node-limit",
    }
}

/// Recognizes a whole-program elementwise `f32` expression.
///
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the unrecognized
/// property: `elementwise-rank` for a rank-zero domain no region iterates,
/// `operation-set` when a reachable occurrence is outside the recognized
/// expression, and every rule [`recognize_elementwise`] reports.
fn recognize_pointwise(
    program: &SemanticProgram,
    output: &tiler_ir::semantic::ProgramOutputRef<'_>,
    declared: &[ValueId],
    shape: Shape,
    plan: ElementwisePlan,
) -> Result<NormalizedPointwise, RequestError> {
    let recognized = resolve_elementwise(plan, declared)?;
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
/// [`declared_ordinal`] report for the epilogue's own walk, and every rule the
/// producing family's recognizer reports.
fn recognize_epilogue(
    program: &SemanticProgram,
    output: &tiler_ir::semantic::ProgramOutputRef<'_>,
    declared: &[ValueId],
    shape: Shape,
    staged: ValueId,
) -> Result<NormalizedEpilogue, RequestError> {
    let leaves = ElementwiseLeaves {
        declared,
        staged: Some(staged),
    };
    let plan =
        plan_elementwise(program, output.value(), &leaves, &shape).map_err(RequestError::from)?;
    // The staged read, then whichever declared inputs the expression names. Only
    // the inputs it reads appear, which is what an epilogue's read list differs
    // in from a whole-program one — so the group walk is spelled here rather
    // than through `canonical_input_reads`, whose `elementwise-reads` refusal is
    // exactly the rule an epilogue does not owe.
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
    let expression = mint_elementwise(&plan, &order)?;
    let reads = order
        .iter()
        .map(|leaf| {
            let read = if leaf.value == staged {
                EpilogueRead::Staged
            } else {
                EpilogueRead::Input(declared_ordinal(declared, leaf.value)?)
            };
            Ok((read, leaf.map.clone()))
        })
        .collect::<Result<Vec<_>, RequestError>>()?;
    let producer = recognize_epilogue_producer(program, staged, output.key().clone())?;
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

/// Recognizes the producer half of one epilogue chain.
///
/// The two folding families and nothing else. The refusal is not dead code
/// standing in for an impossible state: [`plan_elementwise`] names a value only
/// for these two families today, and a third family added to that discovery
/// without a producer region here must refuse rather than acquire one.
fn recognize_epilogue_producer(
    program: &SemanticProgram,
    staged: ValueId,
    output_key: OutputKey,
) -> Result<NormalizedOutput, RequestError> {
    let (member, root) = producer_for_value(program, staged)?;
    if root.key() == &strict_serial_sum_f32_op() {
        recognize_reduction(program, staged, output_key, member, &root)
            .map(NormalizedOutput::SerialSum)
    } else if root.key() == &strict_tensor_contraction_f32_op() {
        normalize_contraction(program, staged, output_key)
            .map(|normalized| NormalizedOutput::Contraction(Box::new(normalized)))
    } else {
        mismatch("operation-set")
    }
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
/// [`recognize_elementwise`] reports for the contributor walk.
fn recognize_reduction(
    program: &SemanticProgram,
    result: ValueId,
    output_key: OutputKey,
    sum_member: u32,
    sum: &tiler_ir::semantic::OperationRef<'_>,
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
    let input_shape = program
        .shape(*contributor)
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "input-handle",
        })?
        .clone();
    if input_shape.rank() == 0 {
        return mismatch("input-rank");
    }
    check_canonical_reduction_axes(&axes, input_shape.rank())?;
    let output_shape = input_shape.without_axes(&axes);
    if program.shape(result).ok() != Some(&output_shape) {
        return mismatch("sum-shape");
    }

    let recognized = recognize_elementwise(program, *contributor, &declared, &input_shape)?;
    // The walk claims an occurrence for every leaf and node it mints except one:
    // a declared input contributes the leaf that reads it and nothing else. So a
    // fold straight over a declared input arrives here with an empty member set
    // and a bare input leaf, and that leaf is the fold's own contributor read
    // rather than a prologue any region computes — which is why the condition
    // tested is the operand itself and not the emptiness that follows from it.
    let prologue = (!declared.contains(contributor)).then_some(recognized.expression);
    // The read list belongs to the prologue *region*, so a fold that has no
    // prologue states none. The walk still returns the fold's own contributor
    // read, and recording it here would describe a region no cover places.
    let prologue_reads = if prologue.is_some() {
        recognized.reads
    } else {
        Vec::new()
    };
    let members = RecognizedSerialSumMembers::new(recognized.members, sum_member);

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
    // Both declared inputs are this contraction's operands, checked below, and
    // the region binds them by *declaration* ordinal — so a program declaring a
    // third input has no ordinal for this strategy's two reads to occupy.
    if program.input_count() != 2 {
        return mismatch("input-arity");
    }
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

    // Each structure operand must be one distinct declared input, and the two
    // together must be both of them. Recorded as declaration ordinal -> operand
    // position, because declaration order is what the ABI binds buffers in and
    // is stable under any reordering of the occurrence's operands.
    let operands: Vec<ValueId> = operation.operands().collect();
    let declared: Vec<ValueId> = program.inputs().map(|input| input.value()).collect();
    let mut operand_positions = [usize::MAX; 2];
    for (position, operand) in operands.iter().enumerate() {
        let Some(declaration) = declared.iter().position(|declared| declared == operand) else {
            return mismatch("contraction-operands");
        };
        let Some(slot) = operand_positions.get_mut(declaration) else {
            return mismatch("contraction-operands");
        };
        if std::mem::replace(slot, position) != usize::MAX {
            return mismatch("contraction-operands");
        }
    }
    if operand_positions.contains(&usize::MAX) {
        return mismatch("contraction-operands");
    }

    let shape_of = |value: ValueId| -> Result<Shape, RequestError> {
        program
            .shape(value)
            .map_err(|_| RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "input-handle",
            })
            .cloned()
    };
    let input_shapes = [shape_of(declared[0])?, shape_of(declared[1])?];
    // One extent per index, bound by the first operand axis naming it. The
    // semantic inferencer already proved agreement at construction, so a
    // disagreement here is invalid state and is refused rather than preferred
    // one way.
    let mut extents: Vec<(ContractionIndex, Extent)> = Vec::new();
    for (declaration, shape) in input_shapes.iter().enumerate() {
        let tuple = structure.operand(operand_positions[declaration]).ok_or(
            RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "contraction-structure",
            },
        )?;
        if shape.rank() != tuple.len() {
            return mismatch("contraction-rank");
        }
        for (axis, index) in tuple.iter().enumerate() {
            let extent = shape.extents()[axis];
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
    if program.shape(result).ok() != Some(&output_shape) {
        return mismatch("contraction-output-shape");
    }

    let input_elements = [
        element_count_u64(&input_shapes[0], "input")?,
        element_count_u64(&input_shapes[1], "input")?,
    ];
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
        input_keys: [
            program
                .inputs()
                .next()
                .ok_or(RequestError::UnsupportedCapability {
                    phase: "strategy",
                    rule: "missing-input",
                })?
                .key()
                .clone(),
            program
                .inputs()
                .nth(1)
                .ok_or(RequestError::UnsupportedCapability {
                    phase: "strategy",
                    rule: "missing-input",
                })?
                .key()
                .clone(),
        ],
        output_key,
        input_shapes,
        output_shape,
        contracted_shape,
        structure,
        operand_positions,
        members: vec![SemanticMemberId(ordinal)],
        inputs: [declared[0], declared[1]],
        output: result,
        input_elements,
        output_elements,
        contracted_elements,
    })
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
        CanonicalValue, CanonicalValueKind, F32Add, F32Constant, F32Multiply,
        NormativeDefinitionRef, OperationArity, OperationAttributeSchema, OperationAttributes,
        OperationConformance, OperationDefinition, OperationDefinitionFacts, OperationEffect,
        OperationInferenceError, OperationInferencer, OperationSchema, ProviderDiagnosticCode,
        ProviderIdentity, RegistryError, ResolvedValueType, SemanticProgramBuilder,
        SemanticRegistryBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar,
        StrictSerialF32Sum, TypeDefinitionFacts, ValueFact, ValueTypeDefinition,
        ValueTypeDefinitionKey,
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

    /// Recognizes one program through the whole boundary, or reports the rule.
    ///
    /// Answers with the sole recognized output, because every fixture reaching
    /// it declares one; [`recognize_outputs`] is the multi-output form.
    fn recognize(program: &SemanticProgram) -> Result<NormalizedOutput, &'static str> {
        strategy_rule(select_supported_strategy(program)).map(|recognized| {
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
        strategy_rule(recognize_program_outputs(program))
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
        assert_eq!(recognized.expression.input_count(), 3);
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
        assert_eq!(recognized.expression.input_count(), 2);
        assert_eq!(recognized.members.len(), deep.operation_count());
        assert_eq!(
            recognized.expression.nodes().len(),
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
        assert_eq!(recognized.expression.input_count(), 1);
        assert_eq!(recognized.input_keys.len(), 1);
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
        assert_eq!(recognized.expression.input_count(), 2);
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
        assert_eq!(recognized.members, vec![SemanticMemberId(0)]);
        assert_eq!(recognized.expression.input_count(), 1);
        assert_eq!(recognized.expression.nodes().len(), 7);

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
        assert_eq!(chain.reads[0].0, EpilogueRead::Staged);

        // `operation-set`, from the one side the discovery deliberately does not
        // open: a fold whose *contributors* are another fold's result. The chain
        // the recognizer admits is one materialization boundary deep, so a
        // prologue reading a staged value has no second staged read to attribute
        // — `TensorRole::Intermediate` carries no ordinal — and is refused
        // rather than silently flattened. The accepted neighbour is the same
        // fold over the same scaling of the *declared input*, so the difference
        // between them is exactly where the scaled value comes from.
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
            "operation-set"
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
        let admitted = select_supported_strategy(&program).expect("the boundary admits it");
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
        let claimed: Vec<Vec<SemanticMemberId>> = recognized
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
        fn encoded(recognized: &NormalizedProgram) -> Vec<(OutputKey, Vec<SemanticMemberId>)> {
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
            verify_planned_request(CompilationRequest::governed(&invalid)),
            Err(RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "operation-set",
            })
        );
    }

    /// Builds a program declaring exactly `inputs` inputs over `operations`
    /// occurrences, so a budget's `actual` can be placed on either side of its
    /// bound.
    ///
    /// Every occurrence is one `f32` add producing one value, so
    /// `value_count() == inputs + operations`. That is the same identity the
    /// decoder layer has — no occurrence in it produces more than one value —
    /// and it is the identity `semantic_values` is sized against. The chain
    /// consumes every declared input before it starts re-reading the last, so no
    /// declared input is left unreached.
    fn budget_probe(inputs: usize, operations: usize) -> SemanticProgram {
        assert!(inputs >= 2, "the chain's first add needs two operands");
        assert!(
            operations >= inputs - 1,
            "fewer adds than inputs would leave a declared input unreached",
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
        for step in 0..operations {
            let operand = declared[(step + 1).min(inputs - 1)];
            accumulator = F32Add::apply(&mut builder, accumulator, operand).unwrap();
        }
        builder
            .output(OutputKey::new("result").unwrap(), accumulator)
            .unwrap();
        let program = builder.build().unwrap();
        assert_eq!(program.input_count(), inputs);
        assert_eq!(program.operation_count(), operations);
        assert_eq!(program.value_count(), inputs + operations);
        program
    }

    /// Each widened budget refuses the program one step past it, and the
    /// decoder layer's own measured counts are admitted.
    ///
    /// The four program-scoped bounds are sized to that layer, so the admitted
    /// neighbours are its two measured rows exactly — eighteen declared inputs
    /// over sixty-two occurrences and eighty values at the decode row, and over
    /// fifty-eight and seventy-six at the prefill row — and the decode row sits
    /// *on* all four bounds rather than under them.
    ///
    /// Refusals are observed through [`verify_program`], which is the entry the
    /// budgets guard; admission is observed at [`check_program_budgets`],
    /// because clearing the budget gate is the whole of what a budget can
    /// promise. `verify_program` still refuses the layer's *shape* at the
    /// recognizer under a rule this widening deliberately does not touch, so an
    /// admitted probe here is evidence about size and about nothing else.
    #[test]
    fn each_widened_budget_refuses_the_program_one_step_past_it() {
        let governed = DeterministicBudgets::governed();

        for (inputs, operations) in [(18, 62), (18, 58)] {
            assert_eq!(
                check_program_budgets(&budget_probe(inputs, operations), governed),
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
            verify_program(&budget_probe(19, 62), governed).err(),
            Some(RequestError::BudgetExceeded {
                resource: "semantic-values",
                limit: 80,
                actual: 81,
            }),
        );

        assert_eq!(
            verify_program(&budget_probe(17, 63), governed).err(),
            Some(RequestError::BudgetExceeded {
                resource: "semantic-operations",
                limit: 62,
                actual: 63,
            }),
        );

        assert_eq!(
            verify_program(&budget_probe(19, 18), governed).err(),
            Some(RequestError::BudgetExceeded {
                resource: "host-expression-nodes",
                limit: 43,
                actual: 45,
            }),
        );

        // `buffers` is reached only once the bound that shadows it moves, and
        // the shadowing is a property of the two bounds rather than of this
        // test: both are derived from the declared input count and both are
        // tight at eighteen, so a nineteen-input program exceeds them together
        // and the earlier check reports. The perturbation widens
        // `host_expression_nodes` to exactly what nineteen inputs need and
        // leaves `buffers` at its governed value, so what is observed refusing
        // is the governed bound.
        let unshadowed = DeterministicBudgets {
            host_expression_nodes: 45,
            ..governed
        };
        assert_eq!(
            verify_program(&budget_probe(19, 18), unshadowed).err(),
            Some(RequestError::BudgetExceeded {
                resource: "buffers",
                limit: 21,
                actual: 22,
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
