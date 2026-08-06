---
id: implement-the-realization-witness-vocabulary
title: Implement the realization witness vocabulary
status: todo
priority: p2
dependencies: []
related: [accept-the-realization-witness-surface, enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle, compose-a-declared-reduction-topology-into-a-semantic-program-evaluation]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, conformance, numerics, witness]
---

## The accepted surface this implements

**Tom decided 2026-08-06 (recorded on [`accept-the-realization-witness-surface`](accept-the-realization-witness-surface.md)):** Item A — `RealizationWitness` in `tiler_ir::schedule`, aggregated by `RealizationWitness::of(&VerifiedScheduledRegion)` — and Item B — `UnpinnedFreedomSite`, a refusal enum with no `Conforms`-shaped arm — are accepted as drafted in [the freedom-sites record](../docs/research/reference/plan-freedom-sites.md) Part 7.2. Item C is **redirected to the plain-scalar form**: the reference's evaluation entry points keep taking plain scalars (`strict_partitioned_sum_under` is the pattern) and the aggregation lives in `tiler_ir` alone. `tiler-reference` must not gain a dependency on any plan structure — that constraint is part of the decision, not a style preference.

## The work

- Implement A and B against the record's Part 7.2 drafts, checking each drafted field against the enumeration's Part 2/Part 3 site list at the current tree rather than inheriting (the record was written at `c335bb5b` and lines have moved).
- The witness covers the evaluable-witness sites the enumeration classifies (the six a reference path can evaluate), refuses the unevaluable and undeclared sites through B by name, and does not paper over the mirror sites — a contract mirror is not a witness, per the record's five-way split.
- Determinism note from the record's Part 5: the builder-canonicalization claim ("the canonical form is a function of the program rather than the spelling") is stated there as UNTESTED — this implementation must test it before the witness's converse property (bit-identical plans agree on the witness) is asserted.
- No reference-side edits beyond what plain-scalar threading requires at existing call sites; if a reference entry point seems to need a new signature, stop and report (that would reopen C).

## Closes when

A and B exist as accepted (unlabelled) surfaces matching the record's drafts with drift corrected against source, the determinism claim is tested, the refusal enum's arms are each watched firing, and the freedom-sites record's Part 7 is updated to state which items landed and where.
