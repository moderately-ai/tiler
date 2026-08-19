---
id: admit-a-materialized-producer-in-a-serial-reduction-contributor
title: Admit a materialized producer in a serial-reduction contributor
status: done
priority: p3
dependencies: [name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set, drive-staged-materialization-boundary-tests-past-elementary-accuracy]
related: [name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set, admit-a-recognized-chain-more-than-one-materialization-boundary-deep]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [research, compiler, planner, identity, decision, needs-tom]
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

## Readiness correction — 2026-08-13 at `aa389fe1`

Independent exact-base review disproved the single-option frontier. Extending the currently unboxed `NormalizedSerialSum` can enlarge every output and does not itself force broad `NormalizedOutput` consumers to classify the materialized arm. A boxed top-level produced-sum variant that shares a fold core may preserve the old layout and make every output match exhaustive, so option (6) remains nondominated until layout and consumer-migration evidence compares the two exact forms. The census also missed `pipeline/verify.rs`: its fused numerical-proof check exempts every serial sum with `prologue.is_none()`, which a materialized contributor could accidentally satisfy unless the verifier matches the declared-input arm explicitly. This ticket is not Tom-ready and returns to `todo`; repair the full census, prototype/measure both carriers, and repeat the decision gate before restoring `awaiting-decision`.

The corrected nondominated frontier is status quo, a bare-producer fail-closed slice with no continuation, a boxed top-level produced-sum arm sharing a fold core, and an exhaustive contributor source whose rare materialized payload is boxed. A true complete replacement would make every producer walk iterative; the earlier option (8) is only a second flat representation and must not stand in for that alternative. Prototype `size_of::<NormalizedSerialSum>()` and `size_of::<NormalizedOutput>()`, count which consumers fail to compile under each carrier, and compare allocation/dispatch cost before eliminating any survivor.

The verifier repair is a correctness requirement, not a cleanup: the no-proof exemption must match the `DeclaredInput` contributor source explicitly. Perturb only the source to `Materialized` and require the ordinary `portfolio-equivalence` proof path to run. Also distinguish recognizer admission from target admission for `sum(rms_norm(...))`: the governed target intentionally has no elementary row, so positive end-to-end evidence depends on [`drive-staged-materialization-boundary-tests-past-elementary-accuracy`](drive-staged-materialization-boundary-tests-past-elementary-accuracy.md) and a caller-declared discharging RMS realization.

Finally, the source-bearing identity consequence is additive, not absent: old `serial-sum-f32.v3` subjects and pins may remain byte-identical, while newly admitted programs naturally produce new request, plan, schedule, KIR, artifact, and cache identities. No existing domain necessarily steps, but every prototype must prove the new subject cannot encode under the old tag and must enumerate the downstream identities it creates.

## Census repair and re-audit — 2026-08-18, base `1957227cc710a7d7f78b8febacc2d6ccb997448e`

Read in full at this base: the recognition, refusal, subject-encoding, and accessor paths of `crates/tiler-compiler/src/request.rs`; all of `crates/tiler-compiler/src/pipeline/verify.rs`; every serial-sum consumer site in `crates/tiler-compiler/src/physical.rs`, `crates/tiler-compiler/src/pipeline.rs`, and `crates/tiler-compiler/src/pipeline/trace.rs`; and the fixture populations in `request.rs` tests and `crates/tiler-compiler/tests/materialized_intermediate_epilogue_wall.rs`. Prototype measurements were taken in a detached worktree at this exact commit and the worktree was removed afterwards; nothing under `crates/` moved on this branch.

### Per-Fact verdicts

- **Facts 1–4 (2026-08-13 audit): verified unchanged.** Anchors re-run at this base: `Folded` is raised under `leaves.staged.is_none() && materializes_its_result`; `Flattens a discovered materialization boundary into the rule a caller` is still the one flattening and still reports `reduction-contributor-materialization`; `recognize_elementwise` still has one production caller (`recognize_reduction`); `StagedOperandAdmission::NoEdge` is constructed only in `recognize_epilogue_producer`; `materializes_its_result` is still exactly the three families; `NormalizedSerialSum` still carries `prologue`, `prologue_reads`, `contributor_input`, and no producer. The Fact 3 control population is unchanged: `folded_prologue(true)`, `staged_contributor`, and the wall-file / `contraction_direct_path` contraction subjects all still expect `reduction-contributor-materialization`, and `folded_prologue(false)` still recognizes as `SerialSum`.
- **The 2026-08-13 "Related claim that is false at this base" is repaired.** `nested_contraction_chain`'s doc now reads `This is not the two-intermediate-read width wall` and `one missing producer carrier` — landed in `dc18557c` ("test: restore staged materialization boundary evidence"). Follow-up item 4 of the earlier packet is discharged; nothing to file.
- **The readiness correction's `pipeline/verify.rs` claim: verified, and the hole is wider than stated.** The exemption arm in `verify_equivalence` matches `(ProgramAlternativeKind::Fused, None)` under the guard `output.try_serial_sum().is_none_or(|serial| serial.prologue.is_none())`. Two consequences the correction did not spell out: (a) a **non**-serial-sum output also satisfies the guard, through `is_none_or(None) == true`, so a boxed top-level produced-sum arm satisfies the exemption without the site ever re-opening; (b) under the exhaustive contributor source the site **fails to compile** — measured below — the one carrier that forces the verifier repair into review. Reachability at this base: `ProgramAlternativeKind::of` classifies `Fused` only for a one-region whole-program cover, and no genuine produced-sum plan has one (no scheduled region computes producer, continuation, and fold), so today the defect sits in the verifier's forged-portfolio independence — its own contract that a tampered receipt `fails closed instead of being carried into a compilation product` — rather than a live wrong compile. That does not discharge the repair; it is filed as [`match-the-declared-input-contributor-in-the-fused-proof-exemption`](match-the-declared-input-contributor-in-the-fused-proof-exemption.md).
- **The identity correction (additive consequence): verified against the encoder.** The serial-sum sub-tag holds at `v3` (anchor: `The sub-tag holds at`), and the established producer pattern is the staged arm's presence byte plus recursion through `encode_output_subject` (anchor: `The producer, present exactly when some operand above is staged`), which encodes a nested producer exactly as the standalone output of its family. The forged-subject analysis is under Identity below.
- **Neighbourhood movement since 2026-08-13 the packet must absorb.** [`admit-a-strict-serial-fold-that-writes-a-materialized-intermediate`](admit-a-strict-serial-fold-that-writes-a-materialized-intermediate.md) is done: a `ScalarProgram::StrictSerialSum` region may commit to `TensorRole::Intermediate`, so the producer half of `sum(producer)` already has its write vocabulary. [`drive-staged-materialization-boundary-tests-past-elementary-accuracy`](drive-staged-materialization-boundary-tests-past-elementary-accuracy.md) is done, discharging this ticket's second dependency; the recognizer-versus-target-admission caveat for `sum(rms_norm(x, w))` is now the ordinary elementary-accuracy control rather than an evidence gap. `verify_region_output_binding` now opens with a `RegionProgram::Numerical` destructure that refuses a partitioned-copy region under `request-binding`; no carrier consequence — every produced-sum region is `Numerical`. Two doc sentences the carrier makes false are confirmed still present and must be repaired at implementation: `StagedOperandAdmission`'s `declared inputs by construction` (the sentence wraps mid-clause, so the longer spelling greps empty) and `encode_output_subject`'s `admits a folding family as a chain's producer and nothing else`.

### Completed consumer census — what each carrier forces, measured

Production (lib-target) consumers of the fold's contributor fields and of the output vocabulary, with whether each carrier's type change forces the site to re-open (measured by compile-error census below). A = exhaustive contributor source, materialized payload boxed; B = boxed top-level produced-sum arm sharing a fold core; C = additive `producer: Option<Box<NormalizedOutput>>` field.

| Site | File | A | B | C |
| --- | --- | --- | --- | --- |
| `verify_equivalence` fused-proof exemption (`serial.prologue.is_none()`) | `pipeline/verify.rs` | **forced** | silently satisfied via `is_none_or(None)` | silently satisfied |
| `contributor_tensor` / `declared_contributor_tensor` | `physical.rs` | forced | not forced | not forced |
| `fused_prologue_constants` / `affine_prologue` gate | `physical.rs` | forced | decided once at `try_serial_sum` for every caller | not forced |
| `spell_output` serial-sum arm (prologue part, parametric guard) | `physical.rs` | forced | forced (new arm) | not forced |
| `pointwise_region`, `fused_region` prologue reads | `physical.rs` | forced | not forced | not forced |
| `declared_input_for_verified_access`, `verify_region_output_binding`, `subject_contributor_tensor` serial-sum arms | `physical.rs` | forced via the subject accessors | forced (subject arm) | not forced |
| `output_region_role` member partition | `pipeline.rs` | not forced (members shape unchanged) | forced (new arm) | not forced |
| `record_numerical_equivalence` fold attribution via `try_serial_sum` | `pipeline/trace.rs` | not forced, stays correct (produced sum is still the serial-sum arm) | not forced, silently degrades to `reduction-provider-missing` | not forced |
| `members` / `owns_region_members` / `prologue_members` partition | `request.rs` | partially forced (`prologue_members`); producer/continuation parts are named hand-work | forced (new arm) | not forced |
| `carries_parametric_broadcast`, `input_elements_at`, `reads_declared_input`, `max_input_elements` | `request.rs` | forced | forced (new arm) | not forced |
| `encode_output_subject` + `output_subject` projection + subject accessors | `request.rs` | forced | forced (subject arm) | not forced |
| `recognize_reduction` constructor | `request.rs` | forced | not forced (new recognizer beside it) | forced |
| `check_output_cover` (via `members`) | `request.rs` | not forced; producer members ride `members()` extension | forced transitively | not forced |
| frontier proposal plumbing (`propose_split`, `propose_workgroup_tree`, …) | `frontier.rs` | not forced (passes `&NormalizedOutput` through) | not forced | not forced |

`selection.rs` and `frontier.rs` `serial_sum()` call sites are `#[cfg(test)]`-only (the accessor itself is `#[cfg(test)]`).

## Prototype measurements — 2026-08-18

Method: detached worktree at `1957227c`, pinned toolchain `nightly-2026-07-19` (`rustc 1.99.0-nightly eff8269f7`). Layout via `size_of` printed from a `#[cfg(test)]` probe module beside the real types; consumer censuses by applying each carrier's exact type change (and its `NormalizedSerialSumSubject` / `NormalizedOutputSubject` mirror) with no consumer repairs, then `cargo check -p tiler-compiler`. Both measurements are host-independent; no timing was taken and none is owed — layout and forced-site counts decide this frontier, and if the per-materialized-contributor `Box` allocation is ever questioned it is an M3-host measurement for the implementation ticket.

| Type | bytes |
| --- | --- |
| baseline `NormalizedSerialSum` | 296 |
| baseline `NormalizedOutput` | 296 (serial sum is the widest inline arm; `NormalizedPointwise` is 208) |
| baseline `NormalizedSerialSumSubject` / `NormalizedOutputSubject` | 240 / 240 |
| A: source enum `{DeclaredInput(ordinal), PointwisePrologue{expr, reads}, Materialized(Box<_>)}` | 48 |
| A: `NormalizedSerialSum` with the three fields replaced by the source | **288** |
| A: `NormalizedOutput` | **288** |
| A-unboxed (`MaterializedContributor` inline in the source enum; the producer inside it stays boxed) `NormalizedSerialSum` / `NormalizedOutput` | 320 / 320 |
| B: `NormalizedOutput` + `ProducedSum(Box<_>)` sixth arm | 296 (heap payload 664: producer 296 + continuation 72 + embedded fold core 296) |
| C: `NormalizedSerialSum` + `producer: Option<Box<NormalizedOutput>>` | 304; `NormalizedOutput` 304 |
| shared parts: `ContributorContinuation {expr, Vec<(BoundaryRead, LogicalAccess)>, members}` | 72 |

Compile-failure census, `cargo check -p tiler-compiler` (lib; `--all-targets` in parentheses):

| Carrier | errors | distribution |
| --- | --- | --- |
| A (field replacement) | 31 (75) | `request.rs` 23, `physical.rs` 7, **`pipeline/verify.rs` 1 — the exemption itself** |
| B (sixth arm) | 21 (46) | `request.rs` 16, `physical.rs` 4, `pipeline.rs` 1; **`pipeline/verify.rs` 0** |
| C (additive field) | 2 (4) | the two constructors only; every classification, spelling, binding, identity, and verifier site compiles unchanged |

The correction's layout premise is now quantified in both directions: the **unboxed** materialized payload does enlarge every output (+24 bytes, 296 → 320), but the **boxed** payload inside an exhaustive source *shrinks* it (296 → 288), because the enum replaces 56 bytes of `Option<expr>` + `Vec` + `Option<ordinal>` with a 48-byte tagged union. The layout argument that kept option (6) nondominated is therefore disproved by measurement: the boxed contributor source is smaller than both the status quo and the produced-sum arm.

## Re-gated decision packet — 2026-08-18

### Option set and eliminations

1. **Status quo.** Fail-closed, correct, outcome unmet. Retained only as "do not admit after Tom rejects admission".
2. **Bare-producer fail-closed slice, no continuation** — expressed in the contributor-source shape with `Materialized(Box<producer>)` carrying no continuation slot; `sum(producer * 2)` refuses under a new named rule (proposed `reduction-contributor-continuation`). Admits `sum(sum(x))`, `sum(contract(a, b))`, `sum(rms_norm(x, w))`; defers both spellings the User-visible outcome names (`sum(sum(x) * 2)`, `sum(contract(a, b) * 2)`).
3. **Boxed top-level produced-sum arm sharing a fold core** (carrier B). **Eliminated by measurement and by the verifier census.** Its layout advantage does not exist (296 vs A's 288). It leaves the `pipeline/verify.rs` exemption compiling and *satisfied* through `is_none_or(None)`, silently degrades `record_numerical_equivalence`'s fold attribution to `reduction-provider-missing`, concentrates the semantics of every `try_serial_sum` caller into one accessor arm decided once, and re-imports the absence encoding this ticket exists to remove — its shared fold core is a `NormalizedSerialSum` whose `prologue: None, contributor_input: None` state means "materialized contributor" by the absence of both fields, the exact implicit XOR being replaced. It also adds a sixth arm at 21 match sites whose fold halves must delegate to or duplicate serial-sum handling.
4. **Exhaustive contributor source with the materialized payload boxed** (carrier A). Survivor; recommended.
5. **A-unboxed.** Eliminated: +24 bytes on every `NormalizedOutput` for zero capability difference — the correction's warning, quantified.
6. **Additive producer field** (carrier C). Eliminated: it forces 2 of the ~27 correctness-bearing sites; every perturbation this ticket's own list demands — the verifier proof path, the fused-constants gate, the subject encoder's producer presence, the member partition — is left to unforced hand edits, and the subject encoder compiling unchanged means a forged produced subject's separation from `serial-sum-f32.v3` would rest on an accidental property of the unread-marker run (see Identity), the unstated-invariant reliance the pointwise arm's identity comment forbids.
7. **The true complete replacement — every producer walk iterative.** Evaluated as instructed: it is a genuine alternative for the *producer-walk representation*, not for this admission's carrier, and it does not belong on this frontier. Even a worklist walk needs a typed place to retain producer and continuation — one of the carriers above — so "iterative everything" is a carrier **plus** a rewrite of the fourteen recursive walkers the census names (`recognize_epilogue_producer`, `encode_output_subject`, `output_subject`, `members`, `owns_region_members`, `producer_shape_for`, `input_elements_at`, `reads_declared_input`, `max_input_elements`, `carries_parametric_broadcast`, `spell_output`, `verify_region_output_binding`, `output_region_role`, `declared_input_for_verified_access`), whose sole additional buy is arbitrary chain depth — an explicit non-goal, and the deferred staged-depth ticket's measured no-program-bought result. It remains the recorded reversal path for the sides-rule (follow-up 3), not a frontier member; presenting it here would present a knowingly dominated composite.

### Pareto comparison of survivors

Dimensions: correctness, fail-closed strictness, long-term maintainability, host runtime/memory. Kernel performance out of scope.

| | (1) Status quo | (2) Bare-producer slice on the source shape | (4) Full contributor source with continuation |
| --- | --- | --- | --- |
| Correctness | correct, admits nothing | same forced-site census as (4); admits the three bare subjects | same; admits the ticket's stated population; continuation reuses the proven epilogue machinery (`BoundaryRead` reads, `RegionSpellingKind::Epilogue`, the staged-leaf mint of `recognize_epilogue`) |
| Fail-closed | already fail-closed | exhaustive source; continuation refused by name | exhaustive source; depth refused by name (`reduction-contributor-depth`), width and staged-depth unchanged |
| Maintainability | no new surface | second landing owed for the continuation; a named refusal that later becomes unreachable | one landing; no successor ticket owed for the outcome |
| Host | 296 | 288 | 288 |

(2) and (4) tie on correctness, strictness, and host; (4) meets the User-visible outcome and the Activation clause's three subjects in one landing, while (2) defers two of the outcome's own example spellings behind a rule that exists only to be deleted. (4) weakly dominates on maintainability; (2) survives only as the smaller-first-diff staging choice. **Recommended: (4), with the sides-rule bound.**

### Carrier, recognition, bound, spelling, numerical sections

Unchanged from the 2026-08-13 packet except as restated by the measurements: the recommended normal form is the 2026-08-13 "Recommended carrier" section's, with the payload boxed — `Materialized(Box<MaterializedContributor>)` where `MaterializedContributor { producer: NormalizedOutput, continuation: Option<ContributorContinuation> }` — and the `ReductionContributorAdmission { OneEdge, NoEdge }` sides-rule exactly as written there (`recognize_output` → `OneEdge`, `recognize_epilogue_producer` → `NoEdge`; host stack constant in program depth; at most two producer hops, matching today's `Epilogue` → `Staged` bound).

### Identity — measured and completed

- New sub-tag, proposed `serial-sum-produced-f32.v1`, written only by the `Materialized` arm; the `DeclaredInput` and `PointwisePrologue` arms keep writing `serial-sum-f32.v3` byte-for-byte. `tiler.compiler.request-subject.v6` does not step; the `domains.rs` pin row stays `(1, 0)`.
- **The new subject cannot encode under the old tag, structurally.** Under the exhaustive source the encoder's serial-sum arm must match the source enum, the `Materialized` arm writes its own framed tag first, and framed tags separate arms before any payload is read. The producer is written through `encode_output_subject` recursion so a nested fold is the same subject as that fold standing alone; the continuation is a presence byte then the epilogue arm's `BoundaryRead`-tagged read vocabulary. Conversely the old grammar has no producer slot: a dropped-producer forgery pushed through the `serial-sum-f32.v3` arm would emit an `encode_elementwise_reads` run of *only* unread markers (`0x04` for every declared ordinal), which no legal old subject produces — every legal fold reads at least one declared input in its own regions — but that separation is an accidental property of the marker run, and resting identity on it is exactly what the pointwise arm's `would be exactly the unstated invariant` comment forbids. The tag split is the structural control; the marker-run accident is why carrier C's unforced encoder was eliminated rather than argued safe.
- Downstream identities a newly admitted program creates, per the correction: a new request subject (new arm) and therefore new evidence/explain subjects; new cover, plan, schedule, KIR, artifact, and cache *content* identities minted within existing domains for the new programs. No domain steps; artifact and cache identity move only with selected packaged content (anchor: `not artifact or cache identity; those move only when the`). The implementation regression is that the existing pinned request qualifiers (`deterministic_trace_is_sealed_and_rendered_separately`, the tiler-build Metal goldens) recompute unchanged.

### Strongest counterargument, reversal evidence, perturbations

**Counterargument to (4) over (2):** a smaller first landing is easier to review, and the continuation partition (a third member part that must not enter `RecognizedSerialSumMembers::pointwise`) is the one genuinely new machine. **Why it does not win:** the continuation's regions, reads, and numbering already exist as the epilogue path, the slice's named refusal is scaffolding that must later be removed from the optimizer contract, and the outcome sentence names two continuation spellings.

**Counterargument to (4) over (3):** every `NormalizedOutput` match already exists and B's new arm makes the produced population visible at each. **Why it does not win, measured:** B's forced set (21 sites) misses the verifier exemption entirely while A's (31) contains it; B is not smaller (296 vs 288); and B's fold core re-encodes the contributor source by field absence.

**Reversal evidence.** A measured layout or forced-site result contradicting the table above (rerun the probe at the implementation base); a demonstration that a genuine `Fused`-classified produced-sum alternative is constructible (would upgrade the verifier defect from forged-portfolio to live and raise the filed ticket's priority); a worklist producer walk with a named budget and a cover that places every edge (reopens the sides-rule, follow-up 3).

**Perturbations (implementation-time), superseding the 2026-08-13 list where they overlap:**

- Perturb only a recognized subject's contributor source to `Materialized` and require the ordinary `portfolio-equivalence` proof path to run — the filed verifier ticket's check, with failure text shown.
- `folded_prologue(false)` byte-identical under `serial-sum-f32.v3`; `folded_prologue(true)` recognized with a `SerialSum` producer and a one-`BoundaryRead::Staged` continuation; `sum(rms_norm(x, w))` a `Staged` producer with no synthesized continuation.
- A forged subject encoding a produced sum under `serial-sum-f32.v3`, or omitting the producer presence, must fail binding — and the test must forge the bytes, not rely on the marker-run accident.
- `fused_prologue_constants` on `staged * 2 + 1` stays `None` (type-forced by the source enum; keep the negative test).
- Continuation members perturbed into `members.pointwise()` must refuse or panic before `pointwise_region` binds declared inputs; `output_region_role` for a continuation region must not answer `whole-program` — both are named hand-work sites A does not force.
- `sum(sum(a) * sum(b))` still refuses width; `sum(sum(sum(x)*2)*2)` refuses `reduction-contributor-depth`, not recursion; `record_numerical_equivalence` resolves the fold of a produced sum (A keeps this automatically; the assertion pins it).
- Repair the two doc sentences named in the census (`declared inputs by construction`; `admits a folding family as a chain's producer and nothing else`).

### One question for Tom

Accept carrier (4) — the exhaustive contributor source with a boxed materialized payload, continuation included, sides-rule bound, new `serial-sum-produced-f32.v1` sub-tag — or restrict the first landing to (2), the bare-producer slice on the same shape with the continuation refused under `reduction-contributor-continuation`? Recommendation: (4); (2) buys review size only and defers the outcome's own examples.

### Follow-ups after this packet

1. **Implementation carrier** — as the 2026-08-13 list's item 1, updated to the boxed payload; the compile-failure census above is the migration map, and the sites it does *not* force (member partition, `output_region_role`, `check_output_cover` extension) are the hand-work list.
2. **`reduction-contributor-depth` diagnostic** — unchanged (2026-08-13 item 2).
3. **Worklist producer walk** — unchanged (2026-08-13 item 3); reversal path, not frontier.
4. ~~Comment repair~~ — already landed in `dc18557c`.
5. **Verifier exemption repair** — filed now as [`match-the-declared-input-contributor-in-the-fused-proof-exemption`](match-the-declared-input-contributor-in-the-fused-proof-exemption.md); under carrier (4) the site is compile-forced and the repair rides the migration, but it carries its own perturbation obligation and reviewer.

### Packet state

Re-gated over the corrected frontier with measurements. Deliberately left `in-progress` and unqueued: independent review of this packet is owed before `awaiting-decision` is restored, per the 2026-08-13 correction.

## Independent review — 2026-08-18, base `236753a3b4ae8fa42da0a67d7f4f4d3c9a864a48`

Reviewed on `tkt/admit-a-materialized-producer-review` at the base above, which sits eighteen commits past the repair base `1957227c` (ADR 0013 stability subject and tiler-metal facade landings). The four serial-sum-bearing files are byte-identical across the two bases — `git rev-parse <base>:<file>` matches for `request.rs` (`b871610a`), `physical.rs` (`02813b7c`), `pipeline/verify.rs` (`65b76fbf`), and `pipeline.rs` (`9ecc095f`) — and `pipeline/trace.rs`'s +65-line delta is the ADR 0013 determinism witness in `record_alternative_explain`, touching no serial-sum consumer, so every measurement and census row transfers to this base unchanged. Read in full this session: `pipeline/verify.rs`; the recognition, refusal, type, subject-encoder, and control-population regions of `request.rs`; the contributor-tensor, fused-constants, and output-binding regions of `physical.rs`; `record_numerical_equivalence` in `trace.rs`; `ProgramAlternativeKind::of` in `pipeline.rs`; the wall-file fixtures; and both ticket files.

### Per-Fact verdicts on the 2026-08-18 audit

- **Facts 1–4 re-verification: verified.** Every anchor re-run and read in context at this base: the `Folded` raise sits under `leaves.staged.is_none() && materializes_its_result(&operation, laws)`; `From<ElementwiseRefusal>` is the one flattening and reports `reduction-contributor-materialization`; `recognize_elementwise` has exactly one production caller, inside `recognize_reduction`; `recognize_reduction` has exactly the two callers `recognize_output` and `recognize_epilogue_producer`; `StagedOperandAdmission::NoEdge` is constructed only in `recognize_epilogue_producer` (the other hit is the comparison in `recognize_staged_family`); `materializes_its_result` is the three families; `NormalizedSerialSum` carries `prologue` / `prologue_reads` / `contributor_input` and no producer.
- **The widened verifier finding: verified by full read of `verify.rs`.** The `(ProgramAlternativeKind::Fused, None)` arm's guard is `output.try_serial_sum().is_none_or(|serial| serial.prologue.is_none())`, so a non-serial-sum output is exempt through the vacuous `is_none_or(None) == true` branch exactly as the packet and the filed ticket state, and any carrier leaving `prologue: None` on a produced sum satisfies the second half. The arm's comment `The condition is the prologue, not the family` and the module contract `fails closed instead of being carried into a compilation product` are both present.
- **The `dc18557c` comment repair: verified.** The wall file's `nested_contraction_chain` doc now opens `one missing producer carrier` / `This is not the two-intermediate-read width wall`, and `git show dc18557c` confirms that commit touched the wall file. Follow-up 4 is genuinely discharged.
- **Control population: verified.** `folded_prologue(false)` asserts `Ok(NormalizedOutput::SerialSum(_))`; `folded_prologue(true)` and `staged_contributor` assert `reduction-contributor-materialization` in `request.rs` tests; the wall file's `nested_reduction_chain` / `nested_contraction_chain` assert the same rule end-to-end.
- **The anchor repair: verified.** `declared inputs by construction` greps as its own line because the sentence wraps after `contributor walk reads`, exactly as the census warns; `admits a folding family as a chain's producer and nothing else` greps whole. Both doc sentences are confirmed still present and become false under the carrier.
- **Identity claims: verified against the encoder.** The `serial-sum-f32.v3` arm, the `The sub-tag holds at` comment, the staged arm's leading producer presence byte with recursion through `encode_output_subject`, the `0x04` `UNREAD_DECLARED_INPUT_TAG` reservation, and the pointwise arm's `would be exactly the unstated invariant` comment are all as cited; the `domains.rs` request-subject pin row is `(1, 0)`. One strengthening the packet understates for carrier C: with the encoder unforced, a produced sum's producer is not encoded *at all*, so beyond the fragile marker-run separation from old subjects, two produced sums differing only in their producers would collide under `serial-sum-f32.v3` — un-repaired C is non-injective among the new population, not merely accidentally separated from the old one. The elimination stands a fortiori.
- **Neighbourhood movement: verified.** `admit-a-strict-serial-fold-that-writes-a-materialized-intermediate` and `drive-staged-materialization-boundary-tests-past-elementary-accuracy` are `done`; the depth ticket remains `deferred`; `verify_region_output_binding` opens with the `RegionProgram::Numerical` destructure refusing `request-binding`. The census's `#[cfg(test)]` claim for `selection.rs` / `frontier.rs` `serial_sum()` call sites checks out: those sites call the `#[cfg(test)]` accessor on the request wrapper, not the production `NormalizedOutput::serial_sum`.

### Measurements reproduced

Method: detached scratch worktree at this base (serial-sum files byte-identical with `1957227c`), same pinned toolchain (`rustc 1.99.0-nightly eff8269f7`), probe module beside the real types, worktree removed afterwards; this branch's tree stayed byte-identical (`git status` clean).

| Measurement | Packet | Reproduced |
| --- | --- | --- |
| baseline `NormalizedSerialSum` / `NormalizedOutput` / `NormalizedPointwise` | 296 / 296 / 208 | 296 / 296 / 208 |
| baseline subjects | 240 / 240 | 240 / 240 |
| A source enum / `NormalizedSerialSum` / `NormalizedOutput` | 48 / 288 / 288 | 48 / 288 / 288 |
| A-unboxed | 320 / 320 | 320 / 320 under the row's intended shape (`MaterializedContributor` inline, producer boxed); a fully inline producer measures **608 / 608** |
| B `NormalizedOutput` / heap payload | 296 / 664 | 296 / 664 |
| C | 304 / 304 | 304 / 304 |
| `ContributorContinuation` | 72 | 72 |

The A-unboxed row label was ambiguous — repaired above in place. Both readings are eliminated (320 is +24 for nothing; 608 is worse), so no frontier consequence.

Compile census, carrier A spot-verified by applying the exact field replacement (struct + subject mirror, no consumer repairs): **31 lib errors, distributed `request.rs` 23 / `physical.rs` 7 / `pipeline/verify.rs` 1, and the one verifier error is the exemption guard itself** — `verify.rs:324: no field 'prologue' on type '&NormalizedSerialSum'`, i.e. `serial.prologue.is_none()`. Exact match with the table. The parenthetical `--all-targets` count did not reproduce: my run reports **42** unique diagnostics (`request.rs` 34 / `physical.rs` 7 / `verify.rs` 1, "could not compile (lib test) due to 42 previous errors") against the table's 75, because cargo deduplicates identical diagnostics across the lib and lib-test units and integration-test targets are never reached once the lib fails; 75 is consistent with counting the lib and lib-test emissions separately. Severity: minor — the parenthetical numbers are method-sensitive and were measured consistently across carriers, and the lib counts, which carry the whole forced-site argument, reproduce exactly. B and C were not re-run (spot-verify brief); their lib rows are consistent with the consumer sites read this session (`fused_prologue_constants` via `try_serial_sum`, `record_numerical_equivalence`'s `reduction-provider-missing` arms, the two constructors for C).

### Frontier attack

- **B's elimination is sound, and is in fact domination.** With layout measured (296 vs 288) B is tied or worse than A on every stated dimension — correctness exposure (verifier exemption vacuously satisfied, verified from `try_serial_sum`'s arms; fold attribution degrading to `reduction-provider-missing`, verified in `trace.rs`), strictness (absence-encoded fold core re-imports the XOR), maintainability (sixth arm at every match), and host (not smaller). The 2026-08-13 correction's ground for keeping it was layout; that premise is disproved by a measurement I reproduced independently.
- **The iterative walk's exclusion is correct.** It is a composite (a carrier plus a walker rewrite) whose only additional buy is the explicit non-goal, with the staged-depth ticket's measured no-program-bought result on point; keeping it as the recorded reversal path rather than a frontier member is exactly what the gate's step 2 requires. The packet's "thirteen" walkers were fourteen by its own list — repaired in place; the count is not load-bearing.
- **The three survivors are presentable, with one caveat the question should carry.** By the packet's own table, (4) weakly dominates (2) on the four stated dimensions; (2) survives only on first-landing review size, which is not one of them. That is defensible because the full-versus-slice split is genuinely Tom's implementation-scoping call, but the packet omits (2)'s identity cost: as specified — `Materialized(Box<producer>)` **carrying no continuation slot** — the `serial-sum-produced-f32.v1` grammar would gain a continuation presence byte at the second landing, moving every (2)-admitted subject's bytes and forcing a tag step to `.v2` under the encoder's own forced-not-chosen standard. The cheap fix if Tom picks (2) is to write the continuation presence byte (always absent) from the first landing, making the second landing recognition-only with no re-tag. Either way this cost belongs beside the question; it further weakens (2) and strengthens the recommendation of (4).
- **The one-question framing is right** once the note above is in view: carrier shape is settled by measurement plus the verifier census (single dominating shape), and the residual full-versus-slice choice is a genuine staging decision on one type, not a manufactured carrier choice.

### Filed ticket `match-the-declared-input-contributor-in-the-fused-proof-exemption`

Verified in full. The `serial.prologue.is_none()` anchor, the `The condition is the prologue, not the family` quote, and the fail-closed contract quote all resolve against `verify.rs`; the two-absences reading is what the guard says; the reachability Fact is confirmed from `ProgramAlternativeKind::of` (`Fused` exactly when one region covers every operation, so a produced-sum plan — at least producer plus fold regions — cannot classify `Fused` and the live defect is the forged-receipt direction); the carrier-decides-forcing Fact matches my census (A forces the site, the E0609 above; B and C leave it compiling). The perturbation perturbs the subject (contributor source → `Materialized`) and demands the failure text; the controls keep `sum(x)`'s exemption and the with-prologue proving path. The dependency edge on this decision ticket with the documented retarget-on-filing is correct wiring given the implementation ticket does not exist yet. No repairs needed.

### Discrepancies

1. **Minor, repaired:** A-unboxed table row was ambiguous about which box is removed; the intended shape reproduces at 320 and the label now says so (fully inline measures 608, eliminated a fortiori).
2. **Minor, repaired:** "thirteen recursive walkers" enumerated fourteen names; corrected.
3. **Minor, recorded:** the `--all-targets` parenthetical counts double-count deduplicated diagnostics (my unique count for A is 42, not 75); the lib counts that decide the frontier reproduce exactly.
4. **Moderate, addressed here:** option (2)'s unstated identity cost — a continuation-slot-less `serial-sum-produced-f32.v1` re-tags at the second landing unless the presence byte is written from day one. Stated above beside the question; it does not change the recommendation, which it strengthens.
5. **Coordination note, not a packet defect:** the review brief described the two bases as separated by ticket-only merges; the separation includes crate movement (ADR 0013, tiler-metal facade). Measurement validity was re-established by blob-hash identity of the serial-sum files rather than by that claim.

### Verdict

**Ready for Tom with the named repairs, which are made above.** Every load-bearing claim reproduced under independent derivation: the audit's anchors, the widened verifier guard, the layout table, carrier A's forced-site census including the compile-forced exemption, B's and C's eliminations (both slightly *understated* by the packet), the iterative-walk exclusion, and the filed ticket's Facts. The coordinator may restore `awaiting-decision`; the one question should be read together with the option (2) identity-cost note above.

## Accepted decision — 2026-08-18

Tom accepted **carrier (4)** — the full exhaustive contributor source with the materialized payload boxed (`Materialized(Box<MaterializedContributor>)`, producer plus optional continuation), the `ReductionContributorAdmission { OneEdge, NoEdge }` sides-rule bound, and the new `serial-sum-produced-f32.v1` sub-tag written only by the materialized arm — over the bare-producer slice and the status-quo refusal, exactly as recommended by the re-gated packet reviewed at `90714048`, with the review's option-(2) identity-cost note in view.

- Provenance: Tom, 2026-08-18, live decision-queue session in the coordination conversation ("agreed next decision" on item 22 of `.ticketsplease/decision-queue.md`, presented in explain-then-recommend form immediately beforehand).
- What this accepts: the carrier shape, its recognition/bound/spelling/numerical sections as restated by the 2026-08-18 packet, the identity consequences (new sub-tag; `serial-sum-f32.v3` bytes unmoved; `tiler.compiler.request-subject.v6` does not step; no artifact/cache/schedule/KIR domain steps), and the named-refusal population including `reduction-contributor-depth` for one-edge-too-deep subjects.
- What this does not accept: arbitrary chain depth (the worklist producer walk remains the recorded reversal path, unfiled), backend emission, or performance selection.
- Remainder: split into the implementation carrier [`replace-the-serial-sum-contributor-fields-with-the-exhaustive-source`](replace-the-serial-sum-contributor-fields-with-the-exhaustive-source.md), which owns the migration, the `reduction-contributor-depth` regression population, the two doc-sentence repairs, and the perturbation list; the verifier-exemption repair [`match-the-declared-input-contributor-in-the-fused-proof-exemption`](match-the-declared-input-contributor-in-the-fused-proof-exemption.md) is retargeted onto that carrier per its own Coordination note. This decision ticket closes as done with the carrier named and accepted; the Activation clause's admitted-or-refused outcome transfers to the carrier ticket.
