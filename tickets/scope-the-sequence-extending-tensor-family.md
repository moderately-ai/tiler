---
id: scope-the-sequence-extending-tensor-family
title: Scope the sequence-extending tensor family the KV cache needs
status: todo
priority: p2
dependencies: [derive-transformer-operation-and-shape-surface]
related: [design-autoregressive-state-and-kv-cache, own-operation-family-support-matrix]
scopes: [contracts/foundation, contracts/navigation, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [research, semantics, structural, language-model, breadth]
---
## User-visible outcome

The corpus says what it means to extend a tensor along one axis — the operation every autoregressive decode step performs twice per layer — instead of leaving it in the one position no ledger records: absent from the support matrix, absent from the normative contracts, and absent from the ticket graph.

## Evidence prerequisite

**Fact — neither candidate mechanism exists, and the absence is unrecorded.** A tensor `Concatenate` has no row on the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix), no normative semantics in [`docs/ir.md`](../docs/ir.md), and no registered key; it is the only family this workload touches that is not even enumerated as absent. The alternative — an in-place windowed write into a preallocated cache buffer — is excluded by the implemented profile: `docs/ir.md` states that "input boundaries may be read but not written, output boundaries may be written but not read, and every declared output boundary requires exactly one complete ordinary write root", and [Q-PLAN-015](../docs/open-questions.md#q-plan-015--advanced-buffer-reuse-and-in-place-execution) defers in-place execution.

**Fact — `Reindex` does not reach it.** `docs/ir.md` admits `Reindex` as bijective permutations, splits, merges, and unit-axis insertion or removal. A concatenate is multi-operand with an output partitioned by operand, which is outside those forms and outside `Broadcast`.

**Fact — the workload evidence, from the L2 derivation.** [The transformer operation and shape surface derivation](../docs/research/shapes/transformer-operation-and-shape-surface.md) counts 56 sequence-extending state writes per forward pass of the pinned `Qwen/Qwen3-0.6B-Base` profile — one for `K` and one for `V` in each of 28 layers. Each appends `T` positions of shape `[T, 8, 128]` F32 to a cache holding `S - T` positions, with `S` bounded at 18 for the conformance row and 8,320 for the benchmark matrix. L1 records the arithmetic: 229,376 F32 bytes per cached token across the whole model.

**Inference — the two mechanisms are not implementations of one design.** A `Concatenate` produces a new value of a larger extent and leaves the physical planner to decide whether that is a copy; a windowed write mutates a buffer whose valid range is state. They differ in semantic identity, in whether the operation is pure, in what the index verifier must prove about write ownership, and in whether the growing extent is a shape symbol or a runtime-tracked capacity. Choosing between them by whichever is easier to schedule would settle a semantic question with a physical argument.

## Required analysis

- State what each mechanism would owe: identity, validation, purity or effect declaration, access relation, write-ownership proof, and the extent-symbol treatment of a growing axis.
- Decide between them, or state exactly what evidence would decide it, running the elimination explicitly rather than presenting two options.
- Record whether a general `Concatenate` and a general `Slice` are wanted at all, since the same derivation shows the workload needs neither inside a layer: the rotary half-split reduces to a bijective split, a permutation, a broadcast multiply, and a merge.
- Add the resulting rows to the support matrix with their rungs and triggers, so the absence is tracked whichever way the decision goes.

## Non-goals

The KV-state model itself — capacity, valid range, growth policy, placement, aliasing, retention, and lifetime — which [`design-autoregressive-state-and-kv-cache`](design-autoregressive-state-and-kv-cache.md) owns at rung L5. This ticket settles the *semantic family* that state model will invoke, and hands it a named mechanism instead of an open one. It implements nothing.

## Reconsideration trigger

Active now for the matrix rows, which record an absence that exists today regardless of the decision. The mechanism decision may reasonably wait for L5 to state its state model, in which case this ticket narrows to the enumeration and L5 inherits the choice with the analysis already done — but it may not stay unrecorded, because an unenumerated family is invisible to every reader who checks the matrix.

## Outcome

The durable record is [Sequence-extending tensor family](../docs/research/shapes/sequence-extending-tensor-family.md), filed under `docs/research/shapes/` beside the L2 derivation that raised the requirement and indexed in the research catalog. **The home was chosen by comparison rather than by default:** `decide-whether-storage-encoding-is-a-missing-boundary-property` kept a one-axis contract decision entirely in its Outcome, and `decide-whether-a-contraction-is-one-keyed-family-or-fixed-arity-keys` put its analysis in an accepted ADR; this ticket has neither shape. It carries a six-obligation comparison across two mechanisms, five reproducible checks, a three-candidate elimination, and a family-wantedness result that L5, `admit-the-reindex-and-broadcast-operation-families`, and two support-matrix rows all cite — and its scopes admit no `contracts/decisions`, so an ADR was not available. A cited, linkable record is what the corpus uses for that, and a ticket Outcome is not reachable from a matrix cell without pointing a normative document at workflow state.

**Decided — the mechanism is the value-producing one, and the elimination closes on obligations rather than on missing evidence.** Mechanism B, the windowed write into preallocated state, owes everything mechanism A owes and then four reserved architectural decisions on top: a second `OperationEffect` class, resource or effect-token value kinds, in-place execution under Q-PLAN-015, and a relaxation of the alias contract requiring an output to alias no input. There is no obligation A owes that B does not. B's only advantage is that it does not copy — a physical argument about a semantic question, and not a durable one, because A's copy-free realization is the same windowed binding B would need.

**The two mechanisms owe the same thing at the layer that actually blocks them, which is the finding that decides it.** Both write part of something and neither can prove the rest. `WriteOwnershipProof::{CoordinatePermutation, Exhaustive}` prove one access total and injective over its own declared boundary; neither expresses "total over a partition and disjoint from a sibling", so both owe a third proof kind. The elimination therefore costs nothing physically and buys the purity invariant.

**Three implemented refusals were traced, and none of them is where the ticket expected the wall.** An ABI binding already addresses a byte window end to end — `accessible_offset` beside `accessible_bytes`, derived from the program's own `ByteWindow`, folded into identity, re-proven at decode, published by the loader, applied at the Metal binding call — and `KernelProgramBuilder::push_view` admits any window inside a value while `check_stage_accesses` requires only that the kernel's addressed byte count equal the window's length. What refuses a windowed append is whole-program verification: `MultipleWriters` refuses a second writer, `ExternalValueWritten` refuses writing a caller-bound input, and nothing proves the untouched bytes of a partially written value. Those three are L5's to address, and the record hands them over by name.

**A fourth gap binds both mechanisms and was not previously recorded anywhere.** `ExtentRelation` admits `Equal`, `Divisible`, `NonNegativeDifference`, `Interval`, and `Factorization` over an `ExtentTerm` that is symbol-or-constant and, in its own doc comment, "deliberately not an arbitrary expression tree" — so the family's defining equality `S == C + T` cannot be stated, and `S` cannot be a derived extent because `SourcedExtent` is static-or-symbol and every symbol needs exactly one root binding. Mechanism B needs the same additive relation as `C + T <= capacity`. **A neighbouring L2 sentence is refined rather than corrected:** L2 inferred from `docs/ir.md` that the append's symbolic offset "is already admitted", which is true of the contract's stated vocabulary and false of the implemented profile — `IndexNode` has five variants, `SourcedExtent` appears only as a `FloorDiv` or `Modulo` divisor, and `LinearCombination`'s constant is a literal `IndexInteger`, so no coordinate expression carries an extent symbol.

**A general `Concatenate` is wanted; a general `Slice` is not.** L2's claim that the workload needs neither inside a layer was verified rather than trusted, and it holds with one qualification: it is conditional on L4's D-10, because if `Reindex` does not admit a within-axis coordinate permutation then `rotate_half` needs a structural form the corpus lacks and slice-plus-concatenate is one candidate spelling. The recommendation is one general concatenation family rather than a narrow sequence-extend key, because a fixed-axis fixed-arity key owes exactly the same extent agreement, additive result extent, partitioned write, and ownership proof — while what *is* axis-dependent, the contiguous byte window, belongs in a physical applicability predicate. `Slice` gets a row and a trigger instead of a family: a prefill pass needing only the final position's logits, which otherwise costs 4,978,634,752 F32 bytes at B1-d against 607,744 for one position.

**Two matrix rows added, both R1**, placed with the structural families and before contraction: sequence extension and sub-tensor selection, each with its evidence and its trigger. Nothing moved rung — a research record is not a normative contract, and the rows record an absence that already existed. One bullet was added to the ladder's prerequisite list so an L5 reader finds the family where they look for it.

**One adjacent stale `Fact` corrected while reading for this analysis.** The effectful row asserted that `OperationEffect` is `#[non_exhaustive]`; the declaration says the opposite, deliberately, because three out-of-crate encoders map the vocabulary totally onto an identity tag. The rung is unaffected and the real property is stronger than the cell claimed.

**One scope added, not silently.** `research/shapes` joins the declared scopes because the durable record lands in `docs/research/shapes/`, which `tkt guard` maps to that scope and which the ticket's original three did not cover; `derive-transformer-operation-and-shape-surface` declared the same scope for the sibling record in the same directory. No open ticket declares it, so the addition creates no contention. Nothing else in the change moved outside the original three.

**Checks.** Every absence claim in the record carries its exact command and a positive control that proves the command can return something — including `grep -rn 'ShapeExpr' crates/`, which returns nothing against a control returning nine files.

**Deliberately not done.** No `docs/ir.md` edit: the analysis needed no reservation in normative text, and the record is the right home for a disposition that is not yet a contract. No key, no ADR, no capability ticket — L5 already owns the state model, and filing the additive-extent-relation gap as its own ticket would duplicate a constraint the record hands to the contract work that will need it.
