use super::super::lower_scheduled_region;
use super::support::{
    OperandLayouts, cooperative_contraction_region_with_layouts, declared_operand_offsets,
    guarded_load_count, round_zero_operand_offsets,
};
use crate::schedule::TailPolicy;

/// Every declared operand layout is addressed as declared, not as assumed.
///
/// **This is the construction the defect was found by.** Before the addressing
/// was derived from the declared sources, all four subjects below emitted the
/// same two addresses — `row * K + k` and `column * K + k` — so the three
/// transposed ones read the wrong elements and nothing refused them. The two
/// operands are asserted separately because they fail separately: a left-only
/// transposition leaves the right address correct and vice versa, and a single
/// combined assertion could not tell which addressing is load-bearing.
///
/// `M`, `N`, and `K` are three different values, and the invocation is chosen so
/// all four candidate addresses differ; equal extents would let a wrong stride
/// produce the right number.
#[test]
fn each_declared_operand_layout_is_addressed_as_declared() {
    const OUTPUT_M: u64 = 32;
    const OUTPUT_N: u64 = 48;
    const CONTRACTED: u64 = 16;
    // Workgroup 4 of the 2x3 grid, participant (2, 5) of the 16x16 block.
    const GLOBAL: u64 = 4 * 256 + 37;
    const LOCAL: u64 = 37;
    // Every operand of every layout is judged, and the verdicts are reported
    // together rather than through the first failing assertion. A left-only
    // transposition and a right-only one are separate behaviours that fail
    // separately, and a run that stopped at the first would say nothing about
    // which of the two the addressing actually honours.
    let mut wrong = Vec::new();
    let mut judged = 0_usize;
    for left_transposed in [false, true] {
        for right_transposed in [false, true] {
            let layouts = OperandLayouts {
                left_transposed,
                right_transposed,
            };
            let region = cooperative_contraction_region_with_layouts(
                OUTPUT_M,
                OUTPUT_N,
                CONTRACTED,
                TailPolicy::Exact,
                layouts,
            );
            let observed = round_zero_operand_offsets(&region, GLOBAL, LOCAL);
            let [left, right] = observed.as_slice() else {
                panic!("{layouts:?}: round zero loads exactly two operands, saw {observed:?}");
            };
            let expected =
                declared_operand_offsets(OUTPUT_M, OUTPUT_N, CONTRACTED, layouts, GLOBAL, LOCAL);
            for (operand, observed, expected) in
                [("left", *left, expected[0]), ("right", *right, expected[1])]
            {
                judged += 1;
                if observed != expected {
                    wrong.push(format!(
                        "{layouts:?} {operand}: read {observed}, declared {expected}"
                    ));
                }
            }
        }
    }
    assert_eq!(judged, 8, "four layouts, two operands each");
    assert!(wrong.is_empty(), "mis-addressed operands: {wrong:#?}");
}

/// A transposed operand is a different kernel, not the same one relabelled.
///
/// The four layouts are four distinct bodies. Stated as a body property rather
/// than as a kernel-identity one on purpose: the identity folds the scheduled
/// region's identity, which already differs when an access map differs, so
/// comparing identities would have passed even while every layout emitted the
/// same addresses.
#[test]
fn the_four_operand_layouts_emit_four_distinct_address_pairs() {
    const GLOBAL: u64 = 4 * 256 + 37;
    const LOCAL: u64 = 37;
    let mut seen = Vec::new();
    for left_transposed in [false, true] {
        for right_transposed in [false, true] {
            let region = cooperative_contraction_region_with_layouts(
                32,
                48,
                16,
                TailPolicy::Exact,
                OperandLayouts {
                    left_transposed,
                    right_transposed,
                },
            );
            seen.push(round_zero_operand_offsets(&region, GLOBAL, LOCAL));
        }
    }
    assert_eq!(seen.len(), 4, "four layouts were built");
    let mut sorted = seen.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 4, "distinct address pairs among {seen:?}");
}

/// A transposed operand still lowers, verifies, and keeps its guarded tail.
///
/// The addressing repair must not have narrowed what the emission accepts: the
/// transposed layouts reach a whole verified kernel, under the predicated tail
/// as well as the exact one.
#[test]
fn a_transposed_operand_pair_lowers_to_a_verified_kernel() {
    for tail in [TailPolicy::Exact, TailPolicy::Predicated] {
        let region = cooperative_contraction_region_with_layouts(
            32,
            48,
            16,
            tail,
            OperandLayouts {
                left_transposed: true,
                right_transposed: true,
            },
        );
        let kernel = lower_scheduled_region(&region)
            .unwrap_or_else(|error| panic!("{tail:?}: transposed operands lower: {error:?}"));
        let expected = match tail {
            TailPolicy::Exact => 0,
            TailPolicy::Predicated => 2,
        };
        assert_eq!(guarded_load_count(&kernel), expected, "{tail:?}");
    }
}
