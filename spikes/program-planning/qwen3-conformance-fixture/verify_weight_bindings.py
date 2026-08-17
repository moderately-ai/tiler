#!/usr/bin/env python3
"""Verify the retained Qwen3-0.6B-Base weight-binding record fail closed."""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import math
import struct
import sys
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
BINDING_FIELDS = {
    "checkpoint_tensor",
    "expected_shape",
    "expected_storage_scalar",
    "interface_key",
    "qualified_slot",
    "uses",
}


# Independent, inventory-first restatement of the completed P2 interface.  This
# verifier constructs the required 310-name population; it does not call or
# import the producer's name parser.
LAYER_SPECS = {
    "input_layernorm.weight": ("w_input_layernorm", [1024]),
    "post_attention_layernorm.weight": ("w_post_attention_layernorm", [1024]),
    "self_attn.q_proj.weight": ("W_q", [2048, 1024]),
    "self_attn.k_proj.weight": ("W_k", [1024, 1024]),
    "self_attn.v_proj.weight": ("W_v", [1024, 1024]),
    "self_attn.q_norm.weight": ("w_q_norm", [128]),
    "self_attn.k_norm.weight": ("w_k_norm", [128]),
    "self_attn.o_proj.weight": ("W_o", [1024, 2048]),
    "mlp.gate_proj.weight": ("W_gate", [3072, 1024]),
    "mlp.up_proj.weight": ("W_up", [3072, 1024]),
    "mlp.down_proj.weight": ("W_down", [1024, 3072]),
}

FRAMING = {
    "binding_order": "checkpoint_tensor ascending by UTF-8 bytes",
    "json": "UTF-8; object keys sorted; indent 2; LF line endings; one final LF",
    "manifest_digest": "SHA-256 over the exact manifest.json bytes",
}

VALIDATION_BOUNDARY = {
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
}


class Stop(RuntimeError):
    """A fail-closed verifier refusal."""


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
            stop(f"JSON object repeats key {key!r}")
        result[key] = value
    return result


def parse_json(raw: bytes, subject: str) -> Any:
    try:
        return json.loads(raw, object_pairs_hook=unique_object)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        stop(f"{subject} is not valid UTF-8 JSON: {error}")


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=True, indent=2, sort_keys=True) + "\n"
    ).encode()


def expect_fields(value: Any, fields: set[str], subject: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        stop(f"{subject} is not an object")
    actual = set(value)
    if actual != fields:
        missing = sorted(fields - actual)
        foreign = sorted(actual - fields)
        stop(f"{subject} fields differ: missing={missing}, foreign={foreign}")
    return value


def expected_bindings() -> dict[str, dict[str, Any]]:
    expected: dict[str, dict[str, Any]] = {
        "model.embed_tokens.weight": {
            "checkpoint_tensor": "model.embed_tokens.weight",
            "expected_shape": [151_936, 1024],
            "expected_storage_scalar": PROGRAM_STORAGE_SCALAR,
            "interface_key": "W_embed",
            "qualified_slot": "P1+P3.shared.W_embed",
            "uses": ["P1.W_embed", "P3.W_embed"],
        },
        "model.norm.weight": {
            "checkpoint_tensor": "model.norm.weight",
            "expected_shape": [1024],
            "expected_storage_scalar": PROGRAM_STORAGE_SCALAR,
            "interface_key": "w_norm",
            "qualified_slot": "P3.w_norm",
            "uses": ["P3.w_norm"],
        },
    }
    for layer in range(LAYER_COUNT):
        for suffix, (interface_key, shape) in LAYER_SPECS.items():
            name = f"model.layers.{layer}.{suffix}"
            qualified_slot = f"P2.layer-{layer:02d}.{interface_key}"
            expected[name] = {
                "checkpoint_tensor": name,
                "expected_shape": shape,
                "expected_storage_scalar": PROGRAM_STORAGE_SCALAR,
                "interface_key": interface_key,
                "qualified_slot": qualified_slot,
                "uses": [qualified_slot],
            }
    if len(expected) != TENSOR_COUNT:
        stop(f"verifier defect: expected inventory has {len(expected)} names")
    return expected


def verify_manifest(
    record: Path,
) -> tuple[dict[str, Any], str, dict[str, dict[str, Any]]]:
    if not record.is_dir():
        stop(f"record is not a directory: {record}")
    actual_files = sorted(path.name for path in record.iterdir() if path.is_file())
    if actual_files != sorted(RECORD_FILES):
        stop(f"record files are {actual_files}; expected {sorted(RECORD_FILES)}")

    manifest_raw = record.joinpath("manifest.json").read_bytes()
    manifest_sha256 = hashlib.sha256(manifest_raw).hexdigest()
    expected_sidecar = f"{manifest_sha256}  manifest.json\n".encode()
    if record.joinpath("manifest.sha256").read_bytes() != expected_sidecar:
        stop(f"manifest.sha256 does not bind manifest.json SHA-256 {manifest_sha256}")

    manifest = parse_json(manifest_raw, "manifest.json")
    if canonical_json(manifest) != manifest_raw:
        stop("manifest.json does not use its declared canonical JSON framing")
    manifest = expect_fields(
        manifest,
        {"bindings", "checkpoint", "framing", "schema", "validation_boundary"},
        "manifest root",
    )
    if manifest["schema"] != SCHEMA:
        stop(f"manifest schema is {manifest['schema']!r}; expected {SCHEMA!r}")
    if manifest["framing"] != FRAMING:
        stop("manifest framing declaration differs from the verifier's framing")
    if manifest["validation_boundary"] != VALIDATION_BOUNDARY:
        stop("manifest validation boundary differs from the verified boundary")

    checkpoint = expect_fields(
        manifest["checkpoint"],
        {
            "bytes",
            "dtype",
            "file",
            "repo_id",
            "revision",
            "safetensors_header_bytes",
            "sha256",
            "tensor_count",
        },
        "manifest checkpoint",
    )
    exact_checkpoint = {
        "bytes": CHECKPOINT_BYTES,
        "dtype": CHECKPOINT_DTYPE,
        "file": CHECKPOINT_FILE,
        "repo_id": REPO_ID,
        "revision": REVISION,
        "safetensors_header_bytes": HEADER_BYTES,
        "sha256": CHECKPOINT_SHA256,
        "tensor_count": TENSOR_COUNT,
    }
    for field, expected in exact_checkpoint.items():
        actual = checkpoint[field]
        if actual != expected:
            label = field.replace("_", " ")
            stop(f"manifest checkpoint {label} is {actual!r}; expected {expected!r}")

    bindings = manifest["bindings"]
    if not isinstance(bindings, list):
        stop("manifest bindings is not an array")
    checked: list[dict[str, Any]] = []
    names: list[str] = []
    slots: list[str] = []
    for index, candidate in enumerate(bindings):
        binding = expect_fields(candidate, BINDING_FIELDS, f"binding row {index}")
        name = binding["checkpoint_tensor"]
        slot = binding["qualified_slot"]
        if not isinstance(name, str):
            stop(f"binding row {index} checkpoint_tensor is not a string")
        if not isinstance(slot, str):
            stop(f"binding row {index} qualified_slot is not a string")
        if name in names:
            stop(f"duplicate checkpoint tensor name {name!r}")
        if slot in slots:
            stop(f"duplicate qualified program slot {slot!r}")
        names.append(name)
        slots.append(slot)
        checked.append(binding)

    expected = expected_bindings()
    expected_names = set(expected)
    actual_names = set(names)
    foreign = sorted(actual_names - expected_names)
    if foreign:
        stop(f"foreign checkpoint tensor name {foreign[0]!r}")
    missing = sorted(expected_names - actual_names)
    if missing:
        stop(
            f"missing checkpoint tensor name {missing[0]!r}; "
            f"record covers {len(actual_names)} of {TENSOR_COUNT}"
        )
    if len(checked) != TENSOR_COUNT:
        stop(f"binding population is {len(checked)}; expected {TENSOR_COUNT}")
    canonical_names = sorted(names, key=lambda name: name.encode("utf-8"))
    if names != canonical_names:
        stop("binding rows are not in checkpoint_tensor UTF-8 byte order")

    by_name = {binding["checkpoint_tensor"]: binding for binding in checked}
    for name in sorted(expected):
        actual = by_name[name]
        wanted = expected[name]
        if (
            actual["qualified_slot"] != wanted["qualified_slot"]
            or actual["interface_key"] != wanted["interface_key"]
        ):
            stop(
                f"binding {name!r} maps to {actual['qualified_slot']!r} "
                f"({actual['interface_key']!r}); expected {wanted['qualified_slot']!r} "
                f"({wanted['interface_key']!r})"
            )
        if actual["uses"] != wanted["uses"]:
            stop(
                f"binding {name!r} uses are {actual['uses']!r}; expected {wanted['uses']!r}"
            )
        if actual["expected_shape"] != wanted["expected_shape"]:
            stop(
                f"binding {name!r} expected_shape is {actual['expected_shape']!r}; "
                f"expected {wanted['expected_shape']!r}"
            )
        if actual["expected_storage_scalar"] != PROGRAM_STORAGE_SCALAR:
            stop(
                f"binding {name!r} expected_storage_scalar is "
                f"{actual['expected_storage_scalar']!r}; expected {PROGRAM_STORAGE_SCALAR!r}"
            )

    expected_verification = verification_bytes(manifest_sha256)
    if record.joinpath("verification.tsv").read_bytes() != expected_verification:
        stop("verification.tsv does not restate the verified record facts exactly")
    return checkpoint, manifest_sha256, by_name


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


def verify_checkpoint(
    checkpoint_path: Path,
    revision: str,
    manifest_checkpoint: dict[str, Any],
    bindings: dict[str, dict[str, Any]],
) -> None:
    if revision != REVISION:
        stop(f"requested revision is {revision!r}; expected {REVISION!r}")
    if manifest_checkpoint["revision"] != revision:
        stop("requested revision and manifest checkpoint revision differ")
    try:
        size = checkpoint_path.stat().st_size
    except OSError as error:
        stop(f"cannot stat checkpoint {checkpoint_path}: {error}")
    if size != CHECKPOINT_BYTES:
        stop(f"checkpoint is {size} bytes; expected {CHECKPOINT_BYTES}")
    digest = sha256_file(checkpoint_path)
    if digest != CHECKPOINT_SHA256:
        stop(f"checkpoint SHA-256 is {digest}; expected {CHECKPOINT_SHA256}")

    with checkpoint_path.open("rb") as source:
        prefix = source.read(8)
        if len(prefix) != 8:
            stop("checkpoint is shorter than the safetensors length prefix")
        header_bytes = struct.unpack("<Q", prefix)[0]
        if header_bytes != HEADER_BYTES:
            stop(f"safetensors header is {header_bytes} bytes; expected {HEADER_BYTES}")
        raw = source.read(header_bytes)
        if len(raw) != header_bytes:
            stop("checkpoint ends inside its safetensors header")
    decoded = parse_json(raw, "safetensors header")
    if not isinstance(decoded, dict):
        stop("safetensors header root is not an object")
    metadata = decoded.pop("__metadata__", None)
    if metadata != {"format": "pt"}:
        stop(f"safetensors metadata is {metadata!r}; expected {{'format': 'pt'}}")

    header_names = set(decoded)
    manifest_names = set(bindings)
    foreign = sorted(header_names - manifest_names)
    if foreign:
        stop(f"safetensors header has foreign tensor name {foreign[0]!r}")
    missing = sorted(manifest_names - header_names)
    if missing:
        stop(f"safetensors header omits tensor name {missing[0]!r}")
    if len(decoded) != TENSOR_COUNT:
        stop(
            f"safetensors header names {len(decoded)} tensors; expected {TENSOR_COUNT}"
        )

    spans: list[tuple[int, int, str]] = []
    for name in sorted(decoded):
        entry = expect_fields(
            decoded[name], {"dtype", "shape", "data_offsets"}, f"header {name!r}"
        )
        if entry["dtype"] != CHECKPOINT_DTYPE:
            stop(f"header tensor {name!r} dtype is {entry['dtype']!r}; expected 'BF16'")
        shape = entry["shape"]
        if shape != bindings[name]["expected_shape"]:
            stop(
                f"header tensor {name!r} shape is {shape!r}; "
                f"manifest expects {bindings[name]['expected_shape']!r}"
            )
        if (
            not isinstance(shape, list)
            or not shape
            or any(type(extent) is not int or extent < 0 for extent in shape)
        ):
            stop(f"header tensor {name!r} shape is invalid")
        offsets = entry["data_offsets"]
        if (
            not isinstance(offsets, list)
            or len(offsets) != 2
            or any(type(offset) is not int or offset < 0 for offset in offsets)
            or offsets[1] < offsets[0]
        ):
            stop(f"header tensor {name!r} data_offsets are invalid")
        required = math.prod(shape) * 2
        if offsets[1] - offsets[0] != required:
            stop(
                f"header tensor {name!r} spans {offsets[1] - offsets[0]} bytes; "
                f"BF16 shape requires {required}"
            )
        spans.append((offsets[0], offsets[1], name))

    cursor = 0
    for start, end, name in sorted(spans):
        if start != cursor:
            stop(
                f"header tensor {name!r} starts at payload offset {start}; "
                f"expected contiguous offset {cursor}"
            )
        cursor = end
    payload_bytes = size - 8 - header_bytes
    if cursor != payload_bytes:
        stop(
            f"safetensors spans cover {cursor} bytes; file payload has {payload_bytes}"
        )


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("record", type=Path)
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--revision", required=True)
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    try:
        checkpoint, manifest_sha256, bindings = verify_manifest(args.record)
        verify_checkpoint(args.checkpoint, args.revision, checkpoint, bindings)
    except (OSError, Stop) as error:
        print(f"VERIFY STOP: {error}", file=sys.stderr)
        return 5
    print(
        "VERIFY PASS: "
        f"checkpoint_sha256={CHECKPOINT_SHA256} checkpoint_bytes={CHECKPOINT_BYTES} "
        f"header_bytes={HEADER_BYTES} header_tensors={TENSOR_COUNT} "
        f"manifest_sha256={manifest_sha256} bindings={len(bindings)} "
        f"unique_checkpoint_names={len(bindings)} unique_qualified_slots={len(bindings)} "
        f"checkpoint_dtype={CHECKPOINT_DTYPE} program_storage_scalar={PROGRAM_STORAGE_SCALAR}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
