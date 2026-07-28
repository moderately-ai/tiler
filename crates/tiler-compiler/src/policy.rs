#![allow(
    dead_code,
    reason = "the preset table, the per-operation capability table, and the representability rule are all on the compile path through `request`; what stays unconstructed is the reserved half — the named preset spelling a public facade would expose, which `expose-the-numerical-contract-preference-list` owns, and the per-operation effective-permission resolution, whose only consumer today is this module's own conformance tests until a rewrite declares the permission it requires"
)]

//! Named numerical policy presets, and the per-operation conformance a preset is
//! resolved against.
//!
//! # What a preset is, and what it is not
//!
//! A preset is a **complete resolution of the numerical contract that a caller
//! requests**. It is a claim about meaning: "this is what my program computes."
//! It is never a claim about what a target can do, and it is never a way to
//! accept a weaker realization than the one stated. ADR 0076 item 5 forbids any
//! authority narrowing, weakening, or substituting the caller's stated contract
//! to make a target feasible, so the only thing a preset does is *name* a
//! contract that would otherwise have to be spelled dimension by dimension.
//!
//! The consequence is worth stating plainly, because the opposite reading is the
//! natural one. Selecting a laxer preset does not make a strict program compile
//! on a target that cannot honour it; it makes a *different program*, with a
//! different meaning, a different identity, and a different artifact. Feasibility
//! then assesses that program exactly as it assessed the strict one, and an
//! unhonourable request is a typed, explainable rejection naming the dimension,
//! the arithmetic type, the required behaviour, the behaviour the target
//! declares, and the declaring profile — never an infinite cost, never a
//! downgrade, and never a fallback.
//!
//! `docs/numerical-semantics.md` already anticipates this shape: "A user-facing
//! named mode may initialize the program ceiling, but an overlapping `fast_math`
//! boolean is avoided", and for accuracy contracts, "A frontend may expose named
//! accuracy presets, but it resolves them before canonical semantic admission."
//! A preset here is exactly that: a resolution performed *before* planning, whose
//! output is an ordinary complete contract with a versioned key.
//!
//! # Why a preset is per arithmetic type
//!
//! **Measurement.** One Apple row flushes subnormals in `f32`, preserves them in
//! `f16`, and flushes them in `bf16`, with the compiled modules declaring
//! `air.compile.denorms_disable` identically for each.
//!
//! **Inference.** A preset that stated one behaviour per dimension for a whole
//! program would therefore be stating something known to be false as soon as a
//! program mixes widths. Each contract this build registers resolves exactly one
//! [`ArithmeticType`] and says which; a program whose arithmetic reaches another
//! type is rejected by name rather than compiled under a contract that never
//! spoke about it.
//!
//! # Three claims kept apart
//!
//! - **Reserved in the type system.** Every dimension in
//!   [`crate::honourability::NumericalDimension`] can be stated, declared, and
//!   assessed.
//! - **Implemented.** [`REALIZED_DIMENSIONS`] names the eight consumable
//!   dimensions the scheduled-region IR carries.
//! - **Tested guarantee.** Only a dimension some admitted operation can consume
//!   *and* the region IR carries has an observable resolution at all, and only
//!   those carry conformance evidence.
//!
//! [`unrepresentable_dimension`] is what keeps the gap between the first two
//! honest: a dimension an admitted operation can consume but the realization
//! cannot carry may hold only the resolution this build actually realizes, and a
//! contract resolving it otherwise is refused rather than compiled under a
//! realization that never mentioned it.
//!
//! # The compound and quantized seam
//!
//! [`ArithmeticType`] names scalar floating-point formats. A compound or
//! quantized tensor value is a scheme-typed value
//! (`tiler_ir::semantic::ResolvedValueType::encoded_numeric`) whose element codes
//! and scales are ordinary operands, and whose conversion behaviour is its own
//! typed contract rather than a resolution of these dimensions. Nothing here
//! claims to cover one: [`operation_capabilities`] enumerates the operations this
//! build admits, all of which are scalar `f32`, and an operation outside that
//! table has no capability entry and therefore no effective permission to
//! compute. The seam is preserved by that absence, not by a placeholder.

use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, MaterializationRounding,
    NumericalPermission, SubnormalMode,
};

use crate::honourability::{DimensionBehaviour, NumericalDimension, NumericalRequirement};
use crate::request::StrictF32NumericalContract;

/// The accuracy envelope the relaxed preset authorizes.
///
/// A *named* envelope rather than a tolerance literal, because
/// `docs/numerical-semantics.md` requires the approximate-intrinsic dimension to
/// resolve to a maximum accuracy envelope: a bound spelled inline could be
/// widened without changing the contract's identity, which is the one thing an
/// accuracy clause must not permit. Nothing in this build emits an approximate
/// intrinsic, so this names an envelope that is authorized and unconsumed.
pub(crate) const RELAXED_APPROXIMATION_ENVELOPE: ApproximationEnvelope =
    ApproximationEnvelope::BackendElementary;

/// The dimensions [`tiler_ir::schedule::NumericalRealization`] carries.
///
/// A dimension outside this set cannot differ between two scheduled regions,
/// because the region has nowhere to record it. That is not a defect on its own —
/// the contract is deliberately wider than the realization, since completeness is
/// what makes an unenumerated dimension fail closed — but it *is* a defect the
/// moment an admitted operation can consume one of the missing dimensions, which
/// is exactly what [`unrepresentable_dimension`] refuses.
pub(crate) const REALIZED_DIMENSIONS: [NumericalDimension; 8] = [
    NumericalDimension::InputSubnormals,
    NumericalDimension::ResultSubnormals,
    NumericalDimension::Contraction,
    NumericalDimension::Reassociation,
    NumericalDimension::Permutation,
    NumericalDimension::SignedZero,
    NumericalDimension::NanAssumptions,
    NumericalDimension::InfinityAssumptions,
];

/// The dimensions one admitted semantic operation family can consume.
///
/// "Consume" means the operation's observable result can differ between two
/// resolutions of that dimension. It is deliberately the *conservative* reading:
/// an entry that is present but never exercised costs a target one declaration,
/// while an entry that is missing drops a requirement and lets a target be
/// admitted without ever being asked. The first is an over-declaration, the
/// second is a silently wrong tensor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OperationNumericalCapability {
    /// The governed operation key, as the semantic registry spells it.
    key: &'static str,
    /// The dimensions this operation can consume, in canonical order.
    consumes: &'static [NumericalDimension],
}

impl OperationNumericalCapability {
    /// The governed operation key this entry speaks about.
    pub(crate) const fn key(self) -> &'static str {
        self.key
    }

    /// The dimensions this operation can consume.
    pub(crate) const fn consumes(self) -> &'static [NumericalDimension] {
        self.consumes
    }

    /// Whether this operation can consume `dimension`.
    pub(crate) fn can_consume(self, dimension: NumericalDimension) -> bool {
        self.consumes.contains(&dimension)
    }

    /// The effective resolution of `dimension` for this operation under `ceiling`.
    ///
    /// `docs/numerical-semantics.md` resolves an operation's effective
    /// permissions as the program ceiling intersected with any tighter
    /// per-operation restriction and with the operation's own capabilities. This
    /// build admits no per-operation restriction, so the intersection is the
    /// ceiling and the capability: an operation that cannot consume the dimension
    /// resolves to `None`, and one that can resolves to the ceiling's own value.
    ///
    /// Returning `None` rather than a strict behaviour is deliberate. "This
    /// operation has no resolution on this dimension" and "this operation
    /// resolves it strictly" are different claims, and collapsing them would let
    /// a later rewrite read a manufactured strictness as an obligation the
    /// contract never stated.
    pub(crate) fn effective(
        self,
        dimension: NumericalDimension,
        ceiling: &StrictF32NumericalContract,
    ) -> Option<DimensionBehaviour> {
        self.can_consume(dimension)
            .then(|| ceiling.behaviour(dimension))
    }
}

/// Every semantic operation family this build admits, with what it can consume.
///
/// **Fact.** The governed semantic registry admits exactly `constant_f32`,
/// `multiply_f32`, `add_f32`, and `strict_serial_sum_f32`; the governed scalar
/// registry admits their scalar counterparts plus the canonical-NaN conversion.
/// None of them divides, computes an elementary function, or converts between
/// dtypes, which is why three dimensions appear in no row below.
///
/// **Inference.** `ReciprocalTransform` needs a division to replace,
/// `ApproximateIntrinsics` needs an elementary function to approximate, and
/// `MaterializationRounding` needs a conversion at a materialization boundary to
/// round. This build's only materialization boundary carries `f32` to `f32` with
/// no conversion, so no rounding is applied there at all. Their resolutions are
/// therefore unconsumable and cannot change any result this build produces —
/// which is what makes it safe for the realization not to carry them, and is
/// checked rather than assumed by
/// `every_registered_preset_is_representable_by_this_build`.
pub(crate) const fn operation_capabilities() -> &'static [OperationNumericalCapability] {
    /// Dimensions any `f32` arithmetic operation can consume.
    const ARITHMETIC: &[NumericalDimension] = &[
        NumericalDimension::InputSubnormals,
        NumericalDimension::ResultSubnormals,
        NumericalDimension::Contraction,
        NumericalDimension::SignedZero,
        NumericalDimension::NanAssumptions,
        NumericalDimension::InfinityAssumptions,
    ];
    /// Dimensions the strict serial sum can consume.
    ///
    /// It adds the two order-contract dimensions and drops contraction: a
    /// reduction's per-contributor step is `accumulator + contributor` with no
    /// product to fuse, so contraction has nothing to act on. Fusing the
    /// pointwise multiply into the reduction produces a different operation, and
    /// that operation's row is the arithmetic one above.
    const REDUCTION: &[NumericalDimension] = &[
        NumericalDimension::InputSubnormals,
        NumericalDimension::ResultSubnormals,
        NumericalDimension::Reassociation,
        NumericalDimension::Permutation,
        NumericalDimension::SignedZero,
        NumericalDimension::NanAssumptions,
        NumericalDimension::InfinityAssumptions,
    ];
    &[
        // A constant retains its declared bit pattern until an operation's
        // semantics produce a new value, so no arithmetic freedom acts on it.
        OperationNumericalCapability {
            key: "tiler::constant-f32@1",
            consumes: &[],
        },
        OperationNumericalCapability {
            key: "tiler::multiply-f32@1",
            consumes: ARITHMETIC,
        },
        OperationNumericalCapability {
            key: "tiler::add-f32@1",
            consumes: ARITHMETIC,
        },
        OperationNumericalCapability {
            key: "tiler::strict-serial-sum-f32@1",
            consumes: REDUCTION,
        },
    ]
}

/// Whether any admitted operation can consume `dimension`.
pub(crate) fn is_consumable(dimension: NumericalDimension) -> bool {
    operation_capabilities()
        .iter()
        .any(|capability| capability.can_consume(dimension))
}

/// The first admitted operation that can consume `dimension`, in canonical order.
fn first_consumer(dimension: NumericalDimension) -> Option<&'static str> {
    operation_capabilities()
        .iter()
        .find(|capability| capability.can_consume(dimension))
        .map(|capability| capability.key())
}

/// A dimension whose stated resolution this build cannot realize.
///
/// The dimension is one some admitted operation can consume, so its resolution
/// changes an observable result, and it is *not* carried by
/// [`tiler_ir::schedule::NumericalRealization`], so no scheduled region can record
/// which resolution was chosen. Compiling such a contract would produce a program
/// whose meaning is not recoverable from its own identity — two contracts
/// resolving the dimension differently would reach the same region — so it is
/// refused instead.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnrepresentableDimension {
    dimension: NumericalDimension,
    arithmetic: ArithmeticType,
    required: DimensionBehaviour,
    realized: DimensionBehaviour,
    consumed_by: &'static str,
}

impl UnrepresentableDimension {
    /// The dimension this build cannot realize as stated.
    pub(crate) const fn dimension(self) -> NumericalDimension {
        self.dimension
    }

    /// The arithmetic type the contract stated it for.
    pub(crate) const fn arithmetic(self) -> ArithmeticType {
        self.arithmetic
    }

    /// The behaviour the contract required.
    pub(crate) const fn required(self) -> DimensionBehaviour {
        self.required
    }

    /// The only behaviour this build realizes on that dimension.
    ///
    /// Reported so a caller can see which contract this build accepts, exactly as
    /// an unhonourable dimension reports the behaviour the target does honour. It
    /// is never substituted for the stated one.
    pub(crate) const fn realized(self) -> DimensionBehaviour {
        self.realized
    }

    /// The first admitted operation that can consume the dimension.
    pub(crate) const fn consumed_by(self) -> &'static str {
        self.consumed_by
    }
}

/// The behaviour this build realizes on a dimension the realization cannot carry.
///
/// These are not "defaults" in the sense ADR 0076 item 2 forbids: nothing here
/// fills in a dimension the caller left unstated, because the contract has no
/// unstated dimensions. This is the single resolution the *emitted program*
/// already implements — the schedule's contributor order is the canonical
/// lexicographic one and its `permits_permutation` is fixed false, every
/// arithmetic result is canonicalized against the contract's NaN pattern, and no
/// rewrite in this build consumes a signed-zero or infinity assumption — so a
/// contract stating anything else on one of them is stating something the build
/// does not do.
const fn realized_behaviour(dimension: NumericalDimension) -> DimensionBehaviour {
    match dimension {
        NumericalDimension::InputSubnormals | NumericalDimension::ResultSubnormals => {
            DimensionBehaviour::Subnormals(SubnormalMode::Preserve)
        }
        NumericalDimension::Contraction
        | NumericalDimension::Reassociation
        | NumericalDimension::Permutation
        | NumericalDimension::SignedZero
        | NumericalDimension::ReciprocalTransform => {
            DimensionBehaviour::Transform(NumericalPermission::Forbidden)
        }
        NumericalDimension::ApproximateIntrinsics => {
            DimensionBehaviour::Approximation(ApproximationEnvelope::Forbidden)
        }
        NumericalDimension::NanAssumptions | NumericalDimension::InfinityAssumptions => {
            DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption)
        }
        NumericalDimension::MaterializationRounding => {
            DimensionBehaviour::Rounding(MaterializationRounding::NearestTiesToEven)
        }
    }
}

/// Returns the first dimension of `contract` this build cannot realize.
///
/// Canonical order, so the reported cause is a function of the contract rather
/// than of iteration order. A dimension the realization carries is never
/// reported: the region records which resolution was chosen, so both resolutions
/// are representable and a target's inability to honour one is a *feasibility*
/// verdict rather than a representability one. The two are different claims and
/// are deliberately not merged — one says "this build cannot express what you
/// asked for" and the other says "this target cannot do it".
pub(crate) fn unrepresentable_dimension(
    contract: &StrictF32NumericalContract,
) -> Option<UnrepresentableDimension> {
    crate::honourability::CANONICAL_DIMENSIONS
        .into_iter()
        .filter(|dimension| !REALIZED_DIMENSIONS.contains(dimension))
        .find_map(|dimension| {
            let required = contract.behaviour(dimension);
            let realized = realized_behaviour(dimension);
            if required == realized {
                return None;
            }
            first_consumer(dimension).map(|consumed_by| UnrepresentableDimension {
                dimension,
                arithmetic: contract.arithmetic,
                required,
                realized,
                consumed_by,
            })
        })
}

/// The per-dimension requirements a contract places on a target profile.
///
/// One requirement per dimension some admitted operation can consume, complete
/// and in canonical order, each carrying the contract's arithmetic type.
///
/// **Why the set is the consumable dimensions and not all of them.**
/// `docs/numerical-semantics.md` resolves effective permissions as the program
/// ceiling intersected with the operation's own capabilities, so a dimension no
/// admitted operation can consume places no obligation on a target and asking a
/// profile to declare it would reject targets over a freedom nothing exercises.
/// The direction that would be unsafe is the opposite one — dropping a
/// requirement for a dimension an operation *can* consume — which is why
/// [`operation_capabilities`] is written conservatively and why
/// [`unrepresentable_dimension`] independently refuses any consumable dimension
/// the realization cannot carry.
pub(crate) fn dimension_requirements(
    contract: &StrictF32NumericalContract,
) -> Vec<NumericalRequirement> {
    crate::honourability::CANONICAL_DIMENSIONS
        .into_iter()
        .filter(|dimension| is_consumable(*dimension))
        .map(|dimension| {
            NumericalRequirement::new(
                dimension,
                contract.arithmetic,
                contract.behaviour(dimension),
            )
        })
        .collect()
}

/// A named numerical policy preset this build registers.
///
/// Each variant is a **claim about what a caller requests**, stated once here so
/// that a caller naming a preset and a caller spelling the same contract
/// dimension by dimension produce byte-identical requests. The variants are not
/// ordered by strength and nothing selects between them: which one a program
/// means is the caller's statement, made before compilation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum NumericalPolicyPreset {
    /// Every freedom refused and both subnormal dimensions preserved.
    ///
    /// The claim: this program's results are the strict IEEE-754 reading under
    /// round-to-nearest, ties-to-even, with gradual underflow, with every
    /// arithmetic NaN canonicalized, and with no reordering, fusion, or
    /// substitution of any operation.
    Strict,
    /// Strict, except that both subnormal dimensions flush to a sign-preserving
    /// zero.
    ///
    /// The claim: flushing subnormals to the zero of their own sign is part of
    /// what this program means. `PreservesSign` because that is what the measured
    /// hardware does — `0x80400000 * 2.0f` returns `0x80000000` — and a contract
    /// must name which zero it accepts, since the two zeros are observably
    /// different results.
    ///
    /// It widens exactly one dimension, so accepting flushing does not silently
    /// accept reassociation.
    FlushSubnormalsToZero,
    /// Subnormals preserved, and the reshaping freedoms this build can express
    /// authorized.
    ///
    /// The claim: this program's results may differ from the strict reading by
    /// fused multiply-add contraction, by regrouping a reduction's contributor
    /// sequence, by replacing a division with a reciprocal multiplication, and by
    /// an approximate elementary function within
    /// [`RELAXED_APPROXIMATION_ENVELOPE`].
    ///
    /// **What it deliberately does not authorize.** Operand permutation,
    /// signed-zero elimination, and both exceptional-value assumptions stay at
    /// their strict resolution. The realization can represent those freedoms,
    /// but this preset does not silently broaden its established meaning.
    Relaxed,
}

impl NumericalPolicyPreset {
    /// Every preset this build registers, in canonical order.
    pub(crate) const ALL: [Self; 3] = [Self::Strict, Self::FlushSubnormalsToZero, Self::Relaxed];

    /// The versioned key of the contract this preset resolves to.
    ///
    /// Each preset resolves to a *different contract*, not to a flag on one:
    /// the three give the same program different observable results, so they must
    /// give it different canonical identities, artifacts, and cache entries.
    pub(crate) const fn contract_key(self) -> &'static str {
        self.contract().key
    }

    /// Resolves this preset into the complete contract it names.
    pub(crate) const fn contract(self) -> StrictF32NumericalContract {
        match self {
            Self::Strict => StrictF32NumericalContract::governed(),
            Self::FlushSubnormalsToZero => StrictF32NumericalContract::governed_flush_to_zero(),
            Self::Relaxed => StrictF32NumericalContract::governed_relaxed(),
        }
    }
}

/// The strict resolution of every dimension, for one arithmetic type.
///
/// Shared by every preset so that "strict on this dimension" has one spelling.
/// A preset that widens a dimension overrides exactly that field, which is what
/// makes "this preset widens exactly one dimension" a readable property of the
/// constructor rather than a claim in a comment.
pub(crate) const fn strict_contract(
    key: &'static str,
    arithmetic: ArithmeticType,
    canonical_arithmetic_nan_bits: u32,
) -> StrictF32NumericalContract {
    StrictF32NumericalContract {
        key,
        arithmetic,
        canonical_arithmetic_nan_bits,
        input_subnormals: SubnormalMode::Preserve,
        result_subnormals: SubnormalMode::Preserve,
        contraction: NumericalPermission::Forbidden,
        reassociation: NumericalPermission::Forbidden,
        permutation: NumericalPermission::Forbidden,
        signed_zero: NumericalPermission::Forbidden,
        reciprocal_transform: NumericalPermission::Forbidden,
        approximate_intrinsics: ApproximationEnvelope::Forbidden,
        nan_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
        infinity_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
        materialization_rounding: MaterializationRounding::NearestTiesToEven,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NumericalPolicyPreset, REALIZED_DIMENSIONS, dimension_requirements, is_consumable,
        operation_capabilities, unrepresentable_dimension,
    };
    use crate::honourability::{CANONICAL_DIMENSIONS, DimensionBehaviour, NumericalDimension};
    use crate::request::StrictF32NumericalContract;
    use tiler_ir::schedule::{
        ExceptionalValueAssumption, NumericalPermission, ValueDomainProvenance,
    };
    use tiler_ir::semantic::{
        add_f32_op, constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
    };

    /// Every registered preset states a contract this build can actually realize.
    ///
    /// This is the check that keeps the preset table honest as it grows: a preset
    /// that widened a consumable dimension the realization cannot carry would
    /// produce two meanings under one region identity, and it fails here rather
    /// than in a cache.
    #[test]
    fn every_registered_preset_is_representable_by_this_build() {
        for preset in NumericalPolicyPreset::ALL {
            let contract = preset.contract();
            assert_eq!(
                unrepresentable_dimension(&contract),
                None,
                "{} states a dimension this build cannot realize",
                contract.key
            );
        }
    }

    /// A newly representable freedom is no longer refused by the build boundary.
    #[test]
    fn permitting_a_representable_consumable_dimension_is_not_refused() {
        let mut contract = StrictF32NumericalContract::governed();
        contract.permutation = NumericalPermission::Permitted;
        assert_eq!(unrepresentable_dimension(&contract), None);
    }

    /// The capability table names the operations the registry actually admits.
    ///
    /// Both directions. A key spelled differently from the registry's would put a
    /// name in a rejection that no operation has, and a *missing* row would drop
    /// every requirement that operation places on a target — the direction that
    /// admits a target without ever asking it. The table was written by hand and
    /// its first spelling used underscores where the registry uses hyphens, which
    /// is precisely why this compares against the keys rather than against a
    /// second list.
    #[test]
    fn the_capability_table_names_exactly_the_admitted_operations() {
        let admitted = [
            constant_f32_op(),
            multiply_f32_op(),
            add_f32_op(),
            strict_serial_sum_f32_op(),
        ];
        let mut declared: Vec<_> = operation_capabilities()
            .iter()
            .map(|capability| capability.key().to_owned())
            .collect();
        let mut expected: Vec<_> = admitted.iter().map(ToString::to_string).collect();
        declared.sort();
        expected.sort();
        assert_eq!(declared, expected);
    }

    /// Each of the four widened dimensions is representable.
    ///
    /// Named individually rather than by a loop over a derived set, so that a
    /// dimension moving between the two classes changes this test rather than
    /// passing vacuously under a set that moved with it.
    /// One case: the dimension expected to be refused, and how to widen it.
    type WideningCase = (NumericalDimension, fn(&mut StrictF32NumericalContract));

    #[test]
    fn every_widened_dimension_is_representable() {
        let assume_absent = ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CompilerProven,
        };
        let cases: [WideningCase; 4] = [
            (NumericalDimension::Permutation, |contract| {
                contract.permutation = NumericalPermission::Permitted;
            }),
            (NumericalDimension::SignedZero, |contract| {
                contract.signed_zero = NumericalPermission::Permitted;
            }),
            (NumericalDimension::NanAssumptions, |contract| {
                contract.nan_assumptions = ExceptionalValueAssumption::AssumeAbsent {
                    provenance: ValueDomainProvenance::CompilerProven,
                };
            }),
            (NumericalDimension::InfinityAssumptions, |contract| {
                contract.infinity_assumptions = ExceptionalValueAssumption::AssumeAbsent {
                    provenance: ValueDomainProvenance::CompilerProven,
                };
            }),
        ];
        assert_eq!(
            DimensionBehaviour::ExceptionalValue(assume_absent).key(),
            "assume-absent.compiler-proven"
        );
        for (dimension, widen) in cases {
            let mut contract = StrictF32NumericalContract::governed();
            widen(&mut contract);
            assert_eq!(
                unrepresentable_dimension(&contract),
                None,
                "{} must be representable",
                dimension.key()
            );
        }
    }

    /// A dimension no admitted operation can consume may take any resolution.
    ///
    /// This is the other half of the rule, and it is what lets the relaxed preset
    /// authorize a reciprocal transform and an approximation envelope while this
    /// build has neither a division nor an elementary function to apply them to.
    #[test]
    fn an_unconsumable_dimension_is_not_refused() {
        let mut contract = StrictF32NumericalContract::governed();
        contract.reciprocal_transform = NumericalPermission::Permitted;
        assert!(!is_consumable(NumericalDimension::ReciprocalTransform));
        assert_eq!(unrepresentable_dimension(&contract), None);
    }

    /// The requirement set covers exactly the consumable dimensions.
    #[test]
    fn requirements_cover_every_consumable_dimension_once() {
        let contract = StrictF32NumericalContract::governed();
        let requirements = dimension_requirements(&contract);
        let consumable: Vec<_> = CANONICAL_DIMENSIONS
            .into_iter()
            .filter(|dimension| is_consumable(*dimension))
            .collect();
        assert_eq!(requirements.len(), consumable.len());
        for (requirement, dimension) in requirements.iter().zip(&consumable) {
            assert_eq!(requirement.dimension(), *dimension);
            assert_eq!(requirement.arithmetic(), contract.arithmetic);
            assert_eq!(requirement.behaviour(), contract.behaviour(*dimension));
        }
    }

    /// Every realized dimension is one an admitted operation can consume.
    ///
    /// The realization carrying a dimension nothing can consume would be dead
    /// weight in every identity; the reverse — a consumable dimension outside the
    /// realization — is the case the representability rule governs.
    #[test]
    fn every_realized_dimension_is_consumable() {
        for dimension in REALIZED_DIMENSIONS {
            assert!(is_consumable(dimension), "{} is dead", dimension.key());
        }
    }

    /// Per-operation effective permissions intersect the ceiling with capability.
    #[test]
    fn effective_permissions_intersect_the_ceiling_with_the_operation_capability() {
        let ceiling = NumericalPolicyPreset::Relaxed.contract();
        let capabilities = operation_capabilities();
        let constant = capabilities
            .iter()
            .find(|capability| capability.key() == "tiler::constant-f32@1")
            .expect("the constant operation is admitted");
        let sum = capabilities
            .iter()
            .find(|capability| capability.key() == "tiler::strict-serial-sum-f32@1")
            .expect("the serial sum is admitted");
        // A constant consumes nothing, so it resolves no dimension at all.
        assert_eq!(
            constant.effective(NumericalDimension::Reassociation, &ceiling),
            None
        );
        // The reduction consumes reassociation and takes the ceiling's value.
        assert_eq!(
            sum.effective(NumericalDimension::Reassociation, &ceiling),
            Some(DimensionBehaviour::Transform(
                NumericalPermission::Permitted
            ))
        );
        // It cannot consume contraction: there is no product in its combine step.
        assert_eq!(
            sum.effective(NumericalDimension::Contraction, &ceiling),
            None
        );
    }

    /// Every preset resolves to a distinct versioned contract key.
    #[test]
    fn preset_contract_keys_are_distinct() {
        let mut keys: Vec<_> = NumericalPolicyPreset::ALL
            .iter()
            .map(|preset| preset.contract_key())
            .collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), NumericalPolicyPreset::ALL.len());
    }

    /// Operation capability keys are unique, so a lookup cannot be ambiguous.
    #[test]
    fn operation_capability_keys_are_unique() {
        let mut keys: Vec<_> = operation_capabilities()
            .iter()
            .map(|capability| capability.key())
            .collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), operation_capabilities().len());
    }
}
