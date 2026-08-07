---
id: admit-a-staged-family-that-reads-a-materialized-intermediate
title: Admit a staged family that reads a materialized intermediate
status: in-progress
priority: p2
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages, admit-a-scheduled-region-for-a-staged-elementary-family]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, planner]
claimed_from: todo
assignee: agent-staged-family
lease_expires_at: 1786136376
---
## User-visible outcome

A registered elementary family whose operand is a value another region materializes — `rms_norm(matmul(a, b), w)` rather than `rms_norm(x, w)` — is recognized instead of refused under `staged-operand`.

## Where the refusal is, and why it is a refusal rather than a gap

`recognize_staged_family` (`crates/tiler-compiler/src/request.rs`) requires every operand of a staged occurrence to be a declared program input and refuses `staged-operand` otherwise. That is the wall [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md) named rather than widened, and it is not arbitrary.

**Fact.** An elementwise consumer expresses a materialized read as `EpilogueRead::Staged`, an entry in the *region's read list* that binds `TensorRole::Intermediate`. **Fact.** A recognized staged family carries no read list at all: its stage split — which stage reads which operand — is region formation's, read off the registered law's own realized sequence, because Tom's Option A′ decision made formation the single authority on that. So there is nowhere in the recognized shape to record that operand zero is served by a materialization edge rather than by a declared buffer.

**Inference.** Two coherent resolutions exist and they are not equivalent, which is why this is filed rather than guessed:

- **The recognized shape carries a per-operand boundary role** (`Input(ordinal)` or `Staged`), and program assembly resolves it against the cover the same way it resolves an epilogue's read list. The stage split stays formation's; only the *source* of each operand becomes the recognizer's.
- **The boundary role is derived where the split is**, from the cover's own materialization edges against the stage that reads each operand, and the recognized shape carries nothing new. This keeps one authority but puts a recognition-time property in a later stage, where a refusal is less attributable.

There is also a prior wall in the same direction that this ticket does **not** own: a walk that already reads one staged value and reaches a second is refused because `TensorRole::Intermediate` carries no ordinal (`plan_elementwise`). A contraction feeding a normalization feeding a pass needs that one too.

## Closes when

A staged family reading one materialized intermediate is recognized, the operand's boundary role is carried by exactly one authority with the derivation recorded, and the deeper-chain wall is either lifted with it or named with its own owner.

## Fact corrections, 2026-08-07 (worker, read against the tree rather than the filing)

**The paragraph above conflates two walls under one mechanism, and the correction changes which tickets own what.** "A walk that already reads one staged value and reaches a second is refused because `TensorRole::Intermediate` carries no ordinal (`plan_elementwise`)" describes two different guards in that function, and only one of them is the unordinalled-role rule:

- `record_leaf` refuses one staged value *read twice by one walk* — `s * reverse(s)` — and that is the unordinalled-role rule. Its owner already exists: [`admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region`](admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region.md), whose "Closes when" names `record_leaf`'s branch by name.
- `plan_elementwise`'s `leaves.staged.is_none()` guard refuses a walk that reaches a *second, different* folded value. That is a rule about chain depth rather than about ordinals: each region would read one intermediate. It had no owner; it has one now, together with this ticket's own `staged-operand-depth`, in [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`](admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md).

**"A contraction feeding a normalization feeding a pass needs that one too" is right about the program and wrong about the wall.** `rms_norm(matmul(a, b), a) * a` is refused by the chain-depth rule above (`staged-operand-depth`), not by anything about `TensorRole::Intermediate` carrying no ordinal; and the plain `rms_norm(matmul(a, b), a)` — which *is* recognized after this ticket — needs a third, separate widening to compile at all, because its consuming stage reads two different materialization edges. That one is [`admit-a-scheduled-region-that-reads-two-materialization-edges`](admit-a-scheduled-region-that-reads-two-materialization-edges.md), and it is `tiler-ir`'s public boundary and identity step before it is this crate's.

**One further wall is named because the ticket's own headline example runs into it.** `rms_norm(matmul(a, b), w)` spelled with a third declared input for the weight is refused by `normalize_contraction`'s `input_count() != 2` rule under `input-arity` — the *contraction* recognizer's declared-arity wall, untouched here and owned by [`name-the-contraction-operand-arity-wall-and-separate-its-rule`](name-the-contraction-operand-arity-wall-and-separate-its-rule.md). The two-declared-input spelling `rms_norm(matmul(a, b), a)` exercises the same widening and is what the tests use.

**Superseded rather than stale:** "a recognized staged family carries no read list at all" was true at this ticket's base and is what the first resolution above changed.
