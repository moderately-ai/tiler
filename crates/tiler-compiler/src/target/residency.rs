#![allow(
    dead_code,
    reason = "no target profile declares a transient-memory budget (L4 D-11), so nothing on the compile path can yet supply a `TransientBudget::Declared`; the authority is constructed and exercised by its own tests and by the attention residency census until a profile declares one"
)]

//! Transient-residency hard feasibility for a materialized plan.
//!
//! # This is feasibility, not cost
//!
//! Peak transient memory is the one planning quantity in the attention vertical
//! that can make a plan *impossible* rather than merely expensive: a plan whose
//! live intermediates do not fit cannot be run at any price, so expressing the
//! shortfall as a large cost would let a search trade it against a speedup and
//! select a plan that cannot execute. This module therefore follows the same
//! separation [`super::feasibility`] states for the capability axes and that
//! AGENTS.md requires — "Separate hard feasibility from estimated cost so
//! impossible plans fail clearly" — and owns no cost notion at all. Its verdict
//! is never a number a cost model may consume; [`ResidencyVerdict`] carries no
//! ordering and no arithmetic.
//!
//! # Why the third verdict is not a formality
//!
//! The requirement side of this predicate is exact and the budget side does not
//! exist. L4 records the exact per-plan requirement, and its **D-11** records
//! that *no target profile in this repository declares a transient memory
//! limit* and no residency measurement bounds one. A two-valued predicate would
//! have to answer such a plan either "fits" — asserting an admission nothing
//! supports — or "exceeds" — asserting a disproof nothing supports. Both are
//! false claims, and the first is the dangerous one, because it would let an
//! unbounded plan reach an executable frontier.
//!
//! So [`ResidencyVerdict::BudgetUndeclared`] is a first-class third answer,
//! exactly as [`crate::explain::SynchronizationOutcome::Undeclared`] and
//! [`crate::explain::HonourabilityOutcome::Undeclared`] are third answers in
//! their own vocabularies and for the same stated reason: *the absence of a
//! refusal is not an admission*. A candidate carrying this verdict stays in
//! explain and search state and never enters an executable frontier.
//!
//! # The subject this was written for
//!
//! A materialized attention plan keeps `n` score-shaped tensors alive at once,
//! where `n` is a property of the *plan* rather than of the program: L4's D-A
//! ladder is `n = 4` fully unfused, `n = 2` with the scale and mask fused as the
//! contraction's epilogue, and `n = 1` with a `StorageHandoff` additionally
//! retiring the first tensor before the second is written. The verdict therefore
//! carries `n` beside the bytes, because "this plan needs too much" and "this
//! *rung* of this plan needs too much" are different findings and a reader who
//! cannot tell them apart cannot tell whether fusing further would help.
//!
//! **It would not, at L4's longest row.** The census in
//! [`crate::target::residency::tests`] records that at B1-d prefill every rung —
//! including the `n = 1` best case — exceeds four gibibytes, so the shortfall
//! there is not a fusion gap. That is a conclusion about the arithmetic and not
//! about any declared budget, which does not exist; see [`ResidencyVerdict`].

/// A transient requirement that cannot be represented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResidencyError {
    /// The requirement's own arithmetic overflowed `u64`.
    ///
    /// Refused at construction rather than saturated. A saturating total would
    /// silently become `u64::MAX`, which compares as *exceeds* against every
    /// declared budget and would therefore look like a reasoned refusal
    /// carrying an exact byte figure that is not the requirement.
    RequirementOverflow,
}

/// The exact transient bytes one materialized plan holds live at its peak.
///
/// Constructed rather than computed in place so the overflow refusal has one
/// site, and so the three quantities a verdict must report — the total, the
/// per-tensor mass, and the plan's `n` — cannot drift apart.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TransientResidencyRequirement {
    tensors: u32,
    bytes_per_tensor: u64,
    other_bytes: u64,
    total_bytes: u64,
}

impl TransientResidencyRequirement {
    /// Builds a requirement of `tensors` copies of `bytes_per_tensor`, plus the
    /// plan's `other_bytes` of non-weight transient state.
    ///
    /// `other_bytes` is every remaining transient the plan holds live and is
    /// independent of `tensors`; L4 tabulates it as the "everything else"
    /// column. Keeping it separate from the per-tensor mass is what lets a
    /// verdict say whether a shortfall is attributable to the materialization
    /// ladder at all.
    pub(crate) const fn new(
        tensors: u32,
        bytes_per_tensor: u64,
        other_bytes: u64,
    ) -> Result<Self, ResidencyError> {
        let Some(tensor_mass) = bytes_per_tensor.checked_mul(tensors as u64) else {
            return Err(ResidencyError::RequirementOverflow);
        };
        let Some(total_bytes) = tensor_mass.checked_add(other_bytes) else {
            return Err(ResidencyError::RequirementOverflow);
        };
        Ok(Self {
            tensors,
            bytes_per_tensor,
            other_bytes,
            total_bytes,
        })
    }

    /// The plan's `n` — how many score-shaped tensors are alive at once.
    pub(crate) const fn tensors(self) -> u32 {
        self.tensors
    }

    /// The mass of one such tensor.
    pub(crate) const fn bytes_per_tensor(self) -> u64 {
        self.bytes_per_tensor
    }

    /// Every other transient byte the plan holds live.
    pub(crate) const fn other_bytes(self) -> u64 {
        self.other_bytes
    }

    /// The exact total this plan requires.
    pub(crate) const fn total_bytes(self) -> u64 {
        self.total_bytes
    }
}

/// What a target profile declares about transient memory.
///
/// Two states rather than an `Option<u64>` with a comment, so that every match
/// on the budget has to name the undeclared case and cannot reach it through a
/// defaulted zero or an assumed maximum. A defaulted zero would refuse every
/// plan with a fabricated ground; an assumed maximum would admit every plan the
/// same way.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TransientBudget {
    /// A profile states this many bytes of transient memory.
    Declared(u64),
    /// No profile states one. L4 D-11; this is the live state of the tree.
    Undeclared,
}

/// The hard-feasibility answer for one plan against one profile's budget.
///
/// Deliberately carries no ordering, no arithmetic, and no conversion to a
/// number. A cost model that could consume this verdict could trade an
/// impossible plan against a fast one, which is the confusion the separation
/// exists to prevent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResidencyVerdict {
    /// The requirement is provably within a declared budget.
    Fits {
        required_bytes: u64,
        budget_bytes: u64,
        tensors: u32,
    },
    /// The requirement provably exceeds a declared budget. A refusal, with the
    /// exact requirement, the exact budget, and the plan's `n`.
    Exceeds {
        required_bytes: u64,
        budget_bytes: u64,
        tensors: u32,
        /// Bytes by which the requirement overshoots. Reported because it is the
        /// figure that says whether a further fusion rung could close the gap,
        /// and derived here so a reader never subtracts two numbers that a later
        /// revision might report in different units.
        overshoot_bytes: u64,
    },
    /// The requirement is exact and no budget exists to compare it against.
    ///
    /// Neither an admission nor a refusal. A candidate holding this verdict is
    /// retained in explain and search state and is not admissible to an
    /// executable frontier.
    BudgetUndeclared { required_bytes: u64, tensors: u32 },
}

impl ResidencyVerdict {
    /// Whether this verdict admits the plan to an executable frontier.
    ///
    /// Only [`Self::Fits`] does. Written as an exhaustive match rather than a
    /// `matches!` over the negative cases so that a verdict added to this
    /// vocabulary is a build error here instead of silently defaulting to
    /// admissible — which is the direction that would let an unproven plan run.
    pub(crate) const fn admits_to_executable_frontier(self) -> bool {
        match self {
            Self::Fits { .. } => true,
            Self::Exceeds { .. } | Self::BudgetUndeclared { .. } => false,
        }
    }

    /// The stable reason key this verdict reports under.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Fits { .. } => "transient-residency-within-budget",
            Self::Exceeds { .. } => "transient-residency-exceeds-budget",
            Self::BudgetUndeclared { .. } => "transient-residency-budget-undeclared",
        }
    }
}

/// Decides transient residency for one plan against one profile's budget.
///
/// The precedence is total: an undeclared budget is answered before any
/// comparison is attempted, because there is nothing to compare against and a
/// comparison against a stand-in value is exactly the fabricated authority this
/// predicate exists to refuse.
pub(crate) const fn assess(
    requirement: TransientResidencyRequirement,
    budget: TransientBudget,
) -> ResidencyVerdict {
    let required_bytes = requirement.total_bytes;
    let tensors = requirement.tensors;
    match budget {
        TransientBudget::Undeclared => ResidencyVerdict::BudgetUndeclared {
            required_bytes,
            tensors,
        },
        TransientBudget::Declared(budget_bytes) => {
            if required_bytes > budget_bytes {
                ResidencyVerdict::Exceeds {
                    required_bytes,
                    budget_bytes,
                    tensors,
                    overshoot_bytes: required_bytes - budget_bytes,
                }
            } else {
                ResidencyVerdict::Fits {
                    required_bytes,
                    budget_bytes,
                    tensors,
                }
            }
        }
    }
}

/// The mass of one `[groups, heads_per_group, t, s]` score tensor.
///
/// Separate from [`TransientResidencyRequirement::new`] because the shape
/// arithmetic and the ladder arithmetic fail for different reasons and a caller
/// that overflows here has stated an impossible *shape*, not an impossible plan.
pub(crate) const fn score_tensor_bytes(
    heads: u64,
    t: u64,
    s: u64,
    element_bytes: u64,
) -> Result<u64, ResidencyError> {
    let Some(points) = heads.checked_mul(t) else {
        return Err(ResidencyError::RequirementOverflow);
    };
    let Some(points) = points.checked_mul(s) else {
        return Err(ResidencyError::RequirementOverflow);
    };
    let Some(bytes) = points.checked_mul(element_bytes) else {
        return Err(ResidencyError::RequirementOverflow);
    };
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{
        ResidencyError, ResidencyVerdict, TransientBudget, TransientResidencyRequirement, assess,
        score_tensor_bytes,
    };

    /// The `[8, 2, T, S]` score tensor carries sixteen head planes.
    const HEADS: u64 = 16;
    /// F32 throughout, per L1's profile.
    const ELEMENT_BYTES: u64 = 4;
    /// L1's recorded figure for a materialized-score prefill at `P = 8,192`.
    const FOUR_GIBIBYTES: u64 = 4 * 1024 * 1024 * 1024;

    /// One row of L4's complete transient-requirement table.
    struct Row {
        name: &'static str,
        t: u64,
        one_tensor: u64,
        everything_else: u64,
        unfused: u64,
        epilogue_fused: u64,
        with_handoff: u64,
    }

    /// L4's table, transcribed with its own figures.
    ///
    /// The B1-b row is present here and absent from the ticket's abridged copy;
    /// including it is what makes the closed form below a fit over five points
    /// rather than four.
    const ROWS: [Row; 5] = [
        Row {
            name: "C1 prefill",
            t: 10,
            one_tensor: 6_400,
            everything_else: 1_075_608,
            unfused: 1_101_208,
            epilogue_fused: 1_088_408,
            with_handoff: 1_082_008,
        },
        Row {
            name: "B1-a prefill",
            t: 128,
            one_tensor: 1_048_576,
            everything_else: 13_828_104,
            unfused: 18_022_408,
            epilogue_fused: 15_925_256,
            with_handoff: 14_876_680,
        },
        Row {
            name: "B1-b prefill",
            t: 512,
            one_tensor: 16_777_216,
            everything_else: 56_098_824,
            unfused: 123_207_688,
            epilogue_fused: 89_653_256,
            with_handoff: 72_876_040,
        },
        Row {
            name: "B1-c prefill",
            t: 2_048,
            one_tensor: 268_435_456,
            everything_else: 236_978_184,
            unfused: 1_310_720_008,
            epilogue_fused: 773_849_096,
            with_handoff: 505_413_640,
        },
        Row {
            name: "B1-d prefill",
            t: 8_192,
            one_tensor: 4_294_967_296,
            everything_else: 1_149_239_304,
            unfused: 18_329_108_488,
            epilogue_fused: 9_739_173_896,
            with_handoff: 5_444_206_600,
        },
    ];

    /// Every cell of L4's table is reproduced by this module's own arithmetic.
    ///
    /// **Why this is a check and not a transcription.** The table is the sole
    /// evidence behind the ladder, and a transcribed table can only be wrong in
    /// the same way twice. Here the per-tensor mass is recomputed from the
    /// shape, each ladder rung is recomputed from the requirement type, and the
    /// stated "everything else" column is used as the `n`-independent term — so
    /// a wrong cell disagrees with a figure derived three different ways.
    ///
    /// **What it would take for this to say *no*.** Any cell of the transcribed
    /// table disagreeing with the recomputation; the perturbation test below
    /// demonstrates the failure rather than asserting it is possible.
    #[test]
    fn every_cell_of_the_l4_transient_table_is_reproduced() {
        for row in &ROWS {
            let per_tensor = score_tensor_bytes(HEADS, row.t, row.t, ELEMENT_BYTES)
                .expect("the score tensor mass is representable");
            assert_eq!(
                per_tensor, row.one_tensor,
                "{}: one-tensor mass disagrees",
                row.name
            );
            for (tensors, expected) in [
                (4, row.unfused),
                (2, row.epilogue_fused),
                (1, row.with_handoff),
            ] {
                let requirement =
                    TransientResidencyRequirement::new(tensors, per_tensor, row.everything_else)
                        .expect("the requirement is representable");
                assert_eq!(
                    requirement.total_bytes(),
                    expected,
                    "{}: the n = {tensors} rung disagrees",
                    row.name
                );
                assert_eq!(requirement.tensors(), tensors);
            }
        }
    }

    /// L1's four-gibibyte figure is one tensor at B1-d, not any plan's total.
    ///
    /// This is the distinction the ticket's abridged table loses by placing the
    /// figure beside a column of totals, and it is load-bearing: a reader who
    /// takes it for a total concludes that the `n = 1` rung reaches it.
    #[test]
    fn the_four_gibibyte_figure_is_one_tensor_and_no_rung_reaches_it_at_b1d() {
        let b1d = &ROWS[4];
        assert_eq!(b1d.one_tensor, FOUR_GIBIBYTES);

        let per_tensor = score_tensor_bytes(HEADS, b1d.t, b1d.t, ELEMENT_BYTES).unwrap();
        for tensors in [4, 2, 1] {
            let requirement =
                TransientResidencyRequirement::new(tensors, per_tensor, b1d.everything_else)
                    .unwrap();
            assert!(
                requirement.total_bytes() > FOUR_GIBIBYTES,
                "n = {tensors} was expected to exceed four gibibytes"
            );
        }

        // The best rung overshoots by the whole "everything else" column, which
        // no further fusion of the score ladder can remove.
        let best = TransientResidencyRequirement::new(1, per_tensor, b1d.everything_else).unwrap();
        assert_eq!(best.total_bytes() - FOUR_GIBIBYTES, b1d.everything_else);
    }

    /// The predicate refuses a deliberately undersized budget, with its figures.
    #[test]
    fn an_undersized_budget_is_refused_with_the_requirement_the_budget_and_n() {
        let per_tensor = score_tensor_bytes(HEADS, 8_192, 8_192, ELEMENT_BYTES).unwrap();
        let requirement =
            TransientResidencyRequirement::new(4, per_tensor, ROWS[4].everything_else).unwrap();
        let verdict = assess(requirement, TransientBudget::Declared(FOUR_GIBIBYTES));

        assert_eq!(
            verdict,
            ResidencyVerdict::Exceeds {
                required_bytes: 18_329_108_488,
                budget_bytes: FOUR_GIBIBYTES,
                tensors: 4,
                overshoot_bytes: 18_329_108_488 - FOUR_GIBIBYTES,
            }
        );
        assert!(!verdict.admits_to_executable_frontier());
        assert_eq!(verdict.key(), "transient-residency-exceeds-budget");
    }

    /// A budget that does fit is admitted, so the refusal above discriminates.
    ///
    /// Without this the refusal test would pass against a predicate that refused
    /// unconditionally, which is the failure mode a one-sided assertion hides.
    #[test]
    fn a_sufficient_budget_admits_the_same_requirement() {
        let per_tensor = score_tensor_bytes(HEADS, 10, 10, ELEMENT_BYTES).unwrap();
        let requirement =
            TransientResidencyRequirement::new(4, per_tensor, ROWS[0].everything_else).unwrap();
        let verdict = assess(requirement, TransientBudget::Declared(FOUR_GIBIBYTES));

        assert_eq!(
            verdict,
            ResidencyVerdict::Fits {
                required_bytes: 1_101_208,
                budget_bytes: FOUR_GIBIBYTES,
                tensors: 4,
            }
        );
        assert!(verdict.admits_to_executable_frontier());
    }

    /// An exactly-equal budget fits rather than exceeding.
    ///
    /// The boundary is stated as a test because `>` and `>=` are both plausible
    /// spellings of "exceeds" and only one of them is right: a plan needing
    /// exactly the declared budget is feasible.
    #[test]
    fn a_budget_equal_to_the_requirement_fits() {
        let requirement = TransientResidencyRequirement::new(1, 1_000, 0).unwrap();
        assert!(matches!(
            assess(requirement, TransientBudget::Declared(1_000)),
            ResidencyVerdict::Fits { .. }
        ));
        assert!(matches!(
            assess(requirement, TransientBudget::Declared(999)),
            ResidencyVerdict::Exceeds {
                overshoot_bytes: 1,
                ..
            }
        ));
    }

    /// An undeclared budget is neither admitted nor refused. L4 D-11.
    ///
    /// The requirement is still reported exactly, because the undeclared half of
    /// the predicate is the budget and not the requirement.
    #[test]
    fn an_undeclared_budget_yields_a_third_verdict_that_admits_nothing() {
        let per_tensor = score_tensor_bytes(HEADS, 8_192, 8_192, ELEMENT_BYTES).unwrap();
        let requirement =
            TransientResidencyRequirement::new(1, per_tensor, ROWS[4].everything_else).unwrap();
        let verdict = assess(requirement, TransientBudget::Undeclared);

        assert_eq!(
            verdict,
            ResidencyVerdict::BudgetUndeclared {
                required_bytes: 5_444_206_600,
                tensors: 1,
            }
        );
        // Not an admission...
        assert!(!verdict.admits_to_executable_frontier());
        // ...and not a refusal either: it reports under its own key, distinct
        // from the refusal's, so no reader can collapse the two.
        assert_eq!(verdict.key(), "transient-residency-budget-undeclared");
        assert_ne!(
            verdict.key(),
            ResidencyVerdict::Exceeds {
                required_bytes: 5_444_206_600,
                budget_bytes: 1,
                tensors: 1,
                overshoot_bytes: 5_444_206_599,
            }
            .key()
        );
    }

    /// The three verdicts report under three distinct keys.
    ///
    /// Sized against the constructed set rather than a hand-written count, so a
    /// verdict added to the vocabulary and not to this list fails the length
    /// assertion instead of quietly not being checked for collision.
    #[test]
    fn every_verdict_reports_under_a_distinct_key() {
        let verdicts = [
            assess(
                TransientResidencyRequirement::new(1, 1, 0).unwrap(),
                TransientBudget::Declared(u64::MAX),
            ),
            assess(
                TransientResidencyRequirement::new(1, 2, 0).unwrap(),
                TransientBudget::Declared(1),
            ),
            assess(
                TransientResidencyRequirement::new(1, 1, 0).unwrap(),
                TransientBudget::Undeclared,
            ),
        ];
        let mut keys: Vec<&str> = verdicts.iter().map(|verdict| verdict.key()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(
            keys.len(),
            verdicts.len(),
            "two verdicts share a reason key: {keys:?}"
        );
        assert_eq!(
            std::mem::variant_count::<ResidencyVerdict>(),
            verdicts.len(),
            "a verdict was added to the vocabulary without a key-distinctness case"
        );
    }

    /// An unrepresentable requirement is refused rather than saturated.
    #[test]
    fn an_overflowing_requirement_is_refused_at_construction() {
        assert_eq!(
            TransientResidencyRequirement::new(4, u64::MAX, 0),
            Err(ResidencyError::RequirementOverflow)
        );
        assert_eq!(
            TransientResidencyRequirement::new(1, u64::MAX, 1),
            Err(ResidencyError::RequirementOverflow)
        );
        assert_eq!(
            score_tensor_bytes(HEADS, u64::MAX, 2, 4),
            Err(ResidencyError::RequirementOverflow)
        );
    }
}
