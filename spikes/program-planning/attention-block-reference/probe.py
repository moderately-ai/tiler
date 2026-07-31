"""Observe what the pinned reference computes for one C1-shaped attention block.

The L4 attention design has to state a *graph*: which coordinate map each
structural step is, which index structure each contraction carries, where the
scale sits, what the mask contributes, and what the value contraction's
contributors are at a masked position. Four of those are claims a reader would
otherwise take on the design's word, and each has a plausible neighbour that is
a different computation:

  1. `rotate_half` is universally written as a slice and a concatenate. The L2
     derivation claims it is instead a `Reindex` split, a coordinate swap on the
     resulting size-2 axis, a broadcast sign multiply, and a `Reindex` merge --
     which is what removes a slice family and a concatenate family from the
     workload's requirements. That claim is checked here bit-for-bit rather than
     argued.
  2. Grouped-query repetition has two readings that differ only in which query
     head reads which key head: `h // n_rep` (repeat-interleave, what
     `repeat_kv` does) and `h % num_kv_heads` (repeat-tile). Both produce a
     correctly shaped tensor and one of them is wrong.
  3. The score contraction's `grtd,gsd->grts` index structure is claimed to
     denote the reference's repeat-then-matmul without materializing the
     repetition. Whether the two agree, and whether any disagreement is
     structural or merely a reduction-order artefact, is measured rather than
     assumed.
  4. A masked position receives exactly `+0.0` from the softmax, so it
     contributes `+0.0 * v` to the value contraction. Where `v` is negative that
     product is `-0.0`, and where an output element's accumulator is `-0.0` a
     later `+0.0` contributor turns it into `+0.0`. That is a signed-zero
     rewrite performed by the mask, inside the contraction, and it is invisible
     in any decimal rendering.

Run from this directory:

    uv run --offline python probe.py            # or drop --offline to resolve

Output is one `key<TAB>value` record per line, ordered and deterministic, so it
can be captured and byte-compared against the retained record beside it.
"""

from __future__ import annotations

import struct

import torch
import transformers
from transformers.models.qwen3 import modeling_qwen3

F32 = torch.float32
ROWS: list[tuple[str, str]] = []

# The C1 conformance row's prefill shape, and the pinned checkpoint's own head
# layout. Every extent here is read from the workload profile rather than chosen.
T = 10  # C1 prompt length; prefill covers positions 0..9 in one pass
S = 10  # prefill has no cache, so the total context equals the new positions
HEADS_Q = 16
HEADS_KV = 8
N_REP = HEADS_Q // HEADS_KV  # num_key_value_groups = 2
HEAD_DIM = 128  # declared, *not* hidden_size // num_attention_heads = 64


def bits(value: float) -> str:
    """The exact F32 bit pattern, because a decimal rendering hides -0.0."""
    return "0x%08x" % struct.unpack("<I", struct.pack("<f", float(value)))[0]


def row(key: str, value: object) -> None:
    ROWS.append((key, str(value)))


def bit_row(key: str, tensor: torch.Tensor) -> None:
    row(key, " ".join(bits(v) for v in tensor.flatten().tolist()))


def differing(left: torch.Tensor, right: torch.Tensor) -> int:
    """Count elements whose F32 bits differ, which is the only equality that matters."""
    return int((left.contiguous().view(torch.int32) != right.contiguous().view(torch.int32)).sum())


# --- environment identity ---------------------------------------------------
row("torch_version", torch.__version__)
row("transformers_version", transformers.__version__)
row("device", "cpu")
row("dtype", "float32")
row("c1_prefill_new_positions", T)
row("c1_prefill_context_positions", S)
row("head_counts_q_kv", f"{HEADS_Q} {HEADS_KV}")
row("num_key_value_groups", N_REP)
row("head_dim", HEAD_DIM)

torch.manual_seed(20260731)
q_heads = torch.randn(1, HEADS_Q, T, HEAD_DIM, dtype=F32)
k_heads = torch.randn(1, HEADS_KV, S, HEAD_DIM, dtype=F32)
v_heads = torch.randn(1, HEADS_KV, S, HEAD_DIM, dtype=F32)

# --- 1. rotate_half as a Reindex/Broadcast composition -----------------------
# `Reindex` split of the 128-wide head axis into (2, 64); a coordinate swap
# i -> 1 - i on that size-2 axis; a broadcast multiply by a [2, 1] sign operand;
# a `Reindex` merge back to 128. No slice and no concatenate.
SIGNS = torch.tensor([[-1.0], [1.0]], dtype=F32)  # a two-element program input


def rotate_half_via_reindex(x: torch.Tensor, signs: torch.Tensor = SIGNS,
                            swap: bool = True) -> torch.Tensor:
    split = x.reshape(*x.shape[:-1], 2, HEAD_DIM // 2)
    ordered = split.flip(-2) if swap else split
    return (ordered * signs).reshape(*x.shape[:-1], HEAD_DIM)


reference_rotate = modeling_qwen3.rotate_half(q_heads)
row("rotate_half_composition_differing_elements",
    differing(rotate_half_via_reindex(q_heads), reference_rotate))
row("rotate_half_element_count", reference_rotate.numel())
# Two perturbations, so a zero above is a property of the composition rather
# than of a comparison that cannot fail.
row("rotate_half_without_the_swap_differing_elements",
    differing(rotate_half_via_reindex(q_heads, swap=False), reference_rotate))
row("rotate_half_with_reversed_signs_differing_elements",
    differing(rotate_half_via_reindex(q_heads, signs=torch.tensor([[1.0], [-1.0]], dtype=F32)),
              reference_rotate))
# The first four and last four lanes of one head-position vector, so a reader
# can see the halves exchanged and one of them negated.
bit_row("rotate_half_input_lanes_0_3", q_heads[0, 0, 0, 0:4])
bit_row("rotate_half_input_lanes_64_67", q_heads[0, 0, 0, 64:68])
bit_row("rotate_half_output_lanes_0_3", reference_rotate[0, 0, 0, 0:4])
bit_row("rotate_half_output_lanes_64_67", reference_rotate[0, 0, 0, 64:68])

# --- 2. the full rotary application through the same composition ------------
# `cos` and `sin` are [T, 128] program inputs broadcast over the head axis; the
# reference unsqueezes them at dim 1 for exactly that broadcast.
cos = torch.randn(1, T, HEAD_DIM, dtype=F32)
sin = torch.randn(1, T, HEAD_DIM, dtype=F32)
ref_q, ref_k = modeling_qwen3.apply_rotary_pos_emb(q_heads, k_heads, cos, sin)
composed_q = q_heads * cos.unsqueeze(1) + rotate_half_via_reindex(q_heads) * sin.unsqueeze(1)
composed_k = k_heads * cos.unsqueeze(1) + rotate_half_via_reindex(k_heads) * sin.unsqueeze(1)
row("rope_q_composition_differing_elements", differing(composed_q, ref_q))
row("rope_k_composition_differing_elements", differing(composed_k, ref_k))
row("rope_q_element_count", ref_q.numel())

# --- 3. grouped-query head layout -------------------------------------------
# `repeat_kv` is repeat-interleave, so query head h reads key head h // n_rep.
# Splitting the 16-head axis as (g = 8, r = 2) with h = 2g + r is that mapping.
repeated = modeling_qwen3.repeat_kv(k_heads, N_REP)
by_floor_div = torch.stack([k_heads[0, h // N_REP] for h in range(HEADS_Q)]).unsqueeze(0)
by_modulo = torch.stack([k_heads[0, h % HEADS_KV] for h in range(HEADS_Q)]).unsqueeze(0)
row("gqa_repeat_kv_matches_floor_div_differing_elements", differing(by_floor_div, repeated))
row("gqa_repeat_kv_matches_modulo_differing_elements", differing(by_modulo, repeated))
row("gqa_heads_whose_source_differs_between_the_two_readings",
    sum(1 for h in range(HEADS_Q) if h // N_REP != h % HEADS_KV))
row("gqa_query_head_to_key_head", " ".join(str(h // N_REP) for h in range(HEADS_Q)))

# --- 4. the causal mask, built exactly as the reference builds it ------------
# `torch.full((T, S), finfo.min)` multiplied elementwise by `j > cache_position[i]`.
min_dtype = torch.finfo(F32).min
cache_position = torch.arange(T)
diagonal_attend_mask = torch.arange(S).unsqueeze(0) > cache_position.reshape(-1, 1)
mask_2d = torch.full((T, S), min_dtype, dtype=F32) * diagonal_attend_mask
mask = mask_2d.reshape(1, 1, T, S)
row("mask_masked_entry", bits(min_dtype))
row("mask_attended_entry", bits(mask_2d[0, 0]))
row("mask_attended_entry_is_negative_zero",
    struct.unpack("<I", struct.pack("<f", float(mask_2d[0, 0])))[0] == 0x80000000)
bit_row("mask_row_t2", mask_2d[2])
row("mask_attended_positions_per_row",
    " ".join(str(int((mask_2d[t] == 0.0).sum())) for t in range(T)))
row("mask_rows_with_no_attended_position",
    int(sum(1 for t in range(T) if int((mask_2d[t] == 0.0).sum()) == 0)))

# --- 5. the score -> scale -> mask -> softmax step at the C1 shape -----------
scaling = HEAD_DIM ** -0.5
row("attention_scaling_f32", bits(scaling))
key_states = modeling_qwen3.repeat_kv(k_heads, N_REP)
value_states = modeling_qwen3.repeat_kv(v_heads, N_REP)
raw_scores = torch.matmul(q_heads, key_states.transpose(2, 3))
scaled = raw_scores * scaling
masked = scaled + mask
probs = torch.nn.functional.softmax(masked, dim=-1, dtype=F32)

# One named row, so the design record can walk a concrete example: query head 0
# (group 0, repetition 0), query position 2, over all ten key positions.
bit_row("row_h0_t2_scores_raw", raw_scores[0, 0, 2])
bit_row("row_h0_t2_scores_scaled", scaled[0, 0, 2])
bit_row("row_h0_t2_scores_masked", masked[0, 0, 2])
bit_row("row_h0_t2_probs", probs[0, 0, 2])
row("row_h0_t2_probs_sum", bits(probs[0, 0, 2].sum()))
row("row_h0_t2_probs_sum_is_exactly_one", bits(probs[0, 0, 2].sum()) == "0x3f800000")
row("row_h0_t2_row_maximum", bits(masked[0, 0, 2].max()))
row("row_h0_t2_exactly_positive_zero_positions",
    " ".join(str(j) for j in range(S)
             if struct.unpack("<I", struct.pack("<f", float(probs[0, 0, 2, j])))[0] == 0x00000000))
row("row_h0_t2_masked_positions", " ".join(str(j) for j in range(S) if bool(diagonal_attend_mask[2, j])))
# The scale sits on the score, not on an operand. Pre-scaling the query is a
# different F32 computation, and this counts where.
prescaled = torch.matmul(q_heads * scaling, key_states.transpose(2, 3))
row("prescaled_query_differing_elements_from_scaled_score", differing(prescaled, scaled))
row("score_element_count", scaled.numel())

# --- 6. the score contraction's index structure -----------------------------
# `grtd,gsd->grts` never forms the repeated key tensor. Whether it denotes the
# same values as repeat-then-matmul is the structural question; whether the bits
# agree is a reduction-order question about torch's two paths, and the two are
# reported separately so neither answer is read as the other.
q_grouped = q_heads.reshape(1, HEADS_KV, N_REP, T, HEAD_DIM)[0]
einsum_scores = torch.einsum("grtd,gsd->grts", q_grouped, k_heads[0])
reference_scores = raw_scores.reshape(1, HEADS_KV, N_REP, T, S)[0]
row("score_structure_einsum_differing_elements", differing(einsum_scores, reference_scores))
row("score_structure_einsum_max_absolute_gap",
    repr(float((einsum_scores.double() - reference_scores.double()).abs().max())))
row("score_structure_f64_differing_elements",
    differing(torch.einsum("grtd,gsd->grts", q_grouped.double(), k_heads[0].double()).float(),
              torch.matmul(q_heads.double(),
                           modeling_qwen3.repeat_kv(k_heads, N_REP).double().transpose(2, 3))
              .reshape(1, HEADS_KV, N_REP, T, S)[0].float()))

# --- 7. the value contraction's signed zeros at masked positions ------------
# Query position 0 attends to position 0 alone, so its probability row is a
# single 1.0 followed by nine exact +0.0 entries. Each of those nine contributes
# `+0.0 * v` to the value contraction: +0.0 where v is positive, -0.0 where v is
# negative. An accumulator that is -0.0 becomes +0.0 on the first such addition,
# so the mask rewrites a signed zero inside a contraction that never sees it.
bit_row("row_h0_t0_probs", probs[0, 0, 0])
negative_v = v_heads.clone()
negative_v[0, 0, 0, 0] = -0.0  # v[key 0, lane 0] is exactly negative zero
negative_value_states = modeling_qwen3.repeat_kv(negative_v, N_REP)
out_reference = torch.matmul(probs, negative_value_states)
row("value_contraction_t0_lane0_reference", bits(out_reference[0, 0, 0, 0]))
strict_fold = negative_value_states[0, 0, 0, 0] * probs[0, 0, 0, 0]
for j in range(1, S):
    strict_fold = strict_fold + negative_value_states[0, 0, j, 0] * probs[0, 0, 0, j]
row("value_contraction_t0_lane0_strict_ascending_fold", bits(strict_fold))
row("value_contraction_t0_lane0_first_product", bits(negative_value_states[0, 0, 0, 0] * probs[0, 0, 0, 0]))
row("value_contraction_t0_lane0_masked_contributor_signs",
    " ".join(bits(negative_value_states[0, 0, j, 0] * probs[0, 0, 0, j]) for j in range(1, 4)))
row("value_contraction_masked_contributor_is_a_signed_zero",
    all(float(negative_value_states[0, 0, j, 0] * probs[0, 0, 0, j]) == 0.0 for j in range(1, S)))

# --- 8. what the C1 row can and cannot discriminate --------------------------
# Decision D-1 (the fully masked row) turns on the mask's fill convention, and
# the two conventions disagree observably *only* on a fully masked row. C1 has
# none, so this counts whether the row can tell them apart at all rather than
# assuming it cannot -- an unreachability claim nothing measured is a claim
# about the author's reading of the mask builder.
neginf_mask = torch.where(diagonal_attend_mask, torch.tensor(float("-inf")), torch.tensor(0.0))
probs_neginf = torch.nn.functional.softmax(scaled + neginf_mask.reshape(1, 1, T, S), dim=-1, dtype=F32)
row("softmax_finite_fill_vs_neginf_fill_differing_elements", differing(probs_neginf, probs))
full_row = torch.full((1, S), min_dtype, dtype=F32)
bit_row("softmax_fully_masked_row_width10_finite_fill",
        torch.nn.functional.softmax(full_row, dim=-1, dtype=F32))
bit_row("softmax_fully_masked_row_width10_neginf_fill",
        torch.nn.functional.softmax(torch.full((1, S), float("-inf"), dtype=F32), dim=-1, dtype=F32))

# Whether a softmax row sums to exactly one in F32 is a per-row accident, so a
# conformance check may assert neither that it does nor that it does not.
sums = probs.sum(-1).flatten()
exactly_one = sum(1 for value in sums.tolist() if bits(value) == "0x3f800000")
row("softmax_rows_total", sums.numel())
row("softmax_rows_summing_to_exactly_one", exactly_one)
row("softmax_rows_not_summing_to_exactly_one", int(sums.numel()) - exactly_one)

# --- 9. the whole step against the reference's own entry point --------------
# Everything above recomputes intermediates the reference does not expose. This
# checks that the recomputation *is* the reference's composition, so the
# intermediates above describe the reference rather than a lookalike.
class _Module(torch.nn.Module):
    num_key_value_groups = N_REP
    training = False


ref_out, ref_weights = modeling_qwen3.eager_attention_forward(
    _Module(), q_heads, k_heads, v_heads, mask, scaling, dropout=0.0
)
row("eager_attention_weights_differing_from_recomputation", differing(ref_weights, probs))
row("eager_attention_output_differing_from_recomputation",
    differing(ref_out, torch.matmul(probs, value_states).transpose(1, 2).contiguous()))
row("eager_attention_output_shape", tuple(ref_out.shape))
row("merged_head_axis_extent", HEADS_Q * HEAD_DIM)

for key, value in ROWS:
    print(f"{key}\t{value}")
