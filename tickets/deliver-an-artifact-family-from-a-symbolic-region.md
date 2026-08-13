---
id: deliver-an-artifact-family-from-a-symbolic-region
title: Deliver an artifact family from a region with symbolic extents
status: todo
priority: p1
dependencies: [admit-live-extent-operands-to-payload-indexing]
related: [carry-symbolic-extents-into-the-semantic-program, prototype-inline-aot-integration-proof, carry-live-extent-operands-through-the-artifact-envelope, prove-one-live-extent-artifact-payload-and-pipeline-at-two-n, admit-symbolic-extents-through-compiler-region-formation]
scopes: [implementation/frontend, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, inline-dx, shapes, milestone-0b]
---
## User-visible outcome

A region declaring `sym n` states `deliver macos;` and reaches the same expansion-time AOT flow a literal region reaches — compiled, cached, embedded, decoded, and routed — with **one** artifact identity across every extent the consumer later binds.

## Why this exists

**Fact.** The refusal exists and still names the research parent by id. `AotRefusal::SymbolicExtent` in `crates/tiler-macros/src/aot.rs` renders "this region declares a symbolic extent, and a `deliver` statement selecting an artifact family compiles the region ahead of time — which needs every extent to be known at expansion time" (Display arm; gate is `program.ok_or(AotRefusal::SymbolicExtent)?` when `ProgramEvidence::verified()` is `None`), and `crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.stderr` is the byte-compared golden. `prototype-inline-aot-integration-proof`'s boundary packet lists it as observable change 4. The Display text still points at [`carry-symbolic-extents-into-the-semantic-program`](carry-symbolic-extents-into-the-semantic-program.md) as "the work that removes this restriction"; that research ticket is done — lifting the gate and retargeting the consumer-facing remedy id to this delivery chain is this ticket's work.

**Fact.** Everything downstream already works for a literal region: the integration-proof measurement under `prototype-inline-aot-integration-proof` compiled through `xcrun`, published a 49,432-byte bundle at that measurement date, hit the cache warm with zero compiler runs, embedded one `MTLB` payload into the produced binary, and routed it with the cache root deleted. Treat 49,432 as that dated measurement, not a live pin of current bundle identity size at this base.

**Fact — dependency correction from the KV layout trace.** Artifact-side `AbiRoot::InputExtent` evaluation can size ranges and launches, but the structured-kernel/Metal signature carries no live scalar into payload address or loop arithmetic. `admit-live-extent-operands-to-payload-indexing` is therefore a hard dependency: this ticket cannot truthfully deliver one compiled symbolic payload while only the host side consumes the symbol.

## Implementation keys

- Lift the refusal only when the region's program is genuinely constructible and compilable. The diagnostic must not become reachable-but-wrong: if a symbolic region can be built and not compiled, the refusal moves to the compiler's typed decline rather than disappearing. When the gate lifts, retarget the consumer-facing diagnostic remedy id (today still names the done research ticket `carry-symbolic-extents-into-the-semantic-program`) to this delivery ticket / chain.
- One artifact for every bound extent. The packaged program must specialize on no extent, and that is a testable property rather than a design intention — assert one artifact identity across a span of bound extents, mirroring L5's own stated check for eight decode steps.
- The ABI expressions for accessible byte range and launch geometry are formulas over bound extents, evaluated at preflight. A failure there is a refusal, not a post-commit surprise; it stays pre-commit.
- `docs/integration/frontends.md` currently states that a symbolic-extent region under a selected family is refused at the AOT stage — still true, and this ticket flips that sentence when the gate lifts. The two sentences the AOT proof originally flagged without editing (selected family refused; cache-root uncalled) are already corrected in the status paragraph. While holding `contracts/integrations`, still (a) flip the symbolic-refusal bullet on landing, (b) retarget the "work that removes this restriction" link from the done research ticket to this delivery ticket / chain, and (c) correct the still-false status-paragraph claim that a disabled cache refuses with a spanned error — `TILER_EXPANSION_CACHE_DIR=off` delivers and publishes no file (ADR 0089 restored meaning).
- Do not widen the delivery vocabulary. Family count, minimum, and language standard stay exactly where `prototype-inline-aot-integration-proof` left them; widening is `deliver-several-artifact-families-from-one-expansion`'s.

## Evidence

- An out-of-tree consumer crate declaring only `tiler`, containing one symbolic `tensor!` with `deliver macos;`, compiles with no `build.rs`, no `include_bytes!`, and one dependency line, and its binary contains the metallib magic exactly once.
- A span of bound extents over that one artifact yields one artifact identity, asserted by hash.
- The cold and warm cache behaviour matches the literal region's measured behaviour, with the same two perturbations — a semantically wrong entry as a typed refusal, a damaged entry quarantined and rebuilt — each watched failing first.
- A symbolic region the compiler declines still produces a spanned diagnostic naming the declined case, so lifting the gate did not convert a refusal into a silent fallback.

## Public boundary

The observable change is that a previously refused invocation now compiles and embeds. The `deliver` grammar is unchanged; the removed diagnostic and the corrected contract sentences are the packet.

## Fact audit — 2026-08-10 at base `c99ac54950f2`

- Line citation `crates/tiler-macros/src/aot.rs:223` had drifted (that line is an import); durable anchors are `AotRefusal::SymbolicExtent` Display and the `program.ok_or(AotRefusal::SymbolicExtent)` gate.
- Consumer-facing remedy still names done research `carry-symbolic-extents-into-the-semantic-program`; retarget on landing.
- The AOT proof's "two other now-false sentences" inventory is obsolete; remaining contract falsehood under this ticket's scope is the disabled-cache refusal claim, plus the symbolic bullet that becomes false only when this ticket lands.
