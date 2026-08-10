---
id: re-transfer-the-adr-0092-span-after-the-item-6-restatement-fork
title: Re-transfer the ADR 0092 span after the item 6 restatement fork
status: done
priority: p1
dependencies: [correct-adr-0092-item-6-s-widening-restatement-trap-and-its-retired-sentence-shorthand]
related: [decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, documentation, transfer]
---

`docs/research/runtime/backend-scoped-route-requirement-answers.md` retains a transferred copy of ADR 0092's decision span. When this ticket was filed, the item-6 restatement had landed in the ADR and **forked that copy**. Under the convention stated beside the span, the repair lands in the ADR first and the span is then re-transferred from it — so this ticket was the second half of that repair. Filing-time purpose was to restore byte-identity after the fork; that half is closed (see Outcome).

## Facts

**Coordinator-verified at the merge that created the fork.** ADR 0092 item 6 was restated to cite `docs/architecture.md` by the anchor `is the one crate an inline-frontend consumer names`, to flag its own quoted rule as the record's shorthand rather than a quotation, and to state that the restatement **adds** the dispatch qualifier to the frontend one already present. No decision clause moved; the bolded decision text and "Every non-dispatching use is unaffected" are byte-unchanged.

**Reported by that worker, not independently verified.** The forked position was the record's item 6 against the ADR's, and the worker recorded the obligation in its own correction note. Before editing, re-establish the exact divergence yourself by diffing the span against the ADR with the span's `###` headings mapped to `##` — that is the comparison the previous re-transfers used, and it should come back clean everywhere except item 6.

**Fact — this is the third such fork, and both predecessors were settled the same way.** The `ResourceFloor` rename and the prototype-referent drift were each corrected in the ADR and re-transferred, leaving the two documents byte-identical again, verified line-for-line rather than by eye.

## What closes this

The span re-transferred from ADR 0092 so the two are byte-identical again, verified with `cmp` or `diff` and **not by eye**, with the comparison shown to be capable of failing before it is trusted. The previous worker on this construct proved its `cmp` could fail (`differ: char 681, line 5`) before relying on it; do the same.

**Do not repair the span in place.** Editing inside the span is what forks the transfer, and the convention beside it says so directly. The whole point of the two-step is that the ADR is the authority and the span follows.

**The span is fenced.** It was wrapped in a `text` code fence at merge `91f67cc5` under `decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span`, because its relative links are spelled for `docs/decisions/` and do not resolve from the record. Read that ticket before touching the region. Your re-transfer must preserve the fence and must not re-root any link inside it — a re-transfer that quietly repointed the span's links would undo a settled decision and reopen a question three workers converged on.

Confirm afterwards that `./check-citations.sh` still reports zero failures and that the fence still closes where it did — break a link *after* the closing fence and confirm it is still caught, which is the perturbation that proved the fence's extent the first time.

## Outcome — 2026-08-08, `23746b12`

Delivered. Item 6 was spliced from ADR 0092 into the fenced span in `docs/research/runtime/backend-scoped-route-requirement-answers.md` rather than re-authored in place. Diff is one line in that record; the `text` fence and every in-span link spelling (for `docs/decisions/`) were preserved.

Verified with `cmp` over the ADR's Context through Traceability first paragraph (ADR-only dated correction notes and the post-Traceability "Three links…" paragraph excluded) against the span with `###` mapped to `##`. Before the splice: `differ: char 3051, line 19`. After: exit 0.

**Correction — 2026-08-10.** Opening prose that said the repair "is currently incomplete" was filing-time purpose only. At `23746b12` and HEAD the transferred region is byte-identical again; `status: done` matches the tree. Predecessor [`correct-adr-0092-item-6-s-widening-restatement-trap-and-its-retired-sentence-shorthand`](correct-adr-0092-item-6-s-widening-restatement-trap-and-its-retired-sentence-shorthand.md) is recorded in `dependencies` for historical accuracy (both nodes remain `done`). Architecture item-6 amendment and public-boundary acceptance were never this ticket's close condition and stay with their own tickets.
