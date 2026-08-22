---
id: repair-the-staged-combine-spikes-citations-the-join-made-stale
title: Repair the staged-combine spike's citations the join made stale
status: in-progress
priority: p3
dependencies: []
related: [join-the-scheduled-region-into-the-contraction-witness, narrow-the-contraction-witness-refusal-to-staging-it-cannot-read, decide-whether-the-citation-checker-should-reach-spike-records]
scopes: [research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, spikes, doc-drift]
claimed_from: todo
assignee: worker-spikecite
lease_expires_at: 1787446178
---
## User-visible outcome

The staged-combine derivability spike's citations resolve against the source they name, so a reader following its record reaches the code it describes rather than text that no longer exists.

## Why this exists

Found 2026-08-22 by `worker-narrow` while working in the file the spike cites. Reported rather than folded in — `spikes/` was outside that lane's scopes.

**Fact (reported, unverified by the coordinator) — three citations return zero hits at tip.** `spikes/reference/staged-combine-derivability/README.md` cites `contraction_witness.rs` by the anchor `A kernel declaring workgroup staging combines inside the workgroup` and by `staging().len() != 0` twice. The scheduled-region join landed the same day the spike was written and moved both. Its other three anchors resolve.

**Fact — the gate cannot see this, by design.** `make citations` walks spike markdown **links** but explicitly **declines spike pinned citations**, on the accepted ground that a spike is evidence about the base its own record names and is repaired on demand. So this is exactly the population that decision left to human repair, arriving on schedule rather than as a surprise.

**Note the frontmatter says `last_verified: 2026-08-22`** — the same day the citations went stale. That is not dishonest: the record was verified against the base it names. It does mean a reader cannot use `last_verified` alone to judge whether a spike's citations still resolve at tip, which is worth stating where the currency convention is described.

## Required work

- Re-audit the Fact at your base and report a verdict — **run each of the six citations yourself**, and say which resolve and which do not, with counts and the unit you report.
- Repair the three stale anchors against the current source. The predicate is now `staging().len() == 0` inside `staged_role`, called from two sites — **verify that at your base rather than inheriting it**; the coordinator confirmed one predicate at `contraction_witness.rs`, not the two the earlier ticket claimed.
- **Preserve the retired wording in a dated correction**, per convention — and expect the record's own grep counts not to shrink.
- Say whether the spike's conclusion still holds. It should: the spike proved staged combine structure is not derivable from program scope, and the join added a route *from the schedule record* rather than from program scope. **If the conclusion has moved, stop and report** — that would change what two dependent tickets rest on.

## Non-goals

Changing the citation checker's declared scope, which is an accepted decision; re-running the spike; and any edit to `crates/`.

## Closes when

Every citation in the record resolves against the file it names, retired wording is preserved in a dated correction, and the record states plainly whether its conclusion survived the join.
