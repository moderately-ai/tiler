//! The resolved numerical contract a caller states, and its canonical key.
//!
//! A contract resolves every governed dimension for exactly one arithmetic
//! type, and its key is minted from that vector rather than chosen, so two
//! contracts that state different meanings cannot share an identity. Coherence
//! — whether a stated vector is a contract this build will hold at all — is
//! decided here, separately from whether this build can *realize* it
//! (`crate::policy`) and from whether a target can *honour* it
//! (`crate::target::honourability`).

use super::*;

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
/// exhaustive per-dimension encoding [`encode_contract`](super::subject::encode_contract) writes into a request
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
pub(super) fn canonical_contract_key(
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
            self.reciprocal_transform,
            self.approximate_intrinsics,
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
    pub(super) stated: Vec<StrictF32NumericalContract>,
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
