---
id: retire-the-bare-spikes-gate-nothing-sentence-in-three-spike-records
title: Retire the bare Spikes-gate-nothing sentence in three spike records
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [research/runtime, research/extensions, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [doc-drift, spikes, falsified-evidence]
claimed_from: todo
assignee: worker-gatenothing
lease_expires_at: 1787454809
---
## User-visible outcome

No spike record still carries the bare sentence `Spikes gate nothing.`, which is false in the one direction that stops a reader checking.

## Why this exists

Filed 2026-08-22 by the coordinator as the bounded remainder of [`correct-the-spike-records-that-still-say-spikes-is-outside-every-gate`](correct-the-spike-records-that-still-say-spikes-is-outside-every-gate.md), which landed as `e139baee` and repaired four sites well. That lane found these three and **deliberately left them**, reasoning that read literally the sentence says a broken spike never blocks a merge, and that this stays true. **That reasoning is wrong, and the coordinator verified it by perturbation rather than by argument.**

**Fact — a spike record can fail the gate today.** The chain is in the `Makefile`: `full: check doc` and `check: citations fmt build lint test`. So `make full` runs `make citations`, which reads every retained spike record. Reproduced by the coordinator at `d07bfb7a` by appending one broken markdown link to `spikes/runtime/backend-provider-portfolio/README.md`:

```text
make citations exit: 2

check-citations: 1 markdown link(s) do not resolve against this tree.
make: *** [citations] Error 1
```

The perturbation was reverted and the tree confirmed clean. `check-citations.sh` records the same event independently at the anchor `planted a broken link`. So a broken spike link **does** block a merge, and both readings of the sentence — "spikes block nothing" and "nothing about spikes is checked" — are false.

**Fact — three sites carry it**, verified by the coordinator at `d07bfb7a`:

- `spikes/runtime/backend-provider-portfolio/README.md:40` — `Spikes gate nothing. The `Makefile` has no target for this directory.`
- `spikes/extensions/forkless-physical-provider/README.md:21` — same opening sentence, followed by a longer and **accurate** account of what is not reached.
- `spikes/program-planning/physical-frontier-budget-calibration/README.md:36` — the bare sentence at the end of a paragraph about the `record` command.

**Inference — the second sentence is what makes this worth repairing rather than tolerating.** At two of the three sites the sentence is immediately followed by a true statement about the `Makefile` having no target, which reads as evidence for the false one. That is the same shape as the `**Verified**` table row `d144e1df` repaired: real evidence placed beside a claim that says more than the evidence says.

## Required work

- Re-audit both Facts at your base, running the perturbation yourself rather than relaying the output above.
- Repair each site to the narrower true claim: no `make` target **builds, runs, or lints** anything under `spikes/`, and `make citations` resolves each record's markdown **links** while **declining** its pinned citations by decision. Do not delete the sentence, and do not overstate in the other direction — a record claiming the gate checks its measurements would be a new false claim.
- **Preserve the retired wording** in a dated correction. Grep counts cannot shrink across a successful repair; a shrinking count is a false progress signal.
- Check whether any *fourth* site carries the same sentence or a paraphrase. The delivering lane's census used a vocabulary that found these three; treat it as a floor and re-derive from the subject side.

## Non-goals

The four sites `e139baee` already repaired. `docs/**`, `tickets/**`, and `docs/decisions/**`, all repaired by earlier lanes. Editing `Makefile`, `AGENTS.md`, or `check-citations.sh`, none of which is wrong. Re-deciding whether the checker should reach spike pins, which is settled.

## Closes when

No spike record carries `Spikes gate nothing.` or a paraphrase of it, each repair states what is and is not reached, the retired wording is preserved, and the perturbation showing a spike link failing the gate has been rerun and quoted.


## Scheduling metadata

Added scopes `research/runtime`, `research/extensions`, and `research/program-planning`, required for the three edited paths (`spikes/runtime/backend-provider-portfolio/README.md`, `spikes/extensions/forkless-physical-provider/README.md`, `spikes/program-planning/physical-frontier-budget-calibration/README.md`), per `ticketsplease.toml`'s scope map. This is scheduling metadata for authorized work, not a new authority.
