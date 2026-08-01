---
id: refresh-the-inline-aot-vertical-status-and-remaining-checks
title: Refresh the inline AOT vertical's status line and remaining-checks list
status: in-progress
priority: p2
dependencies: []
related: [avoid-toolchain-resolution-on-a-warm-expansion-cache-hit, correct-the-warm-expansion-xcrun-requirement-in-the-testing-contract, deliver-an-artifact-family-from-a-symbolic-region]
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, macro-aot, status-drift]
claimed_from: todo
assignee: worker-aot-status
lease_expires_at: 1785609794
---
## User-visible outcome

`docs/integration/frontends.md` stops contradicting itself: its status line and its must-still-demonstrate list agree with the correction and the measurement table the same file carries.

## Why this exists

**Fact — the status line is falsified by the same file.** `docs/integration/frontends.md:15` reads "**Status:** accepted inline AOT contract; rust-analyzer performance remains unmeasured." The measurement table at `:359-369` records `rust-analyzer 1.97.0-nightly (8b03437a 2026-05-12)` on macOS 27.0 arm64 / Apple M4 Max, 2026-08-01, with a live in-region edit `semanticTokens` round trip at 137–217 ms delivering against 10–16 ms fallback-only. It is measured.

**Fact — two clauses in the must-still-demonstrate list are falsified the same way.** `:439-444` reads "The first vertical implementation slice must still demonstrate an actual Tiler macro compiling, embedding, loading, and dispatching a one- and multi-entry bundle; **a production warm cache hit invoking no `xcrun`**; source-spanned retained MSL diagnostics; and the non-Apple semantic fallback path without consumer setup. **rust-analyzer cold/warm performance also remains unmeasured because the component was unavailable.**" The first is unreachable by construction under the correction at `:349-357` — a warm expansion resolves the toolchain, because the compiler fingerprint is an input to compilation identity and `Toolchain::prepare` must observe it before a lookup exists to skip. The second is contradicted by the table twenty lines earlier, whose own preamble records that the analyzer binary and proc-macro server "were both already present" — the unavailability that clause cites is over.

**Inference — sweep the list, do not patch two clauses.** Several other items in that list have landed since it was written, and a repair that fixes exactly the two an audit named leaves the same defect class in the same paragraph. Read every item against the file's own later sections and against `crates/`.

## Boundaries

- Scope is `contracts/integrations` — this file and its siblings. The parallel unreachable requirement at `docs/correctness-and-testing.md:336` belongs to [`correct-the-warm-expansion-xcrun-requirement-in-the-testing-contract`](correct-the-warm-expansion-xcrun-requirement-in-the-testing-contract.md), which holds `contracts/numerics`; do not reach into that file.
- **Coordinate with [`deliver-an-artifact-family-from-a-symbolic-region`](deliver-an-artifact-family-from-a-symbolic-region.md):28**, which holds `contracts/integrations` and is required to sweep two *other* named sentences in this file — the symbolic-region family-selection refusal and the two sentences a previous ticket flagged without scope to fix. The two sweeps do not overlap; confirm that by reading its bullet before starting, and do not absorb its work or leave it re-fixing yours.
- A measurement is bounded by its environment. Replacing "unmeasured" with a number means carrying the number's host, date, toolchain, and procedure with it, not asserting a portable property.

## Closes when

`docs/integration/frontends.md:15` states what is actually unmeasured; no item in the must-still-demonstrate list is unreachable by construction or already discharged elsewhere in the same file; every remaining item was checked rather than the two an audit named; and each replacement measurement carries its exact environment and procedure.
