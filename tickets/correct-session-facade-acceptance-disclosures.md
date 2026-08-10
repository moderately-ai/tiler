---
id: correct-session-facade-acceptance-disclosures
title: Correct residual draft disclosures for the accepted session facade
status: in-progress
priority: p2
dependencies: [accept-the-public-compiler-facade-boundary]
related: [state-and-check-a-bf16-numerical-contract, clarify-the-inline-frontend-facades-consumer-scope]
scopes: [contracts/navigation, contracts/foundation, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [docs, public-boundary, disclosure]
claimed_from: todo
assignee: terra-session-disclosures
lease_expires_at: 1786391758
---
## What this owes

Tom accepted `tiler_compiler::session` in full on 2026-08-05 (five items) and 2026-08-06 (`compile_governed` returned exclusion) under [`accept-the-public-compiler-facade-boundary`](accept-the-public-compiler-facade-boundary.md). Acceptance is not stabilization. The owning module and the optimizer conformance inventory already say so:

- `crates/tiler-compiler/src/session.rs` opens `This boundary is **accepted** under ADR 0075 and ADR 0074 convention 7`
- `docs/correctness-and-testing.md` states `the facade is accepted in full` and keeps acceptance ≠ stabilization

Several portals and the consumer facade crate still describe the session surface (or its BF16 contract additions) as an unaccepted reviewed experimental draft. That is post-acceptance disclosure drift, not a re-decision of the six-item surface.

## Sites to align

Rewrite each live present-tense draft claim to accepted (not stabilized) wording that matches `session.rs` / correctness-and-testing. Do not re-open item-level acceptance; do not stabilize.

1. `docs/status.md` compiler bullet — anchor `facade remains a reviewed experimental draft as a whole` → accepted in full / may still reshape during alpha only as a stability claim, not as unaccepted draft.
2. `docs/status.md` BF16 clause — anchor `recorded above as a reviewed experimental draft; no separate public-boundary acceptance is claimed for them here` → the named BF16 contract items were accepted under [`state-and-check-a-bf16-numerical-contract`](state-and-check-a-bf16-numerical-contract.md) `## Surface accepted — 2026-08-05`; still not a full dtype vertical guarantee.
3. `docs/architecture.md` — anchor `The session module remains a reviewed experimental draft as a complete facade` → accepted complete facade, acceptance not stabilization; shape-environment and budgets remain governed as today.
4. `docs/roadmap.md` — anchor `a public reviewed-draft session facade` → accepted-not-stabilized session facade.
5. `crates/tiler/src/lib.rs` crate header — anchors `reviewed experimental draft rather than an accepted or stabilized API` and `accept-the-public-compiler-facade-boundary owns that decision` → session is accepted (not stabilized); stop naming this ticket as an open owner. Keep the load-bearing claim that `tiler` is the inline frontend facade and not the general semantic-program entry point.
6. `docs/dtype-support.md` — anchor `likewise a reviewed experimental draft rather than an accepted boundary` (BF16 session additions) → accepted contract items per the sibling Surface accepted record; still not a full BF16 vertical guarantee.

## Explicitly not in scope

- No signature, behaviour, identity, or ABI change.
- No re-decision of `compile`, `compile_governed`, `CompileRequest`, `InstalledCapabilities`, `Compilation`, or `CompileFailureClass`.
- Unrelated "reviewed draft" surfaces (runtime adapter, `tiler::value`, artifact program, measured cost row if still labelled, etc.) stay out of this sweep unless a grep hit is only about **this** session facade.

## Closes when

Session/facade-scoped residual draft language is gone or historical/struck only. Suggested check:

```sh
rg -n "reviewed experimental draft|reviewed-draft session|reviewed draft" docs/status.md docs/architecture.md docs/roadmap.md docs/dtype-support.md crates/tiler/src/lib.rs
```

Every remaining hit is either about a different surface or is inside a dated historical/struck passage. `make citations` if ticket links move. If `crates/tiler` rustdoc text changes: `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler --no-deps`.

## Graph

Filed 2026-08-10 from the Phase B audit repair of [`accept-the-public-compiler-facade-boundary`](accept-the-public-compiler-facade-boundary.md). Depends on that closed acceptance so the target wording is the accepted-in-full record, not a reopen.
