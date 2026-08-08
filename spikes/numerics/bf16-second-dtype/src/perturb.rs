//! Deliberate perturbations, each watched failing.
//!
//! Every check this spike reports would still pass if it could not fail. These
//! perturbations each change exactly one thing and assert that the corresponding
//! check *stops* agreeing. A perturbation that does not break its check is
//! reported as a failure of the harness, because it means the check is not
//! measuring what it claims.
//!
//! Each is paired with the unperturbed neighbour it is measured against, which
//! the run reports as agreeing — so a refusal is evidence about the perturbation
//! rather than about a harness that refuses everything.

use tiler_compiler::target::{DTypeDispatchabilityResolution, TargetProfileBuildError};
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::semantic::{ResolvedValueType, TypeKey, builtin_scalar_value_type_facts};

use crate::bf16::{
    Bf16, ExactValue, TieRule, exact_multiply, multiply, round_to_nearest_even, round_with_tie_rule,
};
use crate::corpus::{Operation, witnesses};
use crate::format::BinaryFormat;
use crate::promotion::{
    ACCUMULATOR_WITNESS, FUSED_WITNESS, fold_at, fused_exact, promoted_route, route_through,
};
use crate::routing::{ios_simulator_profile, macos_profile};
use crate::seams::Bf16 as Bf16Marker;

/// One perturbation and what watching it produced.
pub struct Perturbation {
    /// What was perturbed.
    pub subject: &'static str,
    /// Whether the corresponding check noticed.
    pub detected: bool,
    /// The exact observation.
    pub detail: String,
}

/// Flips only the tie rule and confirms the corpus notices.
///
/// The perturbation changes exactly one branch — the exact-halfway case — from
/// ties-to-even to ties-away-from-zero, reusing the same decode, the same exact
/// arithmetic, and the same binade search. Everything that is not the tie rule
/// is held fixed, so a witness that disagrees does so *because of* the tie rule.
///
/// The unperturbed neighbour is reported alongside: under the normative rule
/// every witness agrees, which is what makes the disagreement below a result
/// rather than a harness that disagrees with everything.
#[must_use]
pub fn tie_rule_is_load_bearing() -> Perturbation {
    let mut normative_disagreements = 0_usize;
    let mut perturbed_disagreements = Vec::new();
    for witness in witnesses() {
        let exact = witness.operation.apply(
            Bf16::from_bits(witness.left),
            Bf16::from_bits(witness.right),
        );
        if round_with_tie_rule(&exact, TieRule::ToEven).to_bits() != witness.expected {
            normative_disagreements += 1;
        }
        if round_with_tie_rule(&exact, TieRule::AwayFromZero).to_bits() != witness.expected {
            perturbed_disagreements.push(witness.name);
        }
    }
    Perturbation {
        subject: "ties-to-even replaced by ties-away-from-zero, nothing else changed",
        detected: normative_disagreements == 0 && !perturbed_disagreements.is_empty(),
        detail: format!(
            "under ties-to-even {} of {} witnesses disagree; under ties-away-from-zero {} do, first: {}",
            normative_disagreements,
            witnesses().len(),
            perturbed_disagreements.len(),
            perturbed_disagreements.first().copied().unwrap_or("none")
        ),
    }
}

/// Widens by the wrong shift and confirms the cross-check notices.
#[must_use]
pub fn widening_shift_is_load_bearing() -> Perturbation {
    let mut disagreements = 0_u32;
    for bits in 0..=u16::MAX {
        let value = Bf16::from_bits(bits);
        if value.is_nan() {
            continue;
        }
        // The correct widening is `<< 16`. Shifting by 15 halves every exponent
        // field's placement and must disagree for all but the zeros.
        let wrong = f32::from_bits(u32::from(bits) << 15);
        let right = f32::from_bits(value.widen_to_f32_bits());
        if wrong.to_bits() != right.to_bits() {
            disagreements += 1;
        }
    }
    Perturbation {
        subject: "BF16-to-binary32 widening shifted by 15 instead of 16",
        detected: disagreements > 0,
        detail: format!("{disagreements} of 65,536 encodings disagree under the wrong shift"),
    }
}

/// Confirms an unmeasured dtype resolves `Unknown` rather than inheriting.
///
/// The positive control is BF16 on the macOS profile, which resolves
/// `Dispatchable`. The perturbation asks the *same profile* about a dtype it
/// never declared. If that returned `Dispatchable`, every negative route in this
/// spike would be unreachable.
///
/// # Errors
///
/// Returns the first profile-construction diagnostic.
pub fn unmeasured_dtype_does_not_inherit() -> Result<Perturbation, TargetProfileBuildError> {
    let macos = macos_profile()?;
    let declared = macos.dtype_dispatchability(
        &Bf16Marker::resolved_type(),
        AvailabilityPhase::CompileProfile,
    );
    let undeclared = ResolvedValueType::nominal(
        TypeKey::new("tiler", "f16", 1).expect("the governed F16 key is valid"),
    );
    let resolved = macos.dtype_dispatchability(&undeclared, AvailabilityPhase::CompileProfile);
    Ok(Perturbation {
        subject: "an undeclared dtype asked of the profile that declares BF16",
        detected: matches!(resolved, DTypeDispatchabilityResolution::Unknown)
            && matches!(declared, DTypeDispatchabilityResolution::Dispatchable),
        detail: format!(
            "bf16 resolves {declared:?} on the same profile where f16 resolves {resolved:?}"
        ),
    })
}

/// Confirms the simulator profile's BF16 refusal is not a blanket refusal.
///
/// F32 on the same profile must still resolve `Dispatchable`. A profile that
/// refused everything would produce the same BF16 answer and prove nothing.
///
/// # Errors
///
/// Returns the first profile-construction diagnostic.
pub fn simulator_refusal_is_dtype_specific() -> Result<Perturbation, TargetProfileBuildError> {
    let simulator = ios_simulator_profile()?;
    let bf16 = simulator.dtype_dispatchability(
        &Bf16Marker::resolved_type(),
        AvailabilityPhase::CompileProfile,
    );
    let f32 = simulator.dtype_dispatchability(
        &tiler_ir::semantic::F32::resolved_type(),
        AvailabilityPhase::CompileProfile,
    );
    Ok(Perturbation {
        subject: "the simulator profile asked about its accepted neighbour",
        detected: matches!(bf16, DTypeDispatchabilityResolution::Unsupported)
            && matches!(f32, DTypeDispatchabilityResolution::Dispatchable),
        detail: format!("bf16 resolves {bf16:?} while f32 resolves {f32:?}"),
    })
}

/// Confirms the catalog descriptor lookup can return `None`.
///
/// The positive control is `tiler::bf16@1`, which resolves. The perturbation is
/// a name the catalog does not govern, which must not.
#[must_use]
pub fn descriptor_lookup_can_refuse() -> Perturbation {
    let governed = builtin_scalar_value_type_facts(&Bf16Marker::resolved_type());
    let ungoverned = builtin_scalar_value_type_facts(&ResolvedValueType::nominal(
        TypeKey::new("tiler", "bf17", 1).expect("the probe key is canonical"),
    ));
    Perturbation {
        subject: "a name the catalog does not govern",
        detected: governed.is_some() && ungoverned.is_none(),
        detail: format!(
            "bf16 descriptor present={}, bf17 descriptor present={}",
            governed.is_some(),
            ungoverned.is_some()
        ),
    }
}

/// Confirms the exceptional-value rules are not vacuous.
///
/// `infinity * 0` must be NaN and not infinity. This is the rule most likely to
/// be silently wrong in a reference built by widening to a host type, because
/// the host would produce the same NaN for a different reason.
#[must_use]
pub fn invalid_operations_are_decided() -> Perturbation {
    let product = multiply(Bf16::from_bits(0x7f80), Bf16::from_bits(0x0000));
    let ordinary = multiply(Bf16::from_bits(0x7f80), Bf16::from_bits(0x3f80));
    Perturbation {
        subject: "infinity times zero against infinity times one",
        detected: matches!(product, ExactValue::Nan)
            && matches!(ordinary, ExactValue::Infinite { negative: false }),
        detail: format!("inf*0 = {product:?}, inf*1 = {ordinary:?}"),
    }
}

/// Confirms the operation vocabulary refuses what this spike does not admit.
///
/// FMA is the named non-goal. There is no fused variant on [`Operation`], so a
/// caller cannot express one; this records that as an observed absence rather
/// than a claim, by enumerating the complete admitted set.
///
/// # The declaration is the check, and its failure is a build error
///
/// Alone among the perturbations here, this one's subject is a compile-time
/// property of a type, so no input can turn its line `MISSED`; what a widened
/// [`Operation`] must not do is leave it reporting an absence that stopped being
/// true. Two declarations close that, and neither is a length the list supplies
/// for itself. `ADMITTED` is declared at
/// [`variant_count::<Operation>()`](std::mem::variant_count), so a fused variant
/// added to the enum and not listed is an array-length build error at the
/// declaration. The pattern below then names both entries, so listing it instead
/// is a pattern-length build error at the claim — and naming them settles what a
/// bare length could not, since two entries that repeat one variant and omit the
/// other cannot match `[Multiply, Add]`.
#[must_use]
pub fn fused_operations_are_unexpressible() -> Perturbation {
    const ADMITTED: [Operation; std::mem::variant_count::<Operation>()] =
        [Operation::Multiply, Operation::Add];
    Perturbation {
        subject: "the admitted operation vocabulary",
        detected: matches!(ADMITTED, [Operation::Multiply, Operation::Add]),
        detail: format!(
            "exactly {} operations are expressible ({}); no fused multiply-add variant exists, \
             matching the measured MSL fact that `fma(bfloat, bfloat, bfloat)` does not compile",
            ADMITTED.len(),
            ADMITTED
                .iter()
                .map(|operation| operation.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// Confirms the promoted route's exactness depends on binary32's precision.
///
/// Stage 2 reports zero disagreements between one exact BF16 rounding and a
/// widen/evaluate/narrow route through binary32. A stage that can only report
/// zero is not a check, so this substitutes an intermediate with **BF16's own
/// exponent range and one extra significand bit** — precision 9, where
/// the double-rounding bound `q >= 2p + 2` is `9 >= 18` and fails — and requires the same
/// comparison to start disagreeing. Only the intermediate precision moves: the
/// operands, the operation, and both roundings are the stage's own.
#[must_use]
pub fn promoted_route_depends_on_binary32_precision() -> Perturbation {
    let narrow = BinaryFormat {
        name: "bf16 exponent range at precision 9",
        exponent_bits: 8,
        trailing_significand_bits: 8,
    };
    let partner = Bf16::from_bits(0x3fb2).to_exact();
    let mut binary32_disagreements = 0_usize;
    let mut narrow_disagreements = 0_usize;
    for bits in 0..=u16::MAX {
        let exact = exact_multiply(&Bf16::from_bits(bits).to_exact(), &partner);
        let direct = round_to_nearest_even(&exact);
        if direct != promoted_route(&exact) {
            binary32_disagreements += 1;
        }
        if direct != route_through(narrow, &exact) {
            narrow_disagreements += 1;
        }
    }
    Perturbation {
        subject: "the promoted route's binary32 intermediate replaced by a precision-9 format",
        detected: binary32_disagreements == 0 && narrow_disagreements > 0,
        detail: format!(
            "through binary32 {binary32_disagreements} of 65,536 multiplies disagree with one \
             exact rounding; through precision 9 {narrow_disagreements} do"
        ),
    }
}

/// Confirms the fused witness is about the binary32 rounding boundary.
///
/// The witness disagrees because its addend is small enough for binary32 to round
/// the exact sum back onto a BF16 halfway point. Moving the addend up five
/// binades — to `2^-20`, which binary32 represents exactly beside the product —
/// must make the disagreement vanish. If it did not, the witness would be about
/// the operands rather than about double rounding, and stage 3's conclusion would
/// not follow.
#[must_use]
pub fn the_fused_witness_is_about_double_rounding() -> Perturbation {
    let (a_bits, b_bits, c_bits) = FUSED_WITNESS;
    let witness = fused_exact(a_bits, b_bits, c_bits);
    let witness_differs = round_to_nearest_even(&witness) != promoted_route(&witness);
    // `2^-20`: biased exponent 107, trailing significand zero, sign set.
    let coarse_addend = 0xb580_u16;
    let coarse = fused_exact(a_bits, b_bits, coarse_addend);
    let coarse_differs = round_to_nearest_even(&coarse) != promoted_route(&coarse);
    Perturbation {
        subject: "the fused witness's addend moved from 2^-25 to 2^-20",
        detected: witness_differs && !coarse_differs,
        detail: format!(
            "at {c_bits:#06x} the two routes give {:#06x} and {:#06x}; at {coarse_addend:#06x} \
             both give {:#06x}",
            round_to_nearest_even(&witness).to_bits(),
            promoted_route(&witness).to_bits(),
            round_to_nearest_even(&coarse).to_bits()
        ),
    }
}

/// Confirms the accumulator witness needs its contributors to accumulate.
///
/// Stage 4's two folds differ after four contributors. With **zero** contributors
/// the fold is the seed and the two must agree, which shows the difference is
/// accumulated rounding rather than a disagreement the two accumulators have
/// about the seed itself.
#[must_use]
pub fn the_accumulator_witness_needs_contributors() -> Perturbation {
    let (seed_bits, addend_bits, count) = ACCUMULATOR_WITNESS;
    let accumulated = (
        fold_at(BinaryFormat::BF16, seed_bits, addend_bits, count),
        fold_at(BinaryFormat::BINARY32, seed_bits, addend_bits, count),
    );
    let empty = (
        fold_at(BinaryFormat::BF16, seed_bits, addend_bits, 0),
        fold_at(BinaryFormat::BINARY32, seed_bits, addend_bits, 0),
    );
    Perturbation {
        subject: "the accumulator witness's contributor count reduced to zero",
        detected: accumulated.0 != accumulated.1 && empty.0 == empty.1,
        detail: format!(
            "with {count} contributors the folds give {:#06x} and {:#06x}; with none both give \
             {:#06x}",
            accumulated.0.to_bits(),
            accumulated.1.to_bits(),
            empty.0.to_bits()
        ),
    }
}
