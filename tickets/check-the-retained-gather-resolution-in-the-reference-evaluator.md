---
id: check-the-retained-gather-resolution-in-the-reference-evaluator
title: Check the retained gather resolution in the reference evaluator
status: done
priority: p2
dependencies: []
related: [decide-how-the-oracle-independently-checks-a-gather-proof-identity, admit-the-selected-data-dependent-index-representation, bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, gather, reference, identity]
---
## User-visible outcome

`IndexRegionEvaluator::gather`'s doc comment states what the function actually does, and the reason a static resolution needs no check there — that one cannot reach the function at all — is pinned by a test that fails when either exclusion stops holding.

**Revised 2026-08-22 by `worker-refeval` after the source audit below.** The original outcome asked the evaluator to consult the resolution and compare every component a static proof binds. That work was dropped, and the reason is a finding rather than a descope: **a statically proved gather cannot reach `IndexRegionEvaluator::gather`**, so the comparison would have been a branch that provably never executes, guarded by a public error variant that could never be constructed. AGENTS.md requires stating what it would take for a check to say *no* and confirming that case is reachable; here it is not, and the honest deliverable is the doc repair plus a reachability pin. See `## Outcome` for the evidence.

## Why this exists

Filed 2026-08-22 by `worker-oracleid2` out of the readiness gate on [`decide-how-the-oracle-independently-checks-a-gather-proof-identity`](decide-how-the-oracle-independently-checks-a-gather-proof-identity.md), whose audit found the real gap. Read that packet's audit section before starting; it carries the derivation and the compiler evidence.

**Fact — the evaluator ignores the resolution entirely, while its own doc comment says otherwise.** At `f69829143a387a8e117858dbcaad416715f7e788`, `bounds_resolution`, `statically_proved`, and `invocation_validation_required` occur nowhere in `crates/tiler-reference/src/` — only three times, all in `crates/tiler-reference/tests/index_region_oracle.rs`. Reproduce with `grep -rn "bounds_resolution\|statically_proved\|invocation_validation_required" crates/tiler-reference/`. Meanwhile `crates/tiler-reference/src/oracle.rs` carries a doc comment at the anchor `oracle that trusted a proof would stop being an independent check of it`, which claims a static resolution is also checked defensively. The defensive *bounds* check is genuinely present and is not the gap; the resolution is simply never read.

**Fact — no new public surface is needed, and this was proved by compiling it.** Every component the proof identity binds is comparable from outside `tiler-ir` by typed value: `GatherIndexBoundsProof` exposes `region()`, `access()`, `source()`, `index()`, `source_type()`, `index_type()`, `source_shape()`, `index_shape()`, `result_shape()`, `axis()`, `source_extent()`, `domain()`, `kind()`, and `facts()`, all `pub`; `VerifiedIndexRegion::canonical_identity()` is `pub`; and `CanonicalIndexRegionIdentity`, the three verified handle types, `Shape`, `Extent`, `Axis`, `ResolvedValueType`, and `IndexDomainFactSource` all derive `Eq`. The parent packet records the out-of-workspace probe that compiles this comparison.

**Fact — the identity bytes are deliberately not re-derived.** They cannot be, and should not be. Three of the encoder's inputs are unreachable downstream (`E0624`, recorded in the parent packet), and re-deriving content would weaken what makes holding an identity evidence: its provenance, not its content. Comparing the components against the evaluator's own reading of the region is the stronger check and names the field that disagreed.

**Boundary against the compiler lane — read this before starting.** [`bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence`](bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence.md) is repairing the *same class* of gap one layer down: `gather_accesses_match` in `crates/tiler-compiler/src/physical.rs` elides the `proof` field, so a proof minted for one region can be attached to a shape-compatible different occurrence. That lane owns the compiler-side refusal, and [`record-that-schedule-rule-8-cannot-check-the-proofs-region-identity`](record-that-schedule-rule-8-cannot-check-the-proofs-region-identity.md) records why schedule rule 8 structurally cannot do it. **This ticket is not a duplicate and must not repair either site**: it is the independent oracle's own check, which exists precisely so a compiler-side refusal is not the only thing standing between a transplanted proof and a trusted result. Coordinate on ordering, and if that lane's work makes any part of this one unreachable, say so rather than asserting coverage.

## Required work

- Re-audit both Facts at your own base first, per AGENTS.md, and report a per-Fact verdict before editing.
- ~~In `IndexRegionEvaluator::gather`, read the resolution and branch on both arms totally. A static resolution: compare every component the proof binds against the evaluator's own values — region identity, access id, source and index tensor ids, both value types, all three shapes, axis, source extent, the ordered domain, the kind against an independent classification from the bound shapes, and the fact source. A requirement resolution: keep today's behaviour, minting no receipt or proof.~~ **Withdrawn**: the static arm is unreachable, so the branch could never execute. Evidence in `## Outcome`.
- Keep the defensive bounds check unconditional. A proof that binds correctly still does not exempt the read; that is the point of an oracle. **Unchanged and still true** — `decide_gather_index` runs on every loaded address with no branch on the resolution.
- ~~Add a typed error for a proof that does not bind the gather being evaluated.~~ **Withdrawn**: an unconstructible variant on a public `#[non_exhaustive]` enum is public surface that can never be exercised. The refusal vocabulary was read; no name was added.
- Repair the doc comment so it states what the function does and why the proved case needs nothing, and pin the reachability so the argument fails loudly if it stops holding.
- Size the component list against `GatherIndexBoundsSubject` by reading it. **Done, and it is 12 fields** — `region`, `access`, `source`, `index`, `source_type`, `index_type`, `source_shape`, `index_shape`, `result_shape`, `axis`, `source_extent`, `domain` — plus `kind` and `facts` on the proof, giving the fourteen the packet names. All fourteen are reachable downstream; the list is moot because the comparison is, not because the surface is missing.

## Non-goals

Widening any `tiler-ir` public surface — the packet establishes none is needed, and adding one would need Tom. Re-deriving identity bytes downstream. The encoder drift alarm, which is [`destructure-the-gather-bounds-subject-in-its-identity-encoder`](destructure-the-gather-bounds-subject-in-its-identity-encoder.md).

## Closes when

**Revised.** The original closing condition — "transplant a proof from a structurally different region and show what the evaluator said" — is not satisfiable, and that is the finding rather than an obstacle: there is no transplant vector at this layer. `finish_compaction` in `crates/tiler-ir/src/index/builder/compact.rs` is the sole constructor of a `VerifiedGatherReadAccessData`, and it derives the bounds resolution from the same access, in the same call, from the region identity it has just computed. The fields are module-private and there is no deserializer, so no public API can pair a region with a foreign proof. The transplant vector the packet is right to worry about lives one layer up, in schedule state, and is [`bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence`](bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence.md)'s to close.

Closes when: the doc comment states that the resolution is never read and why the proved case needs no handling; `a_static_gather_proof_cannot_reach_the_evaluator` pins both exclusion legs and sizes the proof-kind vocabulary from the type; the defensive bounds check still runs unconditionally; and each guarded property has been perturbed separately with its failure text quoted.

## Outcome

**Per-Fact verdict at `3e6cc78ea56b54518e1b22c1fe076e523e201a1a`.**

*Fact 1 — the evaluator ignores the resolution, while its doc says otherwise.* **Verified as to the code; imprecise as to the doc.** `grep -rno "bounds_resolution\|statically_proved\|invocation_validation_required" crates/tiler-reference/src/ | wc -l` returns **0 occurrences**; the same pattern over `crates/tiler-reference/` returns **4 occurrences on 3 lines**, all in `tests/index_region_oracle.rs`. The ticket's "only three times" counted *lines*; `grep -c` counts lines, not occurrences, and the unit is stated here as occurrences. The anchor `oracle that trusted a proof would stop being an independent check of it` resolves in `crates/tiler-reference/src/oracle.rs`. The doc's literal claim — "Bounds are checked here regardless of the retained resolution" — was **true**, and the bounds check is genuinely unconditional. What was wrong is the implicature of the following clause, which enumerates two arms as if a static resolution arrives and is handled defensively. So this was a misleading doc, not a false one, and it has been repaired to say that the resolution is never read and that the proved case cannot arrive.

*Fact 2 — no new public surface is needed, proved by compiling it.* **Verified, and now moot.** All fourteen accessors are `pub` in `crates/tiler-ir/src/index/model.rs`, `VerifiedIndexRegion::canonical_identity()` is `pub`, `gather_result_shape` is `pub` and re-exported from `crates/tiler-ir/src/semantic.rs`, and `TensorAccessRef::domain()` supplies the ordered domain. The comparison the surface enables is nevertheless unreachable, so the Fact is correct and no longer load-bearing.

**The finding: a statically proved gather cannot reach `IndexRegionEvaluator::gather`.** Both kinds of `GatherIndexBoundsProofKind` are excluded, for independent reasons.

- `VacuousEmptyResultDomain` **excludes itself**. It is minted exactly when a derived result extent is zero. `gather_read` requires the access domain's extents to equal the derived result extents *as a multiset* — authoring the zero away with a constant coordinate is refused with `GatherDomainShape { expected: Shape([Extent(0), Extent(3)]), actual: Shape([Extent(3)]) }` — and a write root consuming the gathered value must cover that domain, with a narrower root refused at `build()`. So the zero extent reaches `DomainWalk::new`, whose `let exhausted = extents.contains(&0)` ends the walk before any point. Instrumenting the entry of `gather` and evaluating the vacuous region printed no entry line at all, while the evaluation returned `Ok`.
- `U32RangeContainedBySourceExtent` is excluded by the reference tensor budget. It needs a gathered axis of at least `2^32`; `MAX_REFERENCE_TENSOR_ELEMENTS` is `2^24`. Constructing the source returns `Err(ResourceExceeded { resource: TensorElements, limit: 16777216, actual: 17179869184 })`, so no binding for such a region can exist.

Every evaluable gather fixture in the suite reports `REQUIREMENT`.

**Landed.** The doc comment on `fn gather`, and `a_static_gather_proof_cannot_reach_the_evaluator` in `crates/tiler-reference/tests/index_region_oracle.rs`, which sizes `GATHER_PROOF_KINDS` with `std::mem::variant_count` so a third closed argument is a compile error at the declaration.

**Perturbations, each separate, with quoted failure text.**

1. *Third proof kind added.* `expected an array with a size of 3, found one with a size of 2`.
2. *Vacuity rule reverted to the index shape alone.* ``assertion `left == right` failed: the fixture must retain the kind whose exclusion it tests`` with `left: None` and `right: Some(VacuousEmptyResultDomain)`.
3. *`DomainWalk::new` made to walk an empty domain.* `an empty result domain walks no point, so the poison is never read: GatherIndexOutOfBounds { access: ..., index_offset: 0, value: 4294967295, extent: 5 }` — the poisoned index operand is what makes this leg load-bearing.
4. *`U32_UNIVERSE` raised to `1 << 40`.* `left: None / right: Some(U32RangeContainedBySourceExtent)`.
5. *`MAX_REFERENCE_TENSOR_ELEMENTS` raised to `1 << 35`.* `a source spanning the U32 universe exceeds the reference tensor budget`.

**Identity.** No identity value moves. Nothing in `crates/tiler-ir` was modified; the landed change is one doc comment in `crates/tiler-reference/src/oracle.rs` and one test. No encoder, ledger, pin, or golden is touched.
