---
id: retire-the-gate-reproduction-claims-in-the-apple-numerical-record
title: Retire the gate-reproduction claims in the Apple numerical behaviour record
status: done
priority: p2
dependencies: []
related: []
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, evidence, gate]
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

## Outcome — done, 2026-08-08

Landed at merge `a62f8caa` (worker commit `ef468fef`). One file, +27/−19, carries the green gate.

### The population classification was wrong twice, and both were mine to pass on

I briefed "32 occurrences, two of which use *gate* in an unrelated sense". Both figures are wrong:

- **One of the 32 is not the word at all** — it is the substring inside **`aggregate`**. Coordinator-confirmed: exactly one such occurrence. There are **31** real uses.
- **"Two lines in an unrelated sense" undercounts.** There are **three lines carrying five occurrences** — two ticket-dependency uses and three in a precondition-gate sense. And the ticket attributed the dependency sense to "finding 13's iOS row"; **neither site is in finding 13**. The *sense* was right, the *location* wrong.

I passed that classification through from the filing worker without checking it. A sweep built on it would have missed three occurrences and hunted in the wrong finding.

Final classification: **24 stale claims swept, 2 correct accounts kept verbatim, 5 unrelated sense untouched, 1 substring artifact.**

### Every swept sentence had a true claim underneath, and each was kept

The covering record is still "the set a default run compares" and the exhaustive one still is not; the guard tests are still portable and need no toolchain; a divergence still fails a test by name and kernel; the probe is still a one-off leg outside the harness matrix. Two anonymous "the gate checks" references now **name the actual tests**, verified by reading them.

Verified byte-identical to base: every `### N.` finding heading, every table row, and the frontmatter. The measurements were never in question — only the claim that anything runs them.

### A measurement that replaces an assertion

Running the harness today gives **165 passed, 1 skipped in 56.14s**, and the skip declines on **fifteen differing environment fields** — `xcode-select -p` now Xcode-beta, `metalfe-32023.921` against the pinned `32023.883`, all three SDK versions and builds, and both registry IDs. So **a run today exercises the harness's whole portable half and re-verifies not one retained value.** That is now its own dated Measurement rather than a claim.

### Flagged for the parallel ticket

The `contracts/decisions` half of this defect landed separately at `7291350f`. If it inherited the same population claim, its worker would have hit the same three-lines-not-two discrepancy — it did not, having worked from ADR text rather than this record's count.
