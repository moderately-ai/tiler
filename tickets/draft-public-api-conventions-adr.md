---
id: draft-public-api-conventions-adr
title: Draft a proposed ADR for public API shape conventions
status: todo
priority: p1
dependencies: []
related: [draft-public-boundary-approval-policy-adr, draft-public-extension-seam-ownership-adr, harden-public-enums-non-exhaustive, extend-canonical-identity-encodings-for-reserved-variants]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions, public-api, governance]
---
Record, as a **proposed** ADR, the conventions a public Tiler API must satisfy.
The point is to make conformance mechanically checkable so that a public surface
which follows the conventions needs no bespoke design debate, and only genuine
deviations need a decision.

## Why now (evidence, not speculation)

These conventions are not invented here; the codebase already follows them, but
nowhere states them, so each new surface re-litigates them by review. Observed
across the authorities landed so far: four independent agents converged on the
same private-draft authority shape (`pub(crate)` module plus a module-level
`#![allow(dead_code, reason = …)]` until it is wired into `compile()`), matching
the earlier `explain`/`feasibility` precedent, without being instructed to. Over
the same period, per-case public-boundary review caught essentially one
substantive issue — missing `#[non_exhaustive]` on enums and output records
documented as growing — which is a *convention gap*, exactly the class a written
rule catches systematically and a human review catches only by luck.

## Conventions to record (proposal — each open to revision)

Draw them from the landed surfaces rather than from theory:

- **Errors are typed and non-erasing.** Distinct failure kinds stay distinct
  variants; generics over each layer's concrete error types rather than
  `Box<dyn Error>` (see `CheckedBuildError<Admission, Verification>`).
- **Identity types are opaque**, expose canonical bytes via `as_bytes()`, and any
  short digest is presentation-only (`key()`), never an equality or dedup input.
- **Canonical encodings are length-prefixed, domain-separated, exclude transient
  ordinals, and match enums exhaustively** so a new variant is a compile error
  rather than a silent identity collision (this is the trap
  `extend-canonical-identity-encodings-for-reserved-variants` fixes).
- **Construction is a transactional builder plus a consuming `build()`** that
  returns an opaque verified product which cannot be forged or thawed; a
  closure convenience must delegate to that same path.
- **`#[non_exhaustive]` on any public enum or output record documented as
  growing** (owned as work by `harden-public-enums-non-exhaustive`).
- **Verified products expose no `pub` fields**; leaf value-data descriptors may,
  and the ADR should state which is intended where (see the schedule-vs-index
  inconsistency recorded on `unify-schedule-index-region-with-verified-index-region`).

## Deliverable and boundaries

Create the ADR at the next free number (highest today is 0073) with
`decision_status: "proposed"`, its `ticket` field pointing at this ticket, an
honest Context citing the observed evidence above, and every unresolved point
listed explicitly as an open question rather than silently settled. Do **not**
mark it accepted: acceptance is Tom's, and a separate step.

This ticket only records the conventions. It changes no code and does not itself
retrofit any existing surface; the conforming work is owned by the implementation
tickets referenced above, which may in turn clarify or amend open questions this
ADR leaves explicit.

Run `uv run --locked python scripts/docs.py render` (the decisions catalog is a
generated view — edit source frontmatter, never the generated list) and the full
documentation gate before completion.
