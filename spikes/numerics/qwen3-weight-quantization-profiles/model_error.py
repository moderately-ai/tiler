#!/usr/bin/env python3
"""Model-visible effect of candidate quantization profiles on the C1 conformance row.

Stage B. Stage A (`weight_error.py`) measures weight-space reconstruction error, which
bounds nothing about the model on its own: a projection's error is filtered by every
later stage, and 28 residual blocks can amplify or cancel it. This stage runs the
workload profile's own C1 row -- the fixed 10-token prompt and 8-step greedy decode --
once per candidate and reports what a caller would actually observe: the emitted token
sequence, the greedy token at every position, and the logit deviation from the F32
baseline computed in this same environment.

This is a *differential* experiment. Its F32 baseline is computed here, in this
environment, and every quantized reading is compared against that baseline rather than
against the retained C1 fixture. The baseline's own 18-token sequence and per-position
greedy tokens are cross-checked against the retained fixture so that the comparison is
anchored, but the absolute logit bits are not claimed to reproduce it: the fixture
pins `transformers` 4.51.0 and `torch` 2.6.0, and this harness runs whatever the host
interpreter already provides. `environment.tsv` records exactly what ran.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import inspect
import platform
import sys
from pathlib import Path

import numpy as np
import torch

from weight_error import (
    EMBEDDING_NAME,
    PROJECTION_SUFFIXES,
    REPO_ID,
    REVISION,
    Profile,
    grouped,
    resolve_weights_path,
    roundtrip,
    verify_weights,
)

# From docs/research/program-planning/first-metal-lm-workload.md, row C1.
C1_PROMPT_IDS = [785, 3974, 13876, 38835, 34208, 916, 279, 15678, 5562, 13]
C1_DECODE_STEPS = 8
EOS_TOKEN_ID = 151643

# Named candidates carried into the model-level stage. The three per-tensor and
# per-channel U4/U8 forms are the registered and the first-widening maps; the group
# forms bracket the block sizes that divide every contracted extent in the workload
# (1024, 2048, 3072 are all multiples of 32, 64, and 128).
STAGE_B_PROFILES = (
    # The control, and the reason every quantized candidate needs one: the checkpoint's
    # own storage width. Widening BF16 to F32 costs 2x the bytes for zero information,
    # so narrowing back to BF16 is exact against the checkpoint and halves the weight
    # budget. It is not quantization and needs no scheme, no map, and no zero point.
    Profile("bf16-storage-control", 16, None),
    Profile("per-tensor-u4", 4, None),
    Profile("per-tensor-u8", 8, None),
    Profile("per-channel-u4", 4, "row"),
    Profile("per-channel-u8", 8, "row"),
    Profile("per-group32-u4", 4, 32),
    Profile("per-group128-u4", 4, 128),
    Profile("per-group128-u8", 8, 128),
)
BF16_CONTROL = "bf16-storage-control"


def greedy_token(logits: np.ndarray) -> tuple[int, float, float, int]:
    """Applies the profile's declared tie policy: lowest index attaining the maximum."""
    maximum = logits.max()
    attaining = np.flatnonzero(logits == maximum)
    token = int(attaining[0])
    ordered = np.sort(logits)[::-1]
    return token, float(ordered[0]), float(ordered[1]), int(attaining.size)


def run_c1(model, device: str) -> tuple[list[int], np.ndarray]:
    """Runs the C1 row greedily and returns the 18-token sequence and 18 logit rows."""
    with torch.no_grad():
        ids = torch.tensor([C1_PROMPT_IDS], dtype=torch.long, device=device)
        out = model(input_ids=ids, use_cache=True, logits_to_keep=0)
        rows = [out.logits[0].to(torch.float32).cpu().numpy()]
        cache = out.past_key_values
        sequence = list(C1_PROMPT_IDS)
        for _ in range(C1_DECODE_STEPS):
            token, _, _, _ = greedy_token(rows[-1][-1])
            sequence.append(token)
            if token == EOS_TOKEN_ID:
                break
            step = torch.tensor([[token]], dtype=torch.long, device=device)
            out = model(input_ids=step, past_key_values=cache, use_cache=True)
            cache = out.past_key_values
            rows.append(out.logits[0].to(torch.float32).cpu().numpy())
    return sequence, np.concatenate(rows, axis=0)


def target_names(model, include_embedding: bool) -> list[str]:
    """Names the parameters a profile replaces: the weighted projections, plus the
    tied embedding matrix when the variant includes it."""
    names = [
        name
        for name, _ in model.named_parameters()
        if name.endswith(PROJECTION_SUFFIXES) or f"model.{name}".endswith(PROJECTION_SUFFIXES)
    ]
    if include_embedding:
        names += [
            name
            for name, _ in model.named_parameters()
            if name.endswith("embed_tokens.weight")
        ]
    return sorted(names)


def apply_profile(model, originals: dict[str, torch.Tensor], names: list[str], profile: Profile) -> int:
    """Replaces each named weight with its quantize-dequantize round trip.

    Returns the exact stored bytes of the replaced set under this profile.
    """
    stored = 0
    parameters = dict(model.named_parameters())
    with torch.no_grad():
        for name in names:
            parameter = parameters[name]
            original = originals[name]
            if profile.name == BF16_CONTROL:
                # No scheme, no scale, no zero point: one storage width.
                parameter.copy_(original.to(torch.bfloat16).to(torch.float32))
                stored += 2 * int(original.numel())
                continue
            reference = original.numpy()
            decoded, scale, _zero = roundtrip(reference, profile)
            parameter.copy_(torch.from_numpy(decoded))
            groups = scale.size
            code_bits = reference.size * profile.bits
            stored += (code_bits + 7) // 8 + 4 * groups + (groups * profile.bits + 7) // 8
    return stored


def restore(model, originals: dict[str, torch.Tensor], names: list[str]) -> None:
    with torch.no_grad():
        parameters = dict(model.named_parameters())
        for name in names:
            parameters[name].copy_(originals[name])


def read_fixture_sequence(root: Path) -> list[int] | None:
    """Reads the retained C1 fixture's 18-token sequence, when the record is present."""
    path = root / "sequence.tsv"
    if not path.exists():
        return None
    with path.open() as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    key = next((k for k in rows[0] if "token_id" in k or k == "token"), None)
    if key is None:
        return None
    return [int(row[key]) for row in rows]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument(
        "--fixture",
        type=Path,
        default=Path(__file__).resolve().parents[2]
        / "program-planning"
        / "qwen3-conformance-fixture"
        / "results"
        / "2026-07-31-c1-conformance-qwen3-0.6b-base-da87bfb6-f32-eager-cpu-torch2.6.0-transformers4.51.0",
        help="retained C1 fixture record used to anchor the F32 baseline",
    )
    args = parser.parse_args()

    verify_weights(resolve_weights_path())

    import transformers
    from transformers import AutoModelForCausalLM

    torch.set_num_threads(1)
    torch.manual_seed(0)
    model = AutoModelForCausalLM.from_pretrained(
        REPO_ID,
        revision=REVISION,
        dtype=torch.float32,
        attn_implementation="eager",
    )
    model.eval()

    reference_source = Path(inspect.getsourcefile(type(model)))
    reference_digest = hashlib.sha256(reference_source.read_bytes()).hexdigest()

    baseline_sequence, baseline_logits = run_c1(model, "cpu")
    fixture_sequence = read_fixture_sequence(args.fixture)
    anchored = fixture_sequence is not None and fixture_sequence == baseline_sequence

    originals = {
        name: parameter.detach().clone().cpu()
        for name, parameter in model.named_parameters()
        if name.endswith(PROJECTION_SUFFIXES) or name.endswith("embed_tokens.weight")
    }
    f32_bytes = 4 * sum(int(t.numel()) for t in originals.values())
    # Everything a profile never touches -- the 57 RMS-norm weight vectors -- stays F32,
    # and is carried so the reported model total is the model's, not the subset's.
    untouched_bytes = 4 * sum(
        int(parameter.numel())
        for name, parameter in model.named_parameters()
        if name not in originals
    )

    args.out.mkdir(parents=True, exist_ok=True)
    rows = []
    for include_embedding in (False, True):
        names = target_names(model, include_embedding)
        replaced_elements = sum(int(originals[name].numel()) for name in names)
        for profile in STAGE_B_PROFILES:
            stored = apply_profile(model, originals, names, profile)
            sequence, logits = run_c1(model, "cpu")
            restore(model, originals, names)

            positions = min(logits.shape[0], baseline_logits.shape[0])
            deviation = np.abs(
                logits[:positions].astype(np.float64)
                - baseline_logits[:positions].astype(np.float64)
            )
            greedy_matches = 0
            minimum_gap = float("inf")
            top32_deviation = 0.0
            for index in range(positions):
                token, top, runner_up, _ = greedy_token(logits[index])
                base_token, _, _, _ = greedy_token(baseline_logits[index])
                greedy_matches += int(token == base_token)
                minimum_gap = min(minimum_gap, top - runner_up)
                # Restricted to the entries the retained fixture also keeps, so the
                # reading is comparable with L1's measured F32 sensitivity envelope.
                head = np.argsort(baseline_logits[index])[::-1][:32]
                top32_deviation = max(top32_deviation, float(deviation[index][head].max()))

            # Model-total residency: replaced tensors at their profile cost, plus
            # everything not replaced at F32.
            unreplaced = f32_bytes - 4 * replaced_elements
            rows.append(
                {
                    "variant": "projections+embedding" if include_embedding else "projections-only",
                    "profile": profile.name,
                    "tensors_replaced": len(names),
                    "elements_replaced": replaced_elements,
                    "replaced_bytes": stored,
                    "model_weight_bytes": stored + unreplaced + untouched_bytes,
                    "model_weight_bytes_vs_f32": (
                        f"{(stored + unreplaced + untouched_bytes) / (f32_bytes + untouched_bytes):.6f}"
                    ),
                    "sequence_matches_baseline": int(sequence == baseline_sequence),
                    "greedy_agreement": f"{greedy_matches}/{positions}",
                    "max_logit_deviation": f"{float(deviation.max()):.6e}",
                    "max_top32_logit_deviation": f"{top32_deviation:.6e}",
                    "median_logit_deviation": f"{float(np.median(deviation)):.6e}",
                    "min_runner_up_gap": f"{minimum_gap:.6e}",
                    "sequence": " ".join(str(t) for t in sequence),
                }
            )
            print(f"{rows[-1]['variant']:22s} {profile.name:16s} "
                  f"seq_match={rows[-1]['sequence_matches_baseline']} "
                  f"greedy={rows[-1]['greedy_agreement']} "
                  f"maxdev={rows[-1]['max_logit_deviation']}")

    # The differential readings are only meaningful if restoring a weight set is exact.
    # A drifting restore would make every later profile a comparison against a different
    # model, so this is checked rather than assumed, and it stops rather than warns.
    replay_sequence, replay_logits = run_c1(model, "cpu")
    restore_exact = replay_sequence == baseline_sequence and bool(
        np.array_equal(replay_logits, baseline_logits)
    )
    if not restore_exact:
        sys.exit("restore drifted: the post-run F32 baseline is not bit-identical")

    with (args.out / "model-observable.tsv").open("w") as handle:
        # `csv` writes CRLF by default, which lands a carriage return at the end of
        # every retained line and makes the record fail `git diff --check`.
        writer = csv.DictWriter(
            handle, fieldnames=list(rows[0]), delimiter="\t", lineterminator="\n"
        )
        writer.writeheader()
        writer.writerows(rows)

    with (args.out / "environment.tsv").open("w") as handle:
        handle.write("key\tvalue\n")
        for key, value in (
            ("repo_id", REPO_ID),
            ("revision", REVISION),
            ("row", "C1"),
            ("prompt_ids", " ".join(str(t) for t in C1_PROMPT_IDS)),
            ("decode_steps", str(C1_DECODE_STEPS)),
            ("baseline_sequence", " ".join(str(t) for t in baseline_sequence)),
            ("baseline_anchored_to_retained_fixture", str(anchored)),
            ("restore_bit_exact_after_all_profiles", str(restore_exact)),
            ("retained_fixture_sequence",
             " ".join(str(t) for t in fixture_sequence) if fixture_sequence else "unavailable"),
            ("reference_source", str(reference_source)),
            ("reference_source_sha256", reference_digest),
            ("f32_baseline_weight_bytes", str(f32_bytes)),
            ("python", platform.python_version()),
            ("numpy", np.__version__),
            ("torch", torch.__version__),
            ("transformers", transformers.__version__),
            ("torch_num_threads", str(torch.get_num_threads())),
            ("platform", platform.platform()),
            ("machine", platform.machine()),
        ):
            handle.write(f"{key}\t{value}\n")

    if not anchored:
        print(
            "WARNING: the F32 baseline sequence does not match the retained C1 fixture; "
            "the differential readings stand but are not anchored",
            file=sys.stderr,
        )
    print(f"wrote {len(rows)} rows to {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
