#!/usr/bin/env python3
"""Verify a retained C1 conformance fixture without owning a model.

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
    argmax of the position that produced it;
  * `--logit-dir` additionally re-hashes the regenerable F32 logit bytes against
    the per-position digests, when they happen to be present locally.

Every check names its population and counts it, so "nothing failed" is
distinguishable from "nothing ran".

    uv run --locked python verify_fixture.py results/<slug>
    uv run --locked python verify_fixture.py results/<slug> --logit-dir local-work/logits
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

RESULT_FILES = ["environment.tsv", "sequence.tsv", "positions.tsv", "top32.tsv", "envelope.tsv"]


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


def verify(directory: Path, logit_dir: Path | None) -> int:
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
    report.note(f"environment: {len(environment)} keys, 8 asserted")

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
    parser = argparse.ArgumentParser(description="Verify a retained C1 conformance fixture.")
    parser.add_argument("directory", type=Path, help="a results/<slug> directory")
    parser.add_argument(
        "--logit-dir",
        type=Path,
        default=None,
        help="also re-hash the regenerable F32 logit bytes in this directory",
    )
    arguments = parser.parse_args()
    return verify(arguments.directory, arguments.logit_dir)


if __name__ == "__main__":
    raise SystemExit(main())
