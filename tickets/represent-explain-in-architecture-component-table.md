---
id: represent-explain-in-architecture-component-table
title: Represent explain authority in the architecture component table
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/foundation, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, architecture]
claimed_from: todo
assignee: agent-represent-explain-in-architecture-component-table
lease_expires_at: 1784832272
---
ADR 0073 records that typed explain infrastructure is owned by `tiler-compiler`, and declares `applies_to: ["tiler.contract.optimizer"]` only. That was deliberate and correct: `docs/architecture.md`'s component table lists `tiler-compiler` as "Normalization, rule engine, fusion planning, index lowering, schedule search, costing" without enumerating explainability, so claiming `applies_to` over architecture would have asserted authority the document does not currently represent — and the authoring ticket did not hold `contracts/foundation`.

Add explainability to that component row so the architecture contract represents what ADR 0070 already places in `tiler-compiler` and ADR 0073 now governs. Keep the edit minimal: this records an existing accepted ownership, it does not create a new boundary. If adding the row makes `tiler.contract.architecture` a genuine destination for ADR 0073, extend that ADR's `applies_to` in the same change so the typed edge and the prose agree; that requires `contracts/decisions`, so either add the scope before starting or hand the ADR half to a ticket that holds it. Run the full documentation gate before completion.
