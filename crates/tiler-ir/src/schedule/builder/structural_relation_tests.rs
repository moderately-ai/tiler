use crate::schedule::{
    AxisDecode, broadcast_decodes_are_replicating, reindex_decodes_are_bijective,
};
use crate::shape::Shape;

/// The two admission rules, tested directly rather than through a program.
///
/// **These predicates are the region verifier's whole defence for the
/// structural relations, and the compile path cannot exercise their refusing
/// half.** `BroadcastAxisMapping` and `ReindexForm` already refuse a
/// non-widening mapping and a non-bijective form at the *semantic* boundary,
/// so no program the recognizer can build reaches these `false` returns. That
/// makes them unreachable through the compiler and still load-bearing here:
/// `tiler-ir` verifies regions from any producer, including one that builds a
/// `ScheduledRegion` by hand and submits it to `from_region`, and a rule with
/// no test is a rule that silently stops holding.
#[test]
fn the_reindex_rule_admits_a_tiling_and_refuses_everything_else() {
    let operand = Shape::from_dims([2, 3]);
    let transposed = Shape::from_dims([3, 2]);
    // The transposition: operand axis 1 takes result axis 0's window
    // (divisor 2), operand axis 0 takes result axis 1's (divisor 1). Sorted
    // by descending divisor the windows telescope 2*3 == 6 and 1*2 == 2.
    let admitted = vec![AxisDecode::read(1, 2), AxisDecode::read(2, 3)];
    assert!(reindex_decodes_are_bijective(
        &operand,
        &transposed,
        &admitted
    ));
    // Mirroring preserves bijectivity: `c -> modulus - 1 - c` is a bijection
    // of any axis onto itself, so a reversal tiles exactly what its
    // unmirrored twin does.
    let mirrored = vec![
        AxisDecode::read(1, 2),
        AxisDecode {
            divisor: 2,
            modulus: 3,
            mirrored: true,
        },
    ];
    assert!(reindex_decodes_are_bijective(
        &operand,
        &transposed,
        &mirrored
    ));

    // An overlap: both windows claim divisor 1, so two linear coordinates
    // collide on one operand element and the map is not injective.
    let overlapping = vec![AxisDecode::read(1, 2), AxisDecode::read(1, 3)];
    assert!(!reindex_decodes_are_bijective(
        &operand,
        &transposed,
        &overlapping
    ));
    // A gap: the windows are disjoint but leave the coordinate `2..6`
    // unreachable, so the map is injective and not surjective — a slice
    // rather than a reindex.
    let gapped = vec![AxisDecode::read(1, 2), AxisDecode::read(4, 3)];
    assert!(!reindex_decodes_are_bijective(
        &operand,
        &transposed,
        &gapped
    ));
    // **The telescoping rule specifically**, which needs three axes to
    // exercise: with two, a broken tiling always fails the total-window
    // check first. On a `[2, 2, 2]` operand the top window is `4 * 2 == 8`
    // and the bottom divisor is `1`, so both end checks pass — and two axes
    // still claim divisor `1`, which only the telescoping loop detects.
    let cube = Shape::from_dims([2, 2, 2]);
    let untelescoped = vec![
        AxisDecode::read(4, 2),
        AxisDecode::read(1, 2),
        AxisDecode::read(1, 2),
    ];
    assert!(!reindex_decodes_are_bijective(&cube, &cube, &untelescoped));
    // Its admitted neighbour, differing only in the middle window, so the
    // refusal above reads the overlap rather than the shape.
    let telescoped = vec![
        AxisDecode::read(4, 2),
        AxisDecode::read(2, 2),
        AxisDecode::read(1, 2),
    ];
    assert!(reindex_decodes_are_bijective(&cube, &cube, &telescoped));

    // A modulus that is not the operand axis's own extent.
    let wrong_modulus = vec![AxisDecode::read(1, 3), AxisDecode::read(2, 3)];
    assert!(!reindex_decodes_are_bijective(
        &operand,
        &transposed,
        &wrong_modulus
    ));
    // A result domain of a different size cannot be in bijection at all.
    assert!(!reindex_decodes_are_bijective(
        &operand,
        &Shape::from_dims([2, 2]),
        &admitted
    ));
    // One decode per operand axis, never fewer.
    assert!(!reindex_decodes_are_bijective(
        &operand,
        &transposed,
        &admitted[..1]
    ));
}

#[test]
fn the_broadcast_rule_requires_a_real_widening_of_named_result_axes() {
    // A `[2]` weight read across a `[2, 2]` activation: the weight's only
    // axis takes result axis 1's window, and result axis 0 is replicated.
    let operand = Shape::from_dims([2]);
    let widened = Shape::from_dims([2, 2]);
    let admitted = vec![AxisDecode::read(1, 2)];
    assert!(broadcast_decodes_are_replicating(
        &operand, &widened, &admitted
    ));

    // **The widening rule.** A replication that covers the whole result
    // domain is a dense read, and admitting it here would give one region
    // two identities.
    assert!(!broadcast_decodes_are_replicating(
        &operand,
        &Shape::from_dims([2]),
        &admitted
    ));
    // A rank that grew only by an extent-one axis widens nothing either,
    // which is why the rule is stated on element counts rather than ranks.
    assert!(!broadcast_decodes_are_replicating(
        &operand,
        &Shape::from_dims([1, 2]),
        &admitted
    ));
    // A broadcast replicates and never reverses; mirroring belongs to the
    // reindex family, and admitting it here would let one composition be
    // spelled two ways.
    let reversing = vec![AxisDecode {
        divisor: 1,
        modulus: 2,
        mirrored: true,
    }];
    assert!(!broadcast_decodes_are_replicating(
        &operand, &widened, &reversing
    ));
    // A divisor that names no whole result axis is a partial window, which
    // this relation does not admit.
    let partial = vec![AxisDecode::read(3, 2)];
    assert!(!broadcast_decodes_are_replicating(
        &operand, &widened, &partial
    ));
    // Two operand axes may not read one result axis: that is a reindex-style
    // decode of one coordinate into two, not a replication.
    let doubled = vec![AxisDecode::read(1, 2), AxisDecode::read(1, 2)];
    assert!(!broadcast_decodes_are_replicating(
        &Shape::from_dims([2, 2]),
        &Shape::from_dims([2, 2, 2]),
        &doubled
    ));
}
