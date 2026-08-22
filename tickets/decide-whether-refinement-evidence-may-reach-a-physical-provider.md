---
id: decide-whether-refinement-evidence-may-reach-a-physical-provider
title: Decide whether refinement evidence may reach a physical provider
status: in-progress
priority: p1
dependencies: []
related: [carry-the-gather-relation-through-the-compiler-vertical, decide-the-data-dependent-index-representation-public-surface, emit-the-indirect-gather-on-metal]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, gather, identity]
claimed_from: todo
assignee: worker-seam
lease_expires_at: 1787442342
---
## User-visible outcome

Physical planning can either obtain a scheduled gather's bounds proof through an accepted seam, or the repository records that it cannot and says what a provider gets instead — so the gather frontier is blocked by a decision on record rather than by a wall nobody chose.

## Why this exists

Filed 2026-08-22 by the coordinator when `carry-the-gather-relation-through-the-compiler-vertical` **stopped at a typed wall rather than minting a boundary**. That was the right call: it landed recognition, normalization, and request identity — a gather request now advances past arithmetic recognition on a U32-capable profile — and refused to invent the seam the frontier half needs.

**Fact — the wall is typed and named, not a silent gap.** `crates/tiler-compiler/src/physical.rs` declares `RegionVocabularyWall::GatherProofUnavailable`, renders it as `"gather-proof-unavailable"`, and returns it from the region-vocabulary check. Verified by the coordinator at three sites in that file.

**Fact — the blocking property is an identity binding, not an access-control choice.** A scheduled `BoundsProofKind::GatherSource` carries a `GatherIndexBoundsProof` that only the index layer's verifier-private deriver mints, and that proof **binds a `CanonicalIndexRegionIdentity`**. So physical planning cannot re-derive one without binding a schedule to a region nothing has lowered. The value exists at the right time — the delivering lane reports `resolve_lowering` and `enumerate_frontier` in the same scope — but **no seam carries it**, and `ImplementationContext` documents its provider surface exhaustively without refinement on it.

**Fact — the accepted packet does not answer this, and two of its citations are stale.** It has the frontier call `physical::gather_region` without saying where the proof comes from. Separately, the coordinator verified two of its references do not resolve: `crates/tiler-ir/src/index/refinement.rs` **is a 12-file directory, not a file** — the module-split false-absence hazard, inside an accepted packet — and `NormalizedOutput::epilogue()` does not exist, so there was nothing to update.

## The two questions, which are not the same

1. **May refinement evidence reach a physical provider at all?** This is a public-boundary question. `ImplementationContext`'s provider surface is enumerated deliberately; adding refinement to it widens what a third-party provider sees, and the identity binding means what it would see is not a free-standing fact but a claim about a specific region.
2. **How does `pipeline/verify.rs` obtain such a proof?** It re-derives rather than borrowing planning's copy **on purpose** — that independence is the property making verification meaningful. A seam that lets it borrow would quietly retire that property, which is a different decision from (1) and must not ride along with it.

## Required work

- Re-audit all three Facts at your base with a per-Fact verdict before writing packet prose.
- Apply AGENTS.md's decision-packet readiness gate in full. **Enumerate at your base rather than treating the two questions as the option set** — include the status quo, which is honest today: the wall is typed, the refusal is named, and no gather reaches a schedule.
- For each survivor: exactly what public surface it adds, whether a provider could use it to assert a proof it did not earn, what it costs `pipeline/verify.rs`'s independence, and the identity consequence.
- **Eliminate before ranking** any option that lets a provider supply or borrow a proof it cannot independently justify — that inverts the property the verifier-private deriver exists to hold.
- If one option dominates, recommend it rather than manufacturing a choice. If a real trade-off survives, ask Tom exactly one concrete question.
- Repair the packet's two stale citations where you find them, and say so — do not restate them.

## Non-goals

Implementing either seam; removing the typed wall; the Metal gather emission, which now depends on this and on the bounds-proof repair; and re-opening the accepted data-dependent index surface beyond the two citation repairs.

## Closes when

Tom has accepted one route by which physical planning obtains a gather bounds proof — or accepted that it does not, with what a provider receives instead recorded — the effect on `pipeline/verify.rs`'s independence is stated either way, and the packet's stale citations are repaired.
