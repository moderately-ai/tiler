"""Observe what the pinned reference actually computes for the workload's
non-linear, normalization, and masking families.

Every question here is one a Tiler contract must answer and a formula written
from memory answers wrongly. The probe does not assert a Tiler contract; it
records what `transformers` 4.51.0 on `torch` 2.6.0 does, in F32, on CPU, so a
contract can be derived from an observation instead of from a recollection.

Run from this directory:

    uv run --offline python probe.py            # or drop --offline to resolve

Output is one `key<TAB>value` record per line, ordered and deterministic, so it
can be captured and byte-compared against the retained record beside it.
"""

from __future__ import annotations

import itertools
import math
import struct

import torch
import torch.nn.functional as F
import transformers
from transformers.models.qwen3 import modeling_qwen3

F32 = torch.float32
ROWS: list[tuple[str, str]] = []


def bits(value: float) -> str:
    """The exact F32 bit pattern, because a decimal rendering hides -0.0 and NaN payloads."""
    return "0x%08x" % struct.unpack("<I", struct.pack("<f", float(value)))[0]


def row(key: str, value: object) -> None:
    ROWS.append((key, str(value)))


def bit_row(key: str, tensor: torch.Tensor) -> None:
    row(key, " ".join(bits(v) for v in tensor.flatten().tolist()))


# --- environment identity --------------------------------------------------
row("torch_version", torch.__version__)
row("transformers_version", transformers.__version__)
row("device", "cpu")
row("dtype", "float32")

# --- the activation the checkpoint actually declares ------------------------
# `config.json` declares hidden_act "silu"; this resolves that name through the
# reference's own table rather than assuming which activation the name selects.
row("ACT2FN_silu_class", type(transformers.activations.ACT2FN["silu"]).__qualname__)
with open(modeling_qwen3.__file__, encoding="utf-8") as handle:
    row("qwen3_mlp_uses_ACT2FN", "act_fn = ACT2FN[config.hidden_act]" in handle.read())

# --- the causal mask's fill value is finite, not -inf -----------------------
# `_prepare_4d_causal_attention_mask_with_cache_position` fills with
# `torch.finfo(dtype).min` and then multiplies by a boolean attend mask, so an
# attended entry is that value times zero rather than a written zero.
min_dtype = torch.finfo(F32).min
row("mask_fill_masked", bits(min_dtype))
row("mask_fill_attended", bits(min_dtype * 0.0))
row("mask_fill_is_negative_infinity", math.isinf(min_dtype))

# --- the attention scale ----------------------------------------------------
# `Qwen3Attention.scaling` is `head_dim ** -0.5` evaluated as a Python float and
# then applied to an F32 tensor, so the effective multiplier is its F32 rounding.
scaling = 128 ** -0.5
row("attention_scaling_f64", repr(scaling))
row("attention_scaling_f32", bits(scaling))

# --- softmax: is the row maximum subtracted? --------------------------------
# The two forms differ observably: a naive exponential overflows where a
# max-subtracting one does not, so this discriminates rather than agreeing.
equal_large = torch.tensor([[1000.0, 1000.0]], dtype=F32)
bit_row("softmax_equal_large", F.softmax(equal_large, dim=-1, dtype=F32))
naive = torch.exp(equal_large)
bit_row("softmax_equal_large_naive_form", naive / naive.sum(-1, keepdim=True))

# --- softmax under each masking convention ----------------------------------
bit_row("softmax_row_all_masked_finite",
        F.softmax(torch.full((1, 3), min_dtype, dtype=F32), dim=-1, dtype=F32))
bit_row("softmax_row_all_masked_neginf",
        F.softmax(torch.full((1, 3), -math.inf, dtype=F32), dim=-1, dtype=F32))
bit_row("softmax_one_live_score_finite_mask",
        F.softmax(torch.tensor([[0.5, min_dtype, min_dtype]], dtype=F32), dim=-1, dtype=F32))
row("softmax_empty_reduced_axis_shape",
    tuple(F.softmax(torch.zeros((1, 0), dtype=F32), dim=-1, dtype=F32).shape))

# --- softmax: the L3' record's own worked example, and its intermediates -----
# The derivation's worked example is the row every downstream document quotes,
# and until now its bits lived only in that document: a reader could not check
# them against this record, and the divergence between the reference and the
# pinned formula had to be re-measured rather than read. Both sides are recorded
# here, together with the intermediates that localize the difference to one step.
worked = torch.tensor([[1.0, 2.0, 3.0, min_dtype]], dtype=F32)
worked_e = torch.exp(worked - worked.max(-1, keepdim=True).values)
worked_d = worked_e.sum(-1, keepdim=True)
bit_row("softmax_worked_example_scores", worked)
bit_row("softmax_worked_example_exponentials", worked_e)
row("softmax_worked_example_denominator_torch_sum", bits(worked_d))
row("softmax_worked_example_denominator_strict_left_fold",
    bits(((worked_e[0, 0] + worked_e[0, 1]) + worked_e[0, 2]) + worked_e[0, 3]))
bit_row("softmax_worked_example_reference", F.softmax(worked, dim=-1, dtype=F32))
bit_row("softmax_worked_example_reciprocal_form", worked_e * (1.0 / worked_d))
bit_row("softmax_worked_example_divide_form", worked_e / worked_d)
# After the maximum subtraction the largest score's exponential is exactly 1.0,
# so whatever constant the reference multiplies the row by is readable *exactly*
# as its output at that position. This is the reference's implied normalization
# constant, observed rather than inferred.
row("softmax_worked_example_reference_implied_constant",
    bits(F.softmax(worked, dim=-1, dtype=F32)[0, 2]))
row("softmax_worked_example_correctly_rounded_reciprocal", bits(1.0 / worked_d))
# The implied constant is *not* an approximation of the reciprocal: it is the
# correctly rounded reciprocal of a denominator this row's own exponentials reach
# under a different contributor order. The order is named so the claim is
# checkable by hand rather than by trusting the search below.
reordered = ((worked_e[0, 0] + worked_e[0, 2]) + worked_e[0, 1]) + worked_e[0, 3]
row("softmax_worked_example_denominator_under_order_0_2_1_3", bits(reordered))
row("softmax_worked_example_reciprocal_of_that_denominator", bits(1.0 / reordered))

# --- softmax and the row maximum on a NaN score -----------------------------
# The extrema family's NaN rule is a decision the softmax key had to make, and
# this record carried no softmax row with a NaN score, so the evidence had to be
# taken outside it. It is inside it now.
nan_row = torch.tensor([[1.0, math.nan, 3.0]], dtype=F32)
bit_row("softmax_row_with_a_nan_score", F.softmax(nan_row, dim=-1, dtype=F32))
row("torch_max_of_row_with_a_nan_score", bits(nan_row.max()))

# The signed-zero half of the same question, in *both* operand orders and in two
# spellings, because a single order cannot tell an ordering rule apart from an
# order dependence -- and the answer here is the order dependence. Neither
# spelling implements the `-0.0 < +0.0` total ordering: each returns a fixed
# position rather than a fixed value, and the two spellings disagree on which.
for label, pair in (("plus_then_minus", [0.0, -0.0]), ("minus_then_plus", [-0.0, 0.0])):
    zeros = torch.tensor(pair, dtype=F32)
    row(f"torch_max_of_signed_zeros_{label}", bits(zeros.max()))
    row(f"torch_amax_of_signed_zeros_{label}", bits(torch.amax(zeros)))

# --- softmax: does it divide by the denominator or multiply by its reciprocal?
# The two are different F32 computations, and which one the pinned reference
# performs is a formula the Tiler contract must copy rather than choose. The
# count is restricted to elements where the two forms disagree, because
# elsewhere agreement carries no information. Two and three contributors are the
# widths that isolate the normalization form from reduction-order noise in the
# sum -- observed rather than assumed: a two-term sum has no ordering freedom at
# all, a three-term sum does have grouping freedom, and the `softmax_constant_*`
# rows below record that the reference nonetheless lands on the naive sum's
# reciprocal in all 20,000 rows at both widths. It is a measured property of
# these rows, not a theorem about three contributors.
torch.manual_seed(20260731)
sweep: dict[int, tuple[torch.Tensor, ...]] = {}
for width in (2, 3, 4, 8, 18):
    scores = (torch.rand(20000, width, dtype=F32) * 20.0) - 10.0
    shifted = scores - scores.max(-1, keepdim=True).values
    numer = torch.exp(shifted)
    denom = numer.sum(-1, keepdim=True)
    as_divide = (numer / denom).view(torch.int32)
    as_reciprocal = (numer * (1.0 / denom)).view(torch.int32)
    reference = F.softmax(scores, dim=-1, dtype=F32).view(torch.int32)
    discriminating = as_divide != as_reciprocal
    row(f"softmax_form_width_{width}", " ".join((
        f"discriminating={int(discriminating.sum())}",
        f"matches_divide={int((discriminating & (reference == as_divide)).sum())}",
        f"matches_reciprocal={int((discriminating & (reference == as_reciprocal)).sum())}",
        f"matches_neither={int((discriminating & (reference != as_divide) & (reference != as_reciprocal)).sum())}",
    )))
    sweep[width] = (scores, numer, denom)

# --- what the `matches_neither` bucket actually is --------------------------
# The counts above say the reference matches neither spelling above width three;
# they do not say why, and two hypotheses fit them equally at that resolution: a
# denominator whose accumulation order differs from the naive sum, or a
# normalization constant that is not the correctly rounded reciprocal of *any*
# denominator. The rows below separate them, because a count that named one
# cause without eliminating the other would be an attribution rather than a
# measurement.
#
# The lever is the maximum subtraction: it makes the largest score's exponential
# exactly 1.0, so the reference's output at that position IS the constant it
# multiplied the row by, read off exactly rather than solved for.
for width, (scores, numer, denom) in sweep.items():
    reference = F.softmax(scores, dim=-1, dtype=F32)
    argmax = scores.argmax(-1, keepdim=True)
    assert bool(torch.all(numer.gather(1, argmax) == 1.0)), "the shifted maximum exponentiates to one"
    implied = reference.gather(1, argmax)
    # First: is the whole row one scalar multiple of the exponentials? If it is,
    # the reference's exponentials agree with these bit for bit and the entire
    # divergence is that one scalar -- which is also the strongest available
    # evidence for the *reciprocal-multiply form*, since a division by a
    # denominator is not a single-constant multiply.
    explained = torch.all(
        (numer * implied).view(torch.int32) == reference.view(torch.int32), dim=-1
    )
    gap = (implied.view(torch.int32) - (1.0 / denom).view(torch.int32)).flatten()
    row(f"softmax_constant_width_{width}", " ".join((
        f"rows={scores.shape[0]}",
        f"explained_by_one_constant={int(explained.sum())}",
        f"constant_is_correctly_rounded_reciprocal_of_the_naive_sum={int((gap == 0).sum())}",
        f"max_abs_constant_ulp_gap={int(gap.abs().max())}",
    )))

# Second, and only where it can be exhaustive: is the reference's constant the
# correctly rounded reciprocal of a denominator the same exponentials reach under
# *some* summation order? Four contributors admit 24 strict left folds and the
# balanced tree, which is enumerable; eight and eighteen are not, so the question
# is answered where it can be answered and left open where it cannot. The count
# is a lower bound on reachability, because the enumeration is not every legal
# grouping -- so a high count eliminates the second hypothesis and a shortfall
# does not establish it.
scores, numer, _ = sweep[4]
reference4 = F.softmax(scores, dim=-1, dtype=F32)
implied4 = reference4.gather(1, scores.argmax(-1, keepdim=True))
reachable = torch.zeros_like(implied4, dtype=torch.bool)
for order in itertools.permutations(range(4)):
    fold = numer[:, order[0]:order[0] + 1].clone()
    for index in order[1:]:
        fold = fold + numer[:, index:index + 1]
    reachable |= (1.0 / fold).view(torch.int32) == implied4.view(torch.int32)
balanced = (numer[:, 0:1] + numer[:, 1:2]) + (numer[:, 2:3] + numer[:, 3:4])
reachable |= (1.0 / balanced).view(torch.int32) == implied4.view(torch.int32)
row("softmax_constant_reachable_by_some_summation_order_width_4",
    f"rows={scores.shape[0]} reachable={int(reachable.sum())}")

# --- softmax exponent range after max subtraction ---------------------------
# Every argument is <= 0 once the maximum is subtracted, so overflow is
# unreachable and underflow is the only exceptional direction that remains.
row("exp_f32_overflow_threshold_approx", bits(math.log(torch.finfo(F32).max)))


def largest_argument_whose_exp_is(predicate) -> float:
    """Bisect over F32 arguments for the boundary of a monotone exp property."""
    low, high = -200.0, 0.0  # exp(-200) is 0; exp(0) is 1
    for _ in range(200):
        mid = (low + high) / 2.0
        if predicate(float(torch.exp(torch.tensor(mid, dtype=F32)))):
            low = mid
        else:
            high = mid
    return low


row("exp_f32_last_argument_flushing_to_zero",
    bits(largest_argument_whose_exp_is(lambda y: y == 0.0)))
row("exp_f32_last_argument_producing_a_subnormal",
    bits(largest_argument_whose_exp_is(lambda y: y < float(torch.finfo(F32).tiny))))

# --- RMS normalization, exactly as Qwen3RMSNorm.forward spells it -----------


def rms_norm(h: torch.Tensor, eps: float = 1e-6) -> torch.Tensor:
    """Lines 74-75 of the pinned reference: mean of squares, eps inside rsqrt."""
    variance = h.pow(2).mean(-1, keepdim=True)
    return h * torch.rsqrt(variance + eps)


row("rms_eps", repr(1e-6))
row("rsqrt_of_eps_alone", bits(torch.rsqrt(torch.tensor(1e-6, dtype=F32))))
bit_row("rms_zero_vector", rms_norm(torch.zeros(1, 4, dtype=F32)))
bit_row("rms_subnormal_vector", rms_norm(torch.full((1, 4), 1e-40, dtype=F32)))
# The squaring, not the normalization, is what overflows: |x| >= sqrt(FLT_MAX)
# makes the mean of squares infinite, rsqrt(inf) zero, and the result all-zero.
row("rms_square_overflow_threshold", bits(math.sqrt(torch.finfo(F32).max)))
big = torch.full((1, 4), 1e20, dtype=F32)
row("rms_big_vector_mean_of_squares", bits(big.pow(2).mean(-1)))
bit_row("rms_big_vector_result", rms_norm(big))

# --- SiLU, three spellings, on the boundary corpus --------------------------
silu_inputs = torch.tensor(
    [-100.0, -88.0, -20.0, -1.0, -0.0, 0.0, 1.0, 20.0, 100.0,
     -math.inf, math.inf, math.nan],
    dtype=F32,
)
bit_row("silu_inputs", silu_inputs)
spellings = {
    "torch_reference": F.silu(silu_inputs),
    "x_over_one_plus_exp_neg_x": silu_inputs / (1.0 + torch.exp(-silu_inputs)),
    "x_times_sigmoid_x": silu_inputs * torch.sigmoid(silu_inputs),
}
for name, result in spellings.items():
    bit_row(f"silu_{name}", result)

# Two spellings of "the same" activation are the same operation only where their
# bits agree. Naming the disagreeing inputs is the point: a contract that picks
# one spelling has to know it is picking, and a contract that treats them as
# interchangeable is wrong exactly here.
reference = spellings["torch_reference"].view(torch.int32)
for name, result in spellings.items():
    differing = (result.view(torch.int32) != reference).nonzero().flatten().tolist()
    row(f"silu_{name}_inputs_differing_from_reference",
        " ".join(bits(silu_inputs[i]) for i in differing) or "none")

for key, value in ROWS:
    print(f"{key}\t{value}")
