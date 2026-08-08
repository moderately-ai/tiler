---
id: name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set
title: Name the fold-prologue chain boundary instead of reporting operation-set
status: todo
priority: p3
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`sum(matmul(a,b) * 2.0)` refuses under a rule naming what was actually declined — a fold's prologue may not itself be a chain — instead of `operation-set`, which claims the vocabulary cannot spell an operation it spells fine standalone.

## Why this exists (audited 2026-08-06; citations repaired and the mechanism re-measured 2026-08-08)

`recognize_elementwise_output` builds an epilogue chain from `ElementwiseRefusal::Folded`; `recognize_elementwise` — sole caller `recognize_reduction` — discards the finding. The refusal is correct (`NormalizedSerialSum` carries no producer); the rule name and the "the same general walk" doc are the defects.

**Citations repaired 2026-08-08.** The two line pins were stale and are replaced by anchors, per AGENTS.md. The `From` impl is `crates/tiler-compiler/src/request.rs "Flattens a discovered materialization boundary into the rule a caller"`, not line 4259; the doc claiming one walk is `crates/tiler-compiler/src/request.rs "The prologue is recognized by the same general walk"`, not line 5276. Both underlying claims were re-read and hold at `68ba010a`, and `recognize_elementwise` still has exactly one caller; only the pins moved.

**The mechanism was measured rather than inferred (2026-08-08).** Renaming this `From` impl's rule to a probe string makes `sum(sum(x) * 2.0)` — the `folded_prologue(true)` row of `every_refusal_names_its_unrecognized_property` — report the probe: `left: "PROBE-flattened-folded"`. Renaming `plan_elementwise`'s neighbouring `leaves.staged.is_none()` arm instead leaves that row reporting `operation-set`, so this flattening is the sole wall for the shape and the neighbouring guard is never consulted for it. That neighbour is a chain-*width* rule rather than this one; `StagedOperandAdmission`'s doc now separates the three folded-value walls and their owners.

## The work (no admission change)

Distinct rule `reduction-prologue-chain`; the doc stops claiming one walk (shared planner, different findings); `select_supported_strategy`'s boundary paragraph gains the row. Perturbations: the chain-prologue refuses under the new rule, the standalone chain still compiles, and a genuinely unspellable prologue (e.g. `sum(rms_norm(x))`) still reports `operation-set` — else the split renamed rather than separated. File the admission itself as a `deferred` follow-on with the worked-example trigger.

## Closes when

The rule is separated with all three perturbations observed and the deferral filed.
