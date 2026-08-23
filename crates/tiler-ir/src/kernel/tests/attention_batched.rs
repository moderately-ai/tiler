use super::super::lower_scheduled_region;
use super::support::{
    OperandLayouts, blocked_contraction_region, declared_operand_offsets, guarded_load_count,
    round_zero_operand_offsets,
};
use crate::schedule::{ContractionAxisSource, TailPolicy, VerifiedScheduledRegion};
use crate::shape::Shape;

/// Both attention structures reach a lowered, emitted, verified kernel.
///
/// **This is the ticket's closing condition**, and it is stated as a whole
/// verified kernel rather than as an absence of refusal: `lower_scheduled_region`
/// runs the refinement gate, so a body that lowered but did not re-derive
/// identically would fail here rather than pass quietly.
///
/// Both tails are exercised. The predicated one is the tail both structures
/// actually take at `T = 10`, and it is the one that puts `verify.rs`'s abstract
/// interpreter on the path: without a rank-general reading of the workgroup
/// decode, the row and column guards go unrecognized and the guarded loads are
/// refused as `left-load-guard`.
#[test]
fn both_attention_structures_lower_and_emit_through_the_batched_path() {
    let mut lowered = 0_usize;
    for structure in attention_structures() {
        for tail in [TailPolicy::Exact, TailPolicy::Predicated] {
            // The exact tail needs a block-divisible output, which `T = 10` is
            // not; the structures' real launch is the predicated one, and the
            // exact row is exercised on a divisible sibling below.
            if matches!(tail, TailPolicy::Exact) {
                continue;
            }
            let region = structure.region(tail);
            let kernel = lower_scheduled_region(&region).unwrap_or_else(|error| {
                panic!(
                    "{} {tail:?}: must lower and verify, got {error:?}",
                    structure.id
                )
            });
            // Two guarded loads per emitted tile, and a multi-round body emits
            // the tile twice: once in the peeled round zero and once in the round
            // loop's body. Derived from the round count rather than written as a
            // literal, so a structure whose contracted extent changes moves both
            // sides together instead of leaving a stale number.
            let rounds = structure.contracted / 16;
            let expected = if rounds > 1 { 4 } else { 2 };
            assert_eq!(
                guarded_load_count(&kernel),
                expected,
                "{} {tail:?}: each operand load of each emitted tile carries its own guard",
                structure.id,
            );
            lowered += 1;
        }
    }
    assert_eq!(lowered, 2, "both structures were judged");
}

/// A batched output whose trailing pair divides the block takes the exact tail.
///
/// The control for the predicated row above: without it, "the batched path
/// lowers" would rest entirely on the guarded-load machinery, and an exact
/// batched launch — which emits no guarded load at all and reaches the store
/// through the plain iteration guard — would be untested.
#[test]
fn an_exact_batched_contraction_lowers_without_guarded_loads() {
    use attention::{GROUPS, REPEATS};
    let out = |position| ContractionAxisSource::Output { position };
    let inner = ContractionAxisSource::Contracted { position: 0 };
    let region = blocked_contraction_region(
        &Shape::from_dims([GROUPS, REPEATS, 32, 48]),
        16,
        &[out(0), out(1), out(2), inner],
        &[out(0), out(3), inner],
        TailPolicy::Exact,
    );
    let kernel =
        lower_scheduled_region(&region).expect("an exact batched contraction lowers and verifies");
    assert_eq!(
        guarded_load_count(&kernel),
        0,
        "an exact launch guards nothing: every invocation owns a logical position",
    );
}

/// Every batched operand is addressed at its own declared strides.
///
/// **This is the rank-N analogue of the construction the access-map defect was
/// found by, and it exists for the same reason.** A batch coordinate is a
/// bijection of the operand's own index space just as a transposition is, so an
/// operand addressed at batch zero for every batch still lands inside its buffer
/// at a valid element: no bounds proof, no element count, and no verifier can
/// see it. Only the emitted arithmetic can, so this reads the address the kernel
/// will compute and compares it against an independently written derivation.
///
/// The invocation is chosen off every boundary — a non-zero coordinate on both
/// batch axes and inside a partial row tile — because at batch zero a dropped
/// batch term and a correct one agree.
#[test]
fn every_batched_operand_is_addressed_at_its_declared_strides() {
    let mut wrong = Vec::new();
    let mut judged = 0_usize;
    for structure in attention_structures() {
        let global = invocation_at(&structure.output, &PROBE_WORKGROUP, PROBE_LOCAL);
        let region = structure.region(TailPolicy::Predicated);
        let observed = round_zero_operand_offsets(&region, global, PROBE_LOCAL);
        let [left, right] = observed.as_slice() else {
            panic!(
                "{}: round zero loads exactly two operands, saw {observed:?}",
                structure.id
            );
        };
        let (expected_left, expected_right) = declared_blocked_offsets(
            &structure.output,
            structure.contracted,
            &structure.left,
            &structure.right,
            global,
            PROBE_LOCAL,
        );
        for (operand, observed, expected) in [
            ("left", *left, expected_left),
            ("right", *right, expected_right),
        ] {
            judged += 1;
            if observed != expected {
                wrong.push(format!(
                    "{} {operand}: read {observed}, declared {expected}",
                    structure.id
                ));
            }
        }
    }
    assert_eq!(judged, 4, "two structures, two operands each");
    assert!(wrong.is_empty(), "mis-addressed operands: {wrong:#?}");
}

/// The owning store commits to the batch coordinate's own output position.
///
/// A separate behaviour from the operand addressing above, and separately
/// perturbable: a body that carried the batch into its loads and dropped it from
/// its store would read every element correctly and commit every batch's result
/// into batch zero's positions. The `LinearIdentity` write map says the position
/// is the row-major one, so that is what this derives independently.
#[test]
fn the_owning_store_commits_at_the_batched_row_major_position() {
    let mut wrong = Vec::new();
    let mut judged = 0_usize;
    for structure in attention_structures() {
        let global = invocation_at(&structure.output, &PROBE_WORKGROUP, PROBE_LOCAL);
        let region = structure.region(TailPolicy::Predicated);
        let observed = owning_store_offset(&region, global, PROBE_LOCAL);
        // The row-major position of the output coordinate this invocation owns.
        // The batch coordinates are the workgroup's own — a workgroup covers one
        // coordinate on each batch axis — and the trailing pair is the tile
        // origin plus the participant's position within it.
        let coordinate = [
            PROBE_WORKGROUP[0],
            PROBE_WORKGROUP[1],
            PROBE_WORKGROUP[2] * 16 + PROBE_LOCAL / 16,
            PROBE_WORKGROUP[3] * 16 + PROBE_LOCAL % 16,
        ];
        let expected = linear_position(&coordinate, &structure.output);
        judged += 1;
        if observed != expected {
            wrong.push(format!(
                "{}: stored at {observed}, declared {expected}",
                structure.id
            ));
        }
    }
    assert_eq!(judged, 2, "both structures were judged");
    assert!(wrong.is_empty(), "mis-placed owning stores: {wrong:#?}");
}

/// The two independent declared-address derivations agree where both apply.
///
/// `declared_operand_offsets` is the access-map lane's rank-two derivation and
/// `declared_blocked_offsets` is this lane's rank-general one. They are kept
/// separate on purpose — two derivations that cannot share a mistake are worth
/// more than one — and this is what turns that redundancy into a cross-check
/// rather than a place for the two to drift apart unnoticed.
#[test]
fn the_two_declared_derivations_agree_at_rank_two() {
    const OUTPUT_M: u64 = 32;
    const OUTPUT_N: u64 = 48;
    const CONTRACTED: u64 = 16;
    const GLOBAL: u64 = 4 * 256 + 37;
    const LOCAL: u64 = 37;
    let out = |position| ContractionAxisSource::Output { position };
    let inner = ContractionAxisSource::Contracted { position: 0 };
    let mut judged = 0_usize;
    for left_transposed in [false, true] {
        for right_transposed in [false, true] {
            let layouts = OperandLayouts {
                left_transposed,
                right_transposed,
            };
            let operand = |position, transposed| {
                if transposed {
                    vec![inner, out(position)]
                } else {
                    vec![out(position), inner]
                }
            };
            let general = declared_blocked_offsets(
                &[OUTPUT_M, OUTPUT_N],
                CONTRACTED,
                &operand(0, left_transposed),
                &operand(1, right_transposed),
                GLOBAL,
                LOCAL,
            );
            let specific =
                declared_operand_offsets(OUTPUT_M, OUTPUT_N, CONTRACTED, layouts, GLOBAL, LOCAL);
            assert_eq!(
                [general.0, general.1],
                specific,
                "{layouts:?}: the two derivations must agree at rank two",
            );
            judged += 1;
        }
    }
    assert_eq!(judged, 4, "all four rank-two layouts were compared");
}

/// The two attention structures, as the L4 design states them.
///
/// `grtd,gsd->grts` and `grts,gsd->grtd`. Both produce a rank-four output whose
/// trailing two axes carry the participants, and both have a right operand that
/// reads the group and never the repetition — the key and value are *shared*
/// across the grouped-query repeats rather than broadcast into them, which is
/// what makes an unread batch axis part of the subject rather than an edge case.
///
/// The extents are pairwise distinct and every stride they generate is distinct,
/// so a term that took the wrong axis's stride cannot compare equal to the right
/// one by coincidence. `T = 10` does not divide the block, so both structures
/// take the predicated tail — which is what puts the abstract interpreter's
/// row and column guards on the path rather than beside it.
mod attention {
    /// Key/value groups.
    pub(super) const GROUPS: u64 = 8;
    /// Grouped-query repetition within a group.
    pub(super) const REPEATS: u64 = 2;
    /// Query positions. Deliberately not a multiple of the 16-wide block.
    pub(super) const QUERIES: u64 = 10;
    /// Key/value positions.
    pub(super) const CONTEXT: u64 = 32;
    /// The head lane.
    pub(super) const HEAD_DIM: u64 = 128;
}

/// One attention structure's output shape and its two operands' declared sources.
struct AttentionStructure {
    id: &'static str,
    output: Vec<u64>,
    contracted: u64,
    left: Vec<ContractionAxisSource>,
    right: Vec<ContractionAxisSource>,
}

fn attention_structures() -> Vec<AttentionStructure> {
    use attention::{CONTEXT, GROUPS, HEAD_DIM, QUERIES, REPEATS};
    let out = |position| ContractionAxisSource::Output { position };
    let inner = ContractionAxisSource::Contracted { position: 0 };
    vec![
        AttentionStructure {
            // Query `[g, r, t, d]` against key `[g, s, d]`, contracting the head
            // lane. The key's contracted axis is its trailing one.
            id: "score grtd,gsd->grts",
            output: vec![GROUPS, REPEATS, QUERIES, CONTEXT],
            contracted: HEAD_DIM,
            left: vec![out(0), out(1), out(2), inner],
            right: vec![out(0), out(3), inner],
        },
        AttentionStructure {
            // Scores `[g, r, t, s]` against value `[g, s, d]`, contracting the
            // context. **The value's contracted axis sits in the middle**, which
            // is the orientation no hardcoded `[N, K]` addressing can express.
            id: "value grts,gsd->grtd",
            output: vec![GROUPS, REPEATS, QUERIES, HEAD_DIM],
            contracted: CONTEXT,
            left: vec![out(0), out(1), out(2), inner],
            right: vec![out(0), inner, out(3)],
        },
    ]
}

impl AttentionStructure {
    fn region(&self, tail: TailPolicy) -> VerifiedScheduledRegion {
        blocked_contraction_region(
            &Shape::try_new(
                self.output
                    .iter()
                    .map(|extent| crate::shape::Extent::new(*extent))
                    .collect::<Vec<_>>(),
            )
            .expect("an attention output is representable"),
            self.contracted,
            &self.left,
            &self.right,
            tail,
        )
    }
}

/// Row-major suffix products: the element stride of each axis.
fn strides_of(extents: &[u64]) -> Vec<u64> {
    let mut strides = vec![1_u64; extents.len()];
    for axis in (0..extents.len().saturating_sub(1)).rev() {
        strides[axis] = strides[axis + 1] * extents[axis + 1];
    }
    strides
}

/// The row-major linear position of one per-axis coordinate.
fn linear_position(coordinate: &[u64], extents: &[u64]) -> u64 {
    coordinate
        .iter()
        .zip(strides_of(extents))
        .map(|(value, stride)| value * stride)
        .sum()
}

/// Per-axis workgroup counts of a batched blocked launch.
///
/// One workgroup per coordinate on each batch axis — the block extent there is
/// one — and the ceiling quotient on the participants' trailing pair.
fn workgroup_counts(output: &[u64]) -> Vec<u64> {
    let row_axis = output.len() - 2;
    output
        .iter()
        .enumerate()
        .map(|(axis, extent)| {
            if axis < row_axis {
                *extent
            } else {
                extent.div_ceil(16)
            }
        })
        .collect()
}

/// The global invocation landing on one workgroup coordinate and one participant.
///
/// Written as a derivation over the per-axis counts rather than as a literal, so
/// a changed extent moves it instead of leaving it pointing at a different
/// workgroup than its name claims.
fn invocation_at(output: &[u64], workgroup_coordinate: &[u64], local: u64) -> u64 {
    linear_position(workgroup_coordinate, &workgroup_counts(output)) * 256 + local
}

/// The workgroup this lane's batched probes land on: `(5, 1, 0, 1)`.
///
/// Both batch coordinates are non-zero, which is what makes a dropped batch term
/// visible — at batch zero a dropped term and a correct one agree. The row tile
/// is zero and the column tile is one, so the row and column contributions stay
/// distinguishable from each other too.
const PROBE_WORKGROUP: [u64; 4] = [5, 1, 0, 1];

/// The participant the batched probes read: `(m, n) = (2, 5)` of the 16x16 block.
const PROBE_LOCAL: u64 = 37;

/// The addresses the *declared* maps say one blocked operand load must read.
///
/// Rank-general, and derived here from the declaration alone — the workgroup
/// decode is written out directly rather than obtained from the lowering's own
/// term builder — so the two derivations are independent and a shared mistake
/// cannot make them agree. `declared_operand_offsets` is the rank-two
/// derivation the access-map lane wrote; the two are deliberately *not* merged,
/// and [`the_two_declared_derivations_agree_at_rank_two`] holds them against each
/// other on the shapes they both cover.
///
/// Returns `(left, right)` in emission order.
fn declared_blocked_offsets(
    output: &[u64],
    contracted: u64,
    left_sources: &[ContractionAxisSource],
    right_sources: &[ContractionAxisSource],
    global: u64,
    local: u64,
) -> (u64, u64) {
    const BLOCK: u64 = 16;
    let row_axis = output.len() - 2;
    let column_axis = row_axis + 1;
    // Per-axis workgroup counts: one per coordinate on a batch axis, and the
    // ceiling quotient on the participants' pair.
    let workgroups: Vec<u64> = output
        .iter()
        .enumerate()
        .map(|(axis, extent)| {
            if axis < row_axis {
                *extent
            } else {
                extent.div_ceil(BLOCK)
            }
        })
        .collect();
    let workgroup = global / (BLOCK * BLOCK);
    let workgroup_strides = strides_of(&workgroups);
    // The row-major decode of the linear workgroup index.
    let wg_coord: Vec<u64> = (0..output.len())
        .map(|axis| (workgroup / workgroup_strides[axis]) % workgroups[axis])
        .collect();
    let row = wg_coord[row_axis] * BLOCK + local / BLOCK;
    let column = wg_coord[column_axis] * BLOCK + local % BLOCK;
    // Participant `(m, n)` fetches the left tile's column `n` and the right
    // tile's column `m`, which is the tile's staging relation.
    let offset = |sources: &[ContractionAxisSource], contracted_coordinate: u64| {
        let extents: Vec<u64> = sources
            .iter()
            .map(|source| match source {
                ContractionAxisSource::Output { position } => output[*position as usize],
                ContractionAxisSource::Contracted { .. } => contracted,
            })
            .collect();
        let strides = strides_of(&extents);
        sources
            .iter()
            .enumerate()
            .map(|(axis, source)| {
                let coordinate = match source {
                    ContractionAxisSource::Output { position } => {
                        let position = *position as usize;
                        if position == row_axis {
                            row
                        } else if position == column_axis {
                            column
                        } else {
                            wg_coord[position]
                        }
                    }
                    ContractionAxisSource::Contracted { .. } => contracted_coordinate,
                };
                coordinate * strides[axis]
            })
            .sum()
    };
    (
        offset(left_sources, local % BLOCK),
        offset(right_sources, local / BLOCK),
    )
}

/// The linear position the owning store commits to, for one invocation.
///
/// Walks into the predicated blocks the store sits inside, which
/// [`round_zero_operand_offsets`] deliberately does not: the operand loads are
/// at the top level and the store is not, and the batch contribution to the
/// *store* address is a separate behaviour from its contribution to the *load*
/// addresses. A body that carried the batch into its loads and dropped it from
/// its store would read the right elements and commit every batch's result to
/// batch zero's positions, and only this reads that.
fn owning_store_offset(scheduled: &VerifiedScheduledRegion, global: u64, local: u64) -> u64 {
    let data = super::super::lower::derive_canonical(
        scheduled.region(),
        scheduled.canonical_identity(),
        scheduled.requirements(),
    )
    .expect("the canonical body exists");
    let mut values: std::collections::BTreeMap<u32, u64> = std::collections::BTreeMap::new();
    let mut store = None;
    walk_index_arithmetic(&data, 0, global, local, &mut values, &mut store);
    store.expect("a verified body commits exactly one owning store")
}

/// Records one index-producing operation's result, when it has exactly one.
fn define_index(values: &mut std::collections::BTreeMap<u32, u64>, results: &[u32], value: u64) {
    if let [result] = results {
        values.insert(*result, value);
    }
}

/// Interprets one block's index arithmetic, descending into guarded regions.
///
/// The round loop is deliberately not walked: its induction variable has no
/// single value, and the owning store does not sit inside it.
fn walk_index_arithmetic(
    data: &super::super::model::KernelData,
    block: u32,
    global: u64,
    local: u64,
    values: &mut std::collections::BTreeMap<u32, u64>,
    store: &mut Option<u64>,
) {
    use super::super::model::{BinaryOp, Builtin, KernelConstant, OperationKind};
    let Some(block_data) = data.blocks.get(block as usize) else {
        return;
    };
    for operation in &block_data.operations {
        match &operation.kind {
            OperationKind::Builtin { builtin } => {
                let value = match builtin {
                    Builtin::GlobalInvocationIndex => global,
                    Builtin::LocalInvocationIndex => local,
                };
                define_index(values, &operation.results, value);
            }
            OperationKind::Constant {
                value: KernelConstant::Index(value),
            } => define_index(values, &operation.results, *value),
            OperationKind::Binary { op, lhs, rhs } => {
                let (Some(lhs), Some(rhs)) = (values.get(lhs).copied(), values.get(rhs).copied())
                else {
                    continue;
                };
                let value = match op {
                    BinaryOp::IndexAdd => lhs + rhs,
                    BinaryOp::IndexSubtract => lhs - rhs,
                    BinaryOp::IndexMultiply => lhs * rhs,
                    BinaryOp::IndexDivide => lhs / rhs,
                    BinaryOp::IndexModulo => lhs % rhs,
                    _ => continue,
                };
                define_index(values, &operation.results, value);
            }
            OperationKind::Predicated { body, .. } => {
                walk_index_arithmetic(data, *body, global, local, values, store);
            }
            OperationKind::Store { offset, .. } => {
                *store = Some(*values.get(offset).expect(
                    "the owning store's offset is index arithmetic over the launch builtins",
                ));
            }
            _ => {}
        }
    }
}
