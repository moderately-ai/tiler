---
id: retire-the-spikes-untouched-claim-in-the-cache-build-tool-record
title: Retire the spikes-untouched claim in the cache build-tool record
status: in-progress
priority: p3
dependencies: []
related: []
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [doc-drift, spikes, falsified-evidence]
claimed_from: todo
assignee: worker-cache
lease_expires_at: 1787454809
---
## User-visible outcome

The cache build-tool record does not tell a reader that no `make` target touches `spikes/`, when one reads every retained spike record.

## Why this exists

Filed 2026-08-22 by the coordinator. Reported by `worker-research` as a site outside its authorized scopes, and **verified by the coordinator by reading the line** at `d07bfb7a` rather than relayed.

**Fact — `docs/research/cache/build-tool-exercise.md:159` states it.** The line reads, in a deferred-item entry: *"No `make` target touches `spikes/` — a spike is a recorded measurement whose value is its record, run from its own directory when someone is working on it — so this driver runs ad hoc as `spikes/README.md` describes."*

**Fact — the claim is false since `04d5eae9`.** `full: check doc` and `check: citations fmt build lint test`, so `make full` runs `make citations`, which resolves every local markdown link in every retained spike record. The coordinator reproduced the consequence at `d07bfb7a`: appending one broken link to a spike README exits `make citations` at 2 with `make: *** [citations] Error 1`. The perturbation was reverted and the tree confirmed clean.

**The entry's conclusion survives and must be re-grounded, not withdrawn.** Its point is that collecting this driver into the repository gate is *not a gap under the current contract*, because no target builds or runs a spike. That remains true — what is false is the wider "touches". The distinction is the whole repair: no target **builds, runs, or lints** anything under `spikes/`, while `make citations` reaches every record for its markdown **links** and **declines** its pinned citations by decision.

**Note the parenthetical is already a repaired claim.** The entry ends *"(This item originally cited the Python gate scripts; they were retired for the Makefile of cargo commands while this work was stranded, and the conclusion survives the translation.)"* So this line has been corrected once already for a related reason and drifted again on a different clause — worth reading the whole entry rather than only the quoted sentence.

## Required work

- Re-audit both Facts at your base, running the perturbation yourself rather than relaying it.
- Repair to the narrower true claim and re-ground the deferred item's conclusion on it; do not delete the entry and do not overstate in the other direction.
- **Preserve the retired wording** in a dated correction. Grep counts cannot shrink across a successful repair.
- Add the `research/cache` scope and explain it in the ticket as scheduling metadata.
- Check the rest of this record for the same shape; report clean results as well as findings.

## Non-goals

`spikes/**`, `docs/decisions/**`, `tickets/**`, `docs/roadmap.md`, and the other `docs/research/**` sites, all repaired by earlier lanes or owned by their own tickets. Editing `Makefile`, `AGENTS.md`, or `check-citations.sh`. Re-deciding the gate's spike-pin exclusion, which is settled.

## Closes when

The line states what is and is not reached, the deferred item's conclusion is re-grounded on what survives, retired wording is preserved, and the record has been swept for the same shape.
