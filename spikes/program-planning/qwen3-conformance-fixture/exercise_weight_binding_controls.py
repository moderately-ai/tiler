#!/usr/bin/env python3
"""Perturb weight-binding subjects while leaving verifier assertions unchanged."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path
from typing import Any


REVISION = "da87bfb608c14b7cf20ba1ce41287e8de496c0cd"
CHECKPOINT_SHA256 = "cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba"


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=True, indent=2, sort_keys=True) + "\n"
    ).encode()


def binding(manifest: dict[str, Any], name: str) -> dict[str, Any]:
    return next(row for row in manifest["bindings"] if row["checkpoint_tensor"] == name)


def swap_k_v(manifest: dict[str, Any]) -> None:
    k_row = binding(manifest, "model.layers.0.self_attn.k_proj.weight")
    v_row = binding(manifest, "model.layers.0.self_attn.v_proj.weight")
    for field in ("qualified_slot", "interface_key", "uses"):
        k_row[field], v_row[field] = v_row[field], k_row[field]


def alter_checkpoint_digest(manifest: dict[str, Any]) -> None:
    manifest["checkpoint"]["sha256"] = "0" * 64


def alter_checkpoint_revision(manifest: dict[str, Any]) -> None:
    manifest["checkpoint"]["revision"] = "0" * 40


def omit_name(manifest: dict[str, Any]) -> None:
    name = "model.layers.0.input_layernorm.weight"
    manifest["bindings"] = [
        row for row in manifest["bindings"] if row["checkpoint_tensor"] != name
    ]


def duplicate_name(manifest: dict[str, Any]) -> None:
    manifest["bindings"].append(dict(binding(manifest, "model.embed_tokens.weight")))


def duplicate_slot(manifest: dict[str, Any]) -> None:
    target = binding(manifest, "model.layers.0.self_attn.v_proj.weight")
    target["qualified_slot"] = "P2.layer-00.W_k"


def alter_shape(manifest: dict[str, Any]) -> None:
    binding(manifest, "model.embed_tokens.weight")["expected_shape"] = [151_935, 1024]


def alter_scalar(manifest: dict[str, Any]) -> None:
    binding(manifest, "model.embed_tokens.weight")["expected_storage_scalar"] = "BF16"


def foreign_name(manifest: dict[str, Any]) -> None:
    binding(manifest, "model.norm.weight")["checkpoint_tensor"] = "model.foreign.weight"


Control = tuple[str, Callable[[dict[str, Any]], None], str]


CONTROLS: tuple[Control, ...] = (
    (
        "same-shape-map-permutation",
        swap_k_v,
        "VERIFY STOP: binding 'model.layers.0.self_attn.k_proj.weight' maps to "
        "'P2.layer-00.W_v' ('W_v'); expected 'P2.layer-00.W_k' ('W_k')",
    ),
    (
        "checkpoint-digest",
        alter_checkpoint_digest,
        "VERIFY STOP: manifest checkpoint sha256 is "
        f"'{'0' * 64}'; expected '{CHECKPOINT_SHA256}'",
    ),
    (
        "checkpoint-revision",
        alter_checkpoint_revision,
        "VERIFY STOP: manifest checkpoint revision is "
        f"'{'0' * 40}'; expected '{REVISION}'",
    ),
    (
        "omission",
        omit_name,
        "VERIFY STOP: missing checkpoint tensor name "
        "'model.layers.0.input_layernorm.weight'; record covers 309 of 310",
    ),
    (
        "duplicate-name",
        duplicate_name,
        "VERIFY STOP: duplicate checkpoint tensor name 'model.embed_tokens.weight'",
    ),
    (
        "duplicate-qualified-slot",
        duplicate_slot,
        "VERIFY STOP: duplicate qualified program slot 'P2.layer-00.W_k'",
    ),
    (
        "shape",
        alter_shape,
        "VERIFY STOP: binding 'model.embed_tokens.weight' expected_shape is "
        "[151935, 1024]; expected [151936, 1024]",
    ),
    (
        "storage-scalar",
        alter_scalar,
        "VERIFY STOP: binding 'model.embed_tokens.weight' expected_storage_scalar is "
        "'BF16'; expected 'F32'",
    ),
    (
        "foreign-name",
        foreign_name,
        "VERIFY STOP: foreign checkpoint tensor name 'model.foreign.weight'",
    ),
)


def invoke(
    verifier: Path, record: Path, checkpoint: Path
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            sys.executable,
            str(verifier),
            str(record),
            "--checkpoint",
            str(checkpoint),
            "--revision",
            REVISION,
        ],
        check=False,
        text=True,
        capture_output=True,
    )


def write_perturbed(record: Path, mutate: Callable[[dict[str, Any]], None]) -> None:
    manifest_path = record / "manifest.json"
    manifest = json.loads(manifest_path.read_bytes())
    mutate(manifest)
    raw = canonical_json(manifest)
    manifest_path.write_bytes(raw)
    digest = hashlib.sha256(raw).hexdigest()
    record.joinpath("manifest.sha256").write_text(
        f"{digest}  manifest.json\n", encoding="utf-8"
    )


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("record", type=Path)
    parser.add_argument("--checkpoint", type=Path, required=True)
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    verifier = Path(__file__).with_name("verify_weight_bindings.py")
    positive = invoke(verifier, args.record, args.checkpoint)
    if positive.returncode != 0 or not positive.stdout.startswith("VERIFY PASS: "):
        print("CONTROL STOP: unperturbed record did not pass", file=sys.stderr)
        print(positive.stdout, end="", file=sys.stderr)
        print(positive.stderr, end="", file=sys.stderr)
        return 6

    with tempfile.TemporaryDirectory(prefix="tiler-weight-bindings-") as temporary:
        root = Path(temporary)
        for control, mutate, expected_error in CONTROLS:
            subject = root / control
            shutil.copytree(args.record, subject)
            write_perturbed(subject, mutate)
            result = invoke(verifier, subject, args.checkpoint)
            actual_error = result.stderr.rstrip("\n")
            if result.returncode != 5:
                print(
                    f"CONTROL STOP: {control} exited {result.returncode}; expected 5",
                    file=sys.stderr,
                )
                return 6
            if result.stdout:
                print(
                    f"CONTROL STOP: {control} wrote unexpected stdout", file=sys.stderr
                )
                return 6
            if actual_error != expected_error:
                print(
                    f"CONTROL STOP: {control} said {actual_error!r}; "
                    f"expected {expected_error!r}",
                    file=sys.stderr,
                )
                return 6
            print(f"CONTROL PASS: {control}: {actual_error}")
    print(f"CONTROL PASS: unperturbed record and {len(CONTROLS)} subject perturbations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
