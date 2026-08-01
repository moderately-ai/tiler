//! The computation, accumulator, and conversion evidence.
//!
//! This is the second question the spike answers, added by
//! `design-the-bf16-computation-and-accumulator-contract`. The first question
//! was which seams admit a second dtype; this one is what a BF16 program may
//! compute *in*, accumulate *in*, and convert *to*, and it is answered by
//! deriving bit patterns rather than by argument.
//!
//! # The one route every stage here is about
//!
//! No `bfloat` arithmetic primitive is reachable from MSL for the fused case —
//! finding 29 of the retained Apple record has `metal` rejecting
//! `bfloat v6 = fma(v3, v4, v5)` outright — so the only realization a BF16
//! program can be given for anything wider than a separate multiply or add is a
//! **promoted** one: widen every operand to binary32, evaluate there, round back
//! to BF16 once. Whether that route *is* the BF16 semantics or merely resembles
//! them is not a matter of taste, and the stages below decide it per operation
//! shape:
//!
//! - For a **single** multiply or add it is exact, over a named population.
//! - For a **fused** multiply-add it is not, and stage 3 exhibits the operands.
//! - For an **accumulation** the accumulator's width is observable on its own,
//!   independently of any promotion, and stage 4 exhibits the contributors.
//!
//! # What none of this measures
//!
//! Nothing here runs on a GPU. The Apple facts cited are transcribed from the
//! retained record with their own boundaries, exactly as the rest of this spike
//! transcribes them. Every number this module prints is derived from exact
//! rational arithmetic on the host and would be identical on any host.

use crate::bf16::{
    Bf16, ExactValue, TieRule, exact_add, exact_multiply, exceeds, round_to_nearest_even,
    round_with_tie_rule,
};
use crate::format::BinaryFormat;

/// One stage's verdict, with the population it covered stated in the detail.
pub struct Stage {
    /// What the stage decided.
    pub name: &'static str,
    /// The population, the count, and the witness — never a bare boolean.
    pub detail: String,
    /// Whether the stated claim held.
    pub passed: bool,
}

/// Right operands stage 2 sweeps every BF16 encoding against.
///
/// Four, named for the role each plays rather than chosen for coverage in the
/// abstract: `1.0` is the identity that exposes a route doing anything at all,
/// `1.390625` is an ordinary normal whose significand fills all eight bits so a
/// product needs the full sixteen, `0x0040` is the mid subnormal whose binary32
/// widening is itself subnormal, and `0x7f7f` is the largest finite value, so
/// every overflow boundary in the format is crossed by some pair.
const SWEEP_PARTNERS: [(u16, &str); 4] = [
    (0x3f80, "1.0"),
    (0x3fb2, "1.390625, significand 178"),
    (0x0040, "mid subnormal"),
    (0x7f7f, "largest finite"),
];

/// Addends stage 3 sweeps the fused witness's third operand over.
///
/// Negative powers of two spanning the binary32 rounding boundary of a product
/// near 2: `2^-22` is exactly one binary32 ulp there and `2^-25` is an eighth of
/// one, so the sweep contains both a value binary32 represents exactly beside the
/// product and values it does not.
const FUSED_ADDENDS: [(u16, &str); 4] = [
    (0xb300, "-2^-25"),
    (0xb280, "-2^-26"),
    (0xb380, "-2^-24"),
    (0xb480, "-2^-22, one binary32 ulp near 2"),
];

/// Runs every stage of this module in narrative order.
pub fn stages() -> Vec<Stage> {
    vec![
        the_generic_rounder_agrees_with_the_bf16_oracle(),
        a_single_multiply_or_add_admits_the_promoted_route(),
        a_fused_multiply_add_does_not(),
        the_accumulator_width_is_observable(),
        the_widening_is_exact_and_keeps_subnormals_subnormal(),
        the_narrowing_rules_are_decided(),
    ]
}

/// Rounds an exact value to BF16 through a binary32 intermediate.
///
/// The promoted route, stated once: one rounding into binary32, then one into
/// BF16. Both are round-to-nearest, ties-to-even, because that is what the
/// registered BF16 arithmetic facts state and what binary32 hardware delivers by
/// default.
#[must_use]
pub fn promoted_route(value: &ExactValue) -> Bf16 {
    round_to_nearest_even(&BinaryFormat::BINARY32.round(value, TieRule::ToEven))
}

/// Rounds an exact value to BF16 through an arbitrary intermediate format.
///
/// Only the perturbations use a format other than binary32; a route is not a
/// parameter of the contract.
#[must_use]
pub fn route_through(intermediate: BinaryFormat, value: &ExactValue) -> Bf16 {
    round_to_nearest_even(&intermediate.round(value, TieRule::ToEven))
}

/// Stage 1: the generic rounder reproduces the trusted BF16 one.
///
/// Every later stage reads `BinaryFormat`'s answer, so a disagreement here would
/// make each of them a statement about `format.rs` rather than about BF16. The
/// population is every encoding's own exact value plus every product and sum the
/// stage-2 sweep forms, so it covers zeros, subnormals, normals, infinities,
/// NaNs, overflow, and underflow without a separate list.
fn the_generic_rounder_agrees_with_the_bf16_oracle() -> Stage {
    let mut checked = 0_usize;
    let mut disagreements = 0_usize;
    let mut first: Option<String> = None;
    for bits in 0..=u16::MAX {
        let exact = Bf16::from_bits(bits).to_exact();
        for value in [
            exact.clone(),
            exact_multiply(&exact, &Bf16::from_bits(0x3fb2).to_exact()),
            exact_add(&exact, &Bf16::from_bits(0x0040).to_exact()),
        ] {
            checked += 1;
            let trusted = round_to_nearest_even(&value);
            let generic = BinaryFormat::BF16.round(&value, TieRule::ToEven);
            if trusted.to_exact() != generic {
                disagreements += 1;
                first.get_or_insert_with(|| format!("{bits:#06x}"));
            }
        }
    }
    Stage {
        name: "the generic rounder agrees with the BF16 oracle",
        detail: match &first {
            None => format!(
                "{checked} exact values from all 65,536 encodings and their products and sums; \
                 0 disagreements"
            ),
            Some(witness) => {
                format!("{disagreements} of {checked} disagreed, first at {witness}")
            }
        },
        passed: disagreements == 0,
    }
}

/// Stage 2: for one multiply or one add, the promoted route is exact.
///
/// **What this establishes.** Rounding the mathematically exact result into
/// binary32 and then into BF16 returns the same encoding as rounding it into
/// BF16 once, for every case in the population. The reason is a property of the
/// two formats' parameters and not of this population: an exact product of two
/// BF16 values needs at most sixteen significand bits, which binary32's
/// twenty-four hold exactly, and for the remaining operations Figueroa's
/// double-rounding bound `q >= 2p + 2` is `24 >= 18`, which binary32 satisfies.
///
/// **What it does not establish.** Nothing about a *fused* multiply-add, which
/// is not one of the operations that bound covers; stage 3 is that question, and
/// its answer is the opposite one.
fn a_single_multiply_or_add_admits_the_promoted_route() -> Stage {
    let mut checked = 0_usize;
    let mut disagreements = 0_usize;
    let mut first: Option<String> = None;
    for (partner_bits, partner_name) in SWEEP_PARTNERS {
        let partner = Bf16::from_bits(partner_bits).to_exact();
        for bits in 0..=u16::MAX {
            let left = Bf16::from_bits(bits).to_exact();
            for (operation, exact) in [
                ("multiply", exact_multiply(&left, &partner)),
                ("add", exact_add(&left, &partner)),
            ] {
                checked += 1;
                let direct = round_to_nearest_even(&exact);
                let promoted = promoted_route(&exact);
                if direct != promoted {
                    disagreements += 1;
                    first.get_or_insert_with(|| {
                        format!(
                            "{operation} {bits:#06x} by {partner_name}: direct {:#06x}, promoted {:#06x}",
                            direct.to_bits(),
                            promoted.to_bits()
                        )
                    });
                }
            }
        }
    }
    Stage {
        name: "a single multiply or add admits the promoted binary32 route",
        detail: match &first {
            None => format!(
                "{checked} cases -- all 65,536 encodings against 4 named partners, both \
                 operations; 0 disagreements between one exact rounding and widen/evaluate/narrow"
            ),
            Some(witness) => format!("{disagreements} of {checked} disagreed, first {witness}"),
        },
        passed: disagreements == 0,
    }
}

/// The derived fused witness: `1.5 * 1.390625 + (-2^-25)`.
///
/// Chosen rather than found: `192 * 178 = 267 * 2^7`, so the exact product is
/// `267 * 2^-7`, which is exactly a BF16 halfway point whose lower neighbour has
/// an *odd* quantum count. Ties-to-even therefore sends the tie **up**, while any
/// value strictly below the tie rounds **down** — so an addend small enough for
/// binary32 to round the sum back onto the tie flips the answer by one ulp. The
/// addend must be under half a binary32 ulp of a value near 2, which is `2^-23`,
/// and `2^-25` is comfortably inside it.
pub const FUSED_WITNESS: (u16, u16, u16) = (0x3fc0, 0x3fb2, 0xb300);

/// Stage 3: a fused multiply-add does not admit the promoted route.
///
/// **Why this is the decisive stage.** Finding 29 of the retained Apple record
/// establishes that no `bfloat` FMA exists to lower to, so a fused BF16
/// operation could only ever be realized by promoting through binary32. This
/// stage shows that realization is not the correctly rounded BF16 fused result:
/// it differs by one ulp on the witness. A contract naming itself a fused BF16
/// operation and delivering this route would therefore be stating something
/// false, which is why the design proposes a *mixed-precision* operation whose
/// contract says binary32 and one narrowing instead.
fn a_fused_multiply_add_does_not() -> Stage {
    let (a_bits, b_bits, c_bits) = FUSED_WITNESS;
    let witness = fused_exact(a_bits, b_bits, c_bits);
    let direct = round_to_nearest_even(&witness);
    let promoted = promoted_route(&witness);
    // Beside the derived witness, a bounded sweep, so the disagreement is a
    // population with a count rather than one lucky triple.
    let mut checked = 0_usize;
    let mut disagreements = 0_usize;
    for (addend_bits, _) in FUSED_ADDENDS {
        for b in 0..=u16::MAX {
            checked += 1;
            let exact = fused_exact(a_bits, b, addend_bits);
            if round_to_nearest_even(&exact) != promoted_route(&exact) {
                disagreements += 1;
            }
        }
    }
    Stage {
        name: "a fused multiply-add does not admit the promoted binary32 route",
        detail: format!(
            "witness a={a_bits:#06x} b={b_bits:#06x} c={c_bits:#06x}: one exact rounding gives \
             {:#06x}, widen/fma/narrow gives {:#06x}; sweep of {checked} triples (all 65,536 \
             second operands against 4 named addends, first operand 1.5) found {disagreements} \
             disagreements",
            direct.to_bits(),
            promoted.to_bits()
        ),
        passed: direct != promoted && disagreements > 0,
    }
}

/// The exact, unrounded `a * b + c` over three BF16 encodings.
#[must_use]
pub fn fused_exact(a_bits: u16, b_bits: u16, c_bits: u16) -> ExactValue {
    let product = exact_multiply(
        &Bf16::from_bits(a_bits).to_exact(),
        &Bf16::from_bits(b_bits).to_exact(),
    );
    exact_add(&product, &Bf16::from_bits(c_bits).to_exact())
}

/// The accumulator witness: `1.0` followed by four copies of `2^-9`.
///
/// One BF16 ulp at `1.0` is `2^-7` and half of one is `2^-8`, so a single
/// `2^-9` addend rounds away entirely; four of them sum to exactly `2^-7`, which
/// is representable. A BF16 accumulator therefore loses every contributor and
/// returns `1.0`; a binary32 accumulator keeps all four and returns the value one
/// ulp above it. Both are correct implementations of their own stated contract,
/// which is the point: the contract is what decides, and the landed pure-BF16
/// family states BF16.
pub const ACCUMULATOR_WITNESS: (u16, u16, usize) = (0x3f80, 0x3b00, 4);

/// Stage 4: the accumulator's width is observable, with no promotion involved.
///
/// This is the question the ticket exists for, and it is separate from stage 3:
/// no fused primitive, no conversion, and no target is involved. Two folds of the
/// same contributor sequence in the same order, differing only in the type each
/// partial sum is held at, return different bits.
fn the_accumulator_width_is_observable() -> Stage {
    let (seed_bits, addend_bits, count) = ACCUMULATOR_WITNESS;
    let bf16_accumulated = fold_at(BinaryFormat::BF16, seed_bits, addend_bits, count);
    let f32_accumulated = fold_at(BinaryFormat::BINARY32, seed_bits, addend_bits, count);
    // The control: one contributor cannot separate the two, so the difference is
    // accumulated rounding rather than a property of the operands.
    let one_bf16 = fold_at(BinaryFormat::BF16, seed_bits, addend_bits, 1);
    let one_f32 = fold_at(BinaryFormat::BINARY32, seed_bits, addend_bits, 1);
    Stage {
        name: "the accumulator width is observable without any promotion",
        detail: format!(
            "seed {seed_bits:#06x} plus {count} copies of {addend_bits:#06x}: BF16 accumulator \
             {:#06x}, binary32 accumulator narrowed once {:#06x}; with one contributor both give \
             {:#06x} and {:#06x}",
            bf16_accumulated.to_bits(),
            f32_accumulated.to_bits(),
            one_bf16.to_bits(),
            one_f32.to_bits()
        ),
        passed: bf16_accumulated != f32_accumulated && one_bf16 == one_f32,
    }
}

/// Folds `seed + addend * count` left to right, rounding each partial at
/// `accumulator`, then rounding the result into BF16 once.
///
/// When `accumulator` is BF16 the final rounding is a no-op, so one function
/// states both contracts and neither gets an extra boundary the other lacks.
#[must_use]
pub fn fold_at(accumulator: BinaryFormat, seed_bits: u16, addend_bits: u16, count: usize) -> Bf16 {
    let addend = Bf16::from_bits(addend_bits).to_exact();
    let mut partial = accumulator.round(&Bf16::from_bits(seed_bits).to_exact(), TieRule::ToEven);
    for _ in 0..count {
        partial = accumulator.round(&exact_add(&partial, &addend), TieRule::ToEven);
    }
    round_to_nearest_even(&partial)
}

/// Stage 5: BF16 widens to binary32 exactly, and a subnormal stays subnormal.
///
/// **The derivation, which is specific to BF16.** BF16 and binary32 share an
/// exponent width of eight and therefore a bias of 127, and BF16's seven trailing
/// significand bits are a prefix of binary32's twenty-three. So the widening is a
/// left shift of sixteen and every class maps to its own class: a normal to a
/// normal, a subnormal to a **subnormal**, each zero and infinity to itself, and
/// a NaN payload to a zero-extended NaN payload.
///
/// **Why binary16's exactness has a different derivation, and why it matters.**
/// Binary16's exponent range is strictly *inside* binary32's, so every binary16
/// subnormal widens to a binary32 **normal**; its widening is exact because of two
/// separate inclusions — exponent range and precision — rather than because of a
/// shared field. That difference is measured rather than only derived: findings
/// 24 and 25 of the retained Apple record record that on the qualified Apple9 row
/// `bf16` arithmetic flushes subnormals where `f16` preserves them, and attribute
/// it to exactly this exponent-field difference.
fn the_widening_is_exact_and_keeps_subnormals_subnormal() -> Stage {
    let mut widened_subnormals = 0_usize;
    let mut subnormals = 0_usize;
    let mut value_disagreements = 0_usize;
    for bits in 0..=u16::MAX {
        let value = Bf16::from_bits(bits).to_exact();
        let widened = u64::from(bits) << 16;
        if BinaryFormat::BINARY32.decode(widened) != value {
            value_disagreements += 1;
        }
        if BinaryFormat::BF16.is_subnormal_encoding(u64::from(bits)) {
            subnormals += 1;
            if BinaryFormat::BINARY32.is_subnormal_encoding(widened) {
                widened_subnormals += 1;
            }
        }
    }
    // The contrast, derived from binary16's own parameters rather than asserted:
    // its smallest normal exponent is far above binary32's, so no binary16
    // subnormal can land in binary32's subnormal range.
    let binary16_subnormals_are_binary32_normals =
        BinaryFormat::BINARY16.min_subnormal_exponent() > BinaryFormat::BINARY32.min_exponent();
    Stage {
        name: "BF16 widens to binary32 exactly, and its subnormals stay subnormal",
        detail: format!(
            "all 65,536 encodings widen by a 16-bit shift with {value_disagreements} value \
             disagreements; {widened_subnormals} of {subnormals} BF16 subnormals are binary32 \
             subnormals; binary16's smallest subnormal exponent {} is above binary32's smallest \
             normal exponent {}, so its subnormals are binary32 normals instead",
            BinaryFormat::BINARY16.min_subnormal_exponent(),
            BinaryFormat::BINARY32.min_exponent()
        ),
        passed: value_disagreements == 0
            && subnormals == 254
            && widened_subnormals == subnormals
            && binary16_subnormals_are_binary32_normals,
    }
}

/// The binary32 population stage 6 sweeps, and the step that makes it cover a
/// whole binade.
///
/// `0x3f800000 | (k << 7)` for every `k` below 65,536 steps binary32's significand
/// by 128 and therefore covers exactly `[1.0, 2.0)` — 512 samples inside each of
/// BF16's 128 ulps there, including all 128 exact halfway points. One binade is
/// enough because a narrowing's rounding decision depends on the discarded bits
/// and not on the exponent, and stepping the low bits alone would have swept a
/// single ulp and reported one tie.
const NARROWING_SWEEP_BASE: u64 = 0x3f80_0000;
/// Significand step of the narrowing sweep; see [`NARROWING_SWEEP_BASE`].
const NARROWING_SWEEP_STEP: u32 = 7;

/// Stage 6: the narrowing direction's rounding, overflow, and NaN rules are
/// decided, and each decision changes an answer.
///
/// Four separable claims, each with its own witness, because a narrowing contract
/// that stated only "round to nearest" would leave three of them open:
///
/// 1. **Rounding rule.** Round-to-nearest-ties-to-even and truncation to the high
///    sixteen bits are different contracts. Truncation is the spelling the phrase
///    "BF16 is binary32's high half" invites, and it is wrong for a numeric
///    conversion.
/// 2. **Tie direction.** Ties-to-even and ties-away are different contracts too,
///    and the sweep separates them.
/// 3. **Overflow.** Binary32's finite range exceeds BF16's, so the largest finite
///    binary32 value narrows to an *infinity*. A contract with no overflow rule
///    has nothing to say about it.
/// 4. **NaN totality.** Truncating a NaN payload is not even total: a signalling
///    binary32 NaN whose payload lives only in the low sixteen bits truncates to
///    the infinity encoding, which is a different value class.
fn the_narrowing_rules_are_decided() -> Stage {
    let binary32 = BinaryFormat::BINARY32;
    let mut rounding_differs_from_truncation = 0_usize;
    let mut ties_to_even_differs_from_ties_away = 0_usize;
    for step in 0..=u16::MAX {
        let bits = NARROWING_SWEEP_BASE | (u64::from(step) << NARROWING_SWEEP_STEP);
        let exact = binary32.decode(bits);
        let nearest_even = round_to_nearest_even(&exact);
        let truncated =
            Bf16::from_bits(u16::try_from(bits >> 16).expect("the high half is 16 bits"));
        if nearest_even != truncated {
            rounding_differs_from_truncation += 1;
        }
        if nearest_even != round_with_tie_rule(&exact, TieRule::AwayFromZero) {
            ties_to_even_differs_from_ties_away += 1;
        }
    }
    let largest_finite_binary32 = round_to_nearest_even(&binary32.decode(0x7f7f_ffff));
    // Derived rather than asserted: binary32's largest finite magnitude sits at
    // or above BF16's overflow threshold, which is why the narrowing produces an
    // infinity. Stating the inequality is what makes the encoding above a
    // consequence instead of a remembered constant.
    let exceeds_threshold = !exceeds(
        &BinaryFormat::BF16.overflow_threshold(),
        &binary32.largest_finite(),
    );
    let overflows = largest_finite_binary32.to_bits() == 0x7f80 && exceeds_threshold;
    // A signalling binary32 NaN with an all-zero payload prefix. Its high sixteen
    // bits are `0x7f80`, the positive infinity encoding, so a payload-truncating
    // conversion maps a NaN to an infinity.
    let truncation_of_a_low_payload_nan = 0x7f80_0001_u64 >> 16;
    let truncation_is_not_total = truncation_of_a_low_payload_nan == 0x7f80;
    Stage {
        name: "the narrowing direction's rounding, overflow, and NaN rules are decided",
        detail: format!(
            "over 65,536 binary32 patterns covering all of [1,2): nearest-even differs from truncation in \
             {rounding_differs_from_truncation} and from ties-away in \
             {ties_to_even_differs_from_ties_away}; the largest finite binary32 0x7f7fffff \
             narrows to {:#06x}; a signalling NaN 0x7f800001 truncates to \
             {truncation_of_a_low_payload_nan:#06x}, the infinity encoding",
            largest_finite_binary32.to_bits()
        ),
        passed: rounding_differs_from_truncation > 0
            && ties_to_even_differs_from_ties_away > 0
            && overflows
            && truncation_is_not_total,
    }
}
