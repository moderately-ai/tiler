---
id: finish-the-stale-claim-sweep-over-the-candidate-status-ticket-bodies
title: Finish the stale-claim sweep over the candidate-status ticket bodies
status: in-progress
priority: p3
dependencies: []
related: [package-a-multi-entry-bundle-from-one-expansion, decide-the-expansion-cache-collection-schedule, reach-a-verified-kernel-through-the-structural-families]
scopes: [project/tickets]
shared_scopes: []
paths: []
tags: [documentation, work-graph, hygiene]
claimed_from: todo
assignee: agent-stale-sweep
lease_expires_at: 1785903117
---
## User-visible outcome

A reader who picks up a `todo`, `deferred`, `blocked`, or `awaiting-decision` ticket can trust the blockers its body records, because every such claim has been checked against the tree rather than inherited from the run that wrote it.

## Why this exists

**Fact — a partial sweep on 2026-08-02 found four refuted blocker claims out of forty-five examined, and two of the four were holding real work parked.** `package-a-multi-entry-bundle-from-one-expansion` was `blocked` with no unmet graph edge, against a contract limit and a grammar limit that had both been lifted. `decide-the-expansion-cache-collection-schedule` was `deferred` against "there is no proc-macro frontend" while `crates/tiler-macros` is one and opens the cache. `reach-a-verified-kernel-through-the-structural-families` scheduled delivery of a one-input limit that no longer exists. `deliver-several-artifact-families-from-one-expansion` scheduled reconciliation of a documentation sentence already withdrawn. Each is corrected in place with its refuting source line.

**Fact — the sweep's own coverage is the reason this ticket exists, and it is countable rather than gestured at.** 740 ticket files exist; 154 are in a candidate status (`todo` 118, `deferred` 32, `blocked` 2, `awaiting-decision` 2); 45 of those matched a blocker/absence phrase set and **all 45 were examined**. The remaining **109 were never read.**

**Inference — the unread 109 are not safe by omission.** They were excluded by a substring filter, and a stale blocker can be worded outside it: "the only survivor", "fails closed today", "no owner", "not yet reachable", "pending". AGENTS.md's own rule applies to the sweep that produced this ticket — a failed search is evidence the search was wrong, not that the thing is absent, until the file has been read.

## Required work

- Read the 109 candidate-status tickets the phrase filter excluded. Read them; do not re-run a widened grep and call the remainder clear. State the count read so coverage stays countable.
- For each blocker, deferral, or absence claim, extract the specific checkable referent — symbol, file, test, constant, or ticket status — and verify it by reading the source file, not by a search returning nothing.
- Correct each stale claim in place: strike or mark the superseded sentence, keep the original rationale rather than deleting it, and cite the refuting file and line plus a one-line reproducing command.
- Move a ticket's status only when **every** recorded blocker is refuted and no unmet graph edge remains. A ticket with one blocker refuted and another live stays parked, with the live one named.
- Repair the two structural defects the partial sweep found and did not fix, both on `package-a-multi-entry-bundle-from-one-expansion`: a `blocked` status with zero unmet dependency edges, and a coordinator correction naming a replacement dependency that was never written into frontmatter and that points at a `closed` ticket. Sweep for both shapes across the board — a `closed` dependency orphans its dependents rather than satisfying them, and `tkt rollup` surfaces orphans.

## Also fix: line-number citations that drifted

Three tickets cite line numbers that no longer resolve, with the argument still correct. Line drift, not a wrong claim — repair the citation, do not reopen the reasoning.

- `add-subgroup-memory-scope-when-collectives-land`: the 2026-08-01 addendum's "corrected" citations are themselves stale. `barrier_call` is at `crates/tiler-metal/src/emit.rs:1601`, the subgroup binding at `:1604`, the rejection at `:1622`.
- `implement-boundary-property-enforcers`: the original 2026-07-27 deferral cites `frontier.rs:557`/`:563` for `bounded_guarantees`/`bounded_requirements`, which now live only inside `crates/tiler-compiler/src/boundary.rs`'s `mod tests` (`:1995`, `:2010`, `:2025`). The ticket's own 2026-07-28 addendum already declares that claim false, so this is superseded rather than misleading — mark it as such.
- `bind-stage-coverage-to-index-refinement-identity`: substance holds, cited lines `builder.rs:891-892`, `:903`, `model.rs:329-330` are stale. Current: `crates/tiler-ir/src/program/builder.rs:1079` and `:1090`, `crates/tiler-ir/src/program/model.rs:549`.

## Explicit non-goals

Do not do the work any corrected ticket unblocks — correcting the record and dispatching against it are separate acts, and conflating them is how a sweep turns into an unscoped implementation run. Do not touch `done` or `closed` tickets except where one is cited as live evidence elsewhere.

## Closes when

Every candidate-status ticket body has been read, each blocker claim in one either verified against a source line or corrected with its refutation, the count read is stated against the count that exists, the two structural defects above are repaired board-wide, and the three drifted citations are updated.

## Graph maintenance

- File a separate ticket for any real defect this sweep discovers in code; this ticket corrects records and must not absorb an implementation fix.
- A ticket this sweep moves out of `deferred` needs its activation triggers checked as actually fired, not merely plausible — a deferral filed dispatchable has been claimed and read before anyone noticed the triggers had not fired.
