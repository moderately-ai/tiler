---
id: date-the-two-v4-step-paragraphs-trailing-the-v5-block
title: Date the two v4-step paragraphs that now trail the artifact ABI's v5 block
status: in-progress
priority: p3
dependencies: []
related: [date-the-artifact-abis-metal-golden-enumeration-to-its-step]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, identity]
claimed_from: todo
assignee: coord
lease_expires_at: 1786184079
---
## Two `tiler.schedule.v4` paragraphs now trail the `v5` block and read as its continuation

Verified 2026-08-08 at base `c0b2f06b` by [`date-the-artifact-abis-metal-golden-enumeration-to-its-step`](date-the-artifact-abis-metal-golden-enumeration-to-its-step.md), which found it while dating the Metal golden enumeration one paragraph above.

**Fact — the ordering.** In `docs/artifact-abi.md` the scheduled-region chronology runs: the `v4` Fact (anchor `admitting a loop-carried cooperative tile moved the scheduled region`), then the `v5` Fact (anchor `widening the cooperative staging relation to two dimensions`), then its Inference (anchor `an append was not available, and this time the encoding says so on its own terms`), then the golden paragraph (anchor `including the five that carry no cooperative tile`) — and only then two paragraphs whose subject is the `v4` step: anchors `the append that avoided the earlier steps was no longer available` and `deliberately do **not** move with it`. Each of those five anchors is unique in the file at this base.

**Fact — both trailing paragraphs are `v4`-step text and have not been rewritten since.**

```sh
git log --oneline -S'built over a `v4` region can never be confused with one built over a `v3` region' -- docs/artifact-abi.md
# e4d2aa7d Admit a cooperative tile whose staging is rewritten between rounds
```

`e4d2aa7d` is the `v4` step. `git show a395852a -- docs/artifact-abi.md` shows the `v5` block being **inserted above** this pair rather than appended after it: the three added paragraphs land between the `v4` Fact and the two that already followed it.

**Fact — the version pairs in them are correct for the step they record and must not be renumbered.** `a `v6` kernel identity built over a `v4` region can never be confused with one built over a `v3` region` is the `v4` step's own argument for why nothing above the scheduled region moved. Renumbering it to `v5`/`v4` would restate a `v4`-step record as a `v5`-step one — the exact repair the parent ticket refused for the golden enumeration, and for the same reason.

## What survives

A reader hazard of the class the parent ticket repaired, one paragraph further on. `deliberately do **not** move with it` has the `v5` block as its nearest antecedent and the `v4` step as its actual subject, so a reader who trusts position reads a contradiction — a paragraph apparently about `v5` that names `v4` and `v3`.

**Inference — the `v5` Inference also forward-references the one that now follows it.** `an append was not available, and this time the encoding says so on its own terms` contrasts the `v5` step against an earlier step whose append argument was contingent; that earlier argument is `the append that avoided the earlier steps was no longer available`, which is now two paragraphs *below* the sentence contrasting with it. Labelled Inference because it rests on reading what "this time" contrasts with, not on a citation.

## Requirements

- Mark both trailing paragraphs to the `tiler.schedule.v4` step in the chronology's existing dated idiom, as the golden paragraph above them now is (`at the `v5` step every Metal golden's identity moved`).
- **Do not renumber `v6`/`v4`/`v3`**, and state the reason inline so the next audit does not re-raise it.
- **Do not reorder the blocks.** Moving the two paragraphs above the `v5` Fact would put text at a position `git log -S` no longer locates at its own step, and reordering a chronology to fix a tense is a larger change than the defect.
- Resolve or mark the `this time` forward reference.
- Prefer a searchable anchor to a line number, and run its grep before committing to it; `make citations` covers `docs/**`.

## Scheduling note — one file, several live claims

This holds `contracts/artifacts` and edits `docs/artifact-abi.md`, which has been swept four times this week. It must be sequenced against any other live claim on that file rather than run concurrently.

## Closes when

Each of the two trailing paragraphs states the step it records; no version pair is renumbered and no block is reordered; the reason a renumber would be wrong is recorded inline; and `make citations` is green.
