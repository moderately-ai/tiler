---
id: correct-adr-0092-item-6-s-widening-restatement-trap-and-its-retired-sentence-shorthand
title: Correct ADR 0092 item 6 s widening restatement trap and its retired sentence shorthand
status: done
priority: p1
dependencies: []
related: [correct-the-architecture-citation-that-drops-the-inline-frontend-qualifier, accept-the-public-route-requirement-answer-boundary]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, contracts, citations]
---

ADR 0092 decision item 6 instructs a future worker to restate a sentence in `docs/architecture.md`, and a worker following it literally would **widen** that sentence back into a claim the same paragraph refuses. It is p1 because the defect is an instruction that produces a wrong edit, not merely a stale description.

## Facts

**Reported by the worker that repaired the citing record, coordinator-verified where noted.**

**Verified at `299bd259`.** `docs/architecture.md` says *"the one crate an inline-frontend consumer names"*, and the same paragraph explicitly carves out consumers constructing arbitrary semantic programs. A plain `grep` finds it; no whitespace collapsing is needed.

**Verified.** The record's citations were **accurate when written**. At `6f7caf3`, the base that record declares, `docs/architecture.md:389` is the frontend-pair paragraph and carries the flat sentence verbatim. `52e088a2` ("State the consumer-neutral compiler mission explicitly", 2026-08-04) added the qualifier and moved the paragraph to `:435`. **This is citation rot, not authorial error** — treat the ADR's wording the same way, and do not write a correction that blames its author.

**Reported, not independently verified — check both before editing.** ADR 0092 item 6 names the sentence by its flat shorthand and directs that it be "restated that way in `docs/architecture.md`" without saying which qualifier is added. A worker applying it literally lands *"the one crate a **non-dispatching** consumer names"*, silently dropping `inline-frontend` and restoring the flat monopoly. The correct target is the **conjunction**: *non-dispatching inline-frontend*. Separately, the ADR's status paragraph calls it "the 'one crate a consumer names' sentence" — a string `docs/architecture.md` no longer contains.

## The distinction the repair turns on, and it is easy to lose

The architecture contract qualifies by **frontend**; ADR 0092's design qualifies by **dispatch**. These are different axes, and the flat quote hid that by collapsing both. `spikes/runtime/inline-dispatch` delivers through `tiler::tensor!` and then drives Metal by hand, so it *is* an inline-frontend consumer and sits inside the sentence's population — the amendment item 6 asks for is still owed. But granting it settles the dispatch axis **for inline-frontend consumers only**. `crates/tiler/src/lib.rs` states outright that what a consumer not using this frontend may name is undecided and that the crate answers nothing about it.

So the corrected item must say which axis it moves and leave the other parked. Widening it is the failure mode; narrowing it to nothing is the opposite failure.

## What closes this

Item 6 restated so the sentence it targets is quoted accurately and by **searchable anchor**, with the conjunction spelled out, and the status paragraph's shorthand replaced with a string the file contains. Record that the original citations were correct at their base and rotted, so the correction reads as maintenance rather than as an accusation.

**Do not repin by line number.** `:389` is the pin that rotted, and `:424` and `:435` have both already been correct at different times. The citing record's repair demonstrated the anchor form failing loudly on exactly this defect — `anchor occurs nowhere in docs/architecture.md`, exit 1 — where the line pin had passed silently for four days. Use that form.

**Residual sites outside this scope, corrected 2026-08-08.** The parked node `tickets/accept-the-public-route-requirement-answer-boundary.md` was reported here as still pinning `:389` in three places and `:424` in one; **that is no longer true** — the coordinator replaced all four with the anchor `the one crate an inline-frontend consumer names` and removed the emphasis marker the source does not carry, before this ticket was dispatched. Do not "repair" it. What remains is `spikes/runtime/inline-dispatch/Cargo.toml`, whose comment reads "The one crate a Tiler consumer declares" — the flat form again, and outside `contracts/decisions`. Report it; do not edit it here.
