---
id: carry-the-stage-execution-order-in-the-envelope
title: Carry a multi-stage variant's execution order in the envelope
status: todo
priority: p1
dependencies: []
related: [carry-reconstructable-kernel-programs-in-the-neutral-envelope, expose-the-dispatch-record-on-a-decoded-artifact, route-the-runtime-loader-through-the-dispatch-record, carry-the-byte-offset-of-a-partial-binding-view]
scopes: [contracts/artifacts, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [artifact, serialization, runtime]
---
A variant that dispatches more than one stage encodes, and this build's reader refuses it. That refusal is correct and it is not the end state: it is the last gap between the dispatch record Tom decided on [`carry-reconstructable-kernel-programs-in-the-neutral-envelope`](carry-reconstructable-kernel-programs-in-the-neutral-envelope.md) and what the envelope actually carries.

**Fact — the gap, reproducible in one line.** `grep -n "FEATURE_MULTI_STAGE_PROGRAM\|SUPPORTED_FEATURES" crates/tiler-artifact/src/program/codec/model.rs` shows the key derived at the projector and absent from `SUPPORTED_FEATURES`. Its own doc comment states the reason: the neutral program section carries a program's canonical *identity* and not its dependency graph, so entries reach a reader in canonical stage-key order — identity's order, not execution order. Emitting the feature and refusing to read it is the fail-closed form of that gap; treating declaration order as execution order would be the silent one.

**Fact — the owner this inherited from is closed, and nothing live replaced it.** `carry-reconstructable-kernel-programs-in-the-neutral-envelope` is `done`. Its decision named "carrying execution order and dependency obligations explicitly" as a consequence of choosing the dispatch record, and [`expose-the-dispatch-record-on-a-decoded-artifact`](expose-the-dispatch-record-on-a-decoded-artifact.md) implemented every other part of that record. `grep -rln "multi-stage" tickets/` names seven ticket files and no live owner for this gap, which is why it is filed rather than assumed to be tracked.

**Why it is load-bearing rather than a nicety.** [`route-the-runtime-loader-through-the-dispatch-record`](route-the-runtime-loader-through-the-dispatch-record.md) records that its loader "genuinely cannot sequence a multi-entry variant" and keeps an `UnroutableEntries` refusal that is unreachable only because the decoder rejects such an envelope one layer earlier. A loader correct only by another layer's refusal is not correct. `tiler-compiler`'s materialized plans do produce multi-stage variants, so today a program's fused alternative can travel in an envelope and its materialized alternative cannot.

## Scope

Decide and encode what a reader needs in order to sequence a multi-stage variant: the stages' execution order and the dependency obligations between them, as encoded facts a decoder validates, in the same posture as every other dispatch-record row. This is a new encoded fact, so it is a manifest schema step and an identity-domain step — each stating its reason at its own site — rather than an accessor over rows that already exist.

**Do not close it by ordering entries at the encoder and declaring declaration order authoritative.** That is exactly the silent form the current refusal exists to avoid: the order a producer wrote is not a fact a decoder can check, and a consumer sequencing by position would have no way to verify the position meant what it assumed. Whatever is carried must either be checkable against something the envelope already proves, or be derived by the builder from the program's own dependency graph the way `binding_target` is derived from its stage access.

## Closes when

A consumer holding only encoded bytes can sequence a multi-stage variant's entries and name the dependency each edge rests on; `tiler.artifact.feature.multi-stage-program` is either supported or replaced by a key naming what actually remains unsupported; the refusal this build relies on is removed or restated as a narrower one with its reason; `docs/artifact-abi.md`'s required-feature table and item 3 of "Where the implemented profile is narrower than this contract" are updated to match; and `make full` passes.
