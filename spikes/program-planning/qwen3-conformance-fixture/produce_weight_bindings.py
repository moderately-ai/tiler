#!/usr/bin/env python3
"""Produce the retained Qwen3-0.6B-Base weight-binding record.

This is a checkpoint-local research producer.  It never downloads a file and it
does not load tensor payloads: it authenticates the exact local checkpoint, reads
its safetensors header, and emits the intended P1/P2/P3 binding relation.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import math
import struct
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SCHEMA = "tiler-research/qwen3-weight-binding-manifest/v1"
REPO_ID = "Qwen/Qwen3-0.6B-Base"
REVISION = "da87bfb608c14b7cf20ba1ce41287e8de496c0cd"
CHECKPOINT_FILE = "model.safetensors"
CHECKPOINT_SHA256 = "cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba"
CHECKPOINT_BYTES = 1_192_135_096
HEADER_BYTES = 35_248
TENSOR_COUNT = 310
LAYER_COUNT = 28
CHECKPOINT_DTYPE = "BF16"
PROGRAM_STORAGE_SCALAR = "F32"
RECORD_FILES = ("manifest.json", "manifest.sha256", "verification.tsv")


@dataclass(frozen=True)
class Role:
    suffix: str
    interface_key: str
    shape: tuple[int, ...]


# This is the producer's direct name-to-interface transcription from the three
# complete semantic fixtures.  The verifier deliberately owns a separate,
# inventory-first derivation rather than importing this table.
ROLES = (
    Role("input_layernorm.weight", "w_input_layernorm", (1024,)),
    Role("mlp.down_proj.weight", "W_down", (1024, 3072)),
    Role("mlp.gate_proj.weight", "W_gate", (3072, 1024)),
    Role("mlp.up_proj.weight", "W_up", (3072, 1024)),
    Role("post_attention_layernorm.weight", "w_post_attention_layernorm", (1024,)),
    Role("self_attn.k_norm.weight", "w_k_norm", (128,)),
    Role("self_attn.k_proj.weight", "W_k", (1024, 1024)),
    Role("self_attn.o_proj.weight", "W_o", (1024, 2048)),
    Role("self_attn.q_norm.weight", "w_q_norm", (128,)),
    Role("self_attn.q_proj.weight", "W_q", (2048, 1024)),
    Role("self_attn.v_proj.weight", "W_v", (1024, 1024)),
)


class Stop(RuntimeError):
    """A fail-closed producer refusal."""


def stop(message: str) -> None:
    raise Stop(message)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            stop(f"safetensors header repeats JSON key {key!r}")
        result[key] = value
    return result


def read_header(checkpoint: Path) -> tuple[dict[str, dict[str, Any]], dict[str, Any]]:
    with checkpoint.open("rb") as source:
        prefix = source.read(8)
        if len(prefix) != 8:
            stop("checkpoint is shorter than the safetensors length prefix")
        header_bytes = struct.unpack("<Q", prefix)[0]
        if header_bytes != HEADER_BYTES:
            stop(f"safetensors header is {header_bytes} bytes; expected {HEADER_BYTES}")
        raw = source.read(header_bytes)
        if len(raw) != header_bytes:
            stop("checkpoint ends inside its safetensors header")

    try:
        decoded = json.loads(raw, object_pairs_hook=unique_object)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        stop(f"safetensors header is not valid UTF-8 JSON: {error}")
    if not isinstance(decoded, dict):
        stop("safetensors header root is not an object")
    metadata = decoded.pop("__metadata__", {})
    if metadata != {"format": "pt"}:
        stop(f"safetensors metadata is {metadata!r}; expected {{'format': 'pt'}}")
    if len(decoded) != TENSOR_COUNT:
        stop(
            f"safetensors header names {len(decoded)} tensors; expected {TENSOR_COUNT}"
        )

    tensors: dict[str, dict[str, Any]] = {}
    spans: list[tuple[int, int, str]] = []
    for name, entry in decoded.items():
        if not isinstance(name, str) or not isinstance(entry, dict):
            stop("safetensors tensor entries must be name/object pairs")
        if set(entry) != {"dtype", "shape", "data_offsets"}:
            stop(f"safetensors entry {name!r} has foreign or missing fields")
        dtype = entry["dtype"]
        shape = entry["shape"]
        offsets = entry["data_offsets"]
        if dtype != CHECKPOINT_DTYPE:
            stop(f"checkpoint tensor {name!r} is {dtype!r}; expected BF16")
        if (
            not isinstance(shape, list)
            or not shape
            or any(type(extent) is not int or extent < 0 for extent in shape)
        ):
            stop(f"checkpoint tensor {name!r} has an invalid shape")
        if (
            not isinstance(offsets, list)
            or len(offsets) != 2
            or any(type(offset) is not int or offset < 0 for offset in offsets)
            or offsets[1] < offsets[0]
        ):
            stop(f"checkpoint tensor {name!r} has invalid data offsets")
        expected_bytes = math.prod(shape) * 2
        if offsets[1] - offsets[0] != expected_bytes:
            stop(
                f"checkpoint tensor {name!r} spans {offsets[1] - offsets[0]} bytes; "
                f"its BF16 shape requires {expected_bytes}"
            )
        tensors[name] = entry
        spans.append((offsets[0], offsets[1], name))

    cursor = 0
    for start, end, name in sorted(spans):
        if start != cursor:
            stop(
                f"checkpoint tensor {name!r} starts at payload offset {start}; "
                f"expected contiguous offset {cursor}"
            )
        cursor = end
    payload_bytes = CHECKPOINT_BYTES - 8 - HEADER_BYTES
    if cursor != payload_bytes:
        stop(
            f"safetensors payload covers {cursor} bytes; file framing requires {payload_bytes}"
        )
    return tensors, metadata


def binding_for(name: str, entry: dict[str, Any]) -> dict[str, Any]:
    if name == "model.embed_tokens.weight":
        expected_shape = (151_936, 1024)
        interface_key = "W_embed"
        qualified_slot = "P1+P3.shared.W_embed"
        uses = ["P1.W_embed", "P3.W_embed"]
    elif name == "model.norm.weight":
        expected_shape = (1024,)
        interface_key = "w_norm"
        qualified_slot = "P3.w_norm"
        uses = [qualified_slot]
    else:
        prefix = "model.layers."
        if not name.startswith(prefix):
            stop(f"checkpoint tensor name {name!r} has no P1/P2/P3 binding")
        rest = name[len(prefix) :]
        layer_text, separator, suffix = rest.partition(".")
        if not separator or not layer_text.isascii() or not layer_text.isdecimal():
            stop(f"checkpoint tensor name {name!r} has no decimal layer owner")
        layer = int(layer_text)
        if not 0 <= layer < LAYER_COUNT or str(layer) != layer_text:
            stop(f"checkpoint tensor name {name!r} has a noncanonical layer owner")
        role = next(
            (candidate for candidate in ROLES if candidate.suffix == suffix), None
        )
        if role is None:
            stop(f"checkpoint tensor name {name!r} has no decoder-layer interface role")
        expected_shape = role.shape
        interface_key = role.interface_key
        qualified_slot = f"P2.layer-{layer:02d}.{interface_key}"
        uses = [qualified_slot]

    actual_shape = tuple(entry["shape"])
    if actual_shape != expected_shape:
        stop(
            f"checkpoint tensor {name!r} has shape {list(actual_shape)}; "
            f"the program interface requires {list(expected_shape)}"
        )
    return {
        "checkpoint_tensor": name,
        "expected_shape": list(expected_shape),
        "expected_storage_scalar": PROGRAM_STORAGE_SCALAR,
        "interface_key": interface_key,
        "qualified_slot": qualified_slot,
        "uses": uses,
    }


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=True, indent=2, sort_keys=True) + "\n"
    ).encode()


def build_manifest(tensors: dict[str, dict[str, Any]]) -> dict[str, Any]:
    bindings = [binding_for(name, tensors[name]) for name in sorted(tensors)]
    names = [binding["checkpoint_tensor"] for binding in bindings]
    slots = [binding["qualified_slot"] for binding in bindings]
    if len(bindings) != TENSOR_COUNT or len(set(names)) != TENSOR_COUNT:
        stop("the produced binding relation is not total over 310 unique names")
    if len(set(slots)) != TENSOR_COUNT:
        stop("the produced binding relation is not injective over qualified slots")

    return {
        "bindings": bindings,
        "checkpoint": {
            "bytes": CHECKPOINT_BYTES,
            "dtype": CHECKPOINT_DTYPE,
            "file": CHECKPOINT_FILE,
            "repo_id": REPO_ID,
            "revision": REVISION,
            "safetensors_header_bytes": HEADER_BYTES,
            "sha256": CHECKPOINT_SHA256,
            "tensor_count": TENSOR_COUNT,
        },
        "framing": {
            "binding_order": "checkpoint_tensor ascending by UTF-8 bytes",
            "json": "UTF-8; object keys sorted; indent 2; LF line endings; one final LF",
            "manifest_digest": "SHA-256 over the exact manifest.json bytes",
        },
        "schema": SCHEMA,
        "validation_boundary": {
            "manifest_catches": [
                "checkpoint revision or complete-file SHA-256 mismatch",
                "duplicate, omitted, or foreign checkpoint tensor names",
                "duplicate qualified program slots",
                "name-to-qualified-slot or bare interface-key permutation",
                "checkpoint-header or program-interface shape mismatch",
                "checkpoint dtype or program StorageScalar mismatch",
            ],
            "not_caught": [
                "a consumer that validates this record and then ignores it while binding buffers",
                "a same-shape F32 buffer swap after checkpoint authentication and named extraction",
                "any runtime loading, widening, execution, artifact, compiler, or Metal behavior",
            ],
            "tiler_bind_error_checks_after_loading": [
                "BindError::OperandCountMismatch",
                "BindError::UnsupportedCapability",
                "BindError::RankMismatch",
                "BindError::StorageScalarMismatch",
                "BindError::LiteralExtentMismatch",
                "BindError::InconsistentExtent",
                "BindError::StorageLengthMismatch on the dispatch path",
            ],
        },
    }


def verification_bytes(manifest_sha256: str) -> bytes:
    output = io.StringIO(newline="")
    writer = csv.writer(output, delimiter="\t", lineterminator="\n")
    writer.writerow(("fact", "value"))
    for fact, value in (
        ("schema", SCHEMA),
        ("checkpoint.repo_id", REPO_ID),
        ("checkpoint.revision", REVISION),
        ("checkpoint.sha256", CHECKPOINT_SHA256),
        ("checkpoint.bytes", CHECKPOINT_BYTES),
        ("checkpoint.safetensors_header_bytes", HEADER_BYTES),
        ("checkpoint.tensor_count", TENSOR_COUNT),
        ("checkpoint.dtype", CHECKPOINT_DTYPE),
        ("program.storage_scalar", PROGRAM_STORAGE_SCALAR),
        ("binding.count", TENSOR_COUNT),
        ("binding.unique_checkpoint_names", TENSOR_COUNT),
        ("binding.unique_qualified_slots", TENSOR_COUNT),
        ("manifest.sha256", manifest_sha256),
    ):
        writer.writerow((fact, value))
    return output.getvalue().encode()


def expected_record(checkpoint: Path, revision: str) -> dict[str, bytes]:
    if revision != REVISION:
        stop(f"revision is {revision!r}; expected {REVISION!r}")
    try:
        size = checkpoint.stat().st_size
    except OSError as error:
        stop(f"cannot stat checkpoint {checkpoint}: {error}")
    if size != CHECKPOINT_BYTES:
        stop(f"checkpoint is {size} bytes; expected {CHECKPOINT_BYTES}")
    digest = sha256_file(checkpoint)
    if digest != CHECKPOINT_SHA256:
        stop(f"checkpoint SHA-256 is {digest}; expected {CHECKPOINT_SHA256}")
    tensors, _metadata = read_header(checkpoint)
    manifest_bytes = canonical_json(build_manifest(tensors))
    manifest_sha256 = hashlib.sha256(manifest_bytes).hexdigest()
    return {
        "manifest.json": manifest_bytes,
        "manifest.sha256": f"{manifest_sha256}  manifest.json\n".encode(),
        "verification.tsv": verification_bytes(manifest_sha256),
    }


def write_or_compare(record: dict[str, bytes], out: Path, compare: bool) -> None:
    if compare:
        if not out.is_dir():
            stop(f"comparison record is not a directory: {out}")
        actual_names = sorted(path.name for path in out.iterdir() if path.is_file())
        if actual_names != sorted(RECORD_FILES):
            stop(
                f"comparison record files are {actual_names}; expected {sorted(RECORD_FILES)}"
            )
        for name in RECORD_FILES:
            if out.joinpath(name).read_bytes() != record[name]:
                stop(f"regenerated {name} differs from the retained record")
        return

    if out.exists():
        stop(f"output path already exists: {out}")
    out.mkdir(parents=True)
    for name in RECORD_FILES:
        out.joinpath(name).write_bytes(record[name])


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument(
        "--compare",
        action="store_true",
        help="compare a regenerated record byte-for-byte instead of writing",
    )
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    try:
        record = expected_record(args.checkpoint, args.revision)
        write_or_compare(record, args.out, args.compare)
    except (OSError, Stop) as error:
        print(f"PRODUCE STOP: {error}", file=sys.stderr)
        return 4
    action = "matched" if args.compare else "wrote"
    print(
        f"PRODUCE PASS: {action} {TENSOR_COUNT} total, unique checkpoint names "
        f"to {TENSOR_COUNT} unique qualified slots; checkpoint_sha256={CHECKPOINT_SHA256}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
