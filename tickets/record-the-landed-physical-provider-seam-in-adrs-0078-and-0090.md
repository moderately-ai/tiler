---
id: record-the-landed-physical-provider-seam-in-adrs-0078-and-0090
title: Record the landed physical-provider seam in ADRs 0078 and 0090
status: in-progress
priority: p2
dependencies: [drive-an-external-physical-implementation-provider-through-compilation]
related: [accept-the-public-backend-provider-composition-boundary, disclose-offered-and-selected-physical-provider-sets-separately]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, decision, documentation, public-boundary]
claimed_from: todo
assignee: coord
lease_expires_at: 1786180895
---
## User-visible outcome

A reader of ADR 0090 learns that item 2's physical-provider registry is implemented rather than pending, and a reader of ADR 0078 finds the physical-implementation seam in its governed inventory at the rung the evidence supports — so the two records stop describing a tree that has moved.

## Why this exists

[`drive-an-external-physical-implementation-provider-through-compilation`](drive-an-external-physical-implementation-provider-through-compilation.md) landed the seam on 2026-08-08 but does not hold `contracts/decisions`, so it could not touch `docs/decisions/`. Its graph-maintenance section names this obligation and hands it here rather than leaving a partial sweep. This is a carrier ticket: the landing already applied every consequence inside the scopes it held (`docs/compiler/optimizer.md`, `docs/operation-extensions.md`, `docs/glossary.md`).

**Fact — the two statements that are now false, both read at `c81f9257` before the landing.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)'s accepted status paragraph ends "the item-2 physical-provider registry and the item-5 disclosure accessors remain unimplemented"; item 5's Fact paragraph states "today neither is answerable at all", citing `session.rs` as the population site. The first clause is now false for item 2. The second is *half* false: the **selected** set is answerable through `PlanAlternative::selected_physical_providers`, and the **offered** set is still lowering-only, which is exactly the split item 5 exists to preserve — so correcting it must not collapse the two.

**Fact — ADR 0090 item 5's cited line has been stale twice already.** The record cites `session.rs:1513`; a 2026-08-05 audit found it at `:2092`; at `c81f9257` it is `:2208`. Cite by searchable anchor rather than adding a third line number.

## Implementation keys

- Move ADR 0090's `implementation_status` prose forward for item 2 only, naming the landing ticket and its date, and leave item 5's status accurate to its half-landed state.
- ADR 0078's open questions on item 5 were already closed with dated resolutions during the 2026-07-31 acceptance sweep; what this ticket owes ADR 0078 is its **governed seam inventory** row and the maturity rung, not a reopened question. Read the whole record before deciding which.
- **Do not record a tested-guarantee rung without reading the evidence boundary.** `docs/operation-extensions.md` states that rung requires a provider written outside the defining crate to drive the seam through the ordinary compile path. The landed fixture is an integration test: a separate compilation unit reaching only `pub` items, but inside the defining *package*. The operation-extension contract already records that distinction as a Measurement; copy its boundary rather than rounding it up.
- Every public surface named remains a labelled draft under ADR 0075. Do not write acceptance language.

## Closes when

ADR 0090's status paragraph and ADR 0078's inventory agree with the tree, `make citations` passes, and no statement in either record claims an acceptance Tom has not given.

## Graph maintenance

- Editing during transfer forks a record. If the correct edit changes what either ADR *decides* rather than what it reports, stop and say so.
- If re-reading finds a third stale statement, repair it in the same landing and report the repair.
