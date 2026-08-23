---
id: lower-a-recognized-gather-through-a-governed-capability
title: Lower a recognized gather through a governed index-access capability
status: in-progress
priority: p1
dependencies: []
related: [carry-the-gather-relation-through-the-compiler-vertical, thread-resolved-lowering-into-the-governed-spelling-path, decide-the-data-dependent-index-representation-public-surface, emit-the-indirect-gather-on-metal]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, gather, lowering, identity, public-boundary]
claimed_from: todo
assignee: worker-gathercap
lease_expires_at: 1787468243
---
## User-visible outcome

A recognized gather occurrence resolves an installed lowering capability and refines to a verified index region carrying its own statically proved `GatherIndexBoundsProof`, so the proof a scheduled gather must embed exists in `ResolvedLowering` at planning time.

## Why this exists

Filed 2026-08-23 by `worker-thread` while auditing [`thread-resolved-lowering-into-the-governed-spelling-path`](thread-resolved-lowering-into-the-governed-spelling-path.md), which was dispatched on the premise that the proof "sits in `ResolvedLowering` in the same `plan_target` scope — the value exists at the right time and no seam carries it". **That premise is false at `9b61b563`: the value does not exist, for any gather, ever.** No installed capability lowers a gather, so `resolve_lowering` refuses the whole compilation before physical planning is reached.

This is remainder item 2 of [`carry-the-gather-relation-through-the-compiler-vertical`](carry-the-gather-relation-through-the-compiler-vertical.md) — "**The governed lowering capability row (21 to 22)** and its `GovernedGatherF32` provider. Independent of (1) and the correct next lane." — which was written down in that ticket's Remainder and **never filed as a ticket**. Every open gather lane above it inherited the gap.

## Facts, each read at `9b61b563` in the file it names

**Fact — the compiler's lowering facade cannot emit a gather.** `IndexAccessLoweringContext` (`crates/tiler-compiler/src/capability.rs`, anchor `pub struct IndexAccessLoweringContext<'a> {`) exposes `dimension`, `input_tensor`, `output_tensor`, `constant`, `dimension_expr`, `linear_combination`, `floor_div`, `modulo`, `read`, `write`, `apply`, `apply_in`, `reduce`, and `output`. There is **no `gather_read`**, and `grep -rn "fn gather_read" crates/` finds it only at `crates/tiler-ir/src/index/builder.rs` (the IR builder), `crates/tiler-ir/src/index/law.rs`, `crates/tiler-ir/src/index/model.rs`, and one reference-oracle test helper. `git log -S "gather_read" -- crates/tiler-compiler/src/capability.rs` is **empty**: the facade method was drafted and withdrawn inside the parent lane and never landed.

**Fact — the governed registry carries no gather row.** `grep -rn -i "gather" crates/tiler-compiler/src/governed.rs` returns exactly one hit, a comment at line 1868 about a reindex that is "not a gather". `grep -rn -i "gather" crates/tiler-compiler/src/capability.rs` returns **zero**. `GOVERNED_INDEX_ACCESS_CAPABILITIES = 21` is unchanged.

**Fact — so `resolve_lowering` refuses every gather, and the tree already asserts it.** `a_gather_occurrence_resolves_no_lowering_at_this_base` in `crates/tiler-compiler/src/request/tests.rs` requires `resolve_lowering` to answer `Err` with reason `missing-capability` for a gather program, and carries a negative control proving the same call answers `Ok` for an elementwise fixture. It passes at this base.

**Fact — the refusal is *before* physical planning, not inside it.** `enumerate_complete_plans` in `crates/tiler-compiler/src/pipeline/planning.rs` opens with `let lowering = match resolve_lowering(semantic, verified) {` and returns `Err(lowering_failure(&source, cause))` on the error arm — above `enumerate_covers`, above `enumerate_frontier`, above `govern_spelling` and `spell_region`. So **no gather member set can reach the region-vocabulary check through the pipeline at all**; the only live caller of `spell_region` with gather members is a unit test that hands it a request directly.

**Fact — the end-to-end test already names this ticket's work as the prior authority.** `a_governed_gather_refuses_at_dispatch_before_governed_lowering` (same file) pins the widened-profile compile at `("lowering", "missing-capability")` and its own doc names two authorities standing between that refusal and one that could grant a gather a schedule: the governed capability row itself, and `RegionVocabularyWall::GatherProofUnavailable`, *"which is what physical planning answers once a row exists"*. The row is first. That doc comment wraps across `///` lines, so the anchor is the single-line fragment `authorities stand between this refusal and one that could`; the full sentence greps to zero and would read as false absence.

## Required work

- Re-audit every Fact above at your base with a per-Fact verdict before editing. The counts and the empty greps are the load-bearing ones.
- Land `IndexAccessLoweringContext::gather_read` to the accepted spelling in [`decide-the-data-dependent-index-representation-public-surface`](decide-the-data-dependent-index-representation-public-surface.md), **and the governed row that consumes it in the same change** — the parent lane withdrew the facade precisely because "a public surface with no consumer is not decision-ready", and re-landing it alone would repeat that.
- Land the `GovernedGatherF32` provider and its capability row, moving `GOVERNED_INDEX_ACCESS_CAPABILITIES` 21 to 22.
- Establish that the emitted region refines: `refine` must produce `OccurrenceEvidence::Refined` whose `IndexRefinement::single_region()` is `Some`, and whose gather access exposes `bounds_resolution().statically_proved()`. A gather whose bounds obligation is *not* statically discharged mints a validation requirement instead and must stay refused — that population belongs to [`admit-an-invocation-scoped-gather-index-validation-receipt`](admit-an-invocation-scoped-gather-index-validation-receipt.md), not here.
- Flip `a_gather_occurrence_resolves_no_lowering_at_this_base` and the widened half of `a_governed_gather_refuses_at_dispatch_before_governed_lowering` in the same change. Both currently assert the opposite of what this ticket delivers; leaving either would leave the suite asserting a falsehood. **Do not touch `UNPLANNED_OPERATIONS` or `gather_is_absent_from_the_governed_fusion_roles`** — the parent lane verified both remain true and recorded that flipping them asserts the opposite of the tree.

## Public boundary

**`IndexAccessLoweringContext::gather_read` is a `pub` method on a `pub` type**, re-exported to out-of-crate lowering providers. That meets ADR 0075's reservation bar, so its exact signature is Tom's unless it lands verbatim as the already-accepted packet spells it. State which, with the anchor, before writing it.

## Identity domains

**The request-subject domain steps for every program in the repository, not only for gathers.** `CanonicalLoweringRegistryIdentity` folds the capability list, and `crates/tiler-compiler/src/request/subject.rs` writes it into every request subject: `push_slice(&mut bytes, self.lowering_registry.as_bytes());`. Adding one row therefore moves every pinned request-subject golden and everything keyed on one. Derive the full consequence on the merged tree and recompute the pins there rather than on your base. This blast radius is the reason the row is its own lane rather than a detail of the frontier work.

## Non-goals

`physical::gather_region`, the `govern_spelling` gather arm, and retiring `RegionVocabularyWall::GatherProofUnavailable` — those are [`thread-resolved-lowering-into-the-governed-spelling-path`](thread-resolved-lowering-into-the-governed-spelling-path.md), which this unblocks. The invocation-validation vocabulary. Any Metal, artifact, manifest, or cache surface. Re-opening the accepted data-dependent index surface.

## Closes when

A recognized gather resolves a governed capability, refines to a single verified index region whose gather access carries a statically proved bounds resolution, `resolve_lowering` answers `Ok` for a gather program, no test asserts the absence this lane removes, every identity consequence is derived on the merged tree with pins recomputed there, each new refusal has been watched firing on a perturbed subject, and the workspace gate is green.

## Coordinator pre-dispatch note — 2026-08-23 at `5b2e4414`: the accepted spelling exists, and where to find it

Every Fact in this ticket reproduced for the coordinator: `gather_read` is absent from `crates/tiler-compiler/src/capability.rs` (**0** hits), `git log -S "gather_read"` against that file is **empty**, `GOVERNED_INDEX_ACCESS_CAPABILITIES` is **21** at `crates/tiler-compiler/src/governed.rs`, and `enumerate_complete_plans` states the ordering itself at the anchor `Lowering-capability resolution precedes every cover`.

**The Public boundary section asks the worker to state whether the signature is Tom's or already accepted. It is already accepted, and this ticket did not name where.** [`decide-the-data-dependent-index-representation-public-surface`](decide-the-data-dependent-index-representation-public-surface.md) is `done`, and its acceptance record states that Tom accepted **option B** with *"the three exact public `gather_read` refusals for each nonliteral dimension"*, fixing the accepted surface as *"exactly the reviewed packet at `a25f4268b768f1b0391db34798676f910d4f1660`"*. That ticket spells `pub fn gather_read(` in two places. **Land it verbatim from there.** Any deviation from that spelling is a new public boundary and therefore Tom's — stop and report rather than adjusting it.

**A related edge this ticket was missing, which matters for not re-deriving landed work.** The acceptance routes implementation through [`admit-the-selected-data-dependent-index-representation`](admit-the-selected-data-dependent-index-representation.md), which is **`done`** — and the IR half already landed: `pub fn gather_read(` exists at `crates/tiler-ir/src/index/builder.rs`. So this ticket is **not** a duplicate of it, and the distinction is exact: the *IR builder's* `gather_read` is landed; the *compiler's* `IndexAccessLoweringContext::gather_read` facade method is the one that was drafted and withdrawn. Read the landed IR method before writing the facade — its refusals are the accepted three and the facade must not restate them differently.

**Unverified by the coordinator and left for the worker:** the refinement claim (`OccurrenceEvidence::Refined`, `single_region()`, `statically_proved()`), the two test flips, and the full identity blast radius. The identity consequence in particular is stated as reaching every request subject in the repository; derive it on the merged tree and recompute pins there, as this ticket already says.
