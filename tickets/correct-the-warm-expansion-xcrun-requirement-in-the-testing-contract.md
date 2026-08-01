---
id: correct-the-warm-expansion-xcrun-requirement-in-the-testing-contract
title: Correct the warm-expansion xcrun requirement in the testing contract
status: todo
priority: p2
dependencies: []
related: [avoid-toolchain-resolution-on-a-warm-expansion-cache-hit, refresh-the-inline-aot-vertical-status-and-remaining-checks]
scopes: [contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, testing, macro-aot]
---
## User-visible outcome

The proc-macro AOT test list stops requiring a test that cannot pass, so a reader building the inline vertical is not chasing a requirement the accepted design made unreachable by construction.

## Why this exists

**Fact — the requirement stands.** `docs/correctness-and-testing.md:336` reads "Equivalent warm expansions perform no `xcrun` work, including across rustc processes."

**Fact — the accepted design makes it unreachable by construction.** `docs/integration/frontends.md:349-357`, the 2026-08-01 correction under "Why a warm expansion resolves the toolchain", records that the corresponding frontend bullet "read 'warm IDE and `cargo check` expansion must avoid `xcrun`' until 2026-08-01. It was corrected rather than implemented." The structural reason is at `:353`: the compiler fingerprint is an *input* to compilation identity, so `Toolchain::prepare` must observe the toolchain before a lookup exists to skip — reaching a cache entry without observing the toolchain would key on something other than the compiler that would build a miss, the incomplete-key failure ADR 0050 exists to exclude. The corrected invariant at `:357` is narrower and stronger: "identity must fold a fingerprint read by executing the binaries the same prepared token will execute."

**Fact — the measurement is in the same file.** `docs/integration/frontends.md:359-369` records a warm `Toolchain::resolve()` at 44–97 ms with its `xcrun` calls itemized, and `:371` records that a resolution now makes four rather than five since `drop-the-unread-sdk-path-from-the-resolved-toolchain`. Warm expansion resolves the toolchain; it does not avoid it.

**Inference — a test requirement that cannot pass is a validation defect, not stale prose.** It reads as an unmet obligation and would send a worker to implement something the accepted design rules out. [`avoid-toolchain-resolution-on-a-warm-expansion-cache-hit`](avoid-toolchain-resolution-on-a-warm-expansion-cache-hit.md) is `done` and carries the derivation and the numbers, but it never held `contracts/numerics`, so it structurally could not have swept this file — verify with `grep -n "scopes:" tickets/avoid-toolchain-resolution-on-a-warm-expansion-cache-hit.md`.

## Work

Replace `docs/correctness-and-testing.md:336` with the requirement the accepted invariant actually supports — that a warm expansion performs no *compilation* work and that identity folds a fingerprint read by executing the binaries the prepared token will execute — and cite `docs/integration/frontends.md`'s correction as the derivation rather than restating it. While in the list, check the neighbouring proc-macro AOT bullets against the same file for the same class of defect; a sibling of a found bug is the highest-yield place to look.

## Boundaries

Scope is `contracts/numerics` — this file only. The frontend contract's own stale sentences belong to [`refresh-the-inline-aot-vertical-status-and-remaining-checks`](refresh-the-inline-aot-vertical-status-and-remaining-checks.md), which holds `contracts/integrations`; do not reach into that file, and do not re-derive the correction it already carries.

## Closes when

No requirement in the proc-macro AOT test list is unreachable by construction under the accepted design; the replacement requirement is one a test could actually be written against; and each remaining bullet in that list was checked against `docs/integration/frontends.md` rather than only the one this ticket names.
