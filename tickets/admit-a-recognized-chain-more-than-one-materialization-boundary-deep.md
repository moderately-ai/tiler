---
id: admit-a-recognized-chain-more-than-one-materialization-boundary-deep
title: Admit a recognized chain more than one materialization boundary deep
status: deferred
priority: p3
dependencies: []
related: [admit-a-staged-family-that-reads-a-materialized-intermediate, admit-elementwise-epilogues-over-a-materialized-intermediate, admit-a-scheduled-region-that-reads-two-materialization-edges]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, planner]
---
## User-visible outcome

`rms_norm(matmul(a, b), a) * a` is recognized instead of refused under `staged-operand-depth` — a recognized chain whose regions are separated by *two* materialization edges rather than one.

**Measured 2026-08-08 at `68ba010a`: the admission is available and buys no program.** Recognition already nests, the widening is a one-line change, and the widened program refuses `NoFeasiblePlan` instead of compiling — because the shape it nests has no scheduled region for a reason [`admit-a-scheduled-region-that-reads-two-materialization-edges`](admit-a-scheduled-region-that-reads-two-materialization-edges.md) owns, one crate down. See *What was measured* below.

## Where the wall is, and why it is a rule rather than a gap

**Fact (verified 2026-08-08).** Recognition admits at most one materialization edge per recognized shape, and a shape reached *across* an edge admits none. `crates/tiler-compiler/src/request.rs` states it as `StagedOperandAdmission`: `recognize_output` hands its declared output's occurrence `OneEdge`, and `recognize_epilogue_producer` — the one function reached across an edge — hands `NoEdge`. A staged occurrence at the far side that reads its own edge refuses under `staged-operand-depth`.

**Fact (verified 2026-08-08, one refinement).** `recognize_epilogue_producer` is the sole site handing `NoEdge`, and of the three shapes it recognizes only the staged one can place an edge: `normalize_contraction` refuses a non-declared operand under `contraction-operands`, and `recognize_reduction`'s contributor walk reads declared inputs by construction. So the depth rule has **one** guard, at `recognize_staged_family`'s `if admission == StagedOperandAdmission::NoEdge`, not two.

**Fact — corrected 2026-08-08; the previous statement here was false in two independent ways.** It read: *"The same rule is stated a second time, for the elementwise walk, by `plan_elementwise`'s `leaves.staged.is_none()` guard … `every_refusal_names_its_unrecognized_property` drives one instance of it (`sum(sum(x) * 2.0)`, reported as `operation-set`)."* Both halves are wrong:

- **That guard states a rule about chain *width*, not depth.** It refuses one walk that reaches a *second, different* folded value — `sum(a, 1) * sum(b, 1)` — which would give one region two `TensorRole::Intermediate` reads with no ordinal to attribute them by. The walk is still one boundary deep. That is the same unordinalled-role fact `record_leaf` refuses for one staged value read twice, and its region-vocabulary owner is [`admit-a-scheduled-region-that-reads-two-materialization-edges`](admit-a-scheduled-region-that-reads-two-materialization-edges.md), with [`admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region`](admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region.md) owning the one-value-twice spelling. *Measured:* renaming the guard's rule to `PROBE-second-edge-into-one-walk` makes `sum(a, 1) * sum(b, 1)` report the probe.
- **`sum(sum(x) * 2.0)` is not an instance of that guard at all.** Its `leaves.staged.is_none()` condition is *true*, so the `Folded` path is taken and then discarded by `From<ElementwiseRefusal> for RequestError`, because `NormalizedSerialSum` carries no producer field. *Measured:* renaming that `From` impl's rule to `PROBE-flattened-folded` makes the row at `request.rs`'s `every_refusal_names_its_unrecognized_property` report `left: "PROBE-flattened-folded"`, while renaming the `leaves.staged.is_none()` arm leaves it reporting `operation-set`. That third wall is genuinely about depth but is structural rather than this guard's, and [`name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set`](name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set.md) owns its rule name.

**Inference — corrected 2026-08-08.** The previous text read: *"The two guards are one rule about chain depth."* They are not one rule; see above. What survives is the other half, and it holds: the depth rule is **not** the unordinalled-`TensorRole::Intermediate` rule, because a two-boundary chain gives each region *one* intermediate read. What it needs is a recognized shape that can carry a producer at every level and a cover that can place two edges through one output's partition.

## What lifting it costs, read before promising it

The three unknowns this section named were all read and two of them measured.

- **`NormalizedStaged::producer` and `NormalizedEpilogue::producer` already nest — verified, and further than stated.** Every accessor over them recurses: `members`, `owns_region_members`, `producer_shape_for`, `input_elements_at`, `reads_declared_input`, `max_input_elements`, and `physical::spell_output`. So does `encode_output_subject`.
- **The subject arms stay self-delimiting under nesting — verified by reading.** `epilogue-f32.v1` writes its producer through `encode_output_subject` itself, and `staged-family.v2` writes a presence byte first and then the same recursive call. Neither carries a length that a deeper nesting would invalidate, and no subject or identity assertion moved under the widening below.
- **The recursion bound is the open one.** The producer walk *is* recursive rather than worklist-driven, and today its depth is bounded at two by this guard alone. Lifting it makes the depth the caller's program, which `plan_elementwise`'s own doc argues against for the elementwise walk: *"a recognizer that consumed host stack proportional to it would turn an input property into a crash rather than a refusal."* A depth counter would still be the wrong shape; a worklist rewrite of the producer walk is the answer, and it is unwritten.

## What was measured (2026-08-08, base `68ba010a`)

The widening was applied — `recognize_epilogue_producer`'s call site handed `StagedOperandAdmission::OneEdge` instead of `NoEdge` — run, and reverted.

- `rms_norm(matmul(a, b), a) * a` **is** recognized, as `Epilogue { producer: Staged { producer: Some(Contraction), operand_reads: [Staged, Input(0)] } }`. Nothing had to be built for the shape to be expressible.
- Exactly **one** test moved: `a_staged_operand_still_refuses_a_second_edge_and_a_deeper_chain`, the assertion of the refusal itself. The crate then held 784 tests — the two this ticket adds did not yet exist — and no cover, cost, identity, or subject assertion among the other 783 noticed.
- End to end the program still did not compile. It refused `NoFeasiblePlan` where it had refused `UnsupportedCapability { rule: "staged-operand-depth" }`.

**The reason no program is bought.** Every program this guard refuses contains a staged occurrence whose operand is an edge, and `physical::staged_plan` has no region for one: its only law arm, `root_mean_square_scale_plan`, destructures two `BoundaryRead::Input` operands, so such an occurrence is `RegionVocabularyWall::StagedFamilyUnspellable` however deep the chain around it is. That is the same wall [`admit-a-scheduled-region-that-reads-two-materialization-edges`](admit-a-scheduled-region-that-reads-two-materialization-edges.md) owns, and it is `tiler-ir`'s (`reads_bind_boundary_tensors_in_order`) before it is the compiler's — outside this ticket's `implementation/compiler` scope.

**Consequently the branch of *Closes when* that was reachable is the second one**, and the first one's own evidence is unobtainable at this base: no cover placing both edges can be observed, because no region candidate for the staged occurrence exists for a cover to place.

## Closes when

Either a two-boundary chain is recognized with the recursion bound stated and a cover placing both edges observed, or the depth rule keeps its one guard with one shared statement and this ticket records the measured reason it stays.

**The second is what landed.** The rule is stated once, at `StagedOperandAdmission`, which also names the two neighbouring folded-value refusals it is not and their owners; `crates/tiler-compiler/tests/recognized_chain_depth_boundary.rs` holds the end-to-end measurement, a compiling one-boundary control beside it, and the trigger.

## Trigger check log

`staged_family_over_a_materialized_intermediate.rs`'s `a_staged_family_over_an_edge_is_recognized_and_stops_at_the_region_vocabulary` asserts that the *one*-boundary chain `rms_norm(matmul(a, b), a)` remains uncompiled under every stated contract. `STRICT_F32` and `FLUSH_SUBNORMALS_TO_ZERO_F32` isolate the vocabulary census and report `UnsupportedCapability { rule: "region-vocabulary" }`; `RELAXED_F32`, `REASSOCIATE_F32`, and `FLUSH_AND_REASSOCIATE_F32` add fusion-legality `Unknown` and remain `NoFeasiblePlan`. When [`admit-a-scheduled-region-that-reads-two-materialization-edges`](admit-a-scheduled-region-that-reads-two-materialization-edges.md) lands, that test fails, and the measured reason above expires: this ticket should be reopened rather than the assertion relaxed. The assertion is left in that file so one measurement keeps one owner. A diagnostic-class change alone does not fire this trigger; scheduled-region binding of both edges does.

```sh
cargo nextest run -p tiler-compiler -E 'test(recognized_chain_depth_boundary) or test(staged_family_over_a_materialized_intermediate)'
```

- **2026-08-08 — not fired.** The widening buys zero programs at this base, measured rather than argued: applying `OneEdge` at `recognize_epilogue_producer` moves exactly one of `tiler-compiler`'s 784 tests (the refusal's own assertion), and `rms_norm(matmul(a, b), a) * a` still fails to compile — `NoFeasiblePlan` instead of `UnsupportedCapability { rule: "staged-operand-depth" }`. The blocking wall is `reads_bind_boundary_tensors_in_order` in `tiler-ir` (`implementation/ir`), not in this scope. Reproduce: `cargo nextest run -p tiler-compiler --locked -E 'test(/staged_family_over_a_materialized_intermediate|recognized_chain_depth_boundary/)'`.
- **Release condition.** Reopen when a scheduled region can bind two materialization edges — tracked by `admit-a-scheduled-region-that-reads-two-materialization-edges`. The trigger assertion lives in `staged_family_over_a_materialized_intermediate.rs` deliberately, so one measurement keeps one owner; whoever lands the `tiler-ir` widening will see it fail and should reopen this ticket rather than relax the test.
- **2026-08-09 — not fired.** `admit-a-scheduled-region-that-reads-two-materialization-edges` is `blocked`, and the one-boundary trigger test still owns the `NoFeasiblePlan` wall. No scheduled-region vocabulary can yet bind the two edges, so lifting the recognizer depth guard would still buy no executable program.
- **2026-08-10 — not fired.** The failure classifier moved, not the schedule/depth capability. The trigger subject remains uncompiled under all five contracts: strict and flush-only now report `UnsupportedCapability { rule: "region-vocabulary" }`, while the three reassociation-permitting contracts add fusion-legality `Unknown` and remain `NoFeasiblePlan`. `admit-a-scheduled-region-that-reads-two-materialization-edges` is still `blocked`, so no scheduled region binds the external operand edge and the law-internal handed value together; lifting the recognizer depth guard still buys no executable program.
