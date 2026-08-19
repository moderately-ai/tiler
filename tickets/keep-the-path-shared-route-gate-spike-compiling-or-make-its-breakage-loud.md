---
id: keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud
title: Keep the path-shared route-gate spike compiling or make its breakage loud
status: todo
priority: p2
dependencies: []
related: [demote-the-m3-pro-subgroup-declaration-to-an-internal-evidence-fixture]
scopes: [implementation/runtime, research/target-profiles, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`spikes/target-profiles/metal-subgroup-width-route-gate` either compiles at main or its breakage is a recorded, checked state — never a silent rot discovered by the next person who runs it.

## Why this exists — filed 2026-08-18 at integration of the demotion lane

**Fact (verified by the demotion worker; re-verify at your base).** The spike reuses `crates/tiler-runtime/tests/adapter_route/fixture.rs` via `#[path]`. Commit `2cb7c83c` (the p0 live-extent association landing) added four `crate::adapter::ScalarEnvironmentSchema` references to that fixture, which do not resolve inside the spike's separate crate — `cargo check --locked` in the spike fails with four `error[E0433]` at `50327207`, before any visibility change. The m3pro demotion then added a second, independent break (the declaration is now `pub(crate)`), recorded as a build exception in the spike README gated on `decide-the-host-evidence-to-profile-composition-model`.

**The general defect:** nothing checks that a `#[path]`-shared fixture keeps its non-owning consumers compiling. Either give the spike a crate-local shim over the shared fixture (so the sharing has one owner and a compiling consumer), or record + check the broken state (the spike catalogue row and frontmatter now say `blocked` — keep them truthful), or retire the sharing arrangement with a documented copy. Choose with reasons; do not leave the arrangement silently rot-prone.

## Closes when

The spike compiles at main or its blocked state is recorded and mechanically visible; the fixture-sharing arrangement has a stated owner and a stated check (or a stated reason none is owed); and the spike's module doc no longer names the now-private public path.
