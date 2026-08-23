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

## Coordinator census at `344cef97`, 2026-08-22 — two counts corroborated, two need a stated definition

I said above that I had not re-run these. I have now, and the results are recorded here so you can compare against your own rather than inherit either set. **My reachability method is crude and you should not adopt it**: I tested whether the catalog text textually contains each record's directory path, which can report a record reachable because its path appears in unrelated prose, and unreachable because the catalog routes to it through an intermediate page. Derive reachability by *following links*, not by substring containment, and say which you did.

- **9 unreachable — corroborated**, by that crude method. Named: `apple-targets/aot-runtime-compiler-observer`, `apple-targets/code-domain-integer-decode`, `apple-targets/contraction-pragma-runtime-probe`, `extensions/forkless-physical-provider`, `program-planning/identity-growth/results`, `program-planning/qwen3-checkpoint-f32-inputs`, `reference/staged-combine-derivability`, `runtime/backend-provider-portfolio`, `scheduling/metal_contraction_tile_width`. Note the ticket's guess at which three are *correctly* unlisted is not obviously right: `identity-growth/results` is the results file it predicted, but `staged-combine-derivability` and `metal_contraction_tile_width` are full records that other documents cite as authorities, so decide each by reading.
- **6 without frontmatter — corroborated**, and all six are drawn from the nine, as reported.
- **`last_verified` is carried by 56 records, not 55.** Out of 65 tracked spike READMEs. The difference is one record, and it is not worth chasing except as a reminder that a number quoted from another lane drifts — state the population you counted (I counted tracked `spikes/**/README.md`, 65 of them) so the next reader can reproduce it.
- **`verified_at_commit`: the "four records" is right, but only under a definition the ticket does not state.** The string appears in **6** tracked files: four spike records (`numerics/bf16-second-dtype`, `runtime/backend-provider-portfolio`, `runtime/inline-dispatch`, `target-profiles/scalar-cpu-vertical`), plus `spikes/README.md` itself — which is the catalog defining the convention rather than a record using it — and one dated audit transcript under `docs/research/documentation/`. So "four records" holds once the catalog and the transcript are excluded, and that exclusion is a judgement, not a count. **Say which you are reporting.**
- **Confirmed: `verified_at_commit` is not defined in `docs/document-metadata.md`.** A plain substring search of that file returns nothing.

The escalation instruction in the Required work stands unchanged: if you conclude the governed vocabulary should grow to define this field, state the case and **stop for Tom** rather than adding it.
