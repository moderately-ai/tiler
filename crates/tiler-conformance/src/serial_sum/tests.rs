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

use tiler_compiler::session::NumericalContract;
use tiler_ir::schedule::ContributorPartition;
use tiler_metal::applicability::MetalHostPredicate;

use super::{
    COLUMNS, GROUPING_SENSITIVE_OPERANDS, PARALLEL_COLUMNS, PARALLEL_OPERANDS, PARALLEL_ROWS,
    ParallelStrategy, ROW_PATTERNS, ROWS, compile_under, declaration, declared_grouping_admits,
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
        let declared = parallel_split();
        let correct = partitioned_reference(&operands, PARALLEL_ROWS, PARALLEL_COLUMNS, declared)
            .expect("the declared split is evaluable");
        let mut population = 0_usize;
        let mut escaped = 0_usize;
        for slot in 0..4 {
            for source in 0..5 {
                let mut corrupt = operands;
                // Source 4 is the dropped case: the contributor is replaced by
                // the reduction's own identity element.
                corrupt[slot] = if source == 4 {
                    0.0_f32.to_bits()
                } else if source == slot {
                    continue;
                } else {
                    operands[source]
                };
                population += 1;
                let observed =
                    partitioned_reference(&corrupt, PARALLEL_ROWS, PARALLEL_COLUMNS, declared)
                        .expect("a corrupted operand set is still evaluable");
                if declared_grouping_admits(&correct, &observed) {
                    escaped += 1;
                }
            }
        }
        (population, escaped)
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
    // — they diverge from twelve contributors upward. Stating it here is what
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
    // **The deliverable, and it is the *authority* refusal rather than merely a
    // refusal.** A fully observed row carries past every ambient and device
    // predicate and stops at ADR 0086: native device translation of a metallib
    // during pipeline creation is a typed capability fact whose authority is
    // `Unknown` on every macOS row currently observable. An assertion that
    // something refused would pass on a host that never observed its device,
    // which says nothing about ADR 0086 at all.
    assert_eq!(
        refusal.predicate(),
        MetalHostPredicate::NativeTranslationAuthority,
        "an observed host must reach ADR 0086's authority refusal rather than stop earlier",
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
        measured_portfolio(&PARALLEL_OPERANDS),
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
/// **The corpus's only device observation of a different-but-permitted
/// reassociated answer.** The case above runs operands every grouping of which is
/// exact, so its refusal population among legal groupings is empty: no answer a
/// reassociating contract permits would have failed it. This one runs the same
/// three alternatives on operands where the declared regroupings genuinely
/// disagree, so the oracle changes shape — from the serial fold to the exact
/// value the strategy's *own declared grouping* produces, read off the plan
/// rather than assumed.
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
        measured_portfolio(&GROUPING_SENSITIVE_OPERANDS),
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
