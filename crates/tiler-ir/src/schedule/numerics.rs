//! Target-neutral numerical-realization vocabulary for scheduled regions.
//!
//! A scheduled region preserves the declared numerical contract of the
//! computation it implements (ADR 0007). These types describe that contract in
//! target-neutral terms so both the compiler request boundary and the schedule
//! IR share one vocabulary rather than duplicating it.
//!
//! The vocabulary is the one ADR 0019 and ADR 0011 accept: subnormal input and
//! subnormal result handling are independent dimensions, each resolving to
//! preservation or an explicit flush-to-zero behaviour, and each numeric
//! transform is an independently resolved permission. A target that couples two
//! of these dimensions in one execution mode declares that coupling on its own
//! profile; it never collapses the semantic dimensions here (ADR 0019).
//!
//! # Two layers, deliberately not one type
//!
//! [`NumericalRealization`] is the declaration a *scheduled region* preserves,
//! and it carries exactly the dimensions a region's operations can vary. The
//! caller's resolved numerical *contract* is wider: `docs/numerical-semantics.md`
//! makes it complete over every governed dimension, because completeness is what
//! makes an unenumerated dimension fail closed rather than being trivially
//! satisfied. The remaining behaviour spaces a complete contract needs are
//! defined below —  [`ArithmeticType`], [`ExceptionalValueAssumption`],
//! [`ApproximationEnvelope`], and [`MaterializationRounding`] — and they live
//! here rather than in a consumer because the numerical vocabulary is
//! target-neutral and owned by the shared IR (ADR 0070), so the compiler contract
//! and a backend declaration reference one definition instead of duplicating two.
//!
//! **Three distinct claims, kept apart.** A behaviour space defined here is a
//! *type-system reservation*. A dimension carried by [`NumericalRealization`] is
//! *implemented* in the region IR. A dimension some admitted operation can
//! actually consume, whose resolution changes an observable result, is the only
//! one for which a *tested guarantee* is even reachable. This module supplies the
//! first; it does not claim either of the others.
//!
//! None of these enums is `#[non_exhaustive]`, and that is load-bearing rather
//! than incidental. Every consumer that encodes one into canonical identity or
//! matches one to decide target support does so with an exhaustive match, so
//! widening the vocabulary is a build error at each such site instead of a
//! silent identity collision or a silently dropped obligation (ADR 0074
//! convention 5b, ADR 0076 item 6).

/// The zero a flush-to-zero behaviour produces.
///
/// A flush-to-zero mode that does not state which zero it produces cannot be
/// checked against measured hardware and cannot be reference-evaluated, because
/// binary32 has two zeros and they are observably different values (ADR 0076
/// item 1). The sign is carried here, on the behaviour itself, rather than
/// resolved from a separate signed-zero permission: a permission may leave the
/// sign of a zero *unspecified*, and an unspecified flush result is exactly the
/// under-specification this vocabulary exists to remove.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FlushedZeroSign {
    /// The produced zero carries the sign of the value it replaced.
    ///
    /// **Measurement.** Apple M4 Max, macOS 27.0, `Apple metal version
    /// 32023.883`: an emitted `x * 2.0f` returns `0x80000000` for the operand
    /// `0x80400000`, not `0x00000000`.
    PreservesSign,
    /// Every flushed value produces positive zero regardless of its own sign.
    AlwaysPositive,
}

/// Treatment of subnormal floating-point values at each arithmetic operation.
///
/// Both dimensions are **per-operation** rules, not boundary rules, and
/// `docs/numerical-semantics.md` is the authority: input flushing treats an
/// existing subnormal operand as zero *before arithmetic*, and result flushing
/// replaces a *newly produced* subnormal result with zero. A store and a load
/// are neither, so a materialization boundary neither adds nor removes a flush
/// — which is why fusing a region does not change exceptional-value behaviour
/// under any resolution of these dimensions.
///
/// The two dimensions of [`NumericalRealization`] that use this type — inputs
/// and results — are resolved independently (ADR 0019).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SubnormalMode {
    /// Subnormal values are preserved exactly, retaining gradual underflow.
    Preserve,
    /// Subnormal values are replaced by a zero of the stated sign.
    ///
    /// For the input dimension this treats an existing subnormal operand as
    /// zero before arithmetic; for the result dimension it replaces a newly
    /// produced subnormal result. The two are observably different behaviours
    /// and neither implies the other.
    FlushToZero {
        /// Which zero the flush produces.
        zero_sign: FlushedZeroSign,
    },
}

/// Whether a region's arithmetic is bounded away from the subnormal range.
///
/// A declared [`SubnormalMode`] states what a region's arithmetic *means*; this
/// states whether that meaning is observable at all. When no operand and no
/// result of a region's arithmetic in some type can be subnormal, every
/// resolution of that type's subnormal dimensions returns identical bits, so a
/// target whose behaviour differs from the declared one has nothing to differ
/// about. This is a value-domain fact, not a permission: it never authorizes a
/// substitution, it records that the substitution question is vacuous.
///
/// # Derived, never declared
///
/// Nothing constructs this from a caller's assertion. It is a total function of
/// a [`super::VerifiedScheduledRegion`]
/// ([`super::VerifiedScheduledRegion::subnormal_freedom`]), computed from the
/// region's *verified* scalar program, and a region that does not carry the
/// evidence gets [`Self::Unproven`]. A settable witness would let a producer
/// declare a freedom its values do not have, which is the one failure this type
/// must not permit.
///
/// Not `#[non_exhaustive]`, for the reason stated at the top of this module.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SubnormalFreedom {
    /// Nothing bounds this region's arithmetic away from the subnormal range.
    ///
    /// The declared subnormal modes are obligations a target must realize, and
    /// a target that resolves either dimension differently is a hard gap.
    Unproven,
    /// Strict-affine decoding of a value whose scale the type requires normal.
    ///
    /// **Exhaustive over the finite code domain, not sampled.** The `i32`
    /// subtraction of two codes in the inclusive `u8` domain is exact and
    /// cannot overflow. Converting a value of magnitude at most 255 to `f32` is
    /// exact, so the converted operand is `+0.0` or has magnitude at least
    /// `1.0`, and is never subnormal. The remaining operation is the multiply
    /// by the scale: when the codes are equal the exact product is `+0.0`,
    /// which the registered exceptional contract already requires, and
    /// otherwise the product has magnitude at least the scale. So a product
    /// below the `f32` minimum normal requires a scale below it, and
    /// `crate::semantic::ENCODED_NUMERIC_SCALE_DOMAIN` declares the scale
    /// `positive-normal-f32`.
    ///
    /// **Measurement.** Finding 32 of
    /// `docs/research/apple-targets/numerical-behaviour.md`, run 2026-07-31 on
    /// the `apple9-f32-unified-msl4-macos26` row, dispatched this exact chain:
    /// all 1,310,720 normal-scale cells returned bits identical to the exact
    /// rational reference, `code == zero_point` returned `+0.0` in 256/256
    /// diagonal cells of every case, and at a subnormal scale the flush acted
    /// on the *operand* — where the derivation places it — while at the minimum
    /// normal nothing flushed. The boundary is that finding's: one GPU family,
    /// one toolchain and flag row, `u8` codes, one non-overflowing subtraction,
    /// no packed extraction, no timing.
    ///
    /// The claim is about `f32` and nothing else; see [`Self::discharges`].
    StrictAffineNormalScaleDecode,
}

impl SubnormalFreedom {
    /// Returns whether this freedom discharges the subnormal obligation for one
    /// arithmetic type.
    ///
    /// Typed rather than boolean, and both matches are exhaustive, because a
    /// freedom established for one type says nothing about another: the decode
    /// derivation rests on `f32`'s exponent range and on integers up to 255
    /// being exactly representable in `f32`, and neither premise transfers to a
    /// narrower format. A region emitting `f16` arithmetic under a decode's
    /// freedom must still record its gap.
    #[must_use]
    pub const fn discharges(self, arithmetic: ArithmeticType) -> bool {
        match self {
            Self::Unproven => false,
            Self::StrictAffineNormalScaleDecode => match arithmetic {
                ArithmeticType::F32 => true,
                ArithmeticType::F16 | ArithmeticType::Bf16 | ArithmeticType::F64 => false,
            },
        }
    }
}

/// Whether a numeric-reshaping transform is permitted by the contract.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NumericalPermission {
    /// The transform is forbidden and must not change observable results.
    Forbidden,
    /// The transform is permitted and its results may differ from the strict
    /// reading.
    ///
    /// A permission is granted per dimension and never implies another: one
    /// permitted transform authorizes exactly the freedom it names (ADR 0011).
    Permitted,
}

/// The declared numerical realization a scheduled region must preserve.
///
/// The fields are read-transparent value data: a producer may read or assemble
/// one, but only the checked schedule builder can bind it into a
/// [`super::VerifiedScheduledRegion`].
///
/// # Why `profile_key` is `&'static str`
///
/// **Decided, not provisional.** This is compiler IR, and the only thing that
/// mints one is a compiling build whose contract keys are its own compile-time
/// constants. The spelling is what the key *is* on this side of the
/// serialization boundary, and it is what keeps this record `Copy` and
/// `const fn`-constructible across the schedule layer's value-semantic call
/// sites.
///
/// A decoded artifact carries the same four dimensions with an *owned* key,
/// because a key read from bytes is not a compile-time constant. That is
/// `tiler-artifact`'s dispatch record, and the two are not duplicates: decoding
/// produces a dispatch record rather than reconstructed compiler IR, so nothing
/// converts one into the other. `own-the-numerical-realization-profile-key`
/// settled this and records what would reopen it — something needing to turn a
/// decoded artifact back into schedulable IR, which would cost this type its
/// `Copy` and `const fn` to serve a use the accepted policy excludes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NumericalRealization {
    /// Stable key of the governing numerical contract.
    pub profile_key: &'static str,
    /// Canonical arithmetic NaN bit pattern for produced values.
    pub canonical_arithmetic_nan_bits: u32,
    /// Treatment of subnormal inputs.
    pub input_subnormals: SubnormalMode,
    /// Treatment of subnormal results.
    pub result_subnormals: SubnormalMode,
    /// Whether contraction (e.g. fused multiply-add) is permitted.
    pub contraction: NumericalPermission,
    /// Whether ordered reassociation of one same-operation operand sequence is permitted.
    pub reassociation: NumericalPermission,
    /// Whether reduction contributors may be permuted.
    pub permutation: NumericalPermission,
    /// Whether observable signed-zero distinctions may be eliminated.
    pub signed_zero: NumericalPermission,
    /// Whether NaN values may be assumed absent.
    pub nan_assumptions: ExceptionalValueAssumption,
    /// Whether infinity values may be assumed absent.
    pub infinity_assumptions: ExceptionalValueAssumption,
}

impl NumericalRealization {
    /// Assembles a numerical realization from its declared parts.
    #[must_use]
    #[allow(
        clippy::too_many_arguments,
        reason = "every consumable numerical dimension is an explicit required argument so widening the contract breaks every constructor instead of silently defaulting a new obligation"
    )]
    pub const fn new(
        profile_key: &'static str,
        canonical_arithmetic_nan_bits: u32,
        input_subnormals: SubnormalMode,
        result_subnormals: SubnormalMode,
        contraction: NumericalPermission,
        reassociation: NumericalPermission,
        permutation: NumericalPermission,
        signed_zero: NumericalPermission,
        nan_assumptions: ExceptionalValueAssumption,
        infinity_assumptions: ExceptionalValueAssumption,
    ) -> Self {
        Self {
            profile_key,
            canonical_arithmetic_nan_bits,
            input_subnormals,
            result_subnormals,
            contraction,
            reassociation,
            permutation,
            signed_zero,
            nan_assumptions,
            infinity_assumptions,
        }
    }

    /// Returns whether contraction is permitted by this realization.
    #[must_use]
    pub const fn permits_contraction(self) -> bool {
        permits(self.contraction)
    }

    /// Returns whether ordered reassociation is permitted by this realization.
    #[must_use]
    pub const fn permits_reassociation(self) -> bool {
        permits(self.reassociation)
    }

    /// Returns whether contributor permutation is permitted.
    #[must_use]
    pub const fn permits_permutation(self) -> bool {
        permits(self.permutation)
    }

    /// Returns whether signed-zero elimination is permitted.
    #[must_use]
    pub const fn permits_signed_zero_elimination(self) -> bool {
        permits(self.signed_zero)
    }
}

/// Returns whether a permission grants its transform.
///
/// Matched exhaustively rather than written as a negated `matches!`, so a
/// widened [`NumericalPermission`] stops the build here instead of being
/// silently classified with `Forbidden`.
const fn permits(permission: NumericalPermission) -> bool {
    match permission {
        NumericalPermission::Forbidden => false,
        NumericalPermission::Permitted => true,
    }
}

/// A floating-point arithmetic type a numerical behaviour is declared for.
///
/// **Subnormal behaviour is measurably per-dtype, not per-target.** On one Apple
/// row — same GPU, same math modes, modules declaring `air.compile.denorms_disable`
/// identically — `f32` arithmetic flushes subnormals while `f16` arithmetic
/// preserves them, and `bf16` flushes. A declaration keyed by dimension alone
/// therefore has to state one of them wrongly, which is why every honourability
/// key in this workspace carries an arithmetic type beside its dimension (ADR
/// 0076 boundary item 3).
///
/// **This names a dtype; it does not claim support for one.** Recognition and
/// operation support are separate (`docs/numerical-semantics.md`): a variant here
/// is an identity a contract or a profile may speak about, not an assertion that
/// this build admits arithmetic in it.
///
/// The durable identity is [`Self::canonical_type_key`], the same namespaced,
/// versioned nominal spelling a [`crate::semantic::TypeKey`] renders. The variant
/// is a const-usable stand-in for that string, not a second identity system:
/// structural facts such as bit width are descriptor facts and are deliberately
/// absent, because two formats can share a width and differ in bias, special
/// values, or encoding.
///
/// Not `#[non_exhaustive]`, for the reason stated at the top of this module: every
/// consumer that encodes one into canonical identity or matches one to decide
/// target support does so exhaustively, so widening this set is a build error at
/// each such site.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ArithmeticType {
    /// IEEE-754 binary16.
    F16,
    /// The 8-bit-exponent brain floating-point format.
    Bf16,
    /// IEEE-754 binary32.
    F32,
    /// IEEE-754 binary64.
    F64,
}

impl ArithmeticType {
    /// Every recognized arithmetic type, in canonical order.
    pub const ALL: [Self; 4] = [Self::F16, Self::Bf16, Self::F32, Self::F64];

    /// Returns the canonical namespaced, versioned type-key spelling.
    ///
    /// This is the durable dtype identity, rendered exactly as
    /// [`crate::semantic::TypeKey`]'s `Display` renders it. The correspondence is
    /// checked by a test rather than asserted here, so a divergence between the
    /// two spellings fails the build's own gate instead of being discovered by a
    /// consumer comparing two identities that were supposed to be one.
    #[must_use]
    pub const fn canonical_type_key(self) -> &'static str {
        match self {
            Self::F16 => "tiler::f16@1",
            Self::Bf16 => "tiler::bf16@1",
            Self::F32 => "tiler::f32@1",
            Self::F64 => "tiler::f64@1",
        }
    }

    /// Returns the canonical tag naming this type in an identity encoding.
    ///
    /// Written by an exhaustive match rather than read from the discriminant, so
    /// adding or reordering a variant is a build error here instead of a silent
    /// change to every identity ever produced (ADR 0074 convention 3).
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::F16 => 0x01,
            Self::Bf16 => 0x02,
            Self::F32 => 0x03,
            Self::F64 => 0x04,
        }
    }
}

/// The provenance class of a value-domain fact a contract relies on.
///
/// `docs/numerical-semantics.md` requires that every value-domain fact used for
/// correctness carry explicit provenance, and that a caller-declared but
/// unvalidated fact be ineligible to justify a correctness-sensitive rewrite.
/// Carrying the class on the assumption itself is what keeps that rule checkable:
/// an assumption with no provenance is indistinguishable from a proven one, and
/// the difference is exactly what decides whether a rewrite may consume it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ValueDomainProvenance {
    /// Derived soundly from verified producers, constants, or analysis, and
    /// usable without a runtime check.
    CompilerProven,
    /// Established by a guard or validation computation before any plan that
    /// relies on it executes.
    RuntimeValidated,
    /// Recorded and explainable, but ineligible to justify a
    /// correctness-sensitive rewrite.
    CallerDeclaredUnvalidated,
}

/// Whether a contract authorizes assuming an exceptional value absent.
///
/// NaN-result semantics and permission to *assume* NaNs absent are distinct, and
/// so are the NaN and infinity dimensions: one never implies the other (ADR
/// 0011). This type resolves the assumption half; the produced NaN bit pattern is
/// [`NumericalRealization::canonical_arithmetic_nan_bits`] and is a different
/// question.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExceptionalValueAssumption {
    /// The exceptional value may occur and every operation defines its result.
    MakeNoAssumption,
    /// The value is assumed absent, on the stated evidence.
    AssumeAbsent {
        /// How the absence was established.
        provenance: ValueDomainProvenance,
    },
}

/// The accuracy envelope an approximate-intrinsic permission resolves to.
///
/// `docs/numerical-semantics.md` is explicit that this dimension "resolves to a
/// maximum accuracy envelope before semantic optimization, **not a boolean** or a
/// later license to weaken meaning". A `NumericalPermission` would therefore be
/// the wrong space for it: `Permitted` alone states no bound, and an unbounded
/// approximation is not a contract a reference evaluation or a backend intrinsic
/// can be checked against.
/// A governed maximum accuracy envelope, named rather than spelled inline.
///
/// Each variant denotes an **immutable versioned accuracy clause** whose key
/// [`Self::key`] returns. The variant is the identity; a tolerance stated inline
/// would let a contract be widened without changing what the contract *is*,
/// which is the one thing an accuracy clause must not permit.
///
/// The set is closed and widening it is a `tiler-ir` change, exactly as widening
/// any other behaviour vocabulary here is. That is what keeps the widening a
/// build error at every exhaustive site rather than a free-form string a
/// consumer can mint without anyone noticing that no target declares it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ApproximationEnvelope {
    /// Approximate intrinsics are forbidden; every elementary function follows
    /// its own resolved accuracy contract.
    ///
    /// That contract has both a carrier and, since `tiler::silu-f32@1`,
    /// operations carrying one: [`crate::semantic::accuracy::AccuracyContract`]
    /// is the ADR 0042 vocabulary this resolution defers to, and the activation's
    /// definition facts carry a resolved instance of it for the subordinate
    /// exponential while `tiler::rms-norm-f32@1`'s carry one for the subordinate
    /// reciprocal square root. So this variant is no longer forbidding a
    /// relaxation nothing could state — it names real obligations on real
    /// operations, and the two are stated in *different contract forms*, which is
    /// why neither may be read off the other.
    Forbidden,
    /// Permitted up to the backend-elementary envelope.
    ///
    /// `docs/numerical-semantics.md` defines this conformance level as one where
    /// "operation graph is preserved but elementary function results follow the
    /// backend contract". It bounds the approximation by the backend's own stated
    /// accuracy rather than by a Tiler-side numeric tolerance, so a backend that
    /// states none cannot honour it.
    ///
    /// **Not reachable for either operation that could consume it.** The admitted
    /// activation and the admitted RMS normalization both withhold this dimension
    /// from their compiler capability rows — see
    /// `ELEMENTARY_UNCARRIED_DIMENSIONS` in `crates/tiler-compiler/src/policy.rs`
    /// — because [`NumericalRealization`] cannot record which resolution a region
    /// chose, so two contracts differing here would share one identity. Widening
    /// the realization is what makes this variant consumable rather than merely
    /// statable.
    BackendElementary,
}

impl ApproximationEnvelope {
    /// The canonical key naming this resolution.
    ///
    /// The **one** definition of these strings. [`Self::envelope_key`] is derived
    /// from it rather than repeating them, so a renamed envelope cannot be
    /// renamed in one place and not the other.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Forbidden => "approximation.forbidden",
            Self::BackendElementary => "tiler::backend-elementary@1",
        }
    }

    /// The versioned key of the envelope this resolution authorizes, if any.
    ///
    /// `None` for [`Self::Forbidden`], which authorizes no envelope at all — a
    /// different claim from authorizing an empty one, and the reason this is not
    /// simply [`Self::key`].
    #[must_use]
    pub const fn envelope_key(self) -> Option<&'static str> {
        match self {
            Self::Forbidden => None,
            Self::BackendElementary => Some(Self::BackendElementary.key()),
        }
    }

    /// Returns the canonical tag naming this resolution in an identity encoding.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::Forbidden => 0x01,
            Self::BackendElementary => 0x02,
        }
    }
}

/// The rounding an observable materialization boundary applies.
///
/// A cast or quantization boundary is observable even when fusion removes the
/// physical store and reload that would otherwise have realized it, so the
/// rounding it applies is part of what the program means rather than a storage
/// choice. This resolves that rounding.
///
/// **One admitted direction, deliberately.** `docs/numerical-semantics.md` fixes
/// the initial contract at round-to-nearest, ties-to-even and states that
/// directed rounding arrives as *new typed operation contracts*; inventing a
/// second variant here would introduce a semantics the normative text has not
/// defined. The enum exists rather than the direction being implicit because an
/// unstated rounding cannot be checked against a target that rounds otherwise,
/// and because every encoder matches it exhaustively — so the day a second
/// direction is admitted, every identity and feasibility site that must account
/// for it fails the build instead of silently carrying the old assumption.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MaterializationRounding {
    /// Round to nearest, ties to even.
    NearestTiesToEven,
}

/// Versioned domain of the canonical coherent `f32` numerical-contract key.
pub const F32_NUMERICAL_CONTRACT_KEY_DOMAIN: &str = "tiler.contract.f32.v2";

/// Versioned domain of the canonical coherent `bf16` numerical-contract key.
///
/// **A sibling domain rather than a widening of the `f32` one, and the
/// separation is what keeps every existing key byte-identical.** A contract is
/// stated for exactly one [`ArithmeticType`] because subnormal behaviour is
/// measurably per-dtype, so two contracts resolving the same dimensions for
/// different widths are different contracts and must not share a key. Putting
/// `bf16` under its own domain means no `f32` preimage, length, or rendering
/// moves: the two grammars are disjoint by their first character difference in
/// the domain, and a reader holding either can tell which it has before decoding
/// a byte.
///
/// It opens at `v1` rather than `v2` because the version counts *this* domain's
/// rendering revisions and this is its first. The `f32` domain reached `v2` by
/// replacing a preset-naming scheme it never shared with this one.
pub const BF16_NUMERICAL_CONTRACT_KEY_DOMAIN: &str = "tiler.contract.bf16.v1";

// The two exceptional-value rows are each either three bytes (no assumption)
// or four bytes (caller-declared absence). All other rows have fixed width, so
// these are the complete legal rendered lengths. Checking this before scanning
// or decoding bounds caller-controlled work and allocation.
const F32_NUMERICAL_CONTRACT_KEY_LENGTHS: [usize; 3] = [98, 100, 102];

// The same three shapes, over a domain one byte longer and a canonical NaN
// payload two bytes shorter: `bf16`'s pattern is sixteen bits wide where
// `f32`'s is thirty-two.
const BF16_NUMERICAL_CONTRACT_KEY_LENGTHS: [usize; 3] = [95, 97, 99];

/// A validated canonical key for one coherent `f32` numerical contract.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct F32NumericalContractKey(Box<str>);

/// A validated canonical key for one coherent `bf16` numerical contract.
///
/// A distinct type rather than an arithmetic-tagged one, for the same reason the
/// domains are distinct: every site that holds a contract key holds it for one
/// width, and a type that could be either would let a `bf16` key reach a
/// consumer whose exhaustive `f32` reasoning was written before `bf16` existed.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Bf16NumericalContractKey(Box<str>);

/// Why a numerical-contract key is not a canonical coherent v2 `f32` key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum NumericalContractKeyError {
    /// The domain, separator, lowercase-hex rendering, or exact vector grammar is invalid.
    ///
    /// A key rendered under a *different* governed domain falls here rather than
    /// under [`Self::InvalidArithmetic`]: the domain is checked before the
    /// preimage, so an `f32` key offered to the `bf16` parser is refused for its
    /// spelling before its arithmetic tag is ever read.
    InvalidCanonicalKey,
    /// The vector names an arithmetic type or canonical NaN payload other than
    /// the one its domain governs.
    InvalidArithmetic,
    /// The vector contains an assumption provenance that a caller-stated contract cannot use.
    IncoherentAssumption,
}

impl F32NumericalContractKey {
    /// Mints the canonical key from the complete coherent contract vector.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal for an assumption whose provenance cannot be
    /// stated by a caller. The arithmetic type and canonical arithmetic NaN
    /// payload are invariants of this `f32`-specific constructor.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input_subnormals: SubnormalMode,
        result_subnormals: SubnormalMode,
        contraction: NumericalPermission,
        reassociation: NumericalPermission,
        permutation: NumericalPermission,
        signed_zero: NumericalPermission,
        reciprocal_transform: NumericalPermission,
        approximate_intrinsics: ApproximationEnvelope,
        nan_assumptions: ExceptionalValueAssumption,
        infinity_assumptions: ExceptionalValueAssumption,
        materialization_rounding: MaterializationRounding,
    ) -> Result<Self, NumericalContractKeyError> {
        let mut bytes = vec![ArithmeticType::F32.tag()];
        bytes.extend_from_slice(&crate::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS.to_be_bytes());
        encode_contract_dimensions(
            &mut bytes,
            input_subnormals,
            result_subnormals,
            contraction,
            reassociation,
            permutation,
            signed_zero,
            reciprocal_transform,
            approximate_intrinsics,
            nan_assumptions,
            infinity_assumptions,
            materialization_rounding,
        )?;
        Ok(Self(
            render_contract_key(F32_NUMERICAL_CONTRACT_KEY_DOMAIN, &bytes).into_boxed_str(),
        ))
    }

    /// Validates and retains one already-rendered canonical key.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal unless `key` is the exact lowercase canonical
    /// spelling of one coherent caller-statable `f32` contract.
    pub fn try_from_str(key: &str) -> Result<Self, NumericalContractKeyError> {
        Self::try_from_str_with_decoder(key, decode_hex_pair)
    }

    fn try_from_str_with_decoder(
        key: &str,
        decode: impl FnMut(u8, u8) -> Option<u8>,
    ) -> Result<Self, NumericalContractKeyError> {
        let bytes = decode_contract_key(
            key,
            F32_NUMERICAL_CONTRACT_KEY_DOMAIN,
            &F32_NUMERICAL_CONTRACT_KEY_LENGTHS,
            decode,
        )?;
        let cursor = validate_contract_header(
            &bytes,
            ArithmeticType::F32,
            &crate::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS.to_be_bytes(),
        )?;
        validate_dimension_rows(&bytes, cursor)?;
        Ok(Self(key.into()))
    }

    /// Returns the canonical key spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the arithmetic type established by the validated grammar.
    #[must_use]
    pub const fn arithmetic(&self) -> ArithmeticType {
        ArithmeticType::F32
    }
}

impl Bf16NumericalContractKey {
    /// Mints the canonical key from the complete coherent contract vector.
    ///
    /// The dimension rows are encoded by the same writer the `f32` key uses, so
    /// a widened behaviour space is one build error rather than two encoders
    /// that can drift. Only the header differs, and it is what makes the two
    /// key spaces disjoint: the arithmetic tag and the canonical arithmetic NaN
    /// payload of *this* width.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal for an assumption whose provenance cannot be
    /// stated by a caller. The arithmetic type and canonical arithmetic NaN
    /// payload are invariants of this `bf16`-specific constructor.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input_subnormals: SubnormalMode,
        result_subnormals: SubnormalMode,
        contraction: NumericalPermission,
        reassociation: NumericalPermission,
        permutation: NumericalPermission,
        signed_zero: NumericalPermission,
        reciprocal_transform: NumericalPermission,
        approximate_intrinsics: ApproximationEnvelope,
        nan_assumptions: ExceptionalValueAssumption,
        infinity_assumptions: ExceptionalValueAssumption,
        materialization_rounding: MaterializationRounding,
    ) -> Result<Self, NumericalContractKeyError> {
        let mut bytes = vec![ArithmeticType::Bf16.tag()];
        bytes.extend_from_slice(&crate::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS.to_be_bytes());
        encode_contract_dimensions(
            &mut bytes,
            input_subnormals,
            result_subnormals,
            contraction,
            reassociation,
            permutation,
            signed_zero,
            reciprocal_transform,
            approximate_intrinsics,
            nan_assumptions,
            infinity_assumptions,
            materialization_rounding,
        )?;
        Ok(Self(
            render_contract_key(BF16_NUMERICAL_CONTRACT_KEY_DOMAIN, &bytes).into_boxed_str(),
        ))
    }

    /// Validates and retains one already-rendered canonical key.
    ///
    /// # Errors
    ///
    /// Returns a typed refusal unless `key` is the exact lowercase canonical
    /// spelling of one coherent caller-statable `bf16` contract. An `f32` key is
    /// refused here, and a `bf16` key is refused by
    /// [`F32NumericalContractKey::try_from_str`]; neither domain accepts the
    /// other's rendering.
    pub fn try_from_str(key: &str) -> Result<Self, NumericalContractKeyError> {
        Self::try_from_str_with_decoder(key, decode_hex_pair)
    }

    fn try_from_str_with_decoder(
        key: &str,
        decode: impl FnMut(u8, u8) -> Option<u8>,
    ) -> Result<Self, NumericalContractKeyError> {
        let bytes = decode_contract_key(
            key,
            BF16_NUMERICAL_CONTRACT_KEY_DOMAIN,
            &BF16_NUMERICAL_CONTRACT_KEY_LENGTHS,
            decode,
        )?;
        let cursor = validate_contract_header(
            &bytes,
            ArithmeticType::Bf16,
            &crate::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS.to_be_bytes(),
        )?;
        validate_dimension_rows(&bytes, cursor)?;
        Ok(Self(key.into()))
    }

    /// Returns the canonical key spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the arithmetic type established by the validated grammar.
    #[must_use]
    pub const fn arithmetic(&self) -> ArithmeticType {
        ArithmeticType::Bf16
    }
}

/// Writes the eleven dimension rows in canonical order.
///
/// The **one** writer for both key domains. A dimension added to the vocabulary,
/// or a behaviour space widened, is a build error here once rather than a
/// divergence between two per-width encoders — which is the failure mode a
/// second copy of this function would introduce the first time only one of them
/// was updated.
#[allow(clippy::too_many_arguments)]
fn encode_contract_dimensions(
    bytes: &mut Vec<u8>,
    input_subnormals: SubnormalMode,
    result_subnormals: SubnormalMode,
    contraction: NumericalPermission,
    reassociation: NumericalPermission,
    permutation: NumericalPermission,
    signed_zero: NumericalPermission,
    reciprocal_transform: NumericalPermission,
    approximate_intrinsics: ApproximationEnvelope,
    nan_assumptions: ExceptionalValueAssumption,
    infinity_assumptions: ExceptionalValueAssumption,
    materialization_rounding: MaterializationRounding,
) -> Result<(), NumericalContractKeyError> {
    push_dimension(bytes, 0x01, |bytes| {
        encode_subnormal(bytes, input_subnormals);
        Ok(())
    })?;
    push_dimension(bytes, 0x02, |bytes| {
        encode_subnormal(bytes, result_subnormals);
        Ok(())
    })?;
    for (tag, permission) in [
        (0x03, contraction),
        (0x04, reassociation),
        (0x05, permutation),
        (0x06, signed_zero),
        (0x07, reciprocal_transform),
    ] {
        push_dimension(bytes, tag, |bytes| {
            encode_permission(bytes, permission);
            Ok(())
        })?;
    }
    push_dimension(bytes, 0x08, |bytes| {
        bytes.extend_from_slice(&[0x03, approximate_intrinsics.tag()]);
        Ok(())
    })?;
    push_dimension(bytes, 0x09, |bytes| {
        encode_assumption(bytes, nan_assumptions)
    })?;
    push_dimension(bytes, 0x0a, |bytes| {
        encode_assumption(bytes, infinity_assumptions)
    })?;
    push_dimension(bytes, 0x0b, |bytes| match materialization_rounding {
        MaterializationRounding::NearestTiesToEven => {
            bytes.extend_from_slice(&[0x05, 0x01]);
            Ok(())
        }
    })
}

/// Renders one preimage under its versioned domain as lowercase hex.
fn render_contract_key(domain: &str, bytes: &[u8]) -> String {
    let mut key = String::with_capacity(domain.len() + 1 + bytes.len() * 2);
    key.push_str(domain);
    key.push('.');
    for byte in bytes {
        use core::fmt::Write as _;
        write!(key, "{byte:02x}").expect("writing to a String cannot fail");
    }
    key
}

/// Recovers the preimage of one rendered key under its own domain.
///
/// The length check runs before any scanning or allocation, so caller-controlled
/// work stays bounded, and it is per domain because the two headers differ in
/// width.
fn decode_contract_key(
    key: &str,
    domain: &str,
    lengths: &[usize],
    mut decode: impl FnMut(u8, u8) -> Option<u8>,
) -> Result<Vec<u8>, NumericalContractKeyError> {
    if !lengths.contains(&key.len()) {
        return Err(NumericalContractKeyError::InvalidCanonicalKey);
    }
    let Some(hex) = key.strip_prefix(domain) else {
        return Err(NumericalContractKeyError::InvalidCanonicalKey);
    };
    let Some(hex) = hex.strip_prefix('.') else {
        return Err(NumericalContractKeyError::InvalidCanonicalKey);
    };
    if hex.is_empty()
        || !hex.len().is_multiple_of(2)
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(NumericalContractKeyError::InvalidCanonicalKey);
    }
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| decode(pair[0], pair[1]))
        .collect::<Option<Vec<_>>>()
        .ok_or(NumericalContractKeyError::InvalidCanonicalKey)
}

fn push_dimension(
    bytes: &mut Vec<u8>,
    tag: u8,
    encode: impl FnOnce(&mut Vec<u8>) -> Result<(), NumericalContractKeyError>,
) -> Result<(), NumericalContractKeyError> {
    bytes.push(tag);
    encode(bytes)
}

fn encode_subnormal(bytes: &mut Vec<u8>, mode: SubnormalMode) {
    bytes.extend_from_slice(&[
        0x01,
        match mode {
            SubnormalMode::Preserve => 0x01,
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            } => 0x02,
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            } => 0x03,
        },
    ]);
}

fn encode_permission(bytes: &mut Vec<u8>, permission: NumericalPermission) {
    bytes.extend_from_slice(&[
        0x02,
        match permission {
            NumericalPermission::Forbidden => 0x01,
            NumericalPermission::Permitted => 0x02,
        },
    ]);
}

fn encode_assumption(
    bytes: &mut Vec<u8>,
    assumption: ExceptionalValueAssumption,
) -> Result<(), NumericalContractKeyError> {
    bytes.push(0x04);
    match assumption {
        ExceptionalValueAssumption::MakeNoAssumption => bytes.push(0x01),
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
        } => bytes.extend_from_slice(&[0x02, 0x03]),
        ExceptionalValueAssumption::AssumeAbsent { .. } => {
            return Err(NumericalContractKeyError::IncoherentAssumption);
        }
    }
    Ok(())
}

fn decode_hex_pair(high: u8, low: u8) -> Option<u8> {
    fn nibble(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        }
    }
    Some(nibble(high)? << 4 | nibble(low)?)
}

/// Checks the arithmetic tag and canonical NaN payload opening one preimage.
///
/// Returns the cursor the dimension rows start at, which is where the two
/// widths' preimages differ in length and the only place they do.
fn validate_contract_header(
    bytes: &[u8],
    arithmetic: ArithmeticType,
    canonical_nan: &[u8],
) -> Result<usize, NumericalContractKeyError> {
    let cursor = 1 + canonical_nan.len();
    if bytes.first() != Some(&arithmetic.tag()) || bytes.get(1..cursor) != Some(canonical_nan) {
        return Err(NumericalContractKeyError::InvalidArithmetic);
    }
    Ok(cursor)
}

fn validate_dimension_rows(
    bytes: &[u8],
    mut cursor: usize,
) -> Result<(), NumericalContractKeyError> {
    for (tag, space, values) in [
        (0x01, 0x01, &[0x01, 0x02, 0x03][..]),
        (0x02, 0x01, &[0x01, 0x02, 0x03][..]),
        (0x03, 0x02, &[0x01, 0x02][..]),
        (0x04, 0x02, &[0x01, 0x02][..]),
        (0x05, 0x02, &[0x01, 0x02][..]),
        (0x06, 0x02, &[0x01, 0x02][..]),
        (0x07, 0x02, &[0x01, 0x02][..]),
        (0x08, 0x03, &[0x01, 0x02][..]),
    ] {
        if bytes
            .get(cursor..cursor + 3)
            .is_none_or(|row| row[0] != tag || row[1] != space || !values.contains(&row[2]))
        {
            return Err(NumericalContractKeyError::InvalidCanonicalKey);
        }
        cursor += 3;
    }
    for tag in [0x09, 0x0a] {
        if bytes
            .get(cursor..cursor + 3)
            .is_none_or(|row| row[0] != tag || row[1] != 0x04 || !matches!(row[2], 0x01 | 0x02))
        {
            return Err(NumericalContractKeyError::InvalidCanonicalKey);
        }
        if bytes[cursor + 2] == 0x02 {
            if bytes.get(cursor + 3) != Some(&0x03) {
                return Err(NumericalContractKeyError::IncoherentAssumption);
            }
            cursor += 1;
        }
        cursor += 3;
    }
    if bytes.get(cursor..) != Some(&[0x0b, 0x05, 0x01][..]) {
        return Err(NumericalContractKeyError::InvalidCanonicalKey);
    }
    Ok(())
}

impl core::fmt::Display for NumericalContractKeyError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidCanonicalKey => {
                "numerical-contract key is not the canonical spelling of its domain"
            }
            Self::InvalidArithmetic => {
                "numerical-contract key does not name its domain's governed arithmetic"
            }
            Self::IncoherentAssumption => {
                "caller-stated contract uses ineligible assumption provenance"
            }
        })
    }
}

impl std::error::Error for NumericalContractKeyError {}

#[cfg(test)]
mod tests {
    use super::{
        ApproximationEnvelope, ArithmeticType, Bf16NumericalContractKey,
        ExceptionalValueAssumption, F32NumericalContractKey, FlushedZeroSign,
        MaterializationRounding, NumericalContractKeyError, NumericalPermission, SubnormalMode,
        ValueDomainProvenance,
    };
    use crate::semantic::{F32, FrozenSemanticRegistry, TypeKey, builtin_scalar_value_types};

    /// The canonical spelling is one identity with `TypeKey`, not a second.
    ///
    /// `canonical_type_key` is a `const` string because a declaration table is
    /// `&'static` data, while a `TypeKey` owns its components and cannot be built
    /// in a `const`. That is a representation difference and must never become an
    /// identity difference, so the two are compared here: a change to either
    /// spelling fails this test rather than producing two identities for one dtype
    /// in two crates that each believe they name the same thing.
    #[test]
    fn every_arithmetic_type_spells_its_canonical_type_key_exactly() {
        for arithmetic in ArithmeticType::ALL {
            let (namespace, name, version) = match arithmetic {
                ArithmeticType::F16 => ("tiler", "f16", 1),
                ArithmeticType::Bf16 => ("tiler", "bf16", 1),
                ArithmeticType::F32 => ("tiler", "f32", 1),
                ArithmeticType::F64 => ("tiler", "f64", 1),
            };
            let key = TypeKey::new(namespace, name, version).expect("a valid built-in type key");
            assert_eq!(arithmetic.canonical_type_key(), key.to_string());
        }
    }

    /// Every arithmetic type names an identity the standard registry admits.
    ///
    /// The loop above pins the spelling against a key this test constructs, which
    /// would still agree if the registry had moved. This pins it against the
    /// resolved types the standard registry actually admits, so the two cannot
    /// drift apart silently. Admission is recognition and nothing more: a
    /// contract or a target profile may speak about any of these four, and only
    /// F32 has an operation, an evaluator, or a lowering.
    #[test]
    fn every_arithmetic_type_names_a_registered_value_identity() {
        let registry = FrozenSemanticRegistry::standard().expect("the standard registry freezes");
        let admitted: Vec<_> = builtin_scalar_value_types()
            .into_iter()
            .map(|value| {
                value
                    .nominal_key()
                    .expect("the catalog's scalars are nominal")
                    .to_string()
            })
            .collect();
        for arithmetic in ArithmeticType::ALL {
            assert!(
                admitted
                    .iter()
                    .any(|key| key == arithmetic.canonical_type_key()),
                "{} is not a registered value identity",
                arithmetic.canonical_type_key()
            );
        }
        let resolved = F32::resolved_type();
        let nominal = resolved.nominal_key().expect("f32 is a nominal type");
        assert_eq!(
            ArithmeticType::F32.canonical_type_key(),
            nominal.to_string()
        );
        assert!(registry.contains(&resolved));
    }

    /// Distinct types must not collide in an identity encoding.
    #[test]
    fn arithmetic_type_tags_are_unique() {
        let mut tags: Vec<u8> = ArithmeticType::ALL.iter().map(|kind| kind.tag()).collect();
        tags.sort_unstable();
        tags.dedup();
        assert_eq!(tags.len(), ArithmeticType::ALL.len());
    }

    fn strict_f32_key() -> F32NumericalContractKey {
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
            ExceptionalValueAssumption::AssumeAbsent {
                provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
            },
            MaterializationRounding::NearestTiesToEven,
        )
        .unwrap()
    }

    #[test]
    fn f32_contract_key_round_trips_the_exact_canonical_spelling() {
        let key = strict_f32_key();
        assert_eq!(F32NumericalContractKey::try_from_str(key.as_str()), Ok(key));
    }

    #[test]
    fn f32_contract_key_parser_refuses_malformed_and_noncanonical_rows() {
        let key = strict_f32_key();
        let mut malformed = vec![
            String::new(),
            super::F32_NUMERICAL_CONTRACT_KEY_DOMAIN.to_owned(),
            format!("{}.", super::F32_NUMERICAL_CONTRACT_KEY_DOMAIN),
            key.as_str().to_ascii_uppercase(),
            format!("{}0", key.as_str()),
            format!("{}00", key.as_str()),
        ];
        let prefix = super::F32_NUMERICAL_CONTRACT_KEY_DOMAIN.len() + 1;
        for byte_offset in [0, 1, 5, 6, 7, 35] {
            let mut changed = key.as_str().as_bytes().to_vec();
            let hex = prefix + byte_offset * 2;
            changed[hex] = if changed[hex] == b'f' { b'e' } else { b'f' };
            malformed.push(String::from_utf8(changed).unwrap());
        }
        assert_eq!(malformed.len(), 12, "the malformed matrix changed");
        for candidate in malformed {
            assert!(
                F32NumericalContractKey::try_from_str(&candidate).is_err(),
                "malformed key was admitted: {candidate}"
            );
        }
    }

    #[test]
    fn f32_contract_key_parser_bounds_input_before_scanning_or_decoding() {
        let over_bound = format!(
            "{}.{}",
            super::F32_NUMERICAL_CONTRACT_KEY_DOMAIN,
            "0".repeat(82),
        );
        assert_eq!(over_bound.len(), 104, "the fixture is maximum plus two");

        let huge = format!(
            "{}.{}",
            super::F32_NUMERICAL_CONTRACT_KEY_DOMAIN,
            "00".repeat(1024 * 1024),
        );
        for candidate in [&over_bound, &huge] {
            let hex = candidate
                .strip_prefix(super::F32_NUMERICAL_CONTRACT_KEY_DOMAIN)
                .and_then(|suffix| suffix.strip_prefix('.'))
                .expect("the fixture retains the exact governed domain");
            assert!(hex.len().is_multiple_of(2));
            assert!(
                hex.bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            );
            assert_eq!(
                F32NumericalContractKey::try_from_str_with_decoder(candidate, |_, _| {
                    panic!("an invalid total length reached the hex decoder")
                }),
                Err(NumericalContractKeyError::InvalidCanonicalKey),
            );
        }
    }

    #[test]
    fn f32_contract_key_rejects_ineligible_assumption_provenance() {
        assert_eq!(
            F32NumericalContractKey::new(
                SubnormalMode::FlushToZero {
                    zero_sign: FlushedZeroSign::PreservesSign,
                },
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ApproximationEnvelope::Forbidden,
                ExceptionalValueAssumption::AssumeAbsent {
                    provenance: ValueDomainProvenance::CompilerProven,
                },
                ExceptionalValueAssumption::MakeNoAssumption,
                MaterializationRounding::NearestTiesToEven,
            ),
            Err(NumericalContractKeyError::IncoherentAssumption)
        );
    }

    /// The exact rendered `f32` key of the all-strict vector.
    ///
    /// **A pin, not a restatement of the encoder.** Adding the `bf16` domain
    /// moved the dimension rows into a writer both widths share, and the only
    /// evidence that no existing key changed is a value computed before the
    /// refactor and compared after it. Written out rather than derived, because
    /// a derived expectation would move with whatever moved the encoder.
    #[test]
    fn the_strict_f32_key_rendering_is_pinned() {
        let key = F32NumericalContractKey::new(
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
        .unwrap();
        assert_eq!(
            key.as_str(),
            "tiler.contract.f32.v2.037fc0000001010102010103020104020105020106020107020108030109040\
             10a04010b0501"
        );
    }

    fn strict_bf16_key() -> Bf16NumericalContractKey {
        Bf16NumericalContractKey::new(
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
        .unwrap()
    }

    /// The `bf16` key states its own width in its header and its own domain.
    #[test]
    fn the_strict_bf16_key_names_its_own_arithmetic() {
        let key = strict_bf16_key();
        assert_eq!(key.arithmetic(), ArithmeticType::Bf16);
        assert_eq!(
            key.as_str(),
            "tiler.contract.bf16.v1.027fc00101010201010302010402010502010602010702010803010904010a0\
             4010b0501"
        );
        assert_eq!(
            Bf16NumericalContractKey::try_from_str(key.as_str()),
            Ok(key)
        );
    }

    /// Neither domain accepts the other's rendering.
    ///
    /// This is the property that makes two same-vector contracts of different
    /// widths two contracts rather than one: if either parser admitted the
    /// other's key, a consumer holding a validated key would not know which
    /// width it had.
    #[test]
    fn the_two_contract_key_domains_are_mutually_closed() {
        let f32_key = strict_f32_key();
        let bf16_key = strict_bf16_key();
        assert_ne!(f32_key.as_str(), bf16_key.as_str());
        assert_eq!(
            Bf16NumericalContractKey::try_from_str(f32_key.as_str()),
            Err(NumericalContractKeyError::InvalidCanonicalKey),
            "an f32 key was admitted as bf16",
        );
        assert_eq!(
            F32NumericalContractKey::try_from_str(bf16_key.as_str()),
            Err(NumericalContractKeyError::InvalidCanonicalKey),
            "a bf16 key was admitted as f32",
        );
    }

    /// Every statable `bf16` vector renders a distinct key.
    ///
    /// Sampled over the dimensions whose spaces differ in width — the two
    /// subnormal rows and one permission — rather than over the whole product,
    /// because the encoder is shared with `f32`, whose exhaustive injectivity is
    /// checked in `crates/tiler-compiler/src/request.rs`. What is *not* shared
    /// is the header, and the pin above is what fixes that.
    #[test]
    fn bf16_contract_keys_are_distinct_per_resolved_vector() {
        let modes = [
            SubnormalMode::Preserve,
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            },
        ];
        let mut keys = Vec::new();
        for input in modes {
            for result in modes {
                for reassociation in [
                    NumericalPermission::Forbidden,
                    NumericalPermission::Permitted,
                ] {
                    let key = Bf16NumericalContractKey::new(
                        input,
                        result,
                        NumericalPermission::Forbidden,
                        reassociation,
                        NumericalPermission::Forbidden,
                        NumericalPermission::Forbidden,
                        NumericalPermission::Forbidden,
                        ApproximationEnvelope::Forbidden,
                        ExceptionalValueAssumption::MakeNoAssumption,
                        ExceptionalValueAssumption::MakeNoAssumption,
                        MaterializationRounding::NearestTiesToEven,
                    )
                    .unwrap();
                    assert_eq!(
                        Bf16NumericalContractKey::try_from_str(key.as_str()).as_ref(),
                        Ok(&key),
                        "a minted bf16 key did not validate",
                    );
                    keys.push(key.as_str().to_owned());
                }
            }
        }
        assert_eq!(keys.len(), 18, "the sampled population changed");
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), 18, "two bf16 vectors shared one key");
    }

    /// The `bf16` parser refuses the same malformed shapes the `f32` one does.
    #[test]
    fn bf16_contract_key_parser_refuses_malformed_and_noncanonical_rows() {
        let key = strict_bf16_key();
        let mut malformed = vec![
            String::new(),
            super::BF16_NUMERICAL_CONTRACT_KEY_DOMAIN.to_owned(),
            format!("{}.", super::BF16_NUMERICAL_CONTRACT_KEY_DOMAIN),
            key.as_str().to_ascii_uppercase(),
            format!("{}0", key.as_str()),
            format!("{}00", key.as_str()),
        ];
        let prefix = super::BF16_NUMERICAL_CONTRACT_KEY_DOMAIN.len() + 1;
        // Byte 0 is the arithmetic tag, 1..3 the canonical NaN payload, and 3
        // onward the dimension rows; the last is the materialization row.
        for byte_offset in [0, 1, 2, 3, 4, 33] {
            let mut changed = key.as_str().as_bytes().to_vec();
            let hex = prefix + byte_offset * 2;
            changed[hex] = if changed[hex] == b'f' { b'e' } else { b'f' };
            malformed.push(String::from_utf8(changed).unwrap());
        }
        assert_eq!(malformed.len(), 12, "the malformed matrix changed");
        for candidate in malformed {
            assert!(
                Bf16NumericalContractKey::try_from_str(&candidate).is_err(),
                "malformed key was admitted: {candidate}"
            );
        }
    }

    /// A `bf16` key states an ineligible provenance no more than an `f32` one.
    #[test]
    fn bf16_contract_key_rejects_ineligible_assumption_provenance() {
        assert_eq!(
            Bf16NumericalContractKey::new(
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ApproximationEnvelope::Forbidden,
                ExceptionalValueAssumption::MakeNoAssumption,
                ExceptionalValueAssumption::AssumeAbsent {
                    provenance: ValueDomainProvenance::RuntimeValidated,
                },
                MaterializationRounding::NearestTiesToEven,
            ),
            Err(NumericalContractKeyError::IncoherentAssumption)
        );
    }
}
