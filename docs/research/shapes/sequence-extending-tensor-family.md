---
schema: "tiler-doc/v1"
id: "tiler.research.shapes.sequence-extending-tensor-family"
kind: "research"
title: "Sequence-extending tensor family"
topics: ["semantics", "operation-families", "shapes", "extents", "indexing", "kv-cache", "language-model", "state"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.ir"]
depends_on: ["tiler.research.shapes.transformer-operation-and-shape-surface", "tiler.research.program-planning.first-attention-program-vertical"]
ticket: "scope-the-sequence-extending-tensor-family"
---

# Sequence-extending tensor family

**Status:** durable scoping record for the one operation every autoregressive decode step performs twice per layer. It is a research outcome, not a capability: nothing here registers an operation, admits a key, fixes a normative contract, or authorizes implementation. It moves no support-matrix row, and the two rows it adds record an absence that already existed.

## Traceability

- **Work record:** [`scope-the-sequence-extending-tensor-family`](../../../tickets/scope-the-sequence-extending-tensor-family.md).
- **Position:** beneath rung L5 of [the roadmap's language-model ladder](../../roadmap.md#the-ladder). It is not itself a rung. [The L2 derivation](transformer-operation-and-shape-surface.md) found the requirement and declined to settle it; [the L4 program design](../program-planning/first-attention-program-vertical.md) named the seam — `k_rope` and `v_heads` as retained program outputs, with `S` a separate extent symbol from `T` — and chose nothing; [`design-autoregressive-state-and-kv-cache`](../../../tickets/design-autoregressive-state-and-kv-cache.md) at L5 owns the state model and inherits this record's result.
- **Governing contracts read as evidence, not edited:** [IR stack and invariants](../../ir.md) for operation identity and effect, the graph verifier's purity rule, the `Reindex` and `Broadcast` forms, the first out-of-place access profile, the index-expression vocabulary, the alias contract, and the `ShapeEnv` boundary; [Q-PLAN-015](../../open-questions.md#q-plan-015--advanced-buffer-reuse-and-in-place-execution) for the in-place deferral and [Q-SHAPE-006](../../open-questions.md#q-shape-006--finite-piecewise-access-maps) for piecewise access maps; the [operation-family support matrix](../../roadmap.md#operation-family-support-matrix) for the rungs.
- **Inspected source, at this branch's base commit `96f7285`:** `crates/tiler-ir/src/semantic/{operation.rs,registry.rs,quantization.rs}`, `crates/tiler-ir/src/shape.rs`, `crates/tiler-ir/src/shape/env.rs`, `crates/tiler-ir/src/shape/env/constraint.rs`, `crates/tiler-ir/src/index/{model.rs,builder.rs,sourced.rs}`, `crates/tiler-ir/src/program/{model.rs,builder.rs,verify.rs}`, `crates/tiler-ir/src/program/abi.rs`.

Claims are labelled **Fact** when traced to inspected source or to a merged record at a named revision, **Inference** when derived from stated facts, and **Proposal** when not yet accepted or tested. This record contains no measurements; it takes none. Every byte figure below is arithmetic over figures L1 and L4 already state, and is labelled **Inference** for that reason.

## The requirement, restated exactly

**Fact — from the L2 derivation.** One forward pass of the pinned `Qwen/Qwen3-0.6B-Base` profile performs 56 sequence-extending state writes — one for `K` and one for `V` in each of 28 layers. Each extends a cached tensor of shape `[8, C, 128]` F32 by `T` new positions of shape `[8, T, 128]`, giving `[8, S, 128]` with `S = C + T`. `S` is bounded at 18 for the C1 conformance row and 8,320 for the B1 benchmark matrix, and one cached token costs 229,376 F32 bytes across the whole model.

**Fact — the extension is at the block boundary, not inside it.** L4's program names `k_rope` (`[8, S, 128]`) and `v_heads` (`[8, S, 128]`) as retained outputs o1 and o2 beside the residual stream, and states that a decode step is that program with `K` and `V` arriving as inputs of extent `S ≥ T` instead of being produced. So the extension joins two values that cross the program boundary; it is not an intermediate the optimizer may reorganize away.

**Inference — two mechanisms, and they are not two implementations of one design.** Either a semantic operation consumes the cached tensor and the new rows and produces a new value of the larger extent, or a windowed write places the new rows into a preallocated buffer whose valid range is state. The first is a value; the second is a mutation. They differ in identity, in purity, in what a verifier must prove about the write, and in whether the growing axis is an extent symbol or a runtime-tracked cursor. This record derives what each owes, eliminates between them, and hands the residue to L5.

## What each mechanism owes

| Obligation | Mechanism A — semantic `Concatenate` | Mechanism B — windowed write into preallocated state |
| --- | --- | --- |
| Identity | An `OpKey` with a canonical axis attribute, ordered operands whose order is semantic, and one result. Nothing new: `OperationArity::inclusive` already expresses a bounded variadic operand arity, and `StrictSerialSumF32::infer` is the precedent for validating an axis attribute at construction. | Identity of a *resource*, its version, and the ordering edge that makes one write visible to the next read. The semantic graph has no such subject: [IR](../../ir.md) states that all initial semantic values are tensors and that resource or effect-token value kinds are reserved for a later effect model. |
| Validation | Equal rank, equal resolved value type, and equal extents on every axis except the concatenated one, all of which the accepted three-outcome shape path already covers; plus the result extent, which is the sum. Zero-extent operands need an explicit admitted-or-refused rule. | Window offset within capacity, disjointness from the retained valid range, and monotone growth. These are facts about mutable state across executions, not shape facts about one program, and no layer holds them. |
| Purity or effect declaration | `Pure`, unchanged. | Not pure. `OperationEffect` has exactly one variant and is deliberately not `#[non_exhaustive]`, so a second effect class is a compile error at three encoders — the mechanism that makes mutation unrepresentable rather than merely unimplemented. Widening it is necessary and, per the support matrix's own effectful row, not sufficient. |
| Access relation | Either one write root over the whole output with a **piecewise read** selecting per coordinate which operand supplies it — [Q-SHAPE-006](../../open-questions.md#q-shape-006--finite-piecewise-access-maps), plus a predicated selection the registry cannot type because it admits no boolean dtype — or **two write roots partitioning one output**, which [IR](../../ir.md) defers by name: "In-place/read-modify-write relations, output partitions, atomics, and other reduction organizations require later specialized contracts rather than implicit relaxation." | A partial write root over an output boundary the region does not initialize, which is the same output-partition relaxation, plus the in-place relation named in the same sentence, plus [Q-PLAN-015](../../open-questions.md#q-plan-015--advanced-buffer-reuse-and-in-place-execution), plus the alias contract's rule that an output must alias no input. |
| Write-ownership proof | Both implemented forms — `WriteOwnershipProof::{CoordinatePermutation, Exhaustive}` — prove that one access is total and injective over its own declared boundary. Neither expresses "total over a partition and disjoint from a sibling partition", so the partitioned form owes a third proof kind and a joint-coverage obligation across roots. | The same third proof kind, and then a second obligation the first does not have: the untouched range's contents were written by a *previous execution*, so its validity is not a region-local property at all and no verifier in the stack has the subject. |
| Extent symbol of a growing axis | `C` and `T` bind to input dimensions; `S` is their sum. **That sum has no representation.** `ExtentRelation` admits `Equal`, `Divisible`, `NonNegativeDifference`, `Interval`, and `Factorization` over an `ExtentTerm` that is a symbol or a constant and, in its own words, "deliberately not an arbitrary expression tree" — so `S == C + T` cannot be stated, and `S` cannot be a derived extent because `SourcedExtent` is static-or-symbol and every symbol needs exactly one root binding. | The same gap in a different spelling: the capacity invariant is `C + T <= capacity`, a three-term additive relation the same fragment cannot state. It gains nothing on the availability axis either, because growth here is host-driven — `S` is known at or before `LiveDevicePreflight` under both mechanisms. |

**Inference — the two rows that matter are purity and access relation, and they point opposite ways.** On access relation and write ownership the two mechanisms owe *the same* new contract, because both write part of something and neither can prove the rest. On purity, extent representation, and resource identity, B owes strictly more. There is no obligation A owes that B does not, which is what makes the elimination below a derivation rather than a preference.

## Reproducible checks

Each check is one command from the repository root, with the positive control that proves it can return something. A check whose population it cannot name is the shape this repository is elsewhere required to distrust.

```sh
# 1. No concatenation or slice key is registered. Read the two registration
#    functions rather than counting: StandardSemantics::register constructs four
#    F32 operation definitions and register_standard_quantization three, and none
#    of the seven is a concatenation or a selection.
rg -n 'register_operation\(' crates/tiler-ir/src/semantic/registry.rs crates/tiler-ir/src/semantic/quantization.rs
#    Positive control: the same read finds strict-serial-sum-f32, so the
#    enumeration is not empty and a missing key would be visible as an absence
#    from a list that has members.

# 2. The constraint fragment admits no additive relation between two terms.
grep -n 'pub enum ExtentRelation' -A 45 crates/tiler-ir/src/shape/env/constraint.rs
grep -n 'pub enum ExtentTerm' -A 8 crates/tiler-ir/src/shape/env/constraint.rs
#    Positive control: the same read finds NonNegativeDifference, the nearest
#    additive-looking relation, which constrains a difference's sign rather than
#    defining a sum — so the search does reach the relations it would have to
#    miss for the claim to be wrong.

# 3. No coordinate expression carries an extent symbol. IndexNode has five
#    variants; SourcedExtent appears only as a FloorDiv or Modulo divisor, and
#    LinearCombination's constant is a literal IndexInteger.
grep -n 'enum IndexNode' -A 25 crates/tiler-ir/src/index/model.rs
#    Positive control: that same read does find SourcedExtent, in the divisor
#    position, so a search for a symbol-carrying node is not returning nothing.

# 4. One value has at most one writer, and an externally bound input is never
#    written. `definitions` returns MultipleWriters, ExternalValueWritten, and
#    MissingWriter.
grep -n 'fn definitions' -A 35 crates/tiler-ir/src/program/verify.rs
#    Positive control: the same function admits a ValueRole::Output with exactly
#    one writer, which is the case the crate's own two-stage program exercises,
#    so the rules are reachable rather than vacuous.

# 5. `ShapeExpr` is a contract-level name with no implementation.
grep -rn 'ShapeExpr' crates/
#    This returns no output. Positive control: `grep -rln 'ShapeSymbol' crates/`
#    returns nine files, so the tree is searchable and the empty result above is
#    an absence rather than a broken invocation.
```

## The elimination

**Fact — three candidates, not two.** The ticket frames a binary choice; the option space also contains the case where the extension is not in the compiled program at all, and eliminating it is part of the work.

| Candidate | Survives? | Ground |
| --- | --- | --- |
| **A — a semantic `Concatenate` producing a new larger value** | **Yes.** It is the only candidate expressible under the implemented effect model, and its residue is a physical contract rather than a semantic one. | Pure, so the graph stays acyclic tensor SSA; identity, arity, and axis validation need no new mechanism; the growing axis stays an extent symbol bound per execution. What it does not have is a lowering, and what it owes there — a partitioned or piecewise write — it owes in common with B. |
| **B — a windowed write into preallocated state** | **No.** | It owes everything A owes and then four more things, each of which is a reserved architectural decision rather than an implementation gap: a second `OperationEffect` class, resource or effect-token value kinds, in-place execution under Q-PLAN-015, and a relaxation of the alias contract that requires an output aliasing no input. Its only advantage over A is that it does not copy, which is a physical argument about a semantic question — and it is not even a durable advantage, because A's copy-free realization is the same windowed binding B would need. |
| **C — the extension happens outside the compiled program, in the runtime or the consumer** | **No.** | Expressible today, and that is exactly the problem. It moves a required data movement outside identity, cost, explain, and the verifiers: a consumer that blits at the wrong offset returns a plausible tensor and nothing refuses it. [IR](../../ir.md) requires fail-closed typed errors and [the optimizer contract](../../compiler/optimizer.md) requires explain output for what a plan does; a silent movement satisfies neither. It also splits the block at every layer, because the new rows are produced by the same program that must then read the extended tensor — 56 program boundaries per decode token, each with its own routing commit. |

**Inference — the strongest argument for B is residency, and it does not carry.** Under a naive realization of A the whole cache is copied every decode step: at the B1-d row that is 1,908,408,320 bytes read and written per token, 1.60× the 2,384,199,680-byte F32 weight traffic of the model itself, and it doubles peak cache residency because the new value is a second allocation. At the C1 row the same copy is 4,128,768 bytes, about 0.35% of one token's weight traffic. So the copy is irrelevant where the conformance evidence lives and dominant where the benchmark lives — the same spread L4 found for the score tensors — and a decision taken on C1 evidence would be taken where the difference cannot be observed.

**Inference — that argument is about a plan, and the plan it argues for is available under A.** An ABI binding already addresses a byte window rather than a whole value: the binding row carries `accessible_offset` beside `accessible_bytes`, both derived from the packaged program's own `ByteWindow`, folded into artifact identity, re-proven at decode, published by the loader, and applied by the Metal proof host at the binding call. `KernelProgramBuilder::push_view` admits any window inside a value, and `check_stage_accesses` requires only that the kernel's addressed byte count equal the window's length — so a kernel writing `[8, T, 128]` into a window of an `[8, S, 128]` value is already representable. What refuses it today is not the ABI but whole-program verification: `MultipleWriters` forbids a second stage writing the rest, `ExternalValueWritten` forbids writing a caller-bound input, and nothing proves that the untouched bytes of a partially written value hold anything. **Those three are the physical work, and B does not avoid one of them.**

**Inference — so the elimination is decided rather than deferred, and it is decided at the layer that owns the question.** Extending a tensor along an axis *means* forming a value; that a physical plan may realize the forming as a windowed write into a retained allocation is a placement, aliasing, and lifetime contract, which the architectural contract already holds as explicit physical contracts rather than node annotations. Choosing B would settle what the program means by appealing to how a device would execute it, and would pay for that with the purity invariant the whole optimization stack rests on. The ticket's reconsideration trigger anticipated that the honest outcome might be enumeration with the mechanism inherited by L5; it is not, because the elimination closes on obligations that are asymmetric rather than on evidence this record lacks.

**Proposal — the disposition, in L2's vocabulary.** *Sequence extension is **atomic**: one semantic operation family producing a new value.* This is a research disposition of the same kind L2 recorded for softmax and RMS normalization, not an accepted decision; admitting a key remains a delivery ticket, and the public boundary of any such key remains Tom's.

## Should a general `Concatenate` and a general `Slice` exist at all?

**Fact — L2's claim that the workload needs neither inside a layer is verified, and it has bit evidence rather than only a reading.** `rotate_half` reduces to a bijective split of the 128-wide head axis into `(2, 64)`, a coordinate swap on the size-2 axis, a broadcast multiply by a two-element sign tensor, and a merge. L4 measured that composition against the pinned reference on a `[1, 16, 10, 128]` operand: 0 of 20,480 elements differ, while dropping the swap or reversing the sign operand differs at all 20,480. The head splits and merges are bijective reshapes, the grouped-query repetition is free under ADR 0087's structure-carrying contraction, and the causal mask is a host-built additive input.

**Fact — and no pinned row needs a slice either.** The C1 conformance row retains every logit: L1 records that the pinned reference turns `logits_to_keep=0` into `slice(0, None)`, so the value that reads like "keep none" keeps all, which is why the row can retain prefill logits at all. Nothing in the workload selects a sub-tensor.

**Fact — the one qualification this claim carried is discharged, so the result is unconditional.** When this record was written, [IR](../../ir.md) spelled `Reindex`'s initial forms as "bijective permutations/split/merge mappings or legal removal/insertion of unit axes", which read most naturally as a permutation *of axes*, while the rotary swap is a bijective permutation of coordinates *within* one axis; had that form not been admitted, `rotate_half` would have needed a structural form the corpus lacks, and one of its candidate spellings is exactly a slice and a concatenate. That was L4's D-10, and it **closed on 2026-07-31 in favour of admitting the form**: `tiler::reindex-f32@1`'s registered normative definition admits the swap as its `reverse-axis` form, "the within-axis coordinate map `i -> extent - 1 - i`", and admits no other within-axis permutation — one presented under any other name is refused as `reindex.form.unadmitted-kind`, because the affine within-axis bijections of an axis are exactly the identity and the reversal while a general within-axis permutation is a tensor-data-derived index the accepted index vocabulary rejects. So "no slice and no concatenate inside a layer" holds without a qualification: `rotate_half` is expressible over admitted families, and the slice-plus-concatenate spelling that would have put two contract-less families in place of one admitted form is not needed.

**Proposal — one general `Concatenate` family, not a narrow sequence-extend key.** A key that fixed the axis at zero and the arity at two would owe exactly the obligations the general form owes: the same extent agreement on the non-concatenated axes, the same additive result extent, the same partitioned write, the same ownership proof. Specializing buys nothing and guarantees a second family later. What *is* axis-dependent is the cheap physical realization: a contiguous byte window exists only for the slowest-varying axis under a row-major layout, so a concatenation along an inner axis writes a strided destination and cannot use it. That belongs in an applicability predicate over a physical candidate, not in a second semantic identity — which is precisely the separation the architectural contract requires.

**Proposal — no `Slice` family now, with a stated trigger.** No pinned row needs one, and admitting a family with no occurrence would be a contract written from imagination. The trigger is a prefill pass that needs only the final position's logits: projecting all `T` positions at the B1-d row costs 4,978,634,752 F32 bytes against 607,744 for one position, so the crossover is real and reachable rather than hypothetical, exactly as L2 said of the causal mask.

## Typed refusals this family owes

Each is a place where a silent approximation would return a plausible tensor.

- A concatenation whose operands disagree on any axis other than the concatenated one refuses at construction, naming the axis and both observed extents, through the accepted three-outcome shape path rather than a shape comparison invented for this family.
- A concatenation whose operands' resolved value types differ refuses; the family grants no promotion, no weak-scalar rule, and no dtype permission, exactly as the binary elementwise signatures do not.
- An occurrence whose result extent cannot be related to its operands' extents refuses rather than binding a fresh unconstrained symbol. Until the constraint fragment can state an additive relation, this refusal is the whole of the family's extent handling, and a program that bound `S` independently of `C` and `T` would verify while meaning something else.
- A plan that writes part of an output boundary without a proof covering the rest refuses. This is the refusal that does not exist today: `MultipleWriters` refuses the second writer, but a single partial writer passes, and nothing proves the remainder.
- A plan that writes an externally bound input refuses, which `ExternalValueWritten` already does. If L5 relaxes it for a runtime-owned cache, the relaxation is a named contract with its own identity, never a widening of the existing input role.
- A zero-extent operand behaves as the family's normative definition states, and states it explicitly, rather than inheriting whatever the empty case happens to do.

## What this record does not decide

- **The KV-state model.** L5's: capacity, valid range, growth policy, placement, aliasing, retention, and lifetime. This record hands it a named semantic mechanism instead of an open one, and hands it the three implemented refusals its physical realization must address.
- **The physical realization of the append.** Whether the copy is elided by a windowed binding into a retained allocation, by allocation reuse, or not at all, and whether that requires relaxing `MultipleWriters`, `ExternalValueWritten`, or both. The evidence here is that the ABI and loader already carry a binding offset end to end, and that whole-program verification is what refuses the plan.
- **The additive extent relation — settled outside this record.** Tom accepted the fixed two-addend `ExtentRelation::AdditiveEquality` boundary on 2026-08-03. `SourcedExtent` remains static-or-one-symbol and no general expression tree was introduced; invocation-time consumption remains the runtime-preflight consumer's work.
- **Whether the concatenation key is one family or several, and its exact spelling.** A public operation boundary is Tom's, and a research disposition is not an accepted interface.
- **Any cost claim.** The byte figures above are arithmetic over L1's and L4's stated quantities; no schedule for a concatenation has been measured at any shape, and the copy-free realization has not been measured either.
