---
id: admit-a-staged-family-that-reads-a-materialized-intermediate
title: Admit a staged family that reads a materialized intermediate
status: done
priority: p2
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages, admit-a-scheduled-region-for-a-staged-elementary-family, admit-a-recognized-chain-more-than-one-materialization-boundary-deep, admit-a-scheduled-region-that-reads-two-materialization-edges, admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region, name-the-contraction-operand-arity-wall-and-separate-its-rule, admit-a-contraction-over-a-subset-of-the-declared-inputs]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, planner]
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
- `plan_elementwise`'s `leaves.staged.is_none()` guard refuses a walk that reaches a *second, different* folded value — `sum(a, 1) * sum(b, 1)`. That is a rule about chain **width** / unordinalled Intermediate rather than depth: the walk is still one materialization boundary deep, and what it lacks is an ordinal on `TensorRole::Intermediate` so two edges in one region can be attributed. Its region-vocabulary owner is [`admit-a-scheduled-region-that-reads-two-materialization-edges`](admit-a-scheduled-region-that-reads-two-materialization-edges.md); the one-value-twice spelling stays with the second-read ticket above. **Depth** is a separate rule: `StagedOperandAdmission::NoEdge` → `staged-operand-depth` in `recognize_staged_family`, owned alone by [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`](admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md).

**"A contraction feeding a normalization feeding a pass needs that one too" is right about the program and wrong about the wall.** `rms_norm(matmul(a, b), a) * a` is refused by the **depth** rule (`staged-operand-depth` / `StagedOperandAdmission`), not by `plan_elementwise`'s width guard and not by anything about `TensorRole::Intermediate` carrying no ordinal; and the plain `rms_norm(matmul(a, b), a)` — which *is* recognized after this ticket — needs a third, separate widening to compile at all, because its consuming stage reads two different materialization edges. That one is [`admit-a-scheduled-region-that-reads-two-materialization-edges`](admit-a-scheduled-region-that-reads-two-materialization-edges.md), and it is `tiler-ir`'s public boundary and identity step before it is this crate's.

**One further wall is named because the ticket's own headline example runs into it.** `rms_norm(matmul(a, b), w)` spelled with a third declared input for the weight is refused by `normalize_contraction`'s `input_count() != 2` rule under **`contraction-input-arity`** — the *contraction* recognizer's declared-arity wall, untouched here and owned by [`name-the-contraction-operand-arity-wall-and-separate-its-rule`](name-the-contraction-operand-arity-wall-and-separate-its-rule.md) (widening by [`admit-a-contraction-over-a-subset-of-the-declared-inputs`](admit-a-contraction-over-a-subset-of-the-declared-inputs.md)). Program-wide zero-input refusal alone still uses `input-arity`. The two-declared-input spelling `rms_norm(matmul(a, b), a)` exercises the same widening and is what the tests use.

**Superseded rather than stale:** "a recognized staged family carries no read list at all" was true at this ticket's base and is what the first resolution above changed.

## Outcome — delivered 2026-08-07 at `13cb0664`

The operand's boundary role is now the **recognizer's**, chosen over deriving it from the cover: an operand supplied by no declared input and no recognizable producer is a property of the *program*, and a stage discovering it later could only report it as a cover it failed to assemble. `EpilogueRead` became `BoundaryRead`, so one vocabulary serves the epilogue's read list and the staged operand run and `tensor()` stays the single statement of the mapping onto `TensorRole`.

**One change the ticket did not anticipate, and it is forced.** `NormalizedStaged` needed a `producer`, because without it the producing occurrence is claimed by no walk and `check_output_cover` refuses under `operation-set` — **watched failing rather than argued**. It ripples through eight accessors, including `producer_shape` becoming `producer_shape_for(members)`, since such an output holds two shapes whose regions a cover places separately.

The staged ownership predicate stays **one authority**: `owns_stage_members` is the single site both the recognizer and the physical speller read, preserving a landed invariant rather than re-splitting it.

The subject stepped `staged-family.v1 → v2`, and the step is forced rather than chosen: an operand entry used to open with its ordinal and now opens with the role tag, so per-tag injectivity does not close. The enclosing `request-subject.v5` domain does not step.

### The finding worth keeping: a green test that had stopped testing its subject

Perturbing the subject encoding to check the separation test could fail produced a **pass** — the producer field alone was separating the two forgeries, so the field under test was never exercised. The worker rewrote it so each forgery moves exactly one field, then observed both perturbations failing. That is "a verdict is only as good as the check's ability to say no" in its most invisible form, and it was found only because every refusal was watched failing rather than assumed.

Seven perturbations in total, each restored, including one that reproduces exactly where the wall moved.

### A conflation in this ticket's own Facts, repaired in four other places too

The ticket treated one refusal as a single guard. It is **two**: `record_leaf` (one staged value read twice) is the ordinal rule and already had an owner; `plan_elementwise`'s `leaves.staged.is_none()` guard is a **chain-width** / unordinalled-Intermediate rule — one walk reaching a second *different* folded value — owned with [`admit-a-scheduled-region-that-reads-two-materialization-edges`](admit-a-scheduled-region-that-reads-two-materialization-edges.md) (and the second-read ticket for one-value-twice). **Chain depth** is solely `staged-operand-depth` / `StagedOperandAdmission`, owned by [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`](admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md). The same conflation had propagated into four in-crate doc comments, all corrected. The ticket's headline spelling also runs into the *contraction* recognizer's declared-arity wall (`contraction-input-arity`) rather than this one, so the tests use the two-declared-input spelling.

### What still refuses, asserted rather than implied

Five shapes, each with its rule named and its owner identified — and the physical boundary is a **checked assertion** with a control asserted beside it, not prose. Two were unowned and are filed: [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`](admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md) and [`admit-a-scheduled-region-that-reads-two-materialization-edges`](admit-a-scheduled-region-that-reads-two-materialization-edges.md).

**No pin moved**, verified by running both pin tests rather than inferring from a green suite: no pinned identity encodes a staged subject. **No public surface** — `request` and `physical` are private modules and everything added is `pub(crate)`.

Three things deliberately not done, each with its reason: no `tiler-ir` edit (the full admission needs an ordinal on `TensorRole::Intermediate`, a public boundary and a `tiler.schedule.v5` identity step — filed, not attempted); no widening of the contraction arity rule (a separate wall with an existing owner); and no pointwise producer admitted for a staged operand, which would be a second disagreeing account of what a materialization edge is and would materialize a value the caller never asked for.

`make full` exit 0 on the branch and again on the merged tree — 3,054 workspace, 1,068 release.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** The 2026-08-07 Fact corrections and Outcome paragraphs classified `plan_elementwise`'s `leaves.staged.is_none()` guard as a **chain-depth** rule owned together with `staged-operand-depth` by [`admit-a-recognized-chain-more-than-one-materialization-boundary-deep`](admit-a-recognized-chain-more-than-one-materialization-boundary-deep.md). That classification is false and must not be residual-wall authority.

- **Width, not depth.** The guard refuses one walk that reaches a *second, different* folded value (`sum(a, 1) * sum(b, 1)`): one region would take two unordinalled `TensorRole::Intermediate` reads. Source in `crates/tiler-compiler/src/request.rs` states it as chain *width*; `StagedOperandAdmission` docs separate the same neighbour. Owner: [`admit-a-scheduled-region-that-reads-two-materialization-edges`](admit-a-scheduled-region-that-reads-two-materialization-edges.md), with [`admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region`](admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region.md) for one-value-twice.
- **Depth is one guard only.** `StagedOperandAdmission::NoEdge` → `staged-operand-depth` in `recognize_staged_family` is the whole depth rule, owned alone by the depth ticket (its own 2026-08-08 correction already withdrew the elementwise-guard half).
- **Contraction declared-count rule name.** The residual three-input headline refusal is under **`contraction-input-arity`**, not `input-arity` (the latter remains the program-wide zero-input check). Guard site `normalize_contraction`'s `input_count() != 2` is unchanged; naming landed under [`name-the-contraction-operand-arity-wall-and-separate-its-rule`](name-the-contraction-operand-arity-wall-and-separate-its-rule.md).

Live body above this block was rewritten to match; this dated note exists so a reader who stops at the 2026-08-07 "repair" does not re-adopt the false residual-wall map.

## Evidence correction — 2026-08-13 at `c9da757e`

The recognizer's `BoundaryRead` carrier, staged producer relation, ownership predicate, and `staged-family.v2` subject remain present, so this implementation ticket stays `done`. Its external control and residual-wall fixture no longer establish the phases their comments claim: `staged_family_over_a_materialized_intermediate.rs` now sends both the materialized subject and declared-input control through `TargetProfile::governed()`, and both stop first at `accuracy.elementary.no-installed-realization`.

The current green assertions are only evidence that missing elementary authority fails closed. [`drive-staged-materialization-boundary-tests-past-elementary-accuracy`](drive-staged-materialization-boundary-tests-past-elementary-accuracy.md) must supply a caller-declared discharging RMS row, then re-derive and pin the materialized subject's actual downstream wall with a progressing declared-input control. Until then, do not use this fixture as evidence that recognition or the scheduled-region boundary was reached.

## Evidence restoration — 2026-08-13

`staged_family_over_a_materialized_intermediate.rs` now uses the caller-declared discharging RMS fixture owned by [`drive-staged-materialization-boundary-tests-past-elementary-accuracy`](drive-staged-materialization-boundary-tests-past-elementary-accuracy.md). Its two strict-order contracts reach and name `region-vocabulary`; its three reassociation-permitting contracts retain the additional fusion-legality uncertainty and report `NoFeasiblePlan`; and the declared-input control compiles under all five. The counted `2`/`3` cause split and five compiling neighbours make an empty or shared request-gate population fail.

The adjacent request controls keep the layers separate: omitting the row reports `accuracy.elementary.no-installed-realization`, declaring a same-operation non-refining contract reports `accuracy.elementary.unrefined-realization`, and the actual governed profile remains silent. The existing `BoundaryRead` carrier, producer relation, ownership predicate, and `staged-family.v2` subject remain the capability authority; the synthetic profile is not a governed or Metal declaration.
