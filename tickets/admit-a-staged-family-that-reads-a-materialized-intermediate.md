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
lease_expires_at: 1786134331
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
