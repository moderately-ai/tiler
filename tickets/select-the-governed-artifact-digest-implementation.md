---
id: select-the-governed-artifact-digest-implementation
title: Choose the production implementation of the governed artifact digest
status: in-progress
priority: p2
dependencies: []
related: [prototype-neutral-artifact-codec]
scopes: [implementation/artifact, implementation/workspace, research/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, serialization, workspace]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785182866
---
`tiler.research.artifacts.target-neutral-envelope` records "select and govern the initial cryptographic digest algorithm and domain separators" as an open bounded decision, with a measurement attached: hashing cost during proc-macro expansion and during runtime loading.

**What `prototype-neutral-artifact-codec` settled, and what it did not.** It settled the *wire contract*: the envelope header carries a governed algorithm tag, a reader never infers an algorithm from a digest width, and `tiler.digest.sha-256.v1` is the only admitted algorithm. It did **not** settle the implementation. SHA-256 is implemented in `crates/tiler-artifact/src/program/codec/digest.rs` (FIPS 180-4, pinned by the published vectors and by every padding branch) rather than taken from a crate, because adding the workspace's first cryptographic dependency would have answered the open decision by accident.

**Why the choice is cheap to revisit.** The implementation is behind `DigestAlgorithm` and produces the same bytes either way, so swapping it changes no encoded envelope.

**What closes this.** A comparison of the in-crate implementation against an audited crate on: build-time cost, binary size, dependency-policy fit under `AGENTS.md`, and measured hashing throughput on a real artifact during expansion and loading. Record the measurement and either adopt the dependency or record the in-crate implementation as the accepted one with its audit basis.

**Measurement boundary already available.** The codec's corruption test
exhausts the framing header and framed section stream, then samples the
25,000-byte manifest interior at a prime stride of 61 because one manifest
digest already covers every interior byte. It is evidence about corruption
detection, not a current throughput measurement. Measure hashing cost directly
on a representative artifact before using performance to select an
implementation.

## Outcome — `sha2` 0.11.0 adopted (2026-07-27)

**Decision.** The hand-rolled FIPS 180-4 SHA-256 in `codec/digest.rs` is replaced by `sha2` 0.11.0. Tom's call, made on the measurement below: *"we should never have written our own hashing."* This is the workspace's first cryptographic dependency and `tiler-artifact`'s first dependency of any kind.

**Measurement — why the ticket's framing was wrong.** The ticket asked *which* SHA-256. The prior question was *how fast is the one we have*, and the answer was 53 MiB/s, because `working.rotate_right(1)` on a `[u32; 8]` resolved to the **generic slice rotation** — an out-of-line call with a 320-byte frame and three `memmove` stubs, 64× per block. Fixing that alone reached 413 MiB/s. `sha2` 0.11 then reached 2,863 MiB/s.

| implementation | 18,013-byte message | throughput |
| --- | --- | --- |
| hand-rolled, as found | 321.9 µs | 53 MiB/s |
| hand-rolled, rotate fixed | 39.4 µs | 413 MiB/s |
| **`sha2` 0.11.0** | **6.0 µs** | **2,863 MiB/s** |

`sha2` 0.11 is where the aarch64 crypto-extension path appears; 0.10.9 measured 539 MiB/s. No build flags are needed — `cpufeatures` selects it at runtime. This is exactly the capability `unsafe_code = "forbid"` puts out of reach for in-tree code, and it binds workspace crates rather than dependencies, so it does not block the adoption.

**Measurement: artifact decode 75 µs → 18.7 µs**, on top of the 662 µs the programme started from.

**Fact: the output bytes are unchanged**, and this is pinned rather than asserted. The FIPS 180-4 published vectors including the one-million-character case, the chunked-versus-single-shot padding cases, and an exhaustive sweep of every message length `0..=192` against a value produced by Python's `hashlib` all pass unmodified, as do every artifact-identity, cache-key, and proof-sidecar test in the workspace (995 tests).

**The digest tests now go through `digest_parts` rather than the implementation's internals**, so they pin the bytes this module *publishes* — the contract artifact identity actually rests on — and keep meaning the same thing across a change of implementation. Under the old spelling they reached into a private struct and would have had to be rewritten to say anything about the replacement.

**Corrected in passing:** the ~25,000-byte manifest interior cited here and at `codec/tests.rs:499` is **18,013 bytes**. The ~26,000 figure elsewhere is the *envelope*, whose interior it is.
