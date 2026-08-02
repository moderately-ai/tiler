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

use tiler_ir::schedule::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, MaterializationRounding,
    NumericalPermission, SubnormalMode,
};

use crate::request::StrictF32NumericalContract;
use crate::target::honourability::{DimensionBehaviour, NumericalDimension, NumericalRequirement};
use tiler_ir::semantic::{F32, OpKey};

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
/// `crate::request::CONTRACT_KEY_DOMAIN` — so a contract that reached admission
/// without being keyed is refused by name rather than admitted under a plausible
/// string. Nothing outside [`strict_contract`] writes it.
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
/// operation consumes no numerical freedom, so every rewrite that asks for one
/// declines, and adding a row would widen `is_consumable`'s union for an
/// operation no target profile can even state a numerical contract for.
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
    crate::target::honourability::CANONICAL_DIMENSIONS
        .into_iter()
        .filter(|dimension| is_consumable(*dimension))
        .map(|dimension| {
            NumericalRequirement::new(
                dimension,
                contract.arithmetic,
                F32::resolved_type(),
                contract.behaviour(dimension),
            )
        })
        .collect()
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
        ExceptionalValueAssumption, NumericalPermission, ValueDomainProvenance,
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

    /// Operations the semantic registry admits and this build cannot plan.
    ///
    /// A registered operation with no capability row consumes no numerical
    /// freedom and is declined by every rewrite that asks for one. For BF16 that
    /// is the correct state rather than a gap to be filled: a row would enter
    /// each dimension it listed into `is_consumable`'s union, which is what
    /// decides whether a *contract* may permit that dimension at all, so writing
    /// one would widen this build's numerical surface on the strength of an
    /// operation nothing downstream can realize. A BF16 numerical row *is*
    /// statable on a target profile now that `ScalarArithmetic` derives the
    /// arithmetic/value-type association from the registered descriptor, and
    /// that does not change this: a subject a profile can speak about is not an
    /// operation this build can plan, and none is declared here.
    const UNPLANNED_OPERATIONS: &[&str] = &[
        "tiler::add-bf16@1",
        "tiler::constant-bf16@1",
        "tiler::multiply-bf16@1",
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
