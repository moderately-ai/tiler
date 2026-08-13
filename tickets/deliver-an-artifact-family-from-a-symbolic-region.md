---
id: deliver-an-artifact-family-from-a-symbolic-region
title: Deliver an artifact family from a region with symbolic extents
status: in-progress
priority: p1
dependencies: [admit-live-extent-operands-to-payload-indexing]
related: [carry-symbolic-extents-into-the-semantic-program, prototype-inline-aot-integration-proof, carry-live-extent-operands-through-the-artifact-envelope, prove-one-live-extent-artifact-payload-and-pipeline-at-two-n, admit-symbolic-extents-through-compiler-region-formation]
scopes: [implementation/frontend, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, inline-dx, shapes, milestone-0b]
claimed_from: todo
assignee: worker-deliver-symbolic
lease_expires_at: 1786664532
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

## Fact audit — 2026-08-13 at base `0b3ca334793e3975a2057f18424def2c251b1202`

Re-read this session: `crates/tiler-macros/src/aot.rs` (`AotRefusal`, `deliver`, `program_interface_is_symbolic`, `rendered_refusal`), `crates/tiler-macros/src/lib.rs` (`expand` always passes `expansion.program.verified()`), `crates/tiler-macros/src/region.rs` (`ProgramEvidence` is the single `Verified` arm), `crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.{rs,stderr}`, `docs/integration/frontends.md`, `crates/tiler-compiler/src/pipeline.rs` (`target_failure` after `first_symbolic_extent` when the program does not carry a parametric broadcast), `crates/tiler-compiler/src/request.rs` (`a_symbolic_elementwise_neighbour_reaches_region_formation`). Purpose unchanged: lift the frontend gate only when the program is constructible **and** compilable; otherwise keep a typed spanned refuse.

- **False.** Display no longer names done research `carry-symbolic-extents-into-the-semantic-program`. At this base it already named this ticket (`deliver-an-artifact-family-from-a-symbolic-region` is the work that removes this restriction), retargeted by [`repair-the-records-the-symbolic-region-construction-landing-falsifies`](repair-the-records-the-symbolic-region-construction-landing-falsifies.md). Durable former anchor: `` `deliver-an-artifact-family-from-a-symbolic-region` is the work that removes this ``.
- **Imprecise.** The `program.ok_or(AotRefusal::SymbolicExtent)` arm was only reachable if `deliver` was handed `None`. `expand` always passed a verified program. The live frontend gate was `program_interface_is_symbolic` after construction. Durable former anchors: `AOT delivery still needs every extent known at expansion time` and `program.ok_or(AotRefusal::SymbolicExtent)`.
- **Verified, then lifted.** Same-shape symbolic elementwise constructs. `compile()` still declines at schedule: `UnsupportedSymbolicExtent { phase: "schedule", rule: "symbolic-extent" }` because `IndexRegion.iteration_shape` is a fixed `Shape`. Live-extent operands exist on the hand-built `ScheduledRegion` / `LiveRowMajor` path, not on this frontend's `session::compile` path. Durable anchors: `A sourced broadcast must reach physical selection` and `IndexRegion requires a fixed geometry`.
- **Verified.** `docs/integration/frontends.md` status paragraph still claimed a disabled cache refuses with a spanned error. `TILER_EXPANSION_CACHE_DIR=off` delivers and publishes no file (`a_disabled_cache_delivers_the_region_and_publishes_no_file`, ADR 0089).

## Implementation record — 2026-08-13

The frontend-local `AotRefusal::SymbolicExtent` gate is gone. `deliver` takes the verified `&SemanticProgram` an expansion always has. A constructible symbolic region reaches `tiler_compiler::session`. Same-shape elementwise is recognized and formed; `session::compile` then returns `CompileFailure` as `AotRefusal::Compile` with `CompileFailureClass::UnsupportedCapability { rule: "symbolic-extent" }`. `rendered_refusal` names that declined case rather than an unrecognized program shape. The trybuild golden still spans the `deliver` keyword.

Delivery of one artifact family from `sym n` + `deliver macos;` is **not** claimed. `IndexRegion` still requires a fixed launch geometry; teaching `compile()` to emit `LiveRowMajor` over a rank-1 `[n]` region is a public IR / compiler change this ticket's scopes do not own. Lifting the frontend gate into a silent fallback would have been the defect the brief names.

### Evidence

- `a_symbolic_region_reaches_the_compilers_typed_decline` — constructed `sym n` elementwise is `Compile` / `symbolic-extent`; the retired "needs every extent known at expansion time" sentence is absent.
- `crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.stderr` — spanned `compile_error!` on `deliver macos;` naming the compiler's schedule refuse.
- Literal-region cold/warm cache, wrong-entry typed refuse, and damaged-entry quarantine tests are unchanged; they still run over `approved_region()`.
- Identity-across-extents hash: **none**. No artifact is produced for a symbolic region, so there is no identity to hash. A compiled plan specialized on a bound value would have been a different (wrong) program.

### Perturbation

Subject, not assertion: restoring `program_interface_is_symbolic` before `compile()` makes `a_symbolic_region_reaches_the_compilers_typed_decline` panic `unexpected refusal: SymbolicExtent` — except that variant is gone, so the restored gate cannot even name itself. Removing the `symbolic-extent` arm of `rendered_refusal` makes the same test fail:

```
the diagnostic must name the declined case, not an unrecognized program: this region denotes a whole program the compiler does not recognize
```
