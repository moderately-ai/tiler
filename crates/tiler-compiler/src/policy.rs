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
//!   [`crate::target::honourability::NumericalDimension`] can be stated, declared, and
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
//! claims to reinterpret one through these generic freedoms:
//! [`operation_capabilities`] enumerates the strict-affine association and
//! conversion operations with no consumed generic dimension because their
//! complete rounding, saturation, exceptional-value, evaluation-order, and
//! materialization behavior is fixed by their versioned scheme contract.
//! Physical execution remains unsupported until a lowering separately proves
//! that complete contract; an empty generic-dimension row is not a lowering
//! capability.

use tiler_ir::numerics::PolicyLocus;
use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign,
    MaterializationRounding, NumericalPermission, SubnormalMode, ValueDomainProvenance,
};

use crate::request::StrictF32NumericalContract;
use crate::target::ScalarArithmetic;
use crate::target::honourability::{DimensionBehaviour, NumericalDimension, NumericalRequirement};
use tiler_ir::semantic::OpKey;

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

/// The key a composed contract carries before one is derived for it.
///
/// A spelling no canonical key can collide with — every derived key opens with
/// [`tiler_ir::schedule::F32_NUMERICAL_CONTRACT_KEY_DOMAIN`] or its `bf16`
/// sibling, and neither of those domains is this string — so a contract that
/// reached admission without being keyed is refused by name rather than admitted
/// under a plausible string. Nothing outside [`strict_contract`] writes it.
pub(crate) const UNKEYED_CONTRACT: &str = "tiler.contract.unkeyed";

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

    /// Whether this operation's semantics embed an ordered contributor fold.
    ///
    /// **Derived from this table rather than restated beside it.**
    /// [`NumericalDimension::Permutation`] is defined as permission to change
    /// *logical contributor order*, and a contributor order exists only where
    /// something folds a sequence: every row above that lists it justifies the
    /// entry by an embedded fold — the strict serial sum's contributor
    /// sequence, the tensor contraction's, the normalization's, and the
    /// softmax's denominator sum — and every row that omits it does so because
    /// the operation computes each result element independently. A second list
    /// of "these families reduce" would be that same claim written twice, and a
    /// copy is a second place for it to be wrong.
    ///
    /// The fold-bearing set this yields is pinned by name in
    /// `the_fold_bearing_families_are_exactly_the_reducing_ones`, so a
    /// capability row that gained or lost a permutation entry has to change
    /// that test rather than silently move an obligation's locus.
    ///
    /// Spelled as a loop rather than through [`Self::can_consume`] because
    /// `[T]::contains` is not `const` and [`Self::founded_locus`] is; the
    /// membership tested is the same one.
    pub(crate) const fn folds(self) -> bool {
        let mut index = 0;
        while index < self.consumes.len() {
            if matches!(self.consumes[index], NumericalDimension::Permutation) {
                return true;
            }
            index += 1;
        }
        false
    }

    /// The policy position within one occurrence at which `dimension`'s
    /// requirement is founded for this operation.
    ///
    /// **A position this operation's own semantics put the freedom at, never a
    /// variant chosen because the enum has one.** An obligation naming a locus
    /// is a claim that a packaged route relies on the target at *that position*,
    /// so a locus that nothing founds is worse than no row at all: it is an
    /// unfounded assertion carrying real evidence. `None` therefore means this
    /// build founds no position, and [`crate::session`]'s producer refuses
    /// rather than substituting one.
    ///
    /// Each mapping is the dimension's own definition read against
    /// [`PolicyLocus`]'s:
    ///
    /// - `InputSubnormals` is "treatment of subnormal operands **before each
    ///   arithmetic operation**", which is [`PolicyLocus::Input`], "an operand
    ///   read before the operation applies".
    /// - `ResultSubnormals` is "treatment of a newly **produced** subnormal
    ///   arithmetic result", which is [`PolicyLocus::Result`], "the operation's
    ///   produced value".
    /// - `Contraction` and `Reassociation` act wherever the operation puts a
    ///   multiply beside an add and wherever it groups one same-operation
    ///   operand sequence. For a fold-bearing family that place is the
    ///   accumulator: this table's own rows found both entries on the
    ///   per-contributor step — `accumulator + a * b` for the tensor
    ///   contraction, `accumulator + x_i * x_i` for the normalization. For
    ///   pointwise arithmetic there is no fold, and the freedom acts on the
    ///   operation's own arithmetic.
    /// - `Permutation` is permission to change *contributor* order, which only
    ///   a fold has; [`Self::folds`] is derived from this dimension for that
    ///   reason, so the accumulator arm cannot be reached without one.
    /// - `SignedZero`, `NanAssumptions`, and `InfinityAssumptions` are
    ///   properties of the arithmetic the operation itself performs.
    ///
    /// **Three dimensions are deliberately unfounded here, and none of the
    /// three is merely unreached.** `MaterializationRounding` names a boundary
    /// *between* stages rather than a position inside one occurrence, so an
    /// operation capability is the wrong authority to site it — a schedule that
    /// stages a partial tensor is what creates the boundary. `ReciprocalTransform`
    /// and `ApproximateIntrinsics` act on a *subordinate* operation inside a
    /// composite family — the activation's exponential, the normalization's
    /// reciprocal square root — whose accuracy is carried by that family's own
    /// [`tiler_ir::semantic::accuracy::AccuracyContract`] rather than by one of
    /// this occurrence's four generic positions. All three are also outside
    /// [`is_consumable`] today, so no contract places one on a target and no
    /// honoured fact exists to carry; returning `None` is what makes the day one
    /// becomes consumable a typed refusal rather than a silently relocated row.
    pub(crate) const fn founded_locus(self, dimension: NumericalDimension) -> Option<PolicyLocus> {
        match dimension {
            NumericalDimension::InputSubnormals => Some(PolicyLocus::Input),
            NumericalDimension::ResultSubnormals => Some(PolicyLocus::Result),
            NumericalDimension::Contraction | NumericalDimension::Reassociation => {
                if self.folds() {
                    Some(PolicyLocus::Accumulator)
                } else {
                    Some(PolicyLocus::Computation)
                }
            }
            NumericalDimension::Permutation => Some(PolicyLocus::Accumulator),
            NumericalDimension::SignedZero
            | NumericalDimension::NanAssumptions
            | NumericalDimension::InfinityAssumptions => Some(PolicyLocus::Computation),
            NumericalDimension::ReciprocalTransform
            | NumericalDimension::ApproximateIntrinsics
            | NumericalDimension::MaterializationRounding => None,
        }
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
/// **Fact.** The governed semantic registry admits the four scalar `f32`
/// operations, the strict tensor contraction, `Reindex` and `Broadcast`, and
/// strict-affine association, quantization, and dequantization; every one has a
/// row below. The affine operations' exact behavior is carried by the encoded
/// value and operation contracts rather than selected from the caller's generic
/// policy. None of the admitted operations permits a reciprocal substitution or
/// approximate intrinsic.
///
/// **Fact.** The registry also admits `tiler::constant-bf16@1`,
/// `tiler::multiply-bf16@1`, and `tiler::add-bf16@1`, and none of the three has
/// a row here. That is deliberate and is checked by
/// `every_unplanned_operation_is_registered_and_consumes_no_dimension`: a BF16
/// operation consumes no numerical freedom *this build has proved for it*, so
/// every rewrite that asks for one declines.
///
/// **The ground moved and the conclusion did not, which is why it is restated
/// rather than left standing.** The three were rowless because nothing realized
/// BF16 at all — no lowering, no recognizer, no plan. Each now carries a
/// governed index-access lowering and a `bf16` program is recognized and
/// planned, so "no target profile can even state a contract for it" is false.
/// What holds is the narrower and older claim: a row here enters each dimension
/// it lists into `is_consumable`'s union, which decides whether a *contract* may
/// permit that dimension at all, and reassociation and contraction error are
/// bounded by the significand — Finding 28 of the Apple numerical behaviour
/// record measures a target whose contraction behaviour differs between `f16`
/// and `bf16`. So a row copied from the `f32` set would widen this build's
/// numerical surface on evidence about another width.
///
/// **`establish-bf16-optimizer-legality` landed and deliberately did not write
/// these rows.** It decided the three families' *fusion* roles in
/// `crate::fusion_legality`, from `tiler-ir`'s own `arithmetic_bf16_facts` and
/// `constant_bf16_facts` records, which is a different table answering a
/// different question: whether fusing an occurrence preserves the contract, not
/// which freedom an occurrence may consume. Rowlessness here is therefore a
/// decided state rather than an unfinished one, and writing a row would still
/// need evidence of BF16's own.
///
/// **Inference — and this changed when the activation was admitted.**
/// `MaterializationRounding` is still unconsumable: it is not the strict-affine
/// encode rounding rule, which the scheme fixes to nearest-even, and observable
/// materialization of a compound value preserves its exact codes and associated
/// parameters. `ReciprocalTransform` and `ApproximateIntrinsics` are a different
/// case now. Both were unconsumable because no admitted operation had a division
/// to replace or an elementary function to approximate, and `tiler::silu-f32@1`
/// has one of each — so their absence from every row below is no longer derived
/// from the admitted set. [`ELEMENTARY_UNCARRIED_DIMENSIONS`] states the omission
/// explicitly and `the_uncarried_elementary_dimensions_are_outside_the_realization`
/// checks the condition under which it stays honest.
pub(crate) const fn operation_capabilities() -> &'static [OperationNumericalCapability] {
    /// Dimensions any `f32` arithmetic operation can consume.
    const ARITHMETIC: &[NumericalDimension] = &[
        NumericalDimension::InputSubnormals,
        NumericalDimension::ResultSubnormals,
        NumericalDimension::Contraction,
        NumericalDimension::Reassociation,
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
    /// Dimensions the strict tensor contraction can consume.
    ///
    /// The union of the two rows above, and the only admitted operation for
    /// which that union is right. A *tensor* contraction is a reduction, so both
    /// order-contract dimensions act on its contributor fold; and its
    /// per-contributor step is `accumulator + a * b`, an adjacent multiply and
    /// add, so ADR 0015's contraction dimension has a product to fuse — which
    /// the strict serial sum's row above explicitly does not. This is the single
    /// point where the two senses of "contraction" meet, and it is a bit-level
    /// difference rather than a naming curiosity: a device or library GEMM built
    /// on fused multiply-add accumulation is incompatible with a contract that
    /// forbids it, and a target is only ever asked because this row is here.
    ///
    /// Distributivity, which a contraction-order rewrite would consume, is
    /// absent rather than withheld: no contract Tiler can express resolves it,
    /// so it is not a `NumericalDimension` at all.
    const TENSOR_CONTRACTION: &[NumericalDimension] = &[
        NumericalDimension::InputSubnormals,
        NumericalDimension::ResultSubnormals,
        NumericalDimension::Contraction,
        NumericalDimension::Reassociation,
        NumericalDimension::Permutation,
        NumericalDimension::SignedZero,
        NumericalDimension::NanAssumptions,
        NumericalDimension::InfinityAssumptions,
    ];
    /// Dimensions the elementwise activation can consume and this build carries.
    ///
    /// The arithmetic row without contraction: the activation's composition puts
    /// no multiply adjacent to an add, so there is no product for ADR 0015's
    /// permission to fuse into.
    const ELEMENTARY: &[NumericalDimension] = &[
        NumericalDimension::InputSubnormals,
        NumericalDimension::ResultSubnormals,
        NumericalDimension::Reassociation,
        NumericalDimension::SignedZero,
        NumericalDimension::NanAssumptions,
        NumericalDimension::InfinityAssumptions,
    ];
    /// Dimensions the RMS normalization can consume and this build carries.
    ///
    /// **It consumes contraction, and the bare serial sum does not.** The
    /// reduction row above drops contraction because a strict serial sum's
    /// per-contributor step is `accumulator + contributor`, with no product to
    /// fuse. This family's step is `accumulator + x_i * x_i` — the squaring
    /// prologue puts a multiply directly beside the add, which is exactly the
    /// adjacency ADR 0015's permission acts on and exactly why the tensor
    /// contraction's row carries it too. A row copied from the reduction's would
    /// be asking no target about a fused multiply-add this operation genuinely
    /// admits.
    ///
    /// Both order-contract dimensions are present for the embedded fold's sake,
    /// as they are for any ordered reduction.
    const NORMALIZATION: &[NumericalDimension] = &[
        NumericalDimension::InputSubnormals,
        NumericalDimension::ResultSubnormals,
        NumericalDimension::Contraction,
        NumericalDimension::Reassociation,
        NumericalDimension::Permutation,
        NumericalDimension::SignedZero,
        NumericalDimension::NanAssumptions,
        NumericalDimension::InfinityAssumptions,
    ];
    /// Dimensions the softmax can consume and this build carries.
    ///
    /// **It does *not* consume contraction, and the bare reduction row's reason
    /// is not why.** The normalization's row gained contraction because
    /// `accumulator + x_i * x_i` puts a multiply beside the fold's add and fusing
    /// the two removes a rounding. The softmax has a multiply-add adjacency too —
    /// `s_i + (-1) * m`, the maximum subtraction — but its multiply is an *exact*
    /// sign flip, so a fused multiply-add there removes a rounding that never
    /// happened and cannot change a result. Listing the dimension would enter it
    /// into `is_consumable`'s union and place it on every contract, in order to
    /// ask targets about a freedom this operation's answer is invariant under.
    ///
    /// Both order-contract dimensions are present for the *denominator* fold's
    /// sake. The maximum fold consumes neither — it is associative and
    /// commutative on every input — but a capability row is per operation rather
    /// than per embedded fold, and the sum needs both.
    const SOFTMAX: &[NumericalDimension] = &[
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
        // The activation is `f32` arithmetic, so it consumes the arithmetic row's
        // dimensions minus contraction: its composition is a multiply by `-1.0`,
        // an exponential, an add, and a divide, and no multiply is adjacent to an
        // add, so ADR 0015's contraction has no product to fuse.
        //
        // **Two dimensions it can consume are deliberately absent, and the
        // absence is a stated boundary rather than an oversight.** SiLU is the
        // first admitted operation that could consume `ReciprocalTransform` (it
        // contains a division) and the first that could consume
        // `ApproximateIntrinsics` (it contains an elementary function). Listing
        // either would enter it into `is_consumable`'s union, which decides which
        // dimensions every contract must place on a target — and neither is
        // carried by `tiler_ir::schedule::NumericalRealization`, so the
        // `RelaxedF32` preset that authorizes both would become unrepresentable
        // for every program. Widening the realization to carry them is ADR 0076
        // item 1's shape and is filed as `carry-the-elementary-numerical-
        // dimensions-in-the-region-realization`; until then the obligation is
        // enforced where this build can actually enforce it, in the Metal
        // emission, which writes `precise::exp` and the `/` operator and requires
        // `-fmetal-math-fp32-functions=precise`.
        //
        // `ELEMENTARY_UNCARRIED_DIMENSIONS` names the two so the omission is a checked
        // claim rather than a gap, and its test fires the moment the realization
        // grows to carry one.
        OperationNumericalCapability {
            key: "tiler::silu-f32@1",
            consumes: ELEMENTARY,
        },
        // The normalization is an ordered reduction with per-point arithmetic on
        // both sides of its fold, so it consumes the reduction row's dimensions
        // *and* contraction — see `NORMALIZATION` for why the adjacency is real.
        //
        // The same two elementary dimensions the activation withholds are
        // withheld here, for the same reason and by the same constant: the
        // normalization contains a division (by the extent) and an elementary
        // function (the reciprocal square root), so both are real obligations
        // rather than absent ones, and `ELEMENTARY_UNCARRIED_DIMENSIONS` is what
        // states the omission as a checked claim.
        OperationNumericalCapability {
            key: "tiler::rms-norm-f32@1",
            consumes: NORMALIZATION,
        },
        // The softmax is two ordered reductions with per-point arithmetic between
        // and after them, so it consumes the reduction row's dimensions and
        // *not* contraction — see `SOFTMAX` for why the one adjacency it has is
        // inert.
        //
        // The same two elementary dimensions the activation and the
        // normalization withhold are withheld here, for the same reason and by
        // the same constant: the softmax contains a division (one by the
        // denominator) and an elementary function (the exponential), so both are
        // real obligations rather than absent ones.
        OperationNumericalCapability {
            key: "tiler::softmax-f32@1",
            consumes: SOFTMAX,
        },
        OperationNumericalCapability {
            key: "tiler::strict-tensor-contraction-f32@1",
            consumes: TENSOR_CONTRACTION,
        },
        // The two structural families consume nothing, and the reason is not
        // that their rows are unfinished. A reindex and a broadcast compute no
        // value: each result element is an operand element with the same bits,
        // so there is no rounding to relax, no order to change, and no signed
        // zero or NaN to canonicalize. Subnormals in particular are *not*
        // consumable here — a family that never performs an arithmetic operation
        // cannot flush an input or a result, and declaring the dimension would
        // let a target's flush mode be read as permission acting on data these
        // families only move. An empty row is therefore the strict claim rather
        // than the absent one, exactly as it is for `tiler::constant-f32@1`.
        OperationNumericalCapability {
            key: "tiler::reindex-f32@1",
            consumes: &[],
        },
        OperationNumericalCapability {
            key: "tiler::broadcast-f32@1",
            consumes: &[],
        },
        // These operations carry a complete, fixed strict-affine conversion
        // contract. No caller-selected generic freedom can weaken or substitute
        // it, and no physical lowering is implied by these rows.
        OperationNumericalCapability {
            key: "tiler::assemble-strict-affine@1",
            consumes: &[],
        },
        OperationNumericalCapability {
            key: "tiler::quantize-strict-affine@1",
            consumes: &[],
        },
        OperationNumericalCapability {
            key: "tiler::dequantize-strict-affine@1",
            consumes: &[],
        },
    ]
}

/// The dimensions the admitted elementary families can consume and this build withholds.
///
/// **Both are real obligations for all three families, and neither is a row.**
/// `tiler::silu-f32@1` contains a division and an exponential;
/// `tiler::rms-norm-f32@1` contains a division by the extent and a reciprocal
/// square root; `tiler::softmax-f32@1` contains a division of one by the
/// denominator and an exponential. A contract resolving `ReciprocalTransform` or
/// `ApproximateIntrinsics` differently would admit a different observable result
/// for any of them — which is exactly the condition [`operation_capabilities`]
/// says a row exists for.
///
/// The softmax's `ReciprocalTransform` obligation runs in the *opposite*
/// direction from its siblings' and is a real obligation for that reason: the
/// pinned formula already multiplies by the reciprocal, so what the permission
/// would license here is the substitution *back* to a division.
///
/// They are withheld because listing them enters each into [`is_consumable`]'s
/// union, and that union decides which dimensions *every* contract places on a
/// target. Neither is carried by [`tiler_ir::schedule::NumericalRealization`], so
/// [`unrepresentable_dimension`] would then refuse the public `RelaxedF32` preset
/// — which authorizes both — for every program, whether or not it contains an
/// activation. Making them representable means widening the region realization,
/// which is ADR 0076 item 1's shape and a separate change.
///
/// **What holds the line meanwhile.** The obligation is enforced where this build
/// can enforce it: `crates/tiler-metal/src/emit.rs` writes `precise::exp`,
/// `precise::rsqrt`, and the `/` operator rather than a fast intrinsic or a
/// reciprocal multiply, and records
/// `MetalNumericalRequirement::PreciseFp32Functions`. That is a backend guarantee
/// over the operations actually emitted, not a profile-level assessment, and the
/// difference is the whole of what this constant defers.
pub(crate) const ELEMENTARY_UNCARRIED_DIMENSIONS: [NumericalDimension; 2] = [
    NumericalDimension::ReciprocalTransform,
    NumericalDimension::ApproximateIntrinsics,
];

/// Returns the numerical capabilities declared for one governed operation.
///
/// The table's spellings are already checked in both directions against the
/// governed typed keys by `the_capability_table_names_exactly_the_admitted_operations`.
pub(crate) fn operation_capability(key: &OpKey) -> Option<OperationNumericalCapability> {
    let key = key.to_string();
    operation_capabilities()
        .iter()
        .copied()
        .find(|capability| capability.key == key)
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
    crate::target::honourability::CANONICAL_DIMENSIONS
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
    // The subject is derived from the contract's own width through the governed
    // scalar catalog, not written beside it. A profile declares honourability
    // for a `ScalarArithmetic` built by the same constructor, and a requirement
    // matches a declaration only when *both* halves of the subject agree — so a
    // requirement carrying `tiler::f32@1` beside `ArithmeticType::Bf16` would
    // match no `bf16` row a target could ever declare, and every `bf16` contract
    // would resolve `Unknown` for a reason no reader could locate.
    //
    // Every arithmetic type resolves a subject, including the two this build
    // registers no contract key for. That totality is deliberate and is the
    // difference between "no target declares this" and "nothing was asked": a
    // requirement set that shrank for an unregistered width would be *vacuously
    // feasible*, so a contract naming one would be proven by every profile
    // rather than reported `Unknown` by all of them.
    let Some(subject) = arithmetic_subject(contract.arithmetic) else {
        return Vec::new();
    };
    crate::target::honourability::CANONICAL_DIMENSIONS
        .into_iter()
        .filter(|dimension| is_consumable(*dimension))
        .map(|dimension| {
            NumericalRequirement::new(
                dimension,
                subject.arithmetic(),
                subject.resolved_type().clone(),
                contract.behaviour(dimension),
            )
        })
        .collect()
}

/// Whether `required` demands at least as much of a target as `ceiling` does.
///
/// **The rule a locus obligation must satisfy against the dtype-wide ceiling.**
/// The ceiling and the per-locus obligations are separate statements and neither
/// is derived from the other, but they are not independent: a position may
/// demand *more* than the program-wide contract — ADR 0011's per-operation
/// restrictions are exactly that — and may never demand less. A locus obligation
/// weaker than the ceiling would be a route relying on a freedom the caller's
/// contract never granted, and because the obligation carries real target
/// evidence it would read as a proof of the opposite. `crate::session`'s
/// producer checks every row against this before it is retained.
///
/// **A partial order, and refusing where the vocabulary states no order is the
/// point.** Each space has one strict resolution — the one
/// [`strict_contract`] writes — and the widenings away from it are ordered
/// against it. Two behaviours that are merely *different* are not ordered: a
/// flush that always yields `+0` is not a stricter or laxer form of one that
/// preserves the sign, and a compiler-proven absence assumption is not a
/// stricter form of a caller-declared one — they rest on different evidence for
/// the same assumption. Inventing an order over those pairs would be the silent
/// direction, so only equality passes between them and everything else is
/// refused.
///
/// Comparing two spaces is malformed rather than a verdict, exactly as
/// [`NumericalDimension::admits`] treats the same pairing, and answering `true`
/// would let a subnormal requirement pass a transform ceiling unchecked.
pub(crate) const fn is_at_least_as_strict_as(
    required: DimensionBehaviour,
    ceiling: DimensionBehaviour,
) -> bool {
    match (required, ceiling) {
        (DimensionBehaviour::Subnormals(required), DimensionBehaviour::Subnormals(ceiling)) => {
            subnormals_at_least_as_strict(required, ceiling)
        }
        (DimensionBehaviour::Transform(required), DimensionBehaviour::Transform(ceiling)) => {
            transform_at_least_as_strict(required, ceiling)
        }
        (
            DimensionBehaviour::Approximation(required),
            DimensionBehaviour::Approximation(ceiling),
        ) => approximation_at_least_as_strict(required, ceiling),
        (
            DimensionBehaviour::ExceptionalValue(required),
            DimensionBehaviour::ExceptionalValue(ceiling),
        ) => exceptional_value_at_least_as_strict(required, ceiling),
        (DimensionBehaviour::Rounding(required), DimensionBehaviour::Rounding(ceiling)) => {
            rounding_at_least_as_strict(required, ceiling)
        }
        // Every behaviour variant is named on the left, so a widened
        // vocabulary fails to compile here rather than falling into a
        // cross-space arm that answers `false` for a pairing nobody considered.
        (
            DimensionBehaviour::Subnormals(_)
            | DimensionBehaviour::Transform(_)
            | DimensionBehaviour::Approximation(_)
            | DimensionBehaviour::ExceptionalValue(_)
            | DimensionBehaviour::Rounding(_),
            _,
        ) => false,
    }
}

/// `Preserve` is the strict resolution; flushing is the widening away from it.
const fn subnormals_at_least_as_strict(required: SubnormalMode, ceiling: SubnormalMode) -> bool {
    match (required, ceiling) {
        (SubnormalMode::Preserve, SubnormalMode::Preserve | SubnormalMode::FlushToZero { .. }) => {
            true
        }
        (SubnormalMode::FlushToZero { .. }, SubnormalMode::Preserve) => false,
        // Two flush modes differ in the sign of the zero they produce, which is
        // a different observable result rather than a laxer one.
        (
            SubnormalMode::FlushToZero {
                zero_sign: required,
            },
            SubnormalMode::FlushToZero { zero_sign: ceiling },
        ) => match (required, ceiling) {
            (FlushedZeroSign::PreservesSign, FlushedZeroSign::PreservesSign)
            | (FlushedZeroSign::AlwaysPositive, FlushedZeroSign::AlwaysPositive) => true,
            (FlushedZeroSign::PreservesSign, FlushedZeroSign::AlwaysPositive)
            | (FlushedZeroSign::AlwaysPositive, FlushedZeroSign::PreservesSign) => false,
        },
    }
}

/// `Forbidden` is the strict resolution; permitting is the widening.
const fn transform_at_least_as_strict(
    required: NumericalPermission,
    ceiling: NumericalPermission,
) -> bool {
    match (required, ceiling) {
        (
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden | NumericalPermission::Permitted,
        )
        | (NumericalPermission::Permitted, NumericalPermission::Permitted) => true,
        (NumericalPermission::Permitted, NumericalPermission::Forbidden) => false,
    }
}

/// `Forbidden` is the strict resolution; a named envelope is the widening.
///
/// Two *different* named envelopes would be incomparable, and there is one
/// today; the match is written so admitting a second is a build error here
/// rather than an unexamined `true`.
const fn approximation_at_least_as_strict(
    required: ApproximationEnvelope,
    ceiling: ApproximationEnvelope,
) -> bool {
    match (required, ceiling) {
        (
            ApproximationEnvelope::Forbidden,
            ApproximationEnvelope::Forbidden | ApproximationEnvelope::BackendElementary,
        )
        | (ApproximationEnvelope::BackendElementary, ApproximationEnvelope::BackendElementary) => {
            true
        }
        (ApproximationEnvelope::BackendElementary, ApproximationEnvelope::Forbidden) => false,
    }
}

/// `MakeNoAssumption` is the strict resolution; assuming absence is the widening.
///
/// Two assumptions differing only in provenance are not ordered: the provenance
/// records *how* the absence was established, and a caller-declared absence is
/// not a laxer form of a compiler-proven one but a differently evidenced claim.
const fn exceptional_value_at_least_as_strict(
    required: ExceptionalValueAssumption,
    ceiling: ExceptionalValueAssumption,
) -> bool {
    match (required, ceiling) {
        (
            ExceptionalValueAssumption::MakeNoAssumption,
            ExceptionalValueAssumption::MakeNoAssumption
            | ExceptionalValueAssumption::AssumeAbsent { .. },
        ) => true,
        (
            ExceptionalValueAssumption::AssumeAbsent { .. },
            ExceptionalValueAssumption::MakeNoAssumption,
        ) => false,
        (
            ExceptionalValueAssumption::AssumeAbsent {
                provenance: required,
            },
            ExceptionalValueAssumption::AssumeAbsent {
                provenance: ceiling,
            },
        ) => match (required, ceiling) {
            (ValueDomainProvenance::CompilerProven, ValueDomainProvenance::CompilerProven)
            | (ValueDomainProvenance::RuntimeValidated, ValueDomainProvenance::RuntimeValidated)
            | (
                ValueDomainProvenance::CallerDeclaredUnvalidated,
                ValueDomainProvenance::CallerDeclaredUnvalidated,
            ) => true,
            (
                ValueDomainProvenance::CompilerProven
                | ValueDomainProvenance::RuntimeValidated
                | ValueDomainProvenance::CallerDeclaredUnvalidated,
                _,
            ) => false,
        },
    }
}

/// One rounding direction is admitted, so only equality can hold.
///
/// Written as a match rather than `==` so admitting a second direction stops
/// the build here, where someone must decide whether the two are ordered at all.
const fn rounding_at_least_as_strict(
    required: MaterializationRounding,
    ceiling: MaterializationRounding,
) -> bool {
    match (required, ceiling) {
        (
            MaterializationRounding::NearestTiesToEven,
            MaterializationRounding::NearestTiesToEven,
        ) => true,
    }
}

/// The validated scalar subject one arithmetic type computes over.
///
/// **The same association a target profile's declaration is built from, reached
/// by the same constructor.** [`ScalarArithmetic::new`] proves the pair from the
/// governed built-in scalar catalog rather than from either name's spelling, so
/// a requirement and a declaration that both went through it are speaking about
/// one registered value identity by construction. Writing the pairing out here
/// instead would be a second copy of the association, and a copy is a second
/// place for it to be wrong.
///
/// **Total over the arithmetic vocabulary, and that is not the same claim as
/// "this build states contracts in every width".** A subject is what a target is
/// *asked about*; whether a caller may state a contract in that width is decided
/// by whether the key scheme mints one, which is a separate and narrower gate.
/// Keeping the two apart is what lets an `f16` contract be reported `Unknown` by
/// a profile that never mentions `f16`, instead of being proven by a requirement
/// set that quietly emptied itself.
///
/// `None` only if the arithmetic vocabulary and the governed scalar catalog have
/// drifted apart, which `tiler-ir` pins with a test of its own; it is a refusal
/// rather than a panic so drift refuses a subject instead of admitting one no
/// registry describes.
pub(crate) fn arithmetic_subject(arithmetic: ArithmeticType) -> Option<ScalarArithmetic> {
    let resolved_type = crate::target::registered_arithmetic_value_type(arithmetic)?;
    ScalarArithmetic::new(arithmetic, resolved_type).ok()
}

/// The strict resolution of every dimension, for one arithmetic type.
///
/// **The fail-closed default, and the base every composition starts from.**
/// "Strict on this dimension" has one spelling, so a contract that widens a
/// dimension overrides exactly that field and "this contract widens exactly two
/// dimensions" is a readable property of the constructor rather than a claim in
/// a comment. An unstated dimension resolves here, which is what makes omission
/// unable to widen a contract — and what makes a dimension added to the
/// vocabulary later arrive forbidden in every contract that predates it.
///
/// The key is deliberately absent: it is derived from the dimensions by
/// `crate::request::StrictF32NumericalContract::keyed`, so it cannot be stated
/// beside a vector it does not describe. The placeholder this returns is never
/// admitted — `is_governed` compares the key against the canonical encoding of
/// the very fields beside it.
pub(crate) const fn strict_contract(
    arithmetic: ArithmeticType,
    canonical_arithmetic_nan_bits: u32,
) -> StrictF32NumericalContract {
    StrictF32NumericalContract {
        key: UNKEYED_CONTRACT,
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
        ELEMENTARY_UNCARRIED_DIMENSIONS, REALIZED_DIMENSIONS, dimension_requirements,
        is_consumable, operation_capabilities, operation_capability, unrepresentable_dimension,
    };
    use crate::request::StrictF32NumericalContract;
    use crate::target::honourability::{
        CANONICAL_DIMENSIONS, DimensionBehaviour, NumericalDimension,
    };
    use tiler_ir::schedule::{
        ExceptionalValueAssumption, NumericalPermission, SubnormalMode, ValueDomainProvenance,
    };
    use tiler_ir::semantic::{
        FrozenSemanticRegistry, OpKey, add_f32_op, constant_f32_op, multiply_f32_op,
        rms_norm_f32_op, silu_f32_op, softmax_f32_op,
    };

    /// Every named contract states one this build can actually realize.
    ///
    /// This is the check that keeps the named set honest as it grows: a named
    /// contract that widened a consumable dimension the realization cannot carry
    /// would produce two meanings under one region identity, and it fails here
    /// rather than in a cache.
    ///
    /// It is deliberately *not* a claim about the statable space, which is the
    /// whole dimension product and far larger than what this build realizes. The
    /// gate for an arbitrary composed contract is
    /// [`unrepresentable_dimension`] itself, called at the request boundary
    /// before any target is consulted; this loop only pins the points the build
    /// documents.
    #[test]
    fn every_named_contract_is_representable_by_this_build() {
        let named = StrictF32NumericalContract::named_profile();
        assert_eq!(named.len(), 5, "the named set is the population under test");
        for contract in named {
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

    /// Operations the semantic registry admits and this table declares no
    /// numerical capability for.
    ///
    /// **The name is historical and the list is now two different claims.** A
    /// registered operation with no capability row consumes no numerical freedom
    /// and is declined by every rewrite that asks for one; whether the family can
    /// be *planned* is a separate question this table does not answer, and for
    /// the BF16 three the two answers have come apart.
    ///
    /// For BF16 a rowless entry is still the correct state rather than a gap to
    /// be filled: a row would enter each dimension it listed into
    /// `is_consumable`'s union, which is what decides whether a *contract* may
    /// permit that dimension at all, and reassociation and contraction error are
    /// bounded by the significand, so writing one from the `f32` set would widen
    /// this build's numerical surface on evidence about another width.
    ///
    /// **Two grounds that used to be given for it are gone, and the conclusion
    /// is not.** The first was that no arithmetic in this build realizes BF16:
    /// each of the three now carries a governed index-access lowering and a BF16
    /// program is recognized and planned. The second was that a missing row cost
    /// the *fusion* of a multi-occurrence BF16 region:
    /// `establish-bf16-optimizer-legality` gave the three families governed
    /// fusion-capability rows in `crate::fusion_legality`, so such a region now
    /// derives its own legality and fuses, and that table is a different
    /// authority from this one. What a missing row costs today is narrower and
    /// is the whole of it: no BF16 occurrence consumes any numerical freedom, so
    /// every rewrite that asks one for a regrouping or a contraction declines.
    ///
    /// `tiler::concatenate-f32@1` is here for a different reason again, and the
    /// difference is worth keeping. It is unplanned because nothing *physical*
    /// realizes it — no kernel construct writes a partitioned output and the
    /// request boundary refuses the family under `operation-set` — and it
    /// consumes no numerical freedom for a stronger reason than the BF16 rows
    /// do: it performs no arithmetic, so there is no dimension a capability row
    /// could list. A row would be a claim about a target that concatenating
    /// elements never asks of one. It now holds a `CoordinateRelation` fusion
    /// role and a registered index-access lowering with its realization law, and
    /// neither is in tension with its place here: a fusion role answers whether
    /// fusing an occurrence preserves the numerical contract, and a lowering
    /// answers what *logical* index work realizes it. Both are answerable for a
    /// family performing no arithmetic without any target being asked anything,
    /// which is exactly why neither makes the family plannable.
    ///
    /// `tiler::slice-f32@1` is here for the concatenate's reason rather than the
    /// BF16 rows', and the entry sat unexplained until 2026-08-07. A selection
    /// performs no arithmetic — every result element is an operand element
    /// unchanged, which its registered `SLICE_FACT_VALUE_BEHAVIOUR` states in
    /// canonical attribute bytes — so there is no dimension a capability row
    /// could list, and a row would be a claim about a target that reading a
    /// sub-region never asks of one. Nothing physical realizes it either: the
    /// request boundary refuses the family under `operation-set` because the
    /// region vocabulary cannot spell its access relation. It now holds a
    /// `CoordinateRelation` fusion role in `crate::fusion_legality`, which is no
    /// more in tension with its place here than the concatenate's role is, and
    /// for the identical reason.
    const UNPLANNED_OPERATIONS: &[&str] = &[
        "tiler::add-bf16@1",
        "tiler::concatenate-f32@1",
        "tiler::constant-bf16@1",
        "tiler::multiply-bf16@1",
        "tiler::slice-f32@1",
    ];

    /// The capability table names the operations the registry actually admits.
    ///
    /// Both directions. A key spelled differently from the registry's would put a
    /// name in a rejection that no operation has, and a *missing* row would drop
    /// every requirement that operation places on a target — the direction that
    /// admits a target without ever asking it. The table was written by hand and
    /// its first spelling used underscores where the registry uses hyphens, which
    /// is precisely why this compares against the keys rather than against a
    /// second list.
    ///
    /// The one admitted absence is [`UNPLANNED_OPERATIONS`], subtracted by name
    /// rather than by a predicate over the key text, so a newly registered
    /// operation still has to be added to the capability table or listed there
    /// deliberately. Neither direction weakened.
    #[test]
    fn the_capability_table_names_exactly_the_admitted_operations() {
        let mut declared: Vec<_> = operation_capabilities()
            .iter()
            .map(|capability| capability.key().to_owned())
            .collect();
        let registry = FrozenSemanticRegistry::standard().expect("the governed registry composes");
        let mut expected: Vec<_> = registry
            .operation_definitions()
            .map(|definition| definition.key().to_string())
            .filter(|key| !UNPLANNED_OPERATIONS.contains(&key.as_str()))
            .collect();
        declared.sort();
        expected.sort();
        assert_eq!(declared, expected);
    }

    /// Every unplanned operation is registered, rowless, and consumes nothing.
    ///
    /// Without this the subtraction above degrades silently: a name matching no
    /// registered operation would exclude nothing and read as a pass, and a
    /// capability row appearing later for one of these would go unnoticed.
    #[test]
    fn every_unplanned_operation_is_registered_and_consumes_no_dimension() {
        let registry = FrozenSemanticRegistry::standard().expect("the governed registry composes");
        let registered: Vec<String> = registry
            .operation_definitions()
            .map(|definition| definition.key().to_string())
            .collect();
        assert!(
            registered.len() > UNPLANNED_OPERATIONS.len(),
            "the subtraction leaves a nonempty population to compare"
        );
        for key in UNPLANNED_OPERATIONS {
            assert!(
                registered.iter().any(|candidate| candidate == key),
                "{key} is subtracted from the capability comparison, so it must be registered"
            );
            assert!(
                operation_capabilities()
                    .iter()
                    .all(|capability| capability.key() != *key),
                "{key} must carry no capability row"
            );
            let operation = OpKey::new(
                "tiler",
                key.trim_start_matches("tiler::").trim_end_matches("@1"),
                1,
            )
            .expect("an unplanned operation key is well formed");
            assert!(
                operation_capability(&operation).is_none(),
                "{key} resolves to no capability, so every rewrite asking for one declines"
            );
            for dimension in CANONICAL_DIMENSIONS {
                assert!(
                    !operation_capabilities()
                        .iter()
                        .any(|capability| capability.key() == *key
                            && capability.can_consume(dimension)),
                    "{key} consumes no {}",
                    dimension.key()
                );
            }
        }
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

    /// The withheld elementary dimensions are exactly the ones outside the realization.
    ///
    /// **This is the check that fires when the deferral stops being honest.** The
    /// moment `NumericalRealization` grows to carry either dimension, the
    /// reason for withholding its row disappears and this assertion fails,
    /// which is the only signal that would otherwise be missing: nothing else
    /// relates the capability table to the realization's contents.
    #[test]
    fn the_uncarried_elementary_dimensions_are_outside_the_realization() {
        for dimension in ELEMENTARY_UNCARRIED_DIMENSIONS {
            assert!(
                !REALIZED_DIMENSIONS.contains(&dimension),
                "{} is now carried by the region realization, so tiler::silu-f32@1, \
                 tiler::rms-norm-f32@1, and tiler::softmax-f32@1 must gain its capability row \
                 rather than continue to withhold it",
                dimension.key()
            );
            assert!(
                !is_consumable(dimension),
                "{} is withheld from every row, so no contract may be asked to resolve it against \
                 a target",
                dimension.key()
            );
        }
    }

    /// The activation's row is the arithmetic row without contraction.
    ///
    /// Named dimension by dimension rather than compared against another row, so
    /// that a change to the pointwise arithmetic row does not silently move this
    /// one with it.
    #[test]
    fn the_activation_consumes_the_arithmetic_dimensions_except_contraction() {
        let capability = operation_capability(&silu_f32_op()).expect("the activation is admitted");
        for dimension in [
            NumericalDimension::InputSubnormals,
            NumericalDimension::ResultSubnormals,
            NumericalDimension::Reassociation,
            NumericalDimension::SignedZero,
            NumericalDimension::NanAssumptions,
            NumericalDimension::InfinityAssumptions,
        ] {
            assert!(capability.can_consume(dimension), "{}", dimension.key());
        }
        for dimension in [
            NumericalDimension::Contraction,
            NumericalDimension::Permutation,
            NumericalDimension::MaterializationRounding,
        ] {
            assert!(!capability.can_consume(dimension), "{}", dimension.key());
        }
        assert_eq!(capability.consumes().len(), 6);
    }

    /// The normalization's row adds contraction to the reduction dimensions.
    ///
    /// Named dimension by dimension rather than compared against the reduction
    /// or contraction rows, so a change to either does not silently move this
    /// one. The contraction entry is the load-bearing one: the squaring prologue
    /// puts a multiply beside the fold's add, so a target genuinely is asked
    /// about a fused multiply-add here where the bare serial sum never is.
    #[test]
    fn the_normalization_consumes_the_reduction_dimensions_and_contraction() {
        let capability =
            operation_capability(&rms_norm_f32_op()).expect("the normalization is admitted");
        for dimension in [
            NumericalDimension::InputSubnormals,
            NumericalDimension::ResultSubnormals,
            NumericalDimension::Contraction,
            NumericalDimension::Reassociation,
            NumericalDimension::Permutation,
            NumericalDimension::SignedZero,
            NumericalDimension::NanAssumptions,
            NumericalDimension::InfinityAssumptions,
        ] {
            assert!(capability.can_consume(dimension), "{}", dimension.key());
        }
        assert!(!capability.can_consume(NumericalDimension::MaterializationRounding));
        assert_eq!(capability.consumes().len(), 8);
        // The difference from the bare serial sum is exactly the contraction
        // entry, and it is asserted rather than described.
        let serial = operation_capability(
            &OpKey::new("tiler", "strict-serial-sum-f32", 1).expect("a governed key"),
        )
        .expect("the strict serial sum is admitted");
        assert!(!serial.can_consume(NumericalDimension::Contraction));
    }

    /// The softmax's row is the reduction dimensions without contraction.
    ///
    /// Named dimension by dimension rather than compared against another row, so
    /// a change to any of them does not silently move this one. The *absent*
    /// contraction entry is the load-bearing assertion, and it is asserted
    /// against the normalization's presence of it in the same test: the two
    /// families both embed an ordered sum, and only one of them has a
    /// multiply-add adjacency whose fusion can change a result.
    #[test]
    fn the_softmax_consumes_the_reduction_dimensions_without_contraction() {
        let capability = operation_capability(&softmax_f32_op()).expect("the softmax is admitted");
        for dimension in [
            NumericalDimension::InputSubnormals,
            NumericalDimension::ResultSubnormals,
            NumericalDimension::Reassociation,
            NumericalDimension::Permutation,
            NumericalDimension::SignedZero,
            NumericalDimension::NanAssumptions,
            NumericalDimension::InfinityAssumptions,
        ] {
            assert!(capability.can_consume(dimension), "{}", dimension.key());
        }
        assert!(!capability.can_consume(NumericalDimension::Contraction));
        assert!(!capability.can_consume(NumericalDimension::MaterializationRounding));
        assert_eq!(capability.consumes().len(), 7);
        // The normalization *does* consume contraction, so the absence above is
        // a property of this operation rather than of the dimension.
        let normalization =
            operation_capability(&rms_norm_f32_op()).expect("the normalization is admitted");
        assert!(normalization.can_consume(NumericalDimension::Contraction));
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
        let ceiling = StrictF32NumericalContract::governed_relaxed();
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
        // Ordered same-operation regrouping consumes reassociation and takes
        // the ceiling's value for both pointwise arithmetic and reductions.
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

    /// Pointwise arithmetic owns the ordered-reassociation decision.
    #[test]
    fn pointwise_arithmetic_reassociation_is_capability_gated_and_contract_resolved() {
        let strict = StrictF32NumericalContract::governed();
        let relaxed = StrictF32NumericalContract::governed_relaxed();
        for operation in [add_f32_op(), multiply_f32_op()] {
            let capability =
                operation_capability(&operation).expect("pointwise arithmetic is admitted");
            assert_eq!(
                capability.effective(NumericalDimension::Reassociation, &strict),
                Some(DimensionBehaviour::Transform(
                    NumericalPermission::Forbidden
                ))
            );
            assert_eq!(
                capability.effective(NumericalDimension::Reassociation, &relaxed),
                Some(DimensionBehaviour::Transform(
                    NumericalPermission::Permitted
                ))
            );
        }
        assert_eq!(
            operation_capability(&constant_f32_op())
                .expect("constant is admitted")
                .effective(NumericalDimension::Reassociation, &relaxed),
            None
        );
    }

    /// Every named contract resolves to a distinct key.
    ///
    /// A named point of the space, not the space: injectivity over the whole
    /// dimension product is checked exhaustively by
    /// `crate::request::tests::the_canonical_key_is_injective_over_the_statable_space`.
    /// This pins the documented five, so a named contract accidentally spelled
    /// the same as a sibling fails here rather than by two names quietly sharing
    /// one artifact.
    #[test]
    fn named_contract_keys_are_distinct() {
        let named = StrictF32NumericalContract::named_profile();
        let mut keys: Vec<_> = named.iter().map(|contract| contract.key).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), named.len());
    }

    /// The fold-bearing families are exactly the ones that reduce.
    ///
    /// [`OperationNumericalCapability::folds`] reads the permutation entry
    /// rather than a second list, so this is what keeps that derivation honest:
    /// the families are named here one by one, and a capability row that gained
    /// or lost a permutation entry fails this test instead of silently moving
    /// an obligation's locus between an accumulator and a computation.
    #[test]
    fn the_fold_bearing_families_are_exactly_the_reducing_ones() {
        const FOLDING: &[&str] = &[
            "tiler::rms-norm-f32@1",
            "tiler::softmax-f32@1",
            "tiler::strict-serial-sum-f32@1",
            "tiler::strict-tensor-contraction-f32@1",
        ];
        let mut observed: Vec<&str> = operation_capabilities()
            .iter()
            .filter(|capability| capability.folds())
            .map(|capability| capability.key())
            .collect();
        observed.sort_unstable();
        assert_eq!(observed, FOLDING);
        assert!(
            operation_capabilities().len() > FOLDING.len(),
            "the non-folding families are a nonempty population, so the filter \
             above is discriminating rather than vacuous",
        );
        // Each folding family embeds an ordered contributor sequence and each
        // other family computes every result element independently, which is
        // the claim `folds` reads off the permutation entry.
        for capability in operation_capabilities() {
            assert_eq!(
                capability.folds(),
                capability.can_consume(NumericalDimension::Permutation),
                "{} must derive its fold from its own permutation entry",
                capability.key(),
            );
        }
    }

    /// Every consumable dimension of every admitted family founds a locus.
    ///
    /// The producer refuses a consumable dimension with no founded position, so
    /// this is the check that the refusal is unreachable on today's table
    /// rather than merely unexercised. It also pins the four loci this build
    /// emits: nothing here produces a component or a materialization row.
    #[test]
    fn every_consumable_dimension_founds_a_locus() {
        use tiler_ir::numerics::PolicyLocus;

        let mut founded = 0_usize;
        let mut emitted: Vec<PolicyLocus> = Vec::new();
        for capability in operation_capabilities() {
            for dimension in CANONICAL_DIMENSIONS {
                let locus = capability.founded_locus(dimension);
                if !capability.can_consume(dimension) {
                    continue;
                }
                let locus = locus.unwrap_or_else(|| {
                    panic!(
                        "{} consumes {} and must found a position for it",
                        capability.key(),
                        dimension.key(),
                    )
                });
                founded += 1;
                emitted.push(locus);
            }
        }
        assert_eq!(
            founded, 50,
            "the consumable (operation, dimension) pairs are the population under test",
        );
        emitted.sort_unstable();
        emitted.dedup();
        assert_eq!(
            emitted,
            [
                PolicyLocus::Input,
                PolicyLocus::Computation,
                PolicyLocus::Accumulator,
                PolicyLocus::Result,
            ],
            "this build founds four positions; a component obligation needs a \
             compound value whose conversion contract is its own, and a \
             materialization obligation needs a boundary an operation \
             capability cannot site",
        );

        // The three unfounded dimensions are unfounded for every family, and
        // none of them is consumable, so no row is ever dropped by the gap.
        for dimension in [
            NumericalDimension::ReciprocalTransform,
            NumericalDimension::ApproximateIntrinsics,
            NumericalDimension::MaterializationRounding,
        ] {
            assert!(!is_consumable(dimension), "{}", dimension.key());
            for capability in operation_capabilities() {
                assert_eq!(capability.founded_locus(dimension), None);
            }
        }
    }

    /// The dimensions whose freedoms act on any rounded binary32 operation.
    ///
    /// Not a copy of a capability row: it is the six dimensions whose
    /// definitions are properties of *rounding itself* rather than of an
    /// operation's shape. An operand is read before the rounding
    /// (`InputSubnormals`), a value is produced by it (`ResultSubnormals`), the
    /// operation joins an operand sequence some rewrite could regroup
    /// (`Reassociation`), and the rounded arithmetic has a signed zero, a NaN,
    /// and an infinity to answer for. The two dimensions deliberately outside
    /// this set are the two that depend on an operation's *structure* rather
    /// than on its rounding: `Contraction` needs a multiply adjacent to an add
    /// and `Permutation` needs a contributor fold. Those two stay pinned by
    /// name, per family, in the tests around this one.
    const ARITHMETIC_CORE: [NumericalDimension; 6] = [
        NumericalDimension::InputSubnormals,
        NumericalDimension::ResultSubnormals,
        NumericalDimension::Reassociation,
        NumericalDimension::SignedZero,
        NumericalDimension::NanAssumptions,
        NumericalDimension::InfinityAssumptions,
    ];

    /// Whether one governed scalar operation rounds a binary32 result.
    ///
    /// Total over the keys the governed lowerings declare, and an unclassified
    /// one panics rather than answering `false`: a lowering that began emitting
    /// a scalar operation nobody had classified would otherwise silently make
    /// its family look exact, which is the direction this whole check exists to
    /// catch.
    ///
    /// `constant-f32` retains a declared bit pattern and `canonicalize-nan-f32`
    /// replaces a NaN payload with the contract's canonical one; neither rounds,
    /// and neither is the sole emission of any family below.
    /// `strict-affine-u4-dequantize` computes, but its complete rounding,
    /// saturation, and exceptional-value behaviour is fixed by its versioned
    /// scheme contract rather than resolved by these generic dimensions, which
    /// is the same reason [`operation_capabilities`] gives that family an empty
    /// row.
    fn rounds_binary32(scalar: &str) -> bool {
        match scalar {
            "multiply-f32" | "add-f32" | "divide-f32" | "exp-f32" | "rsqrt-f32" => true,
            "constant-f32" | "canonicalize-nan-f32" | "strict-affine-u4-dequantize" => false,
            other => panic!(
                "tiler.scalar::{other} is emitted by a governed lowering and is \
                 classified neither as rounding binary32 nor as exact, so no \
                 capability row can be checked against it",
            ),
        }
    }

    /// A family whose realization rounds must claim the whole arithmetic core.
    ///
    /// **The safety direction of [`OperationNumericalCapability::can_consume`],
    /// turned from a stated intention into a check.** A capability row that
    /// under-claims used to be harmless — the delivered-realization producer
    /// emitted a row per honoured dimension at every covered occurrence, so a
    /// missing entry cost nothing. It now decides whether a row is emitted at
    /// all, and a dimension left with no row is derived by the artifact builder
    /// as `NotRequired`: a positive claim that no packaged route needs the
    /// target to honour it, and the one claim the neutral artifact cannot
    /// re-check. The failure direction inverted, so an under-claim is now a
    /// silently wrong artifact rather than a redundant row.
    ///
    /// **What makes this a check rather than the table restated.** The oracle is
    /// [`crate::governed::governed_index_access_capabilities`]' `emitted`
    /// declaration — the scalar operations each family's *lowering* may apply.
    /// It is a different statement, written for a different purpose, and it is
    /// independently held honest: `crate::legality::refine_index_region` proves
    /// the region a family actually emits is contained in it, so an under-claim
    /// here cannot be hidden by editing that declaration to match. A row
    /// narrowed to make an obligation disappear therefore has to contradict what
    /// the lowering emits.
    ///
    /// The converse arm is the same rule read backwards, and it is the stricter
    /// of the two: a family whose lowering applies no rounding must claim
    /// *nothing*, because an empty row is the positive claim `operation_capabilities`
    /// documents for the constant, the reindex, the broadcast, and the affine
    /// conversions rather than an unfinished one.
    ///
    /// Three admitted families ship no governed index-access lowering, so this
    /// oracle cannot speak for them. They are named rather than skipped, and
    /// their rows are pinned dimension by dimension by the tests above.
    #[test]
    fn an_arithmetic_family_claims_the_whole_arithmetic_core() {
        use std::collections::BTreeSet;

        let lowerings = crate::governed::governed_index_access_capabilities()
            .expect("the governed lowering capabilities compose");
        let mut lowered: BTreeSet<String> = BTreeSet::new();
        let mut rounding = 0_usize;
        let mut exact = 0_usize;
        for lowering in &lowerings {
            let operation = lowering.operation().to_string();
            if UNPLANNED_OPERATIONS.contains(&operation.as_str()) {
                // A family this build cannot plan carries no capability row by
                // design, checked in both directions by
                // `every_unplanned_operation_is_registered_and_consumes_no_dimension`.
                continue;
            }
            let capability = operation_capability(lowering.operation())
                .unwrap_or_else(|| panic!("{operation} lowers but declares no capability row"));
            let rounds = lowering
                .emitted()
                .iter()
                .any(|scalar| rounds_binary32(scalar.name()));
            if rounds {
                rounding += 1;
                for dimension in ARITHMETIC_CORE {
                    assert!(
                        capability.can_consume(dimension),
                        "{operation} emits a rounding binary32 operation and must \
                         consume {}: a row missing it now yields no obligation, and \
                         the artifact asserts `NotRequired` for a dimension the \
                         route genuinely relies on",
                        dimension.key(),
                    );
                }
            } else {
                exact += 1;
                assert!(
                    capability.consumes().is_empty(),
                    "{operation} applies no rounding binary32 operation, so its \
                     empty row is the strict claim rather than an unfinished one: \
                     {:?}",
                    capability.consumes(),
                );
            }
            lowered.insert(operation);
        }
        assert_eq!(
            rounding, 6,
            "the rounding families are the population the core is checked over",
        );
        assert_eq!(
            exact, 4,
            "the exact families are the population the empty row is checked over",
        );
        assert_eq!(
            rounding + exact,
            lowered.len(),
            "every examined lowering fell into exactly one arm",
        );

        // The families this oracle cannot speak for, named so a new one cannot
        // arrive unchecked and unnoticed.
        let unlowered: Vec<&str> = operation_capabilities()
            .iter()
            .map(|capability| capability.key())
            .filter(|key| !lowered.contains(*key))
            .collect();
        assert_eq!(
            unlowered,
            [
                "tiler::softmax-f32@1",
                "tiler::assemble-strict-affine@1",
                "tiler::quantize-strict-affine@1",
            ],
            "these ship no governed index-access lowering, so their rows are \
             pinned by name instead: the softmax by \
             `the_softmax_consumes_the_reduction_dimensions_without_contraction`, \
             and the two affine conversions by their empty rows here and in \
             `the_capability_table_names_exactly_the_admitted_operations`",
        );
    }

    /// A subnormal freedom is founded at an operand read and at a produced value.
    ///
    /// The pair that makes two loci of one occurrence differ, checked against
    /// the table rather than through a compilation, so the mapping is pinned
    /// where it is written.
    #[test]
    fn the_subnormal_dimensions_are_founded_on_opposite_sides_of_the_operation() {
        use tiler_ir::numerics::PolicyLocus;

        let arithmetic = operation_capability(&add_f32_op()).expect("the add is admitted");
        assert_eq!(
            arithmetic.founded_locus(NumericalDimension::InputSubnormals),
            Some(PolicyLocus::Input),
        );
        assert_eq!(
            arithmetic.founded_locus(NumericalDimension::ResultSubnormals),
            Some(PolicyLocus::Result),
        );
        // The reshaping freedoms move with the operation; the subnormal ones do
        // not, because an operand read and a produced value exist either way.
        let fold = operation_capability(
            &OpKey::new("tiler", "strict-serial-sum-f32", 1).expect("a governed key"),
        )
        .expect("the strict serial sum is admitted");
        assert_eq!(
            fold.founded_locus(NumericalDimension::InputSubnormals),
            Some(PolicyLocus::Input),
        );
        assert_eq!(
            fold.founded_locus(NumericalDimension::Reassociation),
            Some(PolicyLocus::Accumulator),
        );
        assert_eq!(
            arithmetic.founded_locus(NumericalDimension::Reassociation),
            Some(PolicyLocus::Computation),
        );
    }

    /// The strictness order agrees with an independent reading of the rule.
    ///
    /// **Exhaustive over the whole behaviour vocabulary, against an oracle
    /// derived from the strict contract rather than from the function under
    /// test.** The documented rule is that each space has one strict resolution
    /// — the one [`strict_contract`] writes — and that a requirement is at
    /// least as strict as a ceiling exactly when the two are equal or the
    /// requirement *is* that strict resolution. Restating the implementation's
    /// own match here would prove nothing; deriving the oracle from the
    /// contract means a widening that changed the order has to disagree with
    /// what this build calls strict.
    #[test]
    fn the_strictness_order_is_equality_or_the_strict_resolution() {
        let strict = super::strict_contract(
            tiler_ir::schedule::ArithmeticType::F32,
            tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS,
        );
        // The strict resolution of each space, read from the contract that
        // defines "strict" rather than written out again.
        let strictest: Vec<DimensionBehaviour> = CANONICAL_DIMENSIONS
            .into_iter()
            .map(|dimension| strict.behaviour(dimension))
            .collect();
        let is_strictest = |behaviour: DimensionBehaviour| strictest.contains(&behaviour);

        let population = behaviour_population();
        assert_eq!(
            population.len(),
            12,
            "the behaviour vocabulary is the population under test",
        );
        let mut compared = 0_usize;
        for required in &population {
            for ceiling in &population {
                let expected = required.space() == ceiling.space()
                    && (required == ceiling || is_strictest(*required));
                assert_eq!(
                    super::is_at_least_as_strict_as(*required, *ceiling),
                    expected,
                    "{} against {}",
                    required.key(),
                    ceiling.key(),
                );
                compared += 1;
            }
        }
        assert_eq!(compared, 144, "every ordered pair was compared");
    }

    /// The order refuses the pairs a silently wrong producer would need.
    ///
    /// Named individually rather than left to the exhaustive sweep above, so
    /// the specific direction that matters — a locus claiming a freedom the
    /// caller's contract forbids — is asserted rather than merely covered.
    #[test]
    fn a_locus_may_not_be_weaker_than_the_ceiling() {
        let forbidden = DimensionBehaviour::Transform(NumericalPermission::Forbidden);
        let permitted = DimensionBehaviour::Transform(NumericalPermission::Permitted);
        assert!(super::is_at_least_as_strict_as(forbidden, permitted));
        assert!(
            !super::is_at_least_as_strict_as(permitted, forbidden),
            "a locus permitting a transform the contract forbids is the silent \
             direction and must be refused",
        );

        let preserve = DimensionBehaviour::Subnormals(SubnormalMode::Preserve);
        let flush = DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
            zero_sign: tiler_ir::schedule::FlushedZeroSign::PreservesSign,
        });
        assert!(super::is_at_least_as_strict_as(preserve, flush));
        assert!(!super::is_at_least_as_strict_as(flush, preserve));

        // Two flush modes are different, not ordered: neither direction passes.
        let positive = DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
            zero_sign: tiler_ir::schedule::FlushedZeroSign::AlwaysPositive,
        });
        assert!(!super::is_at_least_as_strict_as(flush, positive));
        assert!(!super::is_at_least_as_strict_as(positive, flush));

        // Two spaces never compare, in either direction.
        assert!(!super::is_at_least_as_strict_as(preserve, forbidden));
        assert!(!super::is_at_least_as_strict_as(forbidden, preserve));
    }

    /// Every behaviour this vocabulary can take, for an exhaustive sweep.
    fn behaviour_population() -> Vec<DimensionBehaviour> {
        use tiler_ir::schedule::FlushedZeroSign;

        let mut population = vec![
            DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
            DimensionBehaviour::Transform(NumericalPermission::Forbidden),
            DimensionBehaviour::Transform(NumericalPermission::Permitted),
            DimensionBehaviour::Approximation(tiler_ir::schedule::ApproximationEnvelope::Forbidden),
            DimensionBehaviour::Approximation(
                tiler_ir::schedule::ApproximationEnvelope::BackendElementary,
            ),
            DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
            DimensionBehaviour::Rounding(
                tiler_ir::schedule::MaterializationRounding::NearestTiesToEven,
            ),
        ];
        for zero_sign in [
            FlushedZeroSign::PreservesSign,
            FlushedZeroSign::AlwaysPositive,
        ] {
            population.push(DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
                zero_sign,
            }));
        }
        for provenance in [
            ValueDomainProvenance::CompilerProven,
            ValueDomainProvenance::RuntimeValidated,
            ValueDomainProvenance::CallerDeclaredUnvalidated,
        ] {
            population.push(DimensionBehaviour::ExceptionalValue(
                ExceptionalValueAssumption::AssumeAbsent { provenance },
            ));
        }
        population
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
