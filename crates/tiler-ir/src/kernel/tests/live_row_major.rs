use super::super::{BlockRef, OperationView, lower_scheduled_region};
use super::support::{
    ABSENT_SUBGROUP_KERNEL_IDENTITY_HEX, bf16_numerical, linear_schedule, live_row_major_region,
    numerical, pointwise_region,
};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    LogicalAccess, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    PointwiseF32ExpressionBuilder, RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder,
    TensorRole,
};
use crate::shape::{Axis, Shape};
use std::fmt::Write as _;

/// The retired contextual `LiveRowMajor { inner_axis }` fixture's exact bytes.
///
/// Retained rather than deleted so the accepted source-bound replacement's
/// blast radius stays a measured fact: the fixture's read carried tag `0x09`
/// plus its axis and its write carried the same, and the pin test below proves
/// the replaced spelling's bytes differ from these while the all-static
/// neighbour's stay exact. `0x09` is permanently retired; a reader that finds
/// these bytes equal to a current identity has found the collision the fresh
/// `0x0A`/`0x0B` tags exist to prevent.
const RETIRED_CONTEXTUAL_LIVE_ROW_MAJOR_SCHEDULE_IDENTITY_HEX: &str = "74696c65722e7363686564756c652e763700000000000000000100000000000000020000000000000002010001090000000100000000000200020900000001000000010100000000000000000000000200000000010011000000000000000000000001020011000000000000000000000000020000000000000002240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc00000010101010101010101010100000000000000020000000101000000003100000000000000020000000101";
/// The same fixture's retired kernel bytes, which frame the schedule bytes.
const RETIRED_CONTEXTUAL_LIVE_ROW_MAJOR_KERNEL_IDENTITY_HEX: &str = "74696c65722e6b65726e656c2e763900000000000000018474696c65722e7363686564756c652e763700000000000000000100000000000000020000000000000002010001090000000100000000000200020900000001000000010100000000000000000000000200000000010011000000000000000000000001020011000000000000000000000000020000000000000002240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010101010000000000000002000000010100000000310000000000000002000000010100000000000000020100030101000000000000000002000301020000000000000000000000000000000101000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010101000000020000000100000000000000000101000101010101010101010100000000000000120202020102020202020203030303030303020000000000000000000000000000000520000000000000000000000001000000001101000000000000000100000001120200000000000000020000000000000001000000021401000000010000000200000000000000010000000318000000030000000000000000000000000000000312020000000000000000000000000000000100000004120200000000000000000000000000000001000000051f000000040000000000000000000000010000000500000000000000010000000700000000000000020000000600000007000000000000000a13020000000100000000000000000000000100000008130100000008000000060000000000000001000000091600000000000000090000000000000000000000010000000a12034000000000000000000000010000000b13060000000a0000000b00000000000000010000000c15010000000c00000000000000010000000d12033f80000000000000000000010000000e13050000000d0000000e00000000000000010000000f15010000000f00000000000000010000001017000000010000000900000010000000010000000000000000000000000000000000000001000000110000000000000000fe00000000000000010000000000000001";
/// The source-bound live fixture's schedule bytes: `0x0A` plus the axis on the
/// one marker read, the bare `0x0B` on the consuming write, and the bare `0x14`
/// on each of its two bounds proofs.
///
/// Rebaselined twice, deliberately both times. First at the accepted 2026-08-18
/// fieldless-marker replacement, whose exact prior values the retired constants
/// above hold; then again when `BoundsProofKind::LiveExtentReach` replaced the
/// `LinearRange { element_count: 0 }` a live access used to carry, which is
/// **sixteen bytes shorter** — two proofs each trading a nine-byte `0x11` plus
/// a `u64` for a one-byte `0x14`. The kernel pin below frames these bytes, so
/// its own nested length prefix moves from `0x0180` to `0x0170` by the same
/// sixteen.
///
/// Neither rebaseline stepped a domain, and the second one's argument is
/// recorded at `TAG_LIVE_EXTENT_REACH`: `0x14` is unreachable to any earlier
/// region, and the retired spelling becomes unreachable to any later one, so a
/// retained pre-migration identity misses rather than matching a subject it
/// does not name.
const LIVE_ROW_MAJOR_SCHEDULE_IDENTITY_HEX: &str = "74696c65722e7363686564756c652e7637000000000000000001000000000000000200000000000000020100010a0000000100000000000200020b0000000101000000000000000000000002000000000100140000000102001400000000020000000000000002240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc00000010101010101010101010100000000000000020000000101000000003100000000000000020000000101";
const LIVE_ROW_MAJOR_KERNEL_IDENTITY_HEX: &str = "74696c65722e6b65726e656c2e763900000000000000017074696c65722e7363686564756c652e7637000000000000000001000000000000000200000000000000020100010a0000000100000000000200020b0000000101000000000000000000000002000000000100140000000102001400000000020000000000000002240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010101010000000000000002000000010100000000310000000000000002000000010100000000000000020100030101000000000000000002000301020000000000000000000000000000000101000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010101000000020000000100000000000000000101000101010101010101010100000000000000120202020102020202020203030303030303020000000000000000000000000000000520000000000000000000000001000000001101000000000000000100000001120200000000000000020000000000000001000000021401000000010000000200000000000000010000000318000000030000000000000000000000000000000312020000000000000000000000000000000100000004120200000000000000000000000000000001000000051f000000040000000000000000000000010000000500000000000000010000000700000000000000020000000600000007000000000000000a13020000000100000000000000000000000100000008130100000008000000060000000000000001000000091600000000000000090000000000000000000000010000000a12034000000000000000000000010000000b13060000000a0000000b00000000000000010000000c15010000000c00000000000000010000000d12033f80000000000000000000010000000e13050000000d0000000e00000000000000010000000f15010000000f00000000000000010000001017000000010000000900000010000000010000000000000000000000000000000000000001000000110000000000000000fe00000000000000010000000000000001";

#[derive(Clone, Copy)]
enum PointwiseWidth {
    F32,
    Bf16,
}

fn two_input_pointwise_program(width: PointwiseWidth) -> ScalarProgram {
    match width {
        PointwiseWidth::F32 => {
            let mut expression = PointwiseF32ExpressionBuilder::new();
            let left = expression.input(AccessOrdinal::FIRST).unwrap();
            let right = expression.input(AccessOrdinal::new(1)).unwrap();
            let root = expression.add(left, right).unwrap();
            ScalarProgram::PointwiseF32(expression.build(root).unwrap())
        }
        PointwiseWidth::Bf16 => {
            let mut expression = crate::schedule::PointwiseBf16ExpressionBuilder::new();
            let left = expression.input(AccessOrdinal::FIRST).unwrap();
            let right = expression.input(AccessOrdinal::new(1)).unwrap();
            let root = expression.add(left, right).unwrap();
            ScalarProgram::PointwiseBf16(expression.build(root).unwrap())
        }
    }
}

fn two_input_pointwise_builder(
    width: PointwiseWidth,
    rows: u64,
    read_maps: [LogicalAccess; 2],
    write_map: LogicalAccess,
) -> ScheduledRegionBuilder {
    two_input_pointwise_builder_with_proofs(width, rows, read_maps, write_map, [None, None, None])
}

/// The same fixture with per-access bounds-proof overrides.
///
/// An override is how a test perturbs the *pairing* rather than the relation:
/// `ScheduledRegionBuilder`'s proof list is private outside its own module, so
/// a refusal test cannot reach in and demote a proof after the fact. Passing
/// `None` takes the kind the relation implies, which is what every other
/// caller wants.
fn two_input_pointwise_builder_with_proofs(
    width: PointwiseWidth,
    rows: u64,
    read_maps: [LogicalAccess; 2],
    write_map: LogicalAccess,
    proof_overrides: [Option<BoundsProofKind>; 3],
) -> ScheduledRegionBuilder {
    let [read_0_proof, read_1_proof, write_proof] = proof_overrides;
    let mut read_proofs = [read_0_proof, read_1_proof].into_iter();
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(23));
    builder.iteration_shape(Shape::from_dims([rows])).unwrap();
    for (position, map) in read_maps.into_iter().enumerate() {
        // The *kind* is chosen from the relation, not an element count chosen
        // from it. A live access states `LiveExtentReach`; only a static one
        // has a count to state. Deriving a zero here instead is what let this
        // fixture spell a live obligation as a bounded range over a variable,
        // which no literal-`0` census could have found.
        let kind = read_proofs.next().flatten().unwrap_or_else(|| match &map {
            LogicalAccess::LinearIdentity => BoundsProofKind::LinearRange {
                element_count: rows,
            },
            LogicalAccess::LiveRowMajorSource { .. } | LogicalAccess::LiveRowMajor => {
                BoundsProofKind::LiveExtentReach
            }
            _ => panic!("the focused fixture only constructs identity and live accesses"),
        });
        let witness = u32::try_from(position).unwrap();
        builder
            .push_access(Access {
                tensor: TensorRole::Input,
                component_role: None,
                mode: AccessMode::Read,
                map,
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor: TensorRole::Input,
                component_role: None,
                kind,
            })
            .unwrap();
    }
    let write_kind = write_proof.unwrap_or_else(|| match &write_map {
        LogicalAccess::LinearIdentity => BoundsProofKind::LinearRange {
            element_count: rows,
        },
        LogicalAccess::LiveRowMajorSource { .. } | LogicalAccess::LiveRowMajor => {
            BoundsProofKind::LiveExtentReach
        }
        _ => panic!("the focused fixture only constructs identity and live accesses"),
    });
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Write,
            map: write_map,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: write_kind,
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: rows },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: two_input_pointwise_program(width),
            numerical: match width {
                PointwiseWidth::F32 => numerical(),
                PointwiseWidth::Bf16 => bf16_numerical(),
            },
        })
        .unwrap();
    builder
        .schedule(linear_schedule(rows, OwnershipWitnessId::new(0)))
        .unwrap();
    builder
}

fn identity_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        write!(&mut hex, "{byte:02x}").unwrap();
    }
    hex
}

fn assert_live_source_rule(
    builder: ScheduledRegionBuilder,
    expected: crate::schedule::LiveRowMajorSourceRule,
    subject: &str,
) {
    let error = builder
        .build()
        .expect_err("a malformed live-row-major source relation must fail intrinsically");
    assert_eq!(
        error.diagnostics(),
        [crate::schedule::ScheduledRegionDiagnostic::LiveRowMajorSource { rule: expected }],
        "the {subject} perturbation must stop at its dedicated source rule: {error:?}"
    );
    assert_eq!(
        error.diagnostics()[0].rule(),
        expected.rule(),
        "the {subject} refusal must carry the stable dedicated rule identifier"
    );
}

/// Rule 1, absent: fieldless consumers with no marker have no axis authority.
///
/// Both widths, so the refusing boundary is provably the shared verifier.
#[test]
fn a_live_consumer_set_with_no_source_marker_is_refused() {
    for width in [PointwiseWidth::F32, PointwiseWidth::Bf16] {
        assert_live_source_rule(
            two_input_pointwise_builder(
                width,
                2,
                [LogicalAccess::LiveRowMajor, LogicalAccess::LiveRowMajor],
                LogicalAccess::LiveRowMajor,
            ),
            crate::schedule::LiveRowMajorSourceRule::Missing,
            "missing-marker",
        );
    }
}

/// Rule 1, doubled: a second marker would be a second runtime extent authority.
#[test]
fn two_live_source_markers_are_refused_with_both_coordinates() {
    let inner = Axis::new(1);
    assert_live_source_rule(
        two_input_pointwise_builder(
            PointwiseWidth::F32,
            2,
            [
                LogicalAccess::LiveRowMajorSource { inner_axis: inner },
                LogicalAccess::LiveRowMajorSource { inner_axis: inner },
            ],
            LogicalAccess::LiveRowMajor,
        ),
        crate::schedule::LiveRowMajorSourceRule::Multiple {
            first: AccessOrdinal::new(0),
            second: AccessOrdinal::new(1),
        },
        "double-marker",
    );
}

/// Rule 2, wrong role/mode: a marker on the owning write declares a runtime
/// input-axis operand no program input backs.
#[test]
fn a_source_marker_on_the_owning_write_is_refused() {
    let inner = Axis::new(1);
    assert_live_source_rule(
        two_input_pointwise_builder(
            PointwiseWidth::F32,
            2,
            [LogicalAccess::LiveRowMajor, LogicalAccess::LiveRowMajor],
            LogicalAccess::LiveRowMajorSource { inner_axis: inner },
        ),
        crate::schedule::LiveRowMajorSourceRule::SourceNotInputRead {
            source: AccessOrdinal::new(2),
        },
        "marker-on-write",
    );
}

/// Rule 2, wrong boundary: a marker on an intermediate read is equally not a
/// program input, and the refusal names the exact access.
#[test]
fn a_source_marker_on_an_intermediate_read_is_refused() {
    let inner = Axis::new(1);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(23));
    builder.iteration_shape(Shape::from_dims([2])).unwrap();
    for (position, tensor) in [TensorRole::Intermediate, TensorRole::Input]
        .into_iter()
        .enumerate()
    {
        let witness = u32::try_from(position).unwrap();
        builder
            .push_access(Access {
                tensor,
                component_role: None,
                mode: AccessMode::Read,
                map: if position == 0 {
                    LogicalAccess::LiveRowMajorSource { inner_axis: inner }
                } else {
                    LogicalAccess::LiveRowMajor
                },
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LiveExtentReach,
            })
            .unwrap();
    }
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LiveRowMajor,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LiveExtentReach,
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: two_input_pointwise_program(PointwiseWidth::F32),
            numerical: numerical(),
        })
        .unwrap();
    builder
        .schedule(linear_schedule(2, OwnershipWitnessId::new(0)))
        .unwrap();
    assert_live_source_rule(
        builder,
        crate::schedule::LiveRowMajorSourceRule::SourceNotInputRead {
            source: AccessOrdinal::new(0),
        },
        "marker-on-intermediate-read",
    );
}

/// Rule 3, read side: a static read cannot execute inside the selected live
/// loop, and the refusal names its exact access coordinate. Both widths, so
/// the refusing boundary is provably the shared verifier.
///
/// This is the mixed static/live subject the landed
/// `refuse-mixed-pointwise-live-row-major-access-relations-before-lowering`
/// repair closed broadly; it stays closed here under the exact owning source
/// diagnostic.
#[test]
fn a_mixed_live_row_major_read_is_refused_for_f32_and_bf16() {
    let inner = Axis::new(1);
    for width in [PointwiseWidth::F32, PointwiseWidth::Bf16] {
        assert_live_source_rule(
            two_input_pointwise_builder(
                width,
                2,
                [
                    LogicalAccess::LinearIdentity,
                    LogicalAccess::LiveRowMajorSource { inner_axis: inner },
                ],
                LogicalAccess::LiveRowMajor,
            ),
            crate::schedule::LiveRowMajorSourceRule::ConsumerMissingRelation {
                access: AccessOrdinal::new(0),
            },
            "static-read-in-live-loop",
        );
    }
}

/// Rule 3, write side: a static owning write cannot sit inside the live loop
/// selected by its reads, and the refusal names the write's own coordinate.
#[test]
fn a_mixed_live_row_major_write_is_refused() {
    let inner = Axis::new(1);
    assert_live_source_rule(
        two_input_pointwise_builder(
            PointwiseWidth::F32,
            2,
            [
                LogicalAccess::LiveRowMajorSource { inner_axis: inner },
                LogicalAccess::LiveRowMajor,
            ],
            LogicalAccess::LinearIdentity,
        ),
        crate::schedule::LiveRowMajorSourceRule::ConsumerMissingRelation {
            access: AccessOrdinal::new(2),
        },
        "static-write-in-live-loop",
    );
}

/// The accepted four-rule census is total, sized from the type.
///
/// `variant_count` makes a widened rule vocabulary a build error at this
/// enumeration rather than a population that silently shrinks; the stable
/// identifiers are pinned exactly and pairwise distinct. There is deliberately
/// no `AxisMismatch` and no reference rule to list: the fieldless consumer
/// stores no axis and no handle, so neither failure state is representable —
/// the compile-fail doctests on [`LogicalAccess::LiveRowMajor`] prove the
/// fields cannot be added without a build error.
#[test]
fn the_live_row_major_source_rule_census_is_exactly_the_accepted_four() {
    use crate::schedule::LiveRowMajorSourceRule;

    const RULES: [LiveRowMajorSourceRule; std::mem::variant_count::<LiveRowMajorSourceRule>()] = [
        LiveRowMajorSourceRule::Missing,
        LiveRowMajorSourceRule::Multiple {
            first: AccessOrdinal::new(0),
            second: AccessOrdinal::new(1),
        },
        LiveRowMajorSourceRule::SourceNotInputRead {
            source: AccessOrdinal::new(0),
        },
        LiveRowMajorSourceRule::ConsumerMissingRelation {
            access: AccessOrdinal::new(0),
        },
    ];
    assert_eq!(RULES.len(), 4, "the accepted census is exactly four rules");
    let identifiers: Vec<&str> = RULES.iter().map(|rule| rule.rule()).collect();
    assert_eq!(
        identifiers,
        [
            "live-row-major-source-missing",
            "live-row-major-source-multiple",
            "live-row-major-source-not-input-read",
            "live-row-major-source-consumer-missing-relation",
        ],
        "the stable identifiers are pinned exactly"
    );
    let mut deduplicated = identifiers.clone();
    deduplicated.sort_unstable();
    deduplicated.dedup();
    assert_eq!(deduplicated.len(), RULES.len(), "no two rules share a name");
}

/// Precedence: a region wrong on marker count *and* coverage reports the
/// marker count, which is the accepted first-failure order — the consumer set
/// cannot be judged before the region has one axis authority to judge it
/// against.
#[test]
fn the_marker_count_rule_precedes_the_coverage_rule() {
    assert_live_source_rule(
        two_input_pointwise_builder(
            PointwiseWidth::F32,
            2,
            [LogicalAccess::LinearIdentity, LogicalAccess::LiveRowMajor],
            LogicalAccess::LiveRowMajor,
        ),
        crate::schedule::LiveRowMajorSourceRule::Missing,
        "missing-marker-before-coverage",
    );
}

/// The accepted replacement moves every live pin and no static one.
///
/// Three claims in one census: the all-static neighbour's kernel bytes are
/// byte-identical to their pre-replacement pin; the all-live fixture's
/// schedule and kernel bytes equal their rebaselined source-bound pins; and
/// both moved away from the retired contextual `0x09` values, which stay
/// retained above precisely so this movement is a checked fact rather than an
/// assurance.
#[test]
fn static_and_same_axis_live_pointwise_identities_remain_exact() {
    let static_schedule = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let static_kernel = lower_scheduled_region(&static_schedule).unwrap();
    assert_eq!(
        identity_hex(static_kernel.canonical_identity().as_bytes()),
        ABSENT_SUBGROUP_KERNEL_IDENTITY_HEX,
        "the existing all-static kernel pin must not move"
    );

    let live = live_row_major_region(2);
    let live_kernel = lower_scheduled_region(&live).unwrap();
    assert_eq!(
        identity_hex(live.canonical_identity().as_bytes()),
        LIVE_ROW_MAJOR_SCHEDULE_IDENTITY_HEX,
        "the source-bound all-live schedule bytes must match their rebaselined pin"
    );
    assert_eq!(
        identity_hex(live_kernel.canonical_identity().as_bytes()),
        LIVE_ROW_MAJOR_KERNEL_IDENTITY_HEX,
        "the source-bound all-live kernel bytes must match their rebaselined pin"
    );
    assert_ne!(
        LIVE_ROW_MAJOR_SCHEDULE_IDENTITY_HEX,
        RETIRED_CONTEXTUAL_LIVE_ROW_MAJOR_SCHEDULE_IDENTITY_HEX,
        "retiring tag 0x09 must move the live schedule identity"
    );
    assert_ne!(
        LIVE_ROW_MAJOR_KERNEL_IDENTITY_HEX, RETIRED_CONTEXTUAL_LIVE_ROW_MAJOR_KERNEL_IDENTITY_HEX,
        "retiring tag 0x09 must move the framing kernel identity"
    );

    let inner = Axis::new(1);
    for width in [PointwiseWidth::F32, PointwiseWidth::Bf16] {
        let scheduled = two_input_pointwise_builder(
            width,
            2,
            [
                LogicalAccess::LiveRowMajorSource { inner_axis: inner },
                LogicalAccess::LiveRowMajor,
            ],
            LogicalAccess::LiveRowMajor,
        )
        .build()
        .expect("one marker plus fieldless consumers remain valid");
        lower_scheduled_region(&scheduled).expect("both valid all-live widths still lower");
    }
}

/// A live row-major access may not spell its obligation as a zero range.
///
/// The correctness the `LiveExtentReach` variant bought, stated as a refusal.
/// The retired `LinearRange { element_count: 0 }` spelling said "zero in-range
/// positions" about an access that reaches the live inner extent, and the rule
/// that admitted it checked only that the number happened to be zero — so a
/// count that reached zero for an unrelated reason passed a check that had
/// examined nothing. Both live relations are covered because they are separate
/// variants and a rule can stop reaching one of them silently.
#[test]
fn a_live_row_major_access_refuses_a_zero_linear_range() {
    let inner = Axis::new(1);
    let zero = || Some(BoundsProofKind::LinearRange { element_count: 0 });
    for (overrides, label) in [
        ([zero(), None, None], "the source marker"),
        ([None, zero(), None], "the consuming read"),
        ([None, None, zero()], "the consuming write"),
    ] {
        let error = two_input_pointwise_builder_with_proofs(
            PointwiseWidth::F32,
            2,
            [
                LogicalAccess::LiveRowMajorSource { inner_axis: inner },
                LogicalAccess::LiveRowMajor,
            ],
            LogicalAccess::LiveRowMajor,
            overrides,
        )
        .build()
        .unwrap_err();
        assert_eq!(
            error
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.rule())
                .collect::<Vec<_>>(),
            ["bounds-proof"],
            "{label} must refuse a zero linear range by name: {error}"
        );
    }
}

/// The live variant is refused on a static access, which is the other
/// direction of the same exclusivity.
#[test]
fn a_static_access_refuses_the_live_extent_reach() {
    let error = two_input_pointwise_builder_with_proofs(
        PointwiseWidth::F32,
        2,
        [LogicalAccess::LinearIdentity, LogicalAccess::LinearIdentity],
        LogicalAccess::LinearIdentity,
        [Some(BoundsProofKind::LiveExtentReach), None, None],
    )
    .build()
    .unwrap_err();
    assert_eq!(
        error
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.rule())
            .collect::<Vec<_>>(),
        ["bounds-proof"],
        "a static identity access states a count it must not omit: {error}"
    );
}

/// Each fresh tag reaches the canonical bytes on a legal program: moving the
/// unique marker from the first read to the second swaps which access carries
/// `0x0A` and which carries `0x0B`, and the two verified regions must differ
/// in identity while both stay legal.
#[test]
fn the_source_marker_position_separates_live_schedule_identity() {
    let inner = Axis::new(1);
    let marker_first = two_input_pointwise_builder(
        PointwiseWidth::F32,
        2,
        [
            LogicalAccess::LiveRowMajorSource { inner_axis: inner },
            LogicalAccess::LiveRowMajor,
        ],
        LogicalAccess::LiveRowMajor,
    )
    .build()
    .expect("a first-read marker is a legal live region");
    let marker_second = two_input_pointwise_builder(
        PointwiseWidth::F32,
        2,
        [
            LogicalAccess::LiveRowMajor,
            LogicalAccess::LiveRowMajorSource { inner_axis: inner },
        ],
        LogicalAccess::LiveRowMajor,
    )
    .build()
    .expect("a second-read marker is a legal live region");
    assert_ne!(
        marker_first.canonical_identity().as_bytes(),
        marker_second.canonical_identity().as_bytes(),
        "which access is the runtime extent authority is identity-bearing"
    );
}

/// One compiled payload consumes a live input extent; baking the neighbour
/// value is a different kernel.
#[test]
fn a_live_row_major_kernel_reads_the_declared_extent_and_does_not_bake_it() {
    let scheduled = live_row_major_region(2);
    let kernel = lower_scheduled_region(&scheduled).unwrap();
    let extents: Vec<_> = kernel.input_extents().collect();
    assert_eq!(extents.len(), 1);
    assert_eq!(extents[0].access, AccessOrdinal::FIRST);
    assert_eq!(extents[0].axis, Axis::new(1));
    let baked_14 = lower_scheduled_region(&pointwise_region(
        RegionId::new(0),
        &Shape::from_dims([2, 14]),
    ))
    .unwrap();
    let baked_15 = lower_scheduled_region(&pointwise_region(
        RegionId::new(0),
        &Shape::from_dims([2, 15]),
    ))
    .unwrap();
    assert_ne!(
        kernel.canonical_identity().as_bytes(),
        baked_14.canonical_identity().as_bytes(),
        "baking N = 14 must change identity"
    );
    assert_ne!(
        baked_14.canonical_identity().as_bytes(),
        baked_15.canonical_identity().as_bytes(),
        "baking neighbouring extents must change identity"
    );
    let again = lower_scheduled_region(&live_row_major_region(2)).unwrap();
    assert_eq!(
        kernel.canonical_identity().as_bytes(),
        again.canonical_identity().as_bytes()
    );
    // Dense F32 [2, N]: semantic (row = 1, column = 0) is element N, so bytes
    // 4N. The live operand is the stride; baking 14 or 15 is a different
    // kernel, shown above.
    assert_eq!(dense_f32_row_major_bytes(1, 0, 14), 56);
    assert_eq!(dense_f32_row_major_bytes(1, 0, 15), 60);
}

fn count_element_access_placement(
    block: BlockRef<'_>,
    in_live_loop: bool,
    inside: &mut usize,
    outside: &mut usize,
) {
    for operation in block.operations() {
        match operation.view() {
            OperationView::Load { .. }
            | OperationView::GuardedLoad { .. }
            | OperationView::Store { .. } => {
                if in_live_loop {
                    *inside += 1;
                } else {
                    *outside += 1;
                }
            }
            OperationView::Predicated { body, .. } => {
                count_element_access_placement(body, in_live_loop, inside, outside);
            }
            OperationView::SerialLoop(loop_ref) => {
                count_element_access_placement(loop_ref.body(), true, inside, outside);
            }
            _ => {}
        }
    }
}

#[test]
fn every_live_row_major_element_access_is_inside_its_live_range() {
    let kernel = lower_scheduled_region(&live_row_major_region(2)).unwrap();
    let mut inside = 0;
    let mut outside = 0;
    count_element_access_placement(kernel.body(), false, &mut inside, &mut outside);
    assert_eq!(inside, 2, "the one load and one store form the census");
    assert_eq!(
        outside, 0,
        "a zero-trip live range must leave no executable element access"
    );
}

const fn dense_f32_row_major_bytes(row: u64, column: u64, inner_extent: u64) -> u64 {
    4 * (row * inner_extent + column)
}
