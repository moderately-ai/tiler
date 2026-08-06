---
id: move-the-runtime-semantic-validation-cells-for-f32-and-bf16
title: Move the runtime semantic validation cells for f32 and BF16
status: todo
priority: p3
dependencies: [validate-bf16-at-the-runtime-routing-boundary]
related: []
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, dtype, bf16, runtime, maturity-matrix]
---
## User-visible outcome

`docs/dtype-support.md`'s `Runtime semantic validation` column stops reading `absent/unsupported` for the two dtypes that now have a runtime refusal, and says exactly how far that refusal goes.

## Why this is a separate ticket

**Fact.** `validate-bf16-at-the-runtime-routing-boundary` holds `implementation/runtime` and could not edit `docs/dtype-support.md`, which `ticketsplease.toml` maps to `contracts/navigation`.

**Fact.** The mechanism that ticket added is dtype-neutral: `ExecutionEnvironment::classify_dtype` is keyed by `ArithmeticType` and the eligibility filter resolves whatever arithmetic an entry records. Its suite exercises `f32` on all three declared families as well as BF16, so both rows are supported by the same evidence rather than one being inferred from the other. Its own graph-maintenance note required the `f32` row be filed separately rather than claimed, which is this ticket.

## Implementation keys

- The cell states a **bounded** guarantee. What is tested is refusal at the routing boundary — an undispatchable or unmeasured dtype filters its variant before ADR 0051's commit, and the two refusing resolutions are distinguishable. It is not evidence about execution, and BF16's `Backend execution` cell stays `absent/unsupported`.
- Whatever wording the cell takes, the same wording covers both rows or the difference between them is stated.
- Check the prose section the cells link to for sentences that the change makes false, and correct them in the same edit. Nothing validates this corpus.

## Closes when

Both cells state what is now true with their boundary named, the linked prose agrees with them, and no sentence elsewhere in `docs/dtype-support.md` still describes runtime dtype validation as absent.
