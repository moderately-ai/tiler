---
id: correct-adr-0092-alternatives-considered-prototype-citation
title: Correct ADR 0092's alternatives-considered prototype citation
status: in-progress
priority: p3
dependencies: []
related: [correct-the-sdk-apple-family-range-in-the-runtime-answer-record, close-the-serial-sum-run-gpu-family-probe-table]
scopes: [contracts/decisions]
shared_scopes: [research/runtime, project/tickets]
paths: []
tags: [documentation, decisions, status-drift]
claimed_from: todo
assignee: agent-adr0092
lease_expires_at: 1786049402
---
## User-visible outcome

An accepted ADR stops citing a prototype pair table that no longer exists as evidence for one of its eliminations, so a reader checking the elimination against the tree finds what the record describes.

## Why this exists

**Fact — the citation is present tense and false.** [ADR 0092](../docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md):62, under "Alternatives considered", eliminates "Publish the family vocabulary and let each consumer observe the device itself" on ADR 0074 conventions 5b and 5c, and states: "written as a table rather than a match — **which is what the existing prototype does** — a variant added to the vocabulary compiles cleanly, is never probed, and silently under-reports the device."

**Fact — no prototype pair table survives.** `prototypes/serial-sum-run/src/proof.rs:700-706` now documents the opposite construction: "This is the *binding's* vocabulary and not Tiler's, and the two are joined by Apple's own enumerator value rather than by a pair table. `MTLGPUFamily` is `#[repr(i64)]` with each variant declared at the number `MTLDevice.h` gives it, and `AppleGpuFamilyConstant` carries that same number transcribed from the same header, so the correspondence is arithmetic that already exists rather than a second table someone has to keep in step." [`close-the-serial-sum-run-gpu-family-probe-table`](close-the-serial-sum-run-gpu-family-probe-table.md) is `done`, and the candle adapter's table was removed at `662d9be` ("Make a new Apple GPU family stop the build rather than a device").

**Fact — the flag was raised in the research record and never discharged.** `docs/research/runtime/backend-scoped-route-requirement-answers.md:74` reads "**Fact — the identical table still stands in the second prototype, which this record did not cite**", filed by [`correct-the-sdk-apple-family-range-in-the-runtime-answer-record`](correct-the-sdk-apple-family-range-in-the-runtime-answer-record.md). That sentence is now stale in the same direction, so both sites move together.

**Inference — the acceptance sweep was scoped past it.** The sweep under [`land-the-cpu-vector-lane-tier-adr`](land-the-cpu-vector-lane-tier-adr.md) covered status, disposition, span links, and catalogs — not the body prose of an alternatives-considered section. Nothing validates the corpus, so a citation inside an elimination is checked only by a reader following it.

## Boundaries

- **The elimination is unaffected and must survive the correction.** Convention 5b's argument — a total map with no derivable wildcard value — does not depend on any prototype having written a table; the prototype was an illustration. Do not weaken or restate the elimination while repairing its example, and if a current illustration is wanted, the enumerator-value join is one: it is what the convention *asks for*, so cite it as the positive case rather than the negative.
- Correct `docs/research/runtime/backend-scoped-route-requirement-answers.md:74` in the same change; its `research/runtime` claim is shared here for that reason.
- Do not touch ADR 0092's `decision_status`, `implementation_status`, or the seven unaccepted public-boundary items at `:20` — those belong to [`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md).

## Closes when

ADR 0092:62 describes a construction that exists, or drops the prototype clause without weakening the elimination; the research record's parallel flag at `:74` is discharged in the same change; and both sentences were checked against `prototypes/serial-sum-run/src/proof.rs` as it stands rather than against the ticket that changed it.
