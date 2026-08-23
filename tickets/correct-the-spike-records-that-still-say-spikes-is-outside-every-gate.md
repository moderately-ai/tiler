---
id: correct-the-spike-records-that-still-say-spikes-is-outside-every-gate
title: Correct the spike records that still say spikes/ is outside every gate
status: done
priority: p3
dependencies: []
related: [re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree, decide-whether-the-citation-checker-should-reach-spike-records]
scopes: [research/scheduling, research/runtime, research/target-profiles, research/program-planning]
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

## Worker note — 2026-08-22, `worker-spikerec`, base `2c312826b6`

**Scopes added, as scheduling metadata.** `scopes: []` did not cover the paths this ticket edits; `research/scheduling`, `research/runtime`, `research/target-profiles`, and `research/program-planning` were added because the four repaired files sit under `spikes/scheduling/**`, `spikes/runtime/**`, `spikes/target-profiles/**`, and `spikes/program-planning/**` respectively, per `ticketsplease.toml`'s path-to-scope table.

**Re-audited the four named Facts, each re-read at `2c312826b6`.**
- `spikes/reference/staged-combine-derivability/README.md` — the anchor `sits outside every gate, so this harness breaks` no longer exists verbatim. This file was already repaired, separately, at `46f74a90` ("Repair the staged-combine spike's citations the scheduled-region join made stale"), which landed a dated correction retiring both `"no `make` target reaches `spikes/`"` and `"`spikes/` sits outside every gate"` and stating the current gate reach. **No action taken; the Fact was stale before I started.**
- The other three Facts were verified true as stated and repaired in place, each with a dated correction preserving the retired wording verbatim: `spikes/scheduling/metal_contraction_tile_width/PROTOCOL-2026-08-22-contraction-tile-width.md` ("The standing hazard this spike sits inside"), `spikes/runtime/backend-provider-portfolio/README.md` (the "Nothing detected those breaks" paragraph), `spikes/target-profiles/scalar-cpu-vertical/README.md` (the "Two API steps landed" Fact).

**Population re-derived rather than trusted, per the ticket's own instruction.** Built a subject-side census in Python (not shell, per the backtick-corruption warning) over every `spikes/**/*.md` line matching any of: `outside every( repository)? gate`, `outside the workspace \`?members\`?`, `cannot see this directory`, `gates? nothing`, `no \`?make\`? target reaches`, `no target reaches`, `no gate (compiles|runs|reaches|checks)`, `nothing (in the repository )?(gates|checks|reaches|sees)`, `the only detector is a reader`, `so this harness breaks`, `reuses the last green gate` — 42 matching lines across 68 files, read individually. All but one non-ticket-named site fell into the ~35 true "no target builds/runs a spike" population the ticket excludes (run-it-by-hand instructions, several deferring to `spikes/README.md`), or turned out to be a false positive (`spikes/apple-targets/README.md`'s "the gate" refers to an in-harness numerical oracle, not a repository gate). One additional genuinely false site was found and repaired: `spikes/program-planning/physical-frontier-budget-calibration/README.md`, "The rule is adopted, not written" paragraph, which stated `spikes/ sits outside every repository gate` as the reason a copied quiet-host threshold could drift silently. Repaired with a dated correction: the premise is false (citations reaches this file), the conclusion survives (no gate checks whether a quoted constant tracks its source, and no target still compiles/runs/lints the directory).

**Grep counts did not shrink**, confirmed by comparing the base and post-edit counts of the three false-claim phrasings (`outside every gate`, `cannot see this directory`, `the only detector is a reader`) across the seven touched/adjacent files: 7 at base, 10 now — the increase is the preserved retired-wording quotes.

**Checks run:** `tkt lint` (`ok: no problems found`), `make citations` (exit 0; `spikes 609 link(s) ... 62 pinned citation(s) DECLINED`, both up from 601/61 because of the new ticket links and one new pinned citation my corrections quote), `git diff --check` (exit 0). `tkt guard --base 2c312826b60e275002f7332f167dd3c795861b99` run and reviewed.

**Nothing in this ticket contradicted the brief.** `Spikes gate nothing.` (verbatim, in `spikes/runtime/backend-provider-portfolio/README.md`, `spikes/extensions/forkless-physical-provider/README.md`, and `spikes/program-planning/physical-frontier-budget-calibration/README.md`) was deliberately left untouched: read literally it says spikes-as-subjects don't gate anything else (a broken spike never blocks a merge), which stays true, rather than "nothing gates spikes" — the false claim this ticket targets.

## Coordinator close, 2026-08-22 at `d07bfb7a` — four sites repaired, three split out rather than closed over

Merged as `e139baee`. Four sites repaired, each preserving its retired wording, and one of them — `spikes/program-planning/physical-frontier-budget-calibration/README.md` — was **not named by this ticket** and was found by re-deriving the population instead of trusting the handed-over split. That is the census discipline working.

**One reported Fact of this ticket was stale and the lane was right to refuse it.** `spikes/reference/staged-combine-derivability/README.md` had already been repaired by `46f74a90`, which retired the exact wording this ticket cited and stated the current gate reach. No action was correct there.

**The remainder is split out, not closed over.** The lane left three sites carrying the bare sentence `Spikes gate nothing.`, reasoning that read literally it says a broken spike never blocks a merge and that this stays true. **The coordinator verified that reasoning is wrong**, by perturbation rather than argument: appending one broken markdown link to `spikes/runtime/backend-provider-portfolio/README.md` exits `make citations` at 2 with `make: *** [citations] Error 1`, and `full: check doc` with `check: citations …` puts that on the merge path. The perturbation was reverted and the tree confirmed clean. Those three sites are now [`retire-the-bare-spikes-gate-nothing-sentence-in-three-spike-records`](retire-the-bare-spikes-gate-nothing-sentence-in-three-spike-records.md), which carries the reproduction.

Closing this ticket on its four repaired sites with that remainder named, rather than reporting a Closes-when that the three sites do not meet.
