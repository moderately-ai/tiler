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
    Forbidden,
    /// Permitted up to the backend-elementary envelope.
    ///
    /// `docs/numerical-semantics.md` defines this conformance level as one where
    /// "operation graph is preserved but elementary function results follow the
    /// backend contract". It bounds the approximation by the backend's own stated
    /// accuracy rather than by a Tiler-side numeric tolerance, so a backend that
    /// states none cannot honour it.
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

#[cfg(test)]
mod tests {
    use super::ArithmeticType;
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
}
