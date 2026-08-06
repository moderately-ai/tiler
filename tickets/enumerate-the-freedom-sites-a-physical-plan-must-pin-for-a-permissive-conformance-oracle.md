---
id: enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle
title: Enumerate the freedom sites a physical plan must pin for a permissive conformance oracle
status: done
priority: p2
dependencies: []
related: [derive-the-oracle-for-a-permitted-divergence-candidate, apply-the-declared-numerical-conformance-on-every-reference-evaluation-path]
scopes: [research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, reference, conformance, scheduling]
---
## User-visible outcome

A complete enumeration of the places a physical plan can spend a categorical numerical permission, and for each one whether the plan already declares the choice it made — so the order witness an oracle consumes can be built from facts that exist, or the gaps are named.

## Why this exists

**Fact — [the oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) settled the object and left exactly this open.** The surviving oracle takes `(program, contract, realization witness)` and compares bit for bit. Its Part 2.4 records that most of the witness is already carried — `ReductionTopology` (`crates/tiler-ir/src/schedule/model.rs:563`) carries `ContributorPartition`, `ContributorOrder`, `accumulation`, `arrival`, and both permission fields on every folding variant, and the schedule verifier cross-checks them against the region's declared realization. What it does not record is whether that is *all* of them.

**Fact — it is not all of them, and one counterexample is already compilable.** [Numerical semantics](../docs/numerical-semantics.md) records that a one-input, one-output, three-leaf same-family `f32` add or multiply chain compiles through the `PointwiseF32Expression` projection when the contract admits one of the implemented reassociations. Such a region carries `ReductionTopology::None`, so no field of the plan names which grouping it emitted. That is the derivation's `OrderNotPinned` refusal class with a non-empty population today.

**Inference — the witness is an aggregation over freedom sites, not over reductions**, and nobody has counted the sites. Today the aggregation is done by hand, per test: `crates/tiler-compiler/src/pipeline/tests.rs` reads `partition` out of the region and passes it to `strict_partitioned_sum`; `prototypes/serial-sum-run/src/proof.rs` reads it from the plan's published launch geometry. Neither generalizes.

## What this ticket must produce

- **The enumeration**, over the eleven canonical dimensions and every construct a plan can carry, of where a categorical permission is *spendable* — read at source and stated with the exact path, not inferred from a type's name. The derivation's Part 3 table is the starting classification and is to be checked rather than inherited.
- **Per site, whether the plan declares its choice**, with the field named or the absence stated as an exact check a reader can rerun.
- **What a witness would have to determine**, stated so it can be refuted: two plans agreeing on the witness must agree in bits, and the way to refute the enumeration is to exhibit two that do not.
- **The public boundary identified and taken to Tom, never self-accepted.** A witness type, any change to `ReferenceNumericalConformance`'s construction, and any new plan-side field are each a public boundary under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md). This ticket produces the derivation and the exact surface; acceptance is Tom's.

## Explicit non-goals

Implementing a witness or an oracle; editing `crates/`; changing a contract sentence; deciding whether the `PointwiseF32Expression` projection should carry a grouping field, which is a consequence of the enumeration rather than an input to it.

## Closes when

Every freedom site is enumerated at source, each is marked declared or undeclared with a reproducible check, the witness's determination property is stated refutably, and the public surface is written out for Tom without being adopted.

## Outcome

**Delivered:** [The freedom sites a physical plan must pin](../docs/research/reference/plan-freedom-sites.md), written at base `c335bb5b`, linked from the parent derivation's Traceability, Part 7 gap list, and four-outcome roll-up.

**The enumeration: twenty-four sites over the eleven canonical dimensions, split five ways rather than two.** Six are witnesses a reference path can evaluate (both subnormal dimensions; `ReductionTopology::Serial`, `MultiPass`, `CooperativeWorkgroup` at `rounds == 1`, and `Contraction`). Three are witnesses no reference path can evaluate (`ScalarProgram::PointwiseF32`, its `bf16` sibling, and the declared `accumulation` width). Five are **mirrors** — a field exists and is named for the dimension but carries the contract's grant rather than the plan's choice (`FusedMultiplyAddSerialSum.contraction`, `ContributorArrival`, `permits_permutation` on the two `Serial` sites, and both exceptional-value assumptions). Four are undeclared (`StrictTensorContraction`'s fused step, the backend compiler's own contraction, `SquaredSerialSum`'s step, and which semantic candidate was selected). Six are unspendable by construction (`ContributorOrder`, `SignedZero`, `ReciprocalTransform`, both approximate-intrinsic nodes, and `MaterializationRounding`). So: **fourteen declared, four undeclared, six unspendable** — and the fourteen include five that determine nothing.

**Fact — exactly one dimension is ever spent on the compile path.** `grep -rn "\.effective(" --include='*.rs' crates/` returns seven lines; six are inside `policy.rs`'s `#[cfg(test)] mod tests` (opens at `crates/tiler-compiler/src/policy.rs:736`) and the one live caller is `crates/tiler-compiler/src/normalize.rs:749`, passing `Reassociation`. Independently `grep -rn "permits_contraction" --include='*.rs' crates/` returns two lines, neither a transform gate.

**The counterexample verification result: refuted as stated, and reclassified rather than emptied.** The three-leaf `f32` chain does carry `ReductionTopology::None`, and that variant has no grouping field — but the topology was never the site. `PointwiseF32Node::Add { lhs, rhs }` (`crates/tiler-ir/src/schedule/pointwise.rs:91`) is a binary node over dense topological ordinals and `mint_elementwise` (`crates/tiler-compiler/src/request.rs:4697`) mints the tree as a faithful image of the possibly-reassociated semantic DAG, so the grouping **is** pinned, exactly, by `ScalarProgram::PointwiseF32`. The site therefore belongs to `RealizationNotEvaluable`, not `OrderNotPinned`: no reference path can evaluate a `PointwiseF32Expression` (`grep -rn "ScalarProgram\|ReductionTopology\|VerifiedScheduledRegion\|PointwiseF32" crates/tiler-reference/` returns one line, a doc comment in a test), and the rewritten `SemanticProgram` the semantic evaluator *could* have evaluated exactly is not retained (`grep -rn "pub fn .*SemanticProgram" crates/tiler-compiler/src/` returns nothing). `OrderNotPinned` stays non-empty at three other sites.

**This answers the ticket's deferred non-goal in the negative.** The `PointwiseF32Expression` projection should **not** carry a grouping field: it already carries the grouping, and a field would be a second spelling of a fact the node vector determines.

**Eleven discrepancies against the parent derivation, tabulated in the record's Part 6.** The three a reader must know: gap 1 is **closed** — the parent's own check `grep -rn 'apply_to_operand\|apply_to_result' crates/ --include='*.rs'` returns 67 lines across ten files at this base, not fourteen across two, landed by `f64956aa`, and `apply-the-declared-numerical-conformance-on-every-reference-evaluation-path` is `done`; Part 3's `ReciprocalTransform` and `ApproximateIntrinsics` rows are **overstated** (both are authorized-and-unspendable like permutation, withheld from every capability row by `ELEMENTARY_UNCARRIED_DIMENSIONS` at `crates/tiler-compiler/src/policy.rs:476`); and `Contraction`'s "the emitted kernel pins it" is **false** (the only field is a contract mirror and no lowering reads it). Three cited line numbers had drifted, including `ReductionTopology`, which is at `model.rs:715` and not `:563`.

**The public surface is drafted and parked, never self-accepted.** Record Part 7 states three items — `RealizationWitness` in `tiler_ir::schedule`, an `UnpinnedFreedomSite` refusal enum, and `ReferenceNumericalConformance::from_witness` — with the dependency consequence of the third named explicitly and a plain-scalar alternative drafted beside it. Parked at `accept-the-realization-witness-surface`, `awaiting-decision`.

**Navigation note — the catalog row is not this ticket's to write.** `docs/research/README.md` belongs to `contracts/navigation` (`ticketsplease.toml:96`), which this ticket does not hold, so the new record has no catalog row. The row to add, in the file's existing form: `- [The freedom sites a physical plan must pin](reference/plan-freedom-sites.md) — pending; primary-source-synthesis, sound-proof; informs: [Correctness and testing](../correctness-and-testing.md), [Numerical semantics](../numerical-semantics.md)`. Reported to the coordinator rather than written here.

**Measurement boundary.** Nothing was run. Every claim is a source reading at `c335bb5b` with a file, a line, and — for each absence — the one-line command that reproduces it. No cargo build, no test, no device execution.

## Graph maintenance

Filed by [the permitted-divergence oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) as the second of its two ownerless gaps.

**Filed by this ticket:**

- [`accept-the-realization-witness-surface`](accept-the-realization-witness-surface.md) — `awaiting-decision`. The ADR 0075 public boundary, drafted with its evidence and not adopted. Only Tom closes it.
- [`decide-how-a-pinned-pointwise-grouping-becomes-evaluable`](decide-how-a-pinned-pointwise-grouping-becomes-evaluable.md) — `todo`. The architecture fork the enumeration surfaced: retain the selected semantic candidate's program, or write an exact evaluator for the physical projection. Both drafted in the record's Part 7.3; neither is correctness-dominant, so the record parks rather than picking.
- [`record-the-contraction-choice-a-fused-fold-actually-made`](record-the-contraction-choice-a-fused-fold-actually-made.md) — `todo`. The mirror field and the missing field at the two plan-side contraction sites.

**Already filed elsewhere, and confirmed still correct at this base:** the backend-compiler question is `measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order` (`todo`); the multi-round tile evaluator is `derive-the-exact-evaluator-for-a-multi-round-cooperative-fold-order` (`deferred`); carrying the two elementary dimensions is `carry-the-elementary-numerical-dimensions-in-the-region-realization` (`todo`).
