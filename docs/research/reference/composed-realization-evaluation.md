---
schema: "tiler-doc/v1"
id: "tiler.research.reference.composed-realization-evaluation"
kind: "research"
title: "Composing a declared reduction topology into a semantic evaluation"
topics: ["reference", "numerics", "conformance", "reductions", "correctness"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis", "sound-proof"]
informs: ["tiler.contract.correctness-and-testing", "tiler.contract.numerical-semantics"]
depends_on: ["tiler.research.reference.permitted-divergence-oracle", "tiler.research.reference.plan-freedom-sites"]
ticket: "compose-a-declared-reduction-topology-into-a-semantic-program-evaluation"
---

# Composing a declared reduction topology into a semantic evaluation

**Status:** derivation complete. The object that answers for a program spending reassociation at *both* the semantic rewrite and a physical reduction split is derived, five candidate shapes are eliminated against correctness first, exactly one survives, its evaluated population is named, and every case outside that population is an explicit refusal. The surface the survivor needs is drafted in prose and **parked for Tom** under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md); nothing here is self-accepted. No contract changed, no permission was admitted, and no `crates/` file was touched. Every repository claim was read at this record's own base, `b9146836`, and states the exact path, line, or one-line command that reproduces it.

Claims are labelled **Fact** when traced to inspected source or a merged record; **Inference** when derived from stated facts; **Measurement** when tied to an exact environment and procedure; and **Proposal** when not yet accepted or tested.

## Traceability

- **Work record:** [`compose-a-declared-reduction-topology-into-a-semantic-program-evaluation`](../../../tickets/compose-a-declared-reduction-topology-into-a-semantic-program-evaluation.md), filed by [`decide-how-a-pinned-pointwise-grouping-becomes-evaluable`](../../../tickets/decide-how-a-pinned-pointwise-grouping-becomes-evaluable.md) as the bounded residue its surviving design does not reach.
- **Authorities consumed rather than re-derived.** [The permitted-divergence oracle](permitted-divergence-oracle.md) supplies O5 — the oracle is `(program, contract, realization witness)` compared bitwise — and its four refusal classes; nothing here reopens that elimination. [The freedom sites a physical plan must pin](plan-freedom-sites.md) supplies the twenty-four-site enumeration and the five-way declared/mirror/undeclared/unevaluable/unspendable split; this record uses its site numbers unchanged. The pointwise evaluability fork is **settled** and is not re-decided: the reference evaluates the retained semantic candidate `P'`, never the minted `PointwiseF32Expression`.
- **The accepted boundary this derivation is bound by.** Tom decided 2026-08-06 on [`accept-the-realization-witness-surface`](../../../tickets/accept-the-realization-witness-surface.md) that items A and B are accepted as drafted and item C is **redirected to the plain-scalar form**: the reference's evaluation entry points keep taking plain scalars and the aggregation sites in `tiler_ir` alone, so `tiler-reference` must not name a plan structure. That is a decision, not a preference, and Part 2 eliminates one candidate on it.
- **Current disposition:** pending. No ADR adopts this record, no contract sentence has moved for it, no crate changed.
- **Normative destinations, neither edited here:** [Correctness and testing](../../correctness-and-testing.md) owns the oracle rule; [Numerical semantics](../../numerical-semantics.md) owns the dimension definitions. `contracts/numerics` and `contracts/navigation` are both outside the producing ticket's scopes, and the owed catalog row is reported to the coordinator in the ticket's Outcome rather than written here.
- **No literature acquisition failed for this record**, and no claim below rests on a source this repository does not hold.

## Outcome

**Fact — the ticket's premise that "an oracle assembled from the two objects separately answers for neither" is right, and the reason it is repairable is a structural property of the compile path that neither parent record states.** A physical reduction split never absorbs the prologue. `partial_reduction_region`'s own doc says so (`crates/tiler-compiler/src/physical.rs:1508-1522`): "It splits the *materialized* strategy's reduction rather than the fused one: it reads whichever tensor holds the fold's declared contributor domain and writes the partial tensor, so the split replaces one dispatch with two and **leaves the prologue, if there is one, where it was**." Both split constructors bind `contributor_tensor(subject)` — the partial pass at `physical.rs:1529` and the cooperative tile at `:1828` — and `contributor_tensor` (`:80`) answers `TensorRole::Intermediate` for a fold whose contributors come from a staged prologue (`declared_contributor_tensor`, `:65-72`). The one region that *does* carry a prologue inside its own scalar program, `fused_region` (`:1334`), declares `ReductionTopology::Serial` (`:1421`) and never a split.

**Inference — so the two reassociation spends are never inside one region, and the boundary the composition needs is already declared by the plan.** A program spending site 4.5 (the pointwise chain) and one of sites 4.2/4.3 (the multi-pass split, the cooperative tile) is covered by at least two regions with a materialization edge between them, and the value on that edge is exactly the fold's contributor tensor. There is no case in which one evaluation must answer for two freedoms at once; there is a case in which two evaluations must be *chained*, and the chain's shape is a fact the plan carries rather than one an oracle invents.

**Fact — and the boundary the ticket names is the cover's, not the index layer's, which are two staging vocabularies that must not be confused.** `MaterializationEdge` (`crates/tiler-compiler/src/cover.rs:431-450`) and `TensorRole::Intermediate` (`crates/tiler-ir/src/schedule/model.rs:63-64`) stage values between *scheduled regions*; `StagedIntermediate` and `VerifiedIndexRegionSequence` (`crates/tiler-ir/src/index/sequence.rs:80-87`, `:147-153`) stage them between the *canonical index regions* of one occurrence's realization law. The module header says the relationship exactly (`sequence.rs:13-19`): the index layer "mirrors what `tiler_compiler::frontier::derive_subprogram_boundary_contract` does one layer down … The two layers are deliberately separate IRs … so this is a model mirrored, not a mechanism reused." Part 2.4 eliminates the candidate that answers at the wrong one.

**The surviving object: the plan's own stage cover, driving a sequence of evaluations of the retained semantic candidate `P'`, with the declared-order fold substituted at each reduction stage the plan split.** It is O5 unchanged — the subject is `(P', C, W)` and the comparison stays bitwise — with one thing made explicit that the parent derivations left implicit: for a multi-freedom plan the witness `W` is not one topology, it is the cover's ordered stage sequence, and the object that consumes it is a *driver* rather than an evaluator. No new exact evaluator is needed. What is needed is one primitive the reference does not have: the ability to pin a semantic value to a tensor and to observe a value that is not a declared output. Both are keyed by `tiler_ir::semantic::ValueId`, which `crates/tiler-reference/src/evaluate.rs:29` already imports, so the survivor respects the plain-scalar decision rather than reopening it.

**Fact — the one existing site in the workspace that composes the two oracles composes in the direction the settled fork eliminated, and it says so in its own doc.** `the_assembled_split_program_matches_the_partitioned_sum_oracle` (`crates/tiler-compiler/src/pipeline/tests.rs:6501`) computes `pointwise = interpret_fused(&kernels[0], &values)` (`:6523`), wraps it as `pointwise_tensor` (`:6527`), and passes *that* to `strict_partial_sums` and `strict_partitioned_sum` (`:6530-6531`). Its own comment states the choice: "The oracle's input is the program's own prologue output rather than a value re-derived here" (`:6498-6500`). **Inference — that is design 2 of the settled fork, at the composition boundary instead of at the expression:** the expected value is read out of the artifact under test, so no defect at or before the prologue kernel can be refused. It is sound today for one narrow reason and not by construction, which Part 6 states exactly.

## Part 1 — Why the question has a boundary to be answered at

### 1.1 The population, verified reachable

**Fact — the semantic spend is a same-family chain rewrite.** `OrderedReassociationRule::add()` (`crates/tiler-compiler/src/normalize.rs:685`) and `::multiply()` (`:692`) rewrite `(a op b) op c` into `a op (b op c)`; the permission gate accepts with `"numerical.reassociation-permitted"` (`:770`) and `rebuild_ordered_reassociation` (`:993`) emits the rewritten `SemanticProgram` as a sibling candidate.

**Fact — the physical spend is gated on the same dimension at two sites.** `split_reduction_regions` returns `SplitUnavailable::ReassociationForbidden` before any region exists (`crates/tiler-compiler/src/physical.rs:2013-2015`), and `single_workgroup_tree_region` returns `WorkgroupTreeUnavailable::ReassociationForbidden` (`:1811-1813`).

**Fact — three registered contracts permit reassociation:** `RELAXED_F32` (`crates/tiler-compiler/src/session.rs:1454`), `REASSOCIATE_F32` (`:1472`), `FLUSH_AND_REASSOCIATE_F32` (`:1490`).

**Fact — the split's own admission bound is four contributors.** `governed_partition` (`physical.rs:1488-1506`) returns `None` below four and otherwise the balanced exact split; at four contributors it returns `ContributorPartition { partitions: 2, contributors_per_partition: 2 }`.

**Inference — the population is therefore every reduced-elementwise program whose prologue carries a same-family `f32` chain of at least three leaves and whose fold has at least four contributors with an exact balanced split, compiled under one of the three contracts.** It is non-empty, it is the ordinary shape of a normalized reduction, and Part 4 works one member of it.

### 1.2 The two citations this record corrects

**Fact — the ticket's construction-site line numbers have drifted, and the constructs are where it says in substance.** `physical.rs:829-832` is exactly `NormalizedSerialSum::prologue` inside `pointwise_region`, unchanged. The cooperative construction the ticket cites at `:1896` is at **`physical.rs:1913`** at this base (`:1896` is that region's `ownership_proof`); the multi-pass construction it cites at `:2060` is at **`physical.rs:2083`**, inside `multi_pass_topology` (`:2077`), and `:2060` is the `ReductionTopology::MultiPass` pattern inside `declared_partial_partition` (`:2058`). Both citations were verified rather than inherited, per this repository's rule that a line number is evidence only when reread.

### 1.3 The composition theorem, stated so it can be refuted

**Inference — for every plan this build can compile, the following holds.** Let `R` be a region whose scalar program realizes a pointwise chain in which the semantic rewrite was spent, and let `R'` be a region whose reduction topology is `MultiPass` or `CooperativeWorkgroup`. Then `R ≠ R'`, and the value `R` writes is the tensor `R'` reads as its contributor domain.

**The refutation procedure.** Exhibit a region this build constructs whose `ScalarProgram` is `PointwiseF32` or `FusedMultiplyAddSerialSum` **and** whose `KernelSchedule::reduction` is not `ReductionTopology::None` or `ReductionTopology::Serial`. Part 5's refusal R3 is the fail-closed answer for exactly that region, so a change that makes the theorem false is caught rather than silently mis-evaluated. At this base the population of such regions is empty: `elementwise_region` (`physical.rs:888`) builds every `PointwiseF32` region through `linear_schedule` (`:2096`), whose `reduction` is `ReductionTopology::None` (`:2103`), and `fused_region` is the only `FusedMultiplyAddSerialSum` constructor and declares `Serial`.

## Part 2 — The candidates and the elimination

Write `P` for the caller's verified semantic program, `P'` for the candidate the portfolio selected, `C` for the resolved numerical contract, `W` for the plan's realization witness, and `z` for the compiled candidate's observed output bits.

- **C1 — a topology-parameterized evaluation request through the registry.** Carry the declared split into `ReferenceEvaluationRequest` so a `tiler::strict-serial-sum-f32@1` occurrence folds in the plan's order.
- **C2 — a staged evaluation over the plan's own stage cover**, chaining tensors along the materialization edges the cover declares, each stage answered by the reference path its witness names.
- **C3 — a witness-driven evaluation of the whole plan**, one reference entry point taking the plan or an aggregated witness and walking it.
- **C4 — a witness-elaborated semantic program**: rewrite `P'` into `P''` whose *strict* reading is the declared order, and evaluate it with the unchanged evaluator.
- **C5 — refuse**: a program spending both freedoms is `RealizationNotEvaluable`, and only single-freedom programs are qualified.
- **C6 — chain the index-region oracle over the staged region sequence**, reusing the one reference path that already accepts an arbitrary intermediate tensor. This candidate is not in the ticket; it is the shape a reader arrives at from the crate's own surface, which is why it is eliminated in writing rather than passed over.

### 2.1 C1 is eliminated because the fold order is a registered identity fact, not a parameter

**Fact — the request has no room for it and no extension point.** `ReferenceEvaluationRequest` (`crates/tiler-reference/src/registry.rs:126-132`) has exactly four fields — `operands`, `attributes`, `iteration_step_allowance`, `conformance` — and its construction site (`crates/tiler-reference/src/evaluate.rs:214-220`) varies only the first two per occurrence; the other two are fields of the evaluator (`evaluate.rs:52-56`) and are program-wide. Exact check: `grep -rnE "ReductionTopology|ContributorPartition|ArithmeticType" crates/tiler-reference/src` returns nothing.

**Fact — capability lookup does not read attributes, so the per-occurrence channel cannot select an implementation.** `ReferenceCapabilityKey` (`registry.rs:303-307`) is `(operation, signature)`, and `FrozenReferenceRegistry::resolve` (`registry.rs:713-763`) uses `attributes` only to project and compare occurrence authority.

**Fact — the strict left fold is a registered canonical fact of the operation, folded into the semantic snapshot identity.** The normative reference is `"tiler::strict-serial-sum-f32@1; lexicographic serial contributors"` (`crates/tiler-ir/src/semantic/registry.rs:2325-2327`), and the registered facts are `"strict-left-fold"` (`:2331-2333`) and `"binary32-each-step"` (`:2335-2338`), which `compute_identity` (`:2680-2694`) folds. The reference's own contract is that an implementation is "a deterministic function of the request" (`registry.rs:108`).

**Inference — so C1 has only two forms and both are eliminated.** Passing the split *beside* the attributes makes the registry answer a value at which its own registered fact `"strict-left-fold"` is false, which is not a relaxed oracle but a redefinition of the operation — the registry is the normative authority for what `tiler::strict-serial-sum-f32@1` *means*, and an authority that answers a different value per plan has stopped being one. **Fact — passing it *inside* the attributes is refused at the semantic boundary before any evaluation:** `StrictSerialSumF32::infer` requires `attributes.fields().len() == 1` and refuses anything else as `"sum.attributes"`, "Sum requires exactly the axes attribute" (`crates/tiler-ir/src/semantic/registry.rs:2622-2627`). Widening that schema would make the split part of the occurrence's identity and therefore of the program's, which is C4 with the elaboration hidden in an encoding and without C4's honesty about having built a second program. **Eliminated on correctness and identity, not on cost:** the parameter costs nothing; what it costs is the property that a frozen capability revision determines its answer.

**Fact — and C1 does not even reach the whole question.** A cooperative tile declares `accumulation: ArithmeticType` (`crates/tiler-ir/src/schedule/model.rs:806`, set from the contract at `physical.rs:1918` and `:2088`) and a `ContributorArrival`; the reference has no accumulation-width plumbing at all — `contraction.rs:285-293` refuses any non-`f32` accumulator by name rather than rounding through one — so a topology parameter would carry fields nothing could honour.

### 2.2 C3 is eliminated on the accepted boundary and on duplicating an authority

**Fact — the reference cannot see a plan structure, deliberately.** `grep -rn "ScalarProgram\|ReductionTopology\|VerifiedScheduledRegion\|PointwiseF32" crates/tiler-reference/` returns one line, a doc comment in a test (`crates/tiler-reference/tests/index_region_oracle.rs:1414`), and `tiler-compiler` depends on `tiler-reference` only as a dev-dependency, so the two sides can meet only in `tiler-ir`.

**Fact — Tom already decided this exact question in the other direction.** The 2026-08-06 decision on [`accept-the-realization-witness-surface`](../../../tickets/accept-the-realization-witness-surface.md) redirected item C to the plain-scalar form, and [`implement-the-realization-witness-vocabulary`](../../../tickets/implement-the-realization-witness-vocabulary.md) records the constraint as "part of the decision, not a style preference".

**Inference — and even without that decision C3 would be the weaker design, because it duplicates the cover authority.** The obligation "the stages realize the subject's occurrences, each exactly once, and nothing else" is discharged by `crate::cover::verify_cover` over `SemanticStage` atoms (`crates/tiler-compiler/src/region.rs:233-253`, the atom at `:157-227`). A reference-side plan walk would restate that traversal in a crate that cannot see the verifier, and two authorities over one fact drift. **Eliminated on an accepted decision and on maintainability; it is not eliminated on correctness**, and saying so is the honest form — C3 is C2 with the traversal moved to the wrong crate.

### 2.3 C4 is expressible, which is why it needs a real argument rather than a dismissal

**Fact — the semantic vocabulary can state a blocked split, and this record does not get to pretend otherwise.** `tiler::reindex-f32@1` carries `ReindexFormKind::SplitAxis` (`crates/tiler-ir/src/semantic/reindex.rs:196`), documented as replacing "one axis by a row-major factorization of it, major factor first" (`:576-582`), refusing any factorization whose product is not the extent (`SplitNotTotal` at `:332-339`, `SplitNotSurjective` at `:346-353`), evaluated by the reference at `crates/tiler-reference/src/structural.rs:309-331`, and lowered by the compiler as one affine combination whose own test comment says "Row-major order is unchanged, which is what makes a split a reshape" (`crates/tiler-compiler/src/governed.rs:3332-3333`). A sum occurrence's axes must be unique and strictly ascending *within* one occurrence (`crates/tiler-ir/src/semantic/registry.rs:2654-2662`) and `Shape::without_axes` (`crates/tiler-ir/src/shape.rs:195-208`) renumbers survivors, so two occurrences can reduce the inner factor and then the outer one.

**Inference — so `strict_partitioned_sum_under`'s value is expressible as a program**, for `Serial`, `MultiPass`, and a single-round `CooperativeWorkgroup`, by one `split-axis` reindex and two sums. That makes C4 a live candidate rather than a straw man, and three things eliminate it.

**First, it makes a second definition of the split's order, and nothing would check the two agree.** `strict_partial_sums_under`'s index arithmetic is `partition * chunk + within` (`crates/tiler-reference/src/evaluate.rs:688`) and its doc is the normative statement of what a declared split computes (`:544-557`); the reindex's is major-factor-first row-major. They agree at this base, and under C4 that agreement becomes load-bearing while remaining unasserted. The corpus's own rule — a difference is attributed to a named cause or it is a defect — is not served by two spellings of one order in two crates.

**Second, the elaborated program is not `P'`, and the oracle's subject silently becomes a third object.** `SemanticProgram` is verified against a registry snapshot and `ReferenceEvaluator::evaluate` resolves every occurrence through `program.semantic_registry()` (`evaluate.rs:203-208`). A program that never used `reindex-f32` need not carry it in its snapshot, so the elaboration can fail to verify for a reason that has nothing to do with the plan, and the failure would present as an unqualifiable candidate. **Inference, and checkable:** compile a reduced-elementwise program with no structural read, then attempt to build `P''`; if the snapshot admits the reindex the obstacle is absent and this ground weakens to the first one alone.

**Third, it does not reach the fields a topology carries beyond the partition.** `accumulation` has no semantic spelling — the sum's registered fact is `"binary32-each-step"` — and `ContributorArrival` and a tile's `rounds` have none either. C4 would answer the `f32` reading for a topology declaring something else, silently, where C2 has no argument to pass and refuses.

**Eliminated as the composition object, and retained in one strictly weaker role.** An elaborated `P''` is a cheap cross-check that the two definitions of the blocked split agree: evaluate `P''` and `strict_partitioned_sum_under` on the same tensor and require bit equality. That is a test, not an oracle, and it is not filed as work because it has no caller until the survivor exists.

### 2.4 C6 answers at the wrong staging layer, and its subject is one occurrence rather than a fused chain

**Fact — the reference does have a path that accepts an arbitrary intermediate tensor.** `IndexRegionEvaluator::evaluate` (`crates/tiler-reference/src/oracle.rs:1401-1409`) takes `(&VerifiedIndexRegion, IndexRegionAuthority, &[IndexRegionInput])` and returns that region's outputs, and it carries a conformance (`oracle.rs:1325-1331`). So chaining it along a staged sequence is the reuse the crate's surface suggests.

**Fact — but its subject is a canonical realization of *one occurrence*, derived from that occurrence's own attributes, and the closed law vocabulary emits at most two stages** (`crates/tiler-ir/src/index/sequence.rs:35-41`, whose ceiling doc says so; the two staged laws are `StagedStrictSerialSumThenPointwiseF32` and `StagedRootMeanSquareScaleF32`, `crates/tiler-ir/src/index/law.rs:106` and `:138`). The settled fork already recorded what that leaves out: `IndexRealizationLaw` "proves each occurrence is canonically realizable; it says nothing about which operands the fused expression's nodes consume, which is where a regrouping lives."

**Inference — so C6 cannot see the freedom it would have to answer for.** A reassociated prologue is a *fused* pointwise region over several semantic occurrences, and the index layer has no object for it; the sequence it does have is the realization chain of a single occurrence, at a different layer from the cover's materialization edges. A C6 chain would therefore answer for each occurrence in isolation and for none of their fusion, which is the same coverage failure Part 2.1 finds in C1 arriving from a different direction. **Eliminated on subject, not on cost.** It keeps the role the parent derivation already gives it: the index-region oracle stays the authority over per-occurrence canonical realizability, and this record neither widens nor narrows it.

### 2.5 C5 is the fail-closed baseline, and it is beaten rather than assumed away

**Inference — refusing is always correct and is the answer if no boundary exists.** Part 1.3 shows a boundary does exist for every plan this build compiles, so C5 would refuse a population that is evaluable — and the population is not marginal, it is the ordinary reduced-elementwise shape under any of the three reassociating contracts. **Not eliminated as a *rule*:** C5 is exactly what Part 5's refusals do for the cases outside the survivor's population, which is the correct reading of a fail-closed default.

### 2.6 C2 survives

It never admits a wrong result, because every stage is compared against one exactly determined value. It needs no new exact evaluator: `ReferenceEvaluator::evaluate` (`evaluate.rs:178`) and the `strict_*_under` family (`:603`, `:766`) are the two answers, unchanged. It refuses explainably, by naming the stage whose witness no reference path evaluates. It costs one reference evaluation per split fold plus one, which is inside the stated budget. And it composes with the settled pointwise design without modifying it: every stage's value is computed from `P'` and never from the minted expression `E`.

**Performance eliminates nothing here, and no candidate was discarded for being expensive.** C1 and C4 are one evaluation each, C2 is `k + 1` evaluations for `k` split folds and `k` is the number of reductions in a program, C3 is one. The eliminations above name what each candidate would silently accept or redefine, not what it would cost.

## Part 3 — The surviving object, stated so a reader can refute it

**Proposal — the driver.** Given `P'`, the resolved contract `C`, the plan's stage cover, the declared inputs, and a conformance derived from the region's realization:

1. Take the reduction stages the plan split, in cover order. For each, the plan names the semantic value on its incoming materialization edge and the semantic value it produces.
2. Evaluate `P'` under `C`, with every previously computed fold result **pinned** to the tensor the declared-order evaluator produced for it, and **observe** the value on the next split fold's incoming edge.
3. Evaluate that fold with `strict_partitioned_sum_under(t, axes, partitions, contributors_per_partition, conformance)` at the split the plan declared, and record the result against its semantic value.
4. When no split fold remains, evaluate `P'` with all pins in place; its declared outputs are the expected bits.

**Inference — the composite value is a function of `(P', C, W)` and of nothing else, which is the property that keeps the settled fork's elimination intact.** Every pinned tensor is produced by a reference path from `P'` and the witness; no step reads a tensor the compiler produced. That is precisely the property the existing composition site does not have (Part 6), and it is the whole difference between the survivor and the shape it replaces.

**Fact — the driver never has to reconstruct which operations a stage realizes, because the cover already proves it.** `verify_cover` builds a per-atom count over the region graph's stage nodes and requires each to be covered exactly once, refusing `CoverError::UncoveredMember` at zero and checking the occurrence's own duplication legality above one (`crates/tiler-compiler/src/cover.rs:1157-1189`, with the mask obligation stated in the comment at `:1172-1174`). The atom is the pair `SemanticStage { member, stage }` (`crates/tiler-compiler/src/region.rs:176-179`), made a pair by Tom's 2026-08-06 decision precisely so that two regions realizing one occurrence in sequence do not claim the same set (`region.rs:157-174`).

**Inference — and deliberate duplication does not disturb the survivor.** A cover may place one occurrence in two regions under an admitting policy, and the cover's own condition holds that "Every copy is the same value" (`cover.rs:420-429`). The driver computes each stage's value from `P'` rather than from any region, so two copies of one atom contribute one value by construction rather than by trusting the condition — which is the correct direction of dependence.

**Inference — the determination property specializes [the freedom-sites record's](plan-freedom-sites.md) Part 5 rather than restating it.** Two plans compiled from one `P` under one `C` whose stage covers agree on the ordered sequence of `(stage kind, claimed atoms, topology witness)` triples, and whose pointwise stages claim the same atoms of the same selected candidate, produce identical bits. To refute it, exhibit two such plans that disagree in bits; the disagreement names a freedom site the enumeration's twenty-four missed.

**Inference — the survivor's strongest counterpoint, stated rather than answered away.** A pinning primitive is a hole where an oracle should be: a caller that pinned a value taken from the device output would make the comparison vacuous, and the reference's types cannot tell the two provenances apart — a `Tensor` is a `Tensor`. This is real and it is not hypothetical, because it is exactly the mistake the one existing composition site already makes with a hand-written chain. The mitigation is a design choice and belongs in the parked surface rather than in a convention: make the *driver* the public entry point, taking only `(P', W, inputs)` and doing its own pinning, so no caller ever holds the primitive. That keeps the survivor's advantage structural instead of documentary, and it is Part 7's item A.

## Part 4 — The worked example, at four contributors

**The program, deliberately general in shape.** One declared `f32` input of shape `[1, 4]`, a same-family multiply chain `(x * 0.3) * 10.0` applied per element, and a strict serial sum over axis 1. Compiled under `FLUSH_AND_REASSOCIATE_F32`, this spends both freedoms: `OrderedReassociationRule::multiply()` rewrites the chain to `x * (0.3 * 10.0)`, and `governed_partition(4)` offers the `2`-by-`2` split. The constants are chosen only so a reader can see the rewrite's effect by eye — `0.3f * 10.0f` is exactly `3.0` in binary32 — and nothing in the derivation depends on them.

**Fact — the four corners of (which semantic program) × (which fold order) are four distinct binary32 values, at operands where every value and intermediate is normal.**

| Semantic program | Fold order | Result |
| --- | --- | --- |
| `P` baseline, `(x * 0.3) * 10.0` | strict serial | `0x40400006` |
| `P` baseline | declared `2`-by-`2` split | `0x40400005` |
| `P'` rewritten, `x * (0.3 * 10.0)` | strict serial | `0x40400004` |
| **`P'` rewritten** | **declared `2`-by-`2` split** | **`0x40400003`** |

The plan's value is the last row. A fifth object — the declared partition applied to the raw declared input, with no prologue at all — answers `0x3f800002`, which is not near any of them.

**Inference — this is the ticket's "two half-answers" made exact.** The semantic evaluation of `P'` alone answers `0x40400004`: right prologue, wrong grouping, and it would refuse a correct implementation. The declared partition alone answers `0x3f800002`: right grouping, no prologue. The strict reading of the baseline answers `0x40400006`, which is O1 refusing a correct implementation exactly as [the parent derivation's](permitted-divergence-oracle.md) Part 6 shows it doing. Only the composition lands on `0x40400003`, and it lands there without any object knowing more than the plan already declares.

**Reproduce the whole table with the following.** NumPy supplies the binary32 arithmetic and it is not incidental: an `f64` add narrowed back to binary32 double-rounds and is a different computation.

```text
python3 - <<'PY'
import struct, numpy as np
F = np.float32
f = lambda b: F(struct.unpack('>f', struct.pack('>I', b))[0])
h = lambda x: format(struct.unpack('>I', struct.pack('>f', F(x)))[0], '08x')
c1, c2 = F(0.3), F(10.0)
k = F(c1 * c2)
x = [f(b) for b in (0x3f762e6e, 0x3d1d1920, 0x346e45fa, 0x335d516b)]
P  = [F(F(v * c1) * c2) for v in x]   # baseline:  (x * 0.3) * 10.0
Pr = [F(v * k)          for v in x]   # rewritten: x * (0.3 * 10.0)
def ser(v):
    s = v[0]
    for t in v[1:]: s = F(s + t)
    return s
def two_by_two(v): return F(F(v[0] + v[1]) + F(v[2] + v[3]))
print("0.3f * 10.0f =", h(k), float(k))
print("prologue P ", [h(v) for v in P])
print("prologue P'", [h(v) for v in Pr])
print("P  serial", h(ser(P)),  " P  2x2", h(two_by_two(P)))
print("P' serial", h(ser(Pr)), " P' 2x2", h(two_by_two(Pr)))
print("no prologue, 2x2 over x:", h(two_by_two(x)))
PY
```

**Measurement boundary.** Nothing here ran on a device and nothing here compiled. The table is exact binary32 arithmetic reproducible by the command above on any host; it says what the five objects answer, not what any hardware did. Every operand and intermediate is normal, so the two subnormal dimensions are silent in this example exactly as they are in the parent record's — a subnormal-producing operand set at the same shape would make the conformance argument observable, and the survivor threads the conformance to every stage precisely so that it would not be attributed to the grouping.

## Part 5 — The evaluated population, and every refusal

**The population the answer covers.** A plan whose stage cover is complete and verified; whose pointwise stages claim atoms of a retained selected candidate `P'`; whose reduction stages each carry a topology in `{Serial, MultiPass, CooperativeWorkgroup at rounds == 1, Contraction}` with `accumulation` equal to the element type; and whose materialization edges carry `f32` dense values. That is the six evaluable witness sites of [the freedom-sites enumeration](plan-freedom-sites.md) plus site 4.5 through the settled retention design, chained.

**Every case outside it is an explicit refusal, and each is named with the population that proves it can fire.**

1. **`CandidateProgramNotRetained` — the selected semantic candidate is not available.** The whole survivor rests on `P'`, and `grep -rn "pub fn .*SemanticProgram" crates/tiler-compiler/src/` still returns nothing at this base. **Population: every program compiled today under a reassociating contract whose portfolio selected a rewritten candidate.** Closed by the settled design landing; until it does, the survivor is a derivation and not a mechanism, and refusing is the correct interim answer rather than falling back to the baseline `P`.
2. **`RealizationNotEvaluable` — a reduction stage's witness names an order no reference path implements.** Inherited unchanged from the parent derivation's class 2: a cooperative tile with `rounds > 1`, whose order `strict_partial_sums_under`'s flat `partition * chunk + within` (`evaluate.rs:688`) cannot state, and any non-uniform split, which `ContributorPartition::covers` (`crates/tiler-ir/src/schedule/model.rs:1008-1016`) does not admit.
3. **`AccumulationWidthNotHonoured` — a stage declares `accumulation` other than the element type.** Site 4.8. `strict_partial_sums_under` has no width parameter and the contraction reference refuses a non-`f32` accumulator by name (`crates/tiler-reference/src/contraction.rs:285-293`). Empty today because `physical.rs:1918` and `:2088` set it from the contract's arithmetic type, and non-empty the moment a contract resolves a different one.
4. **`FreedomsSharedOneRegion` — a region carries both a spent pointwise chain and a split topology.** This is Part 1.3's theorem inverted into a check: a region whose scalar program is `PointwiseF32` or `FusedMultiplyAddSerialSum` and whose topology is neither `None` nor `Serial` has no staging boundary, and the survivor has nothing to chain. **The population is empty by construction at this base** — which is why it must be a refusal and not an assumption, because the assumption is what a future fused-and-split region would silently break. Reproducing check: read `linear_schedule` (`physical.rs:2096-2110`) and `fused_region`'s topology (`:1421`).
5. **`MaterializationRounds` — an edge whose storage type is not the type the fused path holds the value at.** Empty today: `MaterializationRounding` has one variant (`crates/tiler-ir/src/schedule/numerics.rs:566`) and every edge in this population is dense `f32`. Non-empty the moment a bf16 edge exists, and it must refuse rather than assume the staging is bit-neutral.
6. **`ExecutionOrderNotGuaranteed` and `ContractionUnrecorded`.** Inherited unchanged from the parent derivation's classes 3 and 1 and from sites 3.2 and 3.3; a contraction-permitting contract is outside this record's population for reasons that have nothing to do with composition.
7. **`AssumptionUnvalidated`.** Inherited unchanged.

**Inference — refusals 4 and 5 are this record's own, and both are deliberately empty populations.** A refusal whose population is empty today is not decoration when the emptiness is the load-bearing premise of the answer: they are the two places where the derivation would become wrong, written as checks that fail closed instead of as sentences a reader has to remember.

## Part 6 — The existing composition site, and why it is the shape the fork eliminated

**Fact — the site and its own justification.** `crates/tiler-compiler/src/pipeline/tests.rs:6501-6556` interprets the split plan's three kernels in sequence, then compares the second and third against `strict_partial_sums` and `strict_partitioned_sum` applied to `pointwise_tensor` — the tensor the *first kernel* produced (`:6523`, `:6527`, `:6530-6531`). The doc states the reasoning at `:6498-6500`: re-implementing the prologue in the test "would assert the test's arithmetic".

**Inference — the reasoning is right and the conclusion it reaches is the eliminated one.** Re-implementing the prologue in the test is indeed the wrong repair; so is reading it out of the artifact. The third option is the one [the settled fork](../../../tickets/decide-how-a-pinned-pointwise-grouping-becomes-evaluable.md) derived: evaluate the prologue from the semantic program, which is neither the test's arithmetic nor the compiler's answer. The site is sound at this base for one narrow reason — its fixture's prologue is `2.0 * x + 1.0` (`pipeline/tests.rs:5595-5602`), a multiply and an add, which is not a same-family chain and therefore carries no reassociation site, so the selected candidate is the baseline and `P' = P`. That is the same conditional soundness the settled fork identified in `pipeline/conformance.rs`: the existing gate means what it says only while no fixture spends the semantic freedom.

**Inference — so the survivor's contribution to this site is not a new evaluator but a change of provenance.** The comparison shape stays; what moves is where `pointwise_tensor` comes from. That is a one-line change in kind and a substantial one in what the test can refuse, and it is the concrete reason the composition question is worth a derivation rather than a convention.

## Part 7 — The public surface, drafted and parked

**Everything in this Part is a Proposal.** Under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md) a new reference entry point and a new driver type are each a public boundary. This record designs the surface and states its evidence; acceptance is Tom's, and [`accept-the-composed-realization-evaluation-surface`](../../../tickets/accept-the-composed-realization-evaluation-surface.md) carries it. Nothing is released on it.

**The constraint it must satisfy, inherited rather than rederived.** `tiler-reference` must not name a plan structure (Tom, 2026-08-06). So the reference gains only `ValueId`-keyed vocabulary, and everything that reads a cover or a topology sits in `tiler_ir` or above.

**Item A — the driver, sited where it can see both sides.** One entry point taking the retained selected candidate, the plan's witness sequence, and the declared input bindings, returning the expected output tensors or the first refusal. It performs its own pinning, so no caller holds the primitive of item B and the provenance property of Part 3 is structural rather than documentary. *Enables:* one call answering for a program spending both freedoms. *Prevents:* it does not make an unevaluable topology evaluable, and it does not close sites 3.2, 3.3, or the retention gap — those are absences a driver cannot invent.

**Item B — the reference primitive, `ValueId`-keyed and plain.** An evaluation that accepts a set of `(ValueId, &Tensor)` pins and a set of observed `ValueId`s, returning the declared outputs and the observed tensors. The pins generalize `InputBinding` (`crates/tiler-reference/src/tensor.rs:283-286`) from a declared input to any value; the observations expose values `evaluate` already computes and drops at `evaluate.rs:280-283`. Typed refusals: a pin naming a value that is not an operation result, an observation naming an unreachable value, and a pinned tensor whose shape or resolved type disagrees with the program's — each fail-closed and each watched firing. *Enables:* the chain, at the cost of one new signature and no new fold. *Prevents:* it must not be the public composition entry point, for the counterpoint Part 3 states.

**Item C — no change to the declared-order family, and the derivation for that is the finding.** `strict_partial_sums_under` and `strict_partitioned_sum_under` already take plain `u64` scalars beside a conformance and already state the normative meaning of a declared split. The survivor consumes them unchanged, which is what keeps the composition from adding a second definition of an order the corpus already defines once.

**What is deliberately not drafted.** No topology type reaches `tiler-reference`; no new exact evaluator is proposed; no plan-side field is proposed, because the cover already carries the stage atoms and the edges.

## The four-outcome roll-up

| Axis | Outcome class | What closes or discharges it |
| --- | --- | --- |
| Which object answers for a program spending both freedoms | **Correctness-derived answer, delivered** | C2 survives alone; C1 on identity, C3 on an accepted boundary, C4 on three grounds, C6 on subject, C5 as the fail-closed default it remains for the refusals |
| The boundary the composition needs | **Bounded research deliverable, delivered** | Part 1.3's theorem, with its refutation procedure and its fail-closed check as refusal 4 |
| The evaluated population and its refusals | **Bounded research deliverable, delivered** | Part 5: seven refusal classes, two of them this record's own and deliberately empty |
| The surface the survivor needs | **Parked for Tom under ADR 0075** | `accept-the-composed-realization-evaluation-surface` |
| The retention the survivor depends on | **Explicit dependency, already decided and not yet implemented** | The settled pointwise fork's design 1; refusal 1 is the interim answer |
| The one existing composition site | **Bounded implementation deliverable, identified** | Part 6; the change is the provenance of `pointwise_tensor`, and it belongs to whichever ticket lands the retention |
| The elaborated-program cross-check | **Retained in a weaker role, not filed** | Part 2.3; it has no caller until the survivor exists |

## What this record does not establish

- **No contract changed and no decision was made.** No permission admitted, no ADR proposed, no dimension added, no crate touched, no catalog edited.
- **Nothing here is measured on any device, and nothing here is compiled or run.** Every repository claim is a source reading at `b9146836` with a file and a line, and every absence claim carries the command that reproduces it. Part 4's table is exact binary32 arithmetic reproducible by the quoted command; it is evidence about what the objects answer, not about what any hardware does.
- **The survivor is derived and not built.** It depends on a retention design that is decided and unimplemented, so refusal 1's population is currently every member of the population the answer covers. A derivation whose mechanism does not exist is a derivation, and this record does not claim otherwise.
- **Part 2.3's second ground against C4 is stated as checkable and was not checked.** Whether a program's registry snapshot admits a `reindex-f32` occurrence it never used is answerable by building one; this record read the resolution path (`evaluate.rs:203-208`) rather than exercising it, and if the snapshot admits it, C4's elimination rests on its first and third grounds alone, which are independently sufficient.
- **The composition theorem is exhaustive over the constructors this build has and not over the future.** Part 1.3 states the refutation procedure and Part 5's refusal 4 is the check, so a ninth region constructor is classified by applying the rule rather than by extending a list.
- **The canonical-form claim the freedom-sites record's Part 5 leaves untested is untested still**, and the survivor does not rest on it: every stage's value is computed from `P'`, never from the minted expression.
