#!/usr/bin/env python3
"""Produce the C1 conformance and attribution reference fixture for Qwen/Qwen3-0.6B-Base.

The workload profile at `docs/research/program-planning/first-metal-lm-workload.md`
supplies every constant below -- the pinned revision, the per-file manifest, the
prompt token IDs, the decode budget, the termination rule, and the tie policy.
Nothing here re-derives them; the constants are transcribed and the profile is
the authority. The attribution surface's content is fixed by the L6 record at
`docs/research/program-planning/complete-model-ingestion-and-execution.md`.

What the run produces, in order:

1. The checkpoint is acquired at the pinned revision into the Hugging Face cache
   outside this repository, and every manifest file is hashed locally. This is
   the step that converts the profile's API-reported Git-LFS object id for
   `model.safetensors` into a digest computed from bytes on this host.
2. The installed `transformers` reference sources are hashed against the
   profile's pinned-commit digests, so "the pinned reference was evaluated" is a
   checked claim rather than an inference from a version string.
3. The F32 reference is evaluated on CPU with `attn_implementation="eager"` and
   `logits_to_keep=0`, greedy, over all 10 prefill positions and 8 decode steps.
   Forward hooks observe that same evaluation and retain the attribution surface
   -- see ATTRIBUTION below.
4. The widened F32 weights the F32 pass actually used are digested against the
   checkpoint's own BF16 bytes.
5. The same staging is evaluated twice more in float64 to measure the
   reference's own sensitivity envelope; see ENVELOPE below for why there are
   two float64 passes rather than one. The float64 passes carry no hooks: the
   attribution surface describes the F32 pass and nothing else.

Run it by hand from this directory; no `make` target reaches a spike:

    uv run --locked python produce_fixture.py --out results/<slug>

`--compare DIR` re-runs the whole production into a scratch directory and
byte-compares every retained file against `DIR`, which is how reproducibility is
demonstrated rather than asserted.
"""

from __future__ import annotations

import argparse
import gc
import hashlib
import json
import math
import platform
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# ---------------------------------------------------------------------------
# Transcribed from the workload profile. A mismatch against any of these is a
# stop, never a warning: the retained record is only evidence about the pinned
# workload, so a run against different bytes is not a weaker fixture, it is a
# fixture for a different question.
# ---------------------------------------------------------------------------

REPO_ID = "Qwen/Qwen3-0.6B-Base"
REVISION = "da87bfb608c14b7cf20ba1ce41287e8de496c0cd"

# (filename, byte size, SHA-256 of content bytes)
CHECKPOINT_MANIFEST = [
    ("config.json", 727, "504a6b58c4271583724e66584b6b7698aea18450209df6b2f7582df0e89cee59"),
    ("generation_config.json", 138, "8c970692323e3ea0e9b8b0a4dca79388d31226e41f83c9fd6014804280ebf6e8"),
    ("model.safetensors", 1192135096, "cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba"),
    ("tokenizer.json", 7031645, "c0382117ea329cdf097041132f6d735924b697924d6f6fc3945713e96ce87539"),
    ("tokenizer_config.json", 9678, "3c04ed3ca964ea2f6b2b5faf0dc4d31aec1cb1e8b4bcf63f402d295046b422b5"),
    ("vocab.json", 2776833, "ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910"),
    ("merges.txt", 1671853, "8831e4f1a044471340f7c0a83d7bd71306a5b867e95fd870f74d0c5308a904d5"),
    ("LICENSE", 11343, "832dd9e00a68dd83b3c3fb9f5588dad7dcf337a0db50f7d9483f310cd292e92e"),
    ("README.md", 2973, "910d9be25c648ab1cb5a7b1d20d67ca6d43d43559a705010198886f9af68e8f1"),
]

# The `transformers` v4.51.0 sources the profile pins by git commit
# 0720e206c6ba28887e4d60ef60a6a089f6c1cc76. Verifying the *installed* files
# against these digests is what makes "evaluated the pinned reference" checkable.
REFERENCE_SOURCE_MANIFEST = [
    ("models/qwen3/modeling_qwen3.py", "704c914530530a1acb0b443add1f520404e3ac2c28c0ab7e16f80f86cfe8ccb2"),
    ("models/qwen3/configuration_qwen3.py", "87f0d17326c44f2dfe1bfc329faf9201ab4b19a89ad555da085b4cc81461b201"),
    ("modeling_rope_utils.py", "c28b3e88edca8fdb5497e5c36091bf753db49bd94ace33a84e9f9c61cbf66032"),
]

PROMPT_TEXT = "The quick brown fox jumps over the lazy dog."
PROMPT_IDS = [785, 3974, 13876, 38835, 34208, 916, 279, 15678, 5562, 13]
DECODE_STEPS = 8
EOS_TOKEN_ID = 151643
VOCAB_SIZE = 151936
TOP_K = 32

# 10 prefill positions plus 8 decode forward passes. The eighth decode pass
# consumes the eighth generated token, so the retained set is 18 logit vectors
# at positions 0..17 and the maximum context reached is 18 -- both figures the
# profile's C1 table states. The argmax at position 17 is recorded per position
# but is not appended to the sequence, because the 8-step budget is spent.
EXPECTED_POSITIONS = len(PROMPT_IDS) + DECODE_STEPS

# ---------------------------------------------------------------------------
# ATTRIBUTION -- what the logits cannot say, and the surface that can.
#
# The five model-boundary observables are the pass or fail, and they carry no
# execution ordinal: one forward pass is 30 executions over ten operation
# families and four host computations, so a logit disagreement is attributable
# to any of them at once. The L6 record fixes the remedy as the reference's own
# intermediate values, retained beside its logits under L1's retention policy.
#
# The three configuration constants below are L1's, read from the pinned
# `config.json`; the byte figures are L6's, and are asserted at run time rather
# than only stated, so a shape that drifts stops the run.
#
# Nothing here is or implies a tolerance. `design-model-level-qualification-and-
# optimization` owns the bound, and L1 already fixes that composing one from
# per-operation deviations is the defect rather than the method. This surface is
# where a comparison happens, not how wide it may be.
# ---------------------------------------------------------------------------

NUM_LAYERS = 28
HIDDEN_SIZE = 1024
KV_HEADS = 8
HEAD_DIM = 128

# Per-layer `h_out` across the 18 positions, and the post-RoPE `k_rope` and
# `v_heads` the run's own cache ends holding. Both figures are L6's.
HIDDEN_BYTES = NUM_LAYERS * EXPECTED_POSITIONS * HIDDEN_SIZE * 4  # 2,064,384
CACHE_TENSORS = ("k_rope", "v_heads")
CACHE_BYTES = NUM_LAYERS * len(CACHE_TENSORS) * KV_HEADS * EXPECTED_POSITIONS * HEAD_DIM * 4  # 4,128,768

# One position of one retained tensor is 4,096 bytes under both layouts: a
# hidden state is `[18, 1024]` and a cache tensor is `[8, 18, 128]`, so a
# position slice is 1,024 or 8 x 128 F32 values either way. That is what makes
# one digest rule cover both.
POSITION_SLICE_BYTES = HIDDEN_SIZE * 4

# The bounded comparison values, on L1's footing for the logits: the reference's
# own top entries by magnitude, with the coordinates a candidate is indexed at.
# Four rather than L1's thirty-two because a 1,024-wide vector has no ranking
# semantics to preserve -- only the coordinates where an absolute deviation
# would be largest.
ATTRIBUTION_TOP_K = 4

# L4 fixes the additive causal mask's two values; the producer admits no third.
MASK_MASKED_ENTRY = 0xFF7FFFFF
MASK_ATTENDED_ENTRY = 0x80000000

# L1's tensor inventory, and the F32 weight budget it computes from it.
WEIGHT_TENSOR_COUNT = 310
WIDENED_WEIGHT_BYTES = 2_384_199_680

# ---------------------------------------------------------------------------
# ENVELOPE -- why there are two float64 passes.
#
# The profile asks for "a float64 path rounded to F32 at the observable".
# Loading the pinned reference at float64 does not by itself produce one. Three
# float32 spellings in the pinned source are unconditional, so at model dtype
# float64 they are *downcasts* rather than the upcasts they are for a BF16
# model:
#
#   modeling_qwen3.py:73       Qwen3RMSNorm.forward      hidden_states.to(torch.float32)
#   modeling_qwen3.py:162      eager_attention_forward   softmax(..., dtype=torch.float32)
#   modeling_qwen3.py:336-344  Qwen3RotaryEmbedding.forward  .float() on inv_freq,
#                              position_ids, freqs, cos and sin
#
# Those are the mean-of-squares normalization, the attention softmax, and the
# RoPE table: three of the most cancellation-prone stages in the model. In an
# unmodified float64 pass they round *identically* to the F32 pass and so
# contribute exactly zero to the measured deviation, which understates the
# envelope precisely where it matters most. The run therefore retains both:
#
#   f64_unmodified -- the pinned reference verbatim at dtype float64. Needs no
#       patching, reproduces from the checked-in environment alone, and is the
#       conservative floor.
#   f64_promoted   -- the same three sites promoted to float64, each a
#       line-for-line copy of the pinned source with the float32 spelling
#       changed and nothing else moved. This is the pass that actually reorders
#       the sensitive stages.
#
# Neither sets a budget; `design-model-level-qualification-and-optimization`
# owns that. Both are Measurements bound to the host row recorded beside them.
# ---------------------------------------------------------------------------

RESULT_FILES = [
    "environment.tsv",
    "sequence.tsv",
    "positions.tsv",
    "top32.tsv",
    "envelope.tsv",
    "hidden.tsv",
    "hidden_top.tsv",
    "cache.tsv",
    "cache_top.tsv",
    "rotary.tsv",
    "mask.tsv",
    "host.tsv",
]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def exact_norm(values) -> float:
    """The Euclidean norm, computed so that it is a property of the values alone.

    Each F32 value squares exactly in float64 -- 24 significand bits square to
    at most 48, and the F32 exponent range squares well inside float64's -- and
    `math.fsum` is exactly rounded, so the sum depends on neither the summation
    order nor the host's reduction strategy. `sqrt` is correctly rounded by
    IEEE-754. The retained norm is therefore the one figure in this record that
    is portable, which is what makes it usable as a whole-tensor comparison
    scalar when the per-lane ranking below has permuted.
    """
    return math.sqrt(math.fsum(value * value for value in values.astype("float64").ravel().tolist()))


def fail(message: str):
    print(f"FIXTURE STOP: {message}", file=sys.stderr)
    raise SystemExit(4)


def host_row():
    def command(argv):
        try:
            return subprocess.run(argv, capture_output=True, text=True, check=True).stdout.strip()
        except (subprocess.CalledProcessError, FileNotFoundError):
            return "unavailable"

    return [
        ("host.os_product", command(["sw_vers", "-productName"])),
        ("host.os_version", command(["sw_vers", "-productVersion"])),
        ("host.os_build", command(["sw_vers", "-buildVersion"])),
        ("host.cpu", command(["sysctl", "-n", "machdep.cpu.brand_string"])),
        ("host.machine", platform.machine()),
        ("host.memory_bytes", command(["sysctl", "-n", "hw.memsize"])),
    ]


# ---------------------------------------------------------------------------
# Verification steps, all fail-closed.
# ---------------------------------------------------------------------------


def acquire_and_verify_checkpoint():
    from huggingface_hub import snapshot_download

    snapshot = Path(
        snapshot_download(
            repo_id=REPO_ID,
            revision=REVISION,
            allow_patterns=[name for name, _, _ in CHECKPOINT_MANIFEST],
        )
    )

    records = []
    for name, expected_size, expected_digest in CHECKPOINT_MANIFEST:
        path = snapshot / name
        if not path.exists():
            fail(f"{name} missing from the acquired snapshot at {snapshot}")
        actual_size = path.resolve().stat().st_size
        if actual_size != expected_size:
            fail(f"{name} is {actual_size} bytes, the manifest says {expected_size}")
        actual_digest = sha256_file(path)
        if actual_digest != expected_digest:
            fail(
                f"{name} hashed to {actual_digest}, the manifest says {expected_digest}. "
                "The acquired bytes are not the pinned revision."
            )
        records.append((f"checkpoint.bytes.{name}", str(actual_size)))
        records.append((f"checkpoint.sha256.{name}", actual_digest))
    return snapshot, records


def verify_reference_sources():
    import transformers

    root = Path(transformers.__file__).parent
    records = []
    for relative, expected_digest in REFERENCE_SOURCE_MANIFEST:
        path = root / relative
        if not path.exists():
            fail(f"the installed transformers is missing {relative}")
        actual_digest = sha256_file(path)
        if actual_digest != expected_digest:
            fail(
                f"the installed {relative} hashed to {actual_digest}, the pinned commit's "
                f"digest is {expected_digest}. The evaluated implementation is not the "
                "pinned reference."
            )
        records.append((f"reference.bytes.{relative}", str(path.stat().st_size)))
        records.append((f"reference.sha256.{relative}", actual_digest))
    return records


def verify_prompt_tokenization(snapshot: Path):
    """Cross-check the profile's recorded prompt IDs against the pinned tokenizer.

    The profile fixes the workload as token IDs precisely so that a differing
    tokenizer shows up as a visible mismatch instead of a silent change of
    input. This is that check, run against the tokenizer file whose digest the
    checkpoint step already verified.
    """
    import tokenizers
    from tokenizers import Tokenizer

    tokenizer = Tokenizer.from_file(str(snapshot / "tokenizer.json"))
    encoded = tokenizer.encode(PROMPT_TEXT, add_special_tokens=False).ids
    if encoded != PROMPT_IDS:
        fail(
            f"the pinned tokenizer encodes the prompt as {encoded}, the profile records "
            f"{PROMPT_IDS}. The workload input is not what the profile says it is."
        )
    decoded = tokenizer.decode(PROMPT_IDS)
    if decoded != PROMPT_TEXT:
        fail(f"decoding the recorded IDs returned {decoded!r}, not {PROMPT_TEXT!r}")
    return [
        ("tokenizer.library", "tokenizers"),
        ("tokenizer.version", tokenizers.__version__),
        ("tokenizer.prompt_ids_match_profile", "true"),
        ("tokenizer.roundtrip_matches_prompt_text", "true"),
    ]


# ---------------------------------------------------------------------------
# Evaluation.
# ---------------------------------------------------------------------------


def promote_reference_to_float64(modeling):
    """Replace the three unconditional float32 sites with float64 copies.

    Each replacement is a line-for-line copy of the pinned source with the
    float32 spelling changed; nothing else about the computation moves.
    """
    import torch
    from torch import nn
    from transformers.modeling_rope_utils import dynamic_rope_update

    def rms_norm_forward(self, hidden_states):  # modeling_qwen3.py:71-76
        input_dtype = hidden_states.dtype
        hidden_states = hidden_states.to(torch.float64)
        variance = hidden_states.pow(2).mean(-1, keepdim=True)
        hidden_states = hidden_states * torch.rsqrt(variance + self.variance_epsilon)
        return self.weight * hidden_states.to(input_dtype)

    def eager_attention_forward(  # modeling_qwen3.py:144-167
        module, query, key, value, attention_mask, scaling, dropout=0.0, **kwargs
    ):
        key_states = modeling.repeat_kv(key, module.num_key_value_groups)
        value_states = modeling.repeat_kv(value, module.num_key_value_groups)

        attn_weights = torch.matmul(query, key_states.transpose(2, 3)) * scaling
        if attention_mask is not None:
            causal_mask = attention_mask[:, :, :, : key_states.shape[-2]]
            attn_weights = attn_weights + causal_mask

        attn_weights = nn.functional.softmax(attn_weights, dim=-1, dtype=torch.float64).to(query.dtype)
        attn_weights = nn.functional.dropout(attn_weights, p=dropout, training=module.training)
        attn_output = torch.matmul(attn_weights, value_states)
        attn_output = attn_output.transpose(1, 2).contiguous()

        return attn_output, attn_weights

    @torch.no_grad()
    @dynamic_rope_update
    def rotary_forward(self, x, position_ids):  # modeling_qwen3.py:333-346
        inv_freq_expanded = (
            self.inv_freq[None, :, None].double().expand(position_ids.shape[0], -1, 1).to(x.device)
        )
        position_ids_expanded = position_ids[:, None, :].double()

        device_type = x.device.type if isinstance(x.device.type, str) and x.device.type != "mps" else "cpu"
        with torch.autocast(device_type=device_type, enabled=False):
            freqs = (inv_freq_expanded.double() @ position_ids_expanded.double()).transpose(1, 2)
            emb = torch.cat((freqs, freqs), dim=-1)
            cos = emb.cos() * self.attention_scaling
            sin = emb.sin() * self.attention_scaling

        return cos.to(dtype=x.dtype), sin.to(dtype=x.dtype)

    modeling.Qwen3RMSNorm.forward = rms_norm_forward
    modeling.eager_attention_forward = eager_attention_forward
    modeling.Qwen3RotaryEmbedding.forward = rotary_forward


def load_model(snapshot: Path, dtype):
    from transformers import AutoModelForCausalLM

    model = AutoModelForCausalLM.from_pretrained(
        str(snapshot), torch_dtype=dtype, attn_implementation="eager"
    )
    model.eval()
    if model.config._attn_implementation != "eager":
        fail(
            "the loaded model reports attention implementation "
            f"{model.config._attn_implementation!r}, not 'eager'"
        )
    actual_dtype = next(model.parameters()).dtype
    if actual_dtype != dtype:
        fail(f"the loaded model parameters are {actual_dtype}, not {dtype}")
    return model


def greedy_token(row) -> int:
    """The lowest vocabulary index among all indices attaining the maximum logit.

    This is the profile's declared tie policy, applied rather than inherited
    from whatever `torch.argmax` happens to return for a tie.
    """
    import torch

    maximum = torch.max(row)
    return int(torch.nonzero(row == maximum, as_tuple=False)[0].item())


class AttributionCollector:
    """Observe the F32 pass's own intermediates without altering what it computes.

    Forward hooks are used rather than `output_hidden_states=True`, and the
    reason is a property of the pinned source rather than a preference.
    `Qwen3Model.forward` appends `hidden_states` at the *top* of each layer
    iteration and appends `self.norm(hidden_states)` after the loop, so the
    returned tuple is the embedding output, twenty-seven layer outputs, and the
    normed final state -- it never contains layer 27's own `h_out`, which is
    exactly one of the twenty-eight tensors this surface is for. A hook on each
    `Qwen3DecoderLayer` returns all twenty-eight and changes no value.

    The mask and the rotary rows are read from the same call for the same
    reason: `_update_causal_mask` and `rotary_emb` produce them inside
    `Qwen3Model.forward`, and recomputing them here would retain a lookalike of
    the host computation rather than the one the retained logits were produced
    under.
    """

    def __init__(self) -> None:
        self.hidden = [[] for _ in range(NUM_LAYERS)]
        self.rotary = []  # one (cos, sin) pair per pass
        self.masks = []  # one (stage, [T, S] array) per pass
        self.pass_positions = []  # the absolute positions each pass produced
        self.cache = None  # (k_rope, v_heads) per layer, read once at the end
        self._handles = []
        self._stage = None

    def begin_pass(self, stage: str, positions) -> None:
        self._stage = stage
        self.pass_positions.append(list(positions))

    def attach(self, model) -> None:
        inner = model.model
        if len(inner.layers) != NUM_LAYERS:
            fail(f"the loaded model has {len(inner.layers)} layers, the profile records {NUM_LAYERS}")
        for index, layer in enumerate(inner.layers):
            self._handles.append(
                layer.register_forward_hook(self._layer_hook(index), with_kwargs=True)
            )
        self._handles.append(inner.rotary_emb.register_forward_hook(self._rotary_hook))

    def detach(self) -> None:
        for handle in self._handles:
            handle.remove()
        self._handles = []

    def _layer_hook(self, index: int):
        def hook(module, args, kwargs, output):
            import numpy as np
            import torch

            state = output[0]
            if state.dim() != 3 or state.shape[0] != 1 or state.shape[2] != HIDDEN_SIZE:
                fail(f"layer {index} emitted a hidden state of shape {tuple(state.shape)}")
            self.hidden[index].append(
                np.ascontiguousarray(state[0].to(torch.float32).numpy(), dtype=np.float32)
            )

            if index != 0:
                return
            # The mask reaches the layer as a keyword argument in the pinned
            # source. Reading it positionally would silently retain whatever
            # `args` happened to hold if that ever changed, so this fails closed.
            if "attention_mask" not in kwargs:
                fail("the pinned layer was called without an `attention_mask` keyword argument")
            mask = kwargs["attention_mask"]
            if mask is None:
                fail("the pinned reference produced no causal mask for this pass")
            if mask.dim() != 4 or mask.shape[0] != 1 or mask.shape[1] != 1:
                fail(f"the causal mask has shape {tuple(mask.shape)}, expected [1, 1, T, S]")
            self.masks.append(
                (self._stage, np.ascontiguousarray(mask[0, 0].to(torch.float32).numpy(), dtype=np.float32))
            )

        return hook

    def _rotary_hook(self, module, args, output):
        import numpy as np
        import torch

        cos, sin = output
        for name, table in (("cos", cos), ("sin", sin)):
            if table.dim() != 3 or table.shape[0] != 1 or table.shape[2] != HEAD_DIM:
                fail(f"the rotary {name} table has shape {tuple(table.shape)}, expected [1, T, {HEAD_DIM}]")
        self.rotary.append(
            (
                np.ascontiguousarray(cos[0].to(torch.float32).numpy(), dtype=np.float32),
                np.ascontiguousarray(sin[0].to(torch.float32).numpy(), dtype=np.float32),
            )
        )

    def read_cache(self, cache) -> None:
        """Read the run's own KV cache once it holds all 18 positions.

        `k_rope` is post-RoPE because the pinned `Qwen3Attention.forward` applies
        `apply_rotary_pos_emb` before `past_key_value.update`; `v_heads` is the
        value projection reshaped, with no normalization and no rotary.
        """
        import numpy as np

        if len(cache.key_cache) != NUM_LAYERS or len(cache.value_cache) != NUM_LAYERS:
            fail(
                f"the cache holds {len(cache.key_cache)} key and {len(cache.value_cache)} value "
                f"entries, the model has {NUM_LAYERS} layers"
            )
        collected = []
        expected = (1, KV_HEADS, EXPECTED_POSITIONS, HEAD_DIM)
        for layer in range(NUM_LAYERS):
            entry = {}
            for name, tensor in (("k_rope", cache.key_cache[layer]), ("v_heads", cache.value_cache[layer])):
                if tuple(tensor.shape) != expected:
                    fail(f"layer {layer} {name} has shape {tuple(tensor.shape)}, expected {expected}")
                entry[name] = np.ascontiguousarray(tensor[0].numpy(), dtype=np.float32)
            collected.append(entry)
        self.cache = collected


def evaluate(model, forced_tokens=None, attribution=None):
    """Run prefill plus the decode budget, returning one float32 logit row per position.

    `forced_tokens` teacher-forces the decode input on an already-established
    sequence. The float64 passes use it so that every position compares the same
    inputs; without it a single argmax flip would make every later position a
    comparison of two different computations, and the reported deviation would
    describe divergent sequences rather than reordering. Each pass still records
    its own argmax at every position, so a flip stays visible rather than being
    hidden by the forcing.

    `attribution` is an `AttributionCollector` already attached to `model`; it
    is given the position span of each pass so the per-layer tensors it observes
    can be placed in the row's own position numbering.

    Returns (rows, generated, terminated_on) where `rows` is a list of
    (stage, float32 numpy array) and `generated` holds the tokens this pass
    selected greedily -- 8 of them under the budget, fewer only on EOS.
    """
    import numpy as np
    import torch
    from transformers import DynamicCache

    rows = []
    generated = []

    with torch.inference_mode():
        cache = DynamicCache()
        input_ids = torch.tensor([PROMPT_IDS], dtype=torch.long)
        if attribution is not None:
            attribution.begin_pass("prefill", range(len(PROMPT_IDS)))
        output = model(
            input_ids=input_ids,
            attention_mask=torch.ones_like(input_ids),
            past_key_values=cache,
            use_cache=True,
            logits_to_keep=0,
        )
        prefill_logits = output.logits[0]
        if tuple(prefill_logits.shape) != (len(PROMPT_IDS), VOCAB_SIZE):
            fail(f"prefill produced logits of shape {tuple(prefill_logits.shape)}")
        for index in range(len(PROMPT_IDS)):
            rows.append(("prefill", prefill_logits[index].to(torch.float32).numpy().copy()))

        generated.append(greedy_token(prefill_logits[-1]))
        terminated_on = "eos" if generated[0] == EOS_TOKEN_ID else None

        step = 0
        while terminated_on is None and step < DECODE_STEPS:
            fed = forced_tokens[step] if forced_tokens is not None else generated[step]
            input_ids = torch.tensor([[fed]], dtype=torch.long)
            if attribution is not None:
                attribution.begin_pass("decode", [len(PROMPT_IDS) + step])
            output = model(
                input_ids=input_ids,
                attention_mask=torch.ones((1, len(PROMPT_IDS) + step + 1), dtype=torch.long),
                past_key_values=cache,
                use_cache=True,
                logits_to_keep=0,
            )
            step_logits = output.logits[0]
            if tuple(step_logits.shape) != (1, VOCAB_SIZE):
                fail(f"decode pass {step + 1} produced logits of shape {tuple(step_logits.shape)}")
            rows.append(("decode", step_logits[0].to(torch.float32).numpy().copy()))

            candidate = greedy_token(step_logits[0])
            # The final pass's argmax is computed and retained per position but
            # is not appended: appending it would spend a ninth decode step the
            # profile's budget does not have.
            if step + 1 < DECODE_STEPS:
                generated.append(candidate)
                if candidate == EOS_TOKEN_ID:
                    terminated_on = "eos"
            step += 1

        if terminated_on is None:
            terminated_on = "budget"

        if attribution is not None:
            attribution.read_cache(cache)

    assert all(isinstance(row, np.ndarray) for _, row in rows)
    return rows, generated, terminated_on


def analyse(rows, envelope_rows, logit_dir: Path):
    """Turn raw logit rows into the retained per-position, top-32 and envelope records."""
    import numpy as np

    logit_dir.mkdir(parents=True, exist_ok=True)

    def monotone(bits):
        """Map IEEE-754 binary32 bit patterns to a monotone integer for ULP distance."""
        signed = bits.astype(np.int64)
        return np.where(signed & 0x80000000, 0x80000000 - (signed & 0x7FFFFFFF), signed + 0x80000000)

    def descending_order(values):
        """Rank by descending logit, ties broken toward the lower vocabulary index."""
        return np.lexsort((np.arange(VOCAB_SIZE), -values))

    positions = []
    top32 = []
    envelope = []

    baseline = [np.ascontiguousarray(row, dtype=np.float32) for _, row in rows]

    for index, ((stage, _), values) in enumerate(zip(rows, baseline)):
        raw = values.astype("<f4").tobytes()
        if len(raw) != VOCAB_SIZE * 4:
            fail(f"position {index} serialized to {len(raw)} bytes")
        (logit_dir / f"position-{index:02d}.f32le.bin").write_bytes(raw)

        order = descending_order(values)
        best = int(order[0])
        runner = int(order[1])
        bits = values.view(np.uint32)
        gap = float(values[best]) - float(values[runner])

        positions.append(
            {
                "position": index,
                "stage": stage,
                "logits_sha256": hashlib.sha256(raw).hexdigest(),
                "greedy_token": best,
                "greedy_logit_hex": f"0x{bits[best]:08x}",
                "greedy_logit": repr(float(values[best])),
                "runner_up_token": runner,
                "runner_up_logit_hex": f"0x{bits[runner]:08x}",
                "runner_up_logit": repr(float(values[runner])),
                "runner_up_gap": repr(gap),
                "max_attaining_indices": int(np.count_nonzero(values == values[best])),
                "top_two_bit_identical": str(bool(bits[best] == bits[runner])).lower(),
            }
        )

        for rank in range(TOP_K):
            token = int(order[rank])
            top32.append(
                {
                    "position": index,
                    "rank": rank,
                    "token_id": token,
                    "logit_hex": f"0x{bits[token]:08x}",
                    "logit": repr(float(values[token])),
                }
            )

    for label in sorted(envelope_rows):
        compared_rows = [np.ascontiguousarray(row, dtype=np.float32) for _, row in envelope_rows[label]]
        if len(compared_rows) != len(baseline):
            fail(f"{label} produced {len(compared_rows)} positions, the baseline has {len(baseline)}")
        for index, (reference, compared) in enumerate(zip(baseline, compared_rows)):
            wide_reference = reference.astype(np.float64)
            wide_compared = compared.astype(np.float64)
            difference = np.abs(wide_reference - wide_compared)
            ulp = np.abs(monotone(reference.view(np.uint32)) - monotone(compared.view(np.uint32)))
            scale = np.maximum(np.abs(wide_reference), np.abs(wide_compared))
            relative = np.where(scale > 0.0, difference / np.where(scale > 0.0, scale, 1.0), 0.0)
            top_order = descending_order(reference)[:TOP_K]

            envelope.append(
                {
                    "position": index,
                    "variant": label,
                    "bit_identical_logits": int(np.count_nonzero(difference == 0.0)),
                    "max_abs_deviation": repr(float(difference.max())),
                    "max_ulp_deviation": int(ulp.max()),
                    "max_rel_deviation": repr(float(relative.max())),
                    "top32_max_abs_deviation": repr(float(difference[top_order].max())),
                    "top32_max_ulp_deviation": int(ulp[top_order].max()),
                    "greedy_token_agrees": str(
                        int(descending_order(reference)[0]) == int(descending_order(compared)[0])
                    ).lower(),
                }
            )

    return positions, top32, envelope


def analyse_attribution(attribution, sequence_ids, attribution_dir: Path):
    """Turn the observed intermediates into the retained attribution records.

    The full bytes are regenerable local data, exactly as the logits are; what
    is retained is a digest per tensor slice plus the reference's own top
    entries by magnitude, because a digest proves the reference regenerates
    exactly and cannot support a bounded-error comparison, which needs values.
    Both, for the same reason L1 states for the logits.

    The slice is the unit of attribution: a (layer, position) pair names one of
    the twenty-eight layer executions and the pass that produced the position,
    which is the resolution the model boundary lacks.
    """
    import numpy as np

    hidden_dir = attribution_dir / "hidden"
    cache_dir = attribution_dir / "cache"
    hidden_dir.mkdir(parents=True, exist_ok=True)
    cache_dir.mkdir(parents=True, exist_ok=True)

    covered = [position for span in attribution.pass_positions for position in span]
    if covered != list(range(EXPECTED_POSITIONS)):
        fail(
            f"the observed passes cover positions {covered}, not the row's "
            f"0..{EXPECTED_POSITIONS - 1}"
        )
    pass_count = len(attribution.pass_positions)
    stage_of = ["prefill"] * len(PROMPT_IDS) + ["decode"] * DECODE_STEPS

    def descending_magnitude(values):
        """Rank by descending |value|, ties broken toward the lower flat index.

        Magnitude rather than L1's signed order because a hidden state has no
        ranking semantics to preserve; what a bounded comparison wants is the
        coordinates where an absolute deviation would be largest. The retained
        coordinates are the *reference's*, so a candidate is indexed at them
        rather than re-ranked -- the same discipline L1's envelope uses when it
        restricts a deviation to the reference's top-32 order.
        """
        flat = values.ravel()
        return np.lexsort((np.arange(flat.size), -np.abs(flat.astype(np.float64))))

    # --- per-layer hidden states -------------------------------------------
    hidden_rows = []
    hidden_top = []
    hidden_bytes = 0
    for layer in range(NUM_LAYERS):
        parts = attribution.hidden[layer]
        if len(parts) != pass_count:
            fail(f"layer {layer} was observed on {len(parts)} passes, the row has {pass_count}")
        tensor = np.ascontiguousarray(np.concatenate(parts, axis=0), dtype=np.float32)
        if tensor.shape != (EXPECTED_POSITIONS, HIDDEN_SIZE):
            fail(f"layer {layer} assembled to shape {tensor.shape}, expected ({EXPECTED_POSITIONS}, {HIDDEN_SIZE})")
        raw = tensor.astype("<f4").tobytes()
        (hidden_dir / f"layer-{layer:02d}.f32le.bin").write_bytes(raw)
        hidden_bytes += len(raw)

        for position in range(EXPECTED_POSITIONS):
            values = tensor[position]
            start = position * POSITION_SLICE_BYTES
            slice_raw = raw[start : start + POSITION_SLICE_BYTES]
            bits = values.view(np.uint32)
            order = descending_magnitude(values)
            best = int(order[0])
            hidden_rows.append(
                {
                    "layer": layer,
                    "position": position,
                    "stage": stage_of[position],
                    "sha256": hashlib.sha256(slice_raw).hexdigest(),
                    "l2_norm": repr(exact_norm(values)),
                    "max_abs_lane": best,
                    "max_abs_hex": f"0x{bits[best]:08x}",
                    "max_abs": repr(float(values[best])),
                }
            )
            for rank in range(ATTRIBUTION_TOP_K):
                lane = int(order[rank])
                hidden_top.append(
                    {
                        "layer": layer,
                        "position": position,
                        "rank": rank,
                        "lane": lane,
                        "value_hex": f"0x{bits[lane]:08x}",
                        "value": repr(float(values[lane])),
                    }
                )
    if hidden_bytes != HIDDEN_BYTES:
        fail(f"the hidden states wrote {hidden_bytes} bytes, the L6 figure is {HIDDEN_BYTES}")

    # --- per-layer post-RoPE K and V ---------------------------------------
    cache_rows = []
    cache_top = []
    cache_bytes = 0
    for layer in range(NUM_LAYERS):
        for name in CACHE_TENSORS:
            tensor = attribution.cache[layer][name]
            if tensor.shape != (KV_HEADS, EXPECTED_POSITIONS, HEAD_DIM):
                fail(f"layer {layer} {name} has shape {tensor.shape}")
            raw = tensor.astype("<f4").tobytes()
            (cache_dir / f"layer-{layer:02d}-{name}.f32le.bin").write_bytes(raw)
            cache_bytes += len(raw)

            for position in range(EXPECTED_POSITIONS):
                # The file keeps L4's and L5's declared `[8, S, 128]` head-major
                # layout, so one position is a strided gather rather than a byte
                # range. The digest is over that gather made contiguous, which is
                # what a reader must reproduce.
                values = np.ascontiguousarray(tensor[:, position, :])
                slice_raw = values.astype("<f4").tobytes()
                if len(slice_raw) != POSITION_SLICE_BYTES:
                    fail(f"layer {layer} {name} position {position} serialized to {len(slice_raw)} bytes")
                bits = values.view(np.uint32)
                order = descending_magnitude(values)
                best = int(order[0])
                cache_rows.append(
                    {
                        "layer": layer,
                        "tensor": name,
                        "position": position,
                        "stage": stage_of[position],
                        "sha256": hashlib.sha256(slice_raw).hexdigest(),
                        "l2_norm": repr(exact_norm(values)),
                        "max_abs_head": best // HEAD_DIM,
                        "max_abs_lane": best % HEAD_DIM,
                        "max_abs_hex": f"0x{bits.ravel()[best]:08x}",
                        "max_abs": repr(float(values.ravel()[best])),
                    }
                )
                for rank in range(ATTRIBUTION_TOP_K):
                    flat = int(order[rank])
                    cache_top.append(
                        {
                            "layer": layer,
                            "tensor": name,
                            "position": position,
                            "rank": rank,
                            "head": flat // HEAD_DIM,
                            "lane": flat % HEAD_DIM,
                            "value_hex": f"0x{bits.ravel()[flat]:08x}",
                            "value": repr(float(values.ravel()[flat])),
                        }
                    )
    if cache_bytes != CACHE_BYTES:
        fail(f"the cache tensors wrote {cache_bytes} bytes, the L6 figure is {CACHE_BYTES}")

    # --- host computation 1: the rotary rows -------------------------------
    # Small enough to retain in full, which is what L6 asks of the four host
    # computations: each is checkable without owning the checkpoint.
    tables = {}
    for index, name in ((0, "cos"), (1, "sin")):
        table = np.ascontiguousarray(
            np.concatenate([pair[index] for pair in attribution.rotary], axis=0), dtype=np.float32
        )
        if table.shape != (EXPECTED_POSITIONS, HEAD_DIM):
            fail(f"the rotary {name} table assembled to shape {table.shape}")
        tables[name] = table
    rotary_rows = []
    for position in range(EXPECTED_POSITIONS):
        cos_bits = tables["cos"][position].view(np.uint32)
        sin_bits = tables["sin"][position].view(np.uint32)
        for lane in range(HEAD_DIM):
            rotary_rows.append(
                {
                    "position": position,
                    "lane": lane,
                    "cos_hex": f"0x{cos_bits[lane]:08x}",
                    "cos": repr(float(tables["cos"][position][lane])),
                    "sin_hex": f"0x{sin_bits[lane]:08x}",
                    "sin": repr(float(tables["sin"][position][lane])),
                }
            )

    # --- host computation 2: the additive causal mask ----------------------
    mask_rows = []
    mask_records = []
    admitted = {MASK_MASKED_ENTRY, MASK_ATTENDED_ENTRY}
    for index, (stage, mask) in enumerate(attribution.masks):
        label = "prefill" if index == 0 else f"decode-{index}"
        span = attribution.pass_positions[index]
        if mask.shape[0] != len(span):
            fail(f"the {label} mask has {mask.shape[0]} query rows, the pass covers {len(span)} positions")
        if mask.shape[1] != span[-1] + 1:
            fail(f"the {label} mask spans {mask.shape[1]} keys, the pass reaches context {span[-1] + 1}")
        bits = mask.view(np.uint32)
        unexpected = sorted({int(value) for value in np.unique(bits)} - admitted)
        if unexpected:
            fail(
                f"the {label} mask carries bit patterns {[f'0x{value:08x}' for value in unexpected]}; "
                f"L4 fixes the mask's two values as 0x{MASK_MASKED_ENTRY:08x} and "
                f"0x{MASK_ATTENDED_ENTRY:08x}"
            )
        raw = mask.astype("<f4").tobytes()
        mask_records.append((label, stage, mask, raw))
        for row, query in enumerate(span):
            for key in range(mask.shape[1]):
                mask_rows.append(
                    {
                        "pass": label,
                        "stage": stage,
                        "query_position": query,
                        "key_position": key,
                        "value_hex": f"0x{bits[row, key]:08x}",
                    }
                )

    # --- host computation 3: the token IDs ---------------------------------
    # Already retained as `sequence.tsv`; the digest is over one declared
    # serialization of them so the same evidence can be compared as bytes.
    token_bytes = np.asarray(sequence_ids, dtype="<i4").tobytes()
    if len(token_bytes) != EXPECTED_POSITIONS * 4:
        fail(f"the token IDs serialized to {len(token_bytes)} bytes")

    host = [
        ("host.rotary.shape", f"[{EXPECTED_POSITIONS}, {HEAD_DIM}]"),
        ("host.rotary.cos_bytes", str(tables["cos"].nbytes)),
        ("host.rotary.cos_sha256", hashlib.sha256(tables["cos"].astype("<f4").tobytes()).hexdigest()),
        ("host.rotary.sin_bytes", str(tables["sin"].nbytes)),
        ("host.rotary.sin_sha256", hashlib.sha256(tables["sin"].astype("<f4").tobytes()).hexdigest()),
        ("host.rotary.retained", "in full, one row per (position, lane), in rotary.tsv"),
        ("host.mask.masked_entry", f"0x{MASK_MASKED_ENTRY:08x}"),
        ("host.mask.attended_entry", f"0x{MASK_ATTENDED_ENTRY:08x}"),
        ("host.mask.passes", str(len(mask_records))),
        ("host.mask.retained", "in full, one row per (pass, query, key), in mask.tsv"),
    ]
    for label, _, mask, raw in mask_records:
        host.append((f"host.mask.{label}.shape", f"[{mask.shape[0]}, {mask.shape[1]}]"))
        host.append((f"host.mask.{label}.bytes", str(len(raw))))
        host.append((f"host.mask.{label}.sha256", hashlib.sha256(raw).hexdigest()))
        attended = int(np.count_nonzero(mask.view(np.uint32) == MASK_ATTENDED_ENTRY))
        host.append((f"host.mask.{label}.attended_entries", str(attended)))
    host.extend(
        [
            ("host.tokens.count", str(len(sequence_ids))),
            ("host.tokens.serialization", "little-endian int32, C-contiguous, in sequence order"),
            ("host.tokens.bytes", str(len(token_bytes))),
            ("host.tokens.sha256", hashlib.sha256(token_bytes).hexdigest()),
        ]
    )

    return hidden_rows, hidden_top, cache_rows, cache_top, rotary_rows, mask_rows, host


def digest_widened_weights(model, snapshot: Path):
    """Digest the F32 weights the F32 pass actually used, against the stored BF16.

    This is L6's fourth host computation. The widening happens once at model
    load rather than inside any program, and its evidence is one digest over the
    widened bytes -- so the digest is taken over the *loaded parameters*, not
    over a re-derivation of them, and the checkpoint's own BF16 tensor is
    re-widened beside each one to check the conversion is bit-exact rather than
    merely believed to be.

    The canonical stream absorbs, for each of the checkpoint's tensors in
    lexicographic name order, the name's UTF-8 bytes, one NUL, and the tensor's
    little-endian C-contiguous F32 bytes. Per-tensor digests are deliberately
    not retained here: a total, injective map from checkpoint tensor name to
    interface key is `define-the-model-weight-binding-manifest`'s subject, and a
    second naming authority for one subject is what that ticket exists to avoid.
    """
    import numpy as np
    import torch
    from safetensors import safe_open

    if sys.byteorder != "little":
        fail(f"this host is {sys.byteorder}-endian; the retained byte order is little-endian")

    state = model.state_dict()
    aggregate = hashlib.sha256()
    total = 0
    exact = 0

    with safe_open(str(snapshot / "model.safetensors"), framework="pt") as handle:
        names = sorted(handle.keys())
        if len(names) != WEIGHT_TENSOR_COUNT:
            fail(f"the checkpoint declares {len(names)} tensors, L1's inventory records {WEIGHT_TENSOR_COUNT}")
        for name in names:
            if name not in state:
                fail(f"the loaded model has no parameter named {name}")
            widened = state[name].detach().contiguous()
            if widened.dtype != torch.float32:
                fail(f"{name} loaded as {widened.dtype}, not float32")
            stored = handle.get_tensor(name)
            if stored.dtype != torch.bfloat16:
                fail(f"{name} is stored as {stored.dtype}, the profile records BF16 for every tensor")
            if tuple(stored.shape) != tuple(widened.shape):
                fail(f"{name} is {tuple(stored.shape)} stored and {tuple(widened.shape)} loaded")
            if torch.equal(stored.to(torch.float32).contiguous().view(torch.int32), widened.view(torch.int32)):
                exact += 1
            array = widened.numpy()
            if array.dtype.byteorder not in ("=", "<") or array.dtype.itemsize != 4:
                fail(f"{name} presented as {array.dtype}, not native little-endian F32")
            aggregate.update(name.encode("utf-8"))
            aggregate.update(b"\0")
            aggregate.update(memoryview(array).cast("B"))
            total += array.nbytes
            del stored, widened, array

    if total != WIDENED_WEIGHT_BYTES:
        fail(f"the widened weights are {total} bytes, L1's F32 weight budget is {WIDENED_WEIGHT_BYTES}")

    return [
        ("weights.widened.tensor_count", str(len(names))),
        ("weights.widened.bytes", str(total)),
        (
            "weights.widened.absorption",
            "per tensor in lexicographic name order: name UTF-8, one NUL, little-endian "
            "C-contiguous F32 bytes",
        ),
        ("weights.widened.sha256", aggregate.hexdigest()),
        ("weights.widening_bit_exact_tensors", str(exact)),
    ]


def write_tsv(path: Path, rows, columns) -> None:
    lines = ["\t".join(columns)]
    lines.extend("\t".join(str(row[column]) for column in columns) for row in rows)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_kv_tsv(path: Path, rows) -> None:
    lines = ["key\tvalue"]
    lines.extend(f"{key}\t{value}" for key, value in rows)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def produce(out_dir: Path, logit_dir: Path, attribution_dir: Path) -> None:
    import numpy
    import torch
    import transformers
    from transformers.models.qwen3 import modeling_qwen3

    # One thread removes intra-op reduction-order variation as a source of
    # digest drift between two runs on this host. It is a determinism control,
    # not a performance choice; the record states the thread count so a reader
    # knows which realization the digests describe.
    torch.set_num_threads(1)
    torch.set_grad_enabled(False)

    snapshot, checkpoint_records = acquire_and_verify_checkpoint()
    reference_records = verify_reference_sources()
    tokenizer_records = verify_prompt_tokenization(snapshot)

    model = load_model(snapshot, torch.float32)
    attribution = AttributionCollector()
    attribution.attach(model)
    try:
        rows, generated, terminated_on = evaluate(model, attribution=attribution)
    finally:
        attribution.detach()
    weight_records = digest_widened_weights(model, snapshot)
    del model
    gc.collect()

    if terminated_on not in ("eos", "budget"):
        fail(f"the run terminated on {terminated_on!r}, which is neither EOS nor the budget")
    if terminated_on == "budget" and len(rows) != EXPECTED_POSITIONS:
        fail(f"the F32 pass retained {len(rows)} positions, the profile's C1 row is {EXPECTED_POSITIONS}")
    if terminated_on == "budget" and len(generated) != DECODE_STEPS:
        fail(f"the F32 pass generated {len(generated)} tokens under an {DECODE_STEPS}-step budget")

    envelope_rows = {}

    model = load_model(snapshot, torch.float64)
    envelope_rows["f64_unmodified"] = evaluate(model, forced_tokens=generated)[0]
    del model
    gc.collect()

    saved = {
        "rms": modeling_qwen3.Qwen3RMSNorm.forward,
        "attn": modeling_qwen3.eager_attention_forward,
        "rope": modeling_qwen3.Qwen3RotaryEmbedding.forward,
    }
    try:
        promote_reference_to_float64(modeling_qwen3)
        model = load_model(snapshot, torch.float64)
        envelope_rows["f64_promoted"] = evaluate(model, forced_tokens=generated)[0]
        del model
        gc.collect()
    finally:
        modeling_qwen3.Qwen3RMSNorm.forward = saved["rms"]
        modeling_qwen3.eager_attention_forward = saved["attn"]
        modeling_qwen3.Qwen3RotaryEmbedding.forward = saved["rope"]

    positions, top32, envelope = analyse(rows, envelope_rows, logit_dir)

    sequence = [
        {"index": index, "role": "prompt", "token_id": token} for index, token in enumerate(PROMPT_IDS)
    ]
    sequence.extend(
        {"index": len(PROMPT_IDS) + offset, "role": "generated", "token_id": token}
        for offset, token in enumerate(generated)
    )

    hidden, hidden_top, cache, cache_top, rotary, mask, host = analyse_attribution(
        attribution, [row["token_id"] for row in sequence], attribution_dir
    )
    host.extend(weight_records)

    environment = [
        ("workload.repo_id", REPO_ID),
        ("workload.revision", REVISION),
        ("workload.prompt_text", PROMPT_TEXT),
        ("workload.prompt_token_ids", json.dumps(PROMPT_IDS, separators=(",", ":"))),
        ("workload.prompt_length", str(len(PROMPT_IDS))),
        ("workload.decode_budget", str(DECODE_STEPS)),
        ("workload.eos_token_id", str(EOS_TOKEN_ID)),
        ("workload.vocab_size", str(VOCAB_SIZE)),
        ("workload.retained_positions", str(len(rows))),
        ("workload.sequence_length", str(len(sequence))),
        ("run.dtype", "float32"),
        ("run.device", "cpu"),
        ("run.attn_implementation", "eager"),
        ("run.logits_to_keep", "0"),
        ("run.decode_strategy", "greedy, lowest vocabulary index among maxima"),
        ("run.terminated_on", terminated_on),
        ("run.generated_token_count", str(len(generated))),
        ("run.torch_num_threads", "1"),
        ("run.logit_byte_order", "little-endian IEEE-754 binary32, C-contiguous"),
        ("envelope.variants", "f64_unmodified,f64_promoted"),
        ("envelope.rounding_point", "logits cast to float32 at the observable"),
        ("envelope.decode_input", "teacher-forced on the F32 pass token sequence"),
        ("attribution.pass", "the F32 pass only; the float64 passes carry no hooks"),
        ("attribution.observation", "forward hooks on each Qwen3DecoderLayer and on Qwen3RotaryEmbedding"),
        ("attribution.layers", str(NUM_LAYERS)),
        ("attribution.hidden_size", str(HIDDEN_SIZE)),
        ("attribution.kv_heads", str(KV_HEADS)),
        ("attribution.head_dim", str(HEAD_DIM)),
        ("attribution.top_k", str(ATTRIBUTION_TOP_K)),
        ("attribution.top_order", "descending |value|, ties toward the lower flat index"),
        ("attribution.hidden_shape", f"[{EXPECTED_POSITIONS}, {HIDDEN_SIZE}] per layer, position-major"),
        ("attribution.hidden_bytes", str(HIDDEN_BYTES)),
        (
            "attribution.hidden_digest_unit",
            f"position p of layer L, bytes [p*{POSITION_SLICE_BYTES}, (p+1)*{POSITION_SLICE_BYTES}) "
            "of hidden/layer-LL.f32le.bin",
        ),
        ("attribution.cache_shape", f"[{KV_HEADS}, {EXPECTED_POSITIONS}, {HEAD_DIM}] per layer and tensor, head-major"),
        ("attribution.cache_bytes", str(CACHE_BYTES)),
        (
            "attribution.cache_digest_unit",
            "position p of layer L, the contiguous re-serialization of tensor[:, p, :] "
            f"({KV_HEADS} x {HEAD_DIM} F32 values) -- a strided gather, not a byte range",
        ),
        ("attribution.byte_order", "little-endian IEEE-754 binary32, C-contiguous"),
        ("attribution.norm", "exactly-rounded (math.fsum over exact float64 squares), so it is order-independent"),
        ("attribution.tolerance", "none; design-model-level-qualification-and-optimization owns the bound"),
        ("version.python", platform.python_version()),
        ("version.torch", torch.__version__),
        ("version.transformers", transformers.__version__),
        ("version.numpy", numpy.__version__),
    ]
    environment.extend(tokenizer_records)
    environment.extend(host_row())
    environment.extend(checkpoint_records)
    environment.extend(reference_records)

    out_dir.mkdir(parents=True, exist_ok=True)
    write_kv_tsv(out_dir / "environment.tsv", environment)
    write_tsv(out_dir / "sequence.tsv", sequence, ["index", "role", "token_id"])
    write_tsv(
        out_dir / "positions.tsv",
        positions,
        [
            "position",
            "stage",
            "logits_sha256",
            "greedy_token",
            "greedy_logit_hex",
            "greedy_logit",
            "runner_up_token",
            "runner_up_logit_hex",
            "runner_up_logit",
            "runner_up_gap",
            "max_attaining_indices",
            "top_two_bit_identical",
        ],
    )
    write_tsv(out_dir / "top32.tsv", top32, ["position", "rank", "token_id", "logit_hex", "logit"])
    write_tsv(
        out_dir / "envelope.tsv",
        envelope,
        [
            "position",
            "variant",
            "bit_identical_logits",
            "max_abs_deviation",
            "max_ulp_deviation",
            "max_rel_deviation",
            "top32_max_abs_deviation",
            "top32_max_ulp_deviation",
            "greedy_token_agrees",
        ],
    )
    write_tsv(
        out_dir / "hidden.tsv",
        hidden,
        [
            "layer",
            "position",
            "stage",
            "sha256",
            "l2_norm",
            "max_abs_lane",
            "max_abs_hex",
            "max_abs",
        ],
    )
    write_tsv(out_dir / "hidden_top.tsv", hidden_top, ["layer", "position", "rank", "lane", "value_hex", "value"])
    write_tsv(
        out_dir / "cache.tsv",
        cache,
        [
            "layer",
            "tensor",
            "position",
            "stage",
            "sha256",
            "l2_norm",
            "max_abs_head",
            "max_abs_lane",
            "max_abs_hex",
            "max_abs",
        ],
    )
    write_tsv(
        out_dir / "cache_top.tsv",
        cache_top,
        ["layer", "tensor", "position", "rank", "head", "lane", "value_hex", "value"],
    )
    write_tsv(out_dir / "rotary.tsv", rotary, ["position", "lane", "cos_hex", "cos", "sin_hex", "sin"])
    write_tsv(out_dir / "mask.tsv", mask, ["pass", "stage", "query_position", "key_position", "value_hex"])
    write_kv_tsv(out_dir / "host.tsv", host)

    # The manifest hashes the retained records and the producer's own inputs, so
    # a later reader can tell an edited fixture from a regenerated one without
    # owning a model. `verify_fixture.py` is what checks it.
    manifest = [(f"result.sha256.{name}", sha256_file(out_dir / name)) for name in RESULT_FILES]
    root = Path(__file__).resolve().parent
    for relative in ("produce_fixture.py", "verify_fixture.py", "pyproject.toml", "uv.lock"):
        manifest.append((f"producer.sha256.{relative}", sha256_file(root / relative)))
    write_kv_tsv(out_dir / "manifest.tsv", manifest)


def compare_directories(produced: Path, retained: Path) -> int:
    differences = 0
    for name in RESULT_FILES + ["manifest.tsv"]:
        retained_path = retained / name
        if not retained_path.exists():
            print(f"MISSING from {retained}: {name}")
            differences += 1
            continue
        if (produced / name).read_bytes() != retained_path.read_bytes():
            print(f"DIFFERS: {name}")
            differences += 1
        else:
            print(f"identical: {name}")
    return differences


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Produce the C1 conformance and attribution reference fixture."
    )
    parser.add_argument("--out", type=Path, help="directory to write the retained records into")
    parser.add_argument(
        "--compare",
        type=Path,
        help="re-run production into a scratch directory and byte-compare against this one",
    )
    parser.add_argument(
        "--logit-dir",
        type=Path,
        default=Path(__file__).resolve().parent / "local-work" / "logits",
        help="where the complete F32 logit bytes go; regenerable local data, not version controlled",
    )
    parser.add_argument(
        "--attribution-dir",
        type=Path,
        default=Path(__file__).resolve().parent / "local-work" / "attribution",
        help="where the complete hidden-state and cache bytes go; regenerable local data",
    )
    arguments = parser.parse_args()

    if arguments.compare is not None:
        scratch = Path(tempfile.mkdtemp(prefix="qwen3-fixture-compare-"))
        try:
            produce(scratch, arguments.logit_dir, arguments.attribution_dir)
            differences = compare_directories(scratch, arguments.compare)
            if differences:
                print(f"\n{differences} retained file(s) differ from {arguments.compare}")
                return 5
            print(f"\nall {len(RESULT_FILES) + 1} retained files reproduced byte-for-byte")
            return 0
        finally:
            shutil.rmtree(scratch, ignore_errors=True)

    if arguments.out is None:
        fail("--out is required when not comparing")
    produce(arguments.out, arguments.logit_dir, arguments.attribution_dir)
    print(f"retained records written to {arguments.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
