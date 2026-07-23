---
id: update-adr-0071-schedule-builder-boundary
title: Update ADR 0071 schedule-builder implementation-boundary note
status: todo
priority: p2
dependencies: []
related: [prototype-scheduled-region-ir]
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

Update ADR 0071 to record the implemented schedule builder/verifier, superseding
the durable "unimplemented" statement explicitly rather than silently (per the
documentation contract for superseding accepted decisions). Keep the ADR's
original rationale intact; note only what evidence changed. If the edit makes any
new normative contract a genuine `applies_to` destination, extend the frontmatter
edge so prose and typed edge agree, and regenerate `docs/decisions/README.md`
(the catalog is a generated view — edit source metadata, not list items).

Run `uv run --locked python scripts/docs.py render` and the full documentation
gate before completion.
