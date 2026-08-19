//! One complete causal self-attention block, from the residual stream in to the
//! residual stream out, as a single verified semantic program with three ordered
//! named outputs.
//!
//! # What this is
//!
//! The [L4 attention program vertical](../../../docs/research/program-planning/first-attention-program-vertical.md)
//! writes the pinned checkpoint's attention half down as twenty-two numbered
//! steps over exact shapes. This file builds those steps through the public
//! semantic builder, at the C1 conformance row's prefill extents, and turns the
//! design's table into a program that verifies, refuses its named neighbours, and
//! reference-evaluates.
//!
//! A *step* is the design's unit of exposition and not a semantic occurrence.
//! Steps 1, 8 and 9 each carry a weight broadcast, steps 10 and 11 are ten
//! occurrences each, and steps 12, 16, 17 and 20 are compositions, so the graph
//! carries **forty-eight** occurrences.
//! [`the_block_verifies_at_the_c1_prefill_shape`] counts them by key.
//!
//! Nothing here registers an operation, adds a form, or admits a structure.
//! Every occurrence is one of the eight already-registered keys —
//! `tiler::rms-norm-f32@1`, `tiler::tensor-contraction-f32@1`,
//! `tiler::reindex-f32@1`, `tiler::broadcast-f32@2`, `tiler::multiply-f32@1`,
//! `tiler::add-f32@1`, `tiler::constant-f32@1`, and `tiler::softmax-f32@1` — and
//! the block is a *shape* over them.
//!
//! # Three ordered named outputs, which are the point
//!
//! `h_out` is the block's observable result; `k_rope` and `v_heads` are the values
//! a KV cache would retain. Naming the second and third as program results rather
//! than leaving them internal is the entire seam a decode step attaches to: a
//! single-output framing would force the autoregressive-state work either to
//! recompute them or to reach inside the block, and both are the collapse the
//! multi-result rule exists to prevent. Nothing here implements a cache.
//!
//! # `T` and `S` are separate bounded symbolic extents
//!
//! At batch-1 prefill `S = T`, and they are still two symbols. The block's shapes
//! are read out of a [`ShapeEnv`] that declares, binds, and bounds each
//! separately, and nothing joins their equality classes —
//! [`the_two_extent_symbols_are_never_proved_equal`]. Changing the *row* is then
//! a binding change and not a graph change:
//! [`a_longer_row_changes_no_occurrence`] builds at ten and at eighteen positions
//! and finds every operation key and attribute identical while the extents move.
//!
//! **The direction that does not work is worth stating, because it is the seam.**
//! Binding `S` wider than `T` alone does *not* produce a decode step, and this
//! block refuses it: the prefill block computes its own key from its own input,
//! so the score tensor's key extent is whatever the key path produced, and a mask
//! asserting a wider context disagrees with it at the mask add
//! ([`a_context_wider_than_the_new_positions_is_refused_at_prefill`]). A decode
//! step is this program with `k_rope` and `v_heads` arriving as *inputs* of extent
//! `S >= T` instead of being produced — which is precisely why they are named
//! outputs here, and which is the autoregressive-state work rather than this
//! file's.
//!
//! A semantic value fact carries a *static* extent, so the environment's role
//! here is to be the authority the static shapes are derived from and the place
//! an unbounded extent is refused — [`resolve_static_extent`] declines a symbol
//! with no proved upper bound rather than compiling a generic program, which is
//! the L4 record's fourth feasibility predicate.
//!
//! # Semantics that are not style
//!
//! - **The scale multiplies the score, not an operand.** Operation 16 sits
//!   between the score contraction and the mask add. The probe measures that
//!   pre-scaling the query changes 1,404 of the 1,600 C1 score elements, so the
//!   scale's graph position is semantics and a rewrite that moved it onto
//!   `q_grouped` would be a value change.
//! - **Every broadcast is explicit.** The IR admits no implicit rank padding and
//!   no extent-one stretching, so each of the five mappings below names one
//!   source per result axis. The weight mappings and the mask mapping use
//!   `Replicate` — a rank pad, the operand has no such axis — while the rotary
//!   sign's second axis uses `StretchUnit`, because that axis exists with extent
//!   one. They are different relations, not two spellings of one.
//! - **The mask is an F32 program input**, `[T, S]`, broadcast over the two head
//!   axes and added. The derived-predicate alternative needs a boolean dtype the
//!   registry does not admit and an index-domain comparison ADR 0084's vocabulary
//!   excludes by construction.
//! - **The `[2, 1]` rotary sign is an input** because `tiler::constant-f32@1`
//!   produces rank zero only, so a two-element dense constant is inexpressible.
//!
//! # Where the compared bits come from, and the boundary
//!
//! Two independent sources, kept apart.
//!
//! **The pinned reference's own bits.** The [attention-block probe]'s retained
//! record `results/2026-07-31-c1-attention-block-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv`
//! holds a four-step chain for query head 0 at query position 2 of the C1 prefill
//! score tensor — `row_h0_t2_scores_raw`, `_scaled`, `_masked`, and `_probs` — and
//! that chain is reproducible in tree without the probe's `torch` seed, because
//! the raw scores are given. [`the_pinned_c1_score_row_is_reproduced_bit_for_bit`]
//! drives those ten payloads through this block's own operations 16, 17 and 18 at
//! the C1 row's own width and requires the other three rows back exactly. The
//! same record supplies the masked-position case
//! ([`a_masked_position_contributes_a_signed_zero_to_the_value_contraction`]) and
//! the mask's two fill values.
//!
//! **An independent recomputation**, for the whole-block evaluation. The
//! expectation is written out from the operation table by explicit coordinate
//! arithmetic — every access relation, every coordinate map, and every broadcast
//! restated rather than re-run. Its independence boundary is stated rather than
//! implied: the scalar arithmetic of the two non-linear families is the crate's
//! own certified [`rms_norm_f32`] and [`softmax_f32`], so the comparison
//! discriminates a wrong index binding, a wrong coordinate map, a wrong head
//! pairing, or a wrong composition order, and is silent about the binary32
//! results of `rsqrt` and `exp` themselves — which the RMS-normalization and
//! softmax family corpora own.
//!
//! # The whole block evaluates at the C1 row's own model dimension
//!
//! It did not always. A fold of more than 16,777,216 multiply-accumulate steps is
//! more than one uninterrupted walk of a contraction's iteration space may cost,
//! and at the C1 prefill row the query projection and the output projection are
//! 20,971,520 steps each — so a default evaluator refuses both, and
//! [`the_reference_work_bound_refuses_the_c1_projections`] still watches that
//! refusal and quotes it verbatim.
//!
//! What the end-to-end evaluation below does is **state the allowance** rather
//! than reduce an extent: [`ReferenceEvaluator::with_iteration_step_allowance`]
//! takes [`C1_LARGEST_FOLD`], the block's own largest fold computed from its own
//! extents, and an occurrence over one window is then folded in several windows
//! each of which passes exactly the test a single-window fold passes. No bound
//! moved, and the number is the block's arithmetic rather than a round figure —
//! an occurrence needing one step more would be refused.
//!
//! So the block **verifies and evaluates** at the C1 row's exact extents: ten new
//! positions, ten context positions, a 1,024-wide model dimension, sixteen query
//! heads over eight groups, head dimension 128, the same mask, the same scale, the
//! same rotary composition, and all three contraction index structures. Nothing is
//! reduced, and no extent differs from the row.
//!
//! The two large projections are separately evidenced against a device at exactly
//! these extents: `w_prefill_q` in `contraction_profile_cells.rs` is this block's
//! operation 2 at this row, and reproduces a digest an Apple M4 Max produced by
//! two routes — the staged contraction type, and the same whole-program evaluator
//! this file drives.
//!
//! This file establishes nothing about a plan, a schedule, a cover, a kernel, a
//! device, or any block-level numeric tolerance; none is exercised, and the last
//! is deliberately not composable from per-operation tolerances.
//!
//! [attention-block probe]: ../../../spikes/program-planning/attention-block-reference/README.md

use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::semantic::{
    BROADCAST_AXIS_MAPPING_ATTRIBUTE, BROADCAST_MAPPING_SOURCES, BroadcastAxisMapping,
    BroadcastAxisSource, BuildError, CanonicalValueView, ContractionIndex,
    ContractionIndexStructure, F32, F32Add, F32Broadcast, F32Constant, F32Multiply, F32Reindex,
    F32RmsNorm, F32Softmax, F32TensorContraction, InputKey, OpKey, OutputKey,
    RMS_NORM_F32_REFERENCE_EPS_BITS, RegistryError, ReindexForm, SemanticProgram,
    SemanticProgramBuilder, Value, add_f32_op, broadcast_f32_op, constant_f32_op, multiply_f32_op,
    reindex_f32_op, rms_norm_f32_op, softmax_f32_op, tensor_contraction_f32_op,
};
use tiler_ir::shape::{
    Axis, BindingSource, Extent, ExtentRelation, ExtentTerm, FactProvenance, RootBinding,
    SemanticInputConstraint, Shape, ShapeEnv, ShapeEnvBuilder, ShapeSymbol, SymbolScope,
};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
    rms_norm_f32, softmax_f32,
};

// --- the checkpoint's declared profile ---------------------------------------

/// Query heads the pinned checkpoint declares.
const QUERY_HEADS: usize = 16;
/// Key/value heads, which are also the grouped-query groups.
const GROUPS: usize = 8;
/// Query heads per group: `num_key_value_groups`.
const REPEATS: usize = 2;
/// The *declared* head dimension, not `hidden_size / num_attention_heads`.
const HEAD_DIM: usize = 128;
/// Halves of the head axis the rotary split produces.
const HALVES: usize = 2;
/// Width of one rotary half.
const HALF: usize = HEAD_DIM / HALVES;
/// Width of the query projection, as `q_proj` declares it.
const QUERY_WIDTH: usize = QUERY_HEADS * HEAD_DIM;
/// Width of the key and value projections, as `k_proj` and `v_proj` declare them.
const KEY_VALUE_WIDTH: usize = GROUPS * HEAD_DIM;
/// The checkpoint's `max_position_embeddings`, which bounds both extent symbols.
const MAX_POSITION_EMBEDDINGS: u64 = 32_768;

/// The C1 conformance row's new-position count.
const C1_POSITIONS: u64 = 10;
/// The C1 row's model dimension, which the two dense projections are sized by.
const C1_HIDDEN: usize = 1_024;

/// The largest fold any occurrence of the block performs at the C1 prefill row.
///
/// The query projection folds `10 * 2048` output elements over the 1,024-wide
/// model dimension, and the output projection `10 * 1024` over the 2,048-wide
/// concatenated head axis; both are 20,971,520 multiply-accumulate steps. Every
/// other occurrence is far smaller — the score and value contractions are 204,800
/// steps each, and the key and value projections half of a query projection.
///
/// This is the iteration-step allowance the end-to-end evaluation states, and it
/// is written as the block's own arithmetic rather than as a round number so that
/// an occurrence needing one step more is refused rather than quietly admitted.
const C1_LARGEST_FOLD: usize = 10 * QUERY_WIDTH * C1_HIDDEN;

/// The `10` above and the row's own new-position count are one number.
const _: () = assert!(C1_POSITIONS == 10);

/// `128 ** -0.5` rounded to binary32, from `attention_scaling_f32` in the record.
const ATTENTION_SCALE_BITS: u32 = 0x3db5_04f3;
/// The additive mask's masked entry: the most negative finite binary32.
const MASKED_FILL_BITS: u32 = 0xff7f_ffff;
/// The additive mask's attended entry: **negative** zero, because the reference
/// multiplies the fill by a boolean rather than writing a zero.
const ATTENDED_FILL_BITS: u32 = 0x8000_0000;
/// `-1.0`: the sign the first half of `cat(-x2, x1)` carries.
const NEGATIVE_ONE: u32 = 0xbf80_0000;
/// `+1.0`: the sign the second half carries.
const POSITIVE_ONE: u32 = 0x3f80_0000;

/// The seed every synthetic operand in this file is drawn from.
const FIXTURE_SEED: u64 = 0x0000_c1a7_7e17_0004;

fn axis(value: u32) -> Axis {
    Axis::new(value)
}

fn extent(value: usize) -> Extent {
    Extent::new(u64::try_from(value).expect("a workload extent fits a u64"))
}

fn block_shape<const N: usize>(dims: [usize; N]) -> Shape {
    Shape::try_from_dims(
        dims.into_iter()
            .map(|dim| u64::try_from(dim).expect("a workload extent fits a u64")),
    )
    .expect("a workload shape is admitted")
}

// --- the shape environment ---------------------------------------------------

/// The scope both extent symbols are declared in.
fn block_scope() -> SymbolScope {
    SymbolScope::new("attention-block/0").expect("a nonempty scope")
}

/// `T`, the new-position count.
fn new_positions_symbol() -> ShapeSymbol {
    ShapeSymbol::new(block_scope(), "T").expect("a valid symbol")
}

/// `S`, the context length.
fn context_symbol() -> ShapeSymbol {
    ShapeSymbol::new(block_scope(), "S").expect("a valid symbol")
}

/// Why one symbolic extent could not become a static shape extent.
///
/// Three distinct reasons and not one, because a caller acts differently on each:
/// an undeclared symbol is a construction mistake, an unbounded one is the
/// feasibility refusal the L4 record names, and a bounded-but-wide one is a
/// genuinely symbolic extent the *static* semantic shape vocabulary cannot carry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExtentRefusal {
    Undeclared,
    NoUpperBound,
    NotASinglePoint { lower: u64, upper: u64 },
}

/// Resolves one symbolic extent to the static extent a semantic value fact carries.
///
/// **This is where an unbounded extent symbol is refused rather than compiled**,
/// which is the L4 record's fourth feasibility predicate. `ExtentInterval`'s
/// contract is that a symbol nothing constrains is *present with the whole extent
/// domain* rather than absent, so the condition to test is
/// `states_no_upper_bound` and never a missing interval — a caller testing for
/// the latter would never fire.
fn resolve_static_extent(
    environment: &ShapeEnv,
    symbol: &ShapeSymbol,
) -> Result<u64, ExtentRefusal> {
    let interval = environment
        .extent_interval(symbol)
        .ok_or(ExtentRefusal::Undeclared)?;
    if interval.states_no_upper_bound() {
        return Err(ExtentRefusal::NoUpperBound);
    }
    if interval.lower != interval.upper {
        return Err(ExtentRefusal::NotASinglePoint {
            lower: interval.lower,
            upper: interval.upper,
        });
    }
    Ok(interval.lower)
}

/// Declares `T` and `S` as two separate bounded symbols and pins each to a row.
///
/// Both are bound to an input dimension rather than to a static extent, because
/// that is where the value actually comes from: `T` is `x`'s outermost axis and
/// `S` is the mask's innermost. Neither binding carries a compile-time value, so
/// the *row* enters as a frontend-required interval constraint — which is what
/// makes a decode step a binding change.
///
/// The two symbols are never joined by an equality, so `S = T` at prefill is a
/// coincidence of the row rather than a fact of the program;
/// [`the_two_extent_symbols_are_never_proved_equal`] states that as a check.
fn shape_environment(new_positions: u64, context: u64) -> ShapeEnv {
    let mut draft = ShapeEnvBuilder::new();
    for (symbol, source, row) in [
        (
            new_positions_symbol(),
            BindingSource::InputDimension {
                input: residual_key(),
                axis: axis(0),
            },
            new_positions,
        ),
        (
            context_symbol(),
            BindingSource::InputDimension {
                input: mask_key(),
                axis: axis(1),
            },
            context,
        ),
    ] {
        draft.declare(symbol.clone()).expect("a first declaration");
        draft
            .bind(
                &symbol,
                RootBinding::new(
                    source,
                    AvailabilityPhase::LiveDevicePreflight,
                    FactProvenance::RuntimeValidated,
                )
                .expect("an input dimension is readable at preflight"),
            )
            .expect("a first binding");
        // The checkpoint's own ceiling, and then the row. Both are stated: the
        // ceiling is what makes the symbol bounded at all, and a context past it
        // refuses rather than compiling.
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::interval(
                    ExtentTerm::Symbol(symbol.clone()),
                    1,
                    MAX_POSITION_EMBEDDINGS,
                )
                .expect("a nonempty interval"),
                FactProvenance::FrontendRequired,
            ))
            .expect("a declared symbol");
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::interval(ExtentTerm::Symbol(symbol), row, row)
                    .expect("a nonempty interval"),
                FactProvenance::FrontendRequired,
            ))
            .expect("a declared symbol");
    }
    draft.build().expect("the environment is consistent")
}

// --- the block's ordered interface -------------------------------------------

fn residual_key() -> InputKey {
    InputKey::new("x").expect("a valid key")
}
fn input_layernorm_key() -> InputKey {
    InputKey::new("w_input_layernorm").expect("a valid key")
}
fn query_weight_key() -> InputKey {
    InputKey::new("W_q").expect("a valid key")
}
fn key_weight_key() -> InputKey {
    InputKey::new("W_k").expect("a valid key")
}
fn value_weight_key() -> InputKey {
    InputKey::new("W_v").expect("a valid key")
}
fn query_norm_key() -> InputKey {
    InputKey::new("w_q_norm").expect("a valid key")
}
fn key_norm_key() -> InputKey {
    InputKey::new("w_k_norm").expect("a valid key")
}
fn cosine_key() -> InputKey {
    InputKey::new("cos").expect("a valid key")
}
fn sine_key() -> InputKey {
    InputKey::new("sin").expect("a valid key")
}
fn rope_sign_key() -> InputKey {
    InputKey::new("rope_sign").expect("a valid key")
}
fn mask_key() -> InputKey {
    InputKey::new("mask").expect("a valid key")
}
fn output_weight_key() -> InputKey {
    InputKey::new("W_o").expect("a valid key")
}

/// The block's twelve ordered input keys, in the order they are declared.
fn ordered_input_keys() -> Vec<InputKey> {
    vec![
        residual_key(),
        input_layernorm_key(),
        query_weight_key(),
        key_weight_key(),
        value_weight_key(),
        query_norm_key(),
        key_norm_key(),
        cosine_key(),
        sine_key(),
        rope_sign_key(),
        mask_key(),
        output_weight_key(),
    ]
}

/// The block's three ordered output keys: the residual stream, then the KV seam.
fn ordered_output_keys() -> Vec<OutputKey> {
    vec![
        OutputKey::new("h_out").expect("a valid key"),
        OutputKey::new("k_rope").expect("a valid key"),
        OutputKey::new("v_heads").expect("a valid key"),
    ]
}

/// Every static extent one instantiation of the block is built at.
///
/// `new_positions` and `context` are resolved from the shape environment; the
/// model dimension is a checkpoint constant rather than a symbol, because nothing
/// in the workload varies it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BlockExtents {
    new_positions: usize,
    context: usize,
    hidden: usize,
}

impl BlockExtents {
    /// Reads the two symbolic extents out of the environment.
    fn resolve(environment: &ShapeEnv, hidden: usize) -> Result<Self, ExtentRefusal> {
        let new_positions = resolve_static_extent(environment, &new_positions_symbol())?;
        let context = resolve_static_extent(environment, &context_symbol())?;
        Ok(Self {
            new_positions: usize::try_from(new_positions).expect("a row extent fits this host"),
            context: usize::try_from(context).expect("a row extent fits this host"),
            hidden,
        })
    }
}

/// The twelve declared input handles.
#[derive(Clone, Copy)]
struct BlockInputs {
    residual: Value<F32>,
    input_layernorm: Value<F32>,
    query_weight: Value<F32>,
    key_weight: Value<F32>,
    value_weight: Value<F32>,
    query_norm: Value<F32>,
    key_norm: Value<F32>,
    cosine: Value<F32>,
    sine: Value<F32>,
    rope_sign: Value<F32>,
    mask: Value<F32>,
    output_weight: Value<F32>,
}

/// Declares the twelve inputs, in the design's order.
fn declare_inputs(builder: &mut SemanticProgramBuilder, extents: BlockExtents) -> BlockInputs {
    let BlockExtents {
        new_positions: t,
        context: s,
        hidden,
    } = extents;
    let mut declare = |key: InputKey, shape: Shape| {
        builder
            .input::<F32>(key, shape)
            .expect("a first declaration of an F32 input")
    };
    BlockInputs {
        residual: declare(residual_key(), block_shape([t, hidden])),
        input_layernorm: declare(input_layernorm_key(), block_shape([hidden])),
        query_weight: declare(query_weight_key(), block_shape([QUERY_WIDTH, hidden])),
        key_weight: declare(key_weight_key(), block_shape([KEY_VALUE_WIDTH, hidden])),
        value_weight: declare(value_weight_key(), block_shape([KEY_VALUE_WIDTH, hidden])),
        query_norm: declare(query_norm_key(), block_shape([HEAD_DIM])),
        key_norm: declare(key_norm_key(), block_shape([HEAD_DIM])),
        cosine: declare(cosine_key(), block_shape([t, HEAD_DIM])),
        sine: declare(sine_key(), block_shape([t, HEAD_DIM])),
        rope_sign: declare(rope_sign_key(), block_shape([HALVES, 1])),
        mask: declare(mask_key(), block_shape([t, s])),
        output_weight: declare(output_weight_key(), block_shape([hidden, QUERY_WIDTH])),
    }
}

// --- the coordinate maps and axis mappings -----------------------------------

/// Frontend index labels, deliberately neither dense nor ascending.
const G: u32 = 70;
const R: u32 = 71;
const T_INDEX: u32 = 72;
const S_INDEX: u32 = 73;
const D: u32 = 74;
const O: u32 = 75;

fn index(label: u32) -> ContractionIndex {
    ContractionIndex::new(label)
}

/// Structure 1, `td,od->to`: the four dense projections.
fn projection_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [vec![index(T_INDEX), index(D)], vec![index(O), index(D)]],
        [index(T_INDEX), index(O)],
    )
    .expect("td,od->to is admitted")
}

/// Structure 2, `grtd,gsd->grts`: the score contraction.
///
/// `r` is in the query operand and the result and in neither the key operand nor
/// the contracted set, which is what makes the eight-to-sixteen grouped-query
/// repetition free — no `[16, S, 128]` key is ever materialized.
fn score_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [
            vec![index(G), index(R), index(T_INDEX), index(D)],
            vec![index(G), index(S_INDEX), index(D)],
        ],
        [index(G), index(R), index(T_INDEX), index(S_INDEX)],
    )
    .expect("grtd,gsd->grts is admitted")
}

/// Structure 3, `grts,gsd->grtd`: the value contraction, over the growing `S`.
fn value_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [
            vec![index(G), index(R), index(T_INDEX), index(S_INDEX)],
            vec![index(G), index(S_INDEX), index(D)],
        ],
        [index(G), index(R), index(T_INDEX), index(D)],
    )
    .expect("grts,gsd->grtd is admitted")
}

/// `[hidden] -> [T, hidden]`: the input normalization weight's rank pad.
fn hidden_weight_mapping(t: usize, hidden: usize) -> BroadcastAxisMapping {
    BroadcastAxisMapping::new(
        [extent(t), extent(hidden)],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(axis(0)),
        ],
    )
    .expect("the mapping accounts for every result axis")
}

/// `[128] -> [T, heads, 128]`: a per-head normalization weight's rank pad.
fn head_weight_mapping(t: usize, heads: usize) -> BroadcastAxisMapping {
    BroadcastAxisMapping::new(
        [extent(t), extent(heads), extent(HEAD_DIM)],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(axis(0)),
        ],
    )
    .expect("the mapping accounts for every result axis")
}

/// `[2, 1] -> [T, heads, 2, 64]`: the rotary sign operand.
///
/// Two leading rank pads, a one-to-one correspondence on the size-two axis, and
/// an extent-one *stretch* on the second — the operand has that axis, so widening
/// it is `stretch-unit` and not `replicate`.
fn sign_mapping(t: usize, heads: usize) -> BroadcastAxisMapping {
    BroadcastAxisMapping::new(
        [extent(t), extent(heads), extent(HALVES), extent(HALF)],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(axis(0)),
            BroadcastAxisSource::StretchUnit(axis(1)),
        ],
    )
    .expect("the mapping accounts for every result axis")
}

/// `[T, 128] -> [T, heads, 128]`: a rotary table, over an *interior* rank pad.
///
/// The table has a position axis and a lane axis and no head axis at all, so the
/// head axis is `replicate` while both of the table's own axes are one-to-one, in
/// ascending order — which is what a broadcast may do and a reordering is not.
fn table_mapping(t: usize, heads: usize) -> BroadcastAxisMapping {
    BroadcastAxisMapping::new(
        [extent(t), extent(heads), extent(HEAD_DIM)],
        [
            BroadcastAxisSource::FromOperand(axis(0)),
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(axis(1)),
        ],
    )
    .expect("the mapping accounts for every result axis")
}

/// `[T, S] -> [8, 2, T, S]`: the additive causal mask over the two head axes.
fn mask_mapping(t: usize, s: usize) -> BroadcastAxisMapping {
    BroadcastAxisMapping::new(
        [extent(GROUPS), extent(REPEATS), extent(t), extent(s)],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(axis(0)),
            BroadcastAxisSource::FromOperand(axis(1)),
        ],
    )
    .expect("the mapping accounts for every result axis")
}

/// `[T, width] -> [T, heads, 128]`: a projection read as heads of width 128.
fn projection_split(heads: usize) -> ReindexForm {
    ReindexForm::split_axis(axis(1), [extent(heads), extent(HEAD_DIM)])
        .expect("a declared width factors as heads x 128")
}

/// Which key head a query head reads.
///
/// Both readings are expressible in the `Reindex` family and both produce an
/// identically shaped `[8, 2, T, 128]` query, so nothing structural separates
/// them and only a value comparison can. `repeat_kv` is repeat-interleave, so
/// [`HeadReading::Interleave`] is the one that denotes the reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HeadReading {
    /// `(8, 2)`, group major: `h = 2g + r`, so the group is `h / 2`.
    Interleave,
    /// `(2, 8)`, repetition major: `h = 8r + g`, so the group is `h % 8`.
    Tile,
}

impl HeadReading {
    /// `[T, 16, 128] -> [T, ., ., 128]`: the head axis factorized.
    ///
    /// `split-axis` is normatively a row-major factorization with the major factor
    /// first, so the *order* of the two factors is the whole content of the
    /// reading rather than a spelling of it.
    fn split(self) -> ReindexForm {
        let factors = match self {
            Self::Interleave => [extent(GROUPS), extent(REPEATS)],
            Self::Tile => [extent(REPEATS), extent(GROUPS)],
        };
        ReindexForm::split_axis(axis(1), factors).expect("16 = 8 x 2")
    }

    /// `[T, ., ., 128] -> [8, 2, T, 128]`: the group axis moved outermost.
    fn permute(self) -> ReindexForm {
        let order = match self {
            // [T, g, r, d] -> [g, r, T, d]
            Self::Interleave => [axis(1), axis(2), axis(0), axis(3)],
            // [T, r, g, d] -> [g, r, T, d]
            Self::Tile => [axis(2), axis(1), axis(0), axis(3)],
        };
        ReindexForm::permute_axes(order).expect("a rank-four order")
    }

    /// The key head query head `h` reads under this reading.
    fn group_of(self, head: usize) -> usize {
        match self {
            Self::Interleave => head / REPEATS,
            Self::Tile => head % GROUPS,
        }
    }
}

/// `[S, 8, 128] -> [8, S, 128]`: one map serving both the key and the value edge.
fn key_value_permute() -> ReindexForm {
    ReindexForm::permute_axes([axis(1), axis(0), axis(2)]).expect("[S, g, d] -> [g, S, d]")
}

/// `[8, 2, T, 128] -> [T, 8, 2, 128]`.
fn output_permute() -> ReindexForm {
    ReindexForm::permute_axes([axis(2), axis(0), axis(1), axis(3)])
        .expect("[g, r, T, d] -> [T, g, r, d]")
}

/// `merge_axes([1, 2])`, applied twice: `(g, r) -> h`, then `(h, d) -> width`.
fn output_merge() -> ReindexForm {
    ReindexForm::merge_axes([axis(1), axis(2)]).expect("axes 1 and 2 are an adjacent run")
}

/// `[…, 128] -> […, 2, 64]`: the rotary half split, major factor first.
fn half_split(rank_of_lane_axis: u32) -> ReindexForm {
    ReindexForm::split_axis(axis(rank_of_lane_axis), [extent(HALVES), extent(HALF)])
        .expect("128 = 2 x 64")
}

/// The within-axis coordinate swap, in the one admitted form.
///
/// `reverse-axis` is `i -> extent - 1 - i`, and at extent two that is `i -> 1 - i`
/// exactly. Decision D-10 admits this map and no other within-axis permutation.
fn within_axis_swap(lane_axis: u32) -> ReindexForm {
    ReindexForm::reverse_axis(axis(lane_axis)).expect("the size-two axis reverses")
}

/// `[…, 2, 64] -> […, 128]`: the merge that inverts the half split.
fn half_merge(lane_axis: u32) -> ReindexForm {
    ReindexForm::merge_axes([axis(lane_axis), axis(lane_axis + 1)])
        .expect("the two inner axes are an adjacent run")
}

// --- the block ----------------------------------------------------------------

/// Emits the ten-occurrence rotary composition over a `[T, heads, 128]` operand.
fn rotary(
    builder: &mut SemanticProgramBuilder,
    operand: Value<F32>,
    inputs: &BlockInputs,
    t: usize,
    heads: usize,
) -> Value<F32> {
    let split = F32Reindex::apply(builder, &half_split(2), operand).expect("128 = 2 x 64");
    let swapped = F32Reindex::apply(builder, &within_axis_swap(2), split)
        .expect("the size-two axis reverses");
    let signs = F32Broadcast::apply(builder, &sign_mapping(t, heads), inputs.rope_sign)
        .expect("the sign operand broadcasts over the half width");
    let signed =
        F32Multiply::apply(builder, swapped, signs).expect("both operands are [T, h, 2, 64]");
    let rotated =
        F32Reindex::apply(builder, &half_merge(2), signed).expect("the two inner axes merge");

    let mapping = table_mapping(t, heads);
    let cosine = F32Broadcast::apply(builder, &mapping, inputs.cosine)
        .expect("the cosine table broadcasts over the head axis");
    let direct =
        F32Multiply::apply(builder, operand, cosine).expect("both operands are [T, h, 128]");
    let sine = F32Broadcast::apply(builder, &mapping, inputs.sine)
        .expect("the sine table broadcasts over the head axis");
    let turned = F32Multiply::apply(builder, rotated, sine).expect("both operands are [T, h, 128]");
    F32Add::apply(builder, direct, turned).expect("both operands are [T, h, 128]")
}

/// Emits one RMS normalization together with the broadcast its weight needs.
///
/// Two occurrences and never one: `tiler::rms-norm-f32@1` takes a weight already
/// shaped like the value, because the graph admits no implicit broadcasting, so a
/// per-channel weight is widened by a `tiler::broadcast-f32@2` the caller writes.
fn normalize(
    builder: &mut SemanticProgramBuilder,
    value: Value<F32>,
    weight: Value<F32>,
    mapping: &BroadcastAxisMapping,
    reduced: Axis,
) -> Value<F32> {
    let widened = F32Broadcast::apply(builder, mapping, weight)
        .expect("the weight broadcasts over the value's shape");
    F32RmsNorm::apply(
        builder,
        value,
        widened,
        reduced,
        RMS_NORM_F32_REFERENCE_EPS_BITS,
    )
    .expect("the weight now carries the value's own shape")
}

/// Builds the complete block at the admitted head reading.
fn build_block(extents: BlockExtents) -> SemanticProgram {
    build_block_with(extents, HeadReading::Interleave)
        .expect("a prefill row binds the context to the new positions")
}

/// Builds the complete block: twenty-two steps, forty-eight occurrences.
///
/// The step numbers in the comments are the L4 design's table.
///
/// Fallible in exactly one place, and deliberately. Every `expect` below asserts
/// an invariant that holds for *every* admissible binding — a declared width
/// factors, an axis order is a permutation, a structure is admitted — while the
/// mask add's success is a fact about the binding itself: this is the **prefill**
/// block, which computes its own key and value from its own input, so the context
/// extent is whatever the key path produced and a mask stating a different one
/// does not agree with the score tensor. See
/// [`a_context_wider_than_the_new_positions_is_refused_at_prefill`].
fn build_block_with(
    extents: BlockExtents,
    heads: HeadReading,
) -> Result<SemanticProgram, BuildError> {
    let BlockExtents {
        new_positions: t,
        context: s,
        hidden,
    } = extents;
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let inputs = declare_inputs(&mut builder, extents);

    // 1. RMS normalization of the residual stream, over the model dimension.
    let normalized = normalize(
        &mut builder,
        inputs.residual,
        inputs.input_layernorm,
        &hidden_weight_mapping(t, hidden),
        axis(1),
    );

    // 2, 3, 4. The query, key, and value projections, all structure 1.
    let structure = projection_structure();
    let query_flat =
        F32TensorContraction::apply(&mut builder, &structure, normalized, inputs.query_weight)
            .expect("td,od->to over [T, hidden] and [2048, hidden]");
    let key_flat =
        F32TensorContraction::apply(&mut builder, &structure, normalized, inputs.key_weight)
            .expect("td,od->to over [T, hidden] and [1024, hidden]");
    let value_flat =
        F32TensorContraction::apply(&mut builder, &structure, normalized, inputs.value_weight)
            .expect("td,od->to over [T, hidden] and [1024, hidden]");

    // 5, 6, 7. The three head splits.
    let query_heads = F32Reindex::apply(&mut builder, &projection_split(QUERY_HEADS), query_flat)
        .expect("2048 = 16 x 128");
    let key_heads = F32Reindex::apply(&mut builder, &projection_split(GROUPS), key_flat)
        .expect("1024 = 8 x 128");
    let value_split = F32Reindex::apply(&mut builder, &projection_split(GROUPS), value_flat)
        .expect("1024 = 8 x 128");

    // 8, 9. Per-head normalization over the 128-wide axis.
    let query_norm = normalize(
        &mut builder,
        query_heads,
        inputs.query_norm,
        &head_weight_mapping(t, QUERY_HEADS),
        axis(2),
    );
    let key_norm = normalize(
        &mut builder,
        key_heads,
        inputs.key_norm,
        &head_weight_mapping(t, GROUPS),
        axis(2),
    );

    // 10, 11. Rotary position embedding, ten occurrences each.
    let query_rotary = rotary(&mut builder, query_norm, &inputs, t, QUERY_HEADS);
    let key_rotary = rotary(&mut builder, key_norm, &inputs, t, GROUPS);

    // 12. The grouped-query head layout.
    let query_grouped =
        F32Reindex::apply(&mut builder, &heads.split(), query_rotary).expect("16 = 8 x 2");
    let query_grouped = F32Reindex::apply(&mut builder, &heads.permute(), query_grouped)
        .expect("the group axis moves outermost");

    // 13, 14. The two retained outputs' layouts.
    let key_rope = F32Reindex::apply(&mut builder, &key_value_permute(), key_rotary)
        .expect("[S, g, d] -> [g, S, d]");
    let value_heads = F32Reindex::apply(&mut builder, &key_value_permute(), value_split)
        .expect("[S, g, d] -> [g, S, d]");

    // 15. The score contraction, structure 2.
    let scores =
        F32TensorContraction::apply(&mut builder, &score_structure(), query_grouped, key_rope)
            .expect("grtd,gsd->grts over the grouped query and the key");

    // 16. The scale, on the *score* and not on an operand.
    let scale = F32Constant::apply(&mut builder, ATTENTION_SCALE_BITS).expect("a scalar constant");
    let scaled = F32Multiply::apply(&mut builder, scores, scale)
        .expect("a rank-zero right operand is admitted");

    // 17. The additive causal mask, broadcast over the two head axes.
    let mask = F32Broadcast::apply(&mut builder, &mask_mapping(t, s), inputs.mask)
        .expect("the mask broadcasts over the group and repetition axes");
    // The one binding-dependent step: the scores are `[8, 2, T, T]` because the
    // key they contract against is this block's own, so a mask of a different
    // context width is refused here rather than silently truncated.
    let masked = F32Add::apply(&mut builder, scaled, mask)?;

    // 18. Softmax over the key axis.
    let probabilities =
        F32Softmax::apply(&mut builder, masked, axis(3)).expect("axis 3 is the key axis");

    // 19. The value contraction, structure 3, over the growing extent.
    let context_vectors =
        F32TensorContraction::apply(&mut builder, &value_structure(), probabilities, value_heads)
            .expect("grts,gsd->grtd over the probabilities and the value heads");

    // 20. The head merge, inverting step 12's layout.
    let merged = F32Reindex::apply(&mut builder, &output_permute(), context_vectors)
        .expect("[g, r, T, d] -> [T, g, r, d]");
    let merged = F32Reindex::apply(&mut builder, &output_merge(), merged).expect("(g, r) -> h");
    let context_flat =
        F32Reindex::apply(&mut builder, &output_merge(), merged).expect("(h, d) -> 2048");

    // 21. The output projection, structure 1.
    let attention_out =
        F32TensorContraction::apply(&mut builder, &structure, context_flat, inputs.output_weight)
            .expect("td,od->to over [T, 2048] and [hidden, 2048]");

    // 22. The residual add.
    let residual_out = F32Add::apply(&mut builder, inputs.residual, attention_out)
        .expect("both operands are [T, hidden]");

    // The three ordered named outputs. `h_out` first because it is the block's
    // observable result; `k_rope` and `v_heads` follow as the KV seam.
    let keys = ordered_output_keys();
    builder
        .output(keys[0].clone(), residual_out)
        .expect("a first output key");
    builder
        .output(keys[1].clone(), key_rope)
        .expect("a first output key");
    builder
        .output(keys[2].clone(), value_heads)
        .expect("a first output key");
    Ok(builder.build().expect("the block is complete"))
}

// --- fixtures ------------------------------------------------------------------

/// Deterministic synthetic operands, exactly representable in binary32.
///
/// `SplitMix64` so consecutive draws do not correlate across the strides the
/// contractions read them at, assembled from bits rather than cast from an
/// integer: the mantissa lands in `[1, 2)` exactly, the scale is exact, and the
/// subtraction is exact by Sterbenz, so the generator introduces no rounding the
/// comparison would then be asserting about.
fn samples(count: usize, seed: u64) -> Vec<f32> {
    let mut state = seed | 1;
    (0..count)
        .map(|_| {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            let mantissa = ((z ^ (z >> 31)) >> 40) as u32 & 0x007f_ffff;
            f32::from_bits(0x3f80_0000 | mantissa) * 0.25 - 0.375
        })
        .collect()
}

fn tensor_of_bits(shape: &Shape, bits: impl Fn(usize) -> u32) -> Tensor {
    let count = shape.element_count().expect("a workload shape is bounded");
    let elements = (0..count)
        .map(|position| {
            ReferenceElement::from_float_bits(
                bits(position).to_be_bytes(),
                FloatBitOrder::MostSignificantByteFirst,
            )
            .expect("an f32 payload is four bytes")
        })
        .collect();
    Tensor::dense(F32::resolved_type(), shape.clone(), elements).expect("the tensor is well formed")
}

fn tensor_of(shape: &Shape, values: &[f32]) -> Tensor {
    assert_eq!(
        shape.element_count(),
        Some(values.len()),
        "a fixture states every element"
    );
    tensor_of_bits(shape, |position| values[position].to_bits())
}

fn payload_bits(tensor: &Tensor) -> Vec<u32> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a block result is a dense f32 tensor");
    };
    elements
        .iter()
        .map(|element| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
            )
        })
        .collect()
}

fn payload_floats(tensor: &Tensor) -> Vec<f32> {
    payload_bits(tensor)
        .into_iter()
        .map(f32::from_bits)
        .collect()
}

/// The additive causal mask at one row, in the reference's own two values.
///
/// A masked entry is the most negative finite binary32 and an attended entry is
/// **negative** zero, because the reference multiplies the fill by a boolean
/// rather than writing a zero. The attended entry is value-preserving on every
/// score except `+0.0`.
fn causal_mask(t: usize, s: usize) -> Vec<u32> {
    let mut mask = Vec::with_capacity(t * s);
    for query in 0..t {
        for key in 0..s {
            // At prefill the context equals the new positions, so key position
            // `k` is visible to query position `q` exactly when `k <= q`.
            mask.push(if key <= query {
                ATTENDED_FILL_BITS
            } else {
                MASKED_FILL_BITS
            });
        }
    }
    mask
}

/// Every operand one block evaluation binds, at one set of extents.
struct BlockFixture {
    extents: BlockExtents,
    residual: Tensor,
    input_layernorm: Tensor,
    query_weight: Tensor,
    key_weight: Tensor,
    value_weight: Tensor,
    query_norm: Tensor,
    key_norm: Tensor,
    cosine: Tensor,
    sine: Tensor,
    rope_sign: Tensor,
    mask: Tensor,
    output_weight: Tensor,
}

impl BlockFixture {
    fn new(extents: BlockExtents) -> Self {
        let BlockExtents {
            new_positions: t,
            context: s,
            hidden,
        } = extents;
        let mut salt = FIXTURE_SEED;
        let mut next = |count: usize| {
            salt = salt.wrapping_add(0x0f0f_0f0f_0f0f_0f0f);
            samples(count, salt)
        };
        Self {
            extents,
            residual: tensor_of(&block_shape([t, hidden]), &next(t * hidden)),
            input_layernorm: tensor_of(&block_shape([hidden]), &next(hidden)),
            query_weight: tensor_of(
                &block_shape([QUERY_WIDTH, hidden]),
                &next(QUERY_WIDTH * hidden),
            ),
            key_weight: tensor_of(
                &block_shape([KEY_VALUE_WIDTH, hidden]),
                &next(KEY_VALUE_WIDTH * hidden),
            ),
            value_weight: tensor_of(
                &block_shape([KEY_VALUE_WIDTH, hidden]),
                &next(KEY_VALUE_WIDTH * hidden),
            ),
            query_norm: tensor_of(&block_shape([HEAD_DIM]), &next(HEAD_DIM)),
            key_norm: tensor_of(&block_shape([HEAD_DIM]), &next(HEAD_DIM)),
            cosine: tensor_of(&block_shape([t, HEAD_DIM]), &next(t * HEAD_DIM)),
            sine: tensor_of(&block_shape([t, HEAD_DIM]), &next(t * HEAD_DIM)),
            rope_sign: tensor_of_bits(&block_shape([HALVES, 1]), |half| {
                [NEGATIVE_ONE, POSITIVE_ONE][half]
            }),
            mask: {
                let bits = causal_mask(t, s);
                tensor_of_bits(&block_shape([t, s]), |position| bits[position])
            },
            output_weight: tensor_of(
                &block_shape([hidden, QUERY_WIDTH]),
                &next(hidden * QUERY_WIDTH),
            ),
        }
    }

    fn bindings(&self) -> Vec<(InputKey, &Tensor)> {
        vec![
            (residual_key(), &self.residual),
            (input_layernorm_key(), &self.input_layernorm),
            (query_weight_key(), &self.query_weight),
            (key_weight_key(), &self.key_weight),
            (value_weight_key(), &self.value_weight),
            (query_norm_key(), &self.query_norm),
            (key_norm_key(), &self.key_norm),
            (cosine_key(), &self.cosine),
            (sine_key(), &self.sine),
            (rope_sign_key(), &self.rope_sign),
            (mask_key(), &self.mask),
            (output_weight_key(), &self.output_weight),
        ]
    }
}

/// Evaluates one block program against one fixture and returns its three outputs.
///
/// The allowance is the caller's rather than this helper's, because it is the
/// only thing separating an evaluation of this block at the C1 row from a refusal
/// of it — so a reader of a call site sees which of the two is being asked for.
fn evaluate_block(
    program: &SemanticProgram,
    fixture: &BlockFixture,
    iteration_step_allowance: usize,
) -> [Vec<u32>; 3] {
    let owned = fixture.bindings();
    let bindings: Vec<InputBinding<'_>> = owned
        .iter()
        .map(|(key, tensor)| InputBinding::new(key, tensor))
        .collect();
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .with_iteration_step_allowance(iteration_step_allowance)
        .evaluate(program, &bindings)
        .expect("the block evaluates");
    let [residual, key_rope, value_heads] = outputs.as_slice() else {
        panic!("the block has three outputs");
    };
    let BlockExtents {
        new_positions: t,
        context: s,
        hidden,
    } = fixture.extents;
    assert_eq!(residual.shape(), &block_shape([t, hidden]));
    assert_eq!(key_rope.shape(), &block_shape([GROUPS, s, HEAD_DIM]));
    assert_eq!(value_heads.shape(), &block_shape([GROUPS, s, HEAD_DIM]));
    [
        payload_bits(residual),
        payload_bits(key_rope),
        payload_bits(value_heads),
    ]
}

// --- the independent recomputation ---------------------------------------------

/// One strict ascending fold, seeded from the *first product* and never `+0.0`.
///
/// `fl(+0.0 + x)` equals `x` for every binary32 except `x = -0.0`, so the
/// idiomatic accumulator-starts-at-zero loop silently computes a different
/// operation on a vector whose products are all negative zero — which is exactly
/// the masked-position case below.
fn strict_fold(contributors: impl IntoIterator<Item = f32>) -> f32 {
    let mut accumulator: Option<f32> = None;
    for contributor in contributors {
        accumulator = Some(accumulator.map_or(contributor, |value| value + contributor));
    }
    accumulator.expect("a nonempty contributor sequence")
}

/// The block recomputed from the operation table by explicit coordinate arithmetic.
///
/// **Independence boundary.** Every access relation, coordinate map, broadcast,
/// head pairing, and composition order below is restated from the design's table
/// rather than re-run through the graph, so a wrong index binding fails here. The
/// two non-linear families' *scalar arithmetic* is the crate's own certified
/// [`rms_norm_f32`] and [`softmax_f32`] rather than a second implementation, so
/// this comparison says nothing about the binary32 results of `rsqrt` and `exp` —
/// which the RMS-normalization and softmax corpora own and which a second
/// hand-rolled copy here would only make agree with itself.
struct BlockExpectation {
    residual_out: Vec<u32>,
    key_rope: Vec<u32>,
    value_heads: Vec<u32>,
}

fn recompute_block(fixture: &BlockFixture) -> BlockExpectation {
    let BlockExtents {
        new_positions: t,
        context: s,
        hidden,
    } = fixture.extents;
    let x = payload_floats(&fixture.residual);
    let w_in = payload_floats(&fixture.input_layernorm);
    let w_q = payload_floats(&fixture.query_weight);
    let w_k = payload_floats(&fixture.key_weight);
    let w_v = payload_floats(&fixture.value_weight);
    let query_head_weight = payload_floats(&fixture.query_norm);
    let key_head_weight = payload_floats(&fixture.key_norm);
    let cosine = payload_floats(&fixture.cosine);
    let sine = payload_floats(&fixture.sine);
    let mask = payload_bits(&fixture.mask);
    let w_o = payload_floats(&fixture.output_weight);

    // 1. The input normalization, with the weight replicated over the position
    // axis exactly as the broadcast's mapping states.
    let widened: Vec<f32> = (0..t * hidden).map(|i| w_in[i % hidden]).collect();
    let normalized = rms_norm_f32(
        &block_shape([t, hidden]),
        axis(1),
        RMS_NORM_F32_REFERENCE_EPS_BITS,
        &x,
        &widened,
    )
    .expect("the normalization is well formed");

    // 2, 3, 4. `td,od->to`: output `[t, o]` folds over `d`.
    let project = |weight: &[f32], out_width: usize| -> Vec<f32> {
        let mut result = Vec::with_capacity(t * out_width);
        for position in 0..t {
            for column in 0..out_width {
                result.push(strict_fold((0..hidden).map(|depth| {
                    normalized[position * hidden + depth] * weight[column * hidden + depth]
                })));
            }
        }
        result
    };
    let query_flat = project(&w_q, QUERY_WIDTH);
    let key_flat = project(&w_k, KEY_VALUE_WIDTH);
    let value_flat = project(&w_v, KEY_VALUE_WIDTH);

    // 5, 6, 7. The head splits are row-major refactorings, so `[t, heads*128]`
    // and `[t, heads, 128]` are the same buffer read two ways.

    // 8, 9. Per-head normalization over the 128-wide axis, weight replicated
    // over both the position and the head axis.
    let per_head_norm = |values: &[f32], weight: &[f32], heads: usize| -> Vec<f32> {
        let widened: Vec<f32> = (0..t * heads * HEAD_DIM)
            .map(|i| weight[i % HEAD_DIM])
            .collect();
        rms_norm_f32(
            &block_shape([t, heads, HEAD_DIM]),
            axis(2),
            RMS_NORM_F32_REFERENCE_EPS_BITS,
            values,
            &widened,
        )
        .expect("the normalization is well formed")
    };
    let query_norm = per_head_norm(&query_flat, &query_head_weight, QUERY_HEADS);
    let key_norm = per_head_norm(&key_flat, &key_head_weight, GROUPS);

    // 10, 11. `y = x * cos + rotate_half(x) * sin`, with
    // `rotate_half(x) = cat(-x2, x1)` derived from the coordinate maps: the split
    // makes lane `64i + j` the coordinate `(i, j)`, the reversal sends `i -> 1-i`,
    // and the sign operand carries `-1` at `i = 0` and `+1` at `i = 1`.
    let rotate = |values: &[f32], heads: usize| -> Vec<f32> {
        let mut result = Vec::with_capacity(t * heads * HEAD_DIM);
        for position in 0..t {
            for head in 0..heads {
                let row = (position * heads + head) * HEAD_DIM;
                for lane in 0..HEAD_DIM {
                    let (half, offset) = (lane / HALF, lane % HALF);
                    // The reversed coordinate, and the sign that half carries.
                    let source = row + (1 - half) * HALF + offset;
                    let sign = if half == 0 { -1.0_f32 } else { 1.0_f32 };
                    let rotated = values[source] * sign;
                    let table = position * HEAD_DIM + lane;
                    result.push(values[row + lane] * cosine[table] + rotated * sine[table]);
                }
            }
        }
        result
    };
    let query_rotary = rotate(&query_norm, QUERY_HEADS);
    let key_rotary = rotate(&key_norm, GROUPS);

    // 12. `[t, 16, 128] -> [8, 2, t, 128]`, group major so head `h` is `2g + r`.
    let mut query_grouped = vec![0.0_f32; GROUPS * REPEATS * t * HEAD_DIM];
    for group in 0..GROUPS {
        for repeat in 0..REPEATS {
            for position in 0..t {
                for lane in 0..HEAD_DIM {
                    let head = group * REPEATS + repeat;
                    query_grouped[((group * REPEATS + repeat) * t + position) * HEAD_DIM + lane] =
                        query_rotary[(position * QUERY_HEADS + head) * HEAD_DIM + lane];
                }
            }
        }
    }

    // 13, 14. `[t, 8, 128] -> [8, t, 128]`, one map serving both edges.
    let to_group_major = |values: &[f32]| -> Vec<f32> {
        let mut result = vec![0.0_f32; GROUPS * t * HEAD_DIM];
        for group in 0..GROUPS {
            for position in 0..t {
                for lane in 0..HEAD_DIM {
                    result[(group * t + position) * HEAD_DIM + lane] =
                        values[(position * GROUPS + group) * HEAD_DIM + lane];
                }
            }
        }
        result
    };
    let key_rope = to_group_major(&key_rotary);
    let value_heads = to_group_major(&value_flat);

    // 15, 16, 17, 18. The score contraction, the scale on the score, the mask
    // add, and the softmax over the key axis.
    let scale = f32::from_bits(ATTENTION_SCALE_BITS);
    let mut masked = Vec::with_capacity(GROUPS * REPEATS * t * s);
    for group in 0..GROUPS {
        for repeat in 0..REPEATS {
            for position in 0..t {
                for key in 0..s {
                    let score = strict_fold((0..HEAD_DIM).map(|depth| {
                        query_grouped
                            [((group * REPEATS + repeat) * t + position) * HEAD_DIM + depth]
                            * key_rope[(group * s + key) * HEAD_DIM + depth]
                    }));
                    masked.push(score * scale + f32::from_bits(mask[position * s + key]));
                }
            }
        }
    }
    let probabilities = softmax_f32(&block_shape([GROUPS, REPEATS, t, s]), axis(3), &masked)
        .expect("the softmax is well formed");

    // 19. `grts,gsd->grtd`: the value contraction folds over the key position.
    let mut context_vectors = Vec::with_capacity(GROUPS * REPEATS * t * HEAD_DIM);
    for group in 0..GROUPS {
        for repeat in 0..REPEATS {
            for position in 0..t {
                for lane in 0..HEAD_DIM {
                    context_vectors.push(strict_fold((0..s).map(|key| {
                        probabilities[((group * REPEATS + repeat) * t + position) * s + key]
                            * value_heads[(group * s + key) * HEAD_DIM + lane]
                    })));
                }
            }
        }
    }

    // 20. `[8, 2, t, 128] -> [t, 2048]`, the inverse of step 12's layout.
    let mut context_flat = vec![0.0_f32; t * QUERY_WIDTH];
    for position in 0..t {
        for group in 0..GROUPS {
            for repeat in 0..REPEATS {
                for lane in 0..HEAD_DIM {
                    let head = group * REPEATS + repeat;
                    context_flat[position * QUERY_WIDTH + head * HEAD_DIM + lane] = context_vectors
                        [((group * REPEATS + repeat) * t + position) * HEAD_DIM + lane];
                }
            }
        }
    }

    // 21, 22. The output projection and the residual add.
    let mut residual_out = Vec::with_capacity(t * hidden);
    for position in 0..t {
        for column in 0..hidden {
            let projected = strict_fold((0..QUERY_WIDTH).map(|depth| {
                context_flat[position * QUERY_WIDTH + depth] * w_o[column * QUERY_WIDTH + depth]
            }));
            residual_out.push((x[position * hidden + column] + projected).to_bits());
        }
    }

    BlockExpectation {
        residual_out,
        key_rope: key_rope.into_iter().map(f32::to_bits).collect(),
        value_heads: value_heads.into_iter().map(f32::to_bits).collect(),
    }
}

fn differing(left: &[u32], right: &[u32]) -> usize {
    assert_eq!(left.len(), right.len(), "a comparison is element-wise");
    left.iter().zip(right).filter(|(a, b)| a != b).count()
}

// --- the block's shape ----------------------------------------------------------

/// The C1 prefill row's environment and extents.
fn c1_extents() -> BlockExtents {
    let environment = shape_environment(C1_POSITIONS, C1_POSITIONS);
    BlockExtents::resolve(&environment, C1_HIDDEN).expect("both symbols are pinned and bounded")
}

/// The block's ordered interface and its occurrence census, counted by key.
///
/// A program's ordered inputs and ordered named outputs are part of its contract,
/// and its occurrence count is a measurement of the graph rather than of the
/// prose that describes it — a *step* of exposition may be one occurrence, three,
/// or ten. Counting by key rather than in total is what makes a step that
/// silently became a different family fail here instead of passing on arithmetic,
/// and the output shapes are the families' own derivations, which no caller
/// declared. The worked instance is the L4 design's twenty-two steps, which
/// verify as forty-eight occurrences over eight registered keys and no ninth.
#[test]
fn the_block_verifies_at_the_c1_prefill_shape() {
    let extents = c1_extents();
    assert_eq!(
        extents,
        BlockExtents {
            new_positions: 10,
            context: 10,
            hidden: 1_024,
        }
    );
    let program = build_block(extents);

    // Twelve ordered inputs and three ordered named outputs.
    assert_eq!(program.input_count(), 12);
    assert_eq!(program.output_count(), 3);
    let inputs: Vec<InputKey> = program.inputs().map(|input| input.key().clone()).collect();
    assert_eq!(inputs, ordered_input_keys());
    let outputs: Vec<OutputKey> = program
        .outputs()
        .map(|output| output.key().clone())
        .collect();
    assert_eq!(
        outputs,
        ordered_output_keys(),
        "h_out is the observable result and k_rope and v_heads are the KV seam, \
         in that order"
    );

    // Forty-eight occurrences over the eight already-registered keys, and no
    // ninth. Counted by key rather than in total, so a step that silently became
    // a different family fails here rather than passing on arithmetic.
    let mut counts: Vec<(OpKey, usize)> = Vec::new();
    for operation in program.operations() {
        match counts.iter_mut().find(|(key, _)| key == operation.key()) {
            Some((_, count)) => *count += 1,
            None => counts.push((operation.key().clone(), 1)),
        }
    }
    counts.sort_by(|left, right| left.0.cmp(&right.0));
    let mut expected = vec![
        // 2 rotary adds + 1 mask add + 1 residual add
        (add_f32_op(), 4),
        // 3 normalization weights + 2 rotary signs + 4 rotary tables + 1 mask
        (broadcast_f32_op(), 10),
        (constant_f32_op(), 1),
        // 2 rotary signs + 4 rotary tables + 1 scale
        (multiply_f32_op(), 7),
        // 3 head splits + 2 x 3 rotary + 2 grouping + 2 kv permutes + 3 merges
        (reindex_f32_op(), 16),
        (rms_norm_f32_op(), 3),
        (softmax_f32_op(), 1),
        (tensor_contraction_f32_op(), 6),
    ];
    expected.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(counts, expected);
    assert_eq!(program.operation_count(), 48);

    // The derived output shapes, which no caller declared.
    let resolved: Vec<Shape> = program
        .outputs()
        .map(|output| {
            program
                .shape(output.value())
                .expect("an output value has a shape")
                .as_static()
                .expect("a statically authored program has a fixed output shape")
                .clone()
        })
        .collect();
    assert_eq!(
        resolved,
        vec![
            block_shape([10, 1_024]),
            block_shape([GROUPS, 10, HEAD_DIM]),
            block_shape([GROUPS, 10, HEAD_DIM]),
        ]
    );
}

/// One keyed family carries every contraction, with its structure as an attribute.
///
/// `tiler::tensor-contraction-f32@1` is a single key whose occurrences
/// differ by the index structure they declare, so reading that attribute back off
/// each occurrence is how a program's contractions are told apart — and an
/// unrecognized structure is a panic rather than an uncounted occurrence, so the
/// census cannot silently omit one. The worked instance is this block, the first
/// program in the corpus to exercise all three structures at once.
#[test]
fn all_three_contraction_index_structures_occur_exactly_once_or_four_times() {
    // The block is the first program in the corpus exercising every structure the
    // one keyed contraction family must carry: four structure-1 projections, one
    // structure-2 score contraction, one structure-3 value contraction.
    let program = build_block(c1_extents());
    let mut projections = 0;
    let mut scores = 0;
    let mut values = 0;
    for operation in program
        .operations()
        .filter(|operation| operation.key() == &tensor_contraction_f32_op())
    {
        let attributes = operation.attributes();
        let structure = attributes
            .get(tiler_ir::semantic::CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE)
            .expect("a contraction carries its structure");
        if structure == projection_structure().canonical_value() {
            projections += 1;
        } else if structure == score_structure().canonical_value() {
            scores += 1;
        } else if structure == value_structure().canonical_value() {
            values += 1;
        } else {
            panic!("an unexpected contraction structure occurred");
        }
    }
    assert_eq!((projections, scores, values), (4, 1, 1));
}

/// Each occurrence's key and its *extent-free* attributes.
///
/// A [`BroadcastAxisMapping`] carries its declared result extents in the same
/// canonical record as its per-axis relations, so its attribute bytes are a
/// function of the row. This signature drops that one field and keeps the
/// relations, which is the part that states what the broadcast *means*; every
/// other family's attributes are already extent-free — a reindex form names axes
/// and factors, a contraction structure names indices, a reduction names an axis.
fn occurrence_signature(program: &SemanticProgram) -> Vec<(OpKey, String)> {
    program
        .operations()
        .map(|operation| {
            let attributes = operation.attributes();
            let rendered = if operation.key() == &broadcast_f32_op() {
                let mapping = attributes
                    .get(BROADCAST_AXIS_MAPPING_ATTRIBUTE)
                    .expect("a broadcast carries its mapping");
                let CanonicalValueView::Record(fields) = mapping.view() else {
                    panic!("a broadcast mapping is a record");
                };
                let sources = fields
                    .iter()
                    .find(|field| field.id() == BROADCAST_MAPPING_SOURCES)
                    .expect("a mapping states one source per result axis");
                format!("{:?}", sources.value())
            } else {
                format!("{attributes:?}")
            };
            (operation.key().clone(), rendered)
        })
        .collect()
}

/// A longer prefill row moves extents and nothing else about the graph.
///
/// **What this does and does not establish**, because the two are easy to
/// conflate. The sequence of families, the reindex forms, the contraction index
/// structures, the reduced axes, and every broadcast's per-axis relations are
/// identical at ten positions and at eighteen — so the *graph* is the same graph
/// and only the row moved.
///
/// What is *not* invariant is the declared result extents inside each broadcast's
/// mapping attribute, which move with the row and therefore reach canonical
/// identity. That is a real property of the explicit-broadcast contract rather
/// than an accident: the mapping states the shape it produces. A program that
/// stayed byte-identical across rows would need mappings that carry extent
/// *symbols*, which the semantic vocabulary does not have — a semantic value fact
/// carries a static extent. [`the_broadcast_mappings_are_the_only_row_dependent_attributes`]
/// pins that boundary down rather than leaving it implied.
#[test]
fn a_longer_row_changes_no_occurrence() {
    let prefill = build_block(c1_extents());
    let longer_environment = shape_environment(18, 18);
    let longer = build_block(
        BlockExtents::resolve(&longer_environment, C1_HIDDEN).expect("both symbols are pinned"),
    );

    assert_eq!(prefill.operation_count(), longer.operation_count());
    assert_eq!(
        occurrence_signature(&prefill),
        occurrence_signature(&longer),
        "a longer row is a binding change rather than a graph change"
    );

    // And the extents did move, so the comparison above is not vacuous.
    let context_shape = |program: &SemanticProgram| -> Shape {
        let output = program
            .outputs()
            .nth(1)
            .expect("k_rope is the second output");
        program
            .shape(output.value())
            .expect("an output value has a shape")
            .as_static()
            .expect("a statically authored program has a fixed output shape")
            .clone()
    };
    assert_eq!(context_shape(&prefill), block_shape([GROUPS, 10, HEAD_DIM]));
    assert_eq!(context_shape(&longer), block_shape([GROUPS, 18, HEAD_DIM]));
}

/// An explicit broadcast mapping is the only attribute a row moves — and all move.
///
/// This is the complement of [`a_longer_row_changes_no_occurrence`], so neither is
/// a claim the other's formulation quietly widened. A mapping states the shape it
/// produces, so its declared result extents are a function of the binding and
/// reach canonical identity; every other family's attributes are extent-free — a
/// reindex form names axes and factors, a contraction structure names indices, a
/// reduction names an axis. Counted rather than asserted as "some": the worked
/// instance is this block's ten mappings, every one of which widens to a shape
/// with the position axis in it, so a mapping that stopped depending on the row,
/// or an eleventh that appeared, is visible here.
#[test]
fn the_broadcast_mappings_are_the_only_row_dependent_attributes() {
    // The complement of the check above, so neither is a claim the other's
    // formulation quietly widened: with the extents left *in*, exactly the ten
    // broadcast occurrences differ between the two rows and no other does.
    let prefill = build_block(c1_extents());
    let longer_environment = shape_environment(18, 18);
    let longer = build_block(
        BlockExtents::resolve(&longer_environment, C1_HIDDEN).expect("both symbols are pinned"),
    );

    let full = |program: &SemanticProgram| -> Vec<(OpKey, String)> {
        program
            .operations()
            .map(|operation| {
                (
                    operation.key().clone(),
                    format!("{:?}", operation.attributes()),
                )
            })
            .collect()
    };
    let differing: Vec<OpKey> = full(&prefill)
        .into_iter()
        .zip(full(&longer))
        .filter(|(left, right)| left.1 != right.1)
        .map(|(left, _)| left.0)
        .collect();
    assert!(
        differing.iter().all(|key| key == &broadcast_f32_op()),
        "only an explicit broadcast mapping carries a result extent"
    );

    // All ten, because every one of this block's mappings widens *to* a shape
    // with the position axis in it — the three normalization weights, the two
    // rotary signs, the four rotary tables, and the mask. Counted rather than
    // asserted as "some", so a mapping that stopped depending on the row, or an
    // eleventh that appeared, is visible here.
    assert_eq!(differing.len(), 10);
}

/// A context wider than the new positions is refused, and that is the seam.
///
/// **This is where `S` being a separate symbol earns itself, and it is not the
/// direction a reader expects.** The two symbols are never joined by an equality,
/// so the *graph* is written in terms of both — but this is the **prefill** block,
/// which computes its own key and value from its own input, so the score tensor's
/// key extent is whatever the key path produced and a mask asserting a wider
/// context does not agree with it.
///
/// So a decode step is **not** reachable by rebinding `S` alone. It is this
/// program with `k_rope` and `v_heads` arriving as *inputs* of extent `S >= T`
/// instead of being produced — which is exactly why they are named outputs here,
/// and which is the autoregressive-state work rather than this ticket's. What
/// this check pins down is that the boundary fails closed and names the operand
/// disagreement, rather than truncating a mask or broadcasting one silently.
#[test]
fn a_context_wider_than_the_new_positions_is_refused_at_prefill() {
    let environment = shape_environment(C1_POSITIONS, 18);
    let extents = BlockExtents::resolve(&environment, C1_HIDDEN)
        .expect("both symbols are pinned and bounded");
    assert_eq!(
        (extents.new_positions, extents.context),
        (10, 18),
        "the environment admits the binding; it is the graph that refuses it"
    );
    let refused = build_block_with(extents, HeadReading::Interleave)
        .expect_err("the prefill block produces its own key, so S is not free");
    assert_eq!(
        refusal_code(&refused),
        "binary.shape",
        "the mask add is where the block's own key extent meets the declared \
         context, and it refuses rather than truncating"
    );

    // The admitted neighbour: the same environment with the context bound to the
    // new positions, which is the prefill row's own relationship.
    assert!(build_block_with(c1_extents(), HeadReading::Interleave).is_ok());
}

/// Two extent symbols pinned to the same value are still two symbols.
///
/// An interval constraint is a fact about one symbol in isolation, so an
/// environment does not join two symbols' equality classes merely because a row
/// binds them to the same number — proving each positive and pinned is not
/// proving them equal. The worked instance is `T` and `S` at batch-1 prefill,
/// where `S = T` is therefore a coincidence of the binding rather than a fact of
/// the program: were it ever proved, a decode step would need a different graph
/// rather than a different binding.
#[test]
fn the_two_extent_symbols_are_never_proved_equal() {
    // Both are pinned to ten at prefill and they are still two symbols: an
    // interval is a fact about one symbol in isolation, and nothing in the
    // environment joins their equality classes. If this ever proved true, a
    // decode step would need a different graph rather than a different binding.
    let environment = shape_environment(C1_POSITIONS, C1_POSITIONS);
    let (new_positions, context) = (new_positions_symbol(), context_symbol());
    assert!(!environment.proves_equal(&new_positions, &context));
    assert!(environment.proves_positive(&new_positions));
    assert!(environment.proves_positive(&context));

    let interval = environment
        .extent_interval(&context)
        .expect("a declared symbol has an interval");
    assert_eq!((interval.lower, interval.upper), (10, 10));
    assert!(!interval.states_no_upper_bound());
}

// --- construction-time refusals -------------------------------------------------

/// Returns the provider diagnostic code a refused application carried.
///
/// Asserting the code rather than only `is_err` is what makes each refusal below
/// evidence about the rule it names: a poisoned builder, a foreign handle, or a
/// bound violation would all be errors too, and none of them would be the check.
fn refusal_code(error: &BuildError) -> String {
    let BuildError::SemanticRegistry(RegistryError::RejectedOperationApplication(rejection)) =
        error
    else {
        panic!("a block refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

/// A one-to-one mapping axis refuses an operand extent that disagrees.
///
/// `FromOperand` states a correspondence and not a resize, so an operand whose
/// extent differs from the result axis it supplies is refused under
/// `broadcast.mapping.extent-disagreement` rather than truncated, padded, or
/// stretched to fit — the widening a broadcast performs is confined to the axes
/// that declare it. The worked instance is a causal mask one key position too
/// wide: identically ranked, plausibly shaped, and unable to index the score
/// tensor it would be added to. The block's own mask follows as the admitted
/// neighbour, so the refusal discriminates the operand rather than the mapping.
#[test]
fn a_mask_against_the_wrong_key_extent_refuses_by_name() {
    let extents = c1_extents();
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    // A mask one key position too wide: identically ranked, plausibly shaped, and
    // paired with a score tensor it cannot index.
    let wrong = builder
        .input::<F32>(
            mask_key(),
            block_shape([extents.new_positions, extents.context + 1]),
        )
        .expect("an F32 input");
    assert_eq!(
        refusal_code(
            &F32Broadcast::apply(
                &mut builder,
                &mask_mapping(extents.new_positions, extents.context),
                wrong,
            )
            .unwrap_err()
        ),
        "broadcast.mapping.extent-disagreement",
        "the mapping's innermost result axis is one-to-one with the mask's own, \
         so a key extent that disagrees is refused rather than truncated"
    );

    // The admitted neighbour, so the refusal discriminates the operand rather
    // than the mapping.
    let right = builder
        .input::<F32>(
            InputKey::new("mask_right").expect("a valid key"),
            block_shape([extents.new_positions, extents.context]),
        )
        .expect("an F32 input");
    assert!(
        F32Broadcast::apply(
            &mut builder,
            &mask_mapping(extents.new_positions, extents.context),
            right
        )
        .is_ok()
    );
}

/// A weight of the wrong width is refused twice, because nothing widens implicitly.
///
/// The IR admits no implicit broadcasting, and both halves of that are checked
/// because together they say there is no route around. A family that takes a
/// weight already carrying its value's shape refuses a narrower one directly —
/// `rms-norm.f32.weight-shape` — rather than tiling it; and the explicit
/// broadcast a caller would reach for to widen it refuses first under its own
/// `broadcast.mapping.extent-disagreement`. The worked instance is the 128-wide
/// per-head normalization weight presented to the 1,024-wide input
/// normalization, which an implicit broadcast would have tiled eight times across
/// the model dimension. The block's own hidden-width weight is widened and
/// normalized afterwards, so both refusals discriminate the operand.
#[test]
fn a_per_head_norm_weight_against_the_hidden_axis_refuses_by_name() {
    let extents = c1_extents();
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let residual = builder
        .input::<F32>(
            residual_key(),
            block_shape([extents.new_positions, extents.hidden]),
        )
        .expect("an F32 input");
    let per_head = builder
        .input::<F32>(query_norm_key(), block_shape([HEAD_DIM]))
        .expect("an F32 input");

    // The 128-wide per-head weight presented to the 1,024-wide input
    // normalization. `tiler::rms-norm-f32@1` takes a weight already shaped like
    // the value, so the narrow weight is a typed refusal rather than an implicit
    // broadcast that would silently tile a head's weights eight times across the
    // model dimension.
    assert_eq!(
        refusal_code(
            &F32RmsNorm::apply(
                &mut builder,
                residual,
                per_head,
                axis(1),
                RMS_NORM_F32_REFERENCE_EPS_BITS
            )
            .unwrap_err()
        ),
        "rms-norm.f32.weight-shape"
    );

    // And the broadcast that would have had to widen it refuses first, under its
    // own rule: a `[128]` operand cannot supply a 1,024-wide result axis.
    assert_eq!(
        refusal_code(
            &F32Broadcast::apply(
                &mut builder,
                &hidden_weight_mapping(extents.new_positions, extents.hidden),
                per_head,
            )
            .unwrap_err()
        ),
        "broadcast.mapping.extent-disagreement"
    );

    // The admitted neighbour: the block's own input-normalization weight.
    let hidden_weight = builder
        .input::<F32>(input_layernorm_key(), block_shape([extents.hidden]))
        .expect("an F32 input");
    let widened = F32Broadcast::apply(
        &mut builder,
        &hidden_weight_mapping(extents.new_positions, extents.hidden),
        hidden_weight,
    )
    .expect("the hidden-width weight broadcasts");
    assert!(
        F32RmsNorm::apply(
            &mut builder,
            residual,
            widened,
            axis(1),
            RMS_NORM_F32_REFERENCE_EPS_BITS
        )
        .is_ok()
    );
}

/// A split's factors must exhaust the axis exactly, and both misses are named.
///
/// `split-axis` admits a factorization only when the product is the axis extent,
/// and it distinguishes the two ways of missing rather than reporting one
/// invalidity: a product short of the extent reads a prefix and is refused as
/// `reindex.split.not-surjective`, a product past it as
/// `reindex.split.not-total`. That is what keeps a head split from silently
/// becoming a slice or an over-read. The worked instances are the checkpoint's own
/// traps — `hidden_size / num_attention_heads` is 64 here, so sixteen heads of it
/// account for half of a 2,048-wide projection, and sixteen heads of 128 read
/// past the end of a 1,024-wide one. The block's two admitted splits follow, so
/// the refusals discriminate.
#[test]
fn a_head_split_whose_factors_do_not_multiply_out_refuses_by_name() {
    let extents = c1_extents();
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let query_flat = builder
        .input::<F32>(
            InputKey::new("q_flat").expect("a valid key"),
            block_shape([extents.new_positions, QUERY_WIDTH]),
        )
        .expect("an F32 input");
    let key_flat = builder
        .input::<F32>(
            InputKey::new("k_flat").expect("a valid key"),
            block_shape([extents.new_positions, KEY_VALUE_WIDTH]),
        )
        .expect("an F32 input");

    // The divide a planner reaches for: `hidden_size / num_attention_heads` is 64
    // on this checkpoint, and sixteen heads of 64 account for half of a 2,048-wide
    // projection, so the mapping reads a prefix and is a slice.
    let divided = ReindexForm::split_axis(axis(1), [extent(QUERY_HEADS), extent(64)])
        .expect("the form itself is well shaped");
    assert_eq!(
        refusal_code(&F32Reindex::apply(&mut builder, &divided, query_flat).unwrap_err()),
        "reindex.split.not-surjective"
    );

    // The query head count applied to the key projection: 16 x 128 reads past the
    // end of a 1,024-wide axis, so the mapping is not total.
    assert_eq!(
        refusal_code(
            &F32Reindex::apply(&mut builder, &projection_split(QUERY_HEADS), key_flat).unwrap_err()
        ),
        "reindex.split.not-total"
    );

    // The admitted neighbours: the block's own two splits.
    assert!(F32Reindex::apply(&mut builder, &projection_split(QUERY_HEADS), query_flat).is_ok());
    assert!(F32Reindex::apply(&mut builder, &projection_split(GROUPS), key_flat).is_ok());
}

/// A summed index appearing in one operand is refused at construction.
///
/// Whether an index structure is admissible is a property of its tuples alone, so
/// `contraction.rule.summed-index-in-one-operand` fires before any operand
/// exists. The rule earns itself because such a structure is a projection dressed
/// as a contraction, and nothing downstream would notice: the worked instance
/// drops the head dimension from the score structure's key operand and still
/// produces a correctly shaped `[8, 2, T, S]` result from operands that never
/// pair. The block's own score structure — the same tuples with `d` restored — is
/// the admitted neighbour.
#[test]
fn a_contracted_index_in_one_operand_refuses_by_name() {
    // The score structure with the head dimension dropped from the key operand.
    // `d` is then summed over while appearing in one operand only, which is a
    // projection dressed as a contraction: it would produce a correctly shaped
    // `[8, 2, T, S]` tensor from operands that never pair.
    let rejected = ContractionIndexStructure::new(
        [
            vec![index(G), index(R), index(T_INDEX), index(D)],
            vec![index(G), index(S_INDEX)],
        ],
        [index(G), index(R), index(T_INDEX), index(S_INDEX)],
    )
    .expect_err("a summed index must appear in both operands");
    assert_eq!(
        rejected.diagnostic_code(),
        "contraction.rule.summed-index-in-one-operand"
    );

    // The admitted neighbour, which is the block's own score structure: the same
    // tuples with `d` restored to the key operand.
    assert!(
        ContractionIndexStructure::new(
            [
                vec![index(G), index(R), index(T_INDEX), index(D)],
                vec![index(G), index(S_INDEX), index(D)],
            ],
            [index(G), index(R), index(T_INDEX), index(S_INDEX)],
        )
        .is_ok()
    );
}

/// An extent that is not a proved single point refuses rather than compiling.
///
/// A semantic value fact carries a *static* extent, so resolution has two
/// distinct refusals and not one: a symbol with no proved upper bound cannot be
/// proved against any axis — the L4 record's fourth feasibility predicate — while
/// a symbol bounded to a range is genuinely symbolic and the static shape
/// vocabulary cannot carry it. Which condition to test is not a matter of taste:
/// the decision procedure seeds every symbol at the whole extent domain and
/// narrows from there, so "the environment says nothing" reads as an upper bound
/// still at the ceiling and a caller testing for a missing interval would never
/// fire. The worked instances are a declared and bound but unconstrained `S`, and
/// a context bounded to `2..=8`; the statically bound symbol in the same
/// environment resolves, so the refusal discriminates the constraint rather than
/// every symbol.
#[test]
fn an_unbounded_extent_symbol_refuses_rather_than_compiling_a_generic_program() {
    // A symbol declared and bound but never constrained. The decision procedure
    // seeds every symbol at the whole extent domain and narrows from there, so
    // "the environment says nothing about this extent" reads as an upper bound
    // still at the domain ceiling rather than as a missing interval — which is
    // why the refusal tests `states_no_upper_bound` and not `Option::None`.
    let unbounded = ShapeSymbol::new(block_scope(), "S").expect("a valid symbol");
    let mut draft = ShapeEnvBuilder::new();
    draft
        .declare(new_positions_symbol())
        .expect("a first declaration");
    draft
        .bind(
            &new_positions_symbol(),
            RootBinding::new(
                BindingSource::Static(Extent::new(C1_POSITIONS)),
                AvailabilityPhase::CompileProfile,
                FactProvenance::StaticallyProven,
            )
            .expect("a static extent is readable from the compile profile"),
        )
        .expect("a first binding");
    draft
        .declare(unbounded.clone())
        .expect("a first declaration");
    draft
        .bind(
            &unbounded,
            RootBinding::new(
                BindingSource::InputDimension {
                    input: mask_key(),
                    axis: axis(1),
                },
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )
            .expect("an input dimension is readable at preflight"),
        )
        .expect("a first binding");
    let environment = draft.build().expect("nothing here is contradictory");

    assert_eq!(
        resolve_static_extent(&environment, &unbounded),
        Err(ExtentRefusal::NoUpperBound),
        "an extent with no proved upper bound cannot be proved against any axis"
    );
    assert_eq!(
        BlockExtents::resolve(&environment, C1_HIDDEN),
        Err(ExtentRefusal::NoUpperBound)
    );

    // The admitted neighbour: the statically bound symbol in the same
    // environment resolves, so the refusal discriminates the constraint rather
    // than refusing every symbol.
    assert_eq!(
        resolve_static_extent(&environment, &new_positions_symbol()),
        Ok(C1_POSITIONS)
    );

    // And a symbol bounded to a *range* rather than a point is refused under its
    // own reason: it is genuinely symbolic, and a semantic value fact carries a
    // static extent.
    let ranged = shape_environment_with_range(2, 8);
    assert_eq!(
        resolve_static_extent(&ranged, &context_symbol()),
        Err(ExtentRefusal::NotASinglePoint { lower: 2, upper: 8 })
    );
}

/// An environment whose context symbol is bounded to a range rather than a point.
fn shape_environment_with_range(lower: u64, upper: u64) -> ShapeEnv {
    let mut draft = ShapeEnvBuilder::new();
    for (symbol, bounds) in [
        (new_positions_symbol(), (C1_POSITIONS, C1_POSITIONS)),
        (context_symbol(), (lower, upper)),
    ] {
        draft.declare(symbol.clone()).expect("a first declaration");
        draft
            .bind(
                &symbol,
                RootBinding::new(
                    BindingSource::InputDimension {
                        input: residual_key(),
                        axis: axis(0),
                    },
                    AvailabilityPhase::LiveDevicePreflight,
                    FactProvenance::RuntimeValidated,
                )
                .expect("an input dimension is readable at preflight"),
            )
            .expect("a first binding");
        draft
            .require(SemanticInputConstraint::new(
                ExtentRelation::interval(ExtentTerm::Symbol(symbol), bounds.0, bounds.1)
                    .expect("a nonempty interval"),
                FactProvenance::FrontendRequired,
            ))
            .expect("a declared symbol");
    }
    draft.build().expect("the environment is consistent")
}

// --- the pinned reference's own bits ---------------------------------------------

/// `row_h0_t2_scores_raw`: the C1 score row before the scale, from the record.
const PINNED_SCORES_RAW: [u32; 10] = [
    0xc0f5_4448,
    0x40f2_2030,
    0xc0c2_8c53,
    0x4168_0580,
    0x40d6_28da,
    0xc00e_6ef7,
    0xc134_df10,
    0x401f_ab04,
    0x403e_cad2,
    0xbfa0_4954,
];
/// `row_h0_t2_scores_scaled`.
const PINNED_SCORES_SCALED: [u32; 10] = [
    0xbf2d_6e05,
    0x3f2b_3570,
    0xbf09_90fa,
    0x3fa4_1060,
    0x3f17_6f06,
    0xbe49_6e6b,
    0xbf7f_ca6b,
    0x3e61_ce00,
    0x3e86_e917,
    0xbde2_ade3,
];
/// `row_h0_t2_scores_masked`.
const PINNED_SCORES_MASKED: [u32; 10] = [
    0xbf2d_6e05,
    0x3f2b_3570,
    0xbf09_90fa,
    0xff7f_ffff,
    0xff7f_ffff,
    0xff7f_ffff,
    0xff7f_ffff,
    0xff7f_ffff,
    0xff7f_ffff,
    0xff7f_ffff,
];
/// `row_h0_t2_probs`.
const PINNED_PROBABILITIES: [u32; 10] = [
    0x3e2a_db30,
    0x3f24_260a,
    0x3e44_8ca6,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
];
/// `mask_row_t2`.
const PINNED_MASK_ROW: [u32; 10] = [
    0x8000_0000,
    0x8000_0000,
    0x8000_0000,
    0xff7f_ffff,
    0xff7f_ffff,
    0xff7f_ffff,
    0xff7f_ffff,
    0xff7f_ffff,
    0xff7f_ffff,
    0xff7f_ffff,
];
/// `row_h0_t0_probs`: one attended position and nine exact positive zeros.
const PINNED_T0_PROBABILITIES: [u32; 10] = [
    0x3f80_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
];

/// Runs operations 16, 17 and 18 over one score row at the C1 row's own width.
///
/// The score tensor is `[8, 2, 1, 10]` and the mask `[1, 10]`, so this is **the
/// block's own [`mask_mapping`]** at one query position rather than a reduced
/// stand-in for it: the two head axes are the workload's eight groups and two
/// repetitions, and the mask is replicated across all sixteen exactly as it is in
/// the block. A one-wide head axis would have been refused — `Replicate` onto a
/// result extent of one widens nothing — so using the real extents is what makes
/// this reachable at all.
///
/// The pinned row is written into every one of the sixteen head slabs, and each
/// is compared, so a mapping that reached the wrong slab fails here. Three
/// outputs, so each of the record's three rows is compared against the step that
/// produced it rather than only against the last one.
fn scale_mask_and_softmax(scores: &[u32; 10], mask: &[u32; 10]) -> [Vec<u32>; 3] {
    const SLABS: usize = GROUPS * REPEATS;
    let score_shape = block_shape([GROUPS, REPEATS, 1, 10]);
    let mask_shape = block_shape([1, 10]);

    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let score_key = InputKey::new("scores").expect("a valid key");
    let mask_input_key = InputKey::new("mask").expect("a valid key");
    let score_value = builder
        .input::<F32>(score_key.clone(), score_shape.clone())
        .expect("an F32 input");
    let mask_value = builder
        .input::<F32>(mask_input_key.clone(), mask_shape.clone())
        .expect("an F32 input");

    let scale = F32Constant::apply(&mut builder, ATTENTION_SCALE_BITS).expect("a scalar constant");
    let scaled = F32Multiply::apply(&mut builder, score_value, scale).expect("a rank-zero operand");
    let widened = F32Broadcast::apply(&mut builder, &mask_mapping(1, 10), mask_value)
        .expect("the block's own mask mapping at one query position");
    let masked = F32Add::apply(&mut builder, scaled, widened).expect("both operands agree");
    let probabilities =
        F32Softmax::apply(&mut builder, masked, axis(3)).expect("axis 3 is the key axis");

    for (name, value) in [
        ("scaled", scaled),
        ("masked", masked),
        ("probs", probabilities),
    ] {
        builder
            .output(OutputKey::new(name).expect("a valid key"), value)
            .expect("a first output key");
    }
    let program = builder.build().expect("the row program is complete");

    let score_tensor = tensor_of_bits(&score_shape, |position| scores[position % 10]);
    let mask_tensor = tensor_of_bits(&mask_shape, |lane| mask[lane]);
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .evaluate(
            &program,
            &[
                InputBinding::new(&score_key, &score_tensor),
                InputBinding::new(&mask_input_key, &mask_tensor),
            ],
        )
        .expect("the row program evaluates");
    let [scaled, masked, probabilities] = outputs.as_slice() else {
        panic!("the row program has three outputs");
    };

    // Every head slab carries the same row, so a mapping that widened the mask
    // wrongly would leave the slabs disagreeing rather than all being wrong the
    // same way.
    let one_row_of = |tensor: &Tensor| -> Vec<u32> {
        let bits = payload_bits(tensor);
        assert_eq!(bits.len(), SLABS * 10);
        for slab in 1..SLABS {
            assert_eq!(
                bits[slab * 10..(slab + 1) * 10],
                bits[0..10],
                "the mask is replicated across all sixteen head slabs"
            );
        }
        bits[0..10].to_vec()
    };
    [
        one_row_of(scaled),
        one_row_of(masked),
        one_row_of(probabilities),
    ]
}

/// The block's operations 16, 17 and 18 reproduce `transformers` 4.51.0's bits.
///
/// This is the one comparison in this file that is against the pinned reference's
/// *own* numbers rather than against a recomputation. It needs no `torch` seed
/// because the record retains the score row before the scale, so the three rows
/// after it are a function of this block's own operations and nothing else.
#[test]
fn the_pinned_c1_score_row_is_reproduced_bit_for_bit() {
    let [scaled, masked, probabilities] =
        scale_mask_and_softmax(&PINNED_SCORES_RAW, &PINNED_MASK_ROW);
    assert_eq!(
        scaled, PINNED_SCORES_SCALED,
        "the scale is 0x3db504f3 applied to the score"
    );
    assert_eq!(
        masked, PINNED_SCORES_MASKED,
        "an attended entry is negative zero and is value-preserving on every \
         score except +0.0; a masked entry overwrites"
    );
    assert_eq!(
        probabilities, PINNED_PROBABILITIES,
        "the row maximum subtracted and the denominator's reciprocal multiplied"
    );

    // The seven masked positions receive exactly +0.0, and the positions that do
    // are exactly the seven the mask marks — the property that makes a finite
    // fill behave like an exclusion without being one.
    // `row_h0_t2_exactly_positive_zero_positions` is `3 4 5 6 7 8 9`.
    let zeros: Vec<usize> = probabilities
        .iter()
        .enumerate()
        .filter(|(_, bits)| **bits == 0x0000_0000)
        .map(|(position, _)| position)
        .collect();
    assert_eq!(zeros, vec![3, 4, 5, 6, 7, 8, 9]);

    // `row_h0_t2_probs_sum_is_exactly_one` is True at this row and is *not* a
    // property a conformance check may assert generally: 49 of the tensor's 160
    // rows do not sum to exactly one, so a check written from this row alone
    // would be wrong on those.
    assert_eq!(
        strict_fold(probabilities.iter().copied().map(f32::from_bits)).to_bits(),
        0x3f80_0000
    );

    // The perturbation, which is the record's own `prescaled_query_differing_
    // elements_from_scaled_score` finding in miniature: multiplying before the
    // contraction is a different binary32 computation, so the graph position of
    // operation 16 is semantics. Scaling the raw score row by the constant twice
    // and once are different values, and this row discriminates them.
    let [double_scaled, _, _] = scale_mask_and_softmax(&PINNED_SCORES_SCALED, &PINNED_MASK_ROW);
    assert_ne!(double_scaled, PINNED_SCORES_SCALED);
}

/// A masked position contributes a *signed* zero, and the mask rewrites the sign.
///
/// At query position 0 the probability row is `1.0` followed by nine exact `+0.0`,
/// so each masked position contributes `+0.0 * v` — which is `-0.0` wherever `v`
/// is negative. The fold is seeded from the first product rather than from `+0.0`,
/// so a first product of `-0.0` survives every later `-0.0` and is rewritten to
/// `+0.0` by the first `+0.0`.
///
/// Retained because a schedule that skipped masked contributors as a
/// causal-structure optimization would return the other sign, on an operand the
/// workload can actually contain, and nothing else in the corpus would notice.
#[test]
fn a_masked_position_contributes_a_signed_zero_to_the_value_contraction() {
    let contract = |values: &[f32]| -> Vec<u32> {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the standard builder opens");
        let probability_key = InputKey::new("probs").expect("a valid key");
        let value_key = InputKey::new("v").expect("a valid key");
        let probability_shape = block_shape([1, 1, 1, 10]);
        let value_shape = block_shape([1, 10, 1]);
        let probabilities = builder
            .input::<F32>(probability_key.clone(), probability_shape.clone())
            .expect("an F32 input");
        let value_heads = builder
            .input::<F32>(value_key.clone(), value_shape.clone())
            .expect("an F32 input");
        let result = F32TensorContraction::apply(
            &mut builder,
            &value_structure(),
            probabilities,
            value_heads,
        )
        .expect("grts,gsd->grtd is admitted");
        builder
            .output(OutputKey::new("ctx").expect("a valid key"), result)
            .expect("a first output key");
        let program = builder.build().expect("the program is complete");

        let probability_tensor =
            tensor_of_bits(&probability_shape, |lane| PINNED_T0_PROBABILITIES[lane]);
        let value_tensor = tensor_of(&value_shape, values);
        let outputs = ReferenceEvaluator::standard()
            .expect("the standard evaluator opens")
            .evaluate(
                &program,
                &[
                    InputBinding::new(&probability_key, &probability_tensor),
                    InputBinding::new(&value_key, &value_tensor),
                ],
            )
            .expect("the program evaluates");
        payload_bits(&outputs[0])
    };

    // Every contributor negative: the attended value is `-0.0` and every masked
    // contributor is `+0.0 * negative`, which is `-0.0`.
    let mut all_negative = vec![-1.0_f32; 10];
    all_negative[0] = -0.0;
    assert_eq!(
        contract(&all_negative),
        vec![0x8000_0000],
        "an unseeded fold of ten negative zeros returns -0.0 where a +0.0-seeded \
         one would return +0.0"
    );

    // The record's own C1 observation: `value_contraction_t0_lane0_first_product`
    // is 0x80000000 and `..._strict_ascending_fold` is 0x00000000, because one
    // masked contributor whose value is positive rewrites the sign.
    let mut one_positive = all_negative.clone();
    one_positive[1] = 1.0;
    assert_eq!(
        contract(&one_positive),
        vec![0x0000_0000],
        "fl(-0.0 + +0.0) is +0.0, so a single attended-sign contributor at a \
         masked position flips the result's sign"
    );
}

// --- the reference work bound ----------------------------------------------------

/// A default evaluator still refuses the C1 projections, and says why.
///
/// A fold of more than `MAX_REFERENCE_TENSOR_ELEMENTS` multiply-accumulate steps
/// is more than one uninterrupted walk of a contraction's iteration space may
/// cost, and an evaluator whose caller stated no allowance is held to exactly that
/// number. Both of the C1 row's dense projections are 20,971,520 steps —
/// `10 * 2048 * 1024` for the query and `10 * 1024 * 2048` for the output — so a
/// default evaluator refuses both, exactly as it did before this block evaluated
/// at this row.
///
/// **This is the check the end-to-end test's allowance must not have removed**,
/// which is why it is watched and quoted here rather than derived. Three asks
/// against one program isolate what the allowance does: the default refuses, one
/// step short of the fold refuses under the *stated* number, and the fold's own
/// step count evaluates. The extents never move, so nothing here can be explained
/// by the block instead of by the fold's size.
#[test]
fn the_reference_work_bound_refuses_the_c1_projections() {
    let extents = c1_extents();
    let program = build_block(extents);
    let fixture = BlockFixture::new(extents);
    let owned = fixture.bindings();
    let bindings: Vec<InputBinding<'_>> = owned
        .iter()
        .map(|(key, tensor)| InputBinding::new(key, tensor))
        .collect();
    let evaluator = ReferenceEvaluator::standard().expect("the standard evaluator opens");
    assert_eq!(
        evaluator.iteration_step_allowance(),
        16_777_216,
        "an evaluator nobody told otherwise carries the number it always did"
    );
    let refused = evaluator
        .evaluate(&program, &bindings)
        .expect_err("the C1 query projection exceeds one window's work bound");
    let message = refused.to_string();
    assert!(
        message.contains("iteration space has 20971520 steps, exceeding 16777216"),
        "the refusal names the exact step count and the exact bound: {message}"
    );

    // A stated allowance one step short of the block's largest fold, so
    // authorizing work is watched as a number that can still say no.
    let refused = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .with_iteration_step_allowance(C1_LARGEST_FOLD - 1)
        .evaluate(&program, &bindings)
        .expect_err("an allowance below the largest fold refuses it");
    let message = refused.to_string();
    assert!(
        message.contains(&format!(
            "iteration space has {C1_LARGEST_FOLD} steps, exceeding {}",
            C1_LARGEST_FOLD - 1
        )),
        "a stated allowance names itself and the fold it declined: {message}"
    );

    // The admitted neighbour, at the same extents: one more step of allowance and
    // the identical program evaluates.
    let _ = evaluate_block(&program, &fixture, C1_LARGEST_FOLD);
}

// --- the end-to-end comparison ----------------------------------------------------

/// Every one of the block's forty-eight occurrences, evaluated end to end.
///
/// **At the C1 row's own extents, with nothing reduced**: ten new positions, ten
/// context positions, a 1,024-wide model dimension, sixteen query heads over eight
/// groups, head dimension 128, the causal mask, the scale, the rotary composition,
/// and all three contraction index structures.
///
/// The two 20,971,520-step projections exceed what one uninterrupted walk of a
/// contraction's iteration space may cost, so the evaluator is given
/// [`C1_LARGEST_FOLD`] as its per-occurrence iteration-step allowance and folds
/// each of them in several bounded windows.
/// [`the_reference_work_bound_refuses_the_c1_projections`] is the other half of
/// that: it drives this same program with no allowance stated and with one step
/// too few, and watches both refuse.
///
/// The expectation is the independent recomputation, whose boundary
/// [`recompute_block`] states — and which is what makes the windowing checkable
/// here rather than assumed: `recompute_block` folds each projection in one pass
/// of a hand-written loop, so a window boundary that had moved a value would
/// disagree with it.
#[test]
fn the_block_evaluates_end_to_end_against_an_independent_recomputation() {
    // The premise this test rests on, and the reason it needs an allowance at all:
    // the block's largest fold is over what one window may walk, so each of the
    // two projections is spent as several. `w_prefill_q` in
    // `contraction_profile_cells.rs` is this block's operation 2 at these exact
    // extents, and asserts that window count is two.
    const { assert!(C1_LARGEST_FOLD > 16_777_216) };
    let extents = c1_extents();
    let program = build_block(extents);
    let fixture = BlockFixture::new(extents);

    let [residual_out, key_rope, value_heads] = evaluate_block(&program, &fixture, C1_LARGEST_FOLD);
    let expected = recompute_block(&fixture);

    assert_eq!(
        differing(&key_rope, &expected.key_rope),
        0,
        "the key path — normalization, projection, head split, per-head \
         normalization, rotary, and the group-major permutation — is bit for bit"
    );
    assert_eq!(
        differing(&value_heads, &expected.value_heads),
        0,
        "the value path is bit for bit"
    );
    assert_eq!(
        differing(&residual_out, &expected.residual_out),
        0,
        "the whole block, through both attention contractions and the residual, \
         is bit for bit"
    );
    assert_eq!(residual_out.len(), 10 * C1_HIDDEN);
    assert_eq!(key_rope.len(), GROUPS * 10 * HEAD_DIM);

    // The perturbation, so the three zeros above are properties of the
    // composition rather than of a comparison that cannot fail. Reading the key
    // head as `h % 8` instead of `h / 2` is the grouped-query mistake the probe
    // measures wrong at fourteen of the sixteen heads. It is a `(2, 8)` split and
    // a transpose — a graph this same family admits, of the same occurrence count
    // and the same result shape — so nothing structural separates the two and only
    // a value comparison discriminates them.
    let tiled = build_block_with(extents, HeadReading::Tile).expect("a prefill row");
    assert_eq!(
        tiled.operation_count(),
        program.operation_count(),
        "the two readings differ in two attributes and in nothing else"
    );
    let [tiled_residual, tiled_key_rope, tiled_value_heads] =
        evaluate_block(&tiled, &fixture, C1_LARGEST_FOLD);
    assert_ne!(
        differing(&tiled_residual, &expected.residual_out),
        0,
        "repeat-tile pairs fourteen of sixteen query heads with a different key \
         head, and the residual carries that difference"
    );

    // The two outputs that sit *upstream* of the head pairing are unmoved by it,
    // which is what localizes the perturbation to step 12 rather than leaving it
    // as a difference somewhere in forty-eight occurrences.
    assert_eq!(differing(&tiled_key_rope, &expected.key_rope), 0);
    assert_eq!(differing(&tiled_value_heads, &expected.value_heads), 0);

    // And the two readings coincide at exactly two of the sixteen heads, which is
    // the record's `gqa_heads_whose_source_differs_between_the_two_readings`.
    let differing_heads = (0..QUERY_HEADS)
        .filter(|head| HeadReading::Interleave.group_of(*head) != HeadReading::Tile.group_of(*head))
        .count();
    assert_eq!(differing_heads, 14);
}
