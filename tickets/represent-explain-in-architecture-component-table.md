---
id: represent-explain-in-architecture-component-table
title: Represent explain authority in the architecture component table
status: done
priority: p2
dependencies: []
related: []
scopes: [contracts/foundation, contracts/decisions]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [documentation, architecture]
---
ADR 0073 records that typed explain infrastructure is owned by `tiler-compiler`, and declares `applies_to: ["tiler.contract.optimizer"]` only. That was deliberate and correct: `docs/architecture.md`'s component table lists `tiler-compiler` as "Normalization, rule engine, fusion planning, index lowering, schedule search, costing" without enumerating explainability, so claiming `applies_to` over architecture would have asserted authority the document does not currently represent — and the authoring ticket did not hold `contracts/foundation`.

Add explainability to that component row so the architecture contract represents what ADR 0070 already places in `tiler-compiler` and ADR 0073 now governs. Keep the edit minimal: this records an existing accepted ownership, it does not create a new boundary. If adding the row makes `tiler.contract.architecture` a genuine destination for ADR 0073, extend that ADR's `applies_to` in the same change so the typed edge and the prose agree; that requires `contracts/decisions`, so either add the scope before starting or hand the ADR half to a ticket that holds it. Run the full documentation gate before completion.

## Outcome

The `tiler-compiler` row in `docs/architecture.md`'s component-ownership table now reads "Normalization, rule engine, fusion planning, index lowering, schedule search, costing, typed explain infrastructure", recording the explain ownership that ADR 0070 already assigns to `tiler-compiler` (its "explanations" clause) and that ADR 0073 governs. The row's existing comma-only list style and `Candle` forbidden-dependency cell are unchanged.

Because the Component-ownership section is ADR-derived and therefore normative in this `mixed` contract, `tiler.contract.architecture` became a genuine `applies_to` destination for ADR 0073. Per the metadata contract (`applies_to` is the ADR-to-normative-contract edge; contract `governed_by` is derived from it), ADR 0073's frontmatter now declares `applies_to: ["tiler.contract.optimizer", "tiler.contract.architecture"]` so the typed governance edge agrees with the prose — mirroring how the refined ADR 0070 already lists architecture.

`docs/decisions/README.md` is a generated catalog; `scripts/docs.py render` regenerated ADR 0073's thematic entry to add the `System architecture` contract link, a deterministic view over the edited frontmatter. That file maps to `contracts/navigation`, added here as a shared scope per the catalog-regeneration convention. `contracts/decisions` was added as an exclusive scope to cover the ADR edit.
