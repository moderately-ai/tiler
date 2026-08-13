//! The serial-sum vertical's runs, in two halves that fail for unrelated
//! reasons.
//!
//! Everything above [`the_direct_path_agrees_with_the_oracle_on_the_measured_row`]
//! is deterministic and runs on every host: the operand pair and its counts, the
//! oracle under both groupings, the portfolio the two contracts retain, each
//! alternative's classification, the partition each publishes, and the refusal a
//! declared-grouping oracle produces for a legal grouping a strategy did not
//! declare. The measured runs report their boundary — or its absence — rather
//! than skipping.
//!
//! # The claims this file carries, and where each was before
//!
//! Every case below was reachable only by `cargo run -p tiler-prototype-run`
//! before this crate held it. No `Makefile` target invoked that binary; the only
//! mentions were two Clippy `--exclude` flags. The device-free half now runs on
//! every host in `make full`, and the measured half runs wherever the Apple
//! toolchain and a Metal device are.

use tiler_artifact::program::{BackendEntryKey, MAX_OPAQUE_IDENTITY_BYTES};
use tiler_build::BoundMetalCompileDeclaration;
use tiler_compiler::session::NumericalContract;
use tiler_ir::schedule::ContributorPartition;
use tiler_metal::applicability::{
    MetalGpuFamilySupport, MetalHostApplicabilityPolicy, MetalHostPredicate,
};

use super::{
    COLUMNS, GROUPING_SENSITIVE_OPERANDS, PARALLEL_COLUMNS, PARALLEL_OPERANDS, PARALLEL_ROWS,
    ParallelStrategy, ROW_PATTERNS, ROWS, SEPARATING_COLUMNS, SEPARATING_EXACT_OPERANDS,
    SEPARATING_OPERANDS, SEPARATING_ROWS, compile_under, declaration, declared_grouping_admits,
    declared_partition, input_bits, measured_direct, measured_offer, measured_portfolio,
    ordered_associations, pack_f32, partitioned_reference, reference_bits, serial_sum_program,
    unpack_f32,
};
use crate::applicability::{observe_host_environment, refuse_to_offer_the_declared_profile};
use crate::measurement::require_or_report;

/// The degenerate partition: one contributor each, combined in ascending order.
///
/// **This is the declared serial order**, and naming it once is what lets the
/// calibration step and the refusal case be asked about the same thing.
fn serial_order(contributors: u64) -> ContributorPartition {
    ContributorPartition {
        partitions: contributors,
        contributors_per_partition: 1,
    }
}

/// The blocked split both parallel strategies declare at four contributors.
fn parallel_split() -> ContributorPartition {
    ContributorPartition {
        partitions: 2,
        contributors_per_partition: 2,
    }
}

/// The grouping the single-workgroup tree declares at [`SEPARATING_COLUMNS`].
///
/// Six participants folding two contributors each: `capped_tree_partition` takes
/// the admissible count nearest 256, and at twelve contributors nothing above the
/// cap is admissible — a width past 256 needs at least `2 * 257` contributors to
/// leave two per partition — so the rule is its downward walk alone, from
/// `min(256, 12 / 2) = 6`, and six divides twelve. Written out rather than
/// computed, because a helper that re-ran the compiler's rule would agree with it
/// by construction — the point of the cases below is that the *plan* declares
/// this, and they read it from the plan.
fn separating_tree_partition() -> ContributorPartition {
    ContributorPartition {
        partitions: 6,
        contributors_per_partition: 2,
    }
}

/// The grouping the multi-pass split declares at [`SEPARATING_COLUMNS`].
///
/// Four partitions of three: `governed_partition` walks down from
/// `isqrt(12) = 3` and three divides twelve. Written out for the reason
/// [`separating_tree_partition`] is.
fn separating_split_partition() -> ContributorPartition {
    ContributorPartition {
        partitions: 4,
        contributors_per_partition: 3,
    }
}

/// The corruption census one operand set survives under one declared grouping.
///
/// The population is every single-contributor corruption of the declared
/// grouping — each slot dropped, replaced by the reduction's own identity, and
/// each slot taking each *other* slot's value — and the count returned beside it
/// is how many of them the declared-grouping oracle fails to notice. A set that
/// leaves corruptions undetected is not a contributor-set claim, which is the
/// whole reason the operand sets come in pairs.
fn corruption_census(
    operands: &[u32],
    rows: u64,
    columns: u64,
    declared: ContributorPartition,
) -> (usize, usize) {
    let correct = partitioned_reference(operands, rows, columns, declared)
        .expect("the declared grouping is evaluable");
    let mut population = 0_usize;
    let mut escaped = 0_usize;
    for slot in 0..operands.len() {
        for source in 0..=operands.len() {
            let mut corrupt = operands.to_vec();
            // The last source is the dropped case: the contributor is replaced
            // by the reduction's own identity element.
            corrupt[slot] = if source == operands.len() {
                0.0_f32.to_bits()
            } else if source == slot {
                continue;
            } else {
                operands[source]
            };
            population += 1;
            let observed = partitioned_reference(&corrupt, rows, columns, declared)
                .expect("a corrupted operand set is still evaluable");
            if declared_grouping_admits(&correct, &observed) {
                escaped += 1;
            }
        }
    }
    (population, escaped)
}

/// The direct path's operands reach every value class the contract is about.
///
/// Named and counted rather than asserted as a length: an operand set that
/// silently lost its NaN would still be twelve elements long if something else
/// grew, and every comparison below would keep passing while measuring less.
#[test]
fn the_direct_operands_reach_every_class_the_contract_is_about() {
    let bits = input_bits(ROWS, COLUMNS);
    assert_eq!(
        bits.len(),
        usize::try_from(ROWS * COLUMNS).expect("a bounded operand count"),
    );
    assert_eq!(
        bits,
        ROW_PATTERNS.concat(),
        "the four-by-three shape is one full cycle"
    );

    assert!(
        bits.contains(&0x8000_0000),
        "a negative zero must be reduced"
    );
    assert!(
        bits.contains(&0x0000_0001),
        "the least positive subnormal must be reduced",
    );
    assert!(
        bits.contains(&0x7fc0_1234),
        "a non-canonical NaN payload must be reduced",
    );
    assert!(bits.contains(&0x7f80_0000), "an infinity must be reduced");

    // The interesting operand leads each row, so a narrower reduction keeps
    // every one of them.
    for pattern in ROW_PATTERNS {
        assert!(
            bits.contains(&pattern[0]),
            "{:#010x} leads a row and must survive any column count",
            pattern[0],
        );
    }
}

/// A wrongly derived element width changes what the kernel reads, without a
/// device.
///
/// The host-visible half of the composition perturbation. The kernel addresses
/// its buffer at `f32`'s own four-byte stride; a host that packed at eight bytes
/// hands it a completely different operand sequence, and every layer-local test
/// would still pass because each of them uses one width on both sides.
#[test]
fn a_wrongly_derived_operand_width_changes_what_the_kernel_reads() {
    let operands = input_bits(ROWS, COLUMNS);

    let honest = pack_f32(&operands, 4);
    assert_eq!(honest.len(), operands.len() * 4);
    assert_eq!(unpack_f32(&honest, 4, operands.len()), operands);

    // Packed at a doubled width and addressed at the declared one — the
    // asymmetry a single mis-derived site produces — every other element the
    // kernel reads is a zero the operand set never contained.
    let perturbed = pack_f32(&operands, 8);
    let seen = unpack_f32(&perturbed, 4, operands.len());
    assert_ne!(
        seen, operands,
        "an eight-byte packing must not present the same operands at a four-byte stride",
    );
    assert_eq!(seen[0], operands[0]);
    assert_eq!(seen[1], 0x0000_0000, "the high half of the first slot");
    assert_eq!(seen[2], operands[1]);

    // The symmetric error is the one a layer-local test cannot catch, and
    // stating it is why the perturbation is asymmetric.
    assert_eq!(
        unpack_f32(&perturbed, 8, operands.len()),
        operands,
        "a width error applied to both sides cancels, which is the whole reason this run \
         perturbs one side",
    );
}

/// The operand pair covers what each half alone cannot, counted on both sides.
///
/// **The numbers are the point, and they are the reason two operand sets run
/// rather than one.** Over four contributors there are five order-preserving
/// groupings. On [`PARALLEL_OPERANDS`] all five produce one value, so a
/// comparison against the serial fold has *nothing* it could refuse among legal
/// answers — it cannot observe rounding. On [`GROUPING_SENSITIVE_OPERANDS`] they
/// produce two, so the declared-grouping oracle has a wrong-but-permitted answer
/// to refuse.
///
/// The converse count is asserted too, because the sensitive set is weaker where
/// the exact one is strong: of the sixteen single-contributor corruptions of the
/// declared grouping, the exact set leaves none undetected and the sensitive set
/// leaves one. Neither half is a replacement for the other, and a later edit that
/// dropped one would have to change these numbers to do it.
#[test]
fn the_operand_pair_covers_what_each_half_alone_cannot() {
    let exact = ordered_associations(&PARALLEL_OPERANDS);
    let sensitive = ordered_associations(&GROUPING_SENSITIVE_OPERANDS);
    assert_eq!(exact.len(), 5, "four contributors admit five orderings");
    assert_eq!(sensitive.len(), 5, "four contributors admit five orderings");

    let distinct = |mut values: Vec<u32>| {
        values.sort_unstable();
        values.dedup();
        values
    };
    assert_eq!(
        distinct(exact),
        vec![0x4170_0000],
        "every grouping of the exact operands is the same f32, so nothing legal is refusable",
    );
    assert_eq!(
        distinct(sensitive),
        vec![0x3f80_0000, 0x3f80_0001],
        "the sensitive operands must separate the declared groupings by exactly one rounding step",
    );

    // The corruption counts, over the population this states: each slot dropped,
    // and each slot taking another slot's value. That is the failure a partition
    // boundary off by one or an unsynchronized staged read produces, and it is
    // the property the exact set holds and the sensitive one does not.
    let escaped = |operands: [u32; 4]| {
        corruption_census(&operands, PARALLEL_ROWS, PARALLEL_COLUMNS, parallel_split())
    };
    assert_eq!(
        escaped(PARALLEL_OPERANDS),
        (16, 0),
        "the exact operands must leave no contributor corruption undetected",
    );
    assert_eq!(
        escaped(GROUPING_SENSITIVE_OPERANDS),
        (16, 1),
        "the sensitive operands leave exactly one corruption undetected, which is why the exact \
         set still runs",
    );
}

/// The separating shape's operand pair covers what each half alone cannot, and
/// the gap is far wider than at four contributors.
///
/// **This is the answer to the padding caveat, counted rather than argued.**
/// [`SEPARATING_OPERANDS`] is [`GROUPING_SENSITIVE_OPERANDS`] padded with eight
/// `+0.0`, and padding with the reduction's own identity destroys the
/// dropped-contributor detection [`PARALLEL_OPERANDS`] carries: of the 144
/// single-contributor corruptions it leaves 81 undetected under the tree's
/// declared grouping and 98 under the split's. What it buys instead is the only
/// thing four contributors cannot give — two *parallel* groupings that disagree.
///
/// [`SEPARATING_EXACT_OPERANDS`] is the genuine twelve-wide set that restores
/// the other half: twelve distinct powers of two, every grouping exact, every
/// subset sum distinct, and no corruption undetected under either declared
/// grouping. Neither set is a replacement for the other and both are dispatched,
/// which is the same discipline the four-contributor shape runs under.
#[test]
fn the_separating_operand_pair_covers_what_each_half_alone_cannot() {
    let padded = ordered_associations(&SEPARATING_OPERANDS);
    let exact = ordered_associations(&SEPARATING_EXACT_OPERANDS);
    assert_eq!(
        padded.len(),
        58_786,
        "twelve contributors admit the eleventh Catalan number of orderings",
    );
    assert_eq!(exact.len(), 58_786);

    let distinct = |mut values: Vec<u32>| {
        values.sort_unstable();
        values.dedup();
        values
    };
    assert_eq!(
        distinct(padded),
        vec![0x3f80_0000, 0x3f80_0001],
        "the padded operands must separate the declared groupings by exactly one rounding step",
    );
    assert_eq!(
        distinct(exact),
        vec![0x457f_f000],
        "every grouping of twelve distinct powers of two is 4095.0, so nothing legal is refusable",
    );

    // Counted under *both* declared groupings, because at this contributor count
    // the tree and the split declare different ones and a census taken under one
    // of them would say nothing about the other.
    for (declared, label, padded_escapes) in [
        (separating_tree_partition(), "tree", 81),
        (separating_split_partition(), "split", 98),
    ] {
        assert_eq!(
            corruption_census(
                &SEPARATING_EXACT_OPERANDS,
                SEPARATING_ROWS,
                SEPARATING_COLUMNS,
                declared,
            ),
            (144, 0),
            "the exact twelve-wide operands must leave no contributor corruption undetected under \
             the {label}'s grouping",
        );
        assert_eq!(
            corruption_census(
                &SEPARATING_OPERANDS,
                SEPARATING_ROWS,
                SEPARATING_COLUMNS,
                declared,
            ),
            (144, padded_escapes),
            "padding with the reduction's identity leaves {padded_escapes} of 144 corruptions \
             undetected under the {label}'s grouping, which is exactly why the exact set runs \
             beside it",
        );
    }
}

/// The grouping oracle refuses the split's answer when the tree declared its
/// own, and the refused value is legal under the contract.
///
/// **This is the discriminating refusal, established without a device.** At four
/// contributors the only refusable value is the *serial fold's*, so what the
/// existing case separates is the parallel strategies from the fold. Here the
/// tree declares six partitions of two and the split four of three, those two
/// blocked regroupings return different `f32` values, and each is an
/// order-preserving regrouping the contract permits. Holding the tree to the
/// split's partition therefore refuses a permitted value — the wrong-but-in-range
/// refusal, which is the whole of what this shape adds.
///
/// The split's answer coincides with the serial fold's at this shape, and that
/// is asserted rather than left implicit: it means the fold-versus-parallel
/// separation is carried by the tree alone here, and a reader comparing the two
/// shapes should not have to derive that.
#[test]
fn the_grouping_oracle_refuses_the_answer_the_other_parallel_strategy_declared() {
    let tree = partitioned_reference(
        &SEPARATING_OPERANDS,
        SEPARATING_ROWS,
        SEPARATING_COLUMNS,
        separating_tree_partition(),
    )
    .expect("the tree's declared grouping is evaluable");
    let split = partitioned_reference(
        &SEPARATING_OPERANDS,
        SEPARATING_ROWS,
        SEPARATING_COLUMNS,
        separating_split_partition(),
    )
    .expect("the split's declared grouping is evaluable");
    let serial = partitioned_reference(
        &SEPARATING_OPERANDS,
        SEPARATING_ROWS,
        SEPARATING_COLUMNS,
        serial_order(SEPARATING_COLUMNS),
    )
    .expect("the degenerate partition is the declared serial order");

    assert_eq!(tree, vec![0x3f80_0001]);
    assert_eq!(split, vec![0x3f80_0000]);
    assert_eq!(
        serial, split,
        "at this shape the split rounds the way the serial fold does, so the tree is what carries \
         the separation from the fold as well",
    );

    // The refusal, and its mirror, by the same function that admits a correct
    // answer.
    assert!(
        declared_grouping_admits(&tree, &tree),
        "an oracle that refused the answer its own declared grouping produces would refuse every \
         correct strategy",
    );
    assert!(
        !declared_grouping_admits(&tree, &split),
        "the tree's oracle must refuse the split's answer, which is what separates the two \
         parallel strategies",
    );
    assert!(
        !declared_grouping_admits(&split, &tree),
        "and the split's oracle must refuse the tree's, so neither direction is an accident",
    );

    // What makes the refusal non-vacuous: the refused value is one this contract
    // permits, so a tolerance or a permitted-set membership test would have
    // admitted it.
    let permitted = ordered_associations(&SEPARATING_OPERANDS);
    for (value, label) in [(split[0], "split"), (tree[0], "tree")] {
        assert!(
            permitted.contains(&value),
            "the {label}'s answer {value:#010x} must be an order-preserving regrouping this \
             contract permits, or the refusal is of an illegal value and proves nothing",
        );
    }
}

/// The grouping oracle refuses a legal regrouping the strategy did not declare.
///
/// **This is the refusal the measured run below is watched making, established
/// first without a device.** The value refused is not garbage and not out of
/// tolerance: it is the serial fold's answer, which a reassociation-permitting
/// contract fully authorizes and which any bounded-error oracle would accept.
/// What makes it wrong is only that the strategy under test published a different
/// grouping, and that is the whole distinction a tolerance cannot draw.
///
/// Both directions are asserted, so neither reading is an accident of which
/// grouping happens to round up.
#[test]
fn the_grouping_oracle_refuses_a_legal_grouping_the_strategy_did_not_declare() {
    let parallel = partitioned_reference(
        &GROUPING_SENSITIVE_OPERANDS,
        PARALLEL_ROWS,
        PARALLEL_COLUMNS,
        parallel_split(),
    )
    .expect("the declared parallel split is evaluable");
    let serial = partitioned_reference(
        &GROUPING_SENSITIVE_OPERANDS,
        PARALLEL_ROWS,
        PARALLEL_COLUMNS,
        serial_order(PARALLEL_COLUMNS),
    )
    .expect("the degenerate partition is the declared serial order");

    assert_eq!(parallel, vec![0x3f80_0001]);
    assert_eq!(serial, vec![0x3f80_0000]);
    assert!(
        declared_grouping_admits(&parallel, &parallel),
        "an oracle that refused the answer its own declared grouping produces would refuse every \
         correct strategy",
    );
    assert!(
        !declared_grouping_admits(&parallel, &serial),
        "the parallel oracle must refuse the serial fold's answer, which is legal under this \
         contract and is not what the parallel strategies declared",
    );
    assert!(
        !declared_grouping_admits(&serial, &parallel),
        "and the serial oracle must refuse the parallel answer, so neither direction is an \
         accident",
    );

    // The same refusal is unreachable on the exact operands, which is the
    // measured statement of why they cannot carry this claim.
    let exact_parallel = partitioned_reference(
        &PARALLEL_OPERANDS,
        PARALLEL_ROWS,
        PARALLEL_COLUMNS,
        parallel_split(),
    )
    .expect("the declared parallel split is evaluable");
    let exact_serial = partitioned_reference(
        &PARALLEL_OPERANDS,
        PARALLEL_ROWS,
        PARALLEL_COLUMNS,
        serial_order(PARALLEL_COLUMNS),
    )
    .expect("the degenerate partition is the declared serial order");
    assert!(
        declared_grouping_admits(&exact_parallel, &exact_serial),
        "on the exact operands the two groupings agree, so no refusal exists to watch",
    );
}

/// A partition that does not cover the contributor sequence is refused rather
/// than rounded into one that does.
#[test]
fn a_partition_that_covers_nothing_is_refused_by_the_reference() {
    // Three partitions of two cover six, and this row has four. Both strategies
    // decline an inexact split rather than padding it, so the oracle must
    // decline to answer for one too.
    let refusal = partitioned_reference(
        &GROUPING_SENSITIVE_OPERANDS,
        PARALLEL_ROWS,
        PARALLEL_COLUMNS,
        ContributorPartition {
            partitions: 3,
            contributors_per_partition: 2,
        },
    )
    .expect_err("a split that does not cover the contributors has no exact value");
    assert!(
        matches!(refusal, super::GroupingRefusal::UndeclaredGrouping { .. }),
        "an inexact split must be refused as an undeclarable grouping: {refusal}",
    );
}

/// The oracle at the declared serial order is the reference evaluator's own
/// answer.
///
/// **The calibration, and it needs no device.** The degenerate partition — one
/// contributor each, combined in ascending order — *is* the declared serial
/// order, so the partitioned oracle at it must agree with the reference
/// evaluator's run of the whole program. Agreement establishes two things at
/// once: that this file asks the oracle about the order the program declares, and
/// that the pointwise prologue `x * 1.0 + 0.0` is bit-identity on these operands,
/// which is what lets every partition below be evaluated over the operands rather
/// than over the prologue's output.
#[test]
fn the_partitioned_oracle_is_calibrated_against_the_whole_program() {
    let program = serial_sum_program(PARALLEL_ROWS, PARALLEL_COLUMNS);
    for operands in [PARALLEL_OPERANDS, GROUPING_SENSITIVE_OPERANDS] {
        let evaluator = reference_bits(&program, &operands, PARALLEL_ROWS, PARALLEL_COLUMNS);
        let partitioned = partitioned_reference(
            &operands,
            PARALLEL_ROWS,
            PARALLEL_COLUMNS,
            serial_order(PARALLEL_COLUMNS),
        )
        .expect("the degenerate partition is evaluable");
        assert_eq!(
            evaluator, partitioned,
            "either the orders disagree or the pointwise prologue is not bit-identity on \
             {operands:08x?}, and the reduction oracle may not be applied to them",
        );
    }
}

/// The reassociating contract retains both parallel strategies and the serial
/// fold, and each publishes a covering partition.
///
/// **Device-free, and it is the half that would otherwise only ever be checked on
/// hardware.** A run that found only the fold would mean the contract or the
/// profile stopped reaching the parallel strategies, and a run that found the
/// strategies but not the fold would mean they replaced rather than joined it.
/// Both are compiler-and-profile facts, so both are decidable here — and the
/// measured runs below then only have to establish that each of them *executes*.
#[test]
fn the_reassociating_contract_retains_both_strategies_and_the_fold() {
    let declaration = declaration().expect("the authoritative declaration assembles");
    let program = serial_sum_program(PARALLEL_ROWS, PARALLEL_COLUMNS);
    let compilation = compile_under(
        &declaration,
        &program,
        NumericalContract::FLUSH_AND_REASSOCIATE_F32,
    )
    .expect("a flush-and-reassociate contract compiles this program");

    let mut seen = Vec::new();
    let mut folds = 0_usize;
    let mut counted = 0_usize;
    for alternative in compilation.alternatives() {
        let strategy =
            super::classify_strategy(alternative).expect("every launch quantity is a literal");
        let partition = declared_partition(alternative, strategy, PARALLEL_COLUMNS)
            .expect("every retained alternative publishes a covering partition");
        assert!(
            partition.covers(PARALLEL_COLUMNS),
            "{} publishes {partition:?}, which does not cover {PARALLEL_COLUMNS} contributor(s)",
            super::strategy_label(strategy),
        );
        match strategy {
            Some(strategy) => seen.push(strategy),
            None => folds += 1,
        }
        counted += 1;
    }
    assert_eq!(
        counted,
        compilation.alternatives().len(),
        "every retained alternative was classified, not the ones that happened to parse",
    );
    for strategy in [
        ParallelStrategy::MultiPassSplit,
        ParallelStrategy::SingleWorkgroupTree,
    ] {
        assert!(
            seen.contains(&strategy),
            "the portfolio retained {counted} alternative(s) and none of them is the {strategy}",
        );
    }
    assert!(
        folds > 0,
        "the portfolio retained {counted} alternative(s) and the serial fold is not among them; a \
         portfolio that dropped the fold has become narrower under a contract that only widens \
         permissions",
    );

    // At four contributors both parallel rules return two partitions of two, and
    // that equality is a property of *this extent* rather than of the strategies
    // — they first diverge at twelve contributors, but not at every admitting
    // count thereafter. Stating it here is what
    // keeps the grouping-sensitive derivation attributable.
    for alternative in compilation.alternatives() {
        let strategy = super::classify_strategy(alternative).expect("a literal launch");
        if strategy.is_some() {
            assert_eq!(
                declared_partition(alternative, strategy, PARALLEL_COLUMNS)
                    .expect("a covering partition"),
                parallel_split(),
                "at {PARALLEL_COLUMNS} contributors both parallel rules declare the same split",
            );
        }
    }
}

/// At the separating count the two parallel strategies publish *different*
/// groupings, read from each plan's own launch geometry.
///
/// **Device-free, and it is the fact the whole separating case rests on.** The
/// four-contributor case asserts the two publish the *same* partition; this one
/// asserts they do not, at the smallest count where the rules disagree. Both
/// numbers are read from the compiled plan's published geometry — the tree's
/// declared workgroup width and the split's partial-to-final launch ratio —
/// rather than from the partition functions, so a call site that reverted to one
/// shared rule fails here even though both rules would still return a legal
/// partition.
///
/// The serial fold is asserted beside them for the reason the four-contributor
/// case asserts it: a portfolio that dropped the fold has become narrower under
/// a contract that only widens permissions.
#[test]
fn the_two_parallel_strategies_publish_different_groupings_at_the_separating_count() {
    let declaration = declaration().expect("the authoritative declaration assembles");
    let program = serial_sum_program(SEPARATING_ROWS, SEPARATING_COLUMNS);
    let compilation = compile_under(
        &declaration,
        &program,
        NumericalContract::FLUSH_AND_REASSOCIATE_F32,
    )
    .expect("a flush-and-reassociate contract compiles this program");

    let mut published = Vec::new();
    let mut folds = 0_usize;
    let mut counted = 0_usize;
    for alternative in compilation.alternatives() {
        let strategy =
            super::classify_strategy(alternative).expect("every launch quantity is a literal");
        let partition = declared_partition(alternative, strategy, SEPARATING_COLUMNS)
            .expect("every retained alternative publishes a covering partition");
        assert!(
            partition.covers(SEPARATING_COLUMNS),
            "{} publishes {partition:?}, which does not cover {SEPARATING_COLUMNS} contributor(s)",
            super::strategy_label(strategy),
        );
        match strategy {
            Some(strategy) => published.push((strategy, partition)),
            None => folds += 1,
        }
        counted += 1;
    }
    assert_eq!(
        counted,
        compilation.alternatives().len(),
        "every retained alternative was classified, not the ones that happened to parse",
    );
    assert!(
        folds > 0,
        "the portfolio retained {counted} alternative(s) and the serial fold is not among them",
    );

    for (strategy, expected) in [
        (
            ParallelStrategy::SingleWorkgroupTree,
            separating_tree_partition(),
        ),
        (
            ParallelStrategy::MultiPassSplit,
            separating_split_partition(),
        ),
    ] {
        let observed = published
            .iter()
            .find_map(|(candidate, partition)| (*candidate == strategy).then_some(*partition))
            .unwrap_or_else(|| {
                panic!(
                    "the portfolio retained {counted} alternative(s) and none of them is the \
                     {strategy}"
                )
            });
        assert_eq!(
            observed, expected,
            "the {strategy} published {observed:?} at {SEPARATING_COLUMNS} contributor(s)",
        );
    }
    assert_ne!(
        separating_tree_partition(),
        separating_split_partition(),
        "a separating count at which the two declared partitions are equal separates nothing",
    );
}

/// A flush-only contract grants no regrouping, so it retains no parallel
/// strategy.
///
/// The neighbour that makes the case above about the *contract* rather than
/// about the profile: the same program, the same declaration, and only the
/// permission changed.
#[test]
fn a_flush_only_contract_retains_no_parallel_strategy() {
    let declaration = declaration().expect("the authoritative declaration assembles");
    let program = serial_sum_program(PARALLEL_ROWS, PARALLEL_COLUMNS);
    let compilation = compile_under(
        &declaration,
        &program,
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    )
    .expect("a flush-only contract compiles this program");
    for alternative in compilation.alternatives() {
        assert_eq!(
            super::classify_strategy(alternative).expect("every launch quantity is a literal"),
            None,
            "a contract granting no regrouping must retain no parallel strategy",
        );
    }
}

/// The widest canonical kernel identity the selected plan carries at one shape.
///
/// Returned as bytes rather than a length because the crossing case also has to
/// reach the artifact layer's own constructor with the value it measured, and a
/// length cannot. The widest is the right reduction over a multi-stage plan: an
/// artifact carries *each* entry's identity as its own `BackendEntryKey`, so the
/// bound is crossed as soon as any one of them crosses it.
fn widest_kernel_identity(declaration: &BoundMetalCompileDeclaration, columns: u64) -> Vec<u8> {
    let program = serial_sum_program(ROWS, columns);
    let compilation = compile_under(
        declaration,
        &program,
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    )
    .unwrap_or_else(|cause| {
        panic!("a flush-only contract must compile a {ROWS}-by-{columns} serial sum: {cause}")
    });
    let selected = compilation
        .selected()
        .expect("the portfolio retained a selected plan");
    let kernels = selected.kernels();
    let widest = kernels
        .iter()
        .map(|kernel| kernel.canonical_identity().as_bytes())
        .max_by_key(|identity| identity.len())
        .expect("a selected plan with no kernel has no identity to bound")
        .to_vec();
    eprintln!(
        "serial sum identity: [{ROWS}, {columns}] reducing axis 1 — {} kernel(s), widest canonical \
         identity {} byte(s), against MAX_OPAQUE_IDENTITY_BYTES {MAX_OPAQUE_IDENTITY_BYTES}",
        kernels.len(),
        widest.len(),
    );
    widest
}

/// A real reduction's canonical kernel identity crosses the artifact layer's
/// shared opaque-identity bound at the *second* contributor.
///
/// **The two-sided inequality is the entire argument `BackendEntryKey`'s bound
/// was moved on**, and it is asserted here because nothing asserted it anywhere.
/// Reducing one contributor the identity fits under
/// [`MAX_OPAQUE_IDENTITY_BYTES`]; reducing two it does not. So the shared bound
/// admitted exactly the degenerate reduction and refused every real one, which
/// is why `BackendEntryKey` now takes `tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES`
/// — the bound of the authority that mints the value. Either half alone is
/// satisfied by an identity that stopped growing with the program, so both
/// directions are asserted.
///
/// **No length is pinned, deliberately.** The identity's constant offset moves
/// whenever its encoding steps: the two-contributor case measured 1,121 bytes on
/// 2026-07-25 and 1,309 on 2026-08-08, both dated in
/// [Artifact ABI](../../../../docs/artifact-abi.md)'s "Governed budgets" table
/// together with the construction that regenerates them. A literal here would
/// decay into a claim about a tree that has moved — which is exactly how
/// `an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it` in
/// `crates/tiler-artifact/src/program/tests.rs` came to call a fabricated vector
/// measured. The *crossing* is what the bound rests on, and it survives the
/// offset moving.
///
/// **Here rather than beside the constant**, and the reason is structural.
/// `crates/tiler-artifact/Cargo.toml` carries no `tiler-compiler` edge by
/// design: `tiler-runtime`'s `the_consumer_links_no_compiler_emitter_or_build_provider`
/// walks `Cargo.lock`, which merges normal and development edges per package, so
/// even a development edge would put the compiler into the consumer's closure and
/// breach ADR 0081 item 2. The crate that owns the bound can therefore never
/// compile a real reduction to compare against it; this crate already depends on
/// both sides.
///
/// **The route is stated because two exist and they differ in reachability.**
/// This uses [`compile_under`] — `tiler_compiler::session::compile` against
/// `BoundMetalCompileDeclaration::first_macos_apple9()` — and not
/// `compile_governed`, which the 2026-07-25 sweep used and which at this tree
/// refuses several of that sweep's shapes as `NoFeasiblePlan` before a plan
/// composes. Both admit the two shapes here, and where both admit a shape they
/// agree on the identity exactly.
#[test]
fn the_serial_sum_identity_crosses_the_shared_opaque_bound_at_the_second_contributor() {
    let declaration = declaration().expect("the authoritative declaration assembles");

    let degenerate = widest_kernel_identity(&declaration, 1);
    assert!(
        degenerate.len() < MAX_OPAQUE_IDENTITY_BYTES,
        "the one-contributor reduction's canonical kernel identity is {} byte(s) and the shared \
         bound is {MAX_OPAQUE_IDENTITY_BYTES}; the bound was moved on this being the one case it \
         did admit, so a degenerate reduction above it makes that argument false",
        degenerate.len(),
    );

    let real = widest_kernel_identity(&declaration, 2);
    assert!(
        real.len() > MAX_OPAQUE_IDENTITY_BYTES,
        "the two-contributor reduction's canonical kernel identity is {} byte(s) and the shared \
         bound is {MAX_OPAQUE_IDENTITY_BYTES}, which admits it; the bound was moved on it \
         refusing every reduction of two or more contributors",
        real.len(),
    );

    // The other half of the reconciliation, and the statement the artifact
    // crate's fabricated vector was standing in for: the bound that *does* apply
    // to this subject admits the value its authority actually minted. It says no
    // for an empty identity or one past `MAX_KERNEL_IDENTITY_BYTES`, neither of
    // which this shape reaches — so the two inequalities above are what carry
    // this case, and this is the constructor they are about.
    BackendEntryKey::from_bytes(&real)
        .expect("a real reduction's kernel identity is a legal backend entry key");

    eprintln!(
        "serial sum identity: {} byte(s) at one contributor and {} at two, crossing the shared \
         {MAX_OPAQUE_IDENTITY_BYTES}-byte bound between them",
        degenerate.len(),
        real.len(),
    );
}

/// This host does not earn the right to offer the declared profile, and the
/// refusal is stated before any dispatch.
///
/// **The only question in this crate whose answer would be an authority claim,
/// and it refuses.** ADR 0086 decides that native device translation of a
/// metallib during pipeline creation is a typed capability fact whose authority
/// is `Unknown` on every macOS row currently observable, so the refusal is
/// structural rather than a property of this machine — which is exactly why it
/// does not gate the runs below. Everything measured here routes on
/// producer-declared equality, NOT host-earned eligibility.
///
/// The matching-row authority refusal is proven independently of this machine
/// by `applicability::tests::the_composed_observation_answers_every_predicate`.
/// This live-host case must not assume the coordination host remains
/// `FIRST_MACOS_APPLE9`: a newer OS build is an outside-measured-row refusal,
/// not a reason to widen the pin. Expected predicate is derived from this
/// host's ambient fields plus the policy's device row, then compared to the
/// offer path that observed the real device.
#[test]
fn this_host_is_refused_the_right_to_offer_the_declared_profile() {
    // The refusal is unconditional, so it is stated before a device is even
    // asked for: a host that cannot observe its device refuses earlier, on an
    // unanswered predicate, and never reaches a yes.
    let unobserved = refuse_to_offer_the_declared_profile(&observe_host_environment());
    assert_ne!(
        unobserved.predicate(),
        MetalHostPredicate::NativeTranslationAuthority,
        "an observation missing the device predicates must refuse on one of them",
    );

    let Some((observation, refusal)) =
        require_or_report("host applicability offer", measured_offer())
    else {
        return;
    };
    eprintln!("host applicability observation: {observation}");
    eprintln!(
        "  REFUSED before any dispatch: {refusal}\n  predicate {}, rule {}",
        refusal.predicate(),
        refusal.rule(),
    );
    // A fully observed host that still *is* the measured row reaches ADR 0086.
    // A fully observed host that has left that row must stop at the first
    // mismatched ambient predicate. Reconstructing the expected refusal from
    // this host's `sw_vers` fields plus the policy's device row keeps the pin
    // honest: widening `FIRST_MACOS_APPLE9` is a new measurement, not a test
    // update. The offer path observed the real device; they agree before the
    // device predicates whenever an ambient field already mismatches.
    let policy = MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9;
    let expected = refuse_to_offer_the_declared_profile(
        &observe_host_environment()
            .observing_device_name(policy.device_name())
            .observing_gpu_family(MetalGpuFamilySupport::Highest(policy.gpu_family())),
    );
    assert_eq!(
        refusal.predicate(),
        expected.predicate(),
        "an observed host must refuse the first mismatched measured-row predicate, \
         or ADR 0086 when the ambient row still matches (live os-build {:?}, pin {})",
        observe_host_environment().os_build(),
        policy.os_build(),
    );
    let declaration = declaration().expect("the authoritative declaration assembles");
    eprintln!(
        "  consequence: this host does not offer {}. Every measured run in this module is \
         producer-declared equality, NOT host-earned eligibility.",
        declaration.profile().profile_key(),
    );
}

/// The direct path agrees with the oracle, element for element, on the row that
/// ran.
///
/// The device executes the selected alternative over operands that include a
/// negative zero, the least positive subnormal, a non-canonical NaN payload, and
/// an infinity, and the result is compared against the oracle's evaluation of the
/// *semantic* program — two independent implementations of one declared contract,
/// with no shared host expression between them.
#[test]
fn the_direct_path_agrees_with_the_oracle_on_the_measured_row() {
    let program = serial_sum_program(ROWS, COLUMNS);
    let operands = input_bits(ROWS, COLUMNS);
    let expected = reference_bits(&program, &operands, ROWS, COLUMNS);
    assert_eq!(
        expected.len(),
        usize::try_from(ROWS).expect("a bounded row count"),
        "reducing the inner axis publishes one element per row",
    );

    let Some(observed) = require_or_report("serial sum direct", measured_direct()) else {
        return;
    };
    assert_eq!(
        observed.len(),
        expected.len(),
        "the device returned {} element(s) for a {}-row reduction",
        observed.len(),
        expected.len(),
    );
    assert_eq!(
        observed, expected,
        "the direct path returned {observed:08x?} and the reference requires {expected:08x?}",
    );
    eprintln!(
        "serial sum direct: bit-for-bit agreement on {} element(s)",
        expected.len(),
    );
}

/// Every retained alternative computes the declared contributor set, on operands
/// whose every grouping is exact.
///
/// **The claim the compiling cooperative golden cannot make.** A golden
/// establishes that a cooperative kernel *compiles*; it says nothing about
/// whether the barrier synchronizes, whether the threadgroup allocation is
/// reachable, or whether the tree computes the declared sum. Each strategy is
/// emitted, linked, dispatched, and compared bit for bit against the reference's
/// independent evaluation of the same semantic program.
///
/// The classification is checked here as well as at compile time, because the
/// point of dispatching all three is that they ran *differently*: a "tree" that
/// launched one thread per workgroup and reserved no threadgroup memory would be
/// a misclassification, and the reported quantities are what make that visible
/// instead of plausible.
#[test]
fn every_retained_alternative_computes_the_declared_contributor_set() {
    let program = serial_sum_program(PARALLEL_ROWS, PARALLEL_COLUMNS);
    let expected = reference_bits(
        &program,
        &PARALLEL_OPERANDS,
        PARALLEL_ROWS,
        PARALLEL_COLUMNS,
    );

    let Some(runs) = require_or_report(
        "serial sum parallel strategies",
        measured_portfolio(&PARALLEL_OPERANDS, PARALLEL_ROWS, PARALLEL_COLUMNS),
    ) else {
        return;
    };

    let mut seen = Vec::new();
    let mut folds = 0_usize;
    for run in &runs {
        eprintln!(
            "  {} ({}): {} encoder(s) in order, widest workgroup {}, {} byte(s) of threadgroup \
             memory reserved, {:08x?} against {expected:08x?}",
            run.label(),
            run.stable_id,
            run.encoders,
            run.widest_workgroup,
            run.threadgroup_bytes,
            run.bits,
        );
        assert_eq!(
            run.bits,
            expected,
            "the {} returned {:08x?} and the reference requires {expected:08x?}",
            run.label(),
            run.bits,
        );
        match run.strategy {
            Some(ParallelStrategy::SingleWorkgroupTree) => {
                assert!(
                    run.widest_workgroup > 1,
                    "a tree that launched one thread per workgroup is a misclassification",
                );
                seen.push(ParallelStrategy::SingleWorkgroupTree);
            }
            Some(ParallelStrategy::MultiPassSplit) => {
                assert_eq!(
                    run.encoders, 3,
                    "a split dispatches its map, partial, and combine stages as three encoders",
                );
                seen.push(ParallelStrategy::MultiPassSplit);
            }
            None => folds += 1,
        }
    }
    for strategy in [
        ParallelStrategy::MultiPassSplit,
        ParallelStrategy::SingleWorkgroupTree,
    ] {
        assert!(
            seen.contains(&strategy),
            "{} alternative(s) were dispatched and none of them is the {strategy}",
            runs.len(),
        );
    }
    assert!(folds > 0, "the serial fold was not dispatched beside them");
    eprintln!(
        "serial sum parallel strategies: both strategies and the serial fold agree bit for bit \
         with the reference on {} element(s)",
        expected.len(),
    );
}

/// Every retained alternative rounds the way the grouping it published rounds.
///
/// **The device observation in which *both* parallel strategies diverge from the
/// serial fold.** The case above runs operands every grouping of which is exact,
/// so its refusal population among legal groupings is empty: no answer a
/// reassociating contract permits would have failed it. This one runs the same
/// three alternatives on operands where the declared regroupings genuinely
/// disagree, so the oracle changes shape — from the serial fold to the exact
/// value the strategy's *own declared grouping* produces, read off the plan
/// rather than assumed.
///
/// It was the corpus's *only* such observation until
/// [`the_tree_and_the_split_round_differently_at_the_separating_count`] landed,
/// and the two are not interchangeable: at four contributors both parallel
/// strategies declare one partition, so this case separates them from the fold
/// and not from each other; at twelve they declare two, the split's answer
/// coincides with the fold's, and that case separates the tree from the split
/// alone. Neither shape subsumes the other.
///
/// A serial fold would be wrong here because disagreement with it is the
/// *expected* outcome for a legally regrouped strategy: it would refuse the split
/// and the tree for being right. A tolerance would be wrong because it admits
/// every value in an interval — including the *other* strategy's answer — so it
/// could not tell a strategy that grouped as it declared from one that did not.
#[test]
fn every_retained_alternative_rounds_the_way_its_declared_grouping_rounds() {
    let serial_expected = partitioned_reference(
        &GROUPING_SENSITIVE_OPERANDS,
        PARALLEL_ROWS,
        PARALLEL_COLUMNS,
        serial_order(PARALLEL_COLUMNS),
    )
    .expect("the degenerate partition is the declared serial order");

    let mut permitted = ordered_associations(&GROUPING_SENSITIVE_OPERANDS);
    permitted.sort_unstable();
    permitted.dedup();
    eprintln!(
        "  operands {GROUPING_SENSITIVE_OPERANDS:08x?}: {} distinct value(s) {permitted:08x?} over \
         {PARALLEL_COLUMNS} contributor(s); the declared serial order is {serial_expected:08x?}",
        permitted.len(),
    );

    let Some(runs) = require_or_report(
        "serial sum grouping-sensitive",
        measured_portfolio(
            &GROUPING_SENSITIVE_OPERANDS,
            PARALLEL_ROWS,
            PARALLEL_COLUMNS,
        ),
    ) else {
        return;
    };

    let mut refusals = 0_usize;
    let mut separated = 0_usize;
    for run in &runs {
        let expected = partitioned_reference(
            &GROUPING_SENSITIVE_OPERANDS,
            PARALLEL_ROWS,
            PARALLEL_COLUMNS,
            run.partition,
        )
        .expect("a published partition is evaluable");

        // The refusal population, built by asking the oracle about every value
        // this contract permits and keeping the ones it says no to. The ask *is*
        // the refusal — there is no second pass re-checking the same predicate on
        // the same values, because that pass could not fail. Empty means the
        // oracle had nothing legal to refuse on these operands, which is the
        // exact condition `PARALLEL_OPERANDS` is in and the reason this case
        // exists.
        let foreign: Vec<u32> = permitted
            .iter()
            .copied()
            .filter(|value| !declared_grouping_admits(&expected, std::slice::from_ref(value)))
            .collect();
        assert!(
            !foreign.is_empty(),
            "every order-preserving regrouping of these operands produces {permitted:08x?}, so \
             the {}'s oracle has no wrong-but-permitted answer it could refuse and observes no \
             rounding",
            run.label(),
        );

        let distinguishable = !declared_grouping_admits(&serial_expected, &expected);
        separated += usize::from(distinguishable);
        eprintln!(
            "  {} ({}): declared {} partition(s) of {} contributor(s), {} encoder(s), widest \
             workgroup {}, {} byte(s) of threadgroup memory, {:08x?} against its declared \
             grouping's {expected:08x?} — {} from the serial fold's {serial_expected:08x?}",
            run.label(),
            run.stable_id,
            run.partition.partitions,
            run.partition.contributors_per_partition,
            run.encoders,
            run.widest_workgroup,
            run.threadgroup_bytes,
            run.bits,
            if distinguishable {
                "one legal regrouping away"
            } else {
                "indistinguishable"
            },
        );
        assert!(
            declared_grouping_admits(&expected, &run.bits),
            "the {} declares {} partition(s) of {} contributor(s) and returned {:08x?}, and that \
             grouping produces {expected:08x?}",
            run.label(),
            run.partition.partitions,
            run.partition.contributors_per_partition,
            run.bits,
        );

        // Each refused value is a *legal* answer under this contract, so what the
        // count records is the oracle saying no to a wrong-but-permitted result —
        // by the same function that just admitted the device's bits.
        refusals += foreign.len();
        eprintln!(
            "    refused {} legal grouping(s) this strategy did not declare: {foreign:08x?}",
            foreign.len(),
        );
    }

    assert!(
        separated > 0,
        "no dispatched alternative's declared grouping differed from the serial fold's, so this \
         run observed no reassociation at all",
    );
    eprintln!(
        "serial sum grouping-sensitive: every alternative matched its own declared grouping bit \
         for bit, {separated} of {} of them at a value the serial fold does not produce, and \
         {refusals} wrong-but-permitted grouping(s) were refused",
        runs.len(),
    );
}

/// Every retained alternative computes the declared contributor set at the
/// separating count, on operands whose every grouping is exact.
///
/// **The contributor-set half of the separating shape's pair, and it is not
/// redundant with the four-contributor one.** The tree runs six participants
/// here where it ran two, so the barrier synchronizes six staged slots rather
/// than two and the split stages four partials rather than two. A dropped or
/// double-counted contributor at those widths is a failure the narrower shape
/// cannot reach, and twelve distinct powers of two are what make each such
/// failure land on a value no correct grouping produces.
///
/// It runs on the padded set's shape and deliberately not on the padded set: a
/// set eight of whose twelve slots are the reduction's identity leaves most of
/// that population undetected, which
/// [`the_separating_operand_pair_covers_what_each_half_alone_cannot`] counts.
#[test]
fn every_retained_alternative_computes_the_declared_contributor_set_at_the_separating_count() {
    let program = serial_sum_program(SEPARATING_ROWS, SEPARATING_COLUMNS);
    let expected = reference_bits(
        &program,
        &SEPARATING_EXACT_OPERANDS,
        SEPARATING_ROWS,
        SEPARATING_COLUMNS,
    );
    assert_eq!(
        expected,
        vec![0x457f_f000],
        "twelve distinct powers of two sum to 4095.0 under every grouping, including the declared \
         serial one",
    );

    let Some(runs) = require_or_report(
        "serial sum separating contributor set",
        measured_portfolio(
            &SEPARATING_EXACT_OPERANDS,
            SEPARATING_ROWS,
            SEPARATING_COLUMNS,
        ),
    ) else {
        return;
    };

    let mut seen = Vec::new();
    let mut folds = 0_usize;
    for run in &runs {
        eprintln!(
            "  {} ({}): declared {} partition(s) of {} contributor(s), {} encoder(s) in order, \
             widest workgroup {}, {} byte(s) of threadgroup memory reserved, {:08x?} against \
             {expected:08x?}",
            run.label(),
            run.stable_id,
            run.partition.partitions,
            run.partition.contributors_per_partition,
            run.encoders,
            run.widest_workgroup,
            run.threadgroup_bytes,
            run.bits,
        );
        assert_eq!(
            run.bits,
            expected,
            "the {} returned {:08x?} and the reference requires {expected:08x?}",
            run.label(),
            run.bits,
        );
        match run.strategy {
            Some(ParallelStrategy::SingleWorkgroupTree) => {
                assert_eq!(
                    run.partition,
                    separating_tree_partition(),
                    "the dispatched tree published {:?} rather than the capped rule's choice",
                    run.partition,
                );
                assert_eq!(
                    run.widest_workgroup, 6,
                    "the tree's declared width must follow its participant count",
                );
                seen.push(ParallelStrategy::SingleWorkgroupTree);
            }
            Some(ParallelStrategy::MultiPassSplit) => {
                assert_eq!(
                    run.partition,
                    separating_split_partition(),
                    "the dispatched split published {:?} rather than the balanced rule's choice",
                    run.partition,
                );
                assert_eq!(
                    run.encoders, 3,
                    "a split dispatches its map, partial, and combine stages as three encoders",
                );
                seen.push(ParallelStrategy::MultiPassSplit);
            }
            None => folds += 1,
        }
    }
    for strategy in [
        ParallelStrategy::MultiPassSplit,
        ParallelStrategy::SingleWorkgroupTree,
    ] {
        assert!(
            seen.contains(&strategy),
            "{} alternative(s) were dispatched and none of them is the {strategy}",
            runs.len(),
        );
    }
    assert!(folds > 0, "the serial fold was not dispatched beside them");
    eprintln!(
        "serial sum separating contributor set: all three alternatives agree bit for bit with the \
         reference at {SEPARATING_COLUMNS} contributor(s), at two different declared groupings",
    );
}

/// The tree and the split return different — and each permitted — values at the
/// separating count, and each is refused by the other's declared grouping.
///
/// **This is what the four-contributor run cannot say.** There both parallel
/// strategies declare two partitions of two, so their oracles are the same
/// oracle and the only refusable legal value is the serial fold's. Here the tree
/// declares six partitions of two and the split four of three; the device
/// returns `0x3f800001` for one and `0x3f800000` for the other, both of them
/// order-preserving regroupings this contract permits, and holding either
/// strategy to the *other's* published partition refuses a permitted value.
///
/// A tolerance would admit both. A permitted-set membership test would admit
/// both. Only an oracle asked about the grouping the plan itself published can
/// tell a strategy that grouped as it declared from one that did not, and this
/// is the first device observation in the corpus that separates the two parallel
/// strategies from each other rather than from the serial fold.
#[test]
fn the_tree_and_the_split_round_differently_at_the_separating_count() {
    let tree_expected = partitioned_reference(
        &SEPARATING_OPERANDS,
        SEPARATING_ROWS,
        SEPARATING_COLUMNS,
        separating_tree_partition(),
    )
    .expect("the tree's declared grouping is evaluable");
    let split_expected = partitioned_reference(
        &SEPARATING_OPERANDS,
        SEPARATING_ROWS,
        SEPARATING_COLUMNS,
        separating_split_partition(),
    )
    .expect("the split's declared grouping is evaluable");
    assert_ne!(
        tree_expected, split_expected,
        "the operands must separate the two declared groupings or this run observes nothing new",
    );

    let mut permitted = ordered_associations(&SEPARATING_OPERANDS);
    permitted.sort_unstable();
    permitted.dedup();
    eprintln!(
        "  operands {SEPARATING_OPERANDS:08x?}: {} distinct value(s) {permitted:08x?} over \
         {SEPARATING_COLUMNS} contributor(s); the tree's declared grouping gives \
         {tree_expected:08x?} and the split's {split_expected:08x?}",
        permitted.len(),
    );

    let Some(runs) = require_or_report(
        "serial sum tree-against-split",
        measured_portfolio(&SEPARATING_OPERANDS, SEPARATING_ROWS, SEPARATING_COLUMNS),
    ) else {
        return;
    };

    let mut cross_refusals = 0_usize;
    let mut observed = Vec::new();
    for run in &runs {
        let expected = partitioned_reference(
            &SEPARATING_OPERANDS,
            SEPARATING_ROWS,
            SEPARATING_COLUMNS,
            run.partition,
        )
        .expect("a published partition is evaluable");
        eprintln!(
            "  {} ({}): declared {} partition(s) of {} contributor(s), {} encoder(s), widest \
             workgroup {}, {} byte(s) of threadgroup memory, {:08x?} against its declared \
             grouping's {expected:08x?}",
            run.label(),
            run.stable_id,
            run.partition.partitions,
            run.partition.contributors_per_partition,
            run.encoders,
            run.widest_workgroup,
            run.threadgroup_bytes,
            run.bits,
        );
        assert!(
            declared_grouping_admits(&expected, &run.bits),
            "the {} declares {} partition(s) of {} contributor(s) and returned {:08x?}, and that \
             grouping produces {expected:08x?}",
            run.label(),
            run.partition.partitions,
            run.partition.contributors_per_partition,
            run.bits,
        );

        // The discriminating refusal, on the bits a device actually returned:
        // this alternative's answer against the *other* parallel strategy's
        // published grouping. The refused value is legal under this contract —
        // it is another alternative's correct answer — so what is being watched
        // is an oracle saying no to a wrong-but-permitted result, by the same
        // function that just admitted these bits.
        if let Some(strategy) = run.strategy {
            let other = match strategy {
                ParallelStrategy::SingleWorkgroupTree => separating_split_partition(),
                ParallelStrategy::MultiPassSplit => separating_tree_partition(),
            };
            let foreign = partitioned_reference(
                &SEPARATING_OPERANDS,
                SEPARATING_ROWS,
                SEPARATING_COLUMNS,
                other,
            )
            .expect("the other strategy's published grouping is evaluable");
            assert!(
                permitted.contains(&foreign[0]),
                "the refused value {:#010x} must be one this contract permits, or the refusal \
                 proves nothing",
                foreign[0],
            );
            assert!(
                !declared_grouping_admits(&foreign, &run.bits),
                "the {} returned {:08x?}, which the other parallel strategy's declared \
                 {other:?} also produces; the two groupings did not separate on this device",
                run.label(),
                run.bits,
            );
            cross_refusals += 1;
            eprintln!(
                "    refused the other parallel strategy's legal {foreign:08x?}, produced by \
                 {other:?}",
            );
            observed.push((strategy, run.bits.clone()));
        }
    }

    assert_eq!(
        cross_refusals, 2,
        "both parallel strategies must have been held to the other's declared grouping; \
         {cross_refusals} of them were",
    );
    let value_of = |wanted: ParallelStrategy| {
        observed
            .iter()
            .find_map(|(strategy, bits)| (*strategy == wanted).then(|| bits.clone()))
            .unwrap_or_else(|| panic!("the {wanted} was dispatched"))
    };
    assert_eq!(
        value_of(ParallelStrategy::SingleWorkgroupTree),
        tree_expected
    );
    assert_eq!(value_of(ParallelStrategy::MultiPassSplit), split_expected);
    assert_ne!(
        value_of(ParallelStrategy::SingleWorkgroupTree),
        value_of(ParallelStrategy::MultiPassSplit),
        "the device returned the same bits for both parallel strategies, so this run separated \
         nothing",
    );
    eprintln!(
        "serial sum tree-against-split: the tree returned {:08x?} at {:?} and the split \
         {:08x?} at {:?}; each matched its own declared grouping bit for bit and each was \
         refused by the other's",
        value_of(ParallelStrategy::SingleWorkgroupTree),
        separating_tree_partition(),
        value_of(ParallelStrategy::MultiPassSplit),
        separating_split_partition(),
    );
}
