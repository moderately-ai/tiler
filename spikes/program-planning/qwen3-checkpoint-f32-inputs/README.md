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
