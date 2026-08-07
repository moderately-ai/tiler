---
id: derive-per-locus-numerical-obligations
title: Derive per-locus numerical obligations in the compiler
status: in-progress
priority: p2
dependencies: []
related: [redesign-the-delivered-realization-record-from-typed-evidence, accept-the-delivered-realization-artifact-surface]
scopes: [implementation/compiler, contracts/numerics, implementation/build, research/target-profiles, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, compiler]
claimed_from: todo
assignee: agent-per-locus
lease_expires_at: 1786113672
---
## Why

`redesign-the-delivered-realization-record-from-typed-evidence` shapes the delivered-realization record around `(subject, dimension, locus)` obligation rows, because ADR 0011's per-operation restrictions attach to a *position*: one `f32` operation's accumulator and its observable materialization boundary can carry different legal requirements, and a dtype-wide ceiling alone keeps whichever was written last.

> **The Fact below is stale and was corrected 2026-08-07 by the coordinator at `879dec67`. Do not brief from the struck version.** It read: "the compiler cannot produce such a row today. Exact check: `grep -rni "locus" --include="*.rs" crates/` returns nothing." That check now returns **166 matches**. The locus vocabulary landed, and so did a producer — just a single-locus one. What remains open is narrower than what this ticket was filed for, and a worker briefed on the old text would re-derive vocabulary that already exists.

**Fact, 2026-08-07 — the vocabulary exists and the compiler already emits per-occurrence obligations at one locus.** `PolicyLocus` is declared at `crates/tiler-ir/src/numerics.rs:1024` with `NumericalObligationKey` beside it, and `crates/tiler-compiler/src/session/realization.rs:377` constructs `NumericalObligationKey::new(*occurrence, PolicyLocus::Computation)` — its module header (`:39`) states the shape in terms: one row "per honoured dimension at `PolicyLocus::Computation` of **every** occurrence". So the occurrence half of the key is derived and the locus half is pinned to a constant.

**Fact — `dimension_requirements` is no longer the eight-whole-program projection this ticket described either.** It now derives its subject from the caller's contract through `arithmetic_subject` rather than hard-coding `F32::resolved_type()`; that change landed under `6207fba4` and retired the sibling ticket [`key-numerical-requirements-by-the-contract-s-own-resolved-type`](key-numerical-requirements-by-the-contract-s-own-resolved-type.md), closed `obsolete` on 2026-08-07. The dtype-wide ceiling and the locus obligations remain separate statements, which is the rule below that survives unchanged.

**What is actually open.** Exactly the locus half: an obligation whose locus is drawn from the full set — input, computation, accumulator, result, component, and materialization — rather than fixed at `Computation`. This is the "single-locus producer" this ticket's own Graph maintenance anticipated the record would admit; it landed, and widening it is the remaining work. Recheck the boundary with `grep -rn "PolicyLocus::" crates/tiler-compiler/src/` — while every constructed key reads `PolicyLocus::Computation`, this ticket is open.

## What closes this

The compiler derives, per selected plan, one obligation per `(policy subject, dimension, program occurrence, policy locus)` that a packaged route relies on, with the locus drawn from input, computation, accumulator, result, component, and materialization. The occurrence is `tiler_ir::program::SemanticOccurrence`, so an obligation and the stage coverage implementing it name the position the same way.

Two rules survive unchanged: the dtype-wide ceiling and the locus obligations are separate statements, neither derived from the other; and a locus requirement is at least as strict as the ceiling, never weaker.

## Worker record, 2026-08-07

**Fact — the producer derives the locus from the operation at the occurrence.** `session/realization.rs::materialize` gained the resolved lowering as a fourth authority. It joins the packaged program's proof-derived coverage with `OccurrenceLowering`'s refinement receipt, which carries both the `SemanticOccurrence` it minted and the `OpKey` it realized, so occurrence and operation come from one proof rather than two structures a caller could pair wrongly. `policy::OperationNumericalCapability::founded_locus` then sites the dimension.

**Loci emitted, and what founds each.** Input (input subnormals: "treatment of subnormal operands *before each arithmetic operation*"); Result (result subnormals: "a newly *produced* subnormal arithmetic result"); Accumulator (permutation always, since contributor order exists only in a fold; contraction and reassociation for a fold-bearing family, whose capability rows are founded on the per-contributor step `accumulator + a * b`); Computation (signed zero and both exceptional-value dimensions, and contraction and reassociation for pointwise arithmetic, which has no fold). `folds()` is derived from the permutation entry rather than a second list, and `the_fold_bearing_families_are_exactly_the_reducing_ones` pins the four families by name.

**Loci deliberately not emitted.** *Component* — needs a compound encoded value whose conversion behaviour is its own versioned scheme contract; the three strict-affine families consume no generic dimension, so nothing founds a component position. *Materialization* — would be founded by `MaterializationRounding`, which names a boundary *between* stages rather than a position inside one occurrence, and which no admitted operation consumes, so no contract places it on a target and no honoured fact exists for a row to carry. `ReciprocalTransform` and `ApproximateIntrinsics` are unfounded for the same class of reason: they act on a subordinate operation inside a composite family, carried by that family's own `AccuracyContract`. All three return `None`, and a consumable dimension with no founded position is a typed refusal rather than a substituted computation row.

**Fact — the narrowing moves a locus, not a disposition.** An occurrence whose operation cannot consume a dimension now contributes no row. That is required for the locus to be founded at all: a constant has no position at which a rounding freedom acts. The artifact derives `NotRequired` from an empty row set, so this is only safe because a dimension *some* covered occurrence consumes still carries its rows; a dimension no covered occurrence consumes is genuinely not relied on by any packaged route.

**Fact — the at-least-as-strict rule is enforced, not assumed.** `policy::is_at_least_as_strict_as` is a partial order per behaviour space: each space has one strict resolution (the one `strict_contract` writes), widenings are ordered against it, and behaviours that are merely *different* — two flush modes, two absence provenances — are refused rather than given an invented order. Two spaces never compare. `materialize` checks every row against the contract's own ceiling before retaining it and refuses `structure-numerical-realization-locus-weaker-than-ceiling` otherwise. Watched failing: substituting `Transform(Permitted)` for the contraction requirement under `STRICT_F32` produced exactly that rule identifier, restored and re-run green.

**Measurement — identity pins moved, one domain, no grammar step.** `tiler.artifact-program.v15` folds the delivered-realization record's canonical bytes. For `metal_plan`'s fixture (two constants, a multiply, an add, a strict serial sum) the row set went 20 -> 11: the constants consume nothing and the sum consumes no contraction. Artifact identity `17a16aa4d15b35a0eae7e382b9e96ea3fca7c01a5a1c80495600aace20f2e63d` -> `674ad9aaccf8d98c0fcac3d83da8e5809ce0aba4711d967440371e7f995d48f6`; cache subject `a3d44827bf86b5979f3d79eaf7e9392f997255ae88376edfb6f8f304e51cdfe8` -> `3a34fb8aefafeffcaff58b63387d20e9f7097547145980c3ec3f353573a79ca3`; fixed content 64,710 -> 64,530 bytes, exactly 180 = 9 dropped rows x 20 bytes per row. No identity domain stepped: the row layout, the locus tag space, and every dimension's `Required` disposition are unchanged.

**Scopes added, and why.** `implementation/build` for the three pins in `metal_plan.rs` and their ledger; `research/target-profiles` for the ledger paragraph that mirrors those pins; `research/numerics` for the research record stating the superseded producer rule. Each is documentation or a pin the change itself moved.

**Evidence.** `the_locus_follows_the_operation_at_the_occurrence` counts 5 covered occurrences, 3 obligated, 11 rows, and shows reassociation at `Accumulator` for the sum against `Computation` for the multiply and the add, cross-checked against the contraction rows that name the same two occurrences. `two_loci_of_one_occurrence_carry_different_obligations` compiles a contract flushing operand subnormals while preserving result subnormals and shows each of the three arithmetic occurrences carrying two positions with genuinely different required behaviours — the row set the single-locus producer collapsed onto one key. `the_strictness_order_is_equality_or_the_strict_resolution` sweeps all 144 ordered pairs of the 12-behaviour vocabulary against an oracle derived from `strict_contract`. `every_consumable_dimension_founds_a_locus` counts 50 consumable (operation, dimension) pairs and pins the emitted locus set to exactly four.

**Remainder.** `docs/artifact-abi.md:178` states the manifest schema moved to "13.0" while `MANIFEST_SCHEMA` is `(15, 0)`; that drift predates this ticket and `contracts/artifacts` was not claimed here. The spike packet at `spikes/numerics/delivered-realization-record/README.md` still records the producer gap as open, and is left as the dated review record it is.

## Graph maintenance

- The review packet at `spikes/numerics/delivered-realization-record/` records the shape this must fill; do not re-derive it.
- This does not block `accept-the-delivered-realization-artifact-surface`: the packet is reviewable, and the record admits a single-locus producer.
- `wire-the-delivered-realization-record-into-the-artifact` may land against the single-locus producer; this widens what that path carries.
