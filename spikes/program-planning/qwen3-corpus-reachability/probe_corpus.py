#!/usr/bin/env python3
"""Which model-level conformance rows can the pinned Qwen3-0.6B-Base reach?

Three rows of the adversarial corpus turn on a reachability question this
checkpoint can answer without any Tiler execution, and each of the three would
otherwise be settled by an assumption. The probe answers all three from the same
verified bytes, so a reader gets one environment row and one manifest rather
than three probes that could have run against different files.

**Q1 -- the tie branch.** The C1 conformance row leaves it unexercised. The
retained conformance fixture measures that at all 18 positions exactly one index
attains the maximum and no top-two pair is bit-identical -- a fact about one
prompt, not evidence that the branch is unreachable. The corpus therefore needs
either a demonstrating row or a recorded negative whose search is stated.

Two routes reach a tie, and they are kept apart because only one of them is
cheap to close.

  * **Coincidence.** Two distinct vocabulary entries whose F32 logits happen to
    round to the same bit pattern at the maximum. Nothing makes it likely: it
    needs the top-two gap to be exactly zero in F32.

  * **Structure.** Two vocabulary entries whose *embedding rows are
    bit-identical*. The checkpoint declares `tie_word_embeddings: true` and
    carries no `lm_head.weight`, so one `[151936, 1024]` matrix is both the
    gather source and the vocabulary projection's weight. Two bit-identical
    rows of it are two bit-identical *columns* of that projection, so their
    logits are the same contraction over the same operand sequence at every
    position of every prompt. If the maximum is ever attained by a member of
    such a group, every other member attains it too and the tie is a property
    of the checkpoint rather than of the prompt.

**Q2 -- a subnormal weight.** The corpus records a subnormal-weight row as
deliberately absent, and the ground it inherited is that "a BF16 subnormal
widens to an F32 normal, so the target's flush cannot touch it". That ground is
checkable exhaustively rather than by argument: stage 0 widens all 65,536 BF16
bit patterns and classifies each result, which is a finite population and
therefore exhaustive evidence rather than a sample.

**Q3 -- a NaN or infinite weight.** The same scan counts the checkpoint's own
non-finite stored values, so "the ingestion ticket's one-line check has nothing
to catch on this checkpoint" is a counted statement rather than an expectation.

Q2 and Q3 share one pass over all 310 stored tensors, classifying every stored
BF16 element by its exponent and significand fields.

Stage 0 and stage 1 need no model and no torch: they read the safetensors header
and the stored bytes directly, so the population is the file's own declared
element count rather than whatever a loader chose to materialize.

Stage 2 evaluates prompts through the pinned reference in F32 on CPU and
records, per position, the tie observable the oracle would report *and* how far
the best-placed duplicate-group member is from the maximum -- its logit, its
rank, and its gap. A negative that reports only "no tie found" says nothing
about how close the search came; the rank and the gap are what let a later
reader decide whether a larger search is worth running.

Run it from this directory. No `make` target reaches a spike:

    UV_PROJECT_ENVIRONMENT=local-work/venv uv run --locked python probe_corpus.py \
      --out results/<slug>

`--structural-only` runs stages 0 and 1 alone, which need neither torch nor the
RAM an F32 forward pass wants.
"""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import math
import platform
import struct
import subprocess
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Transcribed from the workload profile, exactly as the conformance fixture
# transcribes them. A mismatch is a stop and not a warning: a run against other
# bytes is not a weaker probe, it is a probe of a different checkpoint.
# ---------------------------------------------------------------------------

REPO_ID = "Qwen/Qwen3-0.6B-Base"
REVISION = "da87bfb608c14b7cf20ba1ce41287e8de496c0cd"

CHECKPOINT_MANIFEST = [
    ("config.json", 727, "504a6b58c4271583724e66584b6b7698aea18450209df6b2f7582df0e89cee59"),
    ("model.safetensors", 1192135096, "cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba"),
    ("tokenizer.json", 7031645, "c0382117ea329cdf097041132f6d735924b697924d6f6fc3945713e96ce87539"),
    ("vocab.json", 2776833, "ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910"),
]

REFERENCE_SOURCE_MANIFEST = [
    ("models/qwen3/modeling_qwen3.py", "704c914530530a1acb0b443add1f520404e3ac2c28c0ab7e16f80f86cfe8ccb2"),
    ("models/qwen3/configuration_qwen3.py", "87f0d17326c44f2dfe1bfc329faf9201ab4b19a89ad555da085b4cc81461b201"),
    ("modeling_rope_utils.py", "c28b3e88edca8fdb5497e5c36091bf753db49bd94ace33a84e9f9c61cbf66032"),
]

EMBEDDING_TENSOR = "model.embed_tokens.weight"
VOCAB_SIZE = 151936
HIDDEN_SIZE = 1024
STORED_DTYPE = "BF16"

# ---------------------------------------------------------------------------
# The control, and why it is the fixture's own numbers rather than a re-run.
#
# The C1 prompt's first ten positions come from one prefill pass, which is
# exactly what this probe evaluates. The conformance fixture retains the greedy
# token, the top-two bit patterns, and the runner-up gap for each of them, so
# transcribing those and demanding equality is a positive control that says
# "this probe evaluated the same reference the fixture did" -- and it can fail,
# which a probe that only re-ran the prompt and compared against itself cannot.
#
# It doubles as the negative control for the tie observable: the fixture
# measures no bit-identical top-two pair at any of these positions, so a probe
# reporting one here is reporting a defect in itself.
# ---------------------------------------------------------------------------

CONTROL_NAME = "c1-control"
CONTROL_IDS = [785, 3974, 13876, 38835, 34208, 916, 279, 15678, 5562, 13]

# (position, greedy token, top-1 logit bits, top-2 logit bits, runner-up gap)
CONTROL_EXPECTED = [
    (0, 2701, 0x4187C758, 0x415C2C2D, 3.211550712585449),
    (1, 323, 0x4195842F, 0x419269A4, 0.38796043395996094),
    (2, 38835, 0x41B882E3, 0x418E4787, 5.278984069824219),
    (3, 34208, 0x41B695E6, 0x41A69E90, 1.9957695007324219),
    (4, 916, 0x41B29AB0, 0x418728EF, 5.430543899536133),
    (5, 279, 0x41AD6055, 0x418BAE84, 4.211824417114258),
    (6, 15678, 0x41C8AACC, 0x4189DD24, 7.8504180908203125),
    (7, 5562, 0x41C0771A, 0x4192CF51, 5.706926345825195),
    (8, 13, 0x41979B77, 0x418CCAA8, 1.3519573211669922),
    (9, 576, 0x418C5593, 0x4189E4E4, 0.3050212860107422),
]

# The candidate prompts, each a repetition of one member of a duplicate group.
# The construction is derived rather than searched: the model cannot distinguish
# two tokens whose embedding rows are bit-identical, in either direction, so a
# prompt that drives the model to predict "the token it has just been shown"
# lands the maximum inside a duplicate group whenever it lands on that token at
# all. Repetition is the cheapest such driver on a base checkpoint.
#
# Each entry names the group member fed, the partner the structural argument
# says must tie with it, and the prompt length.
#
# The list spans the group-size range stage 1 measures rather than one end of
# it, because the two ends fail differently: a 505-member group offers 505
# chances for the maximum to land inside it at a position where the model is
# unsure, while a 2-member group is the cleanest demonstrating row if it lands
# at all. Repetition length is varied for the same reason -- an induction driver
# that needs a longer run to fire would otherwise be recorded as a driver that
# does not fire.
CANDIDATE_PROMPTS = [
    ("dup-184-x8", 184, 128477, 8),
    ("dup-184-x16", 184, 128477, 16),
    ("dup-184-x32", 184, 128477, 32),
    ("dup-129214-x8", 129214, 129243, 8),
    ("dup-129214-x16", 129214, 129243, 16),
    ("dup-131430-x16", 131430, 131436, 16),
    ("dup-132379-x16", 132379, 132383, 16),
    ("dup-147711-x16", 147711, 147712, 16),
    ("dup-123806-x16", 123806, 124027, 16),
    ("dup-123806-x32", 123806, 124027, 32),
    ("dup-151555-x16", 151555, 151556, 16),
    ("dup-151554-x16", 151554, 151738, 16),
    ("dup-177-x16", 177, 178, 16),
    ("dup-177-x32", 177, 178, 32),
    ("dup-124-x16", 124, 125, 16),
    ("dup-77150-x16", 77150, 83971, 16),
    ("dup-124630-x16", 124630, 125454, 16),
    ("dup-124458-x16", 124458, 124479, 16),
]


def fail(message: str):
    print(f"PROBE STOP: {message}", file=sys.stderr)
    raise SystemExit(4)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def f32_bits(value: float) -> int:
    return struct.unpack("<I", struct.pack("<f", value))[0]


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
            fail(f"{name} hashed to {actual_digest}, the manifest says {expected_digest}")
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
                f"the installed {relative} hashed to {actual_digest}, the pinned commit's is "
                f"{expected_digest}. The evaluated implementation is not the pinned reference."
            )
        records.append((f"reference.sha256.{relative}", actual_digest))
    return records


# ---------------------------------------------------------------------------
# Stage 0 -- exceptional stored values. No model, no torch.
# ---------------------------------------------------------------------------

# BF16 and F32 share a sign bit and an eight-bit exponent field and differ only
# in significand width -- 7 bits against 23 -- so the widening BF16 -> F32 is
# the zero-extension of the significand and nothing else. The two classifiers
# below read the fields directly rather than through a float comparison, because
# a comparison against `float('inf')` cannot distinguish a subnormal from a
# small normal and that distinction is the whole question.

BF16_EXPONENT_MASK = 0x7F80
BF16_SIGNIFICAND_MASK = 0x007F
F32_EXPONENT_MASK = 0x7F800000
F32_SIGNIFICAND_MASK = 0x007FFFFF


def classify_bf16(bits: int) -> str:
    exponent = bits & BF16_EXPONENT_MASK
    significand = bits & BF16_SIGNIFICAND_MASK
    if exponent == BF16_EXPONENT_MASK:
        return "nan" if significand else "infinite"
    if exponent == 0:
        return "subnormal" if significand else "zero"
    return "normal"


def classify_f32(bits: int) -> str:
    exponent = bits & F32_EXPONENT_MASK
    significand = bits & F32_SIGNIFICAND_MASK
    if exponent == F32_EXPONENT_MASK:
        return "nan" if significand else "infinite"
    if exponent == 0:
        return "subnormal" if significand else "zero"
    return "normal"


def widening_class_map():
    """Widen every BF16 bit pattern and classify the result.

    The population is all 65,536 patterns, so this is exhaustive finite evidence
    about the widening rather than a sample of it: every (stored class, widened
    class) pair the conversion can produce appears here with its exact count.
    """
    counts = collections.Counter()
    for bits in range(1 << 16):
        counts[(classify_bf16(bits), classify_f32(bits << 16))] += 1
    total = sum(counts.values())
    if total != 1 << 16:
        fail(f"the widening map covers {total} patterns, not {1 << 16}")
    return counts


def stored_value_classes(snapshot: Path):
    """Classify every stored element of every tensor the checkpoint declares.

    Counts are per class over the whole file and per tensor for any tensor
    holding a non-normal, non-zero element, so a reader learns *where* an
    exceptional value is and not only that one exists.
    """
    import numpy

    path = snapshot / "model.safetensors"
    with open(path, "rb") as handle:
        (header_len,) = struct.unpack("<Q", handle.read(8))
        header = json.loads(handle.read(header_len))
    data_start = 8 + header_len

    tensors = {name: entry for name, entry in header.items() if name != "__metadata__"}
    for name, entry in tensors.items():
        if entry["dtype"] != STORED_DTYPE:
            fail(f"{name} is stored as {entry['dtype']}, the profile records {STORED_DTYPE} for all 310")

    raw = numpy.memmap(path, dtype=numpy.uint8, mode="r")
    totals = collections.Counter()
    per_tensor = []
    elements = 0
    for name, entry in sorted(tensors.items()):
        begin, end = entry["data_offsets"]
        span = raw[data_start + begin : data_start + end]
        if span.size != end - begin:
            fail(f"{name} maps {span.size} bytes, the header declares {end - begin}")
        bits = span.view(numpy.uint16)
        declared = 1
        for axis in entry["shape"]:
            declared *= axis
        if bits.size != declared:
            fail(f"{name} holds {bits.size} elements, the header's shape declares {declared}")
        elements += bits.size

        exponent = numpy.bitwise_and(bits, numpy.uint16(BF16_EXPONENT_MASK))
        significand = numpy.bitwise_and(bits, numpy.uint16(BF16_SIGNIFICAND_MASK))
        saturated = exponent == numpy.uint16(BF16_EXPONENT_MASK)
        empty = exponent == numpy.uint16(0)
        has_significand = significand != numpy.uint16(0)

        row = {
            "nan": int(numpy.count_nonzero(saturated & has_significand)),
            "infinite": int(numpy.count_nonzero(saturated & ~has_significand)),
            "subnormal": int(numpy.count_nonzero(empty & has_significand)),
            "zero": int(numpy.count_nonzero(empty & ~has_significand)),
        }
        row["normal"] = bits.size - sum(row.values())
        for key, value in row.items():
            totals[key] += value
        if row["nan"] or row["infinite"] or row["subnormal"]:
            per_tensor.append({"tensor": name, "elements": bits.size, **row})

    counted = sum(totals.values())
    if counted != elements:
        fail(f"classified {counted} elements against {elements} read")
    return {
        "tensors": len(tensors),
        "elements": elements,
        "totals": totals,
        "exceptional_tensors": per_tensor,
    }


# ---------------------------------------------------------------------------
# Stage 1 -- the structural pass. No model, no torch.
# ---------------------------------------------------------------------------


def bf16_row_to_f32(chunk: bytes):
    """Widen one stored BF16 row to F32, which is exact for every finite value.

    BF16 is a truncated F32, so the widening is a zero-extension of the
    significand and introduces no rounding -- which is why a bit-identical BF16
    row pair is a bit-identical F32 column pair rather than merely a close one.
    """
    return [
        struct.unpack("<f", b"\x00\x00" + chunk[2 * lane : 2 * lane + 2])[0]
        for lane in range(len(chunk) // 2)
    ]


def exact_norm(values) -> float:
    """The Euclidean norm, computed so that it is a property of the values alone.

    Each F32 value squares exactly in float64 and `math.fsum` is exactly
    rounded, so the result depends on neither summation order nor host. It is
    the one figure in this record a reader on another machine can compare.
    """
    return math.sqrt(math.fsum(value * value for value in values))


def duplicate_embedding_rows(snapshot: Path):
    """Group the embedding matrix's rows by exact stored bit pattern.

    Reads the safetensors header and then the tensor's bytes directly, so the
    population is the file's own declared row count rather than whatever a
    loader chose to materialize. Every group is re-checked for exact byte
    equality before it is reported, so a digest collision cannot manufacture one.
    """
    path = snapshot / "model.safetensors"
    with open(path, "rb") as handle:
        (header_len,) = struct.unpack("<Q", handle.read(8))
        header = json.loads(handle.read(header_len))
        data_start = 8 + header_len

        entry = header.get(EMBEDDING_TENSOR)
        if entry is None:
            fail(f"the checkpoint header declares no {EMBEDDING_TENSOR}")
        if entry["dtype"] != STORED_DTYPE:
            fail(f"{EMBEDDING_TENSOR} is stored as {entry['dtype']}, the profile records {STORED_DTYPE}")
        rows, cols = entry["shape"]
        if [rows, cols] != [VOCAB_SIZE, HIDDEN_SIZE]:
            fail(f"{EMBEDDING_TENSOR} is {rows}x{cols}, the profile records {VOCAB_SIZE}x{HIDDEN_SIZE}")
        if "lm_head.weight" in header:
            fail(
                "the checkpoint declares an lm_head.weight, so the embedding is not tied and the "
                "structural argument this probe rests on does not apply"
            )

        row_bytes = cols * 2  # BF16
        begin, end = entry["data_offsets"]
        if end - begin != rows * row_bytes:
            fail(f"{EMBEDDING_TENSOR} occupies {end - begin} bytes, not {rows * row_bytes}")
        handle.seek(data_start + begin)
        buffer = handle.read(rows * row_bytes)
    if len(buffer) != rows * row_bytes:
        fail(f"read {len(buffer)} embedding bytes, expected {rows * row_bytes}")

    by_digest = collections.defaultdict(list)
    for index in range(rows):
        chunk = buffer[index * row_bytes : (index + 1) * row_bytes]
        by_digest[hashlib.sha256(chunk).digest()].append(index)

    groups = []
    for members in by_digest.values():
        if len(members) < 2:
            continue
        first = buffer[members[0] * row_bytes : (members[0] + 1) * row_bytes]
        for other in members[1:]:
            if buffer[other * row_bytes : (other + 1) * row_bytes] != first:
                fail(
                    f"rows {members[0]} and {other} share a SHA-256 and differ in bytes; "
                    "the grouping is not a byte-equality grouping"
                )
        groups.append(sorted(members))
    groups.sort(key=lambda members: (-len(members), members[0]))

    def norm_of(index: int) -> float:
        return exact_norm(bf16_row_to_f32(buffer[index * row_bytes : (index + 1) * row_bytes]))

    return {
        "population": rows,
        "distinct_patterns": len(by_digest),
        "groups": groups,
        "duplicated_rows": sum(len(members) for members in groups),
        "norm_of": norm_of,
    }


# ---------------------------------------------------------------------------
# Stage 2 -- evaluate prompts through the pinned reference.
# ---------------------------------------------------------------------------


def load_model(snapshot: Path):
    import torch
    from transformers import AutoModelForCausalLM

    model = AutoModelForCausalLM.from_pretrained(
        str(snapshot), torch_dtype=torch.float32, attn_implementation="eager"
    )
    model.eval()
    if model.config._attn_implementation != "eager":
        fail(f"the loaded model reports {model.config._attn_implementation!r}, not 'eager'")
    if next(model.parameters()).dtype != torch.float32:
        fail("the loaded model parameters are not float32")
    if getattr(model.config, "tie_word_embeddings", None) is not True:
        fail("the loaded config does not declare tie_word_embeddings: true")
    return model


def evaluate_prompt(model, ids):
    """One prefill pass, every position's logits retained.

    `logits_to_keep=0` is the pinned reference's spelling for every position --
    the source turns it into `slice(0, None)` -- and it is what makes a prefill
    pass offer more than one candidate position for a tie.
    """
    import torch

    with torch.no_grad():
        out = model(
            input_ids=torch.tensor([ids], dtype=torch.long),
            use_cache=False,
            logits_to_keep=0,
        )
    logits = out.logits[0]
    if tuple(logits.shape) != (len(ids), VOCAB_SIZE):
        fail(f"logits are {tuple(logits.shape)}, expected {(len(ids), VOCAB_SIZE)}")
    return logits


def analyse_position(logits_row, member_index, group_of):
    """The tie observable at one position, and how close the structural route came."""
    import torch

    maximum = torch.max(logits_row)
    attaining = torch.nonzero(logits_row == maximum, as_tuple=False).flatten().tolist()
    ordered = torch.argsort(logits_row, descending=True, stable=True)
    first, second = int(ordered[0]), int(ordered[1])

    best_value, best_offset = torch.max(logits_row[member_index], dim=0)
    best_member = int(member_index[int(best_offset)])
    # Rank counts strictly greater logits, so the best-placed member's rank is
    # 1 exactly when no entry exceeds it -- which is the condition the
    # structural route needs.
    best_rank = int(torch.sum(logits_row > best_value)) + 1

    greedy = min(attaining)
    group = group_of.get(greedy)
    group_bits = None
    if group is not None:
        group_bits = {f32_bits(float(logits_row[member])) for member in group}

    return {
        "greedy": greedy,
        "attaining_count": len(attaining),
        "top1": first,
        "top2": second,
        "top1_bits": f32_bits(float(logits_row[first])),
        "top2_bits": f32_bits(float(logits_row[second])),
        "top_two_bit_identical": f32_bits(float(logits_row[first])) == f32_bits(float(logits_row[second])),
        "gap": float(logits_row[first]) - float(logits_row[second]),
        "greedy_in_duplicate_group": group is not None,
        "greedy_group_all_bit_identical": None if group_bits is None else len(group_bits) == 1,
        "best_duplicate_member": best_member,
        "best_duplicate_logit": float(best_value),
        "best_duplicate_rank": best_rank,
        "best_duplicate_gap": float(maximum) - float(best_value),
    }


def check_tie_detector(member_index, group_of) -> None:
    """Watch the tie observable say *yes*, on a row built to make it say yes.

    Every reported position says "no bit-identical top-two pair", and a detector
    that could only ever say that would be indistinguishable from one that
    works. So the positive control is mandatory and not decorative: a synthetic
    logit row whose maximum is attained by exactly two indices, checked to
    report the tie, the count, and the lowest attaining index as the greedy
    token -- which is the oracle's declared policy and the thing a demonstrating
    corpus row would have to exercise.
    """
    import torch

    row = torch.arange(VOCAB_SIZE, dtype=torch.float32) * -1.0
    lower, upper = 7, 9
    row[lower] = 1.5
    row[upper] = 1.5
    record = analyse_position(row, member_index, group_of)
    if not record["top_two_bit_identical"]:
        fail("the tie detector reported no tie on a row with two bit-identical maxima")
    if record["attaining_count"] != 2:
        fail(f"the tie detector counted {record['attaining_count']} attaining indices, expected 2")
    if record["greedy"] != lower:
        fail(
            f"the tie detector chose {record['greedy']} as the greedy token; the declared policy "
            f"is the lowest attaining index, which is {lower}"
        )
    if record["gap"] != 0.0:
        fail(f"the tie detector reported a runner-up gap of {record['gap']!r} at a tie")


def check_control(rows) -> None:
    """The transcribed C1 positions must be reproduced exactly, or nothing else counts."""
    if len(rows) != len(CONTROL_EXPECTED):
        fail(f"the control produced {len(rows)} positions, expected {len(CONTROL_EXPECTED)}")
    for row, (position, greedy, top1_bits, top2_bits, gap) in zip(rows, CONTROL_EXPECTED):
        if row["position"] != position:
            fail(f"control row {row['position']} is out of order; expected position {position}")
        if row["greedy"] != greedy:
            fail(
                f"control position {position}: greedy token {row['greedy']}, the retained "
                f"conformance fixture records {greedy}"
            )
        if row["top1_bits"] != f"0x{top1_bits:08x}" or row["top2_bits"] != f"0x{top2_bits:08x}":
            fail(
                f"control position {position}: top-two bits {row['top1_bits']}/{row['top2_bits']}, "
                f"the fixture records 0x{top1_bits:08x}/0x{top2_bits:08x}"
            )
        if row["gap"] != repr(gap):
            fail(
                f"control position {position}: runner-up gap {row['gap']}, the fixture "
                f"records {gap!r}"
            )
        if row["top_two_bit_identical"] != "false":
            fail(
                f"control position {position} reports a bit-identical top-two pair; the fixture "
                "measures none at any C1 position, so this probe disagrees with its own control"
            )


def write_kv_tsv(path: Path, rows) -> None:
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write("key\tvalue\n")
        for key, value in rows:
            handle.write(f"{key}\t{value}\n")


def write_tsv(path: Path, rows, columns) -> None:
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write("\t".join(columns) + "\n")
        for row in rows:
            handle.write("\t".join(str(row[column]) for column in columns) + "\n")


POSITION_COLUMNS = [
    "prompt",
    "fed_token",
    "partner_token",
    "prompt_length",
    "position",
    "greedy",
    "attaining_count",
    "top1",
    "top2",
    "top1_bits",
    "top2_bits",
    "top_two_bit_identical",
    "gap",
    "greedy_in_duplicate_group",
    "greedy_group_all_bit_identical",
    "best_duplicate_member",
    "best_duplicate_logit",
    "best_duplicate_rank",
    "best_duplicate_gap",
]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True, help="directory to write the record into")
    parser.add_argument(
        "--structural-only",
        action="store_true",
        help="run stages 0 and 1 alone; they need neither torch nor a forward pass",
    )
    args = parser.parse_args()

    environment = list(host_row())
    environment.append(("workload.repo_id", REPO_ID))
    environment.append(("workload.revision", REVISION))

    snapshot, checkpoint_records = acquire_and_verify_checkpoint()
    environment.extend(checkpoint_records)

    widening = widening_class_map()
    widening_rows = [
        {
            "stored_class": stored,
            "widened_class": widened,
            "patterns": count,
        }
        for (stored, widened), count in sorted(widening.items())
    ]
    environment.append(("widening.patterns_examined", str(sum(widening.values()))))
    environment.append(
        ("widening.subnormal_to_subnormal", str(widening[("subnormal", "subnormal")]))
    )
    environment.append(("widening.subnormal_to_normal", str(widening[("subnormal", "normal")]))
    )
    environment.append(("widening.class_preserved", str(sum(
        count for (stored, widened), count in widening.items() if stored == widened
    ))))

    stored = stored_value_classes(snapshot)
    environment.append(("checkpoint.tensors_examined", str(stored["tensors"])))
    environment.append(("checkpoint.elements_examined", str(stored["elements"])))
    for name in ("normal", "zero", "subnormal", "infinite", "nan"):
        environment.append((f"checkpoint.stored_{name}", str(stored["totals"][name])))
    environment.append(
        ("checkpoint.tensors_with_an_exceptional_value", str(len(stored["exceptional_tensors"])))
    )

    structural = duplicate_embedding_rows(snapshot)
    group_of = {}
    for members in structural["groups"]:
        for member in members:
            group_of[member] = members
    duplicated = sorted(group_of)

    environment.append(("embedding.rows_examined", str(structural["population"])))
    environment.append(("embedding.distinct_bit_patterns", str(structural["distinct_patterns"])))
    environment.append(("embedding.duplicate_groups", str(len(structural["groups"]))))
    environment.append(("embedding.rows_in_a_duplicate_group", str(structural["duplicated_rows"])))
    environment.append(
        (
            "embedding.largest_duplicate_group",
            str(len(structural["groups"][0])) if structural["groups"] else "0",
        )
    )

    group_rows = []
    for index, members in enumerate(structural["groups"]):
        group_rows.append(
            {
                "group_index": index,
                "size": len(members),
                "lowest_id": members[0],
                "row_l2_norm": repr(structural["norm_of"](members[0])),
                "member_ids": ",".join(str(member) for member in members[:16])
                + ("..." if len(members) > 16 else ""),
            }
        )

    position_rows = []
    if not args.structural_only:
        environment.extend(verify_reference_sources())
        import numpy
        import torch
        import transformers

        torch.set_num_threads(1)
        environment.append(("python.version", platform.python_version()))
        environment.append(("torch.version", torch.__version__))
        environment.append(("transformers.version", transformers.__version__))
        environment.append(("numpy.version", numpy.__version__))
        environment.append(("torch.num_threads", str(torch.get_num_threads())))

        member_index = torch.tensor(duplicated, dtype=torch.long)
        check_tie_detector(member_index, group_of)
        environment.append(("probe.tie_detector_positive_control", "passed"))
        model = load_model(snapshot)

        prompts = [(CONTROL_NAME, None, None, CONTROL_IDS)]
        for name, fed, partner, repeats in CANDIDATE_PROMPTS:
            if partner not in group_of.get(fed, ()):
                fail(
                    f"candidate {name} names {fed} and {partner} as a duplicate pair, and the "
                    "structural pass does not group them together"
                )
            prompts.append((name, fed, partner, [fed] * repeats))

        for name, fed, partner, ids in prompts:
            logits = evaluate_prompt(model, ids)
            for position in range(len(ids)):
                record = analyse_position(logits[position], member_index, group_of)
                position_rows.append(
                    {
                        "prompt": name,
                        "fed_token": "" if fed is None else fed,
                        "partner_token": "" if partner is None else partner,
                        "prompt_length": len(ids),
                        "position": position,
                        "greedy": record["greedy"],
                        "attaining_count": record["attaining_count"],
                        "top1": record["top1"],
                        "top2": record["top2"],
                        "top1_bits": f"0x{record['top1_bits']:08x}",
                        "top2_bits": f"0x{record['top2_bits']:08x}",
                        "top_two_bit_identical": str(record["top_two_bit_identical"]).lower(),
                        "gap": repr(record["gap"]),
                        "greedy_in_duplicate_group": str(record["greedy_in_duplicate_group"]).lower(),
                        "greedy_group_all_bit_identical": (
                            ""
                            if record["greedy_group_all_bit_identical"] is None
                            else str(record["greedy_group_all_bit_identical"]).lower()
                        ),
                        "best_duplicate_member": record["best_duplicate_member"],
                        "best_duplicate_logit": repr(record["best_duplicate_logit"]),
                        "best_duplicate_rank": record["best_duplicate_rank"],
                        "best_duplicate_gap": repr(record["best_duplicate_gap"]),
                    }
                )

        check_control([row for row in position_rows if row["prompt"] == CONTROL_NAME])

        ties = [row for row in position_rows if row["top_two_bit_identical"] == "true"]
        ranks = [row["best_duplicate_rank"] for row in position_rows]
        gaps = [float(row["best_duplicate_gap"]) for row in position_rows]
        environment.append(("probe.prompts_evaluated", str(len(prompts))))
        environment.append(("probe.positions_evaluated", str(len(position_rows))))
        environment.append(("probe.control_positions", str(len(CONTROL_EXPECTED))))
        environment.append(("probe.tie_positions", str(len(ties))))
        environment.append(("probe.best_duplicate_rank_min", str(min(ranks))))
        environment.append(("probe.best_duplicate_rank_max", str(max(ranks))))
        environment.append(("probe.best_duplicate_gap_min", repr(min(gaps))))
        environment.append(("probe.best_duplicate_gap_max", repr(max(gaps))))

    args.out.mkdir(parents=True, exist_ok=True)
    write_kv_tsv(args.out / "environment.tsv", environment)
    write_tsv(
        args.out / "widening.tsv",
        widening_rows,
        ["stored_class", "widened_class", "patterns"],
    )
    write_tsv(
        args.out / "exceptional.tsv",
        stored["exceptional_tensors"],
        ["tensor", "elements", "normal", "zero", "subnormal", "infinite", "nan"],
    )
    write_tsv(
        args.out / "duplicate_groups.tsv",
        group_rows,
        ["group_index", "size", "lowest_id", "row_l2_norm", "member_ids"],
    )
    if position_rows:
        write_tsv(args.out / "positions.tsv", position_rows, POSITION_COLUMNS)

    here = Path(__file__).resolve().parent
    manifest = [
        (name, sha256_file(args.out / name))
        for name in sorted(path.name for path in args.out.iterdir() if path.name != "manifest.tsv")
    ]
    for name in ("probe_corpus.py", "pyproject.toml", "uv.lock"):
        manifest.append((name, sha256_file(here / name)))
    write_kv_tsv(args.out / "manifest.tsv", manifest)

    print(f"BF16 patterns widened: {sum(widening.values())}")
    for (stored_class, widened_class), count in sorted(widening.items()):
        print(f"  stored {stored_class} -> widened {widened_class}: {count}")
    print(f"stored elements examined: {stored['elements']} over {stored['tensors']} tensors")
    for name in ("normal", "zero", "subnormal", "infinite", "nan"):
        print(f"  stored {name}: {stored['totals'][name]}")
    print(f"embedding rows examined: {structural['population']}")
    print(f"distinct embedding bit patterns: {structural['distinct_patterns']}")
    print(f"duplicate groups: {len(structural['groups'])}")
    print(f"rows in a duplicate group: {structural['duplicated_rows']}")
    if position_rows:
        ties = [row for row in position_rows if row["top_two_bit_identical"] == "true"]
        ranks = [row["best_duplicate_rank"] for row in position_rows]
        gaps = [float(row["best_duplicate_gap"]) for row in position_rows]
        print(f"positions evaluated: {len(position_rows)} (control {len(CONTROL_EXPECTED)})")
        print(f"positions with a bit-identical top-two pair: {len(ties)}")
        print(f"best duplicate-group member rank: {min(ranks)} .. {max(ranks)} of {VOCAB_SIZE}")
        print(f"best duplicate-group member gap below the maximum: {min(gaps):.4f} .. {max(gaps):.4f}")
    print(f"record written to {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
