---
id: choose-the-expansion-cache-root-policy
title: Choose the expansion-cache root policy
status: todo
priority: p2
dependencies: [admit-the-tiler-facade-and-proc-macro-crate-boundary]
related: [exercise-the-expansion-cache-under-cargo-and-rust-analyzer, prototype-inline-proc-macro-frontend, prototype-macro-embedding-and-cargo-behavior]
scopes: [implementation/frontend, research/cache, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [cache, frontend, proc-macro]
---
## User-visible outcome

An inline expansion resolves a cache root by a stated, explicit policy rather than by whatever its caller happened to pass, and Q-ART-004 has a live owner instead of a `done` one.

## Why this exists

The [build-tool exercise](../docs/research/cache/build-tool-exercise.md) deferred this with the trigger "the first proc-macro frontend crate. Nothing can decide it earlier, because there is no caller to own the choice." **That trigger fired on 2026-07-31**, when `tiler` and `tiler-macros` were admitted under [ADR 0088](../docs/decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md). The caller now exists and the choice is still unmade.

`docs/open-questions.md`'s Q-ART-004 named [`prototype-expansion-content-cache`](prototype-expansion-content-cache.md) as its owner, and that ticket is `done` with the close condition unmet — the same failure mode Q-ART-008 recorded when its owner closed terminal, which is a question that reads as owned while being unowned in fact. [`add-an-expansion-cache-root-preflight`](add-an-expansion-cache-root-preflight.md) is also `done`; it owns validating whatever root is chosen, never choosing one.

## Implementation keys

**Fact — `tiler-cache` has no default and must not acquire one.** `ExpansionCache::open(root)` takes the root from its caller, performs no I/O, and creates no directory; the crate never consults the environment. The reproducible check the research note states: `grep -rn "env::var\|var_os\|home_dir\|dirs::\|std::env" crates/tiler-cache/src/ --include='*.rs'` matches only `#[cfg(test)]` modules. A host-relative decision inside a storage protocol is the boundary that keeps the protocol testable without a host, so the chooser belongs to the frontend.

**Fact — `tiler-macros` names no root today.** `grep -rn 'ExpansionCache\|cache' crates/tiler-macros/src/` reports no match. Nothing has been decided by default.

Two constraints are already decided and bound the answer:

- `ExpansionCache::open`'s own contract requires a root "private to the user running Tiler".
- A root derived from `CARGO_MANIFEST_DIR` is *measured* reachable under both build tools, which matters because `rust-analyzer` populates a proc-macro's environment from the crate graph it loaded rather than from the editor's process environment — so a variable the editor carries does not necessarily arrive. The same note measured that `CARGO_PKG_NAME` does not distinguish the two drivers and that `std::env::current_exe()` does.

The choice must be explicit rather than defaulted into a home directory, and a documented override must exist for CI and sandboxed builds — `docs/integration/frontends.md`'s Compiler cache section states both as accepted contract.

## Public boundary for Tom

Whatever surface states or overrides the root is a new publicly reachable path on the frontend and needs Tom's review under ADR 0075. Present the exact spelling, the override mechanism, and the failure text a consumer sees for an unusable root before acceptance.

## Closes when

A stated root policy is implemented in the frontend with an explicit override; an unusable or non-private root produces a typed refusal a consumer can read rather than a silent miss; the choice and its rejected alternatives are recorded where a reader finds them; Q-ART-004 is retargeted or closed against this work rather than against a terminal ticket; and the deferred item in the build-tool exercise note is updated with the outcome.

## Graph maintenance

- Do not absorb the *validation* of a chosen root: `add-an-expansion-cache-root-preflight` already delivered that and this ticket supplies it an input.
- Keep the measurement questions in `prototype-macro-embedding-and-cargo-behavior`; that ticket presumes a root exists and does not choose one.
- Q-ART-004 also names accounting and GC policy. If this ticket settles only the root, split the remainder rather than closing the question against a partial answer — `decide-the-expansion-cache-collection-schedule` is `deferred` and holds the collection half.
