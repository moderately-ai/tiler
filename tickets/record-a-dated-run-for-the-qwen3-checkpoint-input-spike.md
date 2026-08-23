---
id: record-a-dated-run-for-the-qwen3-checkpoint-input-spike
title: Record a dated run for the Qwen3 checkpoint input spike
status: in-progress
priority: p3
dependencies: []
related: []
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [spikes, documentation]
claimed_from: todo
assignee: worker-qwen
lease_expires_at: 1787459513
---
## User-visible outcome

The Qwen3 checkpoint input spike carries a dated recorded run, so it can be governed and catalogued honestly rather than left uncatalogued because nothing establishes when it last worked.

## Why this exists

Filed 2026-08-22 by the coordinator as the bounded remainder of [`reach-every-spike-record-from-the-experiment-catalog`](reach-every-spike-record-from-the-experiment-catalog.md), landed as `6cc903e4`. That lane brought spike-record reachability from 61 to 63 of 65 and **correctly refused this one**, hitting the stop condition the ticket set rather than inventing a date.

**Fact — the record carries no dated evidence.** Reported by `worker-catalog` and consistent with the coordinator's independent reachability check. `spikes/program-planning/qwen3-checkpoint-f32-inputs/` contains only `.gitignore`, `Cargo.lock`, `Cargo.toml`, `README.md`, and `src` — no `results/` directory, and the body carries no `## Result on <date>` section. Its sole git history is one commit, `911c24c2` on 2026-08-17, which is a **landing** date rather than a recorded run.

**Why the refusal was right, and must not be undone by stamping a date.** `spikes/README.md`'s currency convention now tells readers to trust `last_verified` as the date the evidence was taken. An invented or landing-derived `last_verified` is **worse than an absent one**, because the convention converts it into a claim a reader will rely on. The repository has already recorded that `last_verified` says *when a human last looked*, not what tree they looked at — which is precisely why it cannot be back-filled from a commit date.

**Inference — this is a real spike, not an abandoned directory.** It is a consumer-owned Cargo workspace ingesting the pinned Qwen3 checkpoint, structurally analogous to its already-catalogued siblings `qwen3-conformance-fixture` and `qwen3-corpus-reachability`, and it is cited by four `done` tickets. So the outcome is to run and record it, not to delete it — but confirm that framing by reading before acting.

## Required work

- Re-audit both Facts at your base with a per-Fact verdict, listing the directory yourself and checking the git history.
- **Run the harness from its own directory** following its README, and record a dated result in the record's own convention — matching how its catalogued siblings record theirs. Note that nothing under `spikes/` is built or run by any `make` target, so this is a deliberate hand run.
- Derive `last_verified` from **that run**, and record `verified_at_commit` for the tree you ran against if that field's definition has by then been settled — see the escalation below.
- Add the governed frontmatter and the catalog row, so the record becomes reachable from the experiment catalog.
- **If the harness cannot be run** — a missing checkpoint, an unavailable input, an API that has moved — **stop and report that instead**. Recording that it cannot currently run, with the reason, is a truthful outcome; a stamped date is not.

## Non-goals

Changing the currency convention. Adding a gate, target, or census over `spikes/`, all eliminated on recorded evidence. Editing the record's substance beyond adding its result. Deciding the `verified_at_commit` vocabulary question, which is escalated to Tom on the parent ticket and is not this ticket's to settle — if it is still open when you run, record `last_verified` alone and say so.

## Closes when

The record carries a dated run derived from an actual execution, its governed frontmatter is present and derived rather than stamped, it is reachable from the experiment catalog, **or** the record states with evidence that the harness cannot currently be run and why.
