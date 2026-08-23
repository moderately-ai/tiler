---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning.qwen3-checkpoint-f32-inputs"
kind: "experiment"
title: "Qwen3 checkpoint F32 program inputs"
topics: ["program-planning", "language-model", "ingestion", "weights", "dtype", "qwen"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement", "executable-model"]
supports: ["tiler.research.program-planning.complete-model-ingestion-and-execution", "tiler.research.program-planning.first-metal-lm-workload"]
entrypoints: ["spikes/program-planning/qwen3-checkpoint-f32-inputs/src/main.rs"]
last_verified: "2026-08-22"
ticket: "ingest-the-checkpoint-as-f32-program-inputs"
---

# Qwen3 checkpoint F32 program inputs

This isolated Cargo workspace is consumer-owned conformance evidence. It is
not a root-workspace member and makes no compiler, runtime, artifact, Metal, or
support claim. Its sole Tiler dependency is the facade, which lets it wrap the
checkpoint's values through `TensorAdapter` without weakening the root
workspace rule that no member depends on that facade.

The loader validates the retained 310-row binding manifest, then authenticates
the complete pinned safetensors SHA-256 before parsing its header or assigning
semantic meaning to payload values. It then streams each BF16 tensor into its
one retained dense row-major F32 byte buffer. The same pass hashes the
concatenated F32 byte runs in canonical manifest order and counts NaN, infinite,
and subnormal F32 values. It checks the resulting digest before it creates any
wrapper, then retains that digest and census beside exactly 310
`TensorAdapter` wrappers. No BF16 tensor is retained, and no program contains a
cast: inputs advertise `StorageScalar::F32` before a program sees them.

## Acquire outside this repository

The checkpoint never belongs beneath this directory. Either acquire it through
the Hugging Face cache, or put it in any external directory:

```sh
hf download Qwen/Qwen3-0.6B-Base --revision da87bfb608c14b7cf20ba1ce41287e8de496c0cd
```

The local working directory `/local-work/` is the only ignored path and is for
regenerable command output, never checkpoint bytes. The loader refuses a path
whose full-file digest is not
`cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba`.

## Run

From this directory, point at the local Hugging Face snapshot (or another
external copy of the same verified file):

```sh
cargo run --release -- \
  /Users/you/.cache/huggingface/hub/models--Qwen--Qwen3-0.6B-Base/snapshots/da87bfb608c14b7cf20ba1ce41287e8de496c0cd/model.safetensors
```

The process intentionally retains 2,384,199,680 F32 payload bytes. Its output
records elapsed time, resident bytes at retained-load completion (a current
`ps` observation, not peak RSS), the canonical widened-byte digest, and all
three exceptional-value counts. The measurement describes only this checkpoint
and host run.

## Checks

```sh
cargo test
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

The focused tests perturb the checkpoint digest, a same-shape manifest
name-to-qualified-slot permutation, BF16 infinity and NaN independently, the
F32 payload digest, and a subnormal census. They also hand a Bf16-stored
operand to an F32-declared region and require `BindError::StorageScalarMismatch`.

## Findings

**Measurement — 2026-08-22, on an Apple M4 Max, macOS 27.0 build 26A5416b.** Before running, the resolved local snapshot file was independently re-hashed with `shasum -a 256`, returning `cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba`, matching the pin this README states, so no checkpoint bytes were acquired for this run. `cargo build --release` then compiled clean, and `cargo run --release -- /Users/tsanterre/.cache/huggingface/hub/models--Qwen--Qwen3-0.6B-Base/snapshots/da87bfb608c14b7cf20ba1ce41287e8de496c0cd/model.safetensors` printed:

```
checkpoint tensors: 310
retained F32 bytes: 2384199680
first qualified slot: P1+P3.shared.W_embed; final checkpoint tensor: model.norm.weight
distinct bare interface keys: 13
widened bytes sha256: d2abe344f7a4e4c0ea79c4a3c524ca851b095d930064e086d980972fe95c8437
widened census: nan=0 infinite=0 subnormal=0
elapsed milliseconds: 3034
resident bytes at retained-load completion: 2392752128
```

**Measurement — the widened digest and census reproduce the 2026-08-17 landing measurement exactly.** The widened-byte digest and the `nan=0 infinite=0 subnormal=0` census both match [the ingestion ticket's original outcome](../../../tickets/ingest-the-checkpoint-as-f32-program-inputs.md), recorded on the same pinned checkpoint. The elapsed-time and resident-byte figures are this run's own host observations rather than retained fixture content, so they are not expected to reproduce exactly across hosts or runs, and they did not: this run measured 3,034 ms and 2,392,752,128 resident bytes against the landing's 4,696 ms and 1,799,782,400 resident bytes.

**Measurement — `cargo nextest run --no-capture` (this repository's preferred runner; the README's own `cargo test -- --nocapture` above is equivalent for this crate).** All 8 tests passed, including the six negative controls, each printing its refusal text verbatim:

```
checkpoint digest mismatch: expected cd2a512003e2f9f3cd3c32a9c3573f820bb28c940f73c57b1ddaa983d9223eba, got 00
manifest mapping mismatch for `model.layers.0.self_attn.k_proj.weight`: expected `P2.layer-00.W_k` / `W_k`, got `P2.layer-00.W_v` / `W_k`
refusing widened checkpoint: 0 NaN, 1 infinite, 0 subnormal values
refusing widened checkpoint: 1 NaN, 0 infinite, 0 subnormal values
widened payload digest mismatch: expected e00e5eb9444182f352323374ef4e08ebcb784725fdd4fd612d7730540b3e0c8c, got d88c86f15bbea365d658ad95a81d45367c465f7af6f7264fb077f01747ddc77d
tiler.bind.storage-scalar-mismatch: operand `weight` is declared as F32 and the supplied value stores Bf16
```

**Measurement — the remaining documented checks pass clean.** `cargo clippy --all-targets -- -D warnings` and `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` both completed with no warnings on this run.

**On `verified_at_commit`.** This record does not carry it. [The parent ticket's escalation](../../../tickets/reach-every-spike-record-from-the-experiment-catalog.md) reports the field's definition as still open at this run's base — reported there, not independently re-verified here beyond confirming `grep -c verified_at_commit docs/document-metadata.md` still returns 0 — and this ticket's own instructions are to record `last_verified` alone while that question is unsettled rather than add an undefined field.
