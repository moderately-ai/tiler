---
id: select-the-governed-artifact-digest-implementation
title: Choose the production implementation of the governed artifact digest
status: todo
priority: p2
dependencies: []
related: [prototype-neutral-artifact-codec]
scopes: [implementation/artifact, implementation/workspace, research/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, serialization, workspace]
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
