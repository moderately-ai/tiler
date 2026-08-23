---
id: census-explain-disposition-reachability-inside-the-compiler
title: Census explain disposition reachability inside the compiler
status: todo
priority: p3
dependencies: []
related: [make-explain-dispositions-assertable-by-a-conformance-suite, decide-the-backend-provider-conformance-harness-public-surface]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [explainability]
---
## User-visible outcome

Which `ExplainDisposition` variants a compilation can actually reach is a counted, typed population that fails when it shrinks, rather than a maturity claim in an `#[allow]` reason.

## Why this exists

Filed 2026-08-22 by `worker-packet` as the named residual of the second re-derivation on `decide-the-backend-provider-conformance-harness-public-surface`. That packet recommends publishing no conformance facade, which closes `make-explain-dispositions-assertable-by-a-conformance-suite` — and closing it must not be read as discharging the underlying obligation.

**Fact — the obligation is real and is not conformance-owned.** `docs/operation-extensions.md` makes it one of the properties jointly admitting a public extension seam that every disposition is a distinct typed outcome reaching the explain trace.

**Fact — part of the vocabulary is reserved and unconstructed, and the compiler says so itself.** `crates/tiler-compiler/src/explain.rs` carries a crate-level allow whose reason reads `what stays unconstructed is the reserved evidence, quantity, disposition, and subject vocabulary the bounded profile does not yet produce`. `ExplainDisposition` is `pub(crate)` with sixteen variants at that base, `BudgetStopped` among them.

**Fact — nothing counts it.** `grep` for a `variant_count` pin or an `ALL` array over `ExplainDisposition` in `crates/tiler-compiler/src/explain.rs` finds neither. Re-verify this at your base; the point of the ticket is the missing instrument, so a pre-existing one would retire it.

## Required work

- Size the population from the type with `core::mem::variant_count`, not by hand. A hand-written length can be satisfied by an enumeration that has stopped covering its domain.
- Print the reached and reserved populations, so `nothing ran` cannot look green, and assert a floor on reached rather than an equality that would forbid reserving a variant.
- Perturb the subject: make one currently reached disposition unreachable and quote what the census said. Widen the enum by one variant and quote the build error.
- This is compiler-internal. It adds no public item; `ExplainDisposition` stays `pub(crate)`, and a structured public accessor remains a separate, unaccepted public-boundary question.

## Closes when

The reached and reserved disposition populations are derived from the type, printed, and held to a floor that has been watched failing; and no rendered explain string is read as a parse target anywhere in the check.
