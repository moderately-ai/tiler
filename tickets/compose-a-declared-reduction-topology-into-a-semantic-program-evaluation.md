---
id: compose-a-declared-reduction-topology-into-a-semantic-program-evaluation
title: Compose a declared reduction topology into a semantic program evaluation
status: done
priority: p2
dependencies: []
related: [decide-how-a-pinned-pointwise-grouping-becomes-evaluable, derive-the-oracle-for-a-permitted-divergence-candidate, enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle, accept-the-realization-witness-surface, accept-the-composed-realization-evaluation-surface]
scopes: [research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, reference, conformance]
---
## User-visible outcome

A derivation of how one reference evaluation answers for a program that spends reassociation at *both* the semantic rewrite and a physical reduction split — so that a plan carrying a reassociated pointwise chain feeding a partitioned fold has one expected value rather than two half-answers.

## Why this exists

**Fact — the two witnesses are answered by two evaluators that do not compose today.** [The permitted-divergence oracle](../docs/research/reference/permitted-divergence-oracle.md) establishes O5: evaluate the program under the plan's own realization witness, compared bitwise. [The freedom-sites enumeration](../docs/research/reference/plan-freedom-sites.md) Part 2 then splits the witness across sites answered by different objects — sites 4.1 through 4.4 (the reduction topologies) by the declared-order evaluators `strict_partial_sums_under` and `strict_partitioned_sum_under` in `crates/tiler-reference/src/evaluate.rs`, and site 4.5 (the pointwise chain) by the semantic evaluator over the selected candidate's program, which [`decide-how-a-pinned-pointwise-grouping-becomes-evaluable`](decide-how-a-pinned-pointwise-grouping-becomes-evaluable.md) derived as the surviving design.

**Fact — the semantic evaluator cannot be told a topology.** `ReferenceEvaluator::evaluate` in `crates/tiler-reference/src/evaluate.rs` dispatches each operation through the frozen reference registry, and a `tiler::strict-serial-sum-f32@1` occurrence therefore resolves to the registered strict left fold. Its signature takes a program and input bindings; nothing in it names a `ReductionTopology`, a `ContributorPartition`, or an accumulation width. Exact check: `grep -n "ReductionTopology\|ContributorPartition" crates/tiler-reference/src/evaluate.rs` returns nothing.

**Fact — the population is non-empty and reachable.** A prologue expression feeding a fold is the ordinary shape of a reduced-elementwise program: `pointwise_region` builds a prologue region from `NormalizedSerialSum::prologue`, and multi-pass / cooperative topologies are constructed via `multi_pass_topology` and the cooperative path that binds `contributor_tensor(subject)` in `crates/tiler-compiler/src/physical.rs`. Under `REASSOCIATE_F32` or `RELAXED_F32` the same contract that admits the semantic rewrite admits the reduction split, so one plan can spend both freedoms.

**Inference — so an oracle assembled from the two objects separately answers for neither.** Evaluating the selected semantic program alone gets the fold's grouping wrong whenever the plan split it; evaluating the declared partition alone has no prologue. The composition is the question, and the shapes it could take — a topology-parameterized evaluation request through the registry, an evaluation staged at the materialization boundary the cover already names, or a witness-driven evaluation of the whole plan — are candidates to eliminate rather than one obvious answer.

## What this ticket must produce

The elimination, run against correctness first: which object answers for a program spending reassociation at both layers, stated so a reader can refute it, with the evaluated population named and every unsupported case an explicit refusal rather than a silent strict reading. If it resolves to a public surface, that surface is drafted and parked for Tom under ADR 0075, never self-accepted.

## Explicit non-goals

Implementing an evaluator or changing `crates/`; re-deciding the pointwise fork, which is settled; accepting the realization witness surface, which is [`accept-the-realization-witness-surface`](accept-the-realization-witness-surface.md).

## Closes when

The composition question has a derived answer with its evaluated population and its refusals named, or it is deferred with the evidence that would close it and a trigger stated.

## Outcome — 2026-08-06

**The composition question has a derived answer, and it is not a third object.** Recommended terminal state: `done` once reviewed. The derivation lands as [`docs/research/reference/composed-realization-evaluation.md`](../docs/research/reference/composed-realization-evaluation.md), a new record; [the freedom-sites record's](../docs/research/reference/plan-freedom-sites.md) Part 7.4 gains a forward pointer so the reader's existing path reaches it. Nothing was implemented, no `crates/` file was touched, and the public surface is parked for Tom.

**The answer.** Neither the semantic program nor the declared partition answers, and no evaluator that consumes both is needed. **A physical reduction split never absorbs the prologue**, so the two reassociation spends are never inside one region and the boundary the composition needs is already declared by the plan. `partial_reduction_region`'s own doc states it: the split "leaves the prologue, if there is one, where it was"; both split constructors bind `contributor_tensor(subject)`, which answers `TensorRole::Intermediate` for a staged prologue, and the one region carrying a prologue in its own scalar program declares `ReductionTopology::Serial` via `fused_region`. (Numeric `file:line` cites that once appeared in this paragraph were readings at `b9146836` only; they are not HEAD-valid — locate the symbols.) **The object that answers is the plan's own stage cover, driving a sequence of evaluations of the retained selected candidate `P'` with the declared-order fold substituted at each reduction stage the plan split.** It is O5 unchanged — subject `(P', C, W)`, compared bitwise — with `W` read as the cover's ordered stage sequence rather than one topology, and it needs no new exact evaluator.

**Strongest counterpoint, stated in the record rather than answered away.** The driver needs one primitive the reference lacks — pinning a `ValueId` to a tensor — and that primitive is a hole where an oracle should be: a caller pinning a tensor taken from the artifact under test makes the comparison vacuous, and the reference's types cannot tell the provenances apart. The mitigation is structural rather than documentary and is part of the parked surface: make the driver the only public entry, so no caller holds the primitive.

**Eliminated, each on a stated ground.** A topology-parameterized registry request — the fold order is a *registered canonical fact* (`"strict-left-fold"`, `"binary32-each-step"`, search `CanonicalValue::utf8("strict-left-fold")` in `crates/tiler-ir/src/semantic/registry.rs`) folded into the semantic snapshot identity, and the request is a closed four-field struct whose only per-occurrence channel is the attributes, which are part of the operation's identity. A witness-driven whole-plan evaluation — reversed by Tom's 2026-08-06 decision that `tiler-reference` never names a plan structure, and it duplicates `verify_cover`'s authority. A witness-elaborated semantic program — expressible, because `ReindexFormKind::SplitAxis` exists in `crates/tiler-ir/src/semantic/reindex.rs`, and eliminated on making a second unasserted definition of the split's order, on the elaborated program not being `P'`, and on not reaching `accumulation`, `arrival`, or `rounds`. A chain of the index-region oracle over the staged index sequence — the candidate the crate's own surface suggests, eliminated because its subject is one occurrence's canonical realization and a fused pointwise region is several. Refusing outright — not eliminated as a rule; it is what the refusals below do.

**Population covered.** Plans whose stage cover is complete and verified, whose pointwise stages claim atoms of a retained `P'`, whose reduction stages carry a topology in `{Serial, MultiPass, CooperativeWorkgroup at rounds == 1, Contraction}` with `accumulation` at the element type, and whose materialization edges carry dense `f32`.

**Refusals, seven, each named with the population that proves it can fire.** `CandidateProgramNotRetained` (non-empty today — `grep -rn "pub fn .*SemanticProgram" crates/tiler-compiler/src/` still returns nothing, so this is the interim answer for the whole population); `RealizationNotEvaluable` (inherited, multi-round tile and non-uniform split); `AccumulationWidthNotHonoured` (site 4.8); `FreedomsSharedOneRegion` (this record's own, **empty by construction** and written as a check because that emptiness is the answer's load-bearing premise); `MaterializationRounds` (this record's own, empty until a bf16 edge exists); `ExecutionOrderNotGuaranteed` and `ContractionUnrecorded` (inherited); `AssumptionUnvalidated` (inherited).

**Finding worth the coordinator's attention.** The one existing site in the workspace that composes the two oracles composes in the direction the settled fork eliminated, and its own doc says so: `the_assembled_split_program_matches_the_partitioned_sum_oracle` in `crates/tiler-compiler/src/pipeline/tests.rs` feeds `strict_partitioned_sum` the tensor its *first kernel* produced, justified in that test's doc as avoiding "the test's arithmetic". It is sound at this base only because its fixture's prologue is `2.0 * x + 1.0` — a multiply and an add, not a same-family chain — so it carries no reassociation site and `P' = P`. The repair is a change of provenance, not a new evaluator, and it belongs to whichever ticket lands the retention.

**Citations at landing (historical — base `b9146836` only).** The in-Outcome “citations corrected” table that once re-pinned cooperative construction to `physical.rs:1913`, multi-pass to `:2083` inside `multi_pass_topology`, and asserted `physical.rs:829-832` (`NormalizedSerialSum::prologue`) “is exact” was exact **only at `b9146836`**. Those numbers are **not** HEAD-valid; do not treat them as current authority. Live construction anchors are symbol names (see **Correction — 2026-08-10**). The research record may still carry the same rot.

**Worked example (test data, not a specialized capability).** `sum((x * 0.3) * 10.0)` over `[1, 4]` under `FLUSH_AND_REASSOCIATE_F32`, at a `2`-by-`2` split. The four corners of (which semantic program) × (which fold order) are four distinct binary32 values — `0x40400006`, `0x40400005`, `0x40400004`, and the plan's `0x40400003` — and the partition applied without a prologue answers `0x3f800002`. The record carries the one-line reproduction.

**Measurement boundary.** Nothing ran, nothing compiled, no device. Every repository claim in the 2026-08-06 Outcome was a source reading at `b9146836` with a file and a line; those line numbers have since moved. The four-corner table is exact binary32 arithmetic reproducible by the quoted command on any host.

**Commits:** three, on `tkt/compose-a-declared-reduction-topology-into-a-semantic-program-evaluation` from base `b9146836`. `11c7ad63` is the derivation; `842838ae` records this hash on the ticket, because a hash cannot name the commit containing it; `f6b98a39` sharpens one elimination ground after rereading the sum's attribute schema. **Integrate the branch tip**, which this paragraph is the last edit before.

**Files changed:** `docs/research/reference/composed-realization-evaluation.md` (new), `docs/research/reference/plan-freedom-sites.md` (forward pointer in Part 7.4), `tickets/compose-a-declared-reduction-topology-into-a-semantic-program-evaluation.md`, `tickets/accept-the-composed-realization-evaluation-surface.md` (new). Nothing outside `docs/research/reference/` and `tickets/`.

**Filed by this ticket:** [`accept-the-composed-realization-evaluation-surface`](accept-the-composed-realization-evaluation-surface.md) — `todo`. The ADR 0075 public boundary the record was forbidden to self-accept: item A the driver, item B the `ValueId`-keyed reference primitive, with the ordering question the record's counterpoint raises.

**Owed navigation row — `docs/research/README.md` is `contracts/navigation`, outside this ticket's scopes, and was not edited.** The coordinator should insert this line verbatim into the `### Numerical operations` group, immediately before the `[Conversion family decomposition across pairs]` row at `docs/research/README.md:50`, which is where the group's ordering puts it:

```text
- [Composing a declared reduction topology into a semantic evaluation](reference/composed-realization-evaluation.md) — pending; primary-source-synthesis, sound-proof; informs: [Correctness and testing](../correctness-and-testing.md), [Numerical semantics](../numerical-semantics.md)
```

## Current correction — 2026-08-09

The terminal handoff above is historical. [`accept-the-composed-realization-evaluation-surface`](accept-the-composed-realization-evaluation-surface.md) is `done`: Tom accepted item A as the sole public composition entry and kept item B crate-internal on 2026-08-06. The research record now states that decision consistently in its status, traceability, roll-up, and non-claims. The navigation row is also present in `docs/research/README.md`. Implementation remains not started because [`retain-the-selected-semantic-candidate-for-the-conformance-oracle`](retain-the-selected-semantic-candidate-for-the-conformance-oracle.md) is still `awaiting-decision`; `CandidateProgramNotRetained` remains the interim refusal.

## Correction — 2026-08-10

**Citation rot only; semantic conclusions and `status: done` are unchanged.** Outcome 2026-08-06 line numbers and the in-Outcome “citations corrected” cooperative/multi-pass re-pins were base-relative to `b9146836` and are **not** HEAD-valid at audit base `c99ac549` (nor on later trees). Why-this-exists live Facts no longer carry numeric `file:line` as current authority. Live construction anchors: `partial_reduction_region` and its doc clause `leaves the prologue, if there is one, where it was`; `fused_region` + `ReductionTopology::Serial`; `strict_partial_sums_under` / `strict_partitioned_sum_under`; `the_assembled_split_program_matches_the_partitioned_sum_oracle`. Optional residual (out of this ticket-only repair): research records `composed-realization-evaluation.md` and `plan-freedom-sites.md` Part 7.4 may share the same line rot.

## Graph maintenance

Filed by [`decide-how-a-pinned-pointwise-grouping-becomes-evaluable`](decide-how-a-pinned-pointwise-grouping-becomes-evaluable.md) as the bounded residue its surviving design does not reach. Not a blocker for that design: a pure pointwise region carries `ReductionTopology::None`, so site 4.5 is answered without this.
