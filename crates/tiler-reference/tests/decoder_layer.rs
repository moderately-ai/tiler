//! One complete decoder layer of the pinned checkpoint — attention, MLP, both
//! residuals, and the two sequence extensions — as a single verified semantic
//! program with three ordered named outputs.
//!
//! # What this is
//!
//! The [complete-model record](../../../docs/research/program-planning/complete-model-ingestion-and-execution.md)
//! calls this program **P2** and gives it eighteen ordered inputs, three ordered
//! named outputs, and twenty-eight executions per forward pass. It is the
//! [attention block](causal_self_attention_block.rs)'s twenty-two steps with
//! three additions, each from a named record:
//!
//! - **The two sequence extensions**, exactly where
//!   [the autoregressive-state record](../../../docs/research/runtime/autoregressive-state-and-kv-cache.md)
//!   places them: steps 13 and 14 each feed one `tiler::concatenate-f32@1`, whose
//!   other operand is a cached tensor and whose result is the retained output the
//!   score and value contractions read.
//! - **The MLP half**, `down(silu(gate(x)) * up(x))` over `[T, 3072]`
//!   intermediates, which introduces no family its constituents do not.
//! - **The second residual add.**
//!
//! Nothing here registers an operation, adds a form, admits a structure, or
//! widens a bound. Every occurrence is one of ten already-registered keys, and
//! the layer is a *shape* over them.
//!
//! # What the counts are, measured rather than estimated
//!
//! [`the_layer_verifies_at_the_c1_prefill_row`] measures them and this file's
//! constants carry them: **eighteen inputs, three outputs, fifty-eight
//! occurrences, and seventy-six values** at the C1 prefill row. The record's
//! derived floors were "at least fifty-one occurrences over at least twenty-one
//! boundary values", and both are replaced by the measured numbers. At the
//! decode row the same layer carries sixty-two occurrences and eighty values,
//! for the reason the next section gives.
//!
//! # The decode row is a different graph, and that is this file's main finding
//!
//! The records say a decode step is this program with a different binding. That
//! is true of the *cache* and false of the *new-position count*, and the two are
//! separated here rather than left as one claim.
//!
//! - **A nonempty cache is a binding change.** At `T = 10` the layer built with
//!   `C = 0` and with `C = 8` has the identical occurrence sequence, families,
//!   forms, structures and broadcast relations — only extents move
//!   ([`a_nonempty_cache_changes_no_occurrence`]).
//! - **A single new position is not.** At `T = 1` the position axis of a
//!   normalization weight or a rotary sign duplicates nothing, and
//!   `tiler::broadcast-f32@2` refuses a many-to-one relation onto an extent-one
//!   result axis under `broadcast.mapping.relation-does-not-widen` — its own
//!   documentation names the replacement, "a reindex's unit-axis insertion". So
//!   six widenings change spelling and the layer carries **sixty-two**
//!   occurrences at the decode row rather than fifty-eight
//!   ([`a_single_new_position_changes_six_widenings`]). The refusal is watched
//!   rather than asserted, in [`a_rank_pad_onto_a_single_position_refuses`].
//!
//! Both rows verify and both reference-evaluate; what is not true is that one
//! graph serves both, and a reader who needed that for artifact-identity reuse
//! needs a semantic vocabulary that can carry an extent *symbol* into a mapping.
//!
//! # Where the compared bits come from, and the boundary
//!
//! **The pinned reference's own bits**, for the operations the pinned record
//! covers. The [attention-block probe]'s retained record holds a four-step chain
//! for query head 0 at query position 2 of the C1 prefill score tensor, and
//! [`the_pinned_c1_score_row_is_reproduced_bit_for_bit`] drives it through this
//! layer's own operations 16, 17 and 18 under this layer's own `mask_mapping`.
//! The same record supplies the mask's two fill values, which
//! [`the_generated_mask_row_is_the_pinned_one`] compares against this file's
//! generated mask.
//!
//! **The boundary, stated rather than implied: the pinned record holds no MLP,
//! cache, or decode row.** Its keys are the rotary composition, the grouped-query
//! reading, the mask, the score chain, the softmax rows, and the eager attention
//! output; there is no `gate`, `up`, `down`, `silu`, or `k_cache` observable in
//! it, and the C1 conformance fixture retains per-layer hidden states as
//! regenerable local data rather than as in-tree bits. So the MLP half and both
//! rows' end-to-end results are compared against an **independent
//! recomputation** written out from the operation table by explicit coordinate
//! arithmetic — every access relation, coordinate map, broadcast, head pairing,
//! concatenation offset, and composition order restated rather than re-run.
//!
//! That recomputation's own boundary: the scalar arithmetic of the three
//! non-linear families is the crate's certified [`rms_norm_f32`], [`softmax_f32`]
//! and [`silu_f32`] rather than a second implementation, so the comparison
//! discriminates a wrong index binding, a wrong coordinate map, a wrong head
//! pairing, a wrong concatenation order, a wrong gating, or a wrong composition
//! order, and is silent about the binary32 results of `rsqrt` and `exp`
//! themselves — which the normalization, softmax and `SiLU` corpora own.
//!
//! This file establishes nothing about a plan, a schedule, a cover, a kernel, a
//! device, or any layer-level numeric tolerance, and it deliberately does not
//! compile the program: the deterministic budgets refuse it, and widening them
//! moves every artifact identity in the corpus.
//!
//! [attention-block probe]: ../../../spikes/program-planning/attention-block-reference/README.md

use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::semantic::{
    BROADCAST_AXIS_MAPPING_ATTRIBUTE, BROADCAST_MAPPING_SOURCES, BroadcastAxisMapping,
    BroadcastAxisSource, BuildError, CanonicalValueView, ContractionIndex,
    ContractionIndexStructure, F32, F32Add, F32Broadcast, F32Concatenate, F32Constant, F32Multiply,
    F32Reindex, F32RmsNorm, F32Silu, F32Softmax, F32TensorContraction, InputKey, OpKey, OutputKey,
    RMS_NORM_F32_REFERENCE_EPS_BITS, RegistryError, ReindexForm, SemanticProgram,
    SemanticProgramBuilder, Value, add_f32_op, broadcast_f32_op, concatenate_f32_op,
    constant_f32_op, multiply_f32_op, reindex_f32_op, rms_norm_f32_op, silu_f32_op, softmax_f32_op,
    tensor_contraction_f32_op,
};
use tiler_ir::shape::{
    Axis, BindingSource, Extent, ExtentRelation, ExtentTerm, FactProvenance, FragmentViolation,
    RootBinding, SemanticInputConstraint, Shape, ShapeEnv, ShapeEnvBuilder, ShapeEnvError,
    ShapeSymbol, SymbolScope,
};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
    rms_norm_f32, silu_f32, softmax_f32,
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
/// The checkpoint's `max_position_embeddings`, which bounds all three symbols.
const MAX_POSITION_EMBEDDINGS: u64 = 32_768;

/// The C1 conformance row's prefill new-position count.
const C1_POSITIONS: u64 = 10;
/// The C1 row's model dimension, which the dense projections are sized by.
const C1_HIDDEN: usize = 1_024;
/// The checkpoint's `intermediate_size`, which the MLP's three weights declare.
const INTERMEDIATE: usize = 3_072;

/// The cached-position count of the C1 row's eighth and last decode step.
const C1_DECODE_CACHED: u64 = 17;
/// One decode step produces exactly one new position.
const C1_DECODE_POSITIONS: u64 = 1;
/// `S = C + T` at that step, and the row's declared maximum context.
const C1_DECODE_CONTEXT: u64 = C1_DECODE_CACHED + C1_DECODE_POSITIONS;

/// The largest fold any occurrence performs at the C1 prefill row.
///
/// The MLP's three contractions are the layer's largest: `gate` and `up` each
/// fold `10 * 3072` output elements over the 1,024-wide model dimension and
/// `down` folds `10 * 1024` over the 3,072-wide intermediate, all three at
/// 31,457,280 multiply-accumulate steps. The attention half's two largest — the
/// query and output projections — are 20,971,520 each, so the MLP raised this
/// layer's allowance above the attention block's.
///
/// Written as the layer's own arithmetic rather than as a round number, so an
/// occurrence needing one step more is refused rather than quietly admitted.
const C1_PREFILL_LARGEST_FOLD: usize = 10 * INTERMEDIATE * C1_HIDDEN;

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
const FIXTURE_SEED: u64 = 0x0000_c1a7_7e17_0006;

fn axis(value: u32) -> Axis {
    Axis::new(value)
}

fn extent(value: usize) -> Extent {
    Extent::new(u64::try_from(value).expect("a workload extent fits a u64"))
}

fn layer_shape<const N: usize>(dims: [usize; N]) -> Shape {
    Shape::try_from_dims(
        dims.into_iter()
            .map(|dim| u64::try_from(dim).expect("a workload extent fits a u64")),
    )
    .expect("a workload shape is admitted")
}

// --- the shape environment ---------------------------------------------------

/// The scope all three extent symbols are declared in.
fn layer_scope() -> SymbolScope {
    SymbolScope::new("decoder-layer/0").expect("a nonempty scope")
}

/// `T`, the new-position count.
fn new_positions_symbol() -> ShapeSymbol {
    ShapeSymbol::new(layer_scope(), "T").expect("a valid symbol")
}

/// `C`, the cached-position count.
fn cached_symbol() -> ShapeSymbol {
    ShapeSymbol::new(layer_scope(), "C").expect("a valid symbol")
}

/// `S`, the total context length.
fn context_symbol() -> ShapeSymbol {
    ShapeSymbol::new(layer_scope(), "S").expect("a valid symbol")
}

/// Why one symbolic extent could not become a static shape extent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExtentRefusal {
    Undeclared,
    NoUpperBound,
    NotASinglePoint { lower: u64, upper: u64 },
}

/// Resolves one symbolic extent to the static extent a semantic value fact carries.
///
/// `ExtentInterval`'s contract is that a symbol nothing constrains is *present
/// with the whole extent domain* rather than absent, so the condition to test is
/// `states_no_upper_bound` and never a missing interval.
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

/// Declares `T`, `C` and `S`, relates them by `S == C + T`, and pins each to a row.
///
/// Each is bound to an input dimension rather than to a static extent, because
/// that is where the value comes from: `T` is `x`'s outermost axis, `C` is
/// `k_cache`'s sequence axis, and `S` is the mask's innermost. None carries a
/// compile-time value, so the row enters as frontend-required interval
/// constraints and the relation is retained rather than statically discharged.
///
/// **`C` is bounded from zero and the other two from one**, because prefill binds
/// an empty cache and a zero-extent operand is what the concatenation family
/// admits — while a program with no new positions and no context computes
/// nothing.
///
/// # Errors
///
/// Returns the environment's own error when the three pinned values do not
/// satisfy `S == C + T`; [`the_extent_relation_refuses_an_inconsistent_row`]
/// watches that.
fn shape_environment(
    new_positions: u64,
    cached: u64,
    context: u64,
) -> Result<ShapeEnv, ShapeEnvError> {
    let mut draft = ShapeEnvBuilder::new();
    for (symbol, source, floor, row) in [
        (
            new_positions_symbol(),
            BindingSource::InputDimension {
                input: residual_key(),
                axis: axis(0),
            },
            1,
            new_positions,
        ),
        (
            cached_symbol(),
            BindingSource::InputDimension {
                input: key_cache_key(),
                axis: axis(1),
            },
            0,
            cached,
        ),
        (
            context_symbol(),
            BindingSource::InputDimension {
                input: mask_key(),
                axis: axis(1),
            },
            1,
            context,
        ),
    ] {
        draft.declare(symbol.clone())?;
        draft.bind(
            &symbol,
            RootBinding::new(
                source,
                AvailabilityPhase::LiveDevicePreflight,
                FactProvenance::RuntimeValidated,
            )?,
        )?;
        // The checkpoint's own ceiling, and then the row. Both are stated: the
        // ceiling is what makes the symbol bounded at all, and a context past it
        // refuses rather than compiling.
        draft.require(SemanticInputConstraint::new(
            ExtentRelation::interval(
                ExtentTerm::Symbol(symbol.clone()),
                floor,
                MAX_POSITION_EMBEDDINGS,
            )?,
            FactProvenance::FrontendRequired,
        ))?;
        draft.require(SemanticInputConstraint::new(
            ExtentRelation::interval(ExtentTerm::Symbol(symbol), row, row)?,
            FactProvenance::FrontendRequired,
        ))?;
    }
    draft.require(SemanticInputConstraint::new(
        ExtentRelation::additive_equality(
            ExtentTerm::Symbol(context_symbol()),
            ExtentTerm::Symbol(cached_symbol()),
            ExtentTerm::Symbol(new_positions_symbol()),
        ),
        FactProvenance::FrontendRequired,
    ))?;
    draft.build()
}

// --- the layer's ordered interface -------------------------------------------

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
fn key_cache_key() -> InputKey {
    InputKey::new("k_cache").expect("a valid key")
}
fn value_cache_key() -> InputKey {
    InputKey::new("v_cache").expect("a valid key")
}
fn post_attention_layernorm_key() -> InputKey {
    InputKey::new("w_post_attention_layernorm").expect("a valid key")
}
fn gate_weight_key() -> InputKey {
    InputKey::new("W_gate").expect("a valid key")
}
fn up_weight_key() -> InputKey {
    InputKey::new("W_up").expect("a valid key")
}
fn down_weight_key() -> InputKey {
    InputKey::new("W_down").expect("a valid key")
}

/// The layer's eighteen ordered input keys: the attention block's twelve, then
/// the two cached tensors, then the MLP's four.
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
        key_cache_key(),
        value_cache_key(),
        post_attention_layernorm_key(),
        gate_weight_key(),
        up_weight_key(),
        down_weight_key(),
    ]
}

/// The layer's three ordered output keys: the residual stream, then the KV seam.
fn ordered_output_keys() -> Vec<OutputKey> {
    vec![
        OutputKey::new("h_out").expect("a valid key"),
        OutputKey::new("k_rope").expect("a valid key"),
        OutputKey::new("v_heads").expect("a valid key"),
    ]
}

/// Every static extent one instantiation of the layer is built at.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LayerExtents {
    new_positions: usize,
    cached: usize,
    context: usize,
    hidden: usize,
    intermediate: usize,
}

impl LayerExtents {
    /// Reads the three symbolic extents out of the environment.
    ///
    /// The model dimension and the intermediate width are checkpoint constants
    /// rather than symbols, because nothing in the workload varies them.
    fn resolve(
        environment: &ShapeEnv,
        hidden: usize,
        intermediate: usize,
    ) -> Result<Self, ExtentRefusal> {
        let read = |symbol: &ShapeSymbol| -> Result<usize, ExtentRefusal> {
            resolve_static_extent(environment, symbol)
                .map(|value| usize::try_from(value).expect("a row extent fits this host"))
        };
        Ok(Self {
            new_positions: read(&new_positions_symbol())?,
            cached: read(&cached_symbol())?,
            context: read(&context_symbol())?,
            hidden,
            intermediate,
        })
    }
}

/// The eighteen declared input handles.
#[derive(Clone, Copy)]
struct LayerInputs {
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
    key_cache: Value<F32>,
    value_cache: Value<F32>,
    post_attention_layernorm: Value<F32>,
    gate_weight: Value<F32>,
    up_weight: Value<F32>,
    down_weight: Value<F32>,
}

/// Declares the eighteen inputs, in the record's order.
fn declare_inputs(builder: &mut SemanticProgramBuilder, extents: LayerExtents) -> LayerInputs {
    let LayerExtents {
        new_positions: t,
        cached: c,
        context: s,
        hidden,
        intermediate,
    } = extents;
    let mut declare = |key: InputKey, shape: Shape| {
        builder
            .input::<F32>(key, shape)
            .expect("a first declaration of an F32 input")
    };
    LayerInputs {
        residual: declare(residual_key(), layer_shape([t, hidden])),
        input_layernorm: declare(input_layernorm_key(), layer_shape([hidden])),
        query_weight: declare(query_weight_key(), layer_shape([QUERY_WIDTH, hidden])),
        key_weight: declare(key_weight_key(), layer_shape([KEY_VALUE_WIDTH, hidden])),
        value_weight: declare(value_weight_key(), layer_shape([KEY_VALUE_WIDTH, hidden])),
        query_norm: declare(query_norm_key(), layer_shape([HEAD_DIM])),
        key_norm: declare(key_norm_key(), layer_shape([HEAD_DIM])),
        cosine: declare(cosine_key(), layer_shape([t, HEAD_DIM])),
        sine: declare(sine_key(), layer_shape([t, HEAD_DIM])),
        rope_sign: declare(rope_sign_key(), layer_shape([HALVES, 1])),
        mask: declare(mask_key(), layer_shape([t, s])),
        output_weight: declare(output_weight_key(), layer_shape([hidden, QUERY_WIDTH])),
        key_cache: declare(key_cache_key(), layer_shape([GROUPS, c, HEAD_DIM])),
        value_cache: declare(value_cache_key(), layer_shape([GROUPS, c, HEAD_DIM])),
        post_attention_layernorm: declare(post_attention_layernorm_key(), layer_shape([hidden])),
        gate_weight: declare(gate_weight_key(), layer_shape([intermediate, hidden])),
        up_weight: declare(up_weight_key(), layer_shape([intermediate, hidden])),
        down_weight: declare(down_weight_key(), layer_shape([hidden, intermediate])),
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

/// Structure 1, `td,od->to`: the four attention projections and the MLP's three.
fn projection_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [vec![index(T_INDEX), index(D)], vec![index(O), index(D)]],
        [index(T_INDEX), index(O)],
    )
    .expect("td,od->to is admitted")
}

/// Structure 2, `grtd,gsd->grts`: the score contraction.
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

/// Structure 3, `grts,gsd->grtd`: the value contraction, over the grown `S`.
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

/// `[hidden] -> [T, hidden]`: a normalization weight's rank pad, at `T >= 2`.
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

/// `[128] -> [T, heads, 128]`: a per-head normalization weight's rank pad, at `T >= 2`.
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

/// `[128] -> [heads, 128]`: the same weight with the position axis left off.
fn head_weight_mapping_over_heads(heads: usize) -> BroadcastAxisMapping {
    BroadcastAxisMapping::new(
        [extent(heads), extent(HEAD_DIM)],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(axis(0)),
        ],
    )
    .expect("the mapping accounts for every result axis")
}

/// `[2, 1] -> [T, heads, 2, 64]`: the rotary sign operand, at `T >= 2`.
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

/// `[2, 1] -> [heads, 2, 64]`: the same operand with the position axis left off.
fn sign_mapping_over_heads(heads: usize) -> BroadcastAxisMapping {
    BroadcastAxisMapping::new(
        [extent(heads), extent(HALVES), extent(HALF)],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(axis(0)),
            BroadcastAxisSource::StretchUnit(axis(1)),
        ],
    )
    .expect("the mapping accounts for every result axis")
}

/// `[T, 128] -> [T, heads, 128]`: a rotary table, over an *interior* rank pad.
///
/// Unaffected by a one-wide position axis, because that axis is a one-to-one
/// correspondence with the table's own and the widening is over the head axis.
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
///
/// Unaffected by a one-wide position axis for the same reason: both of the
/// mask's own axes are one-to-one and the widening is over the group and
/// repetition axes.
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

/// The leading extent-one position axis, as a reindex rather than a broadcast.
fn insert_position_axis() -> ReindexForm {
    ReindexForm::insert_unit_axis(axis(0)).expect("a leading insertion position")
}

/// `[T, width] -> [T, heads, 128]`: a projection read as heads of width 128.
fn projection_split(heads: usize) -> ReindexForm {
    ReindexForm::split_axis(axis(1), [extent(heads), extent(HEAD_DIM)])
        .expect("a declared width factors as heads x 128")
}

/// Which key head a query head reads.
///
/// `repeat_kv` is repeat-interleave, so [`HeadReading::Interleave`] is the one
/// that denotes the reference; [`HeadReading::Tile`] is an identically shaped
/// graph of the same occurrence count that only a value comparison separates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HeadReading {
    /// `(8, 2)`, group major: `h = 2g + r`, so the group is `h / 2`.
    Interleave,
    /// `(2, 8)`, repetition major: `h = 8r + g`, so the group is `h % 8`.
    Tile,
}

impl HeadReading {
    fn split(self) -> ReindexForm {
        let factors = match self {
            Self::Interleave => [extent(GROUPS), extent(REPEATS)],
            Self::Tile => [extent(REPEATS), extent(GROUPS)],
        };
        ReindexForm::split_axis(axis(1), factors).expect("16 = 8 x 2")
    }

    fn permute(self) -> ReindexForm {
        let order = match self {
            Self::Interleave => [axis(1), axis(2), axis(0), axis(3)],
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

/// Which projection the MLP's activation is applied to.
///
/// The checkpoint's `mlp` computes `down(act(gate(x)) * up(x))`.
/// [`Gating::UpActivated`] swaps the two weights, which are the same shape, so
/// the graph is structurally identical and only a value comparison separates
/// them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Gating {
    GateActivated,
    UpActivated,
}

/// Which operand of a sequence extension carries the cached positions.
///
/// Operand order is semantic in `tiler::concatenate-f32@1`, so
/// [`CacheOrder::CacheSuffix`] states a different computation — but only when the
/// cache is nonempty, which is why the prefill row cannot discriminate the two.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CacheOrder {
    CachePrefix,
    CacheSuffix,
}

/// One instantiation's three semantic choices, all of them the reference's.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LayerShape {
    heads: HeadReading,
    gating: Gating,
    cache_order: CacheOrder,
}

impl LayerShape {
    /// The checkpoint's own composition.
    const fn reference() -> Self {
        Self {
            heads: HeadReading::Interleave,
            gating: Gating::GateActivated,
            cache_order: CacheOrder::CachePrefix,
        }
    }
}

/// `[S, 8, 128] -> [8, S, 128]`: one map serving both the key and the value edge.
fn key_value_permute() -> ReindexForm {
    ReindexForm::permute_axes([axis(1), axis(0), axis(2)]).expect("[T, g, d] -> [g, T, d]")
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
fn half_split(lane_axis: u32) -> ReindexForm {
    ReindexForm::split_axis(axis(lane_axis), [extent(HALVES), extent(HALF)]).expect("128 = 2 x 64")
}

/// The within-axis coordinate swap, in the one admitted form.
fn within_axis_swap(lane_axis: u32) -> ReindexForm {
    ReindexForm::reverse_axis(axis(lane_axis)).expect("the size-two axis reverses")
}

/// `[…, 2, 64] -> […, 128]`: the merge that inverts the half split.
fn half_merge(lane_axis: u32) -> ReindexForm {
    ReindexForm::merge_axes([axis(lane_axis), axis(lane_axis + 1)])
        .expect("the two inner axes are an adjacent run")
}

// --- the two widenings whose spelling depends on the row ----------------------

/// Widens a `[hidden]` normalization weight onto the value's `[T, hidden]` shape.
///
/// **Two spellings, and `T` decides which is legal rather than which is nicer.**
/// At `T >= 2` the position axis duplicates the weight and the relation is
/// `tiler::broadcast-f32@2`'s `replicate`. At `T = 1` it duplicates nothing:
/// `BroadcastAxisMapping` refuses a many-to-one relation onto an extent-one
/// result axis under `broadcast.mapping.relation-does-not-widen`, whose own
/// documentation names the replacement — "written as a replication it is a
/// reindex's unit-axis insertion". `[1024] -> [1, 1024]` therefore has no
/// broadcast spelling at all, because the mapping that remains after removing
/// the position axis is the identity and states no widening either.
fn widen_hidden_weight(
    builder: &mut SemanticProgramBuilder,
    weight: Value<F32>,
    t: usize,
    hidden: usize,
) -> Value<F32> {
    if t == 1 {
        F32Reindex::apply(builder, &insert_position_axis(), weight)
            .expect("[hidden] -> [1, hidden] is a unit-axis insertion")
    } else {
        F32Broadcast::apply(builder, &hidden_weight_mapping(t, hidden), weight)
            .expect("the hidden-width weight replicates over the position axis")
    }
}

/// Widens a `[128]` per-head weight onto `[T, heads, 128]`.
///
/// Same rule, one occurrence more at `T = 1`: the head axis is a genuine
/// replication in both rows, so the broadcast survives and only the position axis
/// moves to a reindex.
fn widen_head_weight(
    builder: &mut SemanticProgramBuilder,
    weight: Value<F32>,
    t: usize,
    heads: usize,
) -> Value<F32> {
    if t == 1 {
        let over_heads =
            F32Broadcast::apply(builder, &head_weight_mapping_over_heads(heads), weight)
                .expect("the per-head weight replicates over the head axis");
        F32Reindex::apply(builder, &insert_position_axis(), over_heads)
            .expect("[heads, 128] -> [1, heads, 128] is a unit-axis insertion")
    } else {
        F32Broadcast::apply(builder, &head_weight_mapping(t, heads), weight)
            .expect("the per-head weight replicates over the position and head axes")
    }
}

/// Widens the `[2, 1]` rotary sign operand onto `[T, heads, 2, 64]`.
fn widen_rope_sign(
    builder: &mut SemanticProgramBuilder,
    sign: Value<F32>,
    t: usize,
    heads: usize,
) -> Value<F32> {
    if t == 1 {
        let over_heads = F32Broadcast::apply(builder, &sign_mapping_over_heads(heads), sign)
            .expect("the sign operand widens over the head axis and the half width");
        F32Reindex::apply(builder, &insert_position_axis(), over_heads)
            .expect("[heads, 2, 64] -> [1, heads, 2, 64] is a unit-axis insertion")
    } else {
        F32Broadcast::apply(builder, &sign_mapping(t, heads), sign)
            .expect("the sign operand widens over the position and head axes")
    }
}

// --- the layer ----------------------------------------------------------------

/// Emits the rotary composition over a `[T, heads, 128]` operand.
///
/// Ten occurrences at `T >= 2` and eleven at `T = 1`, the extra one being the
/// sign operand's unit-axis insertion.
fn rotary(
    builder: &mut SemanticProgramBuilder,
    operand: Value<F32>,
    inputs: &LayerInputs,
    t: usize,
    heads: usize,
) -> Value<F32> {
    let split = F32Reindex::apply(builder, &half_split(2), operand).expect("128 = 2 x 64");
    let swapped = F32Reindex::apply(builder, &within_axis_swap(2), split)
        .expect("the size-two axis reverses");
    let signs = widen_rope_sign(builder, inputs.rope_sign, t, heads);
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

/// Joins one cached tensor with the new positions, in the stated operand order.
///
/// This is the whole of what a decode step adds to the graph, and where a KV
/// cache touches it: two occurrences at the block boundary, so the graph stays
/// pure acyclic tensor SSA and nothing writes into a bound input.
fn extend_sequence(
    builder: &mut SemanticProgramBuilder,
    cache: Value<F32>,
    new_rows: Value<F32>,
    order: CacheOrder,
) -> Result<Value<F32>, BuildError> {
    let operands = match order {
        CacheOrder::CachePrefix => [cache, new_rows],
        CacheOrder::CacheSuffix => [new_rows, cache],
    };
    F32Concatenate::apply(builder, &operands, axis(1))
}

/// Emits one RMS normalization together with the widening its weight needs.
///
/// Two occurrences at `T >= 2` and two or three at `T = 1`:
/// `tiler::rms-norm-f32@1` takes a weight already shaped like the value, because
/// the graph admits no implicit broadcasting.
fn normalize(
    builder: &mut SemanticProgramBuilder,
    value: Value<F32>,
    widened: Value<F32>,
    reduced: Axis,
) -> Value<F32> {
    F32RmsNorm::apply(
        builder,
        value,
        widened,
        reduced,
        RMS_NORM_F32_REFERENCE_EPS_BITS,
    )
    .expect("the weight now carries the value's own shape")
}

/// Builds the complete layer at the checkpoint's own composition.
fn build_layer(extents: LayerExtents) -> SemanticProgram {
    build_layer_with(extents, LayerShape::reference())
        .expect("a row whose context is its cache plus its new positions")
}

/// Builds the complete layer: twenty-nine steps over ten registered keys.
///
/// The step numbers in the comments are the attention design's table for 1–22
/// and this file's own for the MLP.
///
/// Fallible in exactly one place, and deliberately: the mask add is where the
/// context extent the mask declares meets the one the concatenation produced, so
/// a row whose `S` is not `C + T` is refused there even when the environment
/// admitted the three bindings separately.
///
/// # Errors
///
/// Returns the mask add's own `binary.shape` refusal for such a row.
fn build_layer_with(
    extents: LayerExtents,
    shape: LayerShape,
) -> Result<SemanticProgram, BuildError> {
    let LayerExtents {
        new_positions: t,
        cached: _,
        context: s,
        hidden,
        intermediate: _,
    } = extents;
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let inputs = declare_inputs(&mut builder, extents);

    // 1. RMS normalization of the residual stream, over the model dimension.
    let input_weight = widen_hidden_weight(&mut builder, inputs.input_layernorm, t, hidden);
    let normalized = normalize(&mut builder, inputs.residual, input_weight, axis(1));

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
    let query_weight = widen_head_weight(&mut builder, inputs.query_norm, t, QUERY_HEADS);
    let query_norm = normalize(&mut builder, query_heads, query_weight, axis(2));
    let key_weight = widen_head_weight(&mut builder, inputs.key_norm, t, GROUPS);
    let key_norm = normalize(&mut builder, key_heads, key_weight, axis(2));

    // 10, 11. Rotary position embedding.
    let query_rotary = rotary(&mut builder, query_norm, &inputs, t, QUERY_HEADS);
    let key_rotary = rotary(&mut builder, key_norm, &inputs, t, GROUPS);

    // 12. The grouped-query head layout.
    let query_grouped =
        F32Reindex::apply(&mut builder, &shape.heads.split(), query_rotary).expect("16 = 8 x 2");
    let query_grouped = F32Reindex::apply(&mut builder, &shape.heads.permute(), query_grouped)
        .expect("the group axis moves outermost");

    // 13, 14. The two retained outputs' layouts, each feeding one extension.
    let key_new = F32Reindex::apply(&mut builder, &key_value_permute(), key_rotary)
        .expect("[T, g, d] -> [g, T, d]");
    let value_new = F32Reindex::apply(&mut builder, &key_value_permute(), value_split)
        .expect("[T, g, d] -> [g, T, d]");
    let key_rope = extend_sequence(&mut builder, inputs.key_cache, key_new, shape.cache_order)
        .expect("[8, C, 128] and [8, T, 128] join on axis 1");
    let value_heads = extend_sequence(
        &mut builder,
        inputs.value_cache,
        value_new,
        shape.cache_order,
    )
    .expect("[8, C, 128] and [8, T, 128] join on axis 1");

    // 15. The score contraction, structure 2, against the extended key.
    let scores =
        F32TensorContraction::apply(&mut builder, &score_structure(), query_grouped, key_rope)
            .expect("grtd,gsd->grts over the grouped query and the extended key");

    // 16. The scale, on the *score* and not on an operand.
    let scale = F32Constant::apply(&mut builder, ATTENTION_SCALE_BITS).expect("a scalar constant");
    let scaled = F32Multiply::apply(&mut builder, scores, scale)
        .expect("a rank-zero right operand is admitted");

    // 17. The additive causal mask, broadcast over the two head axes.
    let mask = F32Broadcast::apply(&mut builder, &mask_mapping(t, s), inputs.mask)
        .expect("the mask broadcasts over the group and repetition axes");
    // The one binding-dependent step: the scores are `[8, 2, T, C + T]` because
    // the key they contract against is the extension's result, so a mask whose
    // declared context is not that sum is refused here rather than truncated.
    let masked = F32Add::apply(&mut builder, scaled, mask)?;

    // 18. Softmax over the key axis.
    let probabilities =
        F32Softmax::apply(&mut builder, masked, axis(3)).expect("axis 3 is the key axis");

    // 19. The value contraction, structure 3, over the grown extent.
    let context_vectors =
        F32TensorContraction::apply(&mut builder, &value_structure(), probabilities, value_heads)
            .expect("grts,gsd->grtd over the probabilities and the extended value");

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

    // 22. The first residual add.
    let residual_mid = F32Add::apply(&mut builder, inputs.residual, attention_out)
        .expect("both operands are [T, hidden]");

    // 23. The post-attention normalization, over the model dimension.
    let post_weight = widen_hidden_weight(&mut builder, inputs.post_attention_layernorm, t, hidden);
    let mlp_input = normalize(&mut builder, residual_mid, post_weight, axis(1));

    // 24, 25. The gate and up projections, structure 1, to the intermediate width.
    let gate = F32TensorContraction::apply(&mut builder, &structure, mlp_input, inputs.gate_weight)
        .expect("td,od->to over [T, hidden] and [3072, hidden]");
    let up = F32TensorContraction::apply(&mut builder, &structure, mlp_input, inputs.up_weight)
        .expect("td,od->to over [T, hidden] and [3072, hidden]");

    // 26, 27. The activation and the elementwise product. Which projection the
    // activation reads is semantics: the two weights are the same shape, so the
    // swapped graph is structurally identical and only a value comparison
    // separates them.
    let (activated_operand, passthrough) = match shape.gating {
        Gating::GateActivated => (gate, up),
        Gating::UpActivated => (up, gate),
    };
    let activated =
        F32Silu::apply(&mut builder, activated_operand).expect("the operand is tiler::f32@1");
    let gated = F32Multiply::apply(&mut builder, activated, passthrough)
        .expect("both operands are [T, intermediate]");

    // 28. The down projection, structure 1, back to the model dimension.
    let projected =
        F32TensorContraction::apply(&mut builder, &structure, gated, inputs.down_weight)
            .expect("td,od->to over [T, intermediate] and [hidden, intermediate]");

    // 29. The second residual add.
    let residual_out = F32Add::apply(&mut builder, residual_mid, projected)
        .expect("both operands are [T, hidden]");

    // The three ordered named outputs. `h_out` first because it is the layer's
    // observable result; `k_rope` and `v_heads` follow as the KV seam, and they
    // are the extensions' results rather than the new rows alone.
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
    Ok(builder.build().expect("the layer is complete"))
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
        panic!("a layer result is a dense f32 tensor");
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
/// The `q`th new position is absolute position `C + q`, so key position `k` of
/// the `S = C + T` context is visible exactly when `k <= C + q`. At prefill
/// `C = 0` and this is the ordinary lower triangle; at `T = 1` every entry is
/// attended, which is the degeneracy the autoregressive-state record names.
fn causal_mask(cached: usize, t: usize, s: usize) -> Vec<u32> {
    let mut mask = Vec::with_capacity(t * s);
    for query in 0..t {
        for key in 0..s {
            mask.push(if key <= cached + query {
                ATTENDED_FILL_BITS
            } else {
                MASKED_FILL_BITS
            });
        }
    }
    mask
}

/// Every operand one layer evaluation binds, at one set of extents.
struct LayerFixture {
    extents: LayerExtents,
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
    key_cache: Tensor,
    value_cache: Tensor,
    post_attention_layernorm: Tensor,
    gate_weight: Tensor,
    up_weight: Tensor,
    down_weight: Tensor,
}

impl LayerFixture {
    fn new(extents: LayerExtents) -> Self {
        let LayerExtents {
            new_positions: t,
            cached: c,
            context: s,
            hidden,
            intermediate,
        } = extents;
        let mut salt = FIXTURE_SEED;
        let mut next = |count: usize| {
            salt = salt.wrapping_add(0x0f0f_0f0f_0f0f_0f0f);
            samples(count, salt)
        };
        Self {
            extents,
            residual: tensor_of(&layer_shape([t, hidden]), &next(t * hidden)),
            input_layernorm: tensor_of(&layer_shape([hidden]), &next(hidden)),
            query_weight: tensor_of(
                &layer_shape([QUERY_WIDTH, hidden]),
                &next(QUERY_WIDTH * hidden),
            ),
            key_weight: tensor_of(
                &layer_shape([KEY_VALUE_WIDTH, hidden]),
                &next(KEY_VALUE_WIDTH * hidden),
            ),
            value_weight: tensor_of(
                &layer_shape([KEY_VALUE_WIDTH, hidden]),
                &next(KEY_VALUE_WIDTH * hidden),
            ),
            query_norm: tensor_of(&layer_shape([HEAD_DIM]), &next(HEAD_DIM)),
            key_norm: tensor_of(&layer_shape([HEAD_DIM]), &next(HEAD_DIM)),
            cosine: tensor_of(&layer_shape([t, HEAD_DIM]), &next(t * HEAD_DIM)),
            sine: tensor_of(&layer_shape([t, HEAD_DIM]), &next(t * HEAD_DIM)),
            rope_sign: tensor_of_bits(&layer_shape([HALVES, 1]), |half| {
                [NEGATIVE_ONE, POSITIVE_ONE][half]
            }),
            mask: {
                let bits = causal_mask(c, t, s);
                tensor_of_bits(&layer_shape([t, s]), |position| bits[position])
            },
            output_weight: tensor_of(
                &layer_shape([hidden, QUERY_WIDTH]),
                &next(hidden * QUERY_WIDTH),
            ),
            // At prefill this is `[8, 0, 128]` and holds no element: the empty
            // cache the concatenation family admits and contributes nothing from.
            key_cache: tensor_of(
                &layer_shape([GROUPS, c, HEAD_DIM]),
                &next(GROUPS * c * HEAD_DIM),
            ),
            value_cache: tensor_of(
                &layer_shape([GROUPS, c, HEAD_DIM]),
                &next(GROUPS * c * HEAD_DIM),
            ),
            post_attention_layernorm: tensor_of(&layer_shape([hidden]), &next(hidden)),
            gate_weight: tensor_of(
                &layer_shape([intermediate, hidden]),
                &next(intermediate * hidden),
            ),
            up_weight: tensor_of(
                &layer_shape([intermediate, hidden]),
                &next(intermediate * hidden),
            ),
            down_weight: tensor_of(
                &layer_shape([hidden, intermediate]),
                &next(hidden * intermediate),
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
            (key_cache_key(), &self.key_cache),
            (value_cache_key(), &self.value_cache),
            (
                post_attention_layernorm_key(),
                &self.post_attention_layernorm,
            ),
            (gate_weight_key(), &self.gate_weight),
            (up_weight_key(), &self.up_weight),
            (down_weight_key(), &self.down_weight),
        ]
    }
}

/// Evaluates one layer program against one fixture and returns its three outputs.
///
/// The allowance is the caller's rather than this helper's, because it is the
/// only thing separating an evaluation of this layer at the C1 prefill row from a
/// refusal of it — so a reader of a call site sees which of the two is asked for.
fn evaluate_layer(
    program: &SemanticProgram,
    fixture: &LayerFixture,
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
        .expect("the layer evaluates");
    let [residual, key_rope, value_heads] = outputs.as_slice() else {
        panic!("the layer has three outputs");
    };
    let LayerExtents {
        new_positions: t,
        context: s,
        hidden,
        ..
    } = fixture.extents;
    assert_eq!(residual.shape(), &layer_shape([t, hidden]));
    assert_eq!(key_rope.shape(), &layer_shape([GROUPS, s, HEAD_DIM]));
    assert_eq!(value_heads.shape(), &layer_shape([GROUPS, s, HEAD_DIM]));
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
/// what a fully masked-out contributor set produces.
fn strict_fold(contributors: impl IntoIterator<Item = f32>) -> f32 {
    let mut accumulator: Option<f32> = None;
    for contributor in contributors {
        accumulator = Some(accumulator.map_or(contributor, |value| value + contributor));
    }
    accumulator.expect("a nonempty contributor sequence")
}

/// The layer recomputed from the operation table by explicit coordinate arithmetic.
struct LayerExpectation {
    residual_out: Vec<u32>,
    key_rope: Vec<u32>,
    value_heads: Vec<u32>,
}

fn recompute_layer(fixture: &LayerFixture) -> LayerExpectation {
    let LayerExtents {
        new_positions: t,
        cached: c,
        context: s,
        hidden,
        intermediate,
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
    let k_cache = payload_floats(&fixture.key_cache);
    let v_cache = payload_floats(&fixture.value_cache);
    let w_post = payload_floats(&fixture.post_attention_layernorm);
    let w_gate = payload_floats(&fixture.gate_weight);
    let w_up = payload_floats(&fixture.up_weight);
    let w_down = payload_floats(&fixture.down_weight);

    // 1, 23. One normalization over the model dimension, with the weight
    // replicated over the position axis exactly as the widening states — which
    // the two spellings agree on, since a unit-axis insertion at `T = 1` and a
    // replication at `T >= 2` both put `w[j]` at every `(i, j)`.
    let normalize_hidden = |values: &[f32], weight: &[f32]| -> Vec<f32> {
        let widened: Vec<f32> = (0..t * hidden).map(|i| weight[i % hidden]).collect();
        rms_norm_f32(
            &layer_shape([t, hidden]),
            axis(1),
            RMS_NORM_F32_REFERENCE_EPS_BITS,
            values,
            &widened,
        )
        .expect("the normalization is well formed")
    };
    let normalized = normalize_hidden(&x, &w_in);

    // 2, 3, 4, 21, 24, 25, 28. `td,od->to`: output `[t, o]` folds over `d`.
    let project = |source: &[f32], depth: usize, weight: &[f32], out_width: usize| -> Vec<f32> {
        let mut result = Vec::with_capacity(t * out_width);
        for position in 0..t {
            for column in 0..out_width {
                result.push(strict_fold((0..depth).map(|index| {
                    source[position * depth + index] * weight[column * depth + index]
                })));
            }
        }
        result
    };
    let query_flat = project(&normalized, hidden, &w_q, QUERY_WIDTH);
    let key_flat = project(&normalized, hidden, &w_k, KEY_VALUE_WIDTH);
    let value_flat = project(&normalized, hidden, &w_v, KEY_VALUE_WIDTH);

    // 5, 6, 7. The head splits are row-major refactorings, so `[t, heads*128]`
    // and `[t, heads, 128]` are the same buffer read two ways.

    // 8, 9. Per-head normalization over the 128-wide axis.
    let per_head_norm = |values: &[f32], weight: &[f32], heads: usize| -> Vec<f32> {
        let widened: Vec<f32> = (0..t * heads * HEAD_DIM)
            .map(|i| weight[i % HEAD_DIM])
            .collect();
        rms_norm_f32(
            &layer_shape([t, heads, HEAD_DIM]),
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
    // `rotate_half(x) = cat(-x2, x1)` derived from the coordinate maps.
    let rotate = |values: &[f32], heads: usize| -> Vec<f32> {
        let mut result = Vec::with_capacity(t * heads * HEAD_DIM);
        for position in 0..t {
            for head in 0..heads {
                let row = (position * heads + head) * HEAD_DIM;
                for lane in 0..HEAD_DIM {
                    let (half, offset) = (lane / HALF, lane % HALF);
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

    // 13, 14. `[t, 8, 128] -> [8, t, 128]`, then the extension: coordinate `j` of
    // the `s`-axis reads the cache while `j < c` and the new rows after it, which
    // is the operand order stated on the occurrence.
    let extend = |cache: &[f32], new_rows: &[f32]| -> Vec<f32> {
        let mut result = vec![0.0_f32; GROUPS * s * HEAD_DIM];
        for group in 0..GROUPS {
            for position in 0..s {
                for lane in 0..HEAD_DIM {
                    result[(group * s + position) * HEAD_DIM + lane] = if position < c {
                        cache[(group * c + position) * HEAD_DIM + lane]
                    } else {
                        // The new rows arrive as `[t, 8, 128]` and the permute
                        // reads them group-major, so this index is the permute
                        // and the concatenation composed.
                        new_rows[((position - c) * GROUPS + group) * HEAD_DIM + lane]
                    };
                }
            }
        }
        result
    };
    let key_rope = extend(&k_cache, &key_rotary);
    let value_heads = extend(&v_cache, &value_flat);

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
    let probabilities = softmax_f32(&layer_shape([GROUPS, REPEATS, t, s]), axis(3), &masked)
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

    // 21, 22. The output projection and the first residual add.
    let attention_out = project(&context_flat, QUERY_WIDTH, &w_o, hidden);
    let residual_mid: Vec<f32> = (0..t * hidden)
        .map(|position| x[position] + attention_out[position])
        .collect();

    // 23, 24, 25. The post-attention normalization and the two wide projections.
    let mlp_input = normalize_hidden(&residual_mid, &w_post);
    let gate = project(&mlp_input, hidden, &w_gate, intermediate);
    let up = project(&mlp_input, hidden, &w_up, intermediate);

    // 26, 27. `silu(gate) * up`, elementwise over the intermediate width.
    let gated: Vec<f32> = (0..t * intermediate)
        .map(|position| {
            silu_f32(gate[position]).expect("the activation is decided at a fixture operand")
                * up[position]
        })
        .collect();

    // 28, 29. The down projection and the second residual add.
    let projected = project(&gated, intermediate, &w_down, hidden);
    let residual_out: Vec<u32> = (0..t * hidden)
        .map(|position| (residual_mid[position] + projected[position]).to_bits())
        .collect();

    LayerExpectation {
        residual_out,
        key_rope: key_rope.into_iter().map(f32::to_bits).collect(),
        value_heads: value_heads.into_iter().map(f32::to_bits).collect(),
    }
}

fn differing(left: &[u32], right: &[u32]) -> usize {
    assert_eq!(left.len(), right.len(), "a comparison is element-wise");
    left.iter().zip(right).filter(|(a, b)| a != b).count()
}

// --- the layer's shape ----------------------------------------------------------

/// The C1 conformance row's prefill extents: ten new positions and no cache.
fn c1_prefill_extents() -> LayerExtents {
    let environment = shape_environment(C1_POSITIONS, 0, C1_POSITIONS)
        .expect("a prefill row's context is its new positions");
    LayerExtents::resolve(&environment, C1_HIDDEN, INTERMEDIATE)
        .expect("all three symbols are pinned and bounded")
}

/// The C1 row's eighth decode step: one new position against seventeen cached.
fn c1_decode_extents() -> LayerExtents {
    let environment = shape_environment(C1_DECODE_POSITIONS, C1_DECODE_CACHED, C1_DECODE_CONTEXT)
        .expect("18 = 17 + 1");
    LayerExtents::resolve(&environment, C1_HIDDEN, INTERMEDIATE)
        .expect("all three symbols are pinned and bounded")
}

/// Counts one program's occurrences by key, so a step that silently became a
/// different family fails rather than passing on arithmetic.
fn occurrences_by_key(program: &SemanticProgram) -> Vec<(OpKey, usize)> {
    let mut counts: Vec<(OpKey, usize)> = Vec::new();
    for operation in program.operations() {
        match counts.iter_mut().find(|(key, _)| key == operation.key()) {
            Some((_, count)) => *count += 1,
            None => counts.push((operation.key().clone(), 1)),
        }
    }
    counts.sort_by(|left, right| left.0.cmp(&right.0));
    counts
}

fn sorted(mut counts: Vec<(OpKey, usize)>) -> Vec<(OpKey, usize)> {
    counts.sort_by(|left, right| left.0.cmp(&right.0));
    counts
}

/// The layer's ordered interface and its occurrence census, measured by key.
///
/// A program's ordered inputs and ordered named outputs are part of its contract,
/// and its occurrence and value counts are measurements of the graph rather than
/// figures derived from a description of it — which is why they replace the
/// record's floors of "at least fifty-one occurrences over at least twenty-one
/// boundary values" rather than confirming them. Counting by key rather than in
/// total is what makes a step that silently became a different family fail here
/// instead of passing on arithmetic. The output shapes are the families' own
/// derivations, and the worked instance shows what an empty cache contributes:
/// at `C = 0` the two retained outputs are the new rows alone, which is the
/// concatenation's zero-extent rule.
#[test]
fn the_layer_verifies_at_the_c1_prefill_row() {
    let extents = c1_prefill_extents();
    assert_eq!(
        extents,
        LayerExtents {
            new_positions: 10,
            cached: 0,
            context: 10,
            hidden: 1_024,
            intermediate: 3_072,
        }
    );
    let program = build_layer(extents);

    // Eighteen ordered inputs and three ordered named outputs.
    assert_eq!(program.input_count(), 18);
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

    // The measured counts, which replace the record's derived floors of "at
    // least fifty-one occurrences over at least twenty-one boundary values".
    // The value count is the eighteen inputs plus one result per occurrence,
    // which is what makes it exactly `input_count + operation_count` here: no
    // occurrence in this layer produces more than one value.
    assert_eq!(program.operation_count(), 58);
    assert_eq!(program.value_count(), 76);
    assert_eq!(
        program.value_count(),
        program.input_count() + program.operation_count()
    );

    let expected = sorted(vec![
        // 2 rotary adds + 1 mask add + 2 residual adds
        (add_f32_op(), 5),
        // 2 hidden weights + 2 head weights + 2 rotary signs + 4 rotary tables
        // + 1 mask
        (broadcast_f32_op(), 11),
        (concatenate_f32_op(), 2),
        (constant_f32_op(), 1),
        // 2 rotary signs + 4 rotary tables + 1 scale + 1 MLP product
        (multiply_f32_op(), 8),
        // 3 head splits + 2 x 3 rotary + 2 grouping + 2 kv permutes + 3 merges
        (reindex_f32_op(), 16),
        (rms_norm_f32_op(), 4),
        (silu_f32_op(), 1),
        (softmax_f32_op(), 1),
        // 4 attention projections + score + value + 3 MLP
        (tensor_contraction_f32_op(), 9),
    ]);
    assert_eq!(occurrences_by_key(&program), expected);

    // The derived output shapes, which no caller declared. At `C = 0` the two
    // retained outputs are the new rows alone, which is what the concatenation's
    // zero-extent rule says an empty operand contributes.
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
            layer_shape([10, C1_HIDDEN]),
            layer_shape([GROUPS, 10, HEAD_DIM]),
            layer_shape([GROUPS, 10, HEAD_DIM]),
        ]
    );
}

/// The same layer at a single new position is a different graph, and it verifies.
///
/// A row that degenerates an axis to extent one is not automatically a rebinding:
/// the widenings that duplicated something at `T >= 2` duplicate nothing at
/// `T = 1` and must be respelled, so the occurrence count moves while the
/// interface does not. Here that is four occurrences more than the prefill row,
/// all unit-axis insertions, and the count is a consequence of a refusal rather
/// than a style choice — [`a_single_new_position_changes_six_widenings`] accounts
/// for them and [`a_rank_pad_onto_a_single_position_refuses`] watches the rule.
/// The retained outputs carry the whole context rather than the new position,
/// because what a KV seam publishes is the extension's result and not its
/// operand.
#[test]
fn the_layer_verifies_at_the_c1_decode_row() {
    let extents = c1_decode_extents();
    assert_eq!(
        extents,
        LayerExtents {
            new_positions: 1,
            cached: 17,
            context: 18,
            hidden: 1_024,
            intermediate: 3_072,
        }
    );
    let program = build_layer(extents);
    assert_eq!(program.input_count(), 18);
    assert_eq!(program.output_count(), 3);

    // Four occurrences more than the prefill row, all of them unit-axis
    // insertions; see `a_single_new_position_changes_six_widenings`.
    assert_eq!(program.operation_count(), 62);
    assert_eq!(program.value_count(), 80);

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
            layer_shape([1, C1_HIDDEN]),
            layer_shape([GROUPS, 18, HEAD_DIM]),
            layer_shape([GROUPS, 18, HEAD_DIM]),
        ],
        "the retained outputs carry the whole context, not the new position"
    );
}

/// One keyed family carries every contraction, with its structure as an attribute.
///
/// Structurally different contractions are structure *values* under
/// `tiler::tensor-contraction-f32@1` rather than separate keys, so reading
/// the attribute back off each occurrence is how a program's contractions are
/// told apart, and an unrecognized structure is a panic rather than an uncounted
/// occurrence. The worked instance is what the MLP adds: no fourth structure, only
/// more occurrences of the projection structure at different widths.
#[test]
fn all_three_contraction_index_structures_occur() {
    // Seven structure-1 projections rather than the attention block's four: the
    // MLP's gate, up and down are the same index structure at different widths.
    let program = build_layer(c1_prefill_extents());
    let (mut projections, mut scores, mut values) = (0, 0, 0);
    for operation in program
        .operations()
        .filter(|operation| operation.key() == &tensor_contraction_f32_op())
    {
        let structure = operation
            .attributes()
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
    assert_eq!((projections, scores, values), (7, 1, 1));
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
        panic!("a layer refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

/// `S == C + T` is a retained relation and it refuses a row that does not satisfy it.
///
/// Both the false direction and the admitted neighbour are stated, because the
/// three intervals alone would admit any triple: what refuses the row is the
/// additive relation, and the environment says so by name.
#[test]
fn the_extent_relation_refuses_an_inconsistent_row() {
    let refused = shape_environment(C1_POSITIONS, 0, 18)
        .expect_err("a context of 18 is not an empty cache plus ten new positions");
    match &refused {
        ShapeEnvError::UnsupportedRelation { violation, .. } => assert_eq!(
            *violation,
            FragmentViolation::UnderdeterminedAdditiveEquality { undetermined: 3 },
            "all three terms are runtime-bound, so the relation is decided against \
             their proved lower bounds rather than against constants"
        ),
        other => panic!("the refusal names the relation, not {other}"),
    }
    assert!(
        refused
            .to_string()
            .contains("shape-env.unsupported-relation"),
        "the refusal carries its stable code: {refused}"
    );

    // The admitted neighbours, one on each side of the row that failed: the
    // prefill row where the cache is empty, and the decode row where it is not.
    assert!(shape_environment(C1_POSITIONS, 0, C1_POSITIONS).is_ok());
    assert!(shape_environment(1, 17, 18).is_ok());
    // And a cache eight deep against ten new positions, which is the same
    // eighteen the refusal above declared and is admitted because it adds up.
    assert!(shape_environment(C1_POSITIONS, 8, 18).is_ok());
}

/// The graph refuses the same inconsistent row independently of the environment.
///
/// The mask add is where the context the mask declares meets the one the
/// extension produced, so a caller that assembled `LayerExtents` by hand — past
/// the environment that would have refused it — is still refused, by name.
#[test]
fn a_context_that_is_not_the_cache_plus_the_new_positions_refuses_at_the_mask_add() {
    let inconsistent = LayerExtents {
        new_positions: 10,
        cached: 0,
        context: 18,
        hidden: C1_HIDDEN,
        intermediate: INTERMEDIATE,
    };
    let refused = build_layer_with(inconsistent, LayerShape::reference())
        .expect_err("the extension produced ten context positions, not eighteen");
    assert_eq!(refusal_code(&refused), "binary.shape");

    // The admitted neighbour: the same ten new positions against an eight-deep
    // cache, where the extension does produce eighteen.
    let consistent = LayerExtents {
        cached: 8,
        ..inconsistent
    };
    assert!(build_layer_with(consistent, LayerShape::reference()).is_ok());
}

/// A rank pad onto a one-wide position axis refuses, and names its replacement.
///
/// This is the rule that makes the decode row a different graph. It is watched
/// here at each of the three widenings the layer performs, beside the admitted
/// neighbour at ten positions, so that "the decode row inserts unit axes" is a
/// consequence of a refusal rather than a style choice.
#[test]
fn a_rank_pad_onto_a_single_position_refuses() {
    for (subject, refused, admitted) in [
        (
            "the hidden-width normalization weight",
            BroadcastAxisMapping::new(
                [extent(1), extent(C1_HIDDEN)],
                [
                    BroadcastAxisSource::Replicate,
                    BroadcastAxisSource::FromOperand(axis(0)),
                ],
            ),
            hidden_weight_mapping(10, C1_HIDDEN),
        ),
        (
            "the per-head normalization weight",
            BroadcastAxisMapping::new(
                [extent(1), extent(QUERY_HEADS), extent(HEAD_DIM)],
                [
                    BroadcastAxisSource::Replicate,
                    BroadcastAxisSource::Replicate,
                    BroadcastAxisSource::FromOperand(axis(0)),
                ],
            ),
            head_weight_mapping(10, QUERY_HEADS),
        ),
        (
            "the rotary sign operand",
            BroadcastAxisMapping::new(
                [extent(1), extent(GROUPS), extent(HALVES), extent(HALF)],
                [
                    BroadcastAxisSource::Replicate,
                    BroadcastAxisSource::Replicate,
                    BroadcastAxisSource::FromOperand(axis(0)),
                    BroadcastAxisSource::StretchUnit(axis(1)),
                ],
            ),
            sign_mapping(10, GROUPS),
        ),
    ] {
        assert_eq!(
            refused
                .expect_err("a replication onto an extent-one result axis duplicates nothing")
                .diagnostic_code(),
            "broadcast.mapping.relation-does-not-widen",
            "{subject} has no broadcast spelling at one position"
        );
        let _ = admitted;
    }
}

/// A cached tensor whose head extent disagrees is refused at the extension.
#[test]
fn a_cache_whose_head_extent_disagrees_refuses_at_the_extension() {
    let extents = c1_decode_extents();
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    // Sixteen query heads' worth of cache against the eight the key path
    // produces: identically ranked, plausibly shaped, and unjoinable.
    let wrong = builder
        .input::<F32>(
            key_cache_key(),
            layer_shape([QUERY_HEADS, extents.cached, HEAD_DIM]),
        )
        .expect("an F32 input");
    let new_rows = builder
        .input::<F32>(
            InputKey::new("k_new").expect("a valid key"),
            layer_shape([GROUPS, extents.new_positions, HEAD_DIM]),
        )
        .expect("an F32 input");
    assert_eq!(
        refusal_code(
            &extend_sequence(&mut builder, wrong, new_rows, CacheOrder::CachePrefix).unwrap_err()
        ),
        "concatenate.operands.extent-disagreement",
        "an axis other than the concatenated one must agree, and the refusal \
         names both observed extents rather than broadcasting one"
    );

    // The admitted neighbour: the same join with the cache's head extent right.
    let right = builder
        .input::<F32>(
            InputKey::new("k_cache_right").expect("a valid key"),
            layer_shape([GROUPS, extents.cached, HEAD_DIM]),
        )
        .expect("an F32 input");
    assert!(extend_sequence(&mut builder, right, new_rows, CacheOrder::CachePrefix).is_ok());
}

// --- what moves with the row, and what does not ---------------------------------

/// Each occurrence's key and its *extent-free* attributes.
///
/// A [`BroadcastAxisMapping`] carries its declared result extents in the same
/// canonical record as its per-axis relations, so its attribute bytes are a
/// function of the row. This signature drops that one field and keeps the
/// relations, which is the part that states what the broadcast *means*; every
/// other family's attributes are already extent-free.
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

/// Filling the cache is a binding change and nothing else.
///
/// This is the half of the records' "one program serves both phases" claim that
/// holds. At a fixed `T` the layer built against an empty cache and against an
/// eight-deep one has the identical occurrence sequence: same families in the
/// same order, same reindex forms, same contraction structures, same broadcast
/// relations. Only extents move — and the two retained outputs' extents do move,
/// so the comparison is not vacuous.
#[test]
fn a_nonempty_cache_changes_no_occurrence() {
    let prefill = build_layer(c1_prefill_extents());
    let continued = build_layer(
        LayerExtents::resolve(
            &shape_environment(C1_POSITIONS, 8, 18).expect("18 = 8 + 10"),
            C1_HIDDEN,
            INTERMEDIATE,
        )
        .expect("all three symbols are pinned"),
    );

    assert_eq!(prefill.operation_count(), continued.operation_count());
    assert_eq!(
        occurrence_signature(&prefill),
        occurrence_signature(&continued),
        "a deeper cache is a binding change rather than a graph change"
    );

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
    assert_eq!(context_shape(&prefill), layer_shape([GROUPS, 10, HEAD_DIM]));
    assert_eq!(
        context_shape(&continued),
        layer_shape([GROUPS, 18, HEAD_DIM])
    );
}

/// A single new position changes six widenings, and the layer's occurrence count.
///
/// This is the half of the claim that does not hold, stated as a count rather
/// than as prose. Six result axes that replicate at `T >= 2` duplicate nothing at
/// `T = 1`: two hidden-width normalization weights lose their broadcast entirely
/// and become unit-axis insertions, and two per-head weights and two rotary signs
/// keep a broadcast over the head axis and gain an insertion. Every other family
/// is unmoved.
///
/// The consequence for artifact-identity reuse is not softened here: a prefill
/// artifact and a decode artifact of this layer are two identities for
/// structural reasons and not only because a mapping carries its extents.
#[test]
fn a_single_new_position_changes_six_widenings() {
    let prefill = occurrences_by_key(&build_layer(c1_prefill_extents()));
    let decode = occurrences_by_key(&build_layer(c1_decode_extents()));

    let count = |counts: &[(OpKey, usize)], key: &OpKey| -> usize {
        counts
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map_or(0, |(_, count)| *count)
    };
    assert_eq!(count(&prefill, &broadcast_f32_op()), 11);
    assert_eq!(count(&decode, &broadcast_f32_op()), 9);
    assert_eq!(count(&prefill, &reindex_f32_op()), 16);
    assert_eq!(count(&decode, &reindex_f32_op()), 22);

    // And nothing else moved: every other key holds the same count at both rows.
    let moved: Vec<OpKey> = prefill
        .iter()
        .map(|(key, _)| key.clone())
        .filter(|key| count(&prefill, key) != count(&decode, key))
        .collect();
    assert_eq!(moved, vec![broadcast_f32_op(), reindex_f32_op()]);
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
/// `mask_row_t2`: the C1 prefill mask's third row.
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

/// This layer's generated mask is the pinned reference's, row for row.
///
/// The mask is a host computation and therefore joins the comparison surface
/// rather than leaving the system, so the generator is compared against the
/// record's own row rather than assumed to agree with it.
#[test]
fn the_generated_mask_row_is_the_pinned_one() {
    let prefill = causal_mask(0, 10, 10);
    assert_eq!(prefill[2 * 10..3 * 10], PINNED_MASK_ROW);

    // At one new position against seventeen cached, every entry is the attended
    // fill: the degeneracy that makes a fully masked row unreachable at decode.
    let decode = causal_mask(17, 1, 18);
    assert_eq!(decode, vec![ATTENDED_FILL_BITS; 18]);
    assert!(!decode.contains(&MASKED_FILL_BITS));
}

/// Runs this layer's operations 16, 17 and 18 over one score row at the C1 row's
/// own width.
///
/// The score tensor is `[8, 2, 1, 10]` and the mask `[1, 10]`, so this is **the
/// layer's own [`mask_mapping`]** at one query position rather than a reduced
/// stand-in: the two head axes are the workload's eight groups and two
/// repetitions, and the mask is replicated across all sixteen exactly as it is in
/// the layer.
fn scale_mask_and_softmax(scores: &[u32; 10], mask: &[u32; 10]) -> [Vec<u32>; 3] {
    const SLABS: usize = GROUPS * REPEATS;
    let score_shape = layer_shape([GROUPS, REPEATS, 1, 10]);
    let mask_shape = layer_shape([1, 10]);

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
        .expect("the layer's own mask mapping at one query position");
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

/// The layer's operations 16, 17 and 18 reproduce `transformers` 4.51.0's bits.
///
/// This is the one comparison in this file against the pinned reference's *own*
/// numbers rather than against a recomputation, and it covers the operations the
/// record covers. It needs no `torch` seed because the record retains the score
/// row before the scale.
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

    // The perturbation, which is the record's own `prescaled_query_differing_
    // elements_from_scaled_score` finding in miniature: scaling twice and once
    // are different binary32 computations, so operation 16's graph position is
    // semantics rather than a place to put a constant.
    let [double_scaled, _, _] = scale_mask_and_softmax(&PINNED_SCORES_SCALED, &PINNED_MASK_ROW);
    assert_ne!(double_scaled, PINNED_SCORES_SCALED);
}

// --- the reference work bound ----------------------------------------------------

/// A default evaluator refuses the C1 prefill row, and the MLP is why.
///
/// A fold of more than `MAX_REFERENCE_TENSOR_ELEMENTS` multiply-accumulate steps
/// is more than one uninterrupted walk of a contraction's iteration space may
/// cost. The attention half's largest fold at this row is 20,971,520 and the
/// MLP's three are 31,457,280 each, so this layer's allowance is strictly above
/// the attention block's and the number is the layer's own arithmetic.
///
/// Three asks against one program isolate what the allowance does: the default
/// refuses, one step short of the layer's largest fold refuses under the *stated*
/// number, and the fold's own step count evaluates. The extents never move, so
/// nothing here can be explained by the layer instead of by the fold's size.
#[test]
fn the_reference_work_bound_refuses_the_c1_prefill_row() {
    let extents = c1_prefill_extents();
    let program = build_layer(extents);
    let fixture = LayerFixture::new(extents);
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
    assert!(
        refused
            .to_string()
            .contains("iteration space has 20971520 steps, exceeding 16777216"),
        "the refusal names the exact step count and the exact bound: {refused}"
    );

    // A stated allowance one step short of the layer's largest fold, so
    // authorizing work is watched as a number that can still say no.
    let refused = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .with_iteration_step_allowance(C1_PREFILL_LARGEST_FOLD - 1)
        .evaluate(&program, &bindings)
        .expect_err("an allowance below the largest fold refuses it");
    assert!(
        refused.to_string().contains(&format!(
            "iteration space has {C1_PREFILL_LARGEST_FOLD} steps, exceeding {}",
            C1_PREFILL_LARGEST_FOLD - 1
        )),
        "a stated allowance names itself and the fold it declined: {refused}"
    );
}

/// The decode row needs no allowance at all, which is the residency point in
/// miniature: one new position makes every fold a factor of `T` smaller.
#[test]
fn the_c1_decode_row_evaluates_under_the_default_work_bound() {
    let extents = c1_decode_extents();
    let program = build_layer(extents);
    let fixture = LayerFixture::new(extents);
    let owned = fixture.bindings();
    let bindings: Vec<InputBinding<'_>> = owned
        .iter()
        .map(|(key, tensor)| InputBinding::new(key, tensor))
        .collect();
    let evaluator = ReferenceEvaluator::standard().expect("the standard evaluator opens");
    // The layer's largest decode fold: `1 * 3072 * 1024`, against the default.
    assert!(INTERMEDIATE * C1_HIDDEN < evaluator.iteration_step_allowance());
    assert_eq!(
        evaluator
            .evaluate(&program, &bindings)
            .expect("no occurrence exceeds one window at one new position")
            .len(),
        3
    );
}

// --- the end-to-end comparison ----------------------------------------------------

/// Every one of the layer's fifty-eight occurrences, evaluated end to end.
///
/// **At the C1 prefill row's own extents, with nothing reduced**: ten new
/// positions, an empty cache, a 1,024-wide model dimension, a 3,072-wide
/// intermediate, sixteen query heads over eight groups, head dimension 128, the
/// causal mask, the scale, the rotary composition, all three contraction index
/// structures, and both residuals.
///
/// **Where the perturbations that keep this comparison honest live, and why not
/// here.** One evaluation of this row walks 157,286,400 multiply-accumulate steps
/// through the reference's certified element arithmetic and 30,720 certified
/// exponentials, and it is this file's whole cost. The three graph-local
/// perturbations — the grouped-query head reading, the MLP's gating, and the
/// extension's operand order — are each about one occurrence's attributes, so
/// they are watched at the decode row, which exercises the identical head
/// geometry, intermediate width, families and forms at one new position instead
/// of ten: [`the_grouped_query_head_reading_is_semantic`],
/// [`the_mlp_gating_is_semantic`], and
/// [`the_extension_operand_order_is_semantic_only_against_a_nonempty_cache`].
/// The row-sized claim this test makes is that the *composition* reproduces the
/// recomputation, and the recomputation is an independent traversal rather than a
/// second run of the graph, so it can fail on its own.
#[test]
fn the_layer_evaluates_end_to_end_at_the_c1_prefill_row() {
    let extents = c1_prefill_extents();
    let program = build_layer(extents);
    let fixture = LayerFixture::new(extents);

    let [residual_out, key_rope, value_heads] =
        evaluate_layer(&program, &fixture, C1_PREFILL_LARGEST_FOLD);
    let expected = recompute_layer(&fixture);

    assert_eq!(
        differing(&key_rope, &expected.key_rope),
        0,
        "the key path through the extension is bit for bit"
    );
    assert_eq!(
        differing(&value_heads, &expected.value_heads),
        0,
        "the value path through the extension is bit for bit"
    );
    assert_eq!(
        differing(&residual_out, &expected.residual_out),
        0,
        "the whole layer, through both attention contractions, the MLP, and both \
         residuals, is bit for bit"
    );
    assert_eq!(residual_out.len(), 10 * C1_HIDDEN);
    assert_eq!(key_rope.len(), GROUPS * 10 * HEAD_DIM);
}

/// Evaluates the decode row under one stated graph choice, against the layer's
/// own reference composition at the same fixture.
///
/// Comparing a perturbation against the *reference program's* outputs rather than
/// against the recomputation is what these three tests need: the recomputation's
/// job is to establish that the reference composition is right, which the two
/// end-to-end tests do, and a perturbation's job is to establish that the
/// comparison can move.
fn decode_outputs_under(shape: LayerShape) -> ([Vec<u32>; 3], usize) {
    let extents = c1_decode_extents();
    let program = build_layer_with(extents, shape).expect("a decode row");
    let fixture = LayerFixture::new(extents);
    let allowance = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .iteration_step_allowance();
    let outputs = evaluate_layer(&program, &fixture, allowance);
    (outputs, program.operation_count())
}

/// Which key head a query head reads is semantics, not spelling.
///
/// `h % 8` instead of `h / 2` is a `(2, 8)` split and a transpose — a graph the
/// same family admits, of the same occurrence count and the same result shapes —
/// so nothing structural separates the two readings and only a value comparison
/// discriminates them.
#[test]
fn the_grouped_query_head_reading_is_semantic() {
    let (reference, reference_occurrences) = decode_outputs_under(LayerShape::reference());
    let (tiled, tiled_occurrences) = decode_outputs_under(LayerShape {
        heads: HeadReading::Tile,
        ..LayerShape::reference()
    });
    assert_eq!(tiled_occurrences, reference_occurrences);

    assert_ne!(
        differing(&tiled[0], &reference[0]),
        0,
        "repeat-tile pairs fourteen of sixteen query heads with a different key \
         head, and the residual carries that difference"
    );
    // The two outputs upstream of the head pairing are unmoved by it, which
    // localizes the perturbation to step 12 rather than to somewhere in
    // sixty-two occurrences.
    assert_eq!(differing(&tiled[1], &reference[1]), 0);
    assert_eq!(differing(&tiled[2], &reference[2]), 0);

    let differing_heads = (0..QUERY_HEADS)
        .filter(|head| HeadReading::Interleave.group_of(*head) != HeadReading::Tile.group_of(*head))
        .count();
    assert_eq!(differing_heads, 14);
}

/// Which projection the activation reads is semantics, not spelling.
///
/// `W_gate` and `W_up` are both `[3072, 1024]`, so `silu(up) * gate` is the
/// identical graph with two operands exchanged: the same occurrence count, the
/// same families, the same structures, the same result shapes, and every check in
/// the stack but a value comparison passes it.
#[test]
fn the_mlp_gating_is_semantic() {
    let (reference, reference_occurrences) = decode_outputs_under(LayerShape::reference());
    let (swapped, swapped_occurrences) = decode_outputs_under(LayerShape {
        gating: Gating::UpActivated,
        ..LayerShape::reference()
    });
    assert_eq!(swapped_occurrences, reference_occurrences);

    assert_ne!(
        differing(&swapped[0], &reference[0]),
        0,
        "the activation is not symmetric in its two operands"
    );
    assert_eq!(
        differing(&swapped[1], &reference[1]),
        0,
        "the MLP is downstream of the KV seam, so the retained outputs are unmoved"
    );
    assert_eq!(differing(&swapped[2], &reference[2]), 0);
}

/// The extension's operand order is semantics — and an empty cache hides it.
///
/// Both halves are watched, because only the pair says what the prefill row can
/// and cannot establish. The occurrence is isolated to its two operands rather
/// than washed through the whole layer, so what the comparison discriminates is
/// the concatenation's ordering rule and not fifty-eight occurrences downstream
/// of it.
#[test]
fn the_extension_operand_order_is_semantic_only_against_a_nonempty_cache() {
    let join = |cached: usize, order: CacheOrder| -> Vec<u32> {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the standard builder opens");
        let cache_shape = layer_shape([GROUPS, cached, HEAD_DIM]);
        let new_shape = layer_shape([GROUPS, 1, HEAD_DIM]);
        let cache = builder
            .input::<F32>(key_cache_key(), cache_shape.clone())
            .expect("an F32 input");
        let new_rows = builder
            .input::<F32>(
                InputKey::new("k_new").expect("a valid key"),
                new_shape.clone(),
            )
            .expect("an F32 input");
        let joined = extend_sequence(&mut builder, cache, new_rows, order)
            .expect("[8, C, 128] and [8, 1, 128] join on axis 1");
        builder
            .output(OutputKey::new("k_rope").expect("a valid key"), joined)
            .expect("a first output key");
        let program = builder.build().expect("the program is complete");

        let cache_tensor = tensor_of(
            &cache_shape,
            &samples(GROUPS * cached * HEAD_DIM, FIXTURE_SEED),
        );
        let new_tensor = tensor_of(&new_shape, &samples(GROUPS * HEAD_DIM, !FIXTURE_SEED));
        let outputs = ReferenceEvaluator::standard()
            .expect("the standard evaluator opens")
            .evaluate(
                &program,
                &[
                    InputBinding::new(&key_cache_key(), &cache_tensor),
                    InputBinding::new(&InputKey::new("k_new").expect("a valid key"), &new_tensor),
                ],
            )
            .expect("the join evaluates");
        payload_bits(&outputs[0])
    };

    // Against a nonempty cache the two orders are different computations, and
    // every one of the eighteen context positions is displaced.
    let prefix = join(17, CacheOrder::CachePrefix);
    let suffix = join(17, CacheOrder::CacheSuffix);
    assert_eq!(prefix.len(), GROUPS * 18 * HEAD_DIM);
    assert_ne!(differing(&prefix, &suffix), 0);

    // Against the empty cache the prefill row binds, they are the same
    // computation — which is the concatenation's zero-extent rule and is exactly
    // why the prefill row cannot discriminate the two.
    assert_eq!(
        differing(
            &join(0, CacheOrder::CachePrefix),
            &join(0, CacheOrder::CacheSuffix)
        ),
        0
    );
}

/// The same layer at the C1 row's eighth decode step: `T = 1`, `C = 17`, `S = 18`.
///
/// This is the row the attention block could not reach. That block computes its
/// own key from its own input, so a mask asserting a wider context is refused at
/// the mask add; here the extension supplies the wider context and the same
/// eighteen-position mask is admitted. Nothing was widened to reach it — the
/// difference is two program inputs and two occurrences.
#[test]
fn the_layer_evaluates_end_to_end_at_the_c1_decode_row() {
    let extents = c1_decode_extents();
    let program = build_layer(extents);
    let fixture = LayerFixture::new(extents);
    let allowance = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .iteration_step_allowance();

    let [residual_out, key_rope, value_heads] = evaluate_layer(&program, &fixture, allowance);
    let expected = recompute_layer(&fixture);

    assert_eq!(
        differing(&key_rope, &expected.key_rope),
        0,
        "the extension's first seventeen positions are the cache bit for bit and \
         its eighteenth is the new row"
    );
    assert_eq!(differing(&value_heads, &expected.value_heads), 0);
    assert_eq!(
        differing(&residual_out, &expected.residual_out),
        0,
        "the whole layer at one new position is bit for bit"
    );
    assert_eq!(residual_out.len(), C1_HIDDEN);
    assert_eq!(key_rope.len(), GROUPS * 18 * HEAD_DIM);

    // The first seventeen positions of the retained key are the bound cache
    // unchanged, which is the concatenation's bit-preservation stated as a
    // comparison against the operand rather than against the recomputation.
    let cache_bits = payload_bits(&fixture.key_cache);
    for group in 0..GROUPS {
        let cached = &cache_bits[group * 17 * HEAD_DIM..(group + 1) * 17 * HEAD_DIM];
        let retained = &key_rope[group * 18 * HEAD_DIM..group * 18 * HEAD_DIM + 17 * HEAD_DIM];
        assert_eq!(cached, retained);
    }

    // The perturbation the prefill row could not make: reversing the extension's
    // operand order puts the new position first and the cache after it, which is
    // a well-formed program of the same occurrence count and the same output
    // shapes, and every one of its eighteen context positions is displaced.
    let reversed = build_layer_with(
        extents,
        LayerShape {
            cache_order: CacheOrder::CacheSuffix,
            ..LayerShape::reference()
        },
    )
    .expect("a decode row");
    assert_eq!(reversed.operation_count(), program.operation_count());
    let [reversed_residual, reversed_key_rope, _] = evaluate_layer(&reversed, &fixture, allowance);
    assert_ne!(differing(&reversed_key_rope, &expected.key_rope), 0);
    assert_ne!(differing(&reversed_residual, &expected.residual_out), 0);
}
