---
id: reach-every-spike-record-from-the-experiment-catalog
title: Reach every spike record from the experiment catalog
status: todo
priority: p3
dependencies: []
related: [state-the-spike-currency-convention-where-readers-look, keep-the-ungated-spikes-compiling-against-the-workspace-api]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, spikes, navigation]
---
## User-visible outcome

Every retained spike record is reachable from the experiment catalog, and every one carries the governed frontmatter its own contract requires — so a reader following the newly-stated currency convention can actually arrive at the record the convention tells them to read.

## Why this exists

Filed 2026-08-22 by the coordinator from findings the `state-the-spike-currency-convention-where-readers-look` lane surfaced and correctly reported rather than folded in. **Re-derive both counts at your base before acting**; they are the reporting lane's, not mine, and I have not re-run them.

**Fact (reported, unverified by me) — nine spike READMEs are unreachable from the experiment catalog.** Named among them: `spikes/runtime/backend-provider-portfolio/README.md`, `spikes/extensions/forkless-physical-provider`, `spikes/program-planning/qwen3-checkpoint-f32-inputs`, and three `apple-targets` sibling probes. Three further entries may be correctly unlisted — a portal, a sub-guide, and a results file — so the population is **not** simply "nine defects".

**Why that matters more than it looks.** `backend-provider-portfolio` is one of the two records that carries the repaired-on-demand currency statement in prose. The convention just landed in `spikes/README.md` tells a reader to go read the spike's own dated claim; for this one, no catalog route reaches it.

**Fact (reported, unverified by me) — six spike READMEs carry no frontmatter at all**, all six drawn from the nine above. `last_verified` is governed frontmatter required on a reproducible experiment, and the same lane measured 55 of the spike READMEs carrying it — so these six are the exception, not the norm.

**Fact (reported, unverified by me) — `verified_at_commit` is used by four records and defined nowhere in `docs/document-metadata.md`.** That field is load-bearing for the currency convention, which instructs readers to compare a spike's recorded base against the tip.

## Required work

- Re-derive the catalog-reachability census and the frontmatter census at your actual base, stating the exact commands and their output. Report the counts **before and after**.
- For each unreachable record, decide by reading whether it *should* be catalogued or is correctly unlisted, and say which for every one. A portal, a sub-guide, and a results file are plausibly correct omissions — do not add rows mechanically to drive a count to zero.
- Add the governed frontmatter where it is genuinely missing, deriving `last_verified` from the record's own evidence rather than stamping today's date. **If a record's last known-good run cannot be established from what it carries, stop on that record and report it** — an invented `last_verified` is worse than an absent one, because the convention now tells readers to trust it.
- Decide whether `verified_at_commit` should be defined in `docs/document-metadata.md`, or replaced by an existing governed field. **`docs/document-metadata.md` is in your scopes**, but adding a field to a shared vocabulary is a decision: if you conclude the vocabulary should grow, state the case and **stop for Tom** rather than adding it.

## Non-goals

Editing spike bodies or their measurements; running any spike; adding a gate, target, or census over `spikes/` — all three were eliminated on recorded evidence and re-litigating them is out of scope. Changing the currency convention itself, which just landed.

## Closes when

Every retained spike record is either catalogued or recorded as deliberately unlisted with its reason, missing governed frontmatter is present and derived rather than stamped, the `verified_at_commit` question is answered or escalated, both censuses are quoted before and after, and `make citations` is green.
