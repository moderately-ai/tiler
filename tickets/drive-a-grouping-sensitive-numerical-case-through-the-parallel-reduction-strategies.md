---
id: drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies
title: Drive a grouping-sensitive numerical case through the parallel reduction strategies
status: in-progress
priority: p2
dependencies: []
related: [realize-parallel-reduction-strategies-on-metal, calibrate-and-activate-parallel-reduction-selection, implement-parallel-reduction-strategies, settle-contraction-chain-distributivity-permission]
scopes: [implementation/runtime, contracts/numerics]
shared_scopes: [project/tickets]
tags: [numerics, reductions, evidence-gap, measurement]
claimed_from: todo
assignee: agent-grouping
lease_expires_at: 1785696201
---
## User-visible outcome

The parallel reduction strategies are exercised on operands where regrouping actually changes the answer, so the evidence that they reduce correctly is separated from the evidence that a reassociation-permitting contract means what it says.

## Why this exists

**Measurement — the existing hardware evidence is exact by construction, and says so.** [`realize-parallel-reduction-strategies-on-metal`](realize-parallel-reduction-strategies-on-metal.md) executed the serial fold, the single-workgroup tree, and the multi-pass split on the qualified host and got `41700000` from all three, matching `tiler-reference` bit for bit. Its operands are `1.0, 2.0, 4.0, 8.0`. **Every grouping of those is exact in `f32`**, which is exactly what makes a single serial-fold oracle *valid* for all three strategies under a contract that permits regrouping — a grouping-sensitive operand set would make a correct regrouped implementation legitimately disagree with a serial reference, and the run would refuse a strategy for being right. Every subset sum is also distinct, so a dropped, double-counted, or unsynchronized contributor cannot cancel.

**Inference — so what is proved is contributor-set correctness, not rounding behaviour.** That ticket states it plainly: "Regrouped rounding was not observed and is not claimed." Nothing in the corpus currently observes a `FLUSH_AND_REASSOCIATE_F32` program producing a different — and permitted — answer from the serial fold. Until something does, the permission the contract grants is exercised only in the negative: the flush-only contract retains no split.

## The design problem this ticket owns, and it is the whole difficulty

A serial oracle cannot be the check. The moment operands are grouping-sensitive, disagreement with a serial fold is the *expected* outcome for a legally regrouped strategy, so the oracle has to change shape. State what replaces it before writing the case. Candidates to run the elimination over, not a menu to pick from:

- An exact higher-precision reference (float64 or exact rational) plus a derived bound on the permitted spread, with the bound derived from the declared contributor count and the contract rather than fitted to the observed answer.
- The exact set of groupings the declared strategy is permitted to produce, enumerated for a small contributor count, with the observed answer required to be a member.
- A pair of operand sets — one exact, one grouping-sensitive — where the exact one pins contributor-set correctness and the sensitive one pins that the answers differ *in the direction the strategy's own grouping predicts*.

**A tolerance is not an answer.** `docs/correctness-and-testing.md`'s standing position is that a difference is attributed to a named cause or it is a defect. An observed spread that is merely "small" proves nothing about whether the strategy grouped as it declared.

## Required evidence

- A grouping-sensitive operand set at a recorded seed or written out explicitly, with the exact `f32` bit patterns stated rather than decimal literals.
- Each strategy's answer recorded as a bit pattern, and each difference from the serial fold attributed to the specific regrouping the strategy declares — not to a tolerance.
- The check watched failing: perturb one strategy's grouping and confirm the oracle refuses the perturbed answer. An oracle that accepts every answer in the permitted spread must still refuse one outside it, and that refusal has to be observed.
- The measurement boundary stated exactly: host, OS build, toolchain, contributor count, contract, and what the result does **not** generalize to.

## Explicit non-goals

Measured crossover and winner activation stay with [`calibrate-and-activate-parallel-reduction-selection`](calibrate-and-activate-parallel-reduction-selection.md). No cost-model work; this is a numerical-meaning question, not a performance one. No new strategy.

## Closes when

A grouping-sensitive case runs both parallel strategies on a qualified host, every difference from the serial fold is attributed to a named regrouping rather than absorbed by a tolerance, the oracle has been watched refusing an answer outside the permitted set, and the boundary is stated.

## Graph maintenance

Filed 2026-08-02 at integration of `realize-parallel-reduction-strategies-on-metal`, which stated this boundary rather than letting an exact-by-construction operand set read as evidence about rounding.

## Outcome — 2026-08-02

**Measurement — regrouped rounding is now observed on hardware, and each difference is attributed to a named split.** Host: Apple M4 Max, macOS 27.0 build `26A5388g`, `arm64`, Apple9 — matching the ledger's execution-environment row in every field — offline compiler `Apple metal version 32023.883`, toolchain `nightly-2026-07-19`. Procedure unchanged: `cargo run -p tiler-prototype-compile -- --out <path>` then `cargo run -p tiler-prototype-run -- --artifact <path>`. At `1x4` under `FLUSH_AND_REASSOCIATE_F32`, on operands `0x3f400000, 0x3e800000, 0x33400000, 0x33000000` (`0.75`, `0.25`, `3 * 2^-26`, `2^-25`):

| Alternative | Declared partition | Answer | Against the serial fold |
| --- | --- | --- | --- |
| serial fold | 4 of 1 | `3f800000` | — |
| single-workgroup tree | **2 of 2** | **`3f800001`** | one legal regrouping away |
| multi-pass split | **2 of 2** | **`3f800001`** | one legal regrouping away |

Every alternative matched *its own* declared grouping bit for bit. The difference is `governed_partition`'s two-by-two blocked split: both share the exact prefix `0.75 + 0.25 = 1.0`, the serial fold then adds `0.375 ulp` and `0.25 ulp` in turn and each rounds back on its own, and the declared regrouping adds them to each other first — `0.625 ulp`, exact — so one add rounds up. No step is a tie, so nothing depends on round-half-to-even.

**Which oracle survived, and what eliminated the others.** The ticket's three candidates were run against correctness, maintainability, and performance. *Higher precision plus a derived bound* was discarded on correctness: a bound admits every value in an interval, including values no legal grouping produces and including the other strategy's answer, so it cannot separate a strategy that grouped as it declared from one that did not — and `docs/correctness-and-testing.md` already refuses "small" as an attribution. *Membership in the enumerated permitted set* is admissible but strictly dominated: the strategy's grouping is fully declared (`ContributorPartition` plus `ContributorOrder` plus `ContributorArrival`), so membership throws that declaration away and accepts a plan that produced some other legal grouping. *The pair of operand sets* survives as a structural decision but not as the oracle, because "the difference is in the direction the grouping predicts" is a weaker claim than the value the grouping produces, which is deterministic. What survives is the **exact value of the strategy's own declared grouping**, and the corpus already owned it: `tiler_reference::strict_partitioned_sum` exists for exactly this and its doc comment carries the same derivation — "a contract that permits reassociation admits a set of results, so no oracle can answer *the* value for it; what a plan can be checked against is the one order it selected". `crates/tiler-compiler/src/pipeline/tests.rs` already drives it CPU-side; this ticket carried it to a device. The partition is read from the plan's own published launch geometry, never assumed, and the degenerate one-contributor partition is cross-checked against `tiler-reference`'s evaluation of the whole semantic program so the oracle is calibrated by an independent path before any strategy is judged by it.

**Finding — at four contributors the two parallel strategies are numerically indistinguishable, by construction.** Both take their split from `governed_partition`, and `ContributorPartition` plus the `CooperativeWorkgroup` tile's `rounds: 1` make the tree's grouping identical to the split's: blocked contiguous ranges, serial within a partition, ascending across them. So this case separates {parallel} from {serial} and **not** the tree from the split. Separating those two needs a contributor count where their declared partitions differ; the profile's four-thread grid-axis row bounds this shape at four, so it is not reachable here. Recorded rather than hunted around, and it is a fact about the strategies rather than about the operands.

**Both operand sets are retained, and the counts say why.** Over four contributors there are five order-preserving groupings. On `PARALLEL_OPERANDS` all five produce `41700000`, so its refusal population among legal answers is **zero** — it cannot observe rounding — but of the sixteen single-contributor corruptions of the declared grouping it leaves **none** undetected. On the new operands the five produce two values, so the refusal population is one, and one corruption of the sixteen escapes. Neither half is a replacement for the other; `the_operand_pair_covers_what_each_half_alone_cannot` pins both counts device-free.

**Every check was watched failing on the device, and two were deleted for not being able to.** Perturbed, observed, reverted, and the clean run then reproduced byte-identically against the pre-perturbation log:

- reporting the tree's partition as the serial order → `the single-workgroup-tree declares 4 partition(s) of 1 contributor(s) and returned [3f800001], and that grouping produces [3f800000]`. The refused value is *legal* under this contract, which is the ticket's "refuses a wrong-but-in-range answer".
- running the case on `PARALLEL_OPERANDS` → `every order-preserving regrouping of these operands produces [41700000], so the serial-fold's oracle has no wrong-but-permitted answer it could refuse and observes no rounding`. The new check states the old gap in its own words.
- calibrating against the wrong partition, and an oracle answering two ULPs high → `the reference evaluator returns [3f800000] ... and the partitioned oracle returns [3f800001] at the declared serial order`.
- reporting three partitions for the split → `the multi-pass-split publishes no contributor partition this oracle can be asked about: 3 partition(s) do not cover 4 contributor(s) exactly once each`.
- an oracle that admits everything → caught three ways: the empty-population refusal on hardware and two of the three device-free cases.

Two variants were written and then **removed as unreachable** rather than kept: a permitted-set membership check (every blocked partition is order-preserving by construction, so it cannot fail, and the calibration and empty-population refusals catch a wrong oracle or a truncated enumeration first) and a second pass re-asking the oracle about values a filter had already refused with the same predicate. The absence of the first is recorded beside the enumeration so a reader does not re-add it.

**Measurement boundary.** One host, one contract, one row, four contributors, `f32` throughout, all operands normal and positive. It does not generalize to other contributor counts, other contracts, other rows, other hosts, subnormal or exceptional operands, or to any performance claim — measured crossover and winner activation remain with [`calibrate-and-activate-parallel-reduction-selection`](calibrate-and-activate-parallel-reduction-selection.md), untouched.
