---
id: bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence
title: Bind a scheduled gather's retained proof to its own occurrence
status: in-progress
priority: p1
dependencies: []
related: [decide-whether-refinement-evidence-may-reach-a-physical-provider]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, gather, identity, fail-closed]
claimed_from: todo
assignee: worker-occupancy
lease_expires_at: 1787445714
---
## User-visible outcome

A retained `GatherIndexBoundsProof` is checked to belong to the occurrence carrying it, so a proof minted for one region cannot be attached to a shape-compatible different one and pass every check in the tree.

## Why this exists

Found 2026-08-22 by the refinement-seam packet, which set out to answer a public-boundary question and found the boundary was never the problem.

**Fact — nothing checks which occurrence a retained proof belongs to.** `gather_accesses_match` in `crates/tiler-compiler/src/physical.rs` **elides the `proof` field with `..`** and never inspects it. Schedule rule 8 in `crates/tiler-ir/src/schedule/builder/elementwise.rs` compares four accessors — `source_shape`, `result_shape`, `index_shape`, `axis` — and none of `region()`, `access()`, `source()`, `index()`, the types, the extent, the domain, `kind()`, or `facts()`.

**Fact — rule 8 structurally *cannot* make that check.** `tiler_ir::schedule::IndexRegion` has no `CanonicalIndexRegionIdentity` counterpart to compare against. This is a layering fact, not an omission, and it is why the check belongs in the compiler rather than the schedule builder. Recorded separately as [`record-that-schedule-rule-8-cannot-check-the-proofs-region-identity`](record-that-schedule-rule-8-cannot-check-the-proofs-region-identity.md) so nobody later "fixes" it at the wrong layer.

**Inference — the failure is identity corruption, not an out-of-bounds read.** A proof minted for region A, attached to a shape-compatible occurrence B, passes everything. It stays *bounds*-sound, because both closed proof kinds are functions of `(source_shape, index_shape, axis)` — exactly what rule 8 compares. What it corrupts is identity: the schedule encodes the proof identity, which folds A's region identity, access ordinal, tensor ordinals, types, and domain. **That is verbatim the failure `RegionVocabularyWall::GatherProofUnavailable`'s own doc gives as its reason for existing** — and it is latent in the accepted surface today, reachable the moment the first gather baseline exists.

**Fact — the evidence is already public, so this cannot be closed by withholding it.** `BoundsProofKind::GatherSource`'s `proof` is a public field of a public variant, `ScheduledRegion.index.bounds_proofs` is public, `ImplementationContext::baseline()` is `pub`, and `GatherIndexBoundsProof` is `Clone`. Verified by the coordinator at `0c086aee`. Refinement evidence already reaches every installed provider under the surface accepted 2026-08-18.

## Required work

- Re-audit every Fact at your base with a per-Fact verdict. The delivering lane's own report flags one inference it did not construct — that a third-party provider can reach the transplant by retaining proofs across `propose()` calls — **treat that as unverified and either construct it or say you did not**.
- Make `gather_accesses_match` stop eliding `proof` and require its `region()` to be the occurrence's own.
- **Construct the transplant and show it refused**, quoting the failure text: a proof minted for one region, attached to a shape-compatible different occurrence. The whole finding is that this state passes today; a repair asserted without that construction has not been demonstrated.
- Perturb each new comparison separately. Before trusting it, state what it would take for it to say *no* and confirm that case is reachable — a lane today found its own gate made a named case unreachable by pigeonhole.
- **State whether any identity value moves. Expected: none** — this is a refusal over already-encoded fields — but rederive, and **stop and report** if one does.

## Non-goals

Threading resolved lowering into the spelling path, which depends on this and is its own ticket; retiring the wall; and any change to `ImplementationContext`'s provider surface, which the packet showed is unnecessary.

## Closes when

A proof belonging to a different occurrence is refused by name with its output quoted, each comparison is perturbed separately, and no identity value has moved.
