---
id: repair-the-private-intra-doc-links-the-public-rustdoc-gate-cannot-see
title: Repair the private intra-doc links the public rustdoc gate cannot see
status: todo
priority: p3
dependencies: []
related: [state-the-rule-that-a-deterministic-budget-is-a-derivation]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

Every intra-doc link in `tiler-compiler`'s private items resolves, and the population the public rustdoc gate cannot reach is either covered by a check that can fail or truthfully recorded as unchecked.

## Why this exists — filed 2026-08-18 from the deterministic-budget delivery

**Fact (worker-reported, coordinator-relayed; re-verify at your base).** The workspace gate runs `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`, which never renders `pub(crate)` items, so a broken intra-doc link in private docs cannot fail it — the AGENTS.md case about confirming a check reaches its subject. `cargo doc --no-deps --document-private-items -p tiler-compiler` exits 101 at base `2e1cef3c` on **sixteen pre-existing broken intra-doc links** across `cover.rs`, `estimate.rs`, `frontier.rs`, `governed.rs`, `lowering.rs`, `pipeline.rs`, `pipeline/trace.rs`, `program.rs`, `region.rs`, `request.rs`, and `target/feasibility.rs`. Reproduce with that command and enumerate the exact population at your own base before repairing.

## Required content

- Repair each broken link (or convert to plain text where the target genuinely has no path from the doc's scope), reading each doc's intent rather than mechanically satisfying the resolver.
- Decide and record whether `--document-private-items` should join a gate: if yes, wire it and make it fail deliberately once (quote the failure); if no, record where the unchecked population is and why the cost is declined. Do not silently leave the check unreachable while implying coverage.
- Census the final state: the command exits 0, or the ticket names exactly what remains unresolved and why.

## Closes when

The private-items rustdoc run is clean for tiler-compiler (or its residual is enumerated with reasons), the gate decision is recorded, and any new check has been observed failing.
