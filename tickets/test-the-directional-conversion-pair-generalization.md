---
id: test-the-directional-conversion-pair-generalization
title: Test the directional conversion-pair generalization on a second pair
status: done
priority: p2
dependencies: []
related: [scope-the-in-type-precision-reduction-family, conform-the-bf16-vertical-end-to-end, carry-bf16-through-the-artifact-encoding-and-identity, derive-the-operation-family-and-signature-delivery-graph, land-the-conversion-pair-decomposition-adr, preserve-the-float-to-integer-conversion-precedent-sources]
scopes: [research/semantic-graph, research/numerics, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, numerics, conversion, dtype]
---
## User-visible outcome

`RQ-OP-04` is answered against evidence rather than by analogy: either every conversion pair decomposes into two directional families with disjoint field sets, or one keyed family parameterized by source, destination, and mode is correct at scale — and the corpus stops carrying an `n²`-growth question it has never tested.

## Why this is dispatchable now rather than deferred

**Fact — the question, and its falsifiable test.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s `RQ-OP-04` asks whether [ADR 0091](../docs/decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md)'s directional-pair decision "generalize[s] to every conversion pair, or is one keyed family parameterized by source, destination, and mode correct at scale?", blocks F-18 and F-19, and fixes the test: "Closes when a second pair is examined field by field. The test is falsifiable: if a second directional pair's field set is *not* disjoint in the way BF16/binary32's is, the generalization is refuted and the parameterized form wins. The `n²` growth is a cost to be stated, not the deciding argument."

**Fact — nothing about the test needs a workload, a target, or a measurement.** ADR 0091 is accepted and states the derivation to be generalized: narrowing owes a rounding rule, an overflow rule, and a subnormal rule that widening does not, and widening owes an exactness claim that narrowing cannot make. [ADR 0041](../docs/decisions/0041-separate-float-to-integer-conversion-families.md) independently accepts four float-to-integer families differing in exactly three fields. A second pair — the float-to-integer directions, or an IEEE binary pair — can be laid out field by field against those two accepted decisions today.

**Fact — the answer is close to load-bearing rather than academic.** BF16 reached R4 on 2026-08-01 with three registered keys and an exact-rational oracle, and ADR 0091's two conversion families are "registered in neither direction". [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s cast-and-convert row states the forcing condition in its own words: "Admitting any second dtype into a profile forces this row, because a mixed-dtype program cannot be expressed without an explicit conversion operation and no implicit promotion exists after semantic admission." A second dtype is admitted at the semantic and reference layers now.

**Inference — deciding the decomposition *after* the first conversion key is registered would be deciding it by accident.** Whichever shape the first registered conversion takes becomes the precedent every later pair is read against, and migrating it later migrates every identity that named it — the same cost ADR 0087 recorded when it chose one keyed family for the contraction.

## What the work is

Pick a second pair and walk its fields against ADR 0091's: what narrowing owes, what widening owes, and whether the two field sets are disjoint in the same way. Record the `n²` growth as a stated cost rather than as the argument. Then answer, and state the consequence for the *four* accepted float-to-integer families, which are already a directional decomposition of one logical conversion and therefore evidence on one side of the question rather than a neutral case.

## Explicit non-goals

- Registering any conversion key or choosing a Rust spelling. Both are Tom's under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md); this ticket produces the decomposition answer and the derivation behind it.
- Reopening ADR 0091 or ADR 0041. Both are accepted; this tests whether the first *generalizes*, and a refutation narrows its scope rather than superseding it.
- In-type precision reduction, which is not a conversion at all — [`scope-the-in-type-precision-reduction-family`](scope-the-in-type-precision-reduction-family.md) owns it, and the taxonomy is explicit that its result type never changes.
- Bit reinterpretation, which changes no numeric value and is a different question entirely.

## Closes when

`RQ-OP-04` is answered with the second pair's field-by-field derivation recorded, the `n²` cost is stated rather than assumed decisive, and the taxonomy's `RQ-OP-04` row names the answer — or the examination shows the two candidates are indistinguishable on this pair and names the third pair that would separate them.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-22** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), covering F-18 and F-19 together because a float-to-integer conversion is a directional pair under ADR 0041 exactly as a float-to-float conversion is under ADR 0091, and the question is whether that shape is the general one.
- Filed at `todo` rather than `deferred` deliberately: unlike every other track this record filed, its closure test names no workload, no target, and no measurement, so its trigger is already satisfied by the two accepted decisions it compares.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s cast-and-convert row is the delivery ledger and this ticket moves no rung.

## Scope added during the work, and why it was required

**`contracts/navigation`, added autonomously.** Two edits the work forces land in that scope and nowhere else. The new research record needs its row in `docs/research/README.md`, and the docs-maintenance rule is that a catalog is edited in the change that moves the metadata behind it — a record with no catalog row is the defect, not a deferral. And `docs/roadmap.md`'s cast-and-convert row named `RQ-OP-04` as "the next thing this row needs" and named this ticket as its owner, so answering the question makes that sentence false in the same commit that answers it. Neither edit expands the ticket's product outcome; both are the bookkeeping the outcome already implied.

**Fact — checked from the board rather than assumed.** No live ticket held `contracts/navigation` when it was added; `tkt list --status in-progress` returned four tickets, holding `research/indexing`, `research/region-search`, this ticket's own two research scopes, and a group of `implementation/*` plus `contracts/numerics`, `contracts/artifacts`, and `contracts/decisions`. `contracts/numerics` and `contracts/decisions` are both held by [`wire-the-delivered-realization-record-into-the-artifact`](wire-the-delivered-realization-record-into-the-artifact.md) and were therefore **not** taken: `docs/numerical-semantics.md` needs no edit, and the ADR is carried rather than written here.

## Outcome — 2026-08-05

**`RQ-OP-04` is answered, and neither of its two candidates is the survivor.** The derivation is [Conversion family decomposition across pairs](../docs/research/numerics/conversion-family-decomposition-across-pairs.md), with the enumerator retained at [`spikes/numerics/conversion_field_obligations.py`](../spikes/numerics/conversion_field_obligations.py).

**The nearest second pair did not decide it.** `f16`/`f32` was walked field by field and matches ADR 0091's shape exactly — `f16 → f32` owes nothing, `f32 → f16` owes all four — so the generalization survived the nearest test. What the walk did surface is that the *derivations* differ: BF16-to-binary32 widening is a sixteen-bit shift that inherits its NaN payload map from the encoding relation, while binary16 renormalizes and must choose one. The ticket's closes-when anticipated this branch and asked for "the third pair that would separate them".

**The third pair separates them, and it is the first incomparable pair in the catalog.** `bf16` has the wider exponent range and `f16` the wider significand, so neither format's value set contains the other. `bf16 → f16` owes `{rounding, overflow, underflow}` — the rounding owed not because `bf16`'s significand is wider (it is narrower) but because `bf16` values in `[2^-24, 2^-14)` land in `f16`'s subnormal range at a coarser step. `f16 → bf16` owes `{rounding, NaN mapping}`, the NaN mapping forced because the signalling `f16` NaN `0x7c01` truncates to the `bf16` infinity encoding `0x7f80`. **The two field sets intersect at `rounding`**, which is `RQ-OP-04`'s own falsification condition.

**Exhaustively, over the five IEEE-convention binary float layouts the dtype taxonomy states:** 20 ordered pairs, **4** distinct owed-field-set classes, **1 of 10** unordered pairs with non-disjoint direction field sets. Nine pairs owe nothing and nine owe all four — ADR 0091's two sets — and the two exceptions are exactly the incomparable pair. The empty-and-full split is a property of *comparability*, not of direction.

**The stated closure test is itself unsound, which is the finding with the longest reach.** It fires, and the candidate it prescribes — "one keyed family parameterized by source, destination, and mode" — is one ADRs 0010, 0041, and 0091 have each already eliminated by name. The record replaces it with a two-part test (constructibility and legibility) derived from the grounds those ADRs actually use.

**A premise in this ticket's own body is corrected.** "What the work is" describes ADR 0041's four families as "already a directional decomposition of one logical conversion". They are not: all four are float-**to**-integer, so they decompose one ordered pair by *mode*, and three of the four share a field set and differ only in what a field says. That refutes field-set disjointness as the corpus's separation criterion before any second pair is examined, since ordered saturation and NaN-to-zero were accepted as two families with identical field sets. The same correction lands in the delivery graph's grouping sentence, whose ground was the same misreading; the grouping itself is unaffected.

**The `n²` cost, stated and not decisive.** The growth is in registered identities — 240 ordered float-to-float pairs over the taxonomy's sixteen float rows alone, before integers and before ADR 0041's modes — and not in schemas: twenty ordered pairs carry four owed-field sets, computed from four predicates rather than looked up. The residual cost is migration, which is why the shape is worth fixing before the first key. The record also shows the "route through a wider intermediate" workaround is unsound in general, with a worked binary64 witness that differs by one `f16` ulp between the direct and routed conversions.

**Failure-path evidence.** The enumerator was watched failing: falsifying `bf16`'s exponent width from 8 to 5 in a copy makes the pair nested, turns `bf16 → f16` into `{}` and `f16 → bf16` into all four, and drops the non-disjoint count from 1 of 10 to 0 of 10. The decisive result rests on a stated layout, not on the script's structure.

**Nothing registered, nothing accepted, no rung moved.** No conversion key, no Rust spelling, no support-matrix movement; ADR 0091 and ADR 0041 are preserved and neither is reopened. The positive rule is labelled **Proposal** and drafted verbatim-landable for a `proposed` ADR.

**Documents deliberately not edited.** `docs/numerical-semantics.md` needs nothing — its section is titled "derived at the BF16/binary32 pair" and already states the derivation does not transfer. [`minimum-correct-physical-realization-profile`](../docs/research/program-planning/minimum-correct-physical-realization-profile.md) says "`RQ-OP-04` leaves conversion's family decomposition open, and neither changes the route"; the route half stays true and its family classification does not move, so only the word "open" is stale. That document is `research/program-planning`, a scope this ticket does not hold; the staleness is recorded in the new record's own divergence section so a reader following the citation is not misled, and it is a one-clause edit rather than a ticket.

**Filed rather than absorbed.** [`land-the-conversion-pair-decomposition-adr`](land-the-conversion-pair-decomposition-adr.md) carries the drafted body to `docs/decisions/`, a scope this ticket does not hold. [`preserve-the-float-to-integer-conversion-precedent-sources`](preserve-the-float-to-integer-conversion-precedent-sources.md) carries a preservation gap the walk surfaced: ADR 0041's sole evidence record cites seven bare URLs and names no preserved-source id, while three of the seven already have a pinned identity in the manifest — `llvm-langref-llvmorg-22.1.8` carries both LLVM claims, `stablehlo-spec-v1.18.0` carries `convert`, and `nvidia-ptx-isa-cuda-13.3.0` is metadata-only with a recorded digest.

**Acquisition requests flagged, neither blocking the answer.** IEEE Std 754-2019 (`ieee-754-2019`, metadata-only, purchase or subscription wall; nothing attempted here because the manifest already documents the wall and the route) would decide whether NaN payload preservation across a widening is a normative recommendation, which would soften one observation about `f16 → f32` and change no field set. OCP OFP8 v1.0 and OCP MX v1.0 (metadata-only, acquired by hand on 2026-07-31 with digests recorded, not vendorable) would let the enumeration extend to the catalog's `FN` and `FNUZ` rows — which owe infinity-mapping and signed-zero-mapping fields that ADR 0091's shape has no slot for, and which therefore strengthen the finding rather than qualifying it.
