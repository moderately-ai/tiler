---
id: correct-the-spike-records-that-still-say-spikes-is-outside-every-gate
title: Correct the spike records that still say spikes/ is outside every gate
status: todo
priority: p3
dependencies: []
related: [re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree, decide-whether-the-citation-checker-should-reach-spike-records]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [doc-drift, graph-hygiene, spikes]
---
## User-visible outcome

Four spike records stop telling a reader that `spikes/` is unchecked by every gate, which stopped being true at `04d5eae9`, and the one that also hands out stale gate-carry advice stops doing that.

## Why this exists

Found 2026-08-22 by the batch sweep in [`re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree`](re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree.md), which repaired the two instances that ticket named and reports these as the remainder. They fail in the direction `AGENTS.md` calls dangerous: a reader is told a check is absent when it exists.

[`decide-whether-the-citation-checker-should-reach-spike-records`](decide-whether-the-citation-checker-should-reach-spike-records.md) put `spikes/**` into the citation gate's populations at `04d5eae9`. `make citations` now resolves every local markdown link under `spikes/` and fails red on a broken one; it still declines spike **pinned citations** by decision. Confirmed at `6f3c2594` by planting a link to a nonexistent file under `spikes/` and observing exit 2, and by the run's own `594 link(s) from the live spike record files` line.

**Fact — the sites, each verified by reading the file at `6f3c2594`.** Two were merged inside the same batch that falsified them; two predate it.

- `spikes/scheduling/metal_contraction_tile_width/PROTOCOL-2026-08-22-contraction-tile-width.md`, anchor `is outside every repository gate`. The first sentence is false. The second — that `make check` and `make full` do not compile, run, or lint anything here — is still true. **The third is the operative defect: it tells a reader the delta carrying this work reuses the last green gate "for exactly that reason", and a `spikes/` delta must now rerun `make citations`**, which `AGENTS.md "Record the carry reasoning and rerun"` requires.
- `spikes/reference/staged-combine-derivability/README.md`, anchor `sits outside every gate, so this harness breaks`. The opening clause is false; the conclusion it draws — that a harness which stops compiling breaks silently — survives, because no gate compiles this directory.
- `spikes/runtime/backend-provider-portfolio/README.md`, anchor `cannot see this directory`. Being outside the workspace `members` is still true; `make full` **can** now see this directory, through `make check`'s `citations` prerequisite.
- `spikes/target-profiles/scalar-cpu-vertical/README.md`, anchor `so the only detector is a reader running it by hand`. False as written; true of the compile breakage the surrounding Fact is actually about.

**Fact — the large "no `make` target reaches" population is NOT in scope and should not be swept.** Roughly thirty-five further sites under `spikes/` carry that phrase inside a Reproduce or run-it-by-hand instruction, and many defer to the canonical statement with "per `spikes/README.md`". That canonical sentence, at `spikes/README.md` anchor `builds or runs a spike`, is **still true** — it says no target *builds or runs* a spike, which the citation gate does not do. The referring sites inherit a true claim, and their operative meaning is intact. Rewriting them would be a large cross-lane edit that repairs nothing.

## Required work

- Re-audit each of the four Facts at your base with a per-Fact verdict, running `make citations` yourself.
- Repair each in place, **preserving the retired wording in a dated correction**; these claims were true when written and the correction is about the base, not the author.
- Where a claim survives its falsified premise, say so and keep it — three of the four have a surviving conclusion.
- Fix the PROTOCOL's gate-carry sentence, which is advice rather than a record.
- Expect grep counts not to shrink: a correction quoting retired wording verbatim makes the retired string searchable again.

## Non-goals

The ~35 run-it-by-hand mentions above. Changing `check-citations.sh`. Editing `crates/`.

## Closes when

Each of the four carries a dated correction whose reproduction supports the claim beside it, and the PROTOCOL no longer tells a reader a `spikes/` delta carries the gate for free.
