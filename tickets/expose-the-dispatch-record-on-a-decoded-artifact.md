---
id: expose-the-dispatch-record-on-a-decoded-artifact
title: A decoded artifact must carry enough to dispatch from bytes alone
status: in-progress
priority: p0
dependencies: []
related: []
scopes: [implementation/artifact]
shared_scopes: []
paths: []
tags: [implementation, artifact, runtime, spine]
claimed_from: todo
assignee: agent-dispatch
lease_expires_at: 1785009033
---
Implements the decision Tom made on `carry-reconstructable-kernel-programs-in-the-neutral-envelope`: **a decoded envelope is a dispatch record.** That decision is recorded and not yet carried out, which is why the runtime proof still bypasses the envelope.

**Fact — what a consumer can read today.** `DecodedArtifact`'s complete public surface is `identity`, `features`, `routing`, `payloads`, `sections`, `variant_count`, and `re_encode` (`crates/tiler-artifact/src/program/codec/view.rs`).

**Fact — what it cannot read.** The entry symbol: `decode_metadata` is `pub(crate)` at `codec/payload.rs:292`, so the `PayloadMetadata` carrying entry mappings and transport slots is unreachable. And the per-variant entries: `EntryRow`, `BindingData` and `LaunchData` are encoded, validated and round-tripped, but `pub(crate)` with no projection.

**Inference — this is why the bypass survives.** A worker attempting `route-the-runtime-proof-through-the-artifact-envelope` concluded the runner must reach the producer's assembler code, and proposed sharing it via a library target or a new crate. That diagnosis is right about the symptom and wrong about the cause: if a consumer needs the producer's *code* to consume an artifact, the artifact is not the interface. **The bytes are supposed to be the interface.** Sharing assembler code would have worked around this gap and left it in place.

**Consequence — this must NOT be closed by sharing producer code.** No library target on a prototype, no assembler crate, no duplicated assembly. If the dispatch record is complete, a runner reads a file the producer wrote and needs nothing else.

## Scope

Project the rows that already exist: per-variant entries with their bindings (transport slot, accessible-byte expression) and launch contract (grid, workgroup, preconditions), and the payload metadata a loader needs to find its entry symbol. Accessors over validated rows, not new encoded facts — `decode_artifact` already proved them.

**One fact is genuinely missing rather than merely unprojected**, and it must be named rather than invented: `BindingData` carries no value or view reference, and the stage reaches the envelope only as an opaque content key, so a decoded envelope cannot say **which buffer a slot addresses**. Tom's decision records this and accepts it. Decide explicitly whether to encode that reference now or to define the dispatch contract as slot-ordered — and if slot-ordered, say what makes the order authoritative, because a consumer binding by position needs that guarantee stated rather than assumed.

A carried value reference would be asserted by the producer rather than re-derived by the decoder. State that asymmetry wherever the record is documented.

## Closes when

A consumer holding only encoded bytes can name every entry, its bindings and their transport slots, its launch geometry, and its backend entry symbol; the which-buffer question is decided and stated; `route-the-runtime-proof-through-the-artifact-envelope` needs no producer code; and `uv run --locked python scripts/check_repository.py` passes.
