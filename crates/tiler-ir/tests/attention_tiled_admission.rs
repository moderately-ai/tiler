//! The `tiled` realization's reach over the two attention contractions.
//!
//! # What this file decides, and what it deliberately does not
//!
//! [`realize-the-attention-contractions-on-metal`](../../../tickets/realize-the-attention-contractions-on-metal.md)
//! requires the tiled realization to be *"gated on its own precondition as a
//! typed refusal"*, with that refusal *"demonstrated firing at `S = 10`"* —
//! because a precondition that has never rejected anything is not a
//! precondition. This file is that demonstration, and it turns the
//! admissibility table the [L4 design](../../../docs/research/program-planning/first-attention-program-vertical.md)
//! states into a checked property rather than a prose claim.
//!
//! It establishes nothing about a kernel, a device, or a cost. The refusals here
//! are the *schedule admission* authority's — `admit_exact_cooperative_contraction`
//! and `admit_predicated_cooperative_contraction` — which is the layer that owns
//! the `K ≡ 0 (mod tile)` relation.
//!
//! # The second wall this file records, which is not a precondition
//!
//! The tiled realization's contracted-extent precondition is one reason it may
//! decline an attention contraction. There is a second, independent one, and
//! conflating them would misreport what blocks this work: the cooperative
//! contraction's *participant space* is rank two, and both attention structures
//! produce a rank-four output.
//!
//! `blocked_operand_tile` — the one tile shape three layers construct — builds a
//! rank-two participant space, stating in its own source that this is deliberate
//! (*"Rank two, deliberately"*). It stays rank two for a reason no widening
//! reaches: `MAX_COOPERATIVE_PARTICIPANT_RANK` is three, so a rank-four space is
//! unrepresentable rather than merely unimplemented.
//!
//! **A rank-four output is nonetheless reachable, by giving the *block* the
//! output's rank rather than the participants.** `participant_space_matches_block`
//! couples the space to the block's **trailing two axes** and requires every
//! leading extent to be one, so one workgroup covers one batch coordinate and a
//! `16x16` tile of the trailing pair.
//! [`a_batched_block_covers_every_output_position_of_both_attention_structures`]
//! is that admission, and it is stated as a cover rather than as an acceptance:
//! the launch is proved to reach every logical output position.
//!
//! **What is still refused, and separately named.** A rank-*two* block against a
//! rank-four output remains `OutputBlockRankMismatch`, because that pairing is a
//! genuine mismatch and not a batched block —
//! [`the_accepted_blocked_tile_cannot_cover_a_rank_four_attention_output`] pins
//! it. And the kernel lowering's `cooperative_contraction_plan` still refuses any
//! `iteration_shape` whose rank is not two, so the batched form is admitted by the
//! schedule layer and not yet lowered or emitted; that remainder is
//! [`admit-a-batched-cooperative-contraction-for-the-attention-structures`](../../../tickets/admit-a-batched-cooperative-contraction-for-the-attention-structures.md).
//!
//! None of this is the `K ≡ 0 (mod 16)` precondition and it must not be reported
//! as it. The contracted-extent refusal is a property of one row's `S`; the rank
//! relation is a property of the realization's vocabulary and holds at every row
//! of both structures, including the ones the table below marks admissible.

use tiler_ir::schedule::{
    CooperativeContractionAdmission, admit_exact_cooperative_contraction,
    admit_predicated_cooperative_contraction, blocked_operand_tile, prove_blocked_predicated_cover,
};
use tiler_ir::shape::Shape;

/// The tile width the first-Metal-contraction record measured.
///
/// **Named as a measurement, not as an authority.** No target profile declares a
/// contraction tile-width policy at this base, and
/// [`decide-the-contraction-tile-width-authority`](../../../tickets/decide-the-contraction-tile-width-authority.md)
/// resolved to declare none until a sweep exists. This constant is the width the
/// retained record's `contract_tiled` kernel was compiled with, used here to ask
/// what the admission authority says about a given contracted extent — never to
/// select a width for a plan.
const MEASURED_TILE: u64 = 16;

/// Key/value groups, grouped-query repetition, and head lane, from the pinned
/// workload. `GROUPS * REPEATS` is the 16 query heads.
const GROUPS: u64 = 8;
const REPEATS: u64 = 2;
const HEAD_DIM: u64 = 128;

/// Asks the admission authority whether a contracted extent divides the tile.
///
/// The output block is held exactly divisible so the *only* variable is the
/// contracted axis. Without that isolation a refusal could be the output
/// block's and would be reported as the contracted extent's.
fn contracted_extent_admits(contracted: u64) -> Result<(), CooperativeContractionAdmission> {
    admit_exact_cooperative_contraction(
        &Shape::from_dims([MEASURED_TILE, MEASURED_TILE]),
        &Shape::from_dims([MEASURED_TILE, MEASURED_TILE]),
        &Shape::from_dims([contracted]),
        &Shape::from_dims([MEASURED_TILE]),
    )
    .map(|_| ())
}

/// The value contraction's precondition refuses the conformance row, naming the
/// precondition and the observed extent.
///
/// Structure 3 (`grts,gsd->grtd`) contracts over `S`, which is 10 at the C1
/// prefill row. This is the refusal the ticket requires to have been watched
/// firing before the precondition is trusted, and it is asserted as an exact
/// typed value rather than through a string, so a refusal that changed to a
/// different rule could not satisfy it.
#[test]
fn the_tiled_precondition_refuses_the_conformance_rows_value_contraction() {
    /// C1 prefill contracts structure 3 over `S = T = 10`.
    const C1_PREFILL_CONTEXT: u64 = 10;

    let refusal = contracted_extent_admits(C1_PREFILL_CONTEXT)
        .expect_err("10 is not a multiple of 16, so the tiled realization must refuse it");

    assert_eq!(
        refusal,
        CooperativeContractionAdmission::ContractedTileNotDivisible {
            axis: 0,
            contracted: C1_PREFILL_CONTEXT,
            tile: MEASURED_TILE,
        },
        "the refusal must name the realization's precondition and the observed extent",
    );
}

/// The score contraction's contracted extent is the static head lane, so its
/// precondition holds at every row of both workloads.
///
/// This is the control for the refusal above: without it, "the tiled realization
/// refuses the attention contractions" would be consistent with a precondition
/// that rejects everything.
#[test]
fn the_score_structures_contracted_extent_admits_at_every_row() {
    assert_eq!(
        contracted_extent_admits(HEAD_DIM),
        Ok(()),
        "structure 2 contracts over the static {HEAD_DIM}, which is a multiple of {MEASURED_TILE}",
    );
}

/// The admissibility table the L4 design states, checked row by row.
///
/// Structure 3's contracted extent is `S`, so `tiled`'s admissibility is decided
/// per binding rather than once. Each row below is the design document's own
/// claim, re-derived here against the admission authority rather than restated.
#[test]
fn the_tiled_admissibility_table_holds_at_every_row_the_design_states() {
    // C1 prefill: `S = 10`, refused. Covered in full by the test above; repeated
    // here so the table is complete where a reader looks for it.
    assert!(contracted_extent_admits(10).is_err(), "C1 prefill, S = 10");

    // C1 decode, steps 1..=8, reaching contexts 11 through 18: admissible at
    // exactly one, `S = 16`.
    let c1_decode_admissible: Vec<u64> = (11..=18)
        .filter(|context| contracted_extent_admits(*context).is_ok())
        .collect();
    assert_eq!(
        c1_decode_admissible,
        vec![16],
        "across C1's decode contexts the tiled realization admits only S = 16",
    );

    // B1-a prefill: `S = 128`, admissible.
    assert_eq!(
        contracted_extent_admits(128),
        Ok(()),
        "B1-a prefill, S = 128"
    );

    // B1-a decode, contexts 129 through 256: the design states 8 of the 128
    // steps. Counted rather than asserted, so a changed tile width would move
    // this number instead of silently keeping a stale one.
    let b1a_decode_admissible = (129..=256)
        .filter(|context| contracted_extent_admits(*context).is_ok())
        .count();
    assert_eq!(
        b1a_decode_admissible, 8,
        "across B1-a's 128 decode steps the tiled realization admits 8",
    );

    // B1-d prefill: `S = 8,192`, admissible.
    assert_eq!(
        contracted_extent_admits(8_192),
        Ok(()),
        "B1-d prefill, S = 8,192"
    );
}

/// No contracted extent is ever padded up to the tile width.
///
/// The refusal above is the *correct* outcome rather than a gap to close.
/// Padding structure 3's contracted extent would add contributors of the form
/// `+0.0 x v`, whose sign follows `v`, to a fold whose seed is its first product
/// rather than `+0.0` — so the padded contributors are exactly the signed zeros
/// the mask already contributes, and they change the result. That is the
/// neutrality obligation `docs/numerical-semantics.md` would require a padding
/// schedule to discharge, and this realization declines to acquire it.
///
/// Asserted structurally: every extent between two tile multiples refuses, so
/// there is no extent the authority quietly rounds.
#[test]
fn no_contracted_extent_is_rounded_up_to_the_tile_width() {
    for contracted in (MEASURED_TILE + 1)..(2 * MEASURED_TILE) {
        let refusal = contracted_extent_admits(contracted)
            .expect_err("an extent strictly between tile multiples must refuse, never round");
        assert_eq!(
            refusal,
            CooperativeContractionAdmission::ContractedTileNotDivisible {
                axis: 0,
                contracted,
                tile: MEASURED_TILE,
            },
        );
    }
}

/// The accepted blocked tile is rank two and cannot cover a rank-four attention
/// output, at any extent.
///
/// This is the wall that is *not* the contracted-extent precondition, and the
/// reason "`tiled` for both structures" is unmet at this base for a reason no
/// choice of `S` repairs. `blocked_operand_tile` is the one constructor the
/// schedule, kernel, and Metal layers share, and it builds a rank-two
/// participant space; `verify_cooperative_contraction` requires the binding's
/// block to match that space's rank. Both attention structures produce a
/// rank-four output — `[g, r, t, s]` and `[g, r, t, d]` — so the admission
/// authority refuses on rank before any extent is examined.
#[test]
fn the_accepted_blocked_tile_cannot_cover_a_rank_four_attention_output() {
    let tile = blocked_operand_tile(MEASURED_TILE, 1).expect("the measured tile is constructible");
    assert_eq!(
        tile.coordinates.participants.rank(),
        2,
        "the one shared blocked tile constructor states a rank-two participant space",
    );

    // Structure 2's C1 prefill result, `[g, r, t, s]`, and structure 3's,
    // `[g, r, t, d]`. Both rank four.
    for (id, output) in [
        (
            "score grtd,gsd->grts",
            Shape::from_dims([GROUPS, REPEATS, 10, 10]),
        ),
        (
            "value grts,gsd->grtd",
            Shape::from_dims([GROUPS, REPEATS, 10, HEAD_DIM]),
        ),
    ] {
        assert_eq!(output.rank(), 4, "{id}: an attention result is rank four");
        let refusal = admit_predicated_cooperative_contraction(
            &output,
            // The block the rank-two participant space above forces.
            &Shape::from_dims([MEASURED_TILE, MEASURED_TILE]),
            &Shape::from_dims([HEAD_DIM]),
            &Shape::from_dims([MEASURED_TILE]),
        )
        .expect_err("a rank-four output cannot take a rank-two block");
        assert_eq!(
            refusal,
            CooperativeContractionAdmission::OutputBlockRankMismatch {
                output_rank: 4,
                block_rank: 2,
            },
            "{id}: the refusal must name the rank mismatch, not the contracted extent",
        );
    }
}

/// A rank-four block whose batch axes are one-wide admits both structures, and
/// its launch genuinely covers every logical output position.
///
/// # Why this is the theorem and the deleted-guard experiment was not
///
/// `realize-the-attention-contractions-on-metal` deleted the rank guard from
/// `ceiling_quotients` and got
/// `PredicatedCooperativeContraction { work_items: 1600, grid_threads: 256 }`
/// for structure 2's C1 prefill output — a launch covering 256 of 1,600
/// positions, admitted as proven. That is what relaxing the rank check buys.
///
/// Carrying the batch axes *as batch axes* instead gives the block the output's
/// own rank, and the quotients are then per-axis: the batch axes get one
/// workgroup each because their block extent is one, and only the trailing pair
/// ceilings. The launch is `4,096` threads against `1,600` logical positions,
/// which is a cover with an inactive remainder rather than a shortfall — and
/// [`prove_blocked_predicated_cover`] accepts it, where it would refuse the
/// `256` above on [`MappingGap`](tiler_ir::schedule::BlockedWorkgroupRule).
///
/// The rank refusal above is untouched by this: a rank-two block against a
/// rank-four output is still `OutputBlockRankMismatch`, because that pairing is
/// a genuine mismatch rather than a batched block.
#[test]
fn a_batched_block_covers_every_output_position_of_both_attention_structures() {
    // One workgroup per `(g, r)` pair, and a `16x16` tile over the trailing two
    // axes. The leading extents are one, which is what makes the participants'
    // rank-two space a bijection onto the block's positions.
    let block = Shape::from_dims([1, 1, MEASURED_TILE, MEASURED_TILE]);

    for (id, output, contracted) in [
        (
            "score grtd,gsd->grts",
            Shape::from_dims([GROUPS, REPEATS, 10, 10]),
            HEAD_DIM,
        ),
        (
            "value grts,gsd->grtd",
            Shape::from_dims([GROUPS, REPEATS, 10, HEAD_DIM]),
            // Structure 3 contracts over `S`, so this row is the admissible
            // `S = 128` one; `S = 10` stays refused by the contracted-extent
            // precondition, which the batched block does not touch.
            128,
        ),
    ] {
        let admitted = admit_predicated_cooperative_contraction(
            &output,
            &block,
            &Shape::from_dims([contracted]),
            &Shape::from_dims([MEASURED_TILE]),
        )
        .unwrap_or_else(|refusal| panic!("{id}: a batched block must admit, got {refusal:?}"));

        let tiler_ir::schedule::ExecutionBinding::BlockedWorkgroup { block, workgroups } =
            &admitted.binding
        else {
            panic!("{id}: the admission returns a blocked binding");
        };

        // The batch axes take one workgroup per coordinate and the trailing pair
        // ceilings, which is the whole content of "carried as batch axes".
        let expected_workgroups: Vec<u64> = output
            .extents()
            .iter()
            .zip(block.extents())
            .map(|(extent, block)| extent.get().div_ceil(block.get()))
            .collect();
        let observed: Vec<u64> = workgroups.extents().iter().map(|e| e.get()).collect();
        assert_eq!(observed, expected_workgroups, "{id}: per-axis quotients");

        // The launch covers every logical position. Stated as an inequality
        // against the counted population rather than as a literal, so a changed
        // tile width moves both sides together instead of leaving a stale number.
        let logical: u64 = output.extents().iter().map(|e| e.get()).product();
        assert_eq!(admitted.work_items, logical, "{id}: work items are logical");
        assert!(
            admitted.grid_threads >= logical,
            "{id}: launch {} must cover {logical} positions",
            admitted.grid_threads,
        );

        let threads = u32::try_from(block.extents().iter().map(|e| e.get()).product::<u64>())
            .expect("the block population fits u32");
        prove_blocked_predicated_cover(
            &output,
            block,
            workgroups,
            admitted.work_items,
            threads,
            admitted.grid_threads,
        )
        .unwrap_or_else(|rule| panic!("{id}: the batched cover must prove, got {rule:?}"));
    }
}
