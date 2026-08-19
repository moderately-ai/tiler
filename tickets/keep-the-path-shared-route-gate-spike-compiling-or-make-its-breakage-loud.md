---
id: keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud
title: Keep the path-shared route-gate spike compiling or make its breakage loud
status: in-progress
priority: p2
dependencies: []
related: [demote-the-m3-pro-subgroup-declaration-to-an-internal-evidence-fixture]
scopes: [implementation/runtime, research/target-profiles, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: worker-routegate
lease_expires_at: 1787164922
---
## User-visible outcome

`spikes/target-profiles/metal-subgroup-width-route-gate` has its breakage as a recorded, checked state — never a silent rot discovered by the next person who runs it. *(Narrowed 2026-08-19: this read "either compiles at main or its breakage is a recorded, checked state"; the first disjunct became unreachable when Tom accepted the composition model, per the release note below.)*

## Why this exists — filed 2026-08-18 at integration of the demotion lane

**Fact (verified by the demotion worker; re-verify at your base).** The spike reuses `crates/tiler-runtime/tests/adapter_route/fixture.rs` via `#[path]`. Commit `2cb7c83c` (the p0 live-extent association landing) added four `crate::adapter::ScalarEnvironmentSchema` references to that fixture, which do not resolve inside the spike's separate crate — `cargo check --locked` in the spike fails with four `error[E0433]` at `50327207`, before any visibility change. The m3pro demotion then added a second, independent break (the declaration is now `pub(crate)`), recorded as a build exception in the spike README gated on `decide-the-host-evidence-to-profile-composition-model`.

**The general defect:** nothing checks that a `#[path]`-shared fixture keeps its non-owning consumers compiling. Either give the spike a crate-local shim over the shared fixture (so the sharing has one owner and a compiling consumer), or record + check the broken state (the spike catalogue row and frontmatter now say `blocked` — keep them truthful), or retire the sharing arrangement with a documented copy. Choose with reasons; do not leave the arrangement silently rot-prone.

## Released and narrowed — 2026-08-19, at acceptance of the composition model

**This ticket is released, and half of its option set is closed.** It was never `blocked` in the graph — its status has been `todo` throughout and it names no dependency — but the README exception it exists beside was written as pending a decision, and that decision has landed: Tom accepted the host-evidence composition model on 2026-08-19 ([ADR 0113](../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md), recorded by [`apply-the-accepted-host-evidence-composition-model`](apply-the-accepted-host-evidence-composition-model.md)). Under its component 3 the M3 Pro width record stays a crate-private evidence fixture **permanently**, so the demotion break in the spike is permanent too.

Three consequences for this ticket, each narrowing it rather than answering it:

1. **"The spike compiles at main" is no longer a reachable outcome.** Repairing the `#[path]`-shared fixture would clear the four `error[E0433]`, and the spike would still not build: `error[E0432]: unresolved import tiler_build::BoundMetalSubgroupDeclaration` is permanent by decision. The outcome disjunction collapses to the recorded-and-checked branch, and the general defect's first option — a crate-local shim that gives the sharing "one owner and a compiling consumer" — keeps its owner half and loses its compiling-consumer half.
2. **The fixture-ownership question is not answered by the accepted model and must stay this ticket's.** ADR 0113 governs how measured host evidence composes into profile identity; it says nothing about who owns a `#[path]`-shared test fixture or what keeps its non-owning consumers honest. That question is about `crates/tiler-runtime/tests/adapter_route/fixture.rs`, whose break at `2cb7c83c` predates the demotion and is independent of it, and it belongs to `implementation/runtime` — a scope this sweep does not hold and a decision this sweep has no evidence for. Answering it here would be inventing an authority the acceptance did not grant.
3. **The third close clause still stands unchanged**: the spike's module doc still names the now-private public path, and that is a spike-source edit this ticket owns.

*(Dated correction to the Fact above: its closing clause, "recorded as a build exception in the spike README gated on `decide-the-host-evidence-to-profile-composition-model`", was true when written and is now historical — that gate is discharged, and the spike README's corresponding sentence carries its own dated supersession beside the retired text.)*

## Closes when

The spike's permanently blocked state is recorded and mechanically visible; the fixture-sharing arrangement has a stated owner and a stated check (or a stated reason none is owed); and the spike's module doc no longer names the now-private public path. *(Revised 2026-08-19: the original first clause read "The spike compiles at main or its blocked state is recorded and mechanically visible"; the first disjunct is unreachable under ADR 0113, per the release note above.)*
