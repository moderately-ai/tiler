---
id: correct-the-spike-portals-false-claim-that-no-make-target-reaches-spikes
title: Correct the spike portal's false claim that no make target reaches spikes
status: in-progress
priority: p3
dependencies: []
related: [decide-whether-the-citation-checker-should-reach-spike-records, repair-the-staged-combine-spikes-citations-the-join-made-stale]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, spikes, gates]
claimed_from: todo
assignee: worker-portal
lease_expires_at: 1787448544
---
## User-visible outcome

The spike portal says which gate reaches `spikes/` and which does not, so a reader is not told no check exists where one does.

## Why this exists

Found 2026-08-22 by `worker-spikecite` while repairing a sibling record that carried the same false claim, and reported rather than folded in — `spikes/README.md` is `contracts/navigation`, outside that lane's scopes.

**Fact — the portal states a gate scope that is no longer true.** `spikes/README.md` reads, verbatim at lines 18–20 and confirmed by the coordinator at `4f53343f`:

> Nothing runs these automatically. The repository's `make` targets cover
> `crates/` and `prototypes/` only, so a spike is exercised by whoever is working
> on it, from its own directory.

**`make citations` covers `spikes/**`, `docs/**`, and `tickets/**`.** Spike markdown links entered the gate on 2026-08-22 under [`decide-whether-the-citation-checker-should-reach-spike-records`](decide-whether-the-citation-checker-should-reach-spike-records.md), with a floor of one link per live record so "nothing ran" cannot read as green.

**This fails in the dangerous direction.** A reader is told no check exists where one does, so a rotted spike link looks like their own problem to catch by hand. The sibling record carried the identical claim and its repair was demonstrated by perturbation: a broken markdown link under `spikes/` fails the gate with `no tracked file or directory at …`, while a false *pinned citation* leaves it green and only raises the declined count — both reverted.

**The surviving half is true and must be preserved.** No `make` target **builds or runs** a spike, and that is deliberate: AGENTS.md keeps spikes out of the build gate so exploratory dependencies do not silently become repository gates. The sentence conflates *runs* with *checks*, and only the second half changed.

## Required work

- Re-audit the Fact at your base and report a verdict; **reproduce both halves yourself** — that a broken spike link fails `make citations`, and that no target builds or runs a spike.
- Repair the sentence to distinguish the two. Say which gate reaches spike markdown, that spike **pinned citations** are declined by decision, and that nothing builds or runs a spike.
- **Preserve the retired wording in a dated correction**; counts will not shrink.
- Check the portal's siblings for the same conflation — any other place claiming spikes are wholly ungated. Report findings **and** clean results.

## Non-goals

Changing the checker's declared scope, which is an accepted decision; adding any build or test target over `spikes/`, which AGENTS.md forbids; and repairing individual spike records, which their own tickets own.

## Closes when

The portal distinguishes what is checked from what is run, both halves are reproduced with their output, retired wording is preserved in a dated correction, and the sibling scan is reported with its clean results.
