---
id: fix-the-sha-256-inner-loop-and-encoder-presizing
title: Fix the SHA-256 inner loop and encoder pre-sizing
status: in-progress
priority: p2
dependencies: [measure-compiler-and-artifact-hot-paths]
related: []
scopes: [implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [performance, artifact]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785180316
---
Constant factors, deliberately sequenced last: they multiply work that the earlier phases largely eliminate, so measuring them first would overstate their value.

## Facts

**`working.rotate_right(1)`** — `digest.rs:378`. `working` is `[u32; 8]`. Arrays have no inherent `rotate_right`, so this derefs to the **generic slice rotation**, the gcd-juggling `ptr::copy`-based routine, invoked with a runtime-opaque `mid = 1`. It runs **64 times per 64-byte block** — roughly 26,000 calls per 26 KB envelope, per digest pass — on the innermost loop of every hash in the workspace. The canonical form shifts eight named variables, which compiles to register renames. The comment at `digest.rs:376-377` explains the readability motivation, so the trade should be made deliberately rather than silently.

**`finish()` pads one byte at a time** through the full `update` path — `digest.rs:322-325`, up to 63 calls per digest, each saving and restoring `message_bytes`, doing a `checked_add`, a `u64::try_from`, a one-byte `copy_from_slice` and an `as_chunks` on an empty remainder. Amortised to noise over 26 KB; a large fraction of the many **small** digests — section descriptors, `CacheKey::derive_bytes`, `payload_identity`.

**Essentially no canonical encoder uses `with_capacity`** — the exception is `compute_graph_identity`. `encode_manifest` (`encode.rs:149`) starts from `Vec::new()` and grows to ~18 KB, about 12 reallocations, twice per decode. ~30 further sites across the identity encoders.

## What is otherwise sound

The implementation is block-at-a-time and streaming, and `digest_parts` (`digest.rs:103`) streams parts through one state without concatenating. The obvious suspicion is ruled out.

## Note for the digest-selection ticket

`select-the-governed-artifact-digest-implementation` frames the question as *which* SHA-256. The measurement says the larger question is *how many times the same bytes are hashed*. It also omits the real trade: an audited crate buys SHA-NI / ARMv8 crypto-extension intrinsics, which this crate **cannot** use under `unsafe_code = "forbid"` — roughly an order of magnitude on the compression function, and the actual comparison.

## Closes when

The rotation and padding paths are fixed with the readability trade recorded; the identity encoders pre-size where the length is known or bounded; hashing throughput is measured before and after; `make full` passes.
