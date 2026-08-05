---
id: land-the-rewrite-search-formalism-conclusion-in-the-optimizer-contract
title: Land the rewrite-search formalism conclusion in the optimizer contract
status: done
priority: p2
dependencies: []
related: [survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature]
scopes: [contracts/optimizer, contracts/navigation, research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: [contract, optimizer, search, carrier]
---
## User-visible outcome

The optimizer contract stops saying "a Cascades-style memo is one possible implementation technique, not a committed architecture" and states the selected formalism, and the invariant that makes it work is written down where a reviewer will cite it: **no semantic alternative is pruned on estimated cost, at any stage.**

## Why this exists

**Fact.** [The rewrite-search formalism record](../docs/research/region-search/rewrite-search-formalism.md) landed 2026-08-05 with the elimination stated and its sources preserved. It could not edit the contract: `docs/compiler/**` is the `contracts/optimizer` scope and that survey held only `research/region-search`. This is the carrier for the transfer.

**Inference.** The record's `disposition` is `pending` precisely because this ticket has not run. Until it does, a reader of the contract's "Bounded hierarchical search" section is told the question is open when it has been answered, which is the exact stale-status hazard AGENTS.md names. *Both halves are discharged as of `eb8b5d96`; the paragraph is kept as the reason the ticket existed rather than as a live claim.*

## What the edit owes

- Replace the uncommitted-memo sentence with the selected staged formalism, citing the record rather than restating its derivation.
- Add the no-cost-pruning invariant beside the four surfaces in "The four surfaces the optimizer may consult", as a review rule with the same shape as the tier/backend review obligation already there — nothing mechanical can check it, and a profitability check added at stage 3 would silently reintroduce the phase-ordering hazard the record's spike demonstrates.
- Record what a budget may and may not be, from the record's Orca comparison: a count of work performed, never a wall-clock time-out or a cost threshold, because the same request must compile to the same portfolio twice.
- Move the record's `disposition` from `pending` to `adopted` **in the same change**, and update the research catalog row that restates it. That field is in `research/region-search`, so this ticket must declare that scope too, or a second commit must carry it — decide which when claiming and say so.
- Sweep for contract sentences whose truth depended on the old status.

## Scope declaration added by this work

`research/region-search` was added to `scopes`, resolving the choice this ticket's fourth bullet left to the claimant in favour of one commit rather than two. The record's `disposition` field and the contract sentence it gates are one statement — AGENTS.md requires a catalog and the metadata behind it to move in the same change, and splitting them across two commits would leave an intermediate tree in which the contract states a selected formalism while the record that selected it still reads `pending`. That is the stale-status hazard this ticket exists to remove, so reproducing it for one commit would be self-defeating. The scope is required by already-authorized work and is declaration metadata, not a product-scope expansion.

**Verified before adding, not assumed.** `tkt claims` listed three live claims — `agent-slice-role` on `scope-the-sub-tensor-selection-fusion-role` (`research/indexing`), `agent-conversion-pair` on `test-the-directional-conversion-pair-generalization` (`research/semantic-graph`, `research/numerics`), and `agent-wire-realization` on `wire-the-delivered-realization-record-into-the-artifact` (`implementation/*`, `contracts/numerics`, `contracts/artifacts`, `contracts/decisions`) — plus this ticket's own. None holds `research/region-search` or `contracts/optimizer`. The scope's globs were read from `ticketsplease.toml` rather than recalled: `research/region-search = ["docs/research/region-search/**", "spikes/region-search/**"]`.

**Correction — the `contracts/navigation` half of that sentence was wrong, and the way it was wrong is worth recording.** It first read that no live ticket held `contracts/navigation`, derived from reading each live ticket's frontmatter *in this worktree*, which carries the base commit's copy. `agent-slice-role` added `contracts/navigation` to `scope-the-sub-tensor-selection-fusion-role` **on its own branch**, so the declaration is invisible to any reader who checks the integration tree — `tkt guard` saw it and reported the collision, which is how it was caught. A live claim's scopes must be read from its branch (`git show tkt/<id>:tickets/<id>.md`), not from the base.

**File-level disjointness against that live claim, verified rather than assumed.** `git diff --name-only $(git merge-base tkt/scope-the-sub-tensor-selection-fusion-role HEAD) tkt/scope-the-sub-tensor-selection-fusion-role` returns eight files, of which exactly one is also touched here: `docs/research/README.md`. The two edits are line-disjoint in different catalog sections — that branch adds a `Sub-tensor selection fusion role` row in "Foundation, semantics, and extensions" at hunk `@@ -36`, this branch changes `pending` to `adopted` on the existing rewrite-search-formalism row in "Physical planning and lowering" at hunk `@@ -87`. The other two live claims, `tkt/test-the-directional-conversion-pair-generalization` and `tkt/wire-the-delivered-realization-record-into-the-artifact`, have empty diffs against their merge-base with this branch and so overlap nothing.

## Graph maintenance

**Landed in `eb8b5d96`.** The contract's "Bounded hierarchical search" section states the staged, alternative-retaining formalism, maps the record's four levels onto the eleven named stages so the two numberings stop colliding, records the three eliminations with the Cascades one marked as the record marks it, and states the open stage-3 representation question against its two deferred owners. "The four surfaces the optimizer may consult" carries the no-cost-pruning invariant, and "The review obligation" gains a second named rule beside the tier/backend one. The budget list gains the count-of-work-performed contract with the time-out and cost-threshold axes forbidden by name.

**Two stale sentences were swept and one section reframed.** The intro's "General memo search, partitioning, and calibrated cost estimation remain unimplemented" claimed partitioning was unimplemented, which the same file contradicts twice since `implement-general-dag-partitioning` landed 2026-08-04. "Possible memo contract" said "If a bounded memo is adopted" and now states that the memoized level is physical enumeration, with the two key components a reader would otherwise misread called out. The heading is deliberately unchanged: `crates/tiler-compiler/src/boundary.rs` and `tickets/implement-boundary-property-model.md` both cite it by name and neither is in this ticket's scopes. `docs/open-questions.md`'s closed Q-PLAN-001 restated the superseded memo sentence and is corrected in place with its own dated marker.

**Filed nothing new.** No out-of-scope defect, public-boundary redesign, or bounded research question surfaced; the two open items the selection leaves — the tractability probe and the e-graph decision — were already filed `deferred` with trigger logs by the survey, and this work fired neither. `scope-search-state-caching-across-shape-families` had a trigger naming the survey's completion, so its log gained a dated line: the first conjunct is now satisfied and the second is not, and the line carries the command that decides the unmet half rather than the one that decides the met half.

## Non-goals

Implementing anything; adopting an e-graph; committing a cost model; re-deriving the elimination.

## Closes when

The contract states the formalism and the invariant, no sentence in `docs/compiler/` still describes the mechanism as unchosen, the record's disposition and both catalog views agree, and `tkt lint` passes.
