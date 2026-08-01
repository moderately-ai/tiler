#!/usr/bin/env python3
"""Weight-space reconstruction error of candidate strict-affine profiles on Qwen3-0.6B-Base.

Stage A of the first-quantized-LM-profile evidence. It reads the pinned checkpoint's
own `model.safetensors`, widens each BF16 weight to F32 exactly, round-trips it through
each candidate quantization profile using the *registered* strict-affine conversion
contract, and records the reconstruction error and the exact byte cost.

No model is executed here and no accuracy claim about the model is made; Stage B
(`model_error.py`) owns the model-visible observable. This stage exists because a
weight-space error large enough to be visible here bounds nothing on its own, while a
profile's byte cost and its scale/zero-point value ranges are exact facts that the
Metal honourability derivation and the memory arithmetic both need.

Fail-closed: the checkpoint digest is verified against the workload profile's manifest
before any number is computed.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import struct
import sys
from dataclasses import dataclass
from pathlib import Path

import numpy as np

# From docs/research/program-planning/first-metal-lm-workload.md, the pinned manifest.
REPO_ID = "Qwen/Qwen3-0.6B-Base"
REVISION = "da87bfb608c14b7cf20ba1ce41287e8de496c0cd"
WEIGHTS_SHA256 = "cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba"
WEIGHTS_BYTES = 1_192_135_096

# The six weighted-projection roles L2/L3 resolved to contraction index structure 1,
# plus the tied embedding matrix. `embed_tokens` is listed separately because it is
# both a gather operand and the vocabulary-projection contraction operand.
PROJECTION_SUFFIXES = (
    "self_attn.q_proj.weight",
    "self_attn.k_proj.weight",
    "self_attn.v_proj.weight",
    "self_attn.o_proj.weight",
    "mlp.gate_proj.weight",
    "mlp.up_proj.weight",
    "mlp.down_proj.weight",
)
EMBEDDING_NAME = "model.embed_tokens.weight"


def resolve_weights_path() -> Path:
    """Returns the cached `model.safetensors` for the pinned revision, or exits."""
    override = os.environ.get("TILER_QWEN3_SAFETENSORS")
    if override:
        return Path(override)
    home = Path(os.environ.get("HF_HOME", Path.home() / ".cache" / "huggingface"))
    hub = home / "hub" if (home / "hub").is_dir() else home
    path = (
        hub
        / f"models--{REPO_ID.replace('/', '--')}"
        / "snapshots"
        / REVISION
        / "model.safetensors"
    )
    if not path.exists():
        sys.exit(
            f"missing checkpoint {path}\n"
            f"acquire it with: hf download {REPO_ID} --revision {REVISION}\n"
            "or point TILER_QWEN3_SAFETENSORS at the file"
        )
    return path


def verify_weights(path: Path) -> None:
    """Hashes the checkpoint and stops on any mismatch with the pinned manifest."""
    size = path.stat().st_size
    if size != WEIGHTS_BYTES:
        sys.exit(f"checkpoint size {size} != pinned {WEIGHTS_BYTES}")
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 22), b""):
            digest.update(chunk)
    if digest.hexdigest() != WEIGHTS_SHA256:
        sys.exit(f"checkpoint sha256 {digest.hexdigest()} != pinned {WEIGHTS_SHA256}")


def read_header(path: Path) -> tuple[dict, int]:
    """Returns the safetensors header JSON and the byte offset of the data section."""
    with path.open("rb") as handle:
        (length,) = struct.unpack("<Q", handle.read(8))
        header = json.loads(handle.read(length))
    return header, 8 + length


def load_f32(path: Path, header: dict, base: int, name: str) -> np.ndarray:
    """Widens one BF16 tensor to F32 exactly, by the shift that defines BF16."""
    entry = header[name]
    if entry["dtype"] != "BF16":
        sys.exit(f"{name} is {entry['dtype']}, expected BF16")
    start, end = entry["data_offsets"]
    raw = np.memmap(
        path, dtype=np.uint16, mode="r", offset=base + start, shape=((end - start) // 2,)
    )
    widened = (np.asarray(raw, dtype=np.uint32) << 16).view(np.float32)
    return widened.reshape(entry["shape"])


@dataclass(frozen=True)
class Profile:
    """One candidate quantization profile: code width and parameter-map granularity."""

    name: str
    bits: int
    # None = per tensor; "row" = per output channel (axis 0); an int = per contiguous
    # group of that many elements along the contracted axis (axis 1).
    granularity: object
    # Ingestion-side calibration: None keeps the group's exact min and max; a fraction
    # in (0, 1) clips each group to that two-sided quantile before calibrating, which
    # trades a saturated tail for a finer step.
    clip_quantile: object = None

    @property
    def code_max(self) -> int:
        return (1 << self.bits) - 1


def grouped(values: np.ndarray, profile: Profile) -> np.ndarray:
    """Reshapes a 2-D weight into (groups, elements) for the profile's granularity."""
    if profile.granularity is None:
        return values.reshape(1, -1)
    if profile.granularity == "row":
        return values.reshape(values.shape[0], -1)
    group = int(profile.granularity)
    if values.shape[1] % group != 0:
        sys.exit(f"group {group} does not divide contracted extent {values.shape[1]}")
    return values.reshape(-1, group)


def round_ties_even(values: np.ndarray) -> np.ndarray:
    """IEEE round-to-nearest-ties-to-even, the registered encode rounding rule."""
    return np.rint(values)


def roundtrip(values: np.ndarray, profile: Profile) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Quantizes and dequantizes under the registered strict-affine contract.

    Returns the reconstruction, the per-group scales, and the per-group zero points.
    Calibration is min-max, which is an ingestion-side choice and not a Tiler
    semantic; the *conversion* below is exactly what `tiler::strict-affine@1`
    registers -- f32 divide, add zero point, clamp to the inclusive code domain,
    round to nearest ties-to-even; then widen code and zero point to i32,
    subtract, convert to f32, multiply by the scale.
    """
    blocks = grouped(values, profile).astype(np.float64, copy=False)
    if profile.clip_quantile is None:
        lo = blocks.min(axis=1, keepdims=True)
        hi = blocks.max(axis=1, keepdims=True)
    else:
        tail = (1.0 - float(profile.clip_quantile)) / 2.0
        lo = np.quantile(blocks, tail, axis=1, keepdims=True)
        hi = np.quantile(blocks, 1.0 - tail, axis=1, keepdims=True)
    # A degenerate group (all elements equal) still needs a positive finite scale,
    # which the registered `positive_finite_scalar_predicate` requires.
    span = np.maximum(hi, 0.0) - np.minimum(lo, 0.0)
    scale = np.where(span > 0.0, span / profile.code_max, np.float64(2.0**-24))
    scale = scale.astype(np.float32).astype(np.float64)
    zero = np.clip(round_ties_even(-np.minimum(lo, 0.0) / scale), 0, profile.code_max)

    codes = np.clip(
        blocks.astype(np.float32) / scale.astype(np.float32) + zero.astype(np.float32),
        0.0,
        float(profile.code_max),
    )
    codes = round_ties_even(codes).astype(np.int32)
    decoded = (codes - zero.astype(np.int32)).astype(np.float32) * scale.astype(np.float32)
    return decoded.reshape(values.shape), scale.astype(np.float32), zero.astype(np.int32)


def profile_bytes(elements: int, groups: int, profile: Profile) -> int:
    """Exact stored bytes: packed codes plus one F32 scale and one code-width zero point."""
    code_bytes = (elements * profile.bits + 7) // 8
    # The zero point's declared component type is the code type, so it costs the code
    # width, not a byte, when packed the same way the codes are.
    zero_bytes = (groups * profile.bits + 7) // 8
    return code_bytes + 4 * groups + zero_bytes


PROFILES = (
    Profile("per-tensor-u4", 4, None),
    Profile("per-tensor-u8", 8, None),
    Profile("per-channel-u4", 4, "row"),
    Profile("per-channel-u8", 8, "row"),
    Profile("per-group32-u4", 4, 32),
    Profile("per-group64-u4", 4, 64),
    Profile("per-group128-u4", 4, 128),
    Profile("per-group128-u8", 8, 128),
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True, help="result directory to write")
    args = parser.parse_args()

    path = resolve_weights_path()
    verify_weights(path)
    header, base = read_header(path)

    names = [EMBEDDING_NAME] + sorted(
        name
        for name in header
        if name != "__metadata__" and name.endswith(PROJECTION_SUFFIXES)
    )

    args.out.mkdir(parents=True, exist_ok=True)
    rows = []
    totals: dict[str, dict[str, float]] = {}
    for name in names:
        weight = load_f32(path, header, base, name)
        reference = weight.astype(np.float64)
        denominator = float(np.sqrt((reference**2).sum()))
        for profile in PROFILES:
            decoded, scale, _zero = roundtrip(weight, profile)
            residual = reference - decoded.astype(np.float64)
            relative = float(np.sqrt((residual**2).sum())) / denominator
            groups = scale.size
            bucket = totals.setdefault(
                profile.name,
                {"sq_error": 0.0, "sq_reference": 0.0, "bytes": 0.0, "elements": 0.0,
                 "min_scale": float("inf"), "max_scale": 0.0},
            )
            bucket["sq_error"] += float((residual**2).sum())
            bucket["sq_reference"] += float((reference**2).sum())
            bucket["bytes"] += profile_bytes(weight.size, groups, profile)
            bucket["elements"] += weight.size
            bucket["min_scale"] = min(bucket["min_scale"], float(scale.min()))
            bucket["max_scale"] = max(bucket["max_scale"], float(scale.max()))
            rows.append(
                (
                    name,
                    profile.name,
                    "x".join(str(d) for d in weight.shape),
                    groups,
                    f"{relative:.6e}",
                    f"{float(np.abs(residual).max()):.6e}",
                    f"{float(scale.min()):.6e}",
                    f"{float(scale.max()):.6e}",
                    profile_bytes(weight.size, groups, profile),
                )
            )
        del weight, reference

    with (args.out / "per-tensor-error.tsv").open("w") as handle:
        handle.write(
            "tensor\tprofile\tshape\tgroups\trelative_frobenius_error\t"
            "max_abs_error\tmin_scale\tmax_scale\tstored_bytes\n"
        )
        for row in rows:
            handle.write("\t".join(str(field) for field in row) + "\n")

    with (args.out / "profile-summary.tsv").open("w") as handle:
        handle.write(
            "profile\telements\trelative_frobenius_error\tstored_bytes\t"
            "bytes_vs_f32\tbits_per_element\tmin_scale\tmax_scale\n"
        )
        for profile in PROFILES:
            bucket = totals[profile.name]
            elements = bucket["elements"]
            f32_bytes = 4.0 * elements
            handle.write(
                "\t".join(
                    [
                        profile.name,
                        f"{int(elements)}",
                        f"{np.sqrt(bucket['sq_error'] / bucket['sq_reference']):.6e}",
                        f"{int(bucket['bytes'])}",
                        f"{bucket['bytes'] / f32_bytes:.6f}",
                        f"{8.0 * bucket['bytes'] / elements:.4f}",
                        f"{bucket['min_scale']:.6e}",
                        f"{bucket['max_scale']:.6e}",
                    ]
                )
                + "\n"
            )

    with (args.out / "environment.tsv").open("w") as handle:
        handle.write("key\tvalue\n")
        for key, value in (
            ("repo_id", REPO_ID),
            ("revision", REVISION),
            ("weights_sha256", WEIGHTS_SHA256),
            ("weights_bytes", str(WEIGHTS_BYTES)),
            ("tensors_measured", str(len(names))),
            ("python", platform.python_version()),
            ("numpy", np.__version__),
            ("platform", platform.platform()),
            ("machine", platform.machine()),
        ):
            handle.write(f"{key}\t{value}\n")

    print(f"wrote {len(rows)} rows over {len(names)} tensors to {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
