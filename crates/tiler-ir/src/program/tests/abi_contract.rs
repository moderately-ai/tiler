//! The program's whole-program contract: the applicability guard, the routing-commit
//! lifecycle, and the ABI expression arena's structural comparator, deduplication, and
//! identity growth.

use super::super::abi::{
    AbiBinaryOp, AbiRoot, AbiType, AbiUnaryOp, AvailabilityPhase, TargetPropertyKey,
};
use super::super::{
    KernelProgramBuildError, KernelProgramDiagnostic, ProgramAbiUse, RoutingCommitState,
    RoutingCommitTransition, StageLaunch, VerifiedKernelProgram,
};
use super::support::{
    AbiGrowth, OTHER_SCALE_BITS, SCALE_BITS, TwoStageShape, canonical_program, complete_two_stage,
    declare_guard, declare_routing_commit, declare_routing_commit_with_fallback, diagnostic,
    grown_guard, literal, occurrences, pointwise_kernel, read, serial_sum_program, two_stage,
    wire_two_stage_structure, write_access,
};
use crate::semantic::SemanticProgram;

#[test]
fn identity_changes_when_the_applicability_guard_changes() {
    // One semantic graph, one pair of bound implementations, one structure and
    // one routing contract: only the predicate deciding whether this program
    // may be routed to differs. Under `tiler.kernel-program.v1` these two were
    // the same bytes, which is the cache hazard the domain bump closes.
    let semantic = serial_sum_program(SCALE_BITS);
    let canonical = canonical_program(&semantic);

    let mut builder = wire_two_stage_structure(two_stage(&semantic, TwoStageShape::Canonical));
    let two = literal(&mut builder, 2);
    let guard = builder
        .push_abi_binary(AbiBinaryOp::Equal, two, two)
        .expect("a differently spelled predicate");
    builder.applicability_guard(guard).expect("guard");
    declare_routing_commit(&mut builder);
    let guarded = builder.build().expect("verified kernel program");

    assert_ne!(
        canonical.canonical_identity().as_bytes(),
        guarded.canonical_identity().as_bytes()
    );
    assert_ne!(canonical, guarded);
}

#[test]
fn identity_changes_when_the_entry_abi_changes() {
    // The two programs agree on every byte count and every launch extent; they
    // disagree only on how those quantities are *computed*. A dynamic subject
    // computes them from bound input extents, so an identity blind to the
    // expression would collapse two programs whose ABI differs at run time.
    let semantic = serial_sum_program(SCALE_BITS);
    let canonical = canonical_program(&semantic);
    let computed = complete_two_stage(two_stage(&semantic, TwoStageShape::ComputedAccessibleBytes))
        .build()
        .expect("verified kernel program");

    let accesses = |program: &VerifiedKernelProgram| {
        program
            .stages()
            .map(|stage| stage.accesses().len())
            .sum::<usize>()
    };
    assert_eq!(accesses(&canonical), accesses(&computed));
    assert_ne!(
        canonical.canonical_identity().as_bytes(),
        computed.canonical_identity().as_bytes()
    );
}

#[test]
fn identity_changes_when_pre_commit_fallback_permission_changes() {
    // A program that may still be abandoned before commit and one that may not
    // are different execution contracts over identical work.
    let semantic = serial_sum_program(SCALE_BITS);
    let permitted = canonical_program(&semantic);

    let mut builder = wire_two_stage_structure(two_stage(&semantic, TwoStageShape::Canonical));
    declare_guard(&mut builder);
    declare_routing_commit_with_fallback(&mut builder, false);
    let forbidden = builder.build().expect("verified kernel program");

    assert!(permitted.routing_commit_contract()[0].fallback_permitted);
    assert!(!forbidden.routing_commit_contract()[0].fallback_permitted);
    assert_ne!(
        permitted.canonical_identity().as_bytes(),
        forbidden.canonical_identity().as_bytes()
    );
}

#[test]
fn a_program_without_an_applicability_guard_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut builder = wire_two_stage_structure(two_stage(&semantic, TwoStageShape::Canonical));
    declare_routing_commit(&mut builder);
    assert_eq!(
        diagnostic(builder),
        KernelProgramDiagnostic::MissingApplicabilityGuard
    );
}

#[test]
fn a_routing_commit_contract_that_stops_short_of_publication_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut builder = wire_two_stage_structure(two_stage(&semantic, TwoStageShape::Canonical));
    declare_guard(&mut builder);
    builder
        .push_routing_commit_transition(RoutingCommitTransition {
            from: RoutingCommitState::Preflight,
            to: RoutingCommitState::Committed,
            fallback_permitted: true,
        })
        .expect("the first transition is well formed");
    assert_eq!(
        diagnostic(builder),
        KernelProgramDiagnostic::IncompleteRoutingCommitContract {
            declared: 1,
            required: 3,
        }
    );
}

#[test]
fn a_routing_commit_step_that_breaks_the_lifecycle_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    wired
        .builder
        .push_routing_commit_transition(RoutingCommitTransition {
            from: RoutingCommitState::Preflight,
            to: RoutingCommitState::Committed,
            fallback_permitted: true,
        })
        .expect("the first transition is well formed");
    assert_eq!(
        wired
            .builder
            .push_routing_commit_transition(RoutingCommitTransition {
                from: RoutingCommitState::Committed,
                to: RoutingCommitState::Executing,
                fallback_permitted: true,
            })
            .expect_err("fallback after commit is rejected"),
        KernelProgramBuildError::RoutingCommitFallbackAfterCommit {
            from: RoutingCommitState::Committed,
        }
    );
    // A step that skips the state the previous one reached is rejected too.
    assert_eq!(
        wired
            .builder
            .push_routing_commit_transition(RoutingCommitTransition {
                from: RoutingCommitState::Executing,
                to: RoutingCommitState::Published,
                fallback_permitted: false,
            })
            .expect_err("the lifecycle order is checked"),
        KernelProgramBuildError::RoutingCommitOutOfOrder {
            expected: RoutingCommitState::Committed,
            actual: RoutingCommitState::Executing,
        }
    );
}

#[test]
fn an_abi_expression_no_use_site_reaches_is_rejected() {
    // Identity writes the reached arena once and names each use by canonical
    // position, so a node no use reaches would be retained program state omitted
    // by that traversal.
    let semantic = serial_sum_program(SCALE_BITS);
    let mut builder = complete_two_stage(two_stage(&semantic, TwoStageShape::Canonical));
    literal(&mut builder, 4_096);
    assert_eq!(
        diagnostic(builder),
        KernelProgramDiagnostic::UnreferencedAbiExpression
    );
}

#[test]
fn an_accessible_range_the_declared_view_contradicts_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::ShiftedCoverage);
    let wrong = literal(&mut wired.builder, 25);
    let abi = wired.abi;
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(2, OTHER_SCALE_BITS),
                &occurrences(&semantic, 3..4),
                &[
                    read(wired.source_view, wrong),
                    write_access(wired.temporary_view, abi.input_bytes),
                ],
                abi.pointwise_launch(),
            )
            .expect_err("an accessible range must equal the view it addresses"),
        KernelProgramBuildError::AccessibleBytesDisagreement {
            position: 0,
            expected: 24,
            actual: 25,
        }
    );
}

#[test]
fn a_workgroup_width_the_bound_kernel_contradicts_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::ShiftedCoverage);
    let wrong_width = literal(&mut wired.builder, 32);
    let abi = wired.abi;
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(2, OTHER_SCALE_BITS),
                &occurrences(&semantic, 3..4),
                &[
                    read(wired.source_view, abi.input_bytes),
                    write_access(wired.temporary_view, abi.input_bytes),
                ],
                StageLaunch {
                    grid_threads: abi.pointwise_threads,
                    threads_per_workgroup: wrong_width,
                },
            )
            .expect_err("a declared workgroup width must be the kernel's"),
        KernelProgramBuildError::ThreadsPerWorkgroupDisagreement {
            expected: 1,
            actual: 32,
        }
    );
}

#[test]
fn an_abi_use_site_rejects_a_mistyped_or_target_dependent_expression() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);

    // A size is not a guard.
    assert_eq!(
        wired
            .builder
            .applicability_guard(wired.abi.input_bytes)
            .expect_err("a guard must be a predicate"),
        KernelProgramBuildError::AbiUseType {
            use_site: ProgramAbiUse::ApplicabilityGuard,
            expected: AbiType::Boolean,
            actual: AbiType::Unsigned,
        }
    );
    // A guard is not a size.
    let predicate = wired
        .builder
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("predicate");
    assert_eq!(
        wired
            .builder
            .push_abi_unary(AbiUnaryOp::NarrowU32, predicate)
            .expect_err("a narrowing operand must be unsigned"),
        KernelProgramBuildError::AbiOperandType {
            expected: AbiType::Unsigned,
            actual: AbiType::Boolean,
        }
    );

    // A launch extent must be computable before any device-dependent query, so
    // a governed target property is refused at that use site.
    let property = wired
        .builder
        .push_abi_root(AbiRoot::TargetProperty {
            key: TargetPropertyKey::new("tiler.test.max-threads").expect("property key"),
            phase: AvailabilityPhase::LiveDevicePreflight,
        })
        .expect("target property root");
    let abi = wired.abi;
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(2, OTHER_SCALE_BITS),
                &occurrences(&semantic, 0..1),
                &[
                    read(wired.source_view, abi.input_bytes),
                    write_access(wired.temporary_view, abi.input_bytes),
                ],
                StageLaunch {
                    grid_threads: property,
                    threads_per_workgroup: abi.threads_per_workgroup,
                },
            )
            .expect_err("a launch extent must read only interface facts"),
        KernelProgramBuildError::AbiNonInterfaceRoot {
            use_site: ProgramAbiUse::GridThreads,
        }
    );
}

#[test]
fn the_abi_arena_is_deduplicated_by_content() {
    // The canonical fixture names the same input byte count at three accesses
    // and the same workgroup width at both stages; the arena keeps one node per
    // distinct formula, so it stays a function of what the program says.
    let semantic = serial_sum_program(SCALE_BITS);
    let program = canonical_program(&semantic);
    // 24, 8, 6, 2, 1, and the guard predicate.
    assert_eq!(program.abi_expressions().len(), 6);

    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    assert_eq!(literal(&mut wired.builder, 24), wired.abi.input_bytes);
}

/// The canonical two-stage program, with its guard grown to `levels`.
fn program_with_grown_abi(
    semantic: &SemanticProgram,
    growth: AbiGrowth,
    levels: usize,
) -> VerifiedKernelProgram {
    let mut builder = wire_two_stage_structure(two_stage(semantic, TwoStageShape::Canonical));
    let guard = grown_guard(&mut builder, growth, levels);
    builder
        .applicability_guard(guard)
        .expect("applicability guard");
    declare_routing_commit(&mut builder);
    builder.build().expect("verified kernel program")
}

/// The structural comparator is a strict total order over an arena.
///
/// This is what the two sorted expression sets in artifact identity rest on, so
/// a comparator that were merely *consistent* would not do: an intransitive one
/// makes `sort_by` produce an order that depends on the input permutation,
/// which is precisely the canonicity the sort exists to provide.
///
/// Checked exhaustively over every ordered pair and triple of a small arena
/// that carries all four constructors, sharing included.
#[test]
fn the_structural_comparator_is_a_total_order() {
    use crate::program::abi::{AbiBinaryOp, AbiRoot, AbiUnaryOp, ExprNode, compare_expr_nodes};
    use core::cmp::Ordering;

    // 0,1 are distinct leaves; 2 shares leaf 0; 3 and 4 differ only in operand
    // order, which is what a comparator that ignored operand position would miss.
    let nodes = vec![
        ExprNode::Root(AbiRoot::UnsignedLiteral(7)),
        ExprNode::Root(AbiRoot::UnsignedLiteral(9)),
        ExprNode::Unary {
            op: AbiUnaryOp::Not,
            operand: 0,
        },
        ExprNode::Binary {
            op: AbiBinaryOp::CheckedAdd,
            left: 0,
            right: 1,
        },
        ExprNode::Binary {
            op: AbiBinaryOp::CheckedAdd,
            left: 1,
            right: 0,
        },
        ExprNode::Select {
            condition: 2,
            if_true: 3,
            if_false: 4,
        },
    ];
    let all: Vec<u32> = (0..u32::try_from(nodes.len()).unwrap()).collect();
    let cmp = |a: u32, b: u32| compare_expr_nodes(&nodes, a, b);

    for &a in &all {
        assert_eq!(cmp(a, a), Ordering::Equal, "not reflexive at {a}");
        for &b in &all {
            assert_eq!(
                cmp(a, b),
                cmp(b, a).reverse(),
                "not antisymmetric at ({a}, {b})"
            );
            // Distinct arena positions holding structurally distinct nodes must
            // not compare equal, or two different expressions would tie and the
            // sorted order would depend on input order.
            if a != b {
                assert_ne!(cmp(a, b), Ordering::Equal, "{a} and {b} tied");
            }
            for &c in &all {
                if cmp(a, b) == Ordering::Less && cmp(b, c) == Ordering::Less {
                    assert_eq!(
                        cmp(a, c),
                        Ordering::Less,
                        "not transitive at ({a}, {b}, {c})"
                    );
                }
            }
        }
    }

    // Operand order is part of the structure: nodes 3 and 4 are `a + b` and
    // `b + a` over the same leaves and must not tie.
    assert_ne!(cmp(3, 4), Ordering::Equal, "operand order was ignored");
}

/// Reports identity size against arena size, and proves the curve is a line.
///
/// **The growth rate is the finding, not the absolute number.** Identity size
/// is deterministic — the same program yields the same byte count on every host
/// — so this needs neither repetition nor statistics, unlike a timing
/// measurement.
///
/// A constant increment per level is exactly the property the `v3` encoding
/// buys, and asserting it is what makes this a guard rather than a print: under
/// `v2`, which named each use site by a key that embedded its operands' keys,
/// the increment grew with the level under `Chain` and doubled under
/// `SharedDag`. See the ticket outcome for the two measured curves.
///
/// Reproduce with:
///
/// ```text
/// cargo nextest run -p tiler-ir -E 'test(abi_identity_size)' --no-capture
/// ```
#[test]
fn abi_identity_size_grows_linearly_with_the_arena() {
    /// Enough levels that a quadratic or an exponential curve is unmistakable,
    /// and few enough that a `SharedDag` fixture still fits in memory for
    /// anyone re-running this against the `v2` encoding.
    const LEVELS: std::ops::Range<usize> = 0..17;

    let semantic = serial_sum_program(SCALE_BITS);
    for growth in [AbiGrowth::Chain, AbiGrowth::SharedDag] {
        let mut sizes = Vec::new();
        for levels in LEVELS {
            let program = program_with_grown_abi(&semantic, growth, levels);
            let nodes = program.abi_expressions().len();
            let bytes = program.canonical_identity().as_bytes().len();
            println!("MEASURE {growth:?} {levels:>2} levels: {nodes:>2} nodes, {bytes} bytes");
            sizes.push((nodes, bytes));
        }

        // The first level is the one that mints the shared `false` leaf under
        // `Chain`, so the constant-increment claim starts after it.
        let increments: Vec<usize> = sizes
            .windows(2)
            .skip(1)
            .map(|pair| pair[1].1 - pair[0].1)
            .collect();
        assert!(
            increments.windows(2).all(|pair| pair[0] == pair[1]),
            "{growth:?} identity size must grow by a constant per level, measured {increments:?}"
        );
        let added_nodes: Vec<usize> = sizes
            .windows(2)
            .skip(1)
            .map(|pair| pair[1].0 - pair[0].0)
            .collect();
        assert!(
            added_nodes.iter().all(|added| *added == 1),
            "each level must add exactly one arena node, measured {added_nodes:?}"
        );
    }
}

/// Two guards over the same node kinds, wired differently, must differ.
///
/// Encoding the arena once and naming nodes by canonical position moves the
/// whole burden of distinguishing two expressions onto those position
/// references. This is the case that a reference encoding losing operand order,
/// or losing which node an operand names, would pass anyway: both programs hold
/// one `true`, one `false`, and two `Or`s, and differ only in what those `Or`s
/// name.
#[test]
fn identity_distinguishes_two_arenas_that_differ_only_in_their_wiring() {
    let semantic = serial_sum_program(SCALE_BITS);
    let build = |nest_left: bool| {
        let mut builder = wire_two_stage_structure(two_stage(&semantic, TwoStageShape::Canonical));
        let yes = builder
            .push_abi_root(AbiRoot::BooleanLiteral(true))
            .expect("true root");
        let no = builder
            .push_abi_root(AbiRoot::BooleanLiteral(false))
            .expect("false root");
        let inner = builder
            .push_abi_binary(AbiBinaryOp::Or, yes, no)
            .expect("inner disjunction");
        let guard = if nest_left {
            builder.push_abi_binary(AbiBinaryOp::Or, inner, no)
        } else {
            builder.push_abi_binary(AbiBinaryOp::Or, yes, inner)
        }
        .expect("outer disjunction");
        builder
            .applicability_guard(guard)
            .expect("applicability guard");
        declare_routing_commit(&mut builder);
        builder.build().expect("verified kernel program")
    };

    let left = build(true);
    let right = build(false);
    assert_eq!(left.abi_expressions().len(), right.abi_expressions().len());
    assert_ne!(
        left.canonical_identity().as_bytes(),
        right.canonical_identity().as_bytes()
    );
}
