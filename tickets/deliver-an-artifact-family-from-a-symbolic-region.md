---
id: deliver-an-artifact-family-from-a-symbolic-region
title: Deliver an artifact family from a region with symbolic extents
status: todo
priority: p1
dependencies: [admit-live-extent-operands-to-payload-indexing]
related: [carry-symbolic-extents-into-the-semantic-program, prototype-inline-aot-integration-proof]
scopes: [implementation/frontend, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, inline-dx, shapes, milestone-0b]
---
## User-visible outcome

A region declaring `sym n` states `deliver macos;` and reaches the same expansion-time AOT flow a literal region reaches — compiled, cached, embedded, decoded, and routed — with **one** artifact identity across every extent the consumer later binds.

## Why this exists

**Fact.** The refusal exists and names this chain by id. `crates/tiler-macros/src/aot.rs:223` renders "this region declares a symbolic extent, and a `deliver` statement selecting an artifact family compiles the region ahead of time — which needs every extent to be known at expansion time", and `crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.stderr` is the byte-compared golden. `prototype-inline-aot-integration-proof`'s boundary packet lists it as observable change 4.

**Fact.** Everything downstream already works for a literal region: the measured proof compiles through `xcrun`, publishes a 49,432-byte bundle, hits the cache warm with zero compiler runs, embeds one `MTLB` payload into the produced binary, and routes it with the cache root deleted.

**Fact — dependency correction from the KV layout trace.** Artifact-side `AbiRoot::InputExtent` evaluation can size ranges and launches, but the structured-kernel/Metal signature carries no live scalar into payload address or loop arithmetic. `admit-live-extent-operands-to-payload-indexing` is therefore a hard dependency: this ticket cannot truthfully deliver one compiled symbolic payload while only the host side consumes the symbol.

## Implementation keys

- Lift the refusal only when the region's program is genuinely constructible and compilable. The diagnostic must not become reachable-but-wrong: if a symbolic region can be built and not compiled, the refusal moves to the compiler's typed decline rather than disappearing.
- One artifact for every bound extent. The packaged program must specialize on no extent, and that is a testable property rather than a design intention — assert one artifact identity across a span of bound extents, mirroring L5's own stated check for eight decode steps.
- The ABI expressions for accessible byte range and launch geometry are formulas over bound extents, evaluated at preflight. A failure there is a refusal, not a post-commit surprise; it stays pre-commit.
- `docs/integration/frontends.md` currently states that a statement selecting a family is refused for a symbolic region. Correct it in the same change. The previous ticket flagged two other now-false sentences in that file without editing them because it lacked the scope; this ticket holds `contracts/integrations` and must sweep them.
- Do not widen the delivery vocabulary. Family count, minimum, and language standard stay exactly where `prototype-inline-aot-integration-proof` left them; widening is `deliver-several-artifact-families-from-one-expansion`'s.

## Evidence

- An out-of-tree consumer crate declaring only `tiler`, containing one symbolic `tensor!` with `deliver macos;`, compiles with no `build.rs`, no `include_bytes!`, and one dependency line, and its binary contains the metallib magic exactly once.
- A span of bound extents over that one artifact yields one artifact identity, asserted by hash.
- The cold and warm cache behaviour matches the literal region's measured behaviour, with the same two perturbations — a semantically wrong entry as a typed refusal, a damaged entry quarantined and rebuilt — each watched failing first.
- A symbolic region the compiler declines still produces a spanned diagnostic naming the declined case, so lifting the gate did not convert a refusal into a silent fallback.

## Public boundary

The observable change is that a previously refused invocation now compiles and embeds. The `deliver` grammar is unchanged; the removed diagnostic and the corrected contract sentences are the packet.
