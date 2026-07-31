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

# --- softmax: does it divide by the denominator or multiply by its reciprocal?
# The two are different F32 computations, and which one the pinned reference
# performs is a formula the Tiler contract must copy rather than choose. The
# count is restricted to elements where the two forms disagree, because
# elsewhere agreement carries no information; at two and three contributors the
# denominator has no accumulation-order freedom left, so those rows isolate the
# normalization form from reduction-order noise in the sum.
torch.manual_seed(20260731)
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
