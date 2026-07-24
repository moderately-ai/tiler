---
id: update-adr-0071-schedule-builder-boundary
title: Update ADR 0071 implementation-boundary notes (schedule builders and closure convenience)
status: todo
priority: p2
dependencies: [add-checked-closure-convenience-for-shared-ir-builders]
related: [prototype-scheduled-region-ir, add-checked-closure-convenience-for-shared-ir-builders]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions]
---
ADR 0071's implementation-boundary note states that schedule builders remain
unimplemented. `prototype-scheduled-region-ir` merged `tiler_ir::schedule` with a
real `ScheduledRegionBuilder` and opaque `VerifiedScheduledRegion` following the
same checked-builder discipline the ADR governs, so that note is now partially
superseded.

A second part of ADR 0071's accepted ergonomic layer — the closure-based
convenience over the shared IR builders — was also described as unimplemented.
`add-checked-closure-convenience-for-shared-ir-builders` implements it (for
`IndexRegionBuilder` first) but is scoped to `implementation/ir` and deliberately
defers the ADR 0071 decision-doc status edit here, so this is the single
consolidated owner of ADR 0071's implementation-status updates. This is why it
depends on that ticket: the ADR should reflect the implemented state.

Update ADR 0071 to record BOTH the implemented schedule builder/verifier
(`tiler_ir::schedule`) AND the implemented closure convenience, superseding the
durable "unimplemented" statements explicitly rather than silently (per the
documentation contract for superseding accepted decisions). Keep the ADR's
original rationale intact; note only what evidence changed. If the edit makes any
new normative contract a genuine `applies_to` destination, extend the frontmatter
edge so prose and typed edge agree, and regenerate `docs/decisions/README.md`
(the catalog is a generated view — edit source metadata, not list items).

Run `uv run --locked python scripts/docs.py render` and the full documentation
gate before completion.
