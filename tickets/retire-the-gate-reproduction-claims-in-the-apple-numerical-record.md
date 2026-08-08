---
id: retire-the-gate-reproduction-claims-in-the-apple-numerical-record
title: Retire the gate-reproduction claims in the Apple numerical behaviour record
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, evidence, gate]
claimed_from: todo
assignee: w-retire-th
lease_expires_at: 1786165341
---
## The record corrects the claim once and then makes it twenty-two more times

Verified 2026-08-07 at base `7c371155` by `verify-and-file-the-remaining-maturity-audit-leads`, while checking the sibling claim in ADR 0076. The audit lead that produced this ticket reported the research record as *agreeing with* ADR 0076's correction. **It does, in one sentence, and it is also the largest concentration of the falsified claim in the repository** — which is the opposite of what the lead implied, and is why this is a separate ticket rather than a note on the ADR one.

**Fact — the ground.** `e197176` replaced the Python gate with the root `Makefile`, and `Makefile "Spikes deliberately have no target."` states the consequence. No target reaches `spikes/`, so nothing collects `spikes/apple-targets/test_numerical_probe.py` and nothing compares the retained `record.tsv` against a fresh run.

**Fact — the record already knows this, at exactly one site.** The provenance paragraph carries `docs/research/apple-targets/numerical-behaviour.md "which nothing collects"`, names `e197176`, gives the by-hand command, and states that a toolchain change altering a measured value will not fail any gate. That sentence is correct and must be kept.

**Fact — and the same line then contradicts it.** That paragraph closes by calling the retained covering record `docs/research/apple-targets/numerical-behaviour.md "the set the gate runs"`. So the correction and a stale claim sit in one line.

**Fact — the Status block makes the strongest form of the claim.** `docs/research/apple-targets/numerical-behaviour.md "reproduced by the repository gate on every run"`, qualified only by the exhaustive-only rows. A reader who stops at Status — which is what a Status block is for — takes away the opposite of the provenance paragraph three lines below it.

**Measurement — the population.** `grep -o 'gate' docs/research/apple-targets/numerical-behaviour.md | wc -l` returns **32** occurrences across **22** lines. Two of those lines use the word in an unrelated sense and must not be swept: finding 13's iOS row is *gated on* a ticket, and finding 32's paragraph is about a measurement precondition gate named by another research record. Both were read; neither is about the repository gate. The remainder assert that the repository gate runs, compares, reproduces, or is failed by this harness, in forms including "fails the gate", "on every gate run", "the covering set the gate runs", "the gate compares", "reproduced by the gate on every run", "a gate assertion requires", and "excluded from the gate".

## The conclusion versus its ground

**The measurements survive intact and none of them is in question.** Every value here was measured on the stated environment row and is retained under `spikes/apple-targets/results/`. What is false is the custody claim wrapped around them — that a toolchain change would be caught. The record's own reasoning about *why* custody matters is also correct and is worth keeping: the provenance paragraph's point that "a hand-run measurement in this repository has already stopped being true within the hour" is the argument for the harness, and losing the gate makes that argument sharper rather than obsolete.

**So this is a custody sweep, not an evidence retraction.** A repair that weakens a measured finding, or that deletes the harness's description, has overshot. Several passages are also making a *second*, still-true claim underneath the gate wording — that the covering set exists and is the set compared, that a guard test is portable, that a row measured only under `TILER_APPLE_NUMERICS_EXHAUSTIVE` is weaker evidence than one in the covering set. Those distinctions are real and survive; only "and the gate does it automatically" is false.

## Requirements

- Sweep all twenty-two lines. Read each in full and decide, per site, which half of the sentence is the gate claim and which is the surviving distinction; do not regex-replace.
- Keep the two unrelated senses (finding 13's ticket gate, finding 32's precondition gate) untouched.
- Reconcile the Status block with the provenance paragraph, so a reader who stops at Status is not told the opposite.
- Record the sweep in the record's own dated-correction form, naming `e197176`, rather than silently rewording.
- Say what re-running costs and what it buys, since the by-hand command is now the only custody: `uv run --with pytest pytest spikes/apple-targets`.
- A dated correction must quote a retired extent in prose or as a bare `:LINE` suffix, never pinned to a path, or `make citations` will demand it resolve.

## Required evidence

- The count of sites corrected, and the count of "gate" occurrences remaining with the reason each survives.
- Confirmation that no retained measurement value, environment row, or finding number changed.

## Closes when

No passage in the record asserts that any gate runs, compares, or reproduces this harness; the Status block and the provenance paragraph agree; the two unrelated gate senses are intact; every retained measurement is unchanged; and `make citations` is green.
