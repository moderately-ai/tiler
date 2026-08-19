---
schema: "tiler-doc/v1"
id: "tiler.research.reference.plan-freedom-sites"
kind: "research"
title: "The freedom sites a physical plan must pin"
topics: ["reference", "numerics", "conformance", "scheduling", "correctness"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "pending"
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis", "sound-proof"]
informs: ["tiler.contract.correctness-and-testing", "tiler.contract.numerical-semantics"]
depends_on: ["tiler.research.reference.permitted-divergence-oracle"]
ticket: "enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle"
---

# The freedom sites a physical plan must pin

**Status:** enumeration complete. Every place a physical plan can spend a categorical numerical permission is enumerated over the eleven canonical dimensions and every construct a plan carries, each read at source at this record's own base, `c335bb5b`. Each site is marked declared or undeclared with the field named or the absence stated as a command a reader can rerun. The witness's determination property is stated so it can be refuted. The public surface is drafted for Tom and is **not** adopted here. Nothing in this record changes a contract, admits a permission, or edits `crates/`. **Two of Part 7's three drafted items have since been accepted and built, and building them corrected this record in five places** — read [Part 7.5](#75-what-landed-where-and-what-the-implementation-corrected) before acting on Part 2's table, Part 3.2, Part 3.3, or Part 5.

Claims are labelled **Fact** when traced to inspected source or a merged record; **Inference** when derived from stated facts; **Measurement** when tied to an exact environment and procedure; and **Proposal** when not yet accepted or tested.

## Traceability

- **Work record:** [`enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle`](../../../tickets/enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle.md), filed by [the permitted-divergence oracle derivation](permitted-divergence-oracle.md) as the second of its two ownerless gaps.
- **Parent derivation, corrected in four places rather than inherited:** [The oracle for a permitted-divergence candidate](permitted-divergence-oracle.md). Its Outcome, its Part 3 table, and its Part 7 gap list were each checked against source at this base; Part 6 below records what moved and why. The parent's *surviving object* — the oracle is `(program, contract, realization witness)` compared bitwise — is unchanged by anything here, and this record's corrections all narrow or relocate its populations rather than reopening its elimination.
- **Current disposition:** pending. No ADR adopts this record and no contract sentence has moved for it. The last two clauses of this bullet have expired and are preserved rather than deleted: at the time of writing no crate had changed and the whole of Part 7 was a draft parked for Tom under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md). Tom accepted items A and B on 2026-08-06 and redirected item C; both landed in `tiler-ir`, and [Part 7.5](#75-what-landed-where-and-what-the-implementation-corrected) records what shipped and the five drift corrections building it established against this record's own Parts 2, 3, and 5. `implementation_status` moved to `partial` with them.
- **Normative destinations, neither edited here:** [Correctness and testing](../../correctness-and-testing.md) owns the oracle rule; [Numerical semantics](../../numerical-semantics.md) owns the dimension definitions. `contracts/numerics` was outside this ticket's scopes, and the catalog row is reported to the coordinator in the ticket's Outcome.
- **Accepted authorities consumed rather than re-derived:** [ADR 0011](../../decisions/0011-per-operation-numerical-permissions.md) (one permission never implies another), [ADR 0014](../../decisions/0014-reassociation-vs-permutation.md), [ADR 0015](../../decisions/0015-fma-vs-contraction.md), [ADR 0021](../../decisions/0021-validated-value-assumptions.md), [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md), [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md).
- **No literature acquisition failed for this record.** Every claim below rests on source in this repository, and every absence claim carries the exact one-line command that reproduces it.

## Outcome

**Fact — of the eleven canonical dimensions, exactly one is ever spent on the compile path, and it is spent at two kinds of site that are not the same kind of object.** `Reassociation` is gated live at the semantic rewrite (`crates/tiler-compiler/src/normalize.rs:749`) and at the two physical reduction-split guards (`crates/tiler-compiler/src/physical.rs:1794`, `:1996`). No other dimension has a live consumer. The exact check: `grep -rn "\.effective(" --include='*.rs' crates/` returns seven lines, six of them inside `policy.rs`'s `#[cfg(test)] mod tests` (which opens at `crates/tiler-compiler/src/policy.rs:736`), and the one live caller is `normalize.rs:749`, which passes `NumericalDimension::Reassociation`. Independently, `grep -rn "permits_contraction" --include='*.rs' crates/` returns two lines — the definition at `crates/tiler-ir/src/schedule/numerics.rs:292` and one term of a "does this realization grant any freedom at all" disjunction at `crates/tiler-ir/src/schedule/builder/elementwise.rs "|| numerical.permits_contraction()"` — so no rewrite or lowering reads it to enable a transform.

**Citation repair — 2026-08-19 by [`re-anchor-the-schedule-builder-line-citations`](../../../tickets/re-anchor-the-schedule-builder-line-citations.md), and the reproduction above no longer returns two lines.** `crates/tiler-ir/src/schedule/builder.rs` became the `crates/tiler-ir/src/schedule/builder/` directory, so the retired `:906` pin is replaced by a quoted anchor against the submodule that holds the disjunction — it is the strict-affine `u4` dequantize gate. Rerun at this base, `grep -rn "permits_contraction" crates/` returns **three** lines rather than two: the definition, now at `crates/tiler-ir/src/schedule/numerics.rs:319`, the disjunct anchored above, and `crates/tiler-ir/src/schedule/witness.rs:401`, where `RealizationWitness` reads the permission to classify a freedom site. The third reads it to *report*, not to enable a transform, so the sentence's conclusion is unchanged; its count is not.

**Fact — the ticket's premise about the counterexample is refuted in the form it was stated, and the correction moves the population between refusal classes rather than emptying it.** A three-leaf same-family `f32` chain does carry `ReductionTopology::None`, and `ReductionTopology` does have no grouping field. But `ReductionTopology` was never the site. The grouping is pinned, exactly and structurally, by `ScalarProgram::PointwiseF32(expression)`: `PointwiseF32Node::Add { lhs, rhs }` (`crates/tiler-ir/src/schedule/pointwise.rs:91`) is an explicit binary node over dense topological ordinals, and `mint_elementwise` (`crates/tiler-compiler/src/request/elementwise.rs "fn mint_elementwise"`) mints that tree as a faithful image of the — possibly reassociated — semantic DAG. So the site is **declared and unevaluable**, which is the parent derivation's refusal class 2 (`RealizationNotEvaluable`), not class 1 (`OrderNotPinned`). Part 4 works the case through.

**Fact — no reference evaluation path in the workspace can consume a plan-side witness today, and the reason is a deliberate dependency boundary rather than an omission.** `grep -rn "ScalarProgram\|ReductionTopology\|VerifiedScheduledRegion\|PointwiseF32" crates/tiler-reference/` returns exactly one line, and it is a doc comment in a test (`crates/tiler-reference/tests/index_region_oracle.rs:1414`). `grep -rn "tiler_ir::schedule" crates/tiler-reference/src --include='*.rs'` returns three lines, all of them `use` statements naming behaviour vocabularies (`SubnormalMode`, `NumericalPermission`, `NumericalRealization`, …) and none naming a plan structure. **Inference — a witness type is therefore not merely a new struct; it is a decision about which layer owns the aggregation**, and Part 7 states the surface that decision has to take.

**Fact — the parent derivation's first and most urgent gap is closed at this base, and the derivation does not know it.** Its Outcome states that the declared numerical conformance is applied at "exactly **three** sites in the workspace" and that its own check `grep -rn 'apply_to_operand\|apply_to_result' crates/ --include='*.rs'` returns fourteen lines in two files. Rerun at `c335bb5b`, that command returns **67 lines across ten files**, and `crates/tiler-reference/src/evaluate.rs` — the file the derivation says returns nothing for `ReferenceNumericalConformance` — now carries the type at seventeen lines including `ReferenceEvaluator::under`. Commit `f64956aa`, "End answering by omission in the reference layer", landed it, and the ticket the derivation filed for it (`apply-the-declared-numerical-conformance-on-every-reference-evaluation-path`) is `status: done`. Part 6 records this as a discrepancy against the parent rather than silently correcting it.

**Inference — the enumeration's headline number is twenty-four sites, and the useful split is five ways rather than two.** "Declared or not" hides the failure mode that actually bites: a field can be present, named, and *still not be a witness*, because it mirrors the contract rather than recording a choice. Five of the twenty-four are exactly that.

## Part 1 — What a freedom site is, and how the enumeration is generated

**Definition.** A **freedom site** is a pair — one canonical numerical dimension, and one construct a physical plan carries — at which a categorically granted permission, if spent, would change the observable bits of a program this build can compile. The pairing is the unit because neither half alone is one: a dimension is a permission with no location, and a construct is a location with no permission.

**Inference — a site is classified by two independent questions, and collapsing them is what produced the parent derivation's misclassification.** The first is *can a plan reach this site at all* — is there a construction path in `crates/tiler-compiler/src/` that builds the construct under a contract granting the dimension. The second is *does the plan record which choice it made there*, and that question has three answers rather than two: the plan may record the choice (a witness), may record the *contract's grant* and call it a record (a mirror), or may record nothing. A mirror is the dangerous middle: it type-checks, it appears in identity, it survives review, and it determines nothing.

The five classes the enumeration sorts into:

- **Witness** — the plan records the concrete choice, and a reference evaluator for that choice exists.
- **Witness, unevaluable** — the plan records the concrete choice, and no reference path can evaluate it.
- **Mirror** — a field exists and is named for the dimension, but it carries the contract's resolution rather than the plan's choice, so two plans making different choices agree on it.
- **Undeclared** — nothing in the plan names the choice.
- **Unspendable** — the dimension may be granted, but no construct a plan can reach spends it, so the site's population is empty by construction.

**Correction, 2026-08-18** (`carry-the-elementary-numerical-dimensions-in-the-region-realization`): the region IR now carries **ten** of the eleven dimensions — `NumericalRealization` gained required `reciprocal_transform` and `approximate_intrinsics` fields, `REALIZED_DIMENSIONS` names ten, and only `MaterializationRounding` stays outside the realization. The paragraph below is retained as the observation at its own base.

**Fact — the eleven-dimension vocabulary is `CANONICAL_DIMENSIONS` (`crates/tiler-ir/src/numerics.rs:149`), and the region IR carries eight of them.** `NumericalRealization` (`crates/tiler-ir/src/schedule/numerics.rs:208`) has `profile_key`, `canonical_arithmetic_nan_bits`, and the eight resolutions `input_subnormals`, `result_subnormals`, `contraction`, `reassociation`, `permutation`, `signed_zero`, `nan_assumptions`, `infinity_assumptions`. `REALIZED_DIMENSIONS` (`crates/tiler-compiler/src/policy.rs:119`) names exactly those eight. The three the realization cannot carry are `ReciprocalTransform`, `ApproximateIntrinsics`, and `MaterializationRounding`.

**Fact — the constructs a plan can carry are a closed set, and both halves are exhaustive enums inside their defining crate.** `ScalarProgram` (`crates/tiler-ir/src/schedule/model.rs:459`) has eight variants and is deliberately **not** `#[non_exhaustive]`. `ReductionTopology` (`crates/tiler-ir/src/schedule/model.rs:715`) has five variants. `KernelSchedule` (`:987`) carries the topology, the launch plan, the binding, and the tail policy; `IndexRegion` (`:628`) carries the scalar program and the realization; `ScheduledRegion` (`:1006`) is the pair.

**Fact — three `ScalarProgram` variants have no production construction site.** `PointwiseBf16`, `SquaredSerialSum`, `StrictSerialMaximum`, and `StrictAffineU4Dequantize` are constructed only under `#[cfg(test)]` or in test fixtures; the production sites are `crates/tiler-compiler/src/physical.rs:957` (`PointwiseF32`), `:1194` (`StrictTensorContraction`), `:1297`, `:1588`, `:1682`, `:1886` (`StrictSerialSum`), and `:1401` (`FusedMultiplyAddSerialSum`). Sites over the unconstructed variants are enumerated as **reserved** rather than omitted, because the type-level reservation and the reachable population are two different maturity claims.

## Part 2 — The enumeration

Twenty-four sites. The table is complete over the eleven dimensions, so nothing is answered by omission; the per-site evidence and reproducible check follow in Part 3.

| # | Dimension | Construct the freedom would be spent at | Class | Field, or the absence |
| --- | --- | --- | --- | --- |
| 1.1 | `InputSubnormals` | every arithmetic operand load in the emitted body | Witness | `IndexRegion.numerical.input_subnormals` |
| 2.1 | `ResultSubnormals` | every arithmetic result in the emitted body | Witness | `IndexRegion.numerical.result_subnormals` |
| 3.1 | `Contraction` | `FusedMultiplyAddSerialSum`'s `scale * x + bias` fold step | **Mirror** | `ScalarProgram::FusedMultiplyAddSerialSum.contraction` |
| 3.2 | `Contraction` | `StrictTensorContraction`'s `accumulator + a * b` step | **Undeclared** | no field on the variant |
| 3.3 | `Contraction` | the Metal backend compiler's own contraction | **Undeclared** | no target fact; flag dropped when permitted |
| 3.4 | `Contraction` | `SquaredSerialSum`'s `accumulator + x * x` step | **Undeclared** (reserved) | no field; no production construction site |
| 4.1 | `Reassociation` | `ReductionTopology::Serial` contributor fold | Witness | `axes` + `order`; no partition means the whole sequence |
| 4.2 | `Reassociation` | `ReductionTopology::MultiPass` split | Witness | `partition`, `order`, `accumulation`, `pass` |
| 4.3 | `Reassociation` | `ReductionTopology::CooperativeWorkgroup` tile | Witness (unevaluable when `rounds > 1`) | `partition`, `tile`, `order`, `accumulation`, `arrival` |
| 4.4 | `Reassociation` | `ReductionTopology::Contraction` contracted fold | Witness (empty spend population) | `contracted_shape` + `order` |
| 4.5 | `Reassociation` | `ScalarProgram::PointwiseF32` same-family chain | **Witness, unevaluable** | the expression DAG itself |
| 4.6 | `Reassociation` | `ScalarProgram::PointwiseBf16` same-family chain | Witness, unevaluable (reserved) | the expression DAG; no production site |
| 4.7 | `Reassociation` | which semantic candidate the portfolio selected | **Undeclared** | explain key string only; the program is not retained |
| 4.8 | `Reassociation` | accumulation width on 4.2 and 4.3 | **Witness, unevaluable** | `accumulation: ArithmeticType` |
| 5.1 | `Permutation` | `ContributorOrder` on any fold | **Unspendable** | one variant |
| 5.2 | `Permutation` | `ContributorArrival` on a cooperative tile | Mirror (empty population) | `arrival`; only one variant admitted |
| 5.3 | `Permutation` | `permits_permutation` on `Serial` topologies | **Mirror** | hard-coded `false` against a cross-check requiring equality |
| 6.1 | `SignedZero` | the comparison relation, not a result | **Unspendable** | no registered contract grants it; no consumer |
| 7.1 | `ReciprocalTransform` | `PointwiseF32Node::Divide` | **Unspendable** | no reciprocal node exists; withheld from every capability row |
| 8.1 | `ApproximateIntrinsics` | `PointwiseF32Node::Exp` | **Unspendable** | emits `precise::exp`; withheld from every row |
| 8.2 | `ApproximateIntrinsics` | `PointwiseF32Node::Rsqrt` | **Unspendable** | emits `precise::rsqrt`; withheld from every row |
| 9.1 | `NanAssumptions` | the plan's own input domain | Mirror (a precondition, not a result freedom) | `IndexRegion.numerical.nan_assumptions` |
| 10.1 | `InfinityAssumptions` | the plan's own input domain | Mirror (a precondition) | `IndexRegion.numerical.infinity_assumptions` |
| 11.1 | `MaterializationRounding` | any observable materialization boundary | **Unspendable** | one variant; outside `REALIZED_DIMENSIONS` |

**The split.** Twenty-four sites: **six** are witnesses a reference path can evaluate (1.1, 2.1, 4.1, 4.2, 4.3 at `rounds == 1`, 4.4); **three** are witnesses no reference path can evaluate (4.5, 4.6, 4.8); **five** are mirrors (3.1, 5.2, 5.3, 9.1, 10.1); **four** are undeclared (3.2, 3.3, 3.4, 4.7); **six** are unspendable by construction (5.1, 6.1, 7.1, 8.1, 8.2, 11.1).

**Three of those rows moved after this table was written, and [Part 7.5](#75-what-landed-where-and-what-the-implementation-corrected) carries the evidence for each.** Site 4.8 is a *witness* with an empty spend population rather than an unevaluable one, because the intrinsic verifier now refuses any accumulation width but the region's own; site 4.9 is a new site this table has no row for, at the fold epilogue a ninth `ScalarProgram` variant introduced; and site 3.1's mirror is a mirror for a different reason than Part 3.2 gives. The current split over the current vocabulary is twenty-five sites: **seven** evaluable witnesses (1.1, 2.1, 4.1, 4.2, 4.3 at `rounds == 1`, 4.4, 4.8) and **three** unevaluable ones (4.5, 4.6, 4.9), with the other three buckets unchanged.

**Inference — the number that matters for an oracle is not "declared" but "determining".** A witness set built from the six evaluable witnesses determines a plan's bits only for programs whose every other site is unspendable. That is a real and non-empty class — it is exactly the reduction-shaped case the corpus already handles — and it is not the class a permissive contract in general produces.

## Part 3 — Per site: the evidence and the check a reader can rerun

### 3.1 The two subnormal dimensions (sites 1.1, 2.1) — witnesses, and the one class the parent derivation named correctly

**Fact.** These are not freedoms at all; each names one function the reference applies, and `ReferenceNumericalConformance` (`crates/tiler-reference/src/conformance.rs:123`) realizes both exactly. `apply` (`conformance.rs:257`) is exhaustive over `SubnormalMode` and `FlushedZeroSign`, replaces exactly the values `f32::is_subnormal` names, and preserves the flushed value's own sign under `PreservesSign`.

**Fact — and the thread is now complete, which it was not when the parent derivation was written.** `ReferenceEvaluator` carries a conformance (`crates/tiler-reference/src/evaluate.rs:55`) with `ReferenceEvaluator::under` at `:90`; `strict_sum` takes one at `:454`; `strict_partial_sums_under` at `:603`; `strict_partitioned_sum_under` at `:766`; `IndexRegionEvaluator::under` at `crates/tiler-reference/src/oracle.rs:1353`; and the pattern reaches `rms_norm_f32_under`, `silu_f32_under`, `softmax_f32_under`, `certified_exp_f32_under`, and `certified_rsqrt_f32_under` (`crates/tiler-reference/src/lib.rs:73-95`).

**Check.** `grep -rn 'apply_to_operand\|apply_to_result' crates/ --include='*.rs' | wc -l` → `67`, across ten files. `grep -n 'ReferenceNumericalConformance' crates/tiler-reference/src/evaluate.rs` → seventeen lines.

### 3.2 Contraction (sites 3.1–3.4) — one mirror and three absences

**Fact — site 3.1's field is a mirror, and the source says so in the direction that matters.** `crates/tiler-compiler/src/physical.rs:1415-1416` sets it:

```rust
contraction: request.numerical_contract().contraction
    != NumericalPermission::Forbidden,
```

The comment above it (`physical.rs:1408-1414`) explains why it is contract-derived — hard-coding `false` would fail the schedule verifier's realization cross-check under a permitting contract — which is a statement about *verification*, not about what the kernel emitted. **Inference — so the field answers "was I allowed to fuse", never "did I fuse".** Two plans that disagree about whether the emitted body fused would carry the same value here.

**The stated ground for that is wrong at `61414b91`, and correcting it strengthens the conclusion rather than weakening it** — see [Part 7.5](#75-what-landed-where-and-what-the-implementation-corrected)'s correction 3. There is no realization cross-check on this field; the intrinsic verifier admits the variant only when the field is `false`, so no verified region carries `true` and the field does not even track the contract reliably.

**Fact — and nothing reads it.** `ScalarProgram::FusedMultiplyAddSerialSum` is destructured at `crates/tiler-ir/src/kernel/lower.rs:880-884` and again at `:1457-1460`, and both bindings take `scale_bits`, `bias_bits`, (`empty_identity_bits`,) and `..`. The `contraction` field reaches no lowering decision.

**Fact — site 3.2 has no field at all.** `ScalarProgram::StrictTensorContraction` (`crates/tiler-ir/src/schedule/model.rs:565-573`) carries exactly `contracted_shape`, `order`, and `canonical_nan_bits`. Its per-contributor step is `accumulator + a * b`, and `TENSOR_CONTRACTION` (`crates/tiler-compiler/src/policy.rs:258-267`) lists `Contraction` precisely because that adjacency is real. **Check:** read `model.rs:565-573`; there are three fields and none names contraction.

**Fact — site 3.3 is the parent derivation's refusal class 3, and it is confirmed at source here.** `realization_requirements` (`crates/tiler-metal/src/emit.rs:1023`) inserts `MetalNumericalRequirement::NoFloatingPointContraction` — which renders `-ffp-contract=off` — **only** in the `Forbidden` arm:

```rust
match realization.contraction {
    NumericalPermission::Forbidden => {
        requirements.insert(MetalNumericalRequirement::NoFloatingPointContraction);
    }
    NumericalPermission::Permitted => {}
}
```

**Inference — so under a contraction-permitting contract Tiler drops the one thing that supplied its own pin, and the executed order becomes a property of a backend compiler no target profile declares.** The function's own doc states the general rule it is following — "A permission the realization *grants* names no flag" — which is correct for a *compiler-selection* set and is exactly why it cannot double as a witness.

**Fact — a permitting contract also invalidates the strict-f32 fusion proof rather than licensing a rewrite.** `crates/tiler-compiler/src/fusion.rs:178-184` returns `invalid(candidate, "strict-f32-proof")` when the contract's contraction or reassociation is anything but `Forbidden`. So contraction's only live effect on the compile path is a refusal.

**Fact — site 3.4 is reserved.** `SquaredSerialSum` carries no contraction field (`model.rs:539-548`), and `NORMALIZATION` (`policy.rs:295-304`) lists `Contraction` for `tiler::rms-norm-f32@1` with the adjacency argument stated at `policy.rs:283-291`. No production site constructs the variant.

### 3.3 Reassociation (sites 4.1–4.8) — the only dimension with a live spend

**Fact — the semantic gate is one site and it is exact.** `crates/tiler-compiler/src/normalize.rs:749-754` requires

```rust
capability.effective(
    crate::target::honourability::NumericalDimension::Reassociation,
    contract,
) != Some(DimensionBehaviour::Transform(NumericalPermission::Permitted))
```

to be false before the rule fires, declining with `"numerical.reassociation-forbidden"` (`:762`) and accepting with `"numerical.reassociation-permitted"` (`:770`). `effective` (`crates/tiler-compiler/src/policy.rs:176-183`) is `self.can_consume(dimension).then(|| ceiling.behaviour(dimension))` — an intersection of the operation's capability row with the resolved contract, exactly as ADR 0011's program-ceiling rule requires. `rebuild_ordered_reassociation` (`normalize.rs:993`) then rebuilds the `SemanticProgram` with `(a op b) op c` replaced by `a op (b op c)` at the first matching site.

**Fact — the physical gates are the other two.** `crates/tiler-compiler/src/physical.rs:1794` returns `WorkgroupTreeUnavailable::ReassociationForbidden` and `:1996` returns `SplitUnavailable::ReassociationForbidden` when the contract forbids it.

**Fact — sites 4.1 through 4.4 are witnesses with named fields, and the cross-check that keeps them honest is code rather than a doc claim.** `crates/tiler-ir/src/schedule/builder.rs` compares the topology's `permits_reassociation` and `permits_permutation` against the region's own `numerical` at `:496-497`, `:987-988`, `:1018-1019`, `:1054-1055`, `:1091-1092`, `:1414-1415`, and `:1544-1545`, and refuses an arrival that requires permutation the realization does not grant at `:1558`. Construction: `Serial` at `physical.rs:1306` and `:1421`, `Contraction` at `:1202`, `MultiPass` via `multi_pass_topology` at `:2066` (called from `:1597` and `:1691`), `CooperativeWorkgroup` at `:1896`.

**Fact — site 4.4's spend population is empty by the variant's own contract.** `ReductionTopology::Contraction`'s `permits_reassociation` doc (`model.rs:800-806`) states it is "recorded and cross-checked against the region's declared realization, and deliberately not consulted to admit the topology: this fold is the declared contributor sequence itself, so it consumes no reassociation." The variant carries no partition, so there is no regrouping to record.

**Fact — site 4.5 is the load-bearing correction, and the grouping is pinned.** `PointwiseF32Expression` (`crates/tiler-ir/src/schedule/pointwise.rs:150`) is `nodes: Box<[PointwiseF32Node]>` plus an explicit `root`; `PointwiseF32Node::Add { lhs, rhs }` (`:91`) and `Multiply { lhs, rhs }` (`:98`) are binary nodes over dense topological ordinals, and `is_valid` (`:181`) requires every operand ordinal to precede its consumer. The expression *is* a grouping. `mint_elementwise` (`crates/tiler-compiler/src/request/elementwise.rs "fn mint_elementwise"`) builds it by replaying `ElementwiseMint::Node(family, operands)` steps recorded by `plan_elementwise` (`:4555`) from the semantic DAG, so a reassociated candidate mints a differently shaped tree. The source states this itself at `request.rs:4641-4645`: "reassociating `(a * 2.0) * 3.0` into `a * (2.0 * 3.0)` is an alternative the algebraic exploration proposes under a contract that permits it, and its inner product is rank zero."

**Fact — and site 4.5 is unevaluable.** No reference path can evaluate a `PointwiseF32Expression`. **Check:** `grep -rn "ScalarProgram\|ReductionTopology\|VerifiedScheduledRegion\|PointwiseF32" crates/tiler-reference/` returns one line, a doc comment at `crates/tiler-reference/tests/index_region_oracle.rs:1414`.

**Fact — site 4.7 is undeclared, and the check is that no accessor exists.** The rewritten `SemanticProgram` lives only in `SemanticCandidate.proposal` (`crates/tiler-compiler/src/pipeline.rs:328-333`), a private pipeline struct dropped after compilation. The retained `ProgramAlternative` (`pipeline.rs:247-271`) has no semantic-program field, and the artifact keeps only the identity digest (`crates/tiler-artifact/src/program/builder.rs:240`). What survives is the candidate *key string* — `"semantic:baseline"` or `"semantic:p{..}:{provider}.r{..}:{rule}@{revision}"` (`pipeline.rs:739-748`) — reachable through explain (`selected_candidate_key`, `crates/tiler-compiler/src/explain.rs:2172`). **Check:** `grep -rn "pub fn .*SemanticProgram" crates/tiler-compiler/src/` returns nothing.

**Inference — that absence is what makes 4.5 unevaluable rather than merely inconvenient.** The reference *does* have an exact evaluator for a `SemanticProgram` (`ReferenceEvaluator::evaluate`, `crates/tiler-reference/src/evaluate.rs:178`). Had the selected candidate's rewritten program been retained, site 4.5's witness would be "which candidate", the existing semantic evaluator would answer it exactly, and no new evaluator would be needed. The gap is a retention gap, not an evaluator gap, and that distinction changes which repair is correct.

**Fact — site 4.8 is a declared width no oracle honours.** `accumulation: ArithmeticType` is set from `request.numerical_contract().arithmetic` at `physical.rs:1901` and `:2071`; its doc (`model.rs:755-761`) states it is carried explicitly because "a strategy that accumulated at a narrower width than the contract admits is a different computation". `strict_partial_sums_under` has no width parameter, so it can only answer for accumulation at the element type.

**Corrected at `61414b91`: the site's spend population is empty, so the reference answers for every width a verified region can declare** — see [Part 7.5](#75-what-landed-where-and-what-the-implementation-corrected)'s correction 2. The intrinsic verifier refuses a declared accumulation that differs from the region's own arithmetic type at both parallel admissions, so the class moves from *Witness, unevaluable* to *Witness* with an empty spend population, beside 4.4.

**Fact — site 4.3 is unevaluable when the tile is loop-carried.** `strict_partial_sums_under`'s index arithmetic is `partition * chunk + within` (`evaluate.rs:688`), a flat blocked split. `CooperativeWorkgroup`'s doc (`model.rs:844-849`) states that on a multi-round tile "participant `p` of round `r` owns the contiguous range at index `r * partitions + p`" — a different order the flat expression cannot state. The compiler's only tile constructor is `workgroup_tree_tile` (`physical.rs:1818`), whose body fixes `rounds: 1` (`crates/tiler-ir/src/schedule/cooperative.rs:887`), while `tiler-ir`'s own fixtures construct and verify a `rounds: 2` tile.

### 3.4 Permutation (sites 5.1–5.3) — granted and unspendable, with one mirror worth naming

**Fact.** `ContributorOrder` (`crates/tiler-ir/src/schedule/model.rs:101-104`) has exactly one variant, `OriginalAxisLexicographic`. `ContributorArrival` (`crates/tiler-ir/src/schedule/cooperative.rs:136-159`) has three, of which only `AscendingParticipant` is admitted, and `requires_permutation()` (`:169-174`) returns `false` for it. Every production construction site writes `ContributorOrder::OriginalAxisLexicographic` as a literal, and `physical.rs:1910` writes `ContributorArrival::AscendingParticipant` as a literal.

**Fact — site 5.3 is a mirror whose asymmetry is documented and load-bearing.** `physical.rs:1311` and `:1430` hard-code `permits_permutation: false` on the two `Serial` topologies, while `multi_pass_topology` (`:2074-2075`) and the cooperative site (`:1908-1909`) derive theirs from the contract. The builder cross-check requires equality (`builder.rs:497`), so under a permutation-permitting contract the two hard-coded sites would produce regions the verifier refuses. **Inference — this is unreachable today and correctly so**: no registered contract grants permutation, and `unrepresentable_dimension` would refuse one that tried. It is enumerated because a plan-side witness that read `permits_permutation` would be reading a field two construction sites do not compute.

### 3.5 Signed zero, reciprocal transform, approximate intrinsics, materialization rounding (sites 6.1, 7.1, 8.1, 8.2, 11.1) — unspendable, and three of them for a reason the parent derivation does not state

**Fact — `SignedZero` has no live consumer.** `permits_signed_zero_elimination` (`crates/tiler-ir/src/schedule/numerics.rs:310`) is read at exactly one site repo-wide, `crates/tiler-ir/src/schedule/builder/elementwise.rs "|| numerical.permits_signed_zero_elimination()"`, as one disjunct of a "does this realization grant any freedom at all" classification. `realized_behaviour` (`crates/tiler-compiler/src/policy.rs:568`) pins it to `Transform(Forbidden)` and the module states the reason at `policy.rs:565`: "no rewrite in this build consumes a signed-zero or infinity assumption."

**Citation repair — 2026-08-19 by [`re-anchor-the-schedule-builder-line-citations`](../../../tickets/re-anchor-the-schedule-builder-line-citations.md), and "exactly one site repo-wide" is now false.** `crates/tiler-ir/src/schedule/builder.rs` became the `crates/tiler-ir/src/schedule/builder/` directory, so the retired `:909` pin is replaced by a quoted anchor against the disjunct it named. Rerun at this base, `grep -rn "permits_signed_zero_elimination" crates/` returns the definition, now at `crates/tiler-ir/src/schedule/numerics.rs:337`, and **two** read sites rather than one: the disjunct anchored above, and `crates/tiler-ir/src/schedule/builder/coverage.rs "if numerical.permits_signed_zero_elimination() {"`, where the padded-coverage gate widens `f32` equality to treat `-0.0` and `+0.0` as observably equal while deriving a padding identity's two-sided neutrality. That second site is a live consumer of the permission, so the heading's claim holds only in the sense the paragraph's last sentence gives it — no *rewrite* consumes it — and the classification of `SignedZero` as unspendable is not re-derived here.

**Fact — `ReciprocalTransform` and `ApproximateIntrinsics` are withheld from every operation capability row, deliberately and by a named constant.** `ELEMENTARY_UNCARRIED_DIMENSIONS` (`crates/tiler-compiler/src/policy.rs:476-479`) names both. Its doc (`:445-475`) states that all three admitted elementary families — `tiler::silu-f32@1`, `tiler::rms-norm-f32@1`, `tiler::softmax-f32@1` — have a real obligation on each, and that listing them would enter each into `is_consumable`'s union, which would make `unrepresentable_dimension` refuse the public `RELAXED_F32` preset for every program, because `NumericalRealization` carries neither.

**Fact — `RELAXED_F32` authorizes both and nothing spends either.** `crates/tiler-compiler/src/session.rs:1454-1458` sets `reciprocal_transform(Permitted)` and `approximate_intrinsics(BackendElementary)`. `crates/tiler-compiler/src/policy.rs:99-101` says of the envelope: "Nothing in this build emits an approximate intrinsic, so this names an envelope that is authorized and unconsumed." `ApproximationEnvelope::BackendElementary`'s own doc (`crates/tiler-ir/src/schedule/numerics.rs:501-508`) says it is "**Not reachable** for either operation that could consume it."

**Fact — and the vocabulary makes the reciprocal substitution unstatable rather than merely forbidden.** `PointwiseF32Node::Divide`'s doc (`crates/tiler-ir/src/schedule/pointwise.rs:104-110`) records that "this vocabulary has no reciprocal node at all, which is what makes the substitution unstatable here rather than merely forbidden", and there is no `Sqrt` beside `Rsqrt` for the same reason (`:128-135`). The Metal emission writes `precise::exp` (`crates/tiler-metal/src/emit.rs:1784`) and `precise::rsqrt` (`:1795`), and a test asserts the emitted source contains none of `fast::exp`, `fast_exp`, `metal::divide(`, `1.0f /` (`crates/tiler-metal/src/tests.rs:1819`).

**Fact — `MaterializationRounding` has one variant** (`crates/tiler-ir/src/schedule/numerics.rs:566-569`) and is outside `REALIZED_DIMENSIONS`, so it admits no divergence.

**Inference — the parent derivation's Part 3 marks `ReciprocalTransform` and `ApproximateIntrinsics` "Yes via `RELAXED_F32`", and the granted-but-unspendable structure it correctly identifies for `Permutation` applies to both.** There are three such dimensions, not one, and the mechanism differs: permutation is unspendable because its vocabulary has one admitted value, while these two are unspendable because the capability table withholds them to keep a preset representable.

### 3.6 Exceptional-value assumptions (sites 9.1, 10.1) — preconditions, correctly classified by the parent

**Fact.** `ExceptionalValueAssumption::AssumeAbsent { provenance }` constrains inputs, and `ReferenceNumericalConformance::from_realization` refuses it by name (`crates/tiler-reference/src/conformance.rs:205-216`) with `NanAbsenceAssumed` / `InfinityAbsenceAssumed`. No registered contract grants either. The parent derivation's narrowing — that a `RuntimeValidated` provenance is answerable over a pinned realization's own values while `CompilerProven` remains the compiler's proof to exhibit — is unchanged by anything here.

## Part 4 — The counterexample, verified at source

**The ticket's claim, restated exactly:** a one-input, one-output, three-leaf same-family `f32` chain through the `PointwiseF32Expression` projection carries `ReductionTopology::None` and no grouping field, giving `OrderNotPinned` a non-empty population.

**Fact — the first half holds.** `linear_schedule` (`crates/tiler-compiler/src/physical.rs:2079-2093`) sets `reduction: ReductionTopology::None`, and every non-reducing region inherits it through `..linear_schedule(..)`; `pointwise_region` (`:810`) is such a region. `ReductionTopology::None` (`crates/tiler-ir/src/schedule/model.rs:717`) is a unit variant with no fields, so it names no grouping.

**Fact — the second half does not.** The plan does name the grouping, on the other half of the same `ScheduledRegion`. Walk the case:

1. The caller writes `add(add(a, k1), k2)` under `REASSOCIATE_F32` or `RELAXED_F32` (`crates/tiler-compiler/src/session.rs:1472`, `:1454`).
2. `OrderedReassociationRule::add()` (`crates/tiler-compiler/src/normalize.rs:685`) finds the left-associated site, passes the semantic gate (`declares_ordered_associativity`, `:711`), the capability gate (`:724`), and the permission gate (`:749`), and `rebuild_ordered_reassociation` (`:993`) emits `add(a, add(k1, k2))` as a sibling `SemanticProgram`.
3. `compile_contract_group` (`crates/tiler-compiler/src/pipeline.rs:695`) pushes the baseline at weight 0 and the alternative beside it (`:768-770`); each is independently readmitted (`:724`) and recognized.
4. For the rewritten candidate, `plan_elementwise` (`crates/tiler-compiler/src/request/elementwise.rs "fn plan_elementwise"`) walks the *rewritten* DAG and `mint_elementwise` (`:4697`) mints `Add { lhs: Input{0}, rhs: Add { lhs: Constant{k1}, rhs: Constant{k2} } }` — a different node vector from the baseline's `Add { lhs: Add { lhs: Input{0}, rhs: Constant{k1} }, rhs: Constant{k2} }`.
5. `elementwise_region` (`physical.rs:888`) stores it at `physical.rs:957` as `ScalarProgram::PointwiseF32(expression)`.

**So the two plans differ in a plan-carried field**, and the field is the scalar program rather than the reduction topology. `PointwiseF32Expression` derives `Eq` (`crates/tiler-ir/src/schedule/pointwise.rs:149`), and the two node vectors are unequal, so the difference is decidable by a reader with the plan in hand.

**Inference — the site's correct refusal class is `RealizationNotEvaluable`, and the population is non-empty for that class instead.** The oracle cannot evaluate the pinned expression: no reference path accepts one (Part 3.3's check), and the rewritten `SemanticProgram` the semantic evaluator *could* have evaluated exactly is not retained (Part 3.3's second check). The verification result is therefore: **the ticket's counterexample is real as a gap and misattributed as a class.** `OrderNotPinned`'s population over `PointwiseF32` regions is empty at this base; `RealizationNotEvaluable`'s is not.

**Inference — one honest residue keeps `OrderNotPinned` from being empty outright.** Site 3.2 (`StrictTensorContraction`'s fused step) and site 3.3 (the backend compiler's contraction) are genuinely undeclared, and site 4.7 is undeclared for the semantic-candidate choice. Those are `OrderNotPinned` populations under a contraction-permitting contract. What this record removes is the *pointwise reassociation* instance the ticket named, not the class.

**Measurement boundary.** This walk is a source reading, not a run. It is `sound-proof` within the stated model: the node vectors are unequal by construction because `mint_elementwise` replays a different `plan.steps` sequence, and no host arithmetic was performed to establish it. A run that compiles both candidates and compares the two `ScalarProgram::PointwiseF32` payloads would upgrade it to a bounded measurement, and is not performed here.

## Part 5 — What a witness must determine, stated so it can be refuted

**Proposal — the determination property.** Let `P` be a verified semantic program, `C` a resolved numerical contract, and `K` a compiled plan. A realization witness `W(K)` is **determining** exactly when: for any two plans `K₁` and `K₂` compiled from the same `P` under the same `C`, if `W(K₁) = W(K₂)` then a correct execution of `K₁` and a correct execution of `K₂` produce identical bits on every output for every admitted input.

**The refutation procedure, stated so a reader can run it rather than only doubt it.** To refute the enumeration, exhibit two plans this build can compile from one `P` under one `C` that agree on every field the enumeration marks a witness, and disagree in bits. Concretely: choose a program and a contract, compile two candidates, compare the twenty-four sites' fields pairwise, and compare the outputs. A disagreement in bits with agreement on the witness set names a freedom site the table missed, and the table is wrong at exactly that site.

**Inference — the enumeration predicts three specific refutations, and predicting them is the strongest evidence it is complete rather than convenient.**

1. **Under a contraction-permitting contract, over a `StrictTensorContraction`.** Two plans agreeing on `contracted_shape`, `order`, and `canonical_nan_bits` — the variant's entire field set — can differ in whether the emitted `accumulator + a * b` fuses, and a fused step rounds once where a separate one rounds twice. Site 3.2 predicts this and no witness field distinguishes the pair.
2. **Under a contraction-permitting contract, over any region at all.** Two identical plans compiled by two backend compilers, or by one compiler at two optimization selections, can differ, because `-ffp-contract=off` is dropped exactly when the contract permits (`crates/tiler-metal/src/emit.rs:1027-1032`). Site 3.3 predicts this, and it is the one refutation whose resolution needs a device measurement rather than a field.
3. **Under a reassociation-permitting contract, over a `PointwiseF32` chain, if the witness is taken to be `ReductionTopology` alone.** The baseline and the rewritten candidate both carry `ReductionTopology::None` and differ in bits. Site 4.5 predicts this, and it is the reason the witness must include the scalar program rather than only the topology — which is precisely the correction Part 4 makes.

**Inference — the converse also has to hold or the witness is useless, and it is the weaker claim.** A determining witness must not be so fine that two plans producing identical bits disagree on it. The expression DAG is fine in exactly this way: two semantically identical expressions that differ only in the order the builder minted independent subtrees would be different node vectors. `PointwiseF32ExpressionBuilder` mitigates this by construction — one leaf per input ordinal, shared on repeat request (`crates/tiler-ir/src/schedule/pointwise.rs:270-277`, `:286-296`), and a deterministic root-first-derived topological order (`:145-148`) — so the canonical form is a function of the program rather than of the spelling. **That mitigation is a claim this record does not test**, and it is stated here as the thing to check before a witness is built on it, not as an established fact.

**Tested since, and refuted in the general form stated above.** [Part 7.5](#75-what-landed-where-and-what-the-implementation-corrected) carries the two tests and their verdict: the claim holds for exactly the two mitigations named here, and fails for a third spelling neither reaches, because nothing shares an identical *constant*. The failure is of the converse rather than of the determination property — the witness is too fine, so a caller may see two witnesses differ when the bits do not, never see them agree when the bits differ — and the paragraph above is preserved as the reasoning that made the check a precondition.

## Part 6 — Discrepancies found against the parent derivation

Each was checked at `c335bb5b` by rerunning the derivation's own stated command or by reading the cited line.

| Parent's claim | Where | Status at this base |
| --- | --- | --- |
| `ReductionTopology` at `model.rs:563` | Part 2.4, and the ticket | **Line drift.** The enum is at `crates/tiler-ir/src/schedule/model.rs:715`. Line 563 is inside `StrictTensorContraction`'s doc comment. |
| `ContributorOrder` at `model.rs:92` | Part 3 | **Line drift.** It is at `model.rs:101`. |
| Cross-checks at `builder.rs:416`, `:764`, `:795` | Part 2.4 | **Line drift, and an undercount.** The comparison appears at `builder.rs:496-497`, `:987-988`, `:1018-1019`, `:1054-1055`, `:1091-1092`, `:1414-1415`, `:1544-1545` — seven sites, not three, plus the arrival check at `:1558`. |
| Conformance applied at "exactly three sites"; `grep` returns fourteen lines in two files | Outcome | **Closed and superseded.** The same command returns 67 lines across ten files. `ReferenceEvaluator` carries a conformance (`evaluate.rs:55`, `:90`). Landed by `f64956aa`; the filed ticket is `status: done`. |
| "The semantic evaluator applies it nowhere" | Outcome | **False at this base.** `grep -n 'ReferenceNumericalConformance' crates/tiler-reference/src/evaluate.rs` returns seventeen lines. |
| "Neither takes a conformance" (of `strict_partitioned_sum` and the semantic evaluator) | Outcome | **False at this base.** `strict_partitioned_sum_under` (`evaluate.rs:766`) and `strict_partial_sums_under` (`:603`) both do, and the `_under` pattern reaches nine public entry points (`lib.rs:73-95`). |
| `ApproximateIntrinsics` reachable "Yes via `RELAXED_F32`" | Part 3 table | **Overstated.** Authorized and unconsumed; withheld from every capability row by `ELEMENTARY_UNCARRIED_DIMENSIONS` (`policy.rs:476`), and `ApproximationEnvelope::BackendElementary` documents itself "not reachable" (`numerics.rs:501`). |
| `ReciprocalTransform` reachable "Yes via `RELAXED_F32`" | Part 3 table | **Overstated, same mechanism.** Additionally unstatable: the pointwise vocabulary has no reciprocal node (`pointwise.rs:104-110`) and `ElementaryPointSink` has no reciprocal step (`elementary.rs:116-119`). |
| `Contraction` reachable "Yes via `RELAXED_F32`"; "the emitted kernel pins it" | Part 3 table | **The first half holds; the second does not.** The only field is a contract mirror (`physical.rs:1415`) and no lowering reads it (`kernel/lower.rs:880-884`, `:1457-1460`). The emitted kernel pins nothing a plan records. |
| `OrderNotPinned`'s population is the `PointwiseF32` chain | Part 4 class 1, Part 7 gap 2 | **Reclassified.** The chain's grouping *is* pinned, by the expression DAG; the site belongs to `RealizationNotEvaluable`. Part 4 above. |
| "the two oracles actually used … both silently ignore the two subnormal dimensions" | Outcome | **No longer true.** Both now take a conformance; `strict_partial_sums_under`'s doc (`evaluate.rs:580-598`) states the independence of the two obligations and gives a discriminating operand set. |
| Contracts are "a five-preset enumeration" corner | Part 6 | **Seven registered presets at this base**, not five: `STRICT_F32`, `FLUSH_SUBNORMALS_TO_ZERO_F32`, `RELAXED_F32`, `REASSOCIATE_F32`, `FLUSH_AND_REASSOCIATE_F32`, `STRICT_BF16`, `FLUSH_SUBNORMALS_TO_ZERO_BF16` (`session.rs:1427-1523`). Three of them permit reassociation, and the derivation names two. |

**Inference — none of these overturns the parent's elimination, and saying so precisely is the point.** O5 survives as the admission authority for exactly the reasons Part 2.3 of that record gives; every correction here is to a *population* or a *location*, not to the argument that a widened comparison admits results the contract does not permit. The one correction with design consequence is the `PointwiseF32` reclassification, because it changes which repair is right: a retention or evaluator change rather than a new plan-side grouping field.

**Inference — and that consequence lands directly on the parent's non-goal.** The ticket explicitly deferred "whether the `PointwiseF32Expression` projection should carry a grouping field" as a consequence of the enumeration. The enumeration's answer is that it should not: the projection already carries the grouping, exactly, and adding a field would be a second spelling of a fact the node vector already determines — which is the duplication the `ContributorOrder` and `ContributorArrival` vocabularies were each designed to avoid.

## Part 7 — The public surface, drafted for Tom and not adopted

**Everything in this Part was a Proposal when it was written, and items A and B no longer are.** Under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md) a witness type, any change to `ReferenceNumericalConformance`'s construction, and any new plan-side field are each a public boundary. This record designed the surface and stated its evidence; acceptance was Tom's. He accepted A and B on 2026-08-06 and redirected C to the plain-scalar alternative drafted beside it, and both accepted items have since shipped — read [Part 7.5](#75-what-landed-where-and-what-the-implementation-corrected) before treating 7.2's drafts as the current surface, because building them corrected five things this record states.

### 7.1 The constraint the surface has to satisfy, derived rather than asserted

**Fact — `tiler-reference` depends on `tiler-ir` and nothing else** (`crates/tiler-reference/Cargo.toml`, one workspace dependency). **Fact — `tiler-compiler` depends on `tiler-reference` only as a dev-dependency**, so no production compiler code can call the reference. **Inference — the aggregation function (plan → witness) and the consumption function (witness → value) therefore cannot live in the same crate unless that crate is `tiler-ir`**, and `tiler-ir::schedule` is the only place both sides can see.

**Inference — the existing precedent already resolves this and should not be broken.** `strict_partitioned_sum_under` takes `(input, axes, partitions, contributors_per_partition, conformance)` — plain scalars, not a schedule type. That keeps the reference independent of the plan vocabulary, which is the property `crates/tiler-reference/src/oracle.rs`'s module header names first: the path "is deliberately independent of any graph-specific host expression."

### 7.2 The drafted surface

Three items. Each is stated with what it enables and what it prevents.

**Item A — a witness type, sited in `tiler_ir::schedule`.**

```rust
/// The concrete realization one verified scheduled region pinned at every site
/// its contract left free.
///
/// Aggregated from the region rather than declared beside it: every field below
/// is read from a construct the region already carries, so a witness cannot
/// disagree with the plan it describes.
pub struct RealizationWitness { /* private */ }

impl RealizationWitness {
    /// Aggregates the witness one verified region determines.
    pub fn of(region: &VerifiedScheduledRegion) -> Self;

    /// The declared contributor split, for a topology that states one.
    pub fn contributor_partition(&self) -> Option<ContributorPartition>;

    /// The width every combining step is performed at.
    pub fn accumulation(&self) -> ArithmeticType;

    /// The contributor combination order.
    pub fn order(&self) -> ContributorOrder;

    /// The staged-partial arrival, for a cooperative tile.
    pub fn arrival(&self) -> Option<ContributorArrival>;

    /// The rounds a cooperative tile executes, for a tile that states one.
    pub fn rounds(&self) -> Option<u64>;

    /// The pinned per-point expression, when the region's scalar program is one.
    pub fn pointwise_f32(&self) -> Option<&PointwiseF32Expression>;

    /// The declared numerical realization the witness was aggregated under.
    pub fn realization(&self) -> &NumericalRealization;
}
```

*Enables:* one object a caller passes to an oracle, replacing the per-test hand aggregation the two precedents perform. *Prevents:* it does not make the pointwise expression evaluable, and it does not close sites 3.2, 3.3, or 4.7 — those are absences a witness type cannot invent.

**Item B — a refusal enum stating what the witness does not determine.**

```rust
/// A freedom site the plan grants and the witness does not pin.
#[non_exhaustive]
pub enum UnpinnedFreedomSite {
    /// The contract permits contraction and the region's fold has an
    /// adjacent multiply the plan records no choice for.
    ContractionUnrecorded { operation: /* … */ },
    /// The contract permits contraction and no target fact declares whether
    /// the backend compiler preserves the emitted order.
    BackendOrderUndeclared,
    /// The witness pins an order no reference evaluator implements.
    RealizationNotEvaluable { reason: /* … */ },
}
```

*Enables:* a fail-closed refusal naming the site, which is what separates "this plan is unqualifiable" from "this plan is wrong". *Prevents:* it must never gain a `Conforms`-shaped arm; the parent derivation's Part 4 argument about typing a refutation-only oracle applies here unchanged.

**Item C — no change to `ReferenceNumericalConformance`'s construction, and the derivation for that is the finding.**

The parent derivation proposed that `from_realization`'s refusal is "not the defect" but that a *third argument* was missing. At this base the third argument's consumers already exist in the shape the tree prefers: the `_under` family takes the conformance beside the plain-scalar order arguments. **Proposal — the correct surface is therefore an additional constructor rather than a relaxed one:**

```rust
impl ReferenceNumericalConformance {
    /// Derives the conformance from a realization *and* the witness that pins
    /// the freedoms the realization alone leaves open.
    ///
    /// # Errors
    ///
    /// Returns [`UnpinnedFreedomSite`] naming the first site the witness does
    /// not determine. It never returns the strict reading for a permissive
    /// contract, which is the refusal `from_realization` exists to make.
    pub fn from_witness(
        realization: &NumericalRealization,
        witness: &RealizationWitness,
    ) -> Result<Self, UnpinnedFreedomSite>;
}
```

*Enables:* a permissive contract becomes evaluable at exactly the sites the witness pins. *Prevents:* `from_realization` keeps its current signature and its current refusals verbatim; a caller holding only a realization still cannot obtain a conformance for a permissive contract, which is the property the module header (`crates/tiler-reference/src/conformance.rs:19-36`) is written around.

**But Item C carries a dependency consequence Tom should decide explicitly:** `from_witness` takes a `RealizationWitness`, which under Item A is a `tiler_ir::schedule` type built from a `VerifiedScheduledRegion`. That makes `tiler-reference` name a plan structure for the first time. The alternative that avoids it is to keep the reference's arguments as plain scalars — as `strict_partitioned_sum_under` already does — and site the *aggregation* alone in `tiler_ir::schedule`, so `RealizationWitness` exists but the reference never sees it and callers destructure it at the call. That is cheaper on dependency direction and more verbose at every call site, and it cannot carry the pointwise expression at all.

### 7.3 What is not drafted, and why

**No new plan-side field is proposed.** The enumeration's answer to the ticket's deferred question is that `PointwiseF32Expression` already pins its grouping, so a grouping field would duplicate it. The three genuinely undeclared sites (3.2, 3.3, 4.7) each want a *different* repair — a field on `StrictTensorContraction`, a target fact plus a measurement, and a retention decision about the selected semantic candidate — and none of them is a grouping field.

**No evaluator is proposed for `PointwiseF32Expression`, and Part 7.4 states why none should be.** Part 3.3's finding is that the gap is a retention gap: the reference already evaluates a `SemanticProgram` exactly, and the rewritten one is discarded. This record parked the fork between retaining the selected candidate's program and writing a second exact evaluator for the physical projection, holding that neither was correctness-dominant. **That parking is superseded by Part 7.4**, which runs the elimination and finds one survivor.

### 7.4 The evaluability fork, resolved: retain the selected candidate's program

**Added by [`decide-how-a-pinned-pointwise-grouping-becomes-evaluable`](../../../tickets/decide-how-a-pinned-pointwise-grouping-becomes-evaluable.md) at base `9466f622`**, whose Outcome carries the full derivation with its line-level evidence. This section states the result and the two grounds that decide it, so a reader of the record does not have to reconstruct them from the ticket.

**Fact — this record's own framing of the fork inverted the coverage relation, and correcting it decides the fork.** Write `P'` for the selected semantic candidate, `E` for the expression `mint_elementwise` produced from it, and `z` for the candidate's observed output bits. The projection sits *between* `P'` and `z`. Retaining the program compares `z` against an exact evaluation of `P'` — across the projection, the lowering, the emission, and the device. An evaluator over `E` compares `z` against a value derived from `E` — across the lowering, the emission, and the device only. The projection defect this record described as the second design's advantage is in fact visible only to the first: an oracle whose reference value is read out of the artifact under test cannot refuse a defect introduced at or before that artifact. This is the elimination [the parent derivation](permitted-divergence-oracle.md) Part 2.3 already ran against O4, one step further along — an evaluator over `E` admits an expression that is not a legal grouping of `P'` at all.

**Fact — the projection is unguarded, so the coverage matters.** No workspace check cross-checks the minted expression against the semantic DAG: `PointwiseF32Expression`'s only non-test consumers are a structural verifier (`crates/tiler-ir/src/schedule/builder/elementwise.rs "pub(super) fn verify_pointwise_f32("`, which hands the expression's `input_count` and `is_valid` to the shared pointwise gate) and the kernel lowering (`crates/tiler-ir/src/kernel/lower.rs:1632`); the request subject's copy shares one `mint_elementwise` ancestor with the region's; `IndexRealizationLaw` re-derives per *occurrence* index regions and never the fused scalar program; and `prove_fused_numerics` refuses outright whenever reassociation is permitted (`crates/tiler-compiler/src/fusion.rs:178-184`). The class has fired once: `LeafRead`'s doc (`crates/tiler-compiler/src/request/tests.rs "permute(a) * permute(a)"`) records `a * permute(a)` having compiled as `permute(a) * permute(a)`.

**Fact — and an evaluator over `E` could not be exact on its own population anyway.** `PointwiseF32Node::Exp` (`crates/tiler-ir/src/schedule/pointwise.rs:117-124`) and `Rsqrt` (`:128-136`) document that their admitted result sets come from "the registered accuracy contract of the operation this expression realizes, never from IEEE-754 alone", and `ElementwiseFamily::Silu` mints exactly those nodes into a production `PointwiseF32` region through `silu_point_body` (`crates/tiler-compiler/src/elementary.rs:125-135`). The expression carries no member attribution to recover the realizing operation from.

**Inference — the surviving design closes site 4.7 with site 4.5.** `E` is a total deterministic function of `P'` and the declared inputs, so retaining `P'` both declares which candidate was selected and makes the expression's grouping evaluable, in one change and with no new evaluator.

**Consequence for Part 7.2, stated and not decided.** Item C's dependency question is unchanged and still Tom's. What the resolution removes is one argument against the plain-scalar alternative drafted beside it: that alternative "cannot carry the pointwise expression at all", and under the surviving design nothing needs it to — site 4.5's witness is a `SemanticProgram`, which `tiler-reference` already consumes. Item A's `pointwise_f32` accessor is correspondingly no longer load-bearing for this site.

**What it does not close.** A plan spending reassociation at both the semantic rewrite and a physical reduction split still needs the semantic evaluation and the declared-order reduction evaluator composed, which `ReferenceEvaluator::evaluate` cannot express; that is filed as [`compose-a-declared-reduction-topology-into-a-semantic-program-evaluation`](../../../tickets/compose-a-declared-reduction-topology-into-a-semantic-program-evaluation.md). Part 5's canonical-form claim was untested when this section was written, and the surviving design does not depend on it for soundness — the reference computes from `P'`, never from `E`. [Part 7.5](#75-what-landed-where-and-what-the-implementation-corrected) has since tested it and found it bounded rather than general, which does not disturb that independence.

**Answered by [the composed-realization-evaluation derivation](composed-realization-evaluation.md), which corrects one framing here rather than extending it.** The composition needs no third evaluator, because the two spends are never inside one region: a physical split reads the *materialized* prologue output as its contributor tensor (`crates/tiler-compiler/src/physical.rs:1508-1522`, `:1529`, `:1828`), and the one region that carries a prologue in its own scalar program declares `ReductionTopology::Serial` and never a split (`physical.rs:1421`). So the answer is a driver over the plan's stage cover chaining the two existing reference paths, and the surface it needs is parked as [`accept-the-composed-realization-evaluation-surface`](../../../tickets/accept-the-composed-realization-evaluation-surface.md). Read that record before treating site 4.5 and sites 4.1–4.4 as needing one object between them.

### 7.5 What landed, where, and what the implementation corrected

**Added by [`implement-the-realization-witness-vocabulary`](../../../tickets/implement-the-realization-witness-vocabulary.md) at base `61414b91`.** Every line citation below was read at that base; the record's own citations were taken at `c335bb5b` and several have moved, which is tabulated at the end of this section rather than silently corrected in place.

**Fact — items A and B shipped, and item C did not, exactly as the acceptance directed.** Both live in `crates/tiler-ir/src/schedule/witness.rs` and are re-exported from `tiler_ir::schedule`:

- **Item A** is `RealizationWitness<'a>`, aggregated by `RealizationWitness::of(&VerifiedScheduledRegion)`. It borrows the region's realization, scalar program, and reduction topology rather than copying them, so "a witness cannot disagree with the plan it describes" is structural instead of a property of the constructor.
- **Item B** is `UnpinnedFreedomSite`, with the three arms 7.2 drafts and no `Conforms`-shaped arm. Its two elided payload types are `UnrecordedFoldContraction` — naming the *adjacency* rather than the scalar program, so the squaring fold and the squaring fold with an epilogue share one variant — and `UnevaluableRealization`.
- **Item C did not land and no reference file changed.** `grep -rn "tiler_ir::schedule" crates/tiler-reference/src --include='*.rs'` returns **five** lines at this base rather than the three this record recorded — `conformance.rs:38` and `:282`, `bf16.rs:98`, `tests.rs:18`, `bf16/tests.rs:23` — and every one is still a behaviour vocabulary with no plan structure among them. The `PointwiseF32` doc-comment line the record cites at `index_region_oracle.rs:1414` is now at `:1510`.

**Fact — the refusal needed a producer, and the redirection left it without one.** 7.2 drafts `UnpinnedFreedomSite` as `from_witness`'s error type; with item C excluded nothing would ever construct one. The producer is sited beside the aggregation as `RealizationWitness::unpinned_freedom_site() -> Option<UnpinnedFreedomSite>`, an `Option` rather than a `Result` deliberately: `None` says the enumeration found no site this contract grants and this plan leaves open, which is a statement about the table and not about values.

**Proposal — the surface beyond 7.2's drafts is labelled a draft in its own rustdoc and is Tom's to accept.** `RealizationWitness::of`, `realization`, `order`, `accumulation`, `contributor_partition`, `arrival`, `rounds`, and `pointwise_f32` are 7.2's accepted list; `reduced_axes`, `contracted_shape`, `pass`, `fold_epilogue`, and `unpinned_freedom_site` are labelled `**Draft surface, not yet accepted.**` following the convention `crates/tiler-ir/src/index/sourced.rs` sets. The first three exist because the record's own Part 2 names `axes`, `contracted_shape`, and `pass` in the field sets of sites 4.1, 4.4, and 4.2, and 7.2 drafts no accessor for any of them; `pass` in particular is load-bearing, since a partial and a final pass of one split agree on every other field and commit different values.

**Acceptance correction — 2026-08-08.** The paragraph above records the implementation-base state. Tom accepted the narrowed `order` signature and all seven public additions on 2026-08-07, without exclusion, under [`accept-the-realization-witness-surface-as-built`](../../../tickets/accept-the-realization-witness-surface-as-built.md). Five of those additions were the labelled accessors the paragraph names; the other two were the already-unlabelled payload enums `UnrecordedFoldContraction` and `UnevaluableRealization`. [`retire-the-draft-labels-on-the-accepted-witness-surface`](../../../tickets/retire-the-draft-labels-on-the-accepted-witness-surface.md) retires the five stale markers and the historical framing on `order`. This later acceptance changes neither this record's `disposition: pending` nor any contract, identity, or implementation fact.

**Correction 1 — `order` is `Option<ContributorOrder>` rather than a total one, because a total one would be a mirror.** 7.2 drafts it total. A region carrying `ReductionTopology::None` combines no contributors, so a total accessor would hand back the vocabulary's single variant for a sequence that does not exist — a value two plans agree on for no reason about either, which is Part 1's mirror class exactly.

**Correction 2 — site 4.8's spend population is empty, so its class is Witness rather than Witness-unevaluable.** Part 3.3 classes the declared accumulation width as a witness no oracle honours, on the ground that `strict_partial_sums_under` has no width parameter and can only answer at the element type. At this base the intrinsic verifier refuses any other width: `crates/tiler-ir/src/schedule/builder.rs` lines 1516 and 1640 both rejected `*accumulation != region_arithmetic_type(&region.index.scalar_program)`, at the multi-pass and the cooperative admission respectively. A verified region's declared accumulation *is* its element width, which is precisely what the reference answers for, so the site joins 4.4 as declared-and-unspendable. `a_split_cannot_declare_an_accumulation_the_region_does_not_perform` in `crates/tiler-ir/src/schedule/witness/tests.rs` exercises all three other arithmetic types against the multi-pass fixture and each is refused. Widening a fold to a second accumulator width is what reopens the site.

**Correction 3 — site 3.1 is a mirror for a different reason than the record gives, and its `true` population is empty too.** Part 3.2 states that hard-coding `false` "would fail the schedule verifier's realization cross-check under a permitting contract". There is no such cross-check at this base. `crates/tiler-compiler/src/physical.rs:1975-1976` still derives the field from the contract, but the intrinsic verifier admits `ScalarProgram::FusedMultiplyAddSerialSum` only when the field is `false` — `crates/tiler-ir/src/schedule/builder.rs` line 1029 for the serial arm, line 1332 for a multi-pass partial pass, and line 1421 for a cooperative tile. So no verified region carries `true`, and a compiler that derived one under a permitting contract would have the region refused rather than admitted. The field is therefore worse than the record says: it does not even carry the contract's resolution reliably. The conclusion is unchanged — it is not a witness — and the witness vocabulary names the site through `UnrecordedFoldContraction::ScaleBiasContributor` rather than reading the field.

**Correction 4 — the vocabulary grew a ninth scalar program since `c335bb5b`, and it is a new freedom site the table does not carry.** `ScalarProgram::SquaredSerialSumThenEpilogue` (`crates/tiler-ir/src/schedule/model.rs:628-646`) pins a second `PointwiseF32Expression` inside a reduction. Call it **site 4.9**: `Reassociation`, spent at a fold's epilogue chain, class **Witness, unevaluable**, field `epilogue`. It is site 4.5's class for site 4.5's reason: the expression is the grouping, exactly, and no reference path evaluates one. The unevaluable bucket therefore becomes `{4.5, 4.6, 4.9}`, the same size as before because correction 2 moved 4.8 out of it, and the headline is twenty-five sites over the current vocabulary. It is aggregated at `RealizationWitness::fold_epilogue` and refused by name as `UnevaluableRealization::FoldEpilogueExpression`, not folded into `pointwise_f32`, because a caller that repaired one has not repaired the other.

**Correction 5 — the refusals are permission-gated, and site 3.3's is narrower than the record's second predicted refutation states.** Part 5's refutation 2 predicts a disagreement "over any region at all" under a contraction-permitting contract, on the ground that `-ffp-contract=off` is dropped exactly when the contract permits. That flag governs the form `a * b + c`, so a fold whose step is `accumulator + contributor` has nothing for it to change: `UnpinnedFreedomSite::BackendOrderUndeclared` is raised only when the region's *pointwise expression* states a multiply an addition consumes, and the fold adjacencies are raised more precisely by `ContractionUnrecorded` ahead of it. A bare `StrictSerialSum` under a contraction-permitting contract is therefore reported as leaving nothing open, which the enumeration's own framing supports: a freedom site is a place a granted permission could be *spent*, and there is no multiply there to spend it on. The same gating is why the two expression sites are raised only under a permitting reassociation — under a forbidding one the minted expression is a total deterministic function of the caller's own program, which the semantic evaluator already evaluates exactly.

**Measurement — Part 5's canonical-form claim holds for the two mitigations it names and is refuted in the general form it is stated in.** The claim — that the builder's one-leaf-per-ordinal sharing and its deterministic root-first topological order make "the canonical form a function of the program rather than of the spelling" — is stated there as the thing to check before a witness is built on it.

- **Holds, for what the mitigations cover.** `the_two_named_canonicalization_mitigations_hold` builds `x * 2.0 + (x + 3.0)` twice, minting the two independent subtrees in opposite orders and asking for the input leaf once-then-cloned in one spelling and twice in the other. The two expressions are equal, the two witnesses agree, and the two regions share a canonical schedule identity.
- **Refuted, for a spelling neither mitigation reaches.** Nothing shares a *constant*: `PointwiseF32ExpressionBuilder::constant` pushes a draft node unconditionally, and `canonicalize_nodes` maps draft nodes to canonical ordinals without hash-consing. `a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse` builds `x * 2.0 + 2.0` with one constant value and with two, and gets a four-node and a five-node expression, two different witnesses, and two different canonical region identities for one binary32 function.

**Inference — the refutation bounds the converse property rather than the determination property, and the direction matters.** The witness is too *fine* here, not too coarse: a caller comparing two witnesses may conclude they differ when the bits do not, never that they agree when the bits differ. Nothing unsound follows, and the implementation's response is to derive no `PartialEq` on `RealizationWitness` and to say why in its rustdoc, rather than to make the builder share constants — which would move the canonical schedule identity of every region that mints a repeated constant, an identity-domain step this ticket carries no evidence for. Whether the compiler's `mint_elementwise` can actually produce the duplicated spelling is **not established here**: two distinct `tiler.constant-f32` occurrences of one payload would mint two nodes by the recognizer's own "two operands naming one value share the node already minted" rule, but no compiled counterexample was produced, and the upgrade from a source reading to a compiled pair is the same one Part 4's measurement boundary describes. It is filed as [`share-identical-constants-in-the-pointwise-expression-canonical-form`](../../../tickets/share-identical-constants-in-the-pointwise-expression-canonical-form.md).

**Fact — line drift in this record's own citations, checked one by one at `61414b91`.**

| This record's citation | At `61414b91` |
| --- | --- |
| `ScalarProgram` at `model.rs:459`, eight variants | `model.rs:474`, **nine** variants |
| `ReductionTopology` at `model.rs:715` | `model.rs:844`, five variants unchanged |
| `IndexRegion` at `model.rs:628` | `model.rs:751` |
| `ScheduledRegion` at `model.rs:1006` | `model.rs:1135` |
| `KernelSchedule` at `model.rs:987` | `model.rs:1116` |
| `ContributorOrder` at `model.rs:101` | `model.rs:102` |
| `StrictTensorContraction` at `model.rs:565-573` | `model.rs:663-671`, three fields unchanged |
| `NumericalRealization` at `schedule/numerics.rs:208` | `schedule/numerics.rs:208`, unmoved |
| `CANONICAL_DIMENSIONS` at `numerics.rs:149` | `numerics.rs:149`, unmoved |
| `PointwiseF32Expression` at `pointwise.rs:150`, `Add` at `:91`, `is_valid` at `:181` | all unmoved |
| The site 3.1 mirror at `physical.rs:1415-1416` | `physical.rs:1975-1976` |

**Citation repair — 2026-08-19 by [`re-anchor-the-schedule-builder-line-citations`](../../../tickets/re-anchor-the-schedule-builder-line-citations.md), on corrections 2 and 3, whose subjects were consolidated rather than moved.** `crates/tiler-ir/src/schedule/builder.rs` became the `crates/tiler-ir/src/schedule/builder/` directory, so the pins those two corrections carried are restated above as historical file-and-line facts about `61414b91` and the live sites are named here instead. Neither correction's conclusion changes; both had two or three separate gates where the tree now has one.

- **Correction 2's two width rejections are one authority.** `crates/tiler-ir/src/schedule/builder/reduction.rs "pub(super) fn verify_accumulation_width("` is the single accumulation-width gate, and it is called from three admissions rather than the two the correction names — the multi-dispatch split, the cooperative fold, and the cooperative contraction, which postdates the correction. The comparison is also spelled differently: it derives the required width with `scalar_arithmetic_type` and compares `declared != required`, so a reader grepping for `region_arithmetic_type` in the verifier finds nothing. The conclusion — a verified region's declared accumulation is its own element width — is unchanged.
- **Correction 3's three `!contraction` gates are one table row.** `crates/tiler-ir/src/schedule/builder/family.rs "(!contraction).then_some(SplitFamily {"` is now the only site: `split_family` returns nothing for a contracting `FusedMultiplyAddSerialSum`, so every topology refuses it at once rather than each arm refusing it separately. The correction's conclusion — no verified region carries `true`, so the field is not a witness — is unchanged, and it is now discharged in one place.

**What this section does not establish.** No contract sentence moved, no ADR was proposed, and no identity domain stepped: `tiler.schedule.v5` is unchanged and no pinned artifact, cache, or region identity in the workspace moved for this work. The witness is aggregated and refused over regions; nothing here evaluates a value, and the composed driver [`accept-the-composed-realization-evaluation-surface`](../../../tickets/accept-the-composed-realization-evaluation-surface.md) parks is still where a caller's end-to-end path lives.

## What this record does not establish

- **No contract changed and no decision was made.** No permission admitted, no ADR proposed, no dimension added, no crate touched, no catalog edited beyond this record's own link from the parent.
- **Nothing here is measured on any device, and nothing here is run at all.** Every claim is a source reading at `c335bb5b` with the file, the line, and — for every absence — the one-line command that reproduces it. The enumeration is `sound-proof` within the stated model and `primary-source-synthesis` for the corpus claims; no compilation, no test, and no device execution was performed.
- **The twenty-four sites are exhaustive over the vocabulary and not over the future.** Distributivity and elementary-function identity are named normatively and ungrantable, so neither is enumerated. Part 1's classification rule is stated so a twelfth dimension or a ninth `ScalarProgram` variant is classified by applying it rather than by extending a list.
- **The canonical-form claim in Part 5 was untested here, and is no longer untested.** That two spellings of one pointwise program mint one node vector is what the builder is written to guarantee, and this record read the guarantee rather than exercising it. A witness built on the expression DAG needed that check first, and got it: [Part 7.5](#75-what-landed-where-and-what-the-implementation-corrected) records the two tests that exercised it, the extent over which the guarantee holds, and the duplicated-constant spelling that refutes it in the general form.
- **The counterexample walk is a reading, not a run.** Part 4 states the upgrade path — compile both candidates and compare the two payloads — and does not take it.
- **`RealizationNotEvaluable`'s population for a multi-round cooperative tile is unchanged from the parent's count** and is repeated here rather than re-derived; the parent's deferral and its trigger remain the authority for it.
