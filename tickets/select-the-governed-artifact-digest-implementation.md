---
id: select-the-governed-artifact-digest-implementation
title: Choose the production implementation of the governed artifact digest
status: todo
priority: p2
dependencies: []
related: [prototype-neutral-artifact-codec]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [artifact, serialization, workspace]
---
`tiler.research.artifacts.target-neutral-envelope` records "select and govern the initial cryptographic digest algorithm and domain separators" as an open bounded decision, with a measurement attached: hashing cost during proc-macro expansion and during runtime loading.

**What `prototype-neutral-artifact-codec` settled, and what it did not.** It settled the *wire contract*: the envelope header carries a governed algorithm tag, a reader never infers an algorithm from a digest width, and `tiler.digest.sha-256.v1` is the only admitted algorithm. It did **not** settle the implementation. SHA-256 is implemented in `crates/tiler-artifact/src/program/codec/digest.rs` (FIPS 180-4, pinned by the published vectors and by every padding branch) rather than taken from a crate, because adding the workspace's first cryptographic dependency would have answered the open decision by accident.

**Why the choice is cheap to revisit.** The implementation is behind `DigestAlgorithm` and produces the same bytes either way, so swapping it changes no encoded envelope.

**What closes this.** A comparison of the in-crate implementation against an audited crate on: build-time cost, binary size, dependency-policy fit under `AGENTS.md`, and measured hashing throughput on a real artifact during expansion and loading. Record the measurement and either adopt the dependency or record the in-crate implementation as the accepted one with its audit basis.

**Measurement boundary already available.** The bounded fixture envelope of the serial-sum artifact is 25,183 bytes; a full single-byte corruption sweep over it costs about 35 seconds in the unoptimized profile, which is the current evidence that the unoptimized hashing path is the dominant cost in tests.
