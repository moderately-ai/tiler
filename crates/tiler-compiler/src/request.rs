use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::sync::{Mutex, OnceLock, PoisonError};

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::index::{FrozenIndexRealizationLawRegistry, FrozenScalarRegistry};
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::schedule::{
    InputOrdinal, PointwiseF32Expression, PointwiseF32ExpressionBuilder, PointwiseF32Node,
    PointwiseF32Value,
};
use tiler_ir::semantic::{
    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CanonicalIntegerWidth, CanonicalValueView,
    ContractionIndex, ContractionIndexStructure, F32, F32_CONSTANT_BITS_ATTRIBUTE, InputKey, OpKey,
    OutputKey, ProviderIdentity, REDUCTION_AXES_ATTRIBUTE, ResolvedValueType, SemanticIdentity,
    SemanticProgram, TypeKey, ValueId, add_f32_op, constant_f32_op, multiply_f32_op,
    strict_serial_sum_f32_op, strict_tensor_contraction_f32_op,
};
use tiler_ir::shape::{Axis, Extent, Shape};

// The numerical-realization vocabulary is target-neutral and owned by the shared
// IR (ADR 0070); the compiler contract references it rather than duplicating it.
pub(crate) use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign,
    MaterializationRounding, NumericalPermission, SubnormalMode, ValueDomainProvenance,
};
use tiler_ir::schedule::{F32NumericalContractKey, NumericalContractKeyError};

use crate::capability::{
    CanonicalLoweringRegistryIdentity, FrozenLoweringCapabilityRegistry, LoweringCapabilityRevision,
};
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
    if contract.arithmetic != ArithmeticType::F32
        || contract.canonical_arithmetic_nan_bits
            != tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS
    {
        return Err(NumericalContractKeyError::InvalidArithmetic);
    }
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
    /// **`regions` and `buffers` are sized for the largest program shape this
    /// profile assembles, which is the split reduction.** `regions` was `2` —
    /// the materialized pointwise-then-reduce program's two stages — and a split
    /// replaces the single reduction dispatch with a partial pass and a final
    /// pass, so its program is three stages.
    ///
    /// `buffers` was `3`, then `4`, and is now `6`, and each step is the same
    /// derivation over a wider recognized program. Three was the one-input
    /// materialized program's input, temporary, and output; four added the
    /// split's staged partial tensor. Six is that same split over the widest
    /// prologue the *target* side admits: the governed profile declares four
    /// buffer bindings and an elementwise region binds one per declared input
    /// plus its write, so a three-input prologue is the widest feasible one, and
    /// its split program declares three inputs, the temporary, the partial
    /// tensor, and the output. It is a bound rather than a requirement —
    /// `verify_program` refuses a request whose declared arity needs more — so
    /// widening it admits program shapes and never demands them.
    ///
    /// The widening is a *deliberate* decision and not a test-enabling edit,
    /// because both numbers are inside the canonical request subject
    /// (`VerifiedRequestSubject::canonical_bytes` writes every budget), which is
    /// carried into artifact identity. Every governed compilation's request
    /// subject, and therefore every artifact identity and cache entry derived
    /// from it, moves with this change — for programs that never assemble a
    /// split as much as for ones that do, because the budget is a property of
    /// the *request* rather than of the plan chosen for it. No pinned golden
    /// encodes these bytes: every request-subject assertion in the corpus is
    /// relational (a mutated budget must not reconstruct its authority), so the
    /// move is invisible to the suite and is stated here instead.
    ///
    /// A budget is an upper bound, so widening admits program shapes and never
    /// requires them: `verify_program` still refuses a request whose shape needs
    /// more, and `verify_host_contract` still refuses a program whose value
    /// count exceeds `buffers`.
    pub(crate) const fn governed() -> Self {
        Self {
            semantic_values: 16,
            semantic_operations: 8,
            regions: 3,
            host_expression_nodes: 32,
            buffers: 6,
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
/// The prologue set is always nonempty. A reduction whose contributor tensor is
/// a declared input needs no prologue region, but the schedule IR's
/// `StrictSerialSum` region requires its contributor access to read
/// `TensorRole::Intermediate`, so this profile has no region for that shape and
/// [`recognize_reduction`] refuses it at the boundary instead.
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedSerialSum {
    pub(crate) input_keys: Vec<InputKey>,
    pub(crate) output_key: OutputKey,
    /// The contributor domain: the shape the prologue writes and the fold reads.
    pub(crate) input_shape: Shape,
    pub(crate) output_shape: Shape,
    pub(crate) reduction_axes: Vec<Axis>,
    /// The recognized elementwise prologue the fold's contributors come from.
    pub(crate) prologue: PointwiseF32Expression,
    pub(crate) members: RecognizedSerialSumMembers,
    pub(crate) inputs: Vec<ValueId>,
    pub(crate) pointwise_result: ValueId,
    pub(crate) output: ValueId,
    pub(crate) input_elements: u64,
    pub(crate) output_elements: u64,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NormalizedProgram {
    SerialSum(NormalizedSerialSum),
    Pointwise(NormalizedPointwise),
    /// Boxed because a contraction carries two operand shapes, an output shape,
    /// a contracted shape, and a validated index structure — roughly twice the
    /// serial sum's payload — and every value of this enum would otherwise pay
    /// for the widest variant.
    Contraction(Box<NormalizedContraction>),
}

impl NormalizedProgram {
    pub(crate) const fn serial_sum(&self) -> &NormalizedSerialSum {
        match self {
            Self::SerialSum(normalized) => normalized,
            Self::Pointwise(_) | Self::Contraction(_) => {
                panic!("request is not a serial-sum program")
            }
        }
    }

    pub(crate) const fn try_serial_sum(&self) -> Option<&NormalizedSerialSum> {
        match self {
            Self::SerialSum(normalized) => Some(normalized),
            Self::Pointwise(_) | Self::Contraction(_) => None,
        }
    }

    pub(crate) const fn pointwise(&self) -> Option<&NormalizedPointwise> {
        match self {
            Self::SerialSum(_) | Self::Contraction(_) => None,
            Self::Pointwise(normalized) => Some(normalized),
        }
    }

    pub(crate) const fn contraction(&self) -> Option<&NormalizedContraction> {
        match self {
            Self::SerialSum(_) | Self::Pointwise(_) => None,
            Self::Contraction(normalized) => Some(normalized),
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
        }
    }

    /// Returns the largest declared input element count.
    ///
    /// The size of the widest thing a plan for this request could stage, which
    /// is what a structural cost estimate and a pre-strategy budget both want.
    /// It is deliberately not "the input's element count": a contraction has two
    /// inputs of different extents and no single answer to that question.
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
        }
    }

    pub(crate) const fn output_elements(&self) -> u64 {
        match self {
            Self::SerialSum(normalized) => normalized.output_elements,
            Self::Pointwise(normalized) => normalized.elements,
            Self::Contraction(normalized) => normalized.output_elements,
        }
    }

    pub(crate) fn all_members(&self) -> Vec<SemanticMemberId> {
        match self {
            Self::SerialSum(normalized) => normalized.members.all(),
            Self::Pointwise(normalized) => normalized.members.clone(),
            Self::Contraction(normalized) => normalized.members.clone(),
        }
    }

    #[cfg(test)]
    fn serial_sum_mut(&mut self) -> &mut NormalizedSerialSum {
        match self {
            Self::SerialSum(normalized) => normalized,
            Self::Pointwise(_) | Self::Contraction(_) => panic!("the fixture is a serial sum"),
        }
    }
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
    prologue: PointwiseF32Expression,
    members: RecognizedSerialSumMembers,
    input_elements: u64,
    output_elements: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NormalizedProgramSubject {
    SerialSum(NormalizedSerialSumSubject),
    Pointwise(NormalizedPointwise),
    /// Boxed for the reason [`NormalizedProgram::Contraction`] is.
    Contraction(Box<NormalizedContraction>),
}

impl VerifiedTargetRequest {
    pub(crate) const fn normalized(&self) -> &NormalizedProgram {
        &self.normalized
    }

    pub(crate) const fn serial_sum(&self) -> &NormalizedSerialSum {
        self.normalized.serial_sum()
    }

    pub(crate) const fn try_serial_sum(&self) -> Option<&NormalizedSerialSum> {
        self.normalized.try_serial_sum()
    }

    pub(crate) const fn pointwise(&self) -> Option<&NormalizedPointwise> {
        self.normalized.pointwise()
    }

    pub(crate) const fn contraction(&self) -> Option<&NormalizedContraction> {
        self.normalized.contraction()
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
        // The enclosing domain steps to `v4` because the installed independent
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
        bytes.extend_from_slice(b"tiler.compiler.request-subject.v4\0");
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
        match &self.normalized {
            NormalizedProgramSubject::SerialSum(normalized) => {
                push_slice(&mut bytes, b"serial-sum-f32.v2");
                push_len(&mut bytes, normalized.input_keys.len());
                for key in &normalized.input_keys {
                    push_slice(&mut bytes, key.as_str().as_bytes());
                }
                push_slice(&mut bytes, normalized.output_key.as_str().as_bytes());
                encode_explain_shape(&mut bytes, &normalized.input_shape);
                encode_explain_shape(&mut bytes, &normalized.output_shape);
                push_len(&mut bytes, normalized.reduction_axes.len());
                for axis in &normalized.reduction_axes {
                    bytes.extend_from_slice(&axis.get().to_be_bytes());
                }
                encode_pointwise_expression(&mut bytes, &normalized.prologue);
                for members in [
                    normalized.members.pointwise(),
                    normalized.members.reduction(),
                ] {
                    push_len(&mut bytes, members.len());
                    for member in members {
                        bytes.extend_from_slice(&member.0.to_be_bytes());
                    }
                }
                bytes.extend_from_slice(&normalized.input_elements.to_be_bytes());
                bytes.extend_from_slice(&normalized.output_elements.to_be_bytes());
            }
            NormalizedProgramSubject::Pointwise(normalized) => {
                // The sub-tag steps to `v3` because this arm's shape changed
                // again: a fixed root family, child family, association, and
                // three leaves became the general expression the recognizer now
                // admits. A `v2` pointwise subject can never be read as a `v3`
                // one.
                push_slice(&mut bytes, b"pointwise-f32.v3");
                push_len(&mut bytes, normalized.input_keys.len());
                for key in &normalized.input_keys {
                    push_slice(&mut bytes, key.as_str().as_bytes());
                }
                push_slice(&mut bytes, normalized.output_key.as_str().as_bytes());
                encode_explain_shape(&mut bytes, &normalized.shape);
                encode_pointwise_expression(&mut bytes, &normalized.expression);
                push_len(&mut bytes, normalized.members.len());
                for member in &normalized.members {
                    bytes.extend_from_slice(&member.0.to_be_bytes());
                }
                bytes.extend_from_slice(&normalized.elements.to_be_bytes());
            }
            // A third sub-tag rather than a step of the enclosing
            // `request-subject.v2` domain: neither existing arm's bytes move, so
            // a subject encoded before this variant existed still encodes to
            // exactly what it did, and a reader that reaches this tag is reading
            // a subject the earlier vocabulary could not express.
            NormalizedProgramSubject::Contraction(normalized) => {
                push_slice(&mut bytes, b"contraction-f32.v1");
                push_len(&mut bytes, normalized.input_keys.len());
                for key in &normalized.input_keys {
                    push_slice(&mut bytes, key.as_str().as_bytes());
                }
                push_slice(&mut bytes, normalized.output_key.as_str().as_bytes());
                for shape in &normalized.input_shapes {
                    encode_explain_shape(&mut bytes, shape);
                }
                encode_explain_shape(&mut bytes, &normalized.output_shape);
                encode_explain_shape(&mut bytes, &normalized.contracted_shape);
                // The canonical structure encoding, not a projection of it: the
                // index tuples are what ADR 0087 makes the operation's identity,
                // and two structures over one set of shapes are two programs.
                push_slice(
                    &mut bytes,
                    normalized.structure.canonical_encoding().as_bytes(),
                );
                for position in normalized.operand_positions {
                    push_len(&mut bytes, position);
                }
                push_len(&mut bytes, normalized.members.len());
                for member in &normalized.members {
                    bytes.extend_from_slice(&member.0.to_be_bytes());
                }
                for elements in normalized.input_elements {
                    bytes.extend_from_slice(&elements.to_be_bytes());
                }
                bytes.extend_from_slice(&normalized.output_elements.to_be_bytes());
                bytes.extend_from_slice(&normalized.contracted_elements.to_be_bytes());
            }
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
/// the reason [`subnormal_tag`] states: a node added to the vocabulary must stop
/// the build here rather than silently encode under a neighbour's tag.
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
    /// The recognized elementwise prologue the fold's contributors come from.
    pub(crate) const fn prologue(&self) -> &PointwiseF32Expression {
        &self.prologue
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
    let normalized = match normalized {
        NormalizedProgram::SerialSum(normalized) => {
            NormalizedProgramSubject::SerialSum(NormalizedSerialSumSubject {
                input_keys: normalized.input_keys.clone(),
                output_key: normalized.output_key.clone(),
                input_shape: normalized.input_shape.clone(),
                output_shape: normalized.output_shape.clone(),
                reduction_axes: normalized.reduction_axes.clone(),
                prologue: normalized.prologue.clone(),
                members: normalized.members.clone(),
                input_elements: normalized.input_elements,
                output_elements: normalized.output_elements,
            })
        }
        NormalizedProgram::Pointwise(normalized) => {
            NormalizedProgramSubject::Pointwise(normalized.clone())
        }
        NormalizedProgram::Contraction(normalized) => {
            NormalizedProgramSubject::Contraction(normalized.clone())
        }
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
    let (normalized, semantic_identity) = verify_program(request.program, request.budgets)?;
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
    let dispatch_types = canonical_program_value_types(request.program);

    // Resolve every structurally admitted target independently. A profile that
    // honours no stated contract is a target-local outcome, not a reason to
    // discard the other ordered slots. Intrinsic profile/authority failures
    // remain outer request errors because no target outcome can make malformed
    // input valid.
    let target_slots = request
        .target_profiles
        .iter()
        .map(|target| {
            let resolution = match require_compile_profile_dispatch(target, &dispatch_types) {
                Ok(()) => match resolve_numerical_contract(&request.numerical_contracts, target) {
                    Ok(numerical_contract) => {
                        let authority = request_subject(
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
                        );
                        VerifiedTargetResolution::Resolved {
                            numerical_contract,
                            authority: Box::new(authority),
                        }
                    }
                    Err(error @ RequestError::NoResolvableNumericalContract { .. }) => {
                        VerifiedTargetResolution::Rejected(error)
                    }
                    Err(error) => return Err(error),
                },
                Err(error @ RequestError::DTypeNotDispatchable { .. }) => {
                    VerifiedTargetResolution::Rejected(error)
                }
                Err(error) => return Err(error),
            };
            Ok(VerifiedTargetSlot {
                target_profile: target.clone(),
                resolution,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(VerifiedCompilationRequest {
        normalized,
        semantic_identity,
        numerical_contracts: request.numerical_contracts,
        budgets: request.budgets,
        target_slots,
        capabilities: request.capabilities,
        realization_laws,
    })
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
    // compiler-output defect. Three regions and four buffers are the split
    // program's pointwise, partial, and final stages over its input, temporary,
    // partial, and output values.
    check_budget("regions", budgets.regions, 3)?;
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
    Ok((
        select_supported_strategy(program)?,
        program.semantic_identity().clone(),
    ))
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
/// exactly one output, `f32` throughout — are checked once and each names its
/// own rule, and the program's shape is then decided by *the occurrence that
/// produces the output*, walked outward through the occurrences that feed it.
/// A program whose exact shape nothing here was taught is admitted when every
/// occurrence it contains is one the physical layer can realize and they compose
/// into a region chain it can assemble; nothing asks whether the whole graph
/// matches a spelling.
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
/// Recognition may only admit what the physical layer can express, so three
/// walls below this boundary are refused *at* it, each under its own rule:
///
/// - **An operation the region vocabulary cannot spell** (`operation-set`).
///   `tiler::silu-f32@1`, `tiler::reindex-f32@1`, and `tiler::broadcast-f32@1`
///   each have a registered *lowering* capability, but no
///   [`tiler_ir::schedule::ScalarProgram`] or
///   [`tiler_ir::schedule::LogicalAccess`] spells them, and decomposing one here
///   into expression nodes would be this boundary re-deriving a provider's
///   lowering — exactly what occurrence refinement exists to prevent.
///   `admit-the-registered-unary-families-at-the-compiler-request-boundary`
///   owns it.
/// - **An elementwise stage reading a materialized intermediate**
///   (`operation-set` from the contraction cover, `elementwise-shape` or
///   `operation-set` from the elementwise walk). Every elementwise region this
///   profile builds reads declared input tensors and nothing else, so a
///   contraction or a reduction feeding an elementwise epilogue has no region
///   to be assembled into. That is a gap in the physical layer rather than in
///   the schedule vocabulary — `tiler_ir::schedule::TensorRole::Intermediate`
///   is a per-region role, so nothing about it forbids the chain — which is
///   exactly why it is refused here instead of admitted and then dropped
///   mid-pipeline. `admit-elementwise-epilogues-over-a-materialized-intermediate`
///   owns the widening.
/// - **A reduction reading a declared input directly** (`reduction-prologue`).
///   `tiler-ir`'s schedule verifier requires a `StrictSerialSum` region's
///   contributor access to read `TensorRole::Intermediate`, so this profile has
///   no region for `sum(x)`; `admit-a-reduction-over-a-declared-input-tensor`
///   owns it.
///
/// Which refusal a rejected program reports is settled by the occurrence it
/// actually ends in rather than by enumeration order: a program whose output is
/// a reduction gets the reduction's reason, one whose output is a contraction
/// gets the contraction's, and any other gets the elementwise walk's.
///
/// # The one refusal here that is *not* a wall below this boundary
///
/// **Multiple outputs** (`output-arity`) is refused for a different reason than
/// the three above, and reading it as a physical-vocabulary gap sends the
/// widening at the wrong crate. `tiler-ir` already expresses ordered
/// multi-output: [`tiler_ir::program::KernelProgramBuilder::push_output`] is
/// general and bounded by [`tiler_ir::program::MAX_PROGRAM_OUTPUTS`] rather than
/// by one, its verifier already rejects a plan naming fewer outputs than the
/// program declares, and its own tests build and verify a two-output program.
/// A region writing one owning tensor is not the obstruction either — several
/// regions write several, and the program layer binds each stage's buffers to
/// values positionally, which [`tiler_ir::program::ValueRole::fills`] states.
///
/// What is missing is *this crate's* planner: `program.rs`'s artifact
/// refinement matches the scheduled regions against three fixed strategy
/// shapes, all single-output pipelines, and nothing upstream produces a cover
/// assigning regions to several ordered outputs. `implement-general-dag-partitioning`
/// closing condition 2 owns exactly that. Relaxing this guard before it lands
/// could only admit a program the planner cannot cover — failing mid-pipeline
/// instead of refusing here, which is strictly worse than refusing.
/// `crates/tiler-compiler/tests/multi_output_boundary.rs` holds the evidence.
fn select_supported_strategy(program: &SemanticProgram) -> Result<NormalizedProgram, RequestError> {
    // Program-wide properties first, each under the rule that names it. A
    // program failing one of these fails it for every shape below, so reporting
    // it here is both the more specific statement and the only one that does not
    // depend on which occurrence happens to produce the output.
    if program.input_count() == 0 {
        return mismatch("input-arity");
    }
    if program.output_count() != 1 {
        return mismatch("output-arity");
    }
    if program
        .values()
        .any(|value| value.resolved_type() != &F32::resolved_type())
    {
        return mismatch("dtype-f32");
    }
    let output = program
        .outputs()
        .next()
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "missing-output",
        })?;
    // A program whose output *is* a declared input computes nothing: it names no
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
        recognize_reduction(program, &output, member, &root).map(NormalizedProgram::SerialSum)
    } else if root.key() == &strict_tensor_contraction_f32_op() {
        normalize_contraction(program)
            .map(|normalized| NormalizedProgram::Contraction(Box::new(normalized)))
    } else {
        recognize_pointwise(program, &output).map(NormalizedProgram::Pointwise)
    }
}

/// One recognized elementwise expression and the occurrences it covers.
struct RecognizedElementwise {
    expression: PointwiseF32Expression,
    members: Vec<SemanticMemberId>,
}

/// The elementwise operation families this recognizer spells directly.
///
/// Exactly the families that are both a registered lowering capability *and* a
/// node of the physical expression vocabulary. `tiler::silu-f32@1` fails the
/// second half — it lowers to a subtree rather than a node — and is refused
/// under `operation-set` rather than expanded here.
#[derive(Clone, Copy)]
enum ElementwiseFamily {
    Add,
    Multiply,
}

/// Classifies one operation as a recognized elementwise family, or declines.
fn elementwise_family(
    operation: &tiler_ir::semantic::OperationRef<'_>,
) -> Option<ElementwiseFamily> {
    if operation.key() == &add_f32_op() {
        Some(ElementwiseFamily::Add)
    } else if operation.key() == &multiply_f32_op() {
        Some(ElementwiseFamily::Multiply)
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
    let mut builder = PointwiseF32ExpressionBuilder::new();
    let mut minted: Vec<(ValueId, PointwiseF32Value)> = Vec::new();
    let mut members: Vec<SemanticMemberId> = Vec::new();
    let mut reads: BTreeSet<u32> = BTreeSet::new();
    let mut pending = vec![(root, false)];
    while let Some((value, operands_visited)) = pending.pop() {
        if minted.iter().any(|(seen, _)| *seen == value) {
            continue;
        }
        if let Some(position) = declared.iter().position(|input| *input == value) {
            let ordinal =
                u32::try_from(position).map_err(|_| RequestError::UnsupportedCapability {
                    phase: "strategy",
                    rule: "input-ordinal",
                })?;
            if program.shape(value).ok() != Some(shape) {
                return mismatch("elementwise-shape");
            }
            let leaf = builder
                .input(InputOrdinal::new(ordinal))
                .map_err(|_| expression_bound())?;
            reads.insert(ordinal);
            minted.push((value, leaf));
            continue;
        }
        let (member, operation) = producer_for_value(program, value)?;
        if operation.results().collect::<Vec<_>>() != [value] {
            return mismatch("elementwise-result-arity");
        }
        if operation.key() == &constant_f32_op() {
            let (bits, _) = constant_bits(program, value)?;
            let leaf = builder.constant(bits).map_err(|_| expression_bound())?;
            members.push(SemanticMemberId(member));
            minted.push((value, leaf));
            continue;
        }
        let Some(family) = elementwise_family(&operation) else {
            return mismatch("operation-set");
        };
        // A recognized elementwise operation of this profile is attribute-free.
        // An attribute is a semantic fact the expression does not carry forward,
        // so admitting one would silently drop it.
        if !operation.attributes().fields().is_empty() {
            return mismatch("elementwise-attributes");
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
            return mismatch("elementwise-shape");
        }
        let operands: Vec<ValueId> = operation.operands().collect();
        let [lhs, rhs] = operands.as_slice() else {
            return mismatch("elementwise-arity");
        };
        if !operands_visited {
            pending.push((value, true));
            pending.push((*rhs, false));
            pending.push((*lhs, false));
            continue;
        }
        let lhs = minted_value(&minted, *lhs)?;
        let rhs = minted_value(&minted, *rhs)?;
        let node = match family {
            ElementwiseFamily::Add => builder.add(lhs, rhs),
            ElementwiseFamily::Multiply => builder.multiply(lhs, rhs),
        }
        .map_err(|_| expression_bound())?;
        members.push(SemanticMemberId(member));
        minted.push((value, node));
    }
    // Every declared input must be read. One that is not would bind a buffer the
    // kernel never loads, and the expression's own dense-ordinal rule would
    // refuse the assembled expression anyway — this reports the property rather
    // than the consequence.
    if reads.len() != declared.len() {
        return mismatch("elementwise-reads");
    }
    let root = minted_value(&minted, root)?;
    let expression = builder
        .build(root)
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "elementwise-expression",
        })?;
    members.sort_unstable();
    members.dedup();
    Ok(RecognizedElementwise {
        expression,
        members,
    })
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
) -> Result<NormalizedPointwise, RequestError> {
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
    let recognized = recognize_elementwise(program, output.value(), &declared, &shape)?;
    // The recognized occurrences must cover the program exactly. A built program
    // retains only output-reachable operations, so an uncovered one is work this
    // region would silently drop.
    if recognized.members.len() != program.operation_count() {
        return mismatch("operation-set");
    }
    let elements = element_count_u64(&shape, "input")?;
    Ok(NormalizedPointwise {
        input_keys: program.inputs().map(|input| input.key().clone()).collect(),
        output_key: output.key().clone(),
        shape,
        expression: recognized.expression,
        members: recognized.members,
        inputs: declared,
        output: output.value(),
        elements,
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
/// # Errors
///
/// Returns [`RequestError::UnsupportedCapability`] naming the unrecognized
/// property: `sum-signature`, `sum-output`, `sum-shape`, `sum-axes*`, and
/// `input-rank` for the reduction itself, `reduction-prologue` when the fold
/// reads a declared input and this profile has no region for it,
/// `operation-set` when the recognized occurrences do not cover the program, and
/// every rule [`recognize_elementwise`] reports for the prologue.
fn recognize_reduction(
    program: &SemanticProgram,
    output: &tiler_ir::semantic::ProgramOutputRef<'_>,
    sum_member: u32,
    sum: &tiler_ir::semantic::OperationRef<'_>,
) -> Result<NormalizedSerialSum, RequestError> {
    let declared: Vec<ValueId> = program.inputs().map(|input| input.value()).collect();
    let sum_operands: Vec<_> = sum.operands().collect();
    let [contributor] = sum_operands.as_slice() else {
        return mismatch("sum-signature");
    };
    if sum.results().collect::<Vec<_>>() != [output.value()] {
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
    if program.shape(output.value()).ok() != Some(&output_shape) {
        return mismatch("sum-shape");
    }

    // The fold's contributors come from an elementwise prologue this recognizer
    // materializes.
    //
    // A reduction whose operand is a *declared input* — `sum(x)`, the simplest
    // fold there is — is refused here rather than admitted, and the wall is
    // below this boundary rather than in this vocabulary: `tiler-ir`'s
    // `verify_access_and_semantics` requires a `ScalarProgram::StrictSerialSum`
    // region's contributor access to read `TensorRole::Intermediate`, so a
    // region reading the input directly is rejected as malformed by the schedule
    // verifier. Synthesizing an identity prologue to satisfy it is not the
    // alternative: that would add a materialization, and its observable rounding
    // boundary, that the caller's program never asked for.
    // `admit-a-reduction-over-a-declared-input-tensor` owns the widening.
    if declared.contains(contributor) {
        return mismatch("reduction-prologue");
    }
    let prologue = recognize_elementwise(program, *contributor, &declared, &input_shape)?;
    let members = RecognizedSerialSumMembers::new(prologue.members, sum_member);
    check_recognized_operation_cover(program, &members)?;

    let input_elements = element_count_u64(&input_shape, "input")?;
    let output_elements = element_count_u64(&output_shape, "output")?;
    Ok(NormalizedSerialSum {
        input_keys: program.inputs().map(|input| input.key().clone()).collect(),
        output_key: output.key().clone(),
        input_shape,
        output_shape,
        reduction_axes: axes,
        prologue: prologue.expression,
        members,
        inputs: declared,
        pointwise_result: *contributor,
        output: output.value(),
        input_elements,
        output_elements,
    })
}

/// Recognized operation count of the bounded contraction strategy.
///
/// Exactly the contraction itself. Its operands are tensors rather than
/// constants, so — unlike the two elementwise strategies — no constant operation
/// belongs to this shape, and an extra reachable operation is work this region
/// would silently drop.
const CONTRACTION_OPERATIONS: usize = 1;

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
fn normalize_contraction(program: &SemanticProgram) -> Result<NormalizedContraction, RequestError> {
    if program.input_count() != 2 {
        return mismatch("input-arity");
    }
    // Exactly the contraction occurrence and nothing else. An elementwise
    // epilogue over a contraction result is a two-region chain this profile
    // cannot assemble: every elementwise region it builds reads declared input
    // tensors, and none reads a materialized intermediate. Refusing here is
    // what keeps the boundary from admitting a program that dies mid-pipeline;
    // `admit-elementwise-epilogues-over-a-materialized-intermediate` owns it.
    if program.operation_count() != CONTRACTION_OPERATIONS {
        return mismatch("operation-set");
    }
    let output = program
        .outputs()
        .next()
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "missing-output",
        })?;
    let (ordinal, operation) =
        producer(program, output.value(), &strict_tensor_contraction_f32_op())?;
    if operation.results().collect::<Vec<_>>() != [output.value()] {
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
    if program.shape(output.value()).ok() != Some(&output_shape) {
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
        output_key: output.key().clone(),
        input_shapes,
        output_shape,
        contracted_shape,
        structure,
        operand_positions,
        members: vec![SemanticMemberId(ordinal)],
        inputs: [declared[0], declared[1]],
        output: output.value(),
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

/// Requires the recognized occurrences to cover the whole program exactly.
///
/// A built program retains only output-reachable operations, so demanding that
/// the reachable count equal the distinct recognized set rejects any operation
/// the recognized prologue and reduction do not claim. One constant shared by
/// two operands is the normalized spelling of the same program and contributes
/// one member rather than two, which is why this compares against the
/// deduplicated set rather than against a spelled-out count.
fn check_recognized_operation_cover(
    program: &SemanticProgram,
    recognized: &RecognizedSerialSumMembers,
) -> Result<(), RequestError> {
    if program.operation_count() != recognized.all().len() {
        return mismatch("operation-set");
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

fn unsupported<T>(phase: &'static str, rule: &'static str) -> Result<T, RequestError> {
    Err(RequestError::UnsupportedCapability { phase, rule })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
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
    fn recognize(program: &SemanticProgram) -> Result<NormalizedProgram, &'static str> {
        select_supported_strategy(program).map_err(|error| match error {
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
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
        let normalized = verified.normalized.serial_sum();
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
        let prologue = &normalized.prologue;
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

        let NormalizedProgram::SerialSum(recognized) =
            recognize(&program).expect("the composed program is recognized")
        else {
            panic!("a program whose output is a reduction recognizes as one");
        };
        assert_eq!(recognized.input_keys.len(), 3);
        assert_eq!(recognized.input_shape, Shape::from_dims([2, 3]));
        assert_eq!(recognized.output_shape, Shape::from_dims([2]));
        assert_eq!(
            recognized.prologue.input_count(),
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
        let verified = verify_request(CompilationRequest::governed(&program))
            .unwrap()
            .for_target(0)
            .unwrap();
        assert_eq!(crate::physical::fused_prologue_constants(&verified), None);
    }

    /// A reduction over a declared input refuses, because no region reads one.
    ///
    /// `sum(x)` is the simplest fold there is, and this is the one place in this
    /// change where the wall is genuinely below the boundary: `tiler-ir`'s
    /// `verify_access_and_semantics` requires a `ScalarProgram::StrictSerialSum`
    /// region's contributor access to read `TensorRole::Intermediate`, so a
    /// region reading the input directly is rejected by the schedule verifier as
    /// malformed compiler output. Admitting it here and failing there is the
    /// failure mode the precedent declined to ship, so it refuses at the
    /// boundary under its own rule.
    ///
    /// Its accepted neighbour is the same fold over the same input with one
    /// elementwise occurrence between them, so what the rule reads is the
    /// missing prologue and not the fold.
    #[test]
    fn a_reduction_over_a_declared_input_refuses_for_its_missing_prologue() {
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
        assert_eq!(recognize(&bare).unwrap_err(), "reduction-prologue");

        let neighbour = fold(true);
        assert!(matches!(
            recognize(&neighbour),
            Ok(NormalizedProgram::SerialSum(_))
        ));
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
        let NormalizedProgram::Pointwise(recognized) =
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
        let NormalizedProgram::Pointwise(recognized) =
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
        let NormalizedProgram::Pointwise(recognized) =
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

        // `output-arity`: two named outputs over one admitted expression. The
        // neighbour is the same graph naming only the root.
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
        assert_eq!(recognize(&two_outputs).unwrap_err(), "output-arity");

        // `operation-set`: a registered family the expression vocabulary cannot
        // spell. `tiler::silu-f32@1` has a lowering capability, and no node.
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
        assert_eq!(recognize(&unary).unwrap_err(), "operation-set");

        // `operation-set` again, from the other side: a contraction with a
        // reachable elementwise epilogue. Its accepted neighbour is the bare
        // contraction, and the difference between them is exactly the epilogue
        // this profile has no region to assemble.
        let contraction = contraction_program(false);
        assert!(matches!(
            recognize(&contraction),
            Ok(NormalizedProgram::Contraction(_))
        ));
        let with_epilogue = contraction_program(true);
        assert_eq!(recognize(&with_epilogue).unwrap_err(), "operation-set");
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
                rule: "operation-set",
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
            let verified = verify_request(CompilationRequest::governed_preferring(
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
        assert_eq!(verify_request(empty), Err(RequestError::EmptyTargetSet));

        let mut duplicate = CompilationRequest::governed(&program);
        duplicate.target_profiles.push(TargetProfile::governed());
        assert_eq!(
            verify_request(duplicate),
            Err(RequestError::DuplicateTargetProfile)
        );
    }

    #[test]
    fn verified_request_receipts_reject_post_verification_mutation() {
        let program = program();
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
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
            affine_expression(3.0_f32.to_bits(), 1.0_f32.to_bits());
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
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
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
            affine_expression(2.0_f32.to_bits(), 1.0_f32.to_bits() ^ 1);
        assert!(!forged.reconstructs_its_authority());

        let mut forged = target;
        forged.normalized.serial_sum_mut().input_keys = vec![InputKey::new("forged").unwrap()];
        assert!(!forged.reconstructs_its_authority());
    }

    #[test]
    fn used_provider_revision_changes_admission_and_snapshot_subjects() {
        let first = governed_test_program(1);
        let second = governed_test_program(2);
        let first = verify_request(request_with_matching_empty_capabilities(&first)).unwrap();
        let second = verify_request(request_with_matching_empty_capabilities(&second)).unwrap();

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
        let first = verify_request(request_with_matching_empty_capabilities(&first)).unwrap();
        let second = verify_request(request_with_matching_empty_capabilities(&second)).unwrap();

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
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
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
