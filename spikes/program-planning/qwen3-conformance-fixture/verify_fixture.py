#!/usr/bin/env python3
"""Verify a retained C1 conformance and attribution fixture without owning a model.

`produce_fixture.py --compare` is the strong check: it regenerates everything
and demands byte equality. It also needs the 1.19 GB checkpoint, the pinned
environment, and a few seconds of a machine. This validator is the cheap one
that any reader can run, and it is what makes an *edited* record distinguishable
from a *regenerated* one:

  * the manifest digests catch any byte that changed in a retained file;
  * the cross-file consistency checks catch a change made consistently enough to
    survive a re-hashed manifest -- a greedy token that no longer agrees with
    the top-32 table, a gap that no longer equals the difference of the two
    logits it is derived from, a generated token that no longer equals the
    argmax of the position that produced it, an attribution head whose ordering
    or extremum disagrees with the row it summarizes, a rotary table whose two
    halves are no longer the duplication the pinned source builds, a mask entry
    that is neither of L4's two admitted values or that is admitted at a
    position causality forbids, a joint band that is not the maximum over the
    rows it summarizes, a P-elem size that is no longer its registered
    contract's, and a P-flush term state that does not follow from its own
    controls;
  * `--logit-dir` and `--attribution-dir` additionally re-hash the regenerable
    F32 bytes against the retained per-slice digests, when they happen to be
    present locally.

Every check names its population and counts it, so "nothing failed" is
distinguishable from "nothing ran".

    uv run --locked python verify_fixture.py results/<slug>
    uv run --locked python verify_fixture.py results/<slug> \\
        --logit-dir local-work/logits --attribution-dir local-work/attribution
"""

from __future__ import annotations

import argparse
import hashlib
import struct
import sys
from pathlib import Path

REVISION = "da87bfb608c14b7cf20ba1ce41287e8de496c0cd"
PROMPT_IDS = [785, 3974, 13876, 38835, 34208, 916, 279, 15678, 5562, 13]
DECODE_STEPS = 8
VOCAB_SIZE = 151936
TOP_K = 32
EXPECTED_POSITIONS = len(PROMPT_IDS) + DECODE_STEPS
ENVELOPE_VARIANTS = {"f64_unmodified", "f64_promoted"}

# The joint band's variants and the two registered accuracy contracts it is
# sized from. The exponential's 12 is the registered `Ulp(tiler::ulp-reference-
# gap@1, 12)` and deliberately not Table 8.1's 4; the reciprocal square root's 1
# is the supremum of the `Faithful` band.
ELEM_SIGN_POLICIES = ("outward", "alternating")
JOINT_ORDERINGS = ("unmodified", "promoted")
JOINT_VARIANTS = {
    f"joint_{ordering}_{policy}" for ordering in JOINT_ORDERINGS for policy in ELEM_SIGN_POLICIES
}
ELEM_EXP_ULPS = 12
ELEM_RSQRT_ULPS = 1

# The attribution surface's shape, from L1's config facts and L6's arithmetic.
NUM_LAYERS = 28
HIDDEN_SIZE = 1024
KV_HEADS = 8
HEAD_DIM = 128
ATTRIBUTION_TOP_K = 4
CACHE_TENSORS = ("k_rope", "v_heads")
HIDDEN_BYTES = NUM_LAYERS * EXPECTED_POSITIONS * HIDDEN_SIZE * 4
CACHE_BYTES = NUM_LAYERS * len(CACHE_TENSORS) * KV_HEADS * EXPECTED_POSITIONS * HEAD_DIM * 4
POSITION_SLICE_BYTES = HIDDEN_SIZE * 4
MASK_MASKED_ENTRY = "0xff7fffff"
MASK_ATTENDED_ENTRY = "0x80000000"
WEIGHT_TENSOR_COUNT = 310
WIDENED_WEIGHT_BYTES = 2_384_199_680

RESULT_FILES = [
    "environment.tsv",
    "sequence.tsv",
    "positions.tsv",
    "top32.tsv",
    "envelope.tsv",
    "joint.tsv",
    "perturbation.tsv",
    "hidden.tsv",
    "hidden_top.tsv",
    "cache.tsv",
    "cache_top.tsv",
    "rotary.tsv",
    "mask.tsv",
    "host.tsv",
]


class Report:
    def __init__(self) -> None:
        self.failures = []
        self.checks = 0

    def require(self, condition: bool, message: str) -> bool:
        self.checks += 1
        if not condition:
            self.failures.append(message)
        return condition

    def note(self, message: str) -> None:
        print(f"  {message}")


def read_tsv(path: Path):
    lines = path.read_text(encoding="utf-8").splitlines()
    header = lines[0].split("\t")
    return [dict(zip(header, line.split("\t"))) for line in lines[1:]]


def read_kv(path: Path):
    return {row["key"]: row["value"] for row in read_tsv(path)}


def float_from_hex(text: str) -> float:
    return struct.unpack("<f", struct.pack("<I", int(text, 16)))[0]


def verify(directory: Path, logit_dir: Path | None, attribution_dir: Path | None) -> int:
    report = Report()

    for name in RESULT_FILES + ["manifest.tsv"]:
        if not (directory / name).exists():
            print(f"FAIL: {directory} is missing {name}", file=sys.stderr)
            return 6

    # --- manifest -----------------------------------------------------------
    manifest = read_kv(directory / "manifest.tsv")
    hashed = 0
    for name in RESULT_FILES:
        key = f"result.sha256.{name}"
        if not report.require(key in manifest, f"manifest has no entry for {name}"):
            continue
        actual = hashlib.sha256((directory / name).read_bytes()).hexdigest()
        report.require(
            actual == manifest[key],
            f"{name} hashes to {actual}, the manifest records {manifest[key]}",
        )
        hashed += 1
    report.note(f"manifest: {hashed} of {len(RESULT_FILES)} retained files re-hashed")

    producer_root = Path(__file__).resolve().parent
    tracked = 0
    for key, relative in [
        ("producer.sha256.produce_fixture.py", "produce_fixture.py"),
        ("producer.sha256.verify_fixture.py", "verify_fixture.py"),
        ("producer.sha256.pyproject.toml", "pyproject.toml"),
        ("producer.sha256.uv.lock", "uv.lock"),
    ]:
        if key not in manifest:
            report.require(False, f"manifest has no entry for {relative}")
            continue
        path = producer_root / relative
        if not path.exists():
            report.require(False, f"{relative} is missing beside the validator")
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        report.require(
            actual == manifest[key],
            f"{relative} hashes to {actual}, the manifest records {manifest[key]}. "
            "The record was produced by different sources than the ones present.",
        )
        tracked += 1
    report.note(f"manifest: {tracked} of 4 producer sources re-hashed")

    # --- environment --------------------------------------------------------
    environment = read_kv(directory / "environment.tsv")
    report.require(
        environment.get("workload.revision") == REVISION,
        f"record names revision {environment.get('workload.revision')}, expected {REVISION}",
    )
    report.require(
        environment.get("workload.retained_positions") == str(EXPECTED_POSITIONS),
        f"record claims {environment.get('workload.retained_positions')} retained positions",
    )
    report.require(
        environment.get("workload.decode_budget") == str(DECODE_STEPS),
        f"record claims a decode budget of {environment.get('workload.decode_budget')}",
    )
    report.require(
        environment.get("run.attn_implementation") == "eager",
        f"record names attention implementation {environment.get('run.attn_implementation')}",
    )
    report.require(
        environment.get("run.dtype") == "float32",
        f"record names dtype {environment.get('run.dtype')}",
    )
    report.require(
        environment.get("run.logits_to_keep") == "0",
        f"record names logits_to_keep {environment.get('run.logits_to_keep')}",
    )
    report.require(
        environment.get("run.terminated_on") in ("eos", "budget"),
        f"record names termination {environment.get('run.terminated_on')}",
    )
    report.require(
        environment.get("checkpoint.sha256.model.safetensors")
        == "cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba",
        "record does not carry the manifest checkpoint digest",
    )
    report.require(
        environment.get("attribution.hidden_bytes") == str(HIDDEN_BYTES),
        f"record claims {environment.get('attribution.hidden_bytes')} hidden-state bytes, "
        f"L6's figure is {HIDDEN_BYTES}",
    )
    report.require(
        environment.get("attribution.cache_bytes") == str(CACHE_BYTES),
        f"record claims {environment.get('attribution.cache_bytes')} cache bytes, "
        f"L6's figure is {CACHE_BYTES}",
    )
    report.require(
        environment.get("attribution.top_k") == str(ATTRIBUTION_TOP_K),
        f"record claims a top-k of {environment.get('attribution.top_k')}",
    )
    report.note(f"environment: {len(environment)} keys, 11 asserted")

    # --- sequence -----------------------------------------------------------
    sequence = read_tsv(directory / "sequence.tsv")
    report.require(
        len(sequence) == EXPECTED_POSITIONS,
        f"sequence has {len(sequence)} tokens, expected {EXPECTED_POSITIONS}",
    )
    prompt_rows = [row for row in sequence if row["role"] == "prompt"]
    generated_rows = [row for row in sequence if row["role"] == "generated"]
    report.require(
        [int(row["token_id"]) for row in prompt_rows] == PROMPT_IDS,
        "the recorded prompt tokens are not the profile's prompt token IDs",
    )
    report.require(
        len(generated_rows) == DECODE_STEPS or environment.get("run.terminated_on") == "eos",
        f"{len(generated_rows)} generated tokens under a {DECODE_STEPS}-step budget without EOS",
    )
    for offset, row in enumerate(sequence):
        report.require(
            int(row["index"]) == offset, f"sequence row {offset} carries index {row['index']}"
        )
    report.note(f"sequence: {len(prompt_rows)} prompt + {len(generated_rows)} generated tokens")

    # --- positions ----------------------------------------------------------
    positions = read_tsv(directory / "positions.tsv")
    report.require(
        len(positions) == EXPECTED_POSITIONS,
        f"positions has {len(positions)} rows, expected {EXPECTED_POSITIONS}",
    )
    prefill = [row for row in positions if row["stage"] == "prefill"]
    decode = [row for row in positions if row["stage"] == "decode"]
    report.require(
        len(prefill) == len(PROMPT_IDS),
        f"{len(prefill)} prefill positions, expected {len(PROMPT_IDS)}",
    )
    report.require(
        len(decode) == DECODE_STEPS, f"{len(decode)} decode positions, expected {DECODE_STEPS}"
    )

    ties = []
    for offset, row in enumerate(positions):
        report.require(int(row["position"]) == offset, f"positions row {offset} carries position {row['position']}")
        report.require(
            len(row["logits_sha256"]) == 64 and all(c in "0123456789abcdef" for c in row["logits_sha256"]),
            f"position {offset} carries a malformed logit digest",
        )
        best = float_from_hex(row["greedy_logit_hex"])
        runner = float_from_hex(row["runner_up_logit_hex"])
        report.require(
            repr(best) == row["greedy_logit"],
            f"position {offset}: greedy logit {row['greedy_logit']} does not decode from "
            f"{row['greedy_logit_hex']} (which is {best!r})",
        )
        report.require(
            repr(runner) == row["runner_up_logit"],
            f"position {offset}: runner-up logit {row['runner_up_logit']} does not decode from "
            f"{row['runner_up_logit_hex']} (which is {runner!r})",
        )
        report.require(
            repr(best - runner) == row["runner_up_gap"],
            f"position {offset}: recorded gap {row['runner_up_gap']} is not "
            f"{repr(best - runner)}, the difference of the two recorded logits",
        )
        report.require(best >= runner, f"position {offset}: the greedy logit is below the runner-up")
        report.require(
            row["top_two_bit_identical"]
            == str(row["greedy_logit_hex"] == row["runner_up_logit_hex"]).lower(),
            f"position {offset}: the tie flag disagrees with the recorded bit patterns",
        )
        report.require(
            int(row["max_attaining_indices"]) >= 1,
            f"position {offset}: no index attains the maximum",
        )
        # The declared tie policy is the lowest vocabulary index among maxima,
        # so a runner-up that ties the maximum must carry a higher index.
        if row["top_two_bit_identical"] == "true":
            ties.append(offset)
            report.require(
                int(row["runner_up_token"]) > int(row["greedy_token"]),
                f"position {offset}: a bit-identical tie resolved to the higher index, "
                "which is not the declared policy",
            )
    report.note(
        f"positions: {len(positions)} rows ({len(prefill)} prefill, {len(decode)} decode), "
        f"{len(ties)} bit-identical top-two tie(s) at {ties or 'no position'}"
    )

    # Every generated token must be the greedy token of the position that
    # produced it: position 9 produces the first, position 9+k the (k+1)th.
    for offset, row in enumerate(generated_rows):
        source = positions[len(PROMPT_IDS) - 1 + offset]
        report.require(
            row["token_id"] == source["greedy_token"],
            f"generated token {offset} is {row['token_id']} but position "
            f"{source['position']} selected {source['greedy_token']}",
        )
    report.note(f"decode chain: {len(generated_rows)} generated tokens tied back to their positions")

    # --- top32 --------------------------------------------------------------
    top32 = read_tsv(directory / "top32.tsv")
    report.require(
        len(top32) == EXPECTED_POSITIONS * TOP_K,
        f"top32 has {len(top32)} rows, expected {EXPECTED_POSITIONS * TOP_K}",
    )
    by_position = {}
    for row in top32:
        by_position.setdefault(int(row["position"]), []).append(row)
    for index in range(EXPECTED_POSITIONS):
        rows = by_position.get(index, [])
        if not report.require(len(rows) == TOP_K, f"position {index} has {len(rows)} top-k rows"):
            continue
        previous = None
        for rank, row in enumerate(rows):
            report.require(int(row["rank"]) == rank, f"position {index} rank {rank} is out of order")
            token = int(row["token_id"])
            report.require(0 <= token < VOCAB_SIZE, f"position {index} rank {rank} token {token} is out of range")
            value = float_from_hex(row["logit_hex"])
            report.require(
                repr(value) == row["logit"],
                f"position {index} rank {rank}: logit {row['logit']} does not decode from {row['logit_hex']}",
            )
            if previous is not None:
                previous_value, previous_token = previous
                report.require(
                    value < previous_value or (value == previous_value and token > previous_token),
                    f"position {index} rank {rank}: ordering violates descending logit with "
                    "ties broken toward the lower vocabulary index",
                )
            previous = (value, token)
        head = positions[index]
        report.require(
            rows[0]["token_id"] == head["greedy_token"] and rows[0]["logit_hex"] == head["greedy_logit_hex"],
            f"position {index}: the top-32 rank 0 entry disagrees with the recorded greedy token",
        )
        report.require(
            rows[1]["token_id"] == head["runner_up_token"] and rows[1]["logit_hex"] == head["runner_up_logit_hex"],
            f"position {index}: the top-32 rank 1 entry disagrees with the recorded runner-up",
        )
    report.note(f"top32: {len(top32)} rows over {len(by_position)} positions, ordering and heads checked")

    # --- envelope -----------------------------------------------------------
    envelope = read_tsv(directory / "envelope.tsv")
    variants = {row["variant"] for row in envelope}
    report.require(
        variants == ENVELOPE_VARIANTS,
        f"envelope carries variants {sorted(variants)}, expected {sorted(ENVELOPE_VARIANTS)}",
    )
    report.require(
        len(envelope) == EXPECTED_POSITIONS * len(ENVELOPE_VARIANTS),
        f"envelope has {len(envelope)} rows, expected {EXPECTED_POSITIONS * len(ENVELOPE_VARIANTS)}",
    )
    for row in envelope:
        report.require(
            0 <= int(row["bit_identical_logits"]) <= VOCAB_SIZE,
            f"envelope position {row['position']} {row['variant']}: implausible bit-identical count",
        )
        report.require(
            float(row["max_abs_deviation"]) >= 0.0,
            f"envelope position {row['position']} {row['variant']}: negative deviation",
        )
        report.require(
            float(row["top32_max_abs_deviation"]) <= float(row["max_abs_deviation"]),
            f"envelope position {row['position']} {row['variant']}: the top-32 deviation exceeds "
            "the whole-vocabulary deviation it is a subset of",
        )
        report.require(
            row["greedy_token_agrees"] in ("true", "false"),
            f"envelope position {row['position']} {row['variant']}: malformed agreement flag",
        )
    report.note(f"envelope: {len(envelope)} rows over {len(variants)} variants")

    # --- joint band ---------------------------------------------------------
    # The joint deviation is one relation between two complete computations, so
    # every check here is a structural identity or a recomputation from values
    # the record already carries. None of them asserts a magnitude: the band is
    # the measurement, and a threshold over it belongs to the corpus and
    # regression tickets rather than to this validator.
    joint = read_tsv(directory / "joint.tsv")
    joint_variants = {row["variant"] for row in joint}
    report.require(
        joint_variants == JOINT_VARIANTS,
        f"joint carries variants {sorted(joint_variants)}, expected {sorted(JOINT_VARIANTS)}",
    )
    report.require(
        len(joint) == EXPECTED_POSITIONS * len(JOINT_VARIANTS),
        f"joint has {len(joint)} rows, expected {EXPECTED_POSITIONS * len(JOINT_VARIANTS)}",
    )
    greedy_of = {row["position"]: row["greedy_token"] for row in positions}
    gap_of = {row["position"]: row["runner_up_gap"] for row in positions}
    joint_agreeing = 0
    for row in joint:
        label = f"joint {row['variant']} position {row['position']}"
        report.require(
            0 <= int(row["bit_identical_logits"]) <= VOCAB_SIZE,
            f"{label}: implausible bit-identical count",
        )
        report.require(float(row["max_abs_deviation"]) >= 0.0, f"{label}: negative deviation")
        report.require(
            float(row["top32_max_abs_deviation"]) <= float(row["max_abs_deviation"]),
            f"{label}: the top-32 deviation exceeds the whole-vocabulary deviation it is a subset of",
        )
        report.require(
            int(row["top32_max_ulp_deviation"]) <= int(row["max_ulp_deviation"]),
            f"{label}: the top-32 ULP deviation exceeds the whole-vocabulary one",
        )
        report.require(
            row["runner_up_gap"] == gap_of.get(row["position"]),
            f"{label}: the recorded gap {row['runner_up_gap']} is not the gap positions.tsv "
            f"records for that position ({gap_of.get(row['position'])})",
        )
        report.require(
            row["deviation_over_gap"]
            == repr(float(row["max_abs_deviation"]) / float(row["runner_up_gap"])),
            f"{label}: the recorded ratio is not the deviation divided by the recorded gap",
        )
        agrees = row["greedy_token"] == greedy_of.get(row["position"])
        report.require(
            row["greedy_token_agrees"] == str(agrees).lower(),
            f"{label}: the agreement flag disagrees with the greedy token positions.tsv records",
        )
        joint_agreeing += 1 if row["greedy_token_agrees"] == "true" else 0
    report.note(
        f"joint: {len(joint)} rows over {len(joint_variants)} variants, "
        f"{joint_agreeing} agreeing greedy tokens"
    )

    # --- the perturbation record --------------------------------------------
    perturbation = read_kv(directory / "perturbation.tsv")
    report.require(
        perturbation.get("pelem.exp.ulps") == str(ELEM_EXP_ULPS),
        f"the record sizes the exponential perturbation at {perturbation.get('pelem.exp.ulps')} ULPs; "
        f"the registered Ulp(tiler::ulp-reference-gap@1, {ELEM_EXP_ULPS}) contract is {ELEM_EXP_ULPS}",
    )
    report.require(
        perturbation.get("pelem.exp.contract") == "Ulp(tiler::ulp-reference-gap@1, 12)",
        f"the record names exponential contract {perturbation.get('pelem.exp.contract')}",
    )
    report.require(
        perturbation.get("pelem.rsqrt.ulps") == str(ELEM_RSQRT_ULPS)
        and perturbation.get("pelem.rsqrt.contract") == "Faithful",
        "the record does not size the reciprocal square root from the Faithful contract",
    )
    report.require(
        set(perturbation.get("joint.variants", "").split(",")) == JOINT_VARIANTS,
        f"perturbation.tsv names variants {perturbation.get('joint.variants')}",
    )
    report.require(
        environment.get("joint.variants") == perturbation.get("joint.variants"),
        "environment.tsv and perturbation.tsv name different joint variants",
    )
    report.require(
        environment.get("joint.exp_ulps") == perturbation.get("pelem.exp.ulps")
        and environment.get("joint.rsqrt_ulps") == perturbation.get("pelem.rsqrt.ulps")
        and environment.get("joint.sign_policies") == perturbation.get("pelem.sign_policies"),
        "environment.tsv and perturbation.tsv disagree about the P-elem sizes or sign policies",
    )

    band = max(float(row["max_abs_deviation"]) for row in joint)
    report.require(
        perturbation.get("joint.band.max_abs_deviation") == repr(band),
        f"the recorded band {perturbation.get('joint.band.max_abs_deviation')} is not the maximum "
        f"over joint.tsv, {band!r}",
    )
    report.require(
        perturbation.get("joint.band.top32_max_abs_deviation")
        == repr(max(float(row["top32_max_abs_deviation"]) for row in joint)),
        "the recorded top-32 band is not the maximum over joint.tsv",
    )
    report.require(
        perturbation.get("joint.band.max_ulp_deviation")
        == str(max(int(row["max_ulp_deviation"]) for row in joint)),
        "the recorded ULP band is not the maximum over joint.tsv",
    )
    report.require(
        perturbation.get("joint.band.top32_max_ulp_deviation")
        == str(max(int(row["top32_max_ulp_deviation"]) for row in joint)),
        "the recorded top-32 ULP band is not the maximum over joint.tsv",
    )
    report.require(
        perturbation.get("joint.band.rows") == str(len(joint)),
        "the recorded joint population disagrees with joint.tsv",
    )
    report.require(
        perturbation.get("joint.band.greedy_agreeing_rows") == str(joint_agreeing),
        "the recorded greedy-agreement count disagrees with joint.tsv",
    )
    report.require(
        perturbation.get("joint.band.greedy_agrees_everywhere")
        == str(joint_agreeing == len(joint)).lower(),
        "the recorded everywhere-agrees flag disagrees with its own count",
    )

    smallest_gap = min(float(row["runner_up_gap"]) for row in positions)
    report.require(
        perturbation.get("joint.band.smallest_runner_up_gap") == repr(smallest_gap),
        f"the record's smallest runner-up gap is not the minimum over positions.tsv, {smallest_gap!r}",
    )
    report.require(
        perturbation.get("joint.band.gap_ratio") == repr(band / smallest_gap),
        "the recorded gap ratio is not the band divided by the smallest runner-up gap",
    )
    report.require(
        perturbation.get("joint.band.exact_greedy_gate_holds") == str(band < smallest_gap).lower(),
        "the recorded gate verdict disagrees with the band it is derived from",
    )

    # The P-flush mechanism is a claim about a measurement, so the record must
    # carry both arms of both controls and a term state that follows from them.
    controls_passed = True
    for label in ("elementwise", "blas"):
        for suffix in (
            "mode_off_returned_the_exact_subnormal",
            "mode_on_flushed_to_zero",
            "flush_preserved_the_sign",
        ):
            key = f"pflush.control.{label}.{suffix}"
            value = perturbation.get(key)
            report.require(value in ("true", "false"), f"perturbation.tsv carries no verdict at {key}")
            controls_passed = controls_passed and value == "true"
        off = perturbation.get(f"pflush.control.{label}.mode_off_hex", "")
        on = perturbation.get(f"pflush.control.{label}.mode_on_hex", "")
        report.require(
            off == perturbation.get(f"pflush.control.{label}.exact_subnormal_hex")
            or perturbation.get(f"pflush.control.{label}.mode_off_returned_the_exact_subnormal") == "false",
            f"the {label} control claims the mode-off arm returned the exact subnormal but records {off}",
        )
        report.require(
            on in ("0x00000000", "0x80000000")
            or perturbation.get(f"pflush.control.{label}.mode_on_flushed_to_zero") == "false",
            f"the {label} control claims a flush but records {on}",
        )
        report.require(
            off != on,
            f"the {label} control's two arms returned the same bit pattern, so it cannot say no",
        )
    report.require(
        perturbation.get("pflush.controls_passed") == str(controls_passed).lower(),
        "the recorded control verdict disagrees with the individual control rows",
    )

    reachable = perturbation.get("pflush.f32_reachability.bit_identical_positions")
    report.require(
        perturbation.get("pflush.f32_reachability.positions") == str(EXPECTED_POSITIONS),
        "the flush-reachability population is not the row's 18 positions",
    )
    if not controls_passed:
        expected_state = "unknown"
    elif reachable == str(EXPECTED_POSITIONS):
        expected_state = "established, and measured to be the identity on this row"
    else:
        expected_state = "unknown in the joint carrier"
    report.require(
        perturbation.get("pflush.term_state") == expected_state,
        f"the recorded P-flush term state {perturbation.get('pflush.term_state')!r} does not follow "
        f"from its own controls and reachability count ({expected_state!r})",
    )
    report.require(
        perturbation.get("joint.terms_carried")
        == ("P-reorder, P-flush, P-elem" if expected_state.startswith("established") else "P-reorder, P-elem"),
        "the terms the band claims to carry disagree with the P-flush term state",
    )
    # A perturbation that moved nothing would leave the band indistinguishable
    # from the unperturbed re-spelling, so the control is required to have moved.
    controlled_variant = perturbation.get("pelem.control.elem_zero.controlled_variant", "")
    controlled_rows = [row for row in joint if row["variant"] == controlled_variant]
    report.require(
        len(controlled_rows) == EXPECTED_POSITIONS,
        f"the control names variant {controlled_variant!r}, which joint.tsv has "
        f"{len(controlled_rows)} rows for",
    )
    if controlled_rows:
        controlled_band = max(float(row["max_abs_deviation"]) for row in controlled_rows)
        report.require(
            perturbation.get("pelem.control.elem_zero.controlled_variant_max_abs_deviation")
            == repr(controlled_band),
            "the control's recorded variant maximum is not the maximum over that variant's joint.tsv rows",
        )
        # The control is the same pass at zero ULPs, so it must be compared
        # against the variant it controls: a perturbation that reached nothing
        # there would otherwise hide behind another variant's deviation.
        report.require(
            float(perturbation.get("pelem.control.elem_zero.max_abs_deviation", "inf")) < controlled_band,
            "the zero-magnitude control's deviation is not below the variant it controls",
        )
        report.require(
            perturbation.get("pelem.control.elem_zero.band_moved")
            == str(
                controlled_band
                > float(perturbation.get("pelem.control.elem_zero.max_abs_deviation", "inf"))
            ).lower(),
            "the recorded band-moved flag disagrees with the two deviations it is derived from",
        )
    report.require(
        perturbation.get("pelem.control.elem_zero.band_moved") == "true",
        "the zero-magnitude P-elem control did not move the band, so the perturbation reached nothing",
    )
    report.note(
        f"perturbation: {len(perturbation)} keys; both P-flush controls in both arms, "
        f"P-flush term {perturbation.get('pflush.term_state')!r}"
    )

    # --- attribution: per-layer hidden states -------------------------------
    # The digest proves exact regeneration and the head carries the values a
    # bounded comparison needs, so both are checked: the head must agree with
    # the row it summarizes, and the ordering must be the one the record
    # declares. Neither check asserts a magnitude -- no tolerance lives here.
    stage_of = {row["position"]: row["stage"] for row in positions}

    def check_head(rows, label, extent, coordinate):
        """Check a top-k block: rank order, coordinate range, hex/decimal agreement.

        The declared order is descending |value| with ties toward the lower flat
        index, and a block that violates it is a record whose retained
        coordinates no longer name the reference's largest components.
        """
        if not report.require(len(rows) == ATTRIBUTION_TOP_K, f"{label} has {len(rows)} top-k rows"):
            return None
        previous = None
        for rank, row in enumerate(rows):
            report.require(int(row["rank"]) == rank, f"{label} rank {rank} is out of order")
            flat = coordinate(row)
            report.require(0 <= flat < extent, f"{label} rank {rank} names flat index {flat}, out of range")
            value = float_from_hex(row["value_hex"])
            report.require(
                repr(value) == row["value"],
                f"{label} rank {rank}: value {row['value']} does not decode from {row['value_hex']}",
            )
            if previous is not None:
                previous_value, previous_flat = previous
                report.require(
                    abs(value) < abs(previous_value)
                    or (abs(value) == abs(previous_value) and flat > previous_flat),
                    f"{label} rank {rank}: ordering violates descending magnitude with "
                    "ties broken toward the lower index",
                )
            previous = (value, flat)
        return rows[0]

    hidden = read_tsv(directory / "hidden.tsv")
    hidden_top = read_tsv(directory / "hidden_top.tsv")
    report.require(
        len(hidden) == NUM_LAYERS * EXPECTED_POSITIONS,
        f"hidden has {len(hidden)} rows, expected {NUM_LAYERS * EXPECTED_POSITIONS}",
    )
    report.require(
        len(hidden_top) == NUM_LAYERS * EXPECTED_POSITIONS * ATTRIBUTION_TOP_K,
        f"hidden_top has {len(hidden_top)} rows, "
        f"expected {NUM_LAYERS * EXPECTED_POSITIONS * ATTRIBUTION_TOP_K}",
    )
    hidden_heads = {}
    for row in hidden_top:
        hidden_heads.setdefault((int(row["layer"]), int(row["position"])), []).append(row)
    for offset, row in enumerate(hidden):
        layer, position = divmod(offset, EXPECTED_POSITIONS)
        key = f"hidden layer {layer} position {position}"
        report.require(
            int(row["layer"]) == layer and int(row["position"]) == position,
            f"hidden row {offset} carries layer {row['layer']} position {row['position']}",
        )
        report.require(
            row["stage"] == stage_of.get(str(position)),
            f"{key}: stage {row['stage']} disagrees with positions.tsv",
        )
        report.require(
            len(row["sha256"]) == 64 and all(c in "0123456789abcdef" for c in row["sha256"]),
            f"{key}: malformed digest",
        )
        extreme = float_from_hex(row["max_abs_hex"])
        report.require(
            repr(extreme) == row["max_abs"],
            f"{key}: max_abs {row['max_abs']} does not decode from {row['max_abs_hex']}",
        )
        norm = float(row["l2_norm"])
        # A vector's Euclidean norm is at least the magnitude of its largest
        # component. This is exact rather than approximate, so it needs no
        # tolerance and still fails on a fabricated norm or extremum.
        report.require(
            norm >= abs(extreme),
            f"{key}: the recorded norm {norm} is below the largest component magnitude {abs(extreme)}",
        )
        head = check_head(
            hidden_heads.get((layer, position), []), key, HIDDEN_SIZE, lambda entry: int(entry["lane"])
        )
        if head is not None:
            report.require(
                head["lane"] == row["max_abs_lane"] and head["value_hex"] == row["max_abs_hex"],
                f"{key}: the rank 0 head entry disagrees with the recorded extremum",
            )
    report.note(
        f"hidden: {len(hidden)} slices over {NUM_LAYERS} layers x {EXPECTED_POSITIONS} positions, "
        f"{len(hidden_top)} head rows"
    )

    # --- attribution: per-layer post-RoPE K and V ---------------------------
    cache = read_tsv(directory / "cache.tsv")
    cache_top = read_tsv(directory / "cache_top.tsv")
    expected_cache_rows = NUM_LAYERS * len(CACHE_TENSORS) * EXPECTED_POSITIONS
    report.require(
        len(cache) == expected_cache_rows, f"cache has {len(cache)} rows, expected {expected_cache_rows}"
    )
    report.require(
        len(cache_top) == expected_cache_rows * ATTRIBUTION_TOP_K,
        f"cache_top has {len(cache_top)} rows, expected {expected_cache_rows * ATTRIBUTION_TOP_K}",
    )
    cache_heads = {}
    for row in cache_top:
        cache_heads.setdefault(
            (int(row["layer"]), row["tensor"], int(row["position"])), []
        ).append(row)
    seen_tensors = set()
    for offset, row in enumerate(cache):
        layer, remainder = divmod(offset, len(CACHE_TENSORS) * EXPECTED_POSITIONS)
        index, position = divmod(remainder, EXPECTED_POSITIONS)
        name = CACHE_TENSORS[index]
        key = f"cache layer {layer} {name} position {position}"
        seen_tensors.add(row["tensor"])
        report.require(
            int(row["layer"]) == layer and row["tensor"] == name and int(row["position"]) == position,
            f"cache row {offset} carries layer {row['layer']} {row['tensor']} position {row['position']}",
        )
        report.require(
            row["stage"] == stage_of.get(str(position)),
            f"{key}: stage {row['stage']} disagrees with positions.tsv",
        )
        report.require(
            len(row["sha256"]) == 64 and all(c in "0123456789abcdef" for c in row["sha256"]),
            f"{key}: malformed digest",
        )
        extreme = float_from_hex(row["max_abs_hex"])
        report.require(
            repr(extreme) == row["max_abs"],
            f"{key}: max_abs {row['max_abs']} does not decode from {row['max_abs_hex']}",
        )
        report.require(
            float(row["l2_norm"]) >= abs(extreme),
            f"{key}: the recorded norm is below the largest component magnitude",
        )
        head = check_head(
            cache_heads.get((layer, name, position), []),
            key,
            KV_HEADS * HEAD_DIM,
            lambda entry: int(entry["head"]) * HEAD_DIM + int(entry["lane"]),
        )
        if head is not None:
            report.require(
                head["head"] == row["max_abs_head"]
                and head["lane"] == row["max_abs_lane"]
                and head["value_hex"] == row["max_abs_hex"],
                f"{key}: the rank 0 head entry disagrees with the recorded extremum",
            )
    report.require(
        seen_tensors == set(CACHE_TENSORS),
        f"cache carries tensors {sorted(seen_tensors)}, expected {sorted(CACHE_TENSORS)}",
    )
    report.note(
        f"cache: {len(cache)} slices over {NUM_LAYERS} layers x {len(CACHE_TENSORS)} tensors x "
        f"{EXPECTED_POSITIONS} positions, {len(cache_top)} head rows"
    )

    # --- attribution: the rotary rows ---------------------------------------
    # Two exact structural properties of the pinned construction, so neither
    # needs a threshold. `emb = cat((freqs, freqs))` duplicates the 64-wide
    # frequency block across the 128-wide head, and position 0's angle is zero.
    rotary = read_tsv(directory / "rotary.tsv")
    report.require(
        len(rotary) == EXPECTED_POSITIONS * HEAD_DIM,
        f"rotary has {len(rotary)} rows, expected {EXPECTED_POSITIONS * HEAD_DIM}",
    )
    rotary_by_position = {}
    for offset, row in enumerate(rotary):
        position, lane = divmod(offset, HEAD_DIM)
        report.require(
            int(row["position"]) == position and int(row["lane"]) == lane,
            f"rotary row {offset} carries position {row['position']} lane {row['lane']}",
        )
        for name in ("cos", "sin"):
            value = float_from_hex(row[f"{name}_hex"])
            report.require(
                repr(value) == row[name],
                f"rotary position {position} lane {lane}: {name} {row[name]} does not decode "
                f"from {row[f'{name}_hex']}",
            )
        rotary_by_position.setdefault(position, []).append(row)
    halves = 0
    for position, rows in rotary_by_position.items():
        if len(rows) != HEAD_DIM:
            continue
        for lane in range(HEAD_DIM // 2):
            report.require(
                rows[lane]["cos_hex"] == rows[lane + HEAD_DIM // 2]["cos_hex"]
                and rows[lane]["sin_hex"] == rows[lane + HEAD_DIM // 2]["sin_hex"],
                f"rotary position {position} lane {lane}: the two halves of the table disagree, "
                "which the pinned `cat((freqs, freqs))` construction cannot produce",
            )
            halves += 1
    zero = rotary_by_position.get(0, [])
    report.require(
        all(row["cos_hex"] == "0x3f800000" and row["sin_hex"] == "0x00000000" for row in zero),
        "rotary position 0 is not exactly cos 1.0 and sin 0.0 on every lane",
    )
    report.note(
        f"rotary: {len(rotary)} rows over {len(rotary_by_position)} positions, "
        f"{halves} half-duplication pairs checked"
    )

    # --- attribution: the additive causal mask ------------------------------
    mask = read_tsv(directory / "mask.tsv")
    host = read_kv(directory / "host.tsv")
    report.require(
        host.get("host.mask.masked_entry") == MASK_MASKED_ENTRY
        and host.get("host.mask.attended_entry") == MASK_ATTENDED_ENTRY,
        "host.tsv does not carry L4's two mask values",
    )
    by_pass = {}
    for row in mask:
        report.require(
            row["value_hex"] in (MASK_MASKED_ENTRY, MASK_ATTENDED_ENTRY),
            f"mask {row['pass']} ({row['query_position']}, {row['key_position']}) carries "
            f"{row['value_hex']}, which is neither of L4's two values",
        )
        # Causality is exact: a query attends to a key at or below its own
        # position and to no other. A record that admitted one more would be a
        # different computation, not a looser one.
        attended = row["value_hex"] == MASK_ATTENDED_ENTRY
        report.require(
            attended == (int(row["key_position"]) <= int(row["query_position"])),
            f"mask {row['pass']} ({row['query_position']}, {row['key_position']}) is "
            f"{'attended' if attended else 'masked'}, which causality forbids",
        )
        by_pass.setdefault(row["pass"], []).append(row)
    expected_passes = ["prefill"] + [f"decode-{step + 1}" for step in range(DECODE_STEPS)]
    report.require(
        sorted(by_pass) == sorted(expected_passes),
        f"mask carries passes {sorted(by_pass)}, expected {sorted(expected_passes)}",
    )
    for label in expected_passes:
        rows = by_pass.get(label, [])
        shape = host.get(f"host.mask.{label}.shape")
        if not report.require(shape is not None, f"host.tsv has no shape for the {label} mask"):
            continue
        queries, keys = (int(part) for part in shape.strip("[]").split(","))
        report.require(
            len(rows) == queries * keys,
            f"mask {label} has {len(rows)} rows, its declared shape {shape} is {queries * keys}",
        )
        report.require(
            int(host.get(f"host.mask.{label}.bytes", "-1")) == queries * keys * 4,
            f"host.tsv's byte count for the {label} mask disagrees with its declared shape",
        )
        report.require(
            int(host.get(f"host.mask.{label}.attended_entries", "-1"))
            == sum(1 for row in rows if row["value_hex"] == MASK_ATTENDED_ENTRY),
            f"host.tsv's attended count for the {label} mask disagrees with mask.tsv",
        )
    report.note(f"mask: {len(mask)} entries over {len(by_pass)} passes, all two-valued and causal")

    # --- attribution: the remaining host computations -----------------------
    report.require(
        host.get("weights.widened.tensor_count") == str(WEIGHT_TENSOR_COUNT),
        f"host.tsv claims {host.get('weights.widened.tensor_count')} widened tensors, "
        f"L1's inventory records {WEIGHT_TENSOR_COUNT}",
    )
    report.require(
        host.get("weights.widened.bytes") == str(WIDENED_WEIGHT_BYTES),
        f"host.tsv claims {host.get('weights.widened.bytes')} widened bytes, "
        f"L1's F32 weight budget is {WIDENED_WEIGHT_BYTES}",
    )
    report.require(
        host.get("weights.widening_bit_exact_tensors") == str(WEIGHT_TENSOR_COUNT),
        f"only {host.get('weights.widening_bit_exact_tensors')} of {WEIGHT_TENSOR_COUNT} widenings "
        "are bit-exact; L1 records the BF16-to-F32 widening as exact for every finite value",
    )
    report.require(
        host.get("host.tokens.count") == str(EXPECTED_POSITIONS)
        and host.get("host.tokens.bytes") == str(EXPECTED_POSITIONS * 4),
        "host.tsv's token-ID population disagrees with the row's 18 positions",
    )
    for key in (
        "host.rotary.cos_sha256",
        "host.rotary.sin_sha256",
        "host.tokens.sha256",
        "weights.widened.sha256",
    ):
        report.require(
            len(host.get(key, "")) == 64,
            f"host.tsv carries no well-formed digest at {key}",
        )
    report.note(f"host: {len(host)} keys covering the four host computations")

    # --- optional: the regenerable attribution bytes ------------------------
    if attribution_dir is not None:
        present = 0
        for layer in range(NUM_LAYERS):
            path = attribution_dir / "hidden" / f"layer-{layer:02d}.f32le.bin"
            if not path.exists():
                continue
            present += 1
            raw = path.read_bytes()
            if not report.require(
                len(raw) == EXPECTED_POSITIONS * POSITION_SLICE_BYTES,
                f"{path.name} is {len(raw)} bytes, expected {EXPECTED_POSITIONS * POSITION_SLICE_BYTES}",
            ):
                continue
            for position in range(EXPECTED_POSITIONS):
                start = position * POSITION_SLICE_BYTES
                recorded = hidden[layer * EXPECTED_POSITIONS + position]["sha256"]
                report.require(
                    hashlib.sha256(raw[start : start + POSITION_SLICE_BYTES]).hexdigest() == recorded,
                    f"{path.name} position {position} does not hash to its recorded digest",
                )
        report.note(f"hidden bytes: {present} of {NUM_LAYERS} layer files present and re-hashed")

        cache_present = 0
        for offset, row in enumerate(cache):
            path = (
                attribution_dir / "cache" / f"layer-{int(row['layer']):02d}-{row['tensor']}.f32le.bin"
            )
            if not path.exists():
                continue
            raw = path.read_bytes()
            if not report.require(
                len(raw) == KV_HEADS * EXPECTED_POSITIONS * HEAD_DIM * 4,
                f"{path.name} is {len(raw)} bytes, expected {KV_HEADS * EXPECTED_POSITIONS * HEAD_DIM * 4}",
            ):
                continue
            cache_present += 1
            # The file keeps the head-major `[8, 18, 128]` layout, so a position
            # is a strided gather rather than a byte range; this is that gather.
            position = int(row["position"])
            gathered = b"".join(
                raw[
                    (head * EXPECTED_POSITIONS + position) * HEAD_DIM * 4 : (
                        head * EXPECTED_POSITIONS + position + 1
                    )
                    * HEAD_DIM
                    * 4
                ]
                for head in range(KV_HEADS)
            )
            report.require(
                hashlib.sha256(gathered).hexdigest() == row["sha256"],
                f"{path.name} position {position} does not hash to its recorded digest",
            )
        report.note(f"cache bytes: {cache_present} of {len(cache)} slices re-hashed from present files")

        if present == 0 and cache_present == 0:
            report.require(
                False, f"--attribution-dir {attribution_dir} was given but held no retained bytes"
            )

    # --- optional: the regenerable logit bytes ------------------------------
    if logit_dir is not None:
        present = 0
        for row in positions:
            path = logit_dir / f"position-{int(row['position']):02d}.f32le.bin"
            if not path.exists():
                continue
            present += 1
            raw = path.read_bytes()
            report.require(
                len(raw) == VOCAB_SIZE * 4,
                f"{path.name} is {len(raw)} bytes, expected {VOCAB_SIZE * 4}",
            )
            report.require(
                hashlib.sha256(raw).hexdigest() == row["logits_sha256"],
                f"{path.name} does not hash to the digest recorded for position {row['position']}",
            )
        report.note(f"logit bytes: {present} of {EXPECTED_POSITIONS} position files present and re-hashed")
        if present == 0:
            report.require(False, f"--logit-dir {logit_dir} was given but held no position files")

    # --- verdict ------------------------------------------------------------
    print(f"\n{report.checks} checks run over {directory}")
    if report.failures:
        print(f"{len(report.failures)} FAILED:\n", file=sys.stderr)
        for failure in report.failures:
            print(f"  FAIL: {failure}", file=sys.stderr)
        return 5
    print("all checks passed")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Verify a retained C1 conformance and attribution fixture."
    )
    parser.add_argument("directory", type=Path, help="a results/<slug> directory")
    parser.add_argument(
        "--logit-dir",
        type=Path,
        default=None,
        help="also re-hash the regenerable F32 logit bytes in this directory",
    )
    parser.add_argument(
        "--attribution-dir",
        type=Path,
        default=None,
        help="also re-hash the regenerable hidden-state and cache bytes in this directory",
    )
    arguments = parser.parse_args()
    return verify(arguments.directory, arguments.logit_dir, arguments.attribution_dir)


if __name__ == "__main__":
    raise SystemExit(main())
