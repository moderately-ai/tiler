---
id: check-the-retained-gather-resolution-in-the-reference-evaluator
title: Check the retained gather resolution in the reference evaluator
status: todo
priority: p2
dependencies: []
related: [decide-how-the-oracle-independently-checks-a-gather-proof-identity, admit-the-selected-data-dependent-index-representation, bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, gather, reference, identity]
---
## User-visible outcome

`IndexRegionEvaluator::gather` consults the retained gather bounds resolution instead of ignoring it, and a static resolution's proof is checked to bind *this* region, *this* access, and *these* operands before the evaluation trusts anything about it.

## Why this exists

Filed 2026-08-22 by `worker-oracleid2` out of the readiness gate on [`decide-how-the-oracle-independently-checks-a-gather-proof-identity`](decide-how-the-oracle-independently-checks-a-gather-proof-identity.md), whose audit found the real gap. Read that packet's audit section before starting; it carries the derivation and the compiler evidence.

**Fact — the evaluator ignores the resolution entirely, while its own doc comment says otherwise.** At `f69829143a387a8e117858dbcaad416715f7e788`, `bounds_resolution`, `statically_proved`, and `invocation_validation_required` occur nowhere in `crates/tiler-reference/src/` — only three times, all in `crates/tiler-reference/tests/index_region_oracle.rs`. Reproduce with `grep -rn "bounds_resolution\|statically_proved\|invocation_validation_required" crates/tiler-reference/`. Meanwhile `crates/tiler-reference/src/oracle.rs` carries a doc comment at the anchor `oracle that trusted a proof would stop being an independent check of it`, which claims a static resolution is also checked defensively. The defensive *bounds* check is genuinely present and is not the gap; the resolution is simply never read.

**Fact — no new public surface is needed, and this was proved by compiling it.** Every component the proof identity binds is comparable from outside `tiler-ir` by typed value: `GatherIndexBoundsProof` exposes `region()`, `access()`, `source()`, `index()`, `source_type()`, `index_type()`, `source_shape()`, `index_shape()`, `result_shape()`, `axis()`, `source_extent()`, `domain()`, `kind()`, and `facts()`, all `pub`; `VerifiedIndexRegion::canonical_identity()` is `pub`; and `CanonicalIndexRegionIdentity`, the three verified handle types, `Shape`, `Extent`, `Axis`, `ResolvedValueType`, and `IndexDomainFactSource` all derive `Eq`. The parent packet records the out-of-workspace probe that compiles this comparison.

**Fact — the identity bytes are deliberately not re-derived.** They cannot be, and should not be. Three of the encoder's inputs are unreachable downstream (`E0624`, recorded in the parent packet), and re-deriving content would weaken what makes holding an identity evidence: its provenance, not its content. Comparing the components against the evaluator's own reading of the region is the stronger check and names the field that disagreed.

**Boundary against the compiler lane — read this before starting.** [`bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence`](bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence.md) is repairing the *same class* of gap one layer down: `gather_accesses_match` in `crates/tiler-compiler/src/physical.rs` elides the `proof` field, so a proof minted for one region can be attached to a shape-compatible different occurrence. That lane owns the compiler-side refusal, and [`record-that-schedule-rule-8-cannot-check-the-proofs-region-identity`](record-that-schedule-rule-8-cannot-check-the-proofs-region-identity.md) records why schedule rule 8 structurally cannot do it. **This ticket is not a duplicate and must not repair either site**: it is the independent oracle's own check, which exists precisely so a compiler-side refusal is not the only thing standing between a transplanted proof and a trusted result. Coordinate on ordering, and if that lane's work makes any part of this one unreachable, say so rather than asserting coverage.

## Required work

- Re-audit both Facts at your own base first, per AGENTS.md, and report a per-Fact verdict before editing.
- In `IndexRegionEvaluator::gather`, read the resolution and branch on both arms totally. A static resolution: compare every component the proof binds against the evaluator's own values — region identity, access id, source and index tensor ids, both value types, all three shapes, axis, source extent, the ordered domain, the kind against an independent classification from the bound shapes, and the fact source. A requirement resolution: keep today's behaviour, minting no receipt or proof.
- Keep the defensive bounds check unconditional. A proof that binds correctly still does not exempt the read; that is the point of an oracle.
- Add a typed error for a proof that does not bind the gather being evaluated. Do not reuse `MalformedRegion` if a distinguishable name is warranted — decide by reading the surrounding refusal vocabulary, and say what you chose and why.
- Size the component list against `GatherIndexBoundsSubject` by reading it, and state which components you compared and why that set is complete. A hand-written census is only as good as that argument.

## Non-goals

Widening any `tiler-ir` public surface — the packet establishes none is needed, and adding one would need Tom. Re-deriving identity bytes downstream. The encoder drift alarm, which is [`destructure-the-gather-bounds-subject-in-its-identity-encoder`](destructure-the-gather-bounds-subject-in-its-identity-encoder.md).

## Closes when

The evaluator consults the retained resolution on both arms; a static resolution's proof is checked to bind the gather being evaluated across every component the identity frames; the defensive bounds check still runs unconditionally; and a perturbation shows the new check failing with its message quoted — transplant a proof from a structurally different region and show what the evaluator said.
