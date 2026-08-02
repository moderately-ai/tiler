---
id: state-a-numerical-contract-in-the-inline-dispatch-spike
title: State a numerical contract in the inline-dispatch spike's two regions
status: in-progress
priority: p2
dependencies: []
related: [state-the-numerical-contract-in-the-region-grammar]
scopes: [research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [spike, frontend, numerics, runtime]
claimed_from: todo
assignee: agent-inline-numerics
lease_expires_at: 1785685152
---
## User-visible outcome

`spikes/runtime/inline-dispatch` builds and runs again: both of its `tiler::tensor!` regions state the numerical contract the frontend now requires.

## Why this is a separate ticket

**Fact.** `state-the-numerical-contract-in-the-region-grammar` made a region that states no numerical contract a refusal at expansion. The spike's two regions — `dispatch_region` and `fallback_only_region` in `spikes/runtime/inline-dispatch/src/main.rs` — state none, so they no longer expand.

**Fact — no gate caught it, and none would.** `spikes/runtime/inline-dispatch` declares its own `[workspace]`, so `make check` and `make full` never reach it; AGENTS.md states that no `make` target reaches `spikes/`. The breakage is real and silent until someone runs the spike by hand from its own directory.

**Fact — why the landing ticket did not fix it.** `spikes/runtime/**` maps to the `research/runtime` scope, which that ticket did not hold and which live spike workers did.

## Implementation keys

- `dispatch_region` states `deliver macos;` and compiles ahead of time, so its contract must be one the bound macOS declaration honours: `flush_subnormals_to_zero_f32` preserves the artifact identity the spike's recorded run was taken under, and `flush_and_reassociate_f32` is the other admissible one. Anything else is a feasibility refusal at the `deliver` keyword.
- `fallback_only_region` states no `deliver`, so nothing checks its contract today; state the same one, and say in the comment beside it that the statement is inert on the fallback path — `check-the-stated-contract-on-the-semantic-fallback-path` owns that gap.
- Re-run the spike by hand per its README and confirm the recorded output still holds, including the artifact identity if the README pins one.

## Closes when

Both regions state a contract, the spike builds and runs from its own directory, and any recorded output or pinned identity it carries is re-verified rather than assumed.
