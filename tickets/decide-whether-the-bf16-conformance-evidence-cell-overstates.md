---
id: decide-whether-the-bf16-conformance-evidence-cell-overstates
title: Decide whether the BF16 conformance-evidence cell overstates without the end-to-end run
status: done
priority: p3
dependencies: []
related: [conform-the-bf16-vertical-end-to-end, carry-a-bf16-subnormal-realization-the-reference-can-be-told, re-read-the-bf16-and-elementary-support-rows-against-source]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [navigation, bf16, maturity-claims]
---

## The question (raised by the BF16 vertical's discovery stop, 2026-08-06)

**State at raise (2026-08-06).** `docs/dtype-support.md`'s BF16 `Conformance evidence` cell was quoted in the question as `[tested guarantee, macOS Apple9 only](#other-ieee-binary-floats-and-bf16)` (that text is the **`Target-family dispatchability`** cell — see Verdict). The vertical ticket (`conform-the-bf16-vertical-end-to-end`) was then `blocked` and established that no end-to-end BF16 run existed — the layers were each tested against their neighbour and nothing tested the composition — and separately that the reference could not yet apply the measured flush (`docs/correctness-and-testing.md` then carried that exception with its reproducing check). The question was whether "tested guarantee" in that cell already claimed more than the per-layer evidence supported, or whether the anchor section's own text bounded the claim to what existed.

**Superseded 2026-08-07 / correction 2026-08-10.** `conform-the-bf16-vertical-end-to-end` and `carry-a-bf16-subnormal-realization-the-reference-can-be-told` are both `done`. The flush-applied oracle comparison is live, and a device-reaching BF16 vertical exists under `crates/tiler-conformance` (`bf16_vertical`). Live cell text is under Verdict; do not read the raise-time framing as current repository state.

## The work

Read the cell's column definition and the full anchor section, compare against the vertical ticket's evidence list and the correctness-and-testing exception, and either qualify the cell (e.g. per-layer only, composition untested and blocked behind the flush ticket) or record why the current text is already bounded. Whichever way it resolves, the derivation lands in the section so the next maturity audit does not re-derive it. AGENTS.md's maturity-claim rule governs: a tested guarantee is the strongest of the four claims and must not cover an untested composition.

## Closes when

The cell and its anchor agree with the verified evidence, and the resolution's derivation is recorded at the anchor.

## Verdict — it overstated, and the cell is qualified

**On 2026-08-06 this ticket set the cell to `tested guarantee, per-layer corpora only; no end-to-end run`.** The derivation is recorded at `#other-ieee-binary-floats-and-bf16` so the next maturity audit reads it rather than re-deriving it.

**Correction — 2026-08-10.** That intermediate wording is not current cell text. On 2026-08-07 `conform-the-bf16-vertical-end-to-end` restated the physical-matrix `Conformance evidence` field to `tested guarantee, per-layer corpora and one device run crossing neither the optimizer, the artifact envelope, nor the routing commit` (Decision paragraph under `#other-ieee-binary-floats-and-bf16`, **Corrected 2026-08-07**: the run exists and the cell is restated; what replaces "no end-to-end run" is that narrower bound, not an unqualified "end to end"). Status `done` remains correct for the qualification and derivation this ticket owned; the later restatement is a separate landing.

**Fact — this ticket quoted the wrong cell, and the true text was weaker than the question assumed.** The question says the `Conformance evidence` cell reads `[tested guarantee, macOS Apple9 only](#other-ieee-binary-floats-and-bf16)`. That text is the **`Target-family dispatchability`** cell, one column to its left; splitting the BF16 physical row on `|` gives ten fields, of which field 9 is the dispatchability cell and field 10 was a bare `[tested guarantee](#other-ieee-binary-floats-and-bf16)` carrying no boundary at all. So the cell was less bounded than the ticket that suspected it of overstating — and identical, word for word, to the `f32` conformance cell, which rests on a device-executed thirty-case bit-for-bit comparison (`prototypes/serial-sum-run/src/proof.rs`). Reproduce: `awk 'NR==70' docs/dtype-support.md | tr '|' '\n' | nl` against the pre-edit file.

**The column's own definition is what settles it.** `Cell vocabulary` defines a tested guarantee as "checked evidence exercises **the linked claim** within the boundary stated in the family notes", and rung 13 of the dtype-addition recipe defines conformance as "the corpus that would catch a regression", evidence class "exhaustive-finite where the format allows it, else a stated bounded profile". The family notes state a boundary for every *other* tested BF16 cell and stated none for this one, so the qualifier was the missing half of the vocabulary's own contract rather than an extra caution.

**What exists at raise, verified at the anchor's own citations (historical 2026-08-06 evidence base for the qualification):** the exact-rational reference oracle, exhaustive-finite over all 65,536 encodings plus thirty hand-derived witnesses with a tie-rule perturbation disagreeing at exactly four; a BF16 kernel agreeing with that independent oracle bit for bit over ten witnesses, with the canonicalization perturbation disagreeing at exactly the NaN element; a producer-built artifact that round-trips, re-derives its identity, and re-encodes byte-identically; a BF16 golden compiled and linked offline on the measured row; and the routing refusals, perturbation-checked. Each is a corpus over one layer or one adjacency.

**What did not exist at raise, and why "per-layer" was the honest bound then.** At 2026-08-06 `conform-the-bf16-vertical-end-to-end` established that nothing composed them: no BF16 kernel had dispatched, so no device result had been compared against the oracle. Its own reasoning was the strongest argument for the qualifier — a two-byte element counted as four survives every single-layer test that uses consistent counts on both sides, so the untested composition is exactly where this family's characteristic defect would live. Worse than unrun, the composed comparison was then **unstatable**: `docs/correctness-and-testing.md#semantic-authority` recorded that `ReferenceNumericalConformance` applied its subnormal dimensions through `f32`-typed operand and result functions the three BF16 capabilities read nowhere (`grep -n conformance crates/tiler-reference/src/bf16.rs` → two hits, both module header), so the measured flush could not be applied to the oracle and the oracle was the side that would be called wrong. `carry-a-bf16-subnormal-realization-the-reference-can-be-told` owned that gap, and the end-to-end ticket was `blocked` behind it.

**Correction — 2026-08-10 (composition and flush).** Both raise-time clauses above were discharged on 2026-08-07 rather than argued away: `wire-the-bf16-reference-to-the-realization-it-is-told` made the flush statable against the oracle, and `conform-the-bf16-vertical-end-to-end` (status `done`) ran a device composition under flushing conformance. The Decision paragraph's **Corrected 2026-08-07** clause records that restatement; the live cell still excludes runs that cross the optimizer, the artifact envelope, or the routing commit.

**Why the answer is not "already bounded".** The anchor's per-layer paragraphs do each state their boundary, and a reader who read all of them start to finish could infer that no composition exists. But a maturity ledger is read by cell, its own preamble says so ("Read a cell only for its named family and layer"), and AGENTS.md's rule that a tested guarantee must not cover an untested composition is a constraint on the cell rather than on the section. An unqualified cell beside eight qualified ones reads as deliberate breadth.

**The `f16` conformance cell was checked and needs no change, which is what isolates BF16's defect.** It is also a bare `tested guarantee`, but its whole evidence base is the Apple measurement record, and the family notes bound it in one sentence — "The checked Apple record supplies conformance evidence for F16 and BF16 arithmetic **on its exact host/toolchain/family rows**". Every other `f16` cell in both matrices is `absent/unsupported`, so nothing has accrued for that sentence to stop covering. BF16 is where it broke: the same sentence once bounded its cell too, and five further corpora have landed under it since — reference, kernel, artifact, offline compile, routing — none of which it describes and none of which composes with the others. The failure mode is a boundary sentence outliving the evidence it was written for, not a wrong sentence.

**Not done here.** No cell other than `Conformance evidence` moved on this ticket's account. `docs/roadmap.md`'s BF16 row makes no conformance claim this contradicts and was left alone.
