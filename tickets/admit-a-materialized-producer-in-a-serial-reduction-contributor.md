---
id: admit-a-materialized-producer-in-a-serial-reduction-contributor
title: Admit a materialized producer in a serial-reduction contributor
status: in-progress
priority: p3
dependencies: [name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set]
related: [name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set, admit-a-recognized-chain-more-than-one-materialization-boundary-deep]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [research, compiler, planner, identity]
claimed_from: todo
assignee: worker-materialized-producer
lease_expires_at: 1786664925
---
## User-visible outcome

A strict serial reduction can consume contributors computed across a materialization boundary, such as `sum(sum(x) * 2)` or `sum(contract(a, b) * 2)`, without flattening away the producer or changing the program's numerical meaning.

## Facts at filing — 2026-08-12, base `0a67f558`

**Fact — recognition finds the boundary and discards it.** At the cited base, `plan_elementwise` returns `ElementwiseRefusal::Folded(ValueId)` when a serial reduction's contributor walk reaches a strict reduction, contraction, or registered staged family, and `recognize_reduction` reaches that result only through `recognize_elementwise`. The paired accepted diagnostic ticket changes the conversion from the stale `operation-set` classification to `reduction-contributor-materialization`; it does not retain the producer.

**Fact — this is not the staged-family depth guard.** `StagedOperandAdmission::NoEdge` governs a staged family reached across a materialization edge. The serial-reduction path never consults that guard; it fails earlier because `NormalizedSerialSum` carries an optional pointwise expression but no producer relation. `admit-a-recognized-chain-more-than-one-materialization-boundary-deep` remains a distinct deferred boundary.

**Fact — several producer families expose one missing relation.** `materializes_its_result` recognizes strict serial reduction, strict tensor contraction, and every registered region-sequence family. Admission must model one serial-reduction contributor supplied by a materialized producer rather than specialize the normal form around a producer key.

**Fact — the existing accepted neighbor is shallower.** A reduction over a pointwise expression of declared inputs already compiles because its contributor expression can be retained directly as the optional prologue. An elementwise epilogue over a materialized producer also compiles because `NormalizedEpilogue` carries a producer. Neither shape supplies the missing serial-reduction producer field.

## Research and decision required

Before implementation, derive and compare the smallest exact normal form that retains the producing `NormalizedOutput`, the elementwise contributor continuation, their materialization edge, occurrence partition, numerical materialization boundary, subject encoding, cover formation, and physical/KIR consequences. Audit recursion and deterministic work bounds: the current producer forms are recursive, and accepting caller-proportional nesting without an iterative or explicitly bounded representation would turn program depth into host-stack risk.

The design must preserve the producer's own semantic and numerical identity, prove producer-before-consumer ordering, and refuse unsupported producer/continuation combinations by name. It must never synthesize a declared-input baseline, flatten the boundary, reuse a nearby pointwise prologue, or fall back after a failed admission.

## Non-goals

Producer-specific diagnostic keys, widening a staged family to read two materialization edges, arbitrary chain depth, backend emission, or performance selection.

## Activation and closure

Move this ticket to `awaiting-decision` only after the complete construction and consumption census identifies a bounded, injective carrier and its identity/schema consequences. Close only when at least the nested-reduction, contraction, and staged-family subjects are either admitted through that shared carrier or each refused under a narrower named prerequisite, with the declared-input neighbor unchanged.

## Current-base Fact audit — 2026-08-13, base `4275c14bb3c5fb1d73f8ae41cdc803d871742481`

Filing Facts were written against `0a67f558`. Re-read in full at this base. Commands were run in this worktree at `HEAD = 4275c14bb3c5fb1d73f8ae41cdc803d871742481`.

### Fact 1 — recognition finds the boundary and discards it

**Verified**, with one precision: the diagnostic key change has already landed; the discard remains.

- `plan_elementwise` still returns `ElementwiseRefusal::Folded(value)` when `leaves.staged.is_none() && materializes_its_result(&operation, laws)`. Command: `rg -n 'ElementwiseRefusal::Folded' crates/tiler-compiler/src/request.rs`.
- `recognize_elementwise` has one production caller, `recognize_reduction`. Command: `rg -n 'recognize_elementwise\(' crates/tiler-compiler/src/request.rs` — definition plus the one call inside `recognize_reduction`.
- `recognize_elementwise` maps `Folded` through `RequestError::from`. That `From` impl is still the only flattening path and still reports `rule: "reduction-contributor-materialization"`. Anchor: `Flattens a discovered materialization boundary into the rule a caller`.
- `recognize_elementwise_output` is the other `Folded` consumer and *retains* the value as `NormalizedEpilogue`. The reduction path never reaches that arm.

### Fact 2 — this is not the staged-family depth guard

**Verified.**

- The only `StagedOperandAdmission::NoEdge` construction is inside `recognize_epilogue_producer`. Command: `rg -n 'StagedOperandAdmission::NoEdge' crates/tiler-compiler`.
- `NormalizedSerialSum` still has `prologue`, `prologue_reads`, and `contributor_input`, and no `producer` field. The two `producer` fields in `request.rs` belong to `NormalizedEpilogue` and `NormalizedStaged`. Command: `rg -n 'pub(crate) producer' crates/tiler-compiler/src/request.rs`.
- `StagedOperandAdmission` still states that `recognize_reduction`'s contributor walk "reads declared inputs by construction" and names this ticket's flattening as the neighbouring structural wall. That sentence is true today and becomes false the moment `recognize_reduction` retains a `Folded` finding.
- [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`](admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md) remains `deferred` and still owns `staged-operand-depth` only.

### Fact 3 — several producer families expose one missing relation

**Verified.** `materializes_its_result` is still exactly `strict_serial_sum_f32_op()`, `strict_tensor_contraction_f32_op()`, or `laws.family_realizes_region_sequence`. Command: `rg -n -A 8 'fn materializes_its_result' crates/tiler-compiler/src/request.rs`.

The in-crate control population is still three subjects sharing one key: `folded_prologue(true)` (`sum(sum(x)*2)`), `staged_contributor` (`sum(rms_norm(value, weight))`), and the wall-file / `contraction_direct_path` contraction subjects. The declared-input neighbour `folded_prologue(false)` still recognizes as `NormalizedOutput::SerialSum`.

### Fact 4 — the existing accepted neighbor is shallower

**Verified.** `NormalizedSerialSum::prologue` is still `Option<PointwiseF32Expression>` over declared-input reads. `NormalizedEpilogue::producer` is still `Box<NormalizedOutput>`. `NormalizedStaged::producer` is the same relation for a staged operand. None of those is a serial-reduction contributor supplied by a materialized producer.

### Related claim that is false at this base

`materialized_intermediate_epilogue_wall.rs`'s `nested_contraction_chain` comment (`The admission is one materialization boundary wide, because TensorRole::Intermediate carries no ordinal`) classifies `sum(contract(a, b) * 2.0)` as a *width* problem — one region reading two staged values. That is the `leaves.staged.is_none()` / `admit-a-scheduled-region-that-reads-two-materialization-edges` rule, and this program does not hit it. `plan_elementwise` sees `leaves.staged.is_none() == true`, returns `Folded`, and `From<ElementwiseRefusal>` discards the producer. The program needs a producer region, an optional continuation region, and a fold region, each reading at most one `TensorRole::Intermediate`. Repairing that comment is not this ticket's work; implementers must not inherit the width classification.

## Construction and consumption census — 2026-08-13

### Construction

| Site | What it does today | What a carrier must change |
| --- | --- | --- |
| `recognize_output` | Root `strict-serial-sum-f32@1` → `recognize_reduction` | Pass a one-edge contributor admission |
| `recognize_reduction` | Always runs `recognize_elementwise` on the contributor; `Folded` becomes `reduction-contributor-materialization` | On `Folded`, re-plan the contributor with that value as the staged leaf (the `recognize_epilogue` numbering) and retain `recognize_epilogue_producer`'s result |
| `plan_elementwise` | Discovers the edge; does not retain it | Unchanged |
| `recognize_elementwise` | Only caller of the flattening `From` | Stop being the only reduction path, or stop flattening when admission is `OneEdge` |
| `recognize_epilogue_producer` | Recognizes the three `materializes_its_result` families; hands `NoEdge` to staged | Reused as the producer recognizer; its reduction arm must *not* open a second contributor edge |
| `recognize_elementwise_output` / `recognize_epilogue` | Inverse neighbor: elementwise root over a fold | Unchanged |

`recognize_reduction` has exactly two callers: `recognize_output` and `recognize_epilogue_producer`. Command: `rg -n 'recognize_reduction\(' crates/tiler-compiler/src/request.rs`.

### Validation and refusal

- `check_output_cover` partitions occurrences via `NormalizedOutput::members`. A producer whose occurrences are not claimed here is `operation-set`.
- `From<ElementwiseRefusal>` is the current named refusal. After a carrier exists it must fire only when admission is closed, never as a fallback after a failed retain.
- Neighbouring refusals that must stay distinct: `operation-set` (unrecognized family), `staged-operand-depth` (staged occurrence already across an edge), `contraction-operands` (contraction operand is not a declared input), `structural-access-conflict` / the `leaves.staged.is_none()` width guard (second folded value in one walk), `input-rank` (rank-zero contributor).
- `tiler-ir`'s `ContributorTensor::DeclaredDomain` already admits `TensorRole::Intermediate` or a declared input. A fold reading a materialized producer is already a legal scheduled region. Anchor: `The fold's declared contributor domain, wherever the plan placed it`.

### Consumption

Recursive producer walk, today only on `Epilogue` and `Staged` (SerialSum arms are flat): `members`, `owns_region_members`, `producer_shape_for`, `input_elements_at`, `reads_declared_input`, `max_input_elements`, `carries_parametric_broadcast`, `output_subject`, `encode_output_subject`, `physical::spell_output`, `pipeline::output_region_role`.

Physical fold path already binds `TensorRole::Intermediate` when `contributor_input` is `None` (`contributor_tensor` / `declared_contributor_tensor`). That is the prologue-produced intermediate. A producer-produced intermediate is the same role, a different preceding region.

`fused_prologue_constants` / `affine_prologue` exist only for a one-leaf declared-input affine prologue. They must keep reading `prologue` / `prologue_reads` and must not see a continuation over a staged value. `staged * 2 + 1` would otherwise satisfy `affine_prologue` and bind `Input { ordinal: 0 }`.

`RegionSpellingKind::Epilogue` already builds an elementwise region whose reads use `BoundaryRead`. A continuation over a staged producer is that region, not `RegionSpellingKind::Pointwise`.

Cover assembly already derives one `MaterializationEdge` per cross-region value (`derive_materializations`), refuses a region that consumes or produces more than one edge (`cover-intermediate-read-attribution`, `cover-region-multiple-materializations`), and orders stages so every producer precedes each consumer (`execution_order`, `cover-materialization-cycle`). A chain of two edges through two regions is the epilogue-over-prologue-and-fold shape that already compiles.

KIR is per scheduled region. Producer, continuation, and fold reuse existing scalar programs (`StrictSerialSum`, `PointwiseF32`, contraction, staged law arms). No new KIR form is required. Backend emission remains a non-goal.

### Identity and schema

- `tiler.compiler.request-subject.v6` is the enclosing domain (`domains.rs` pin; `canonical_explain_subject_bytes` writes that prefix).
- Producer-less sums encode as `serial-sum-f32.v3`. The arm is self-delimiting; an absent prologue is a framed zero. Adding a trailing presence byte to every serial-sum subject would move every already-encodable sum, including the declared-input neighbor.
- Established pattern for a newly expressible output: a new sub-tag (`epilogue-f32.v1`, `staged-family.v2`, `contraction-f32.v1`). The enclosing `v6` domain does not step. Existing pins that encode a `serial-sum-f32.v3` subject stay put.
- Request-subject bytes are compiler identity, not artifact or cache identity. `DeterministicBudgets` comments already state that a request-subject change does not move artifact or cache identity unless selected packaged content moves.
- No artifact, cache, schedule (`tiler.schedule.v5`), or KIR schema step is owed.

### Recursion and work bounds

Producer recognition, subject projection, encoding, membership, and spelling are recursive `Box<NormalizedOutput>` walks. `plan_elementwise` is already a worklist because "a recognizer that consumed host stack proportional to it would turn an input property into a crash rather than a refusal." The producer walk has no worklist. Today's bound is structural: `StagedOperandAdmission::NoEdge` plus `NormalizedSerialSum` having no producer.

`DeterministicBudgets::semantic_operations` (governed 62) bounds program size and therefore maximum nest depth, but a budget is not a host-stack bound. Accepting caller-proportional `recognize_reduction` ↔ `recognize_epilogue_producer` recursion without a sides-rule or a worklist is the crash the ticket forbids.

`encode_output_subject` currently claims "the recursion is bounded by the recognizer, which admits a folding family as a chain's producer and nothing else, so a chain of chains is not a subject this function can be handed." A SerialSum producer field makes that sentence false unless admission stays one-edge.

## Option set

1. **Status quo.** Keep flattening `Folded` to `reduction-contributor-materialization`. No identity move. User-visible outcome is not met.
2. **Reuse `prologue` for the continuation** (store `* 2` in `NormalizedSerialSum::prologue` and treat the producer as the missing declared input, or as `contributor_input = None` Intermediate). Forbidden by this ticket. `fused_prologue_constants` can mis-bind `staged * 2 + 1` as an affine fold over `Input { 0 }`. Silent wrong result.
3. **Synthesize a declared-input baseline** or flatten the producer into the continuation. Forbidden. Changes numerical meaning (materialization / rounding boundary disappears).
4. **Fall back to the current refusal after a failed retain.** Forbidden. Unstated policy.
5. **Specialize the carrier by producer family** (one field or key per of nested-sum / contraction / staged). Forbidden. Mixes cause with subject; every new `materializes_its_result` family widens the form.
6. **New `NormalizedOutput` variant** holding producer + optional continuation + fold, leaving `NormalizedSerialSum` untouched.
7. **Extend `NormalizedSerialSum` with an exhaustive contributor source** that keeps the declared-input and pointwise-prologue neighbors as arms and adds one materialized arm (producer `Box<NormalizedOutput>` + optional continuation over `BoundaryRead`s). New request-subject sub-tag only for the new arm.
8. **Iterative producer frame list** (`Vec` of flat producers) so recognition, encoding, and spelling never recurse. Designed for arbitrary chain depth, which is a non-goal and a second producer encoding beside `Epilogue` / `Staged`.
9. **Recursive `Box` with no sides-rule and no worklist.** Unbounded host stack. Eliminated.
10. **Lift `StagedOperandAdmission::NoEdge` / admit two edges into one region** as part of this carrier. Owned by [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`](admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md) and [`admit-a-scheduled-region-that-reads-two-materialization-edges`](admit-a-scheduled-region-that-reads-two-materialization-edges.md). Out of scope; the latter is `blocked` on a public `TensorRole::Intermediate` ordinal.
11. **Further bounded research** before naming a carrier. The construction, consumption, identity, and recursion paths were read at this base. Remaining questions are implementation proofs (cover search places the parts; reference bit-match), not carrier shape.
12. **Deferral.** Park without a carrier. Dominated by (1) for a ticket whose job is to name the carrier.

### Eliminated before ranking

(2), (3), (4), (5), (9) can silently return a wrong result, flatten a numerical boundary, invent a baseline, or crash the host. (10) claims completeness while depending on unresolved IR/public-boundary work. (11) and (12) do not identify a carrier. (8) is a complete replacement aimed at a non-goal and would give this crate two producer representations.

## Pareto comparison of survivors

Dimensions: correctness, fail-closed strictness, long-term maintainability, Tiler host runtime/memory. Kernel performance is out of scope.

| | (1) Status quo | (6) New `NormalizedOutput` variant | (7) Exhaustive contributor source on `NormalizedSerialSum` |
| --- | --- | --- | --- |
| Correctness | Keeps today's programs correct; does not admit the subjects | Can be injective; compile-fails if a match arm is omitted | Can be injective if the contributor source is an enum, not a boolean plus a reused prologue |
| Fail-closed | Already fail-closed; names the missing relation | New combinations must refuse by name | Same, if `OneEdge` / `NoEdge` is explicit |
| Maintainability | No further type surface | Sixth `NormalizedOutput` arm at every census site | Same house style as `NormalizedStaged::producer`; existing `SerialSum` matches become exhaustive on the contributor enum |
| Host | No new recursion | Recursion must be bounded by sides, not depth | Same bound |

(6) and (7) are not distinct on correctness or host once both use an exhaustive contributor source and the same sides-rule. (6) is worse on maintainability (every `NormalizedOutput` match grows an arm whose fold half duplicates `SerialSum`). (7) is not worse on any key dimension. **(7) dominates (6).**

(1) is strictly weaker on the user-visible outcome and is not a correctness alternative — it is the refusal this ticket exists to replace. It remains available only as "do not implement after Tom rejects admission."

**Single dominating option: (7), with a sides-rule bound, not a worklist and not a depth counter.**

## Recommended carrier

### Normal form

Keep `NormalizedOutput::SerialSum`. Replace the implicit `prologue` XOR `contributor_input` pair with one exhaustive contributor source (field names are implementation detail; the arms are not):

- **Declared input.** Today's `sum(x)` / `sum(b)` neighbour. `contributor_input: Some(ordinal)`, no prologue, no producer.
- **Pointwise prologue.** Today's `sum(x * 2 + 1)` neighbour. `prologue` + `prologue_reads: Vec<(u32, LogicalAccess)>` over declared inputs only. No producer. `fused_prologue_constants` continues to apply only here.
- **Materialized producer.** `producer: Box<NormalizedOutput>` plus optional `ContributorContinuation { expression: PointwiseF32Expression, reads: Vec<(BoundaryRead, LogicalAccess)>, members: Vec<SemanticStage> }`. Exactly one `BoundaryRead::Staged` when a continuation is present. Continuation members are a third partition part; they must not enter `RecognizedSerialSumMembers::pointwise` (that set is the declared-input prologue, and `members.all()` is the fused affine candidate).

The producer is whatever `recognize_epilogue_producer` already returns — `SerialSum`, `Contraction`, or `Staged` — so the form is not keyed by producer family.

Mutual exclusion is the type: a program cannot be both a declared-input prologue and a materialized producer. No identity prologue is synthesized for a bare `sum(producer)`.

### Recognition

Mirror `recognize_elementwise_output`:

1. `plan_elementwise` on the contributor with `staged: None`.
2. `Ok(plan)` of a declared input or pointwise expression → existing arms.
3. `Err(Folded(staged))` and admission `OneEdge` → re-plan with `staged: Some(staged)`, mint the continuation (or record none when the contributor *is* the folded value), call `recognize_epilogue_producer`.
4. `Err(Folded(_))` and admission `NoEdge` → refuse by a new rule that names depth, not the missing field. Proposed key: `reduction-contributor-depth`. Not a producer-family key.
5. Never map `Folded` through `From<ElementwiseRefusal>` after a retain is attempted.

### Bound — sides, not a counter, not a worklist

Introduce `ReductionContributorAdmission { OneEdge, NoEdge }` beside `StagedOperandAdmission`, same shape, same rationale ("a depth counter would be the wrong shape").

- `recognize_output` → `recognize_reduction(..., OneEdge)`.
- `recognize_epilogue_producer` → `recognize_reduction(..., NoEdge)`.

Call graph is then at most `recognize_reduction(OneEdge)` → `recognize_epilogue_producer` → `recognize_reduction(NoEdge)` → no further `Folded` retain. Host stack is constant in program depth. Recursive accessors on a recognized tree are at most two producer hops for this shape, matching today's `Epilogue` → `Staged` → flat producer.

This is the same bound the staged depth ticket measured and kept. Lifting it requires a worklist rewrite of the whole producer walk (`recognize_epilogue_producer`, `spell_output`, `encode_output_subject`, `members`, …), which that ticket already recorded as unwritten and which this ticket lists as a non-goal.

### Occurrence partition and spelling

`members()` = producer.members ∪ continuation.members ∪ prologue.pointwise ∪ reduction.

`owns_region_members` / `spell_output`:

- any producer part (recurse, as `Epilogue` / `Staged` already do);
- continuation members → `RegionSpellingKind::Epilogue` (reuse the staged-read builder; do not send continuation through `pointwise_region`);
- prologue members → existing `Pointwise`;
- reduction members → existing `SerialSum`;
- `RecognizedSerialSumMembers::all()` (prologue ∪ fold only) → existing `FusedSerialSum` when `fused_prologue_constants` answers;
- continuation ∪ fold is *not* a part (no fused spelling; grouping them is declined, not flattened).

`contributor_tensor` stays `Intermediate` whenever the fold does not name a declared input. Assembly binds that unique consumed edge.

### Numerical materialization

Each retained edge is a rounding boundary the caller wrote. Producer identity is the producer subject's own encoding (`encode_output_subject` already writes a nested producer as the standalone family). Producer-before-consumer is `execution_order` over `cover.materializations()`, already proved for epilogue chains. Fusion across the new edge is not offered; `fused_prologue_constants` does not read the continuation.

### Identity / schema consequences

- New sub-tag, proposed `serial-sum-produced-f32.v1`, written only for the materialized arm. `serial-sum-f32.v3` bytes for declared-input and pointwise-prologue sums do not move.
- `tiler.compiler.request-subject.v6` does not step.
- `NormalizedSerialSumSubject` either gains an optional producer slot that is encoded only under the new tag, or a sibling subject variant carries the produced arm. Either is injective; the encoder must write the producer through `encode_output_subject` so a nested fold is the same subject as that fold standing alone.
- No artifact, cache, schedule, or KIR domain steps.
- Public observed-value change, implementation-time: programs this carrier admits stop reporting `reduction-contributor-materialization` and compile. Programs that remain one edge too deep (`sum(sum(contract(a,b))*2)`, `sum(sum(sum(x)*2)*2)`) move from `reduction-contributor-materialization` to `reduction-contributor-depth`. That is a truthful key change, same class as [`name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set`](name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set.md), and needs its own complete regression population when implemented.
- `reduction-contributor-materialization` remains the key for a missing carrier only while this ticket is open. After the carrier lands, that key should be unreachable; a test that still expects it on a one-edge subject is the admission check.

### What is admitted vs refused by name

Admitted through the shared carrier (continuation optional):

- `sum(sum(x) * 2)` / `sum(sum(x * x) * 2)` — producer is a flat `SerialSum` (declared input or pointwise prologue).
- `sum(contract(a, b) * 2)` / `sum(contract(a, b))` — producer is `Contraction` with declared operands.
- `sum(rms_norm(x, w))` / `sum(rms_norm(x, w) * 2)` — producer is `Staged` with declared operands.

Refused under a narrower named prerequisite (not by inventing a family key):

- `reduction-contributor-depth` — producer `SerialSum` whose own contributor is materialized (`sum(sum(contract)*2)`, `sum(sum(sum(x)*2)*2)`, and an epilogue whose producer is a produced sum, e.g. `(sum(sum(x)*2))*3`).
- `staged-operand-depth` — already, `sum(rms_norm(matmul(a, b), w))`.
- `contraction-operands` — already, a contraction producer with a non-declared operand.
- width / `operation-set` — already, `sum(sum(a)*sum(b))` (two folded values in one walk).

Declared-input and pointwise-prologue neighbours unchanged, including `serial-sum-f32.v3` identity.

### Strongest counterargument, reversal, perturbations

**Counterargument.** The sides-rule refuses `(sum(sum(x)*2))*3` and `sum(sum(contract)*2)` even though both are well-formed trees of already-admitted shapes. A worklist producer walk would admit them without a new IR ordinal.

**Why it does not dominate.** Arbitrary chain depth is a non-goal. The staged depth ticket measured that lifting `NoEdge` without a spellable two-edge region buys no program and unbounds recursion. The same is true here for the deeper reduction subjects until a worklist exists. Shipping unbounded recursion to buy those subjects violates the host-stack requirement.

**Reversal evidence.** A worklist rewrite of the producer walk, with a named deterministic budget and a cover that places every edge, would make `NoEdge` on `recognize_reduction` unnecessary. That is a new ticket, not a silent lift.

**Negative controls / subject perturbations (implementation-time):**

- `folded_prologue(false)` must remain `Ok(SerialSum)` with no producer and identical `serial-sum-f32.v3` bytes.
- `folded_prologue(true)` must become `SerialSum` whose producer is `SerialSum` and whose continuation is the `* 2` expression with one `BoundaryRead::Staged`.
- `sum(rms_norm(x, w))` must retain a `Staged` producer and an empty continuation, not a synthesized prologue.
- `sum(sum(a) * sum(b))` must still refuse width, not take the first fold and drop the second.
- `sum(sum(sum(x)*2)*2)` must refuse `reduction-contributor-depth`, not recurse.
- A forged subject that encodes a produced sum under `serial-sum-f32.v3`, or that omits the producer presence tag, must fail injectivity / binding.
- `fused_prologue_constants` on a produced sum whose continuation is `staged * 2 + 1` must stay `None`.
- Perturb the continuation members into `members.pointwise()`: fused spelling or `pointwise_region` must refuse or panic rather than bind declared inputs.

## Follow-up tickets so no work is implicit

These are not part of this research wave. Coordinator files them after Tom accepts (or rejects) the packet.

1. **Implementation carrier** (`implementation/compiler`) — land option (7) only after calibrate's exclusive `implementation/compiler` claim is clear. Includes recognition, partition, spelling, subject tag `serial-sum-produced-f32.v1`, physical binding, cover/ordering reuse, and the three admitted subjects plus declared-input unchanged. No crate edits on this branch.
2. **`reduction-contributor-depth` diagnostic** — if not bundled with (1). Complete affected population: `contraction_direct_path`'s `sum(sum(contract)*2)`, a triple nested sum, and an epilogue over a produced sum. Optimizer contract sentence beside `reduction-contributor-materialization`.
3. **Worklist producer walk** — only if Tom wants arbitrary produced-sum nesting. Depends on (1). Sibling of the deferred staged-depth ticket; do not lift `NoEdge` without it.
4. **Comment repair** on `nested_contraction_chain` (width vs missing producer). Can ride with (1).

No ADR is required for a `pub(crate)` contributor enum and a new request-subject *sub-tag* that does not step `tiler.compiler.request-subject.v6`. If Tom wants the carrier on the decisions catalog, a drafting ticket can copy this packet; do not fork the body during transfer.

## Packet readiness

The census names one bounded injective carrier, its identity/schema consequences, the admitted versus named-refusal population, and the implicit-work tickets. This packet is ready for Tom. Status stays `in-progress`; the coordinator moves it to `awaiting-decision`.
