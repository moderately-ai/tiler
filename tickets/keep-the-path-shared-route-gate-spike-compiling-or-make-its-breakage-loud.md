---
id: keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud
title: Keep the path-shared route-gate spike compiling or make its breakage loud
status: done
priority: p2
dependencies: []
related: [demote-the-m3-pro-subgroup-declaration-to-an-internal-evidence-fixture]
scopes: [implementation/runtime, research/target-profiles, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
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

## Fact audit at base `f7a356de` — 2026-08-19, worker-routegate

Every claim below was re-read at this base before any edit. Commands are reproducible from the repository root.

- **Verified — the spike reuses the runtime fixture through `#[path]`, and takes `image.rs` the same way.** `spikes/target-profiles/metal-subgroup-width-route-gate/src/main.rs`, anchors `mod fixture;` and `mod image;`, each preceded by a `#[path]` attribute naming `crates/tiler-runtime/tests/adapter_route/`. The ticket names only `fixture.rs`; two modules are shared, and the check delivered below covers both. Imprecise only in extent, not in direction.
- **Verified — `2cb7c83c` added four `crate::adapter::ScalarEnvironmentSchema` references.** `git show 586c508a:crates/tiler-runtime/tests/adapter_route/fixture.rs | grep -c 'crate::adapter'` returns `0`; the same at `2cb7c83c` returns `4`; at this base `grep -c 'crate::adapter' crates/tiler-runtime/tests/adapter_route/fixture.rs` returned `4`.
- **Verified, and the count was understated by one.** `cargo check --locked` in the spike directory at this base failed with **five** errors, not four: the four `error[E0433]: cannot find adapter in crate` the ticket names, plus `error[E0432]: unresolved import tiler_build::BoundMetalSubgroupDeclaration`. The ticket is internally consistent — it names the `E0432` separately in the release note — so this is a reading hazard rather than a false claim, recorded because "four `error[E0433]`" and "`cargo check` fails with four errors" are one sentence apart.
- **Verified — the demotion break is real and unreachable from another crate.** `crates/tiler-build/src/metal_subgroup_declaration.rs`, anchor `pub(crate) struct BoundMetalSubgroupDeclaration`; the crate root exports no such name (`grep -n 'BoundMetalSubgroupDeclaration' crates/tiler-build/src/lib.rs` returns nothing).
- **Verified — rustc's diagnostic for that import is actively misleading**, which the ticket does not say and which changes what "loud" has to mean here. The `E0432` above carries ``help: a similar name exists in the module`` proposing `BoundMetalCompileDeclaration` — a different declaration, whose width would be a different number arriving silently. Recorded because a worker following the compiler's suggestion produces wrong evidence rather than a build failure.
- **Verified — the catalogue row and frontmatter say `blocked`.** `spikes/target-profiles/metal-subgroup-width-route-gate/README.md`, anchor `experiment_status: "blocked"`; `spikes/README.md`, anchor `blocked, permanently as of 2026-08-19`.
- **Verified — the four cited commits exist and are what the ticket says.** `2cb7c83c` *Associate live extent operands with source-bearing semantic axes*; `50327207` *File profile-row reseat carrier; claim contraction replacement and demotion lanes*; `60235bb2` *Close the item-25 acceptance sweep*; `586c508a` *Declare the evidence-backed M3 Pro Metal subgroup realization*.
- **Verified — the spike's module doc named the now-private public path.** `spikes/target-profiles/metal-subgroup-width-route-gate/src/main.rs`, retired anchor `tiler_build::BoundMetalSubgroupDeclaration::first_m3_pro_apple9` inside the `//!` block. Repaired below.
- **False, in the ticket's framing rather than its wording — "nothing checks that a `#[path]`-shared fixture keeps its non-owning consumers compiling" is true of `fixture.rs` and was already false of `image.rs`.** `crates/tiler-runtime/tests/identity_join/main.rs`, anchor `#[path = "../adapter_route/image.rs"]`, is a second non-owning consumer that has been inside the ordinary package gate the whole time — which is exactly why `image.rs` never acquired a back-edge and `fixture.rs` did. The defect is narrower and more diagnosable than stated: the shared set had one member with an in-gate second root and one without. That observation is what the delivery is built on.

## Delivered — 2026-08-19

**Repaired, not merely recorded.** `ScalarEnvironmentSchema` moved from `tests/adapter_route/adapter.rs` to `tests/adapter_route/fixture.rs`, where both of its constructors already lived — `producer_schema` is private to `fixture.rs` and cannot reach out of it, so declaring the type in `adapter.rs` made a back-edge the module's non-owning consumers could not resolve. `adapter.rs` now imports it (`use crate::fixture::{self, ScalarEnvironmentSchema};`) and the shared modules reach only each other. One intra-doc link in `image.rs` that rooted at `crate::adapter` became a code span for the same reason.

**Owner and check, both stated in source.** The owner is `crates/tiler-runtime/tests/adapter_route`, stated in that directory's own `fixture.rs` module documentation under the heading `# This module is path-shared, and what that costs`. The check is the new test target `crates/tiler-runtime/tests/adapter_route_portability.rs`, which compiles the shared set from a second root — the idiom `prototypes/serial-sum-run/tests/lint_table.rs` already uses one layer up — and additionally enumerates every `#[path]` consumer in the repository, failing closed on any spelling it cannot resolve. rustc is the checker for portability, because a grep for `crate::` would miss an alias, a macro-expanded path, and a re-export.

**Made loud where the failure happens.** `src/main.rs` in the spike now carries a `compile_error!` that fires before rustc's misleading `similar name` suggestion, names ADR 0113 as the reason, states that no substitution is admissible, and gives `586c508a` as the reproduction. The spike README, its catalogue row, and the module doc were updated to match; the module doc no longer presents `tiler_build::BoundMetalSubgroupDeclaration` as a reachable path.

**Perturbation evidence — each assertion broken separately, at its subject.**

1. *Portability.* Re-adding the two `crate::adapter::ScalarEnvironmentSchema` references to `fixture.rs` **and** re-exporting the type from `adapter.rs` reproduces `2cb7c83c` exactly: the owning suite compiles clean and only the new target fails, with `error[E0433]: cannot find adapter in crate` at `fixture.rs` and `error: could not compile tiler-runtime (test "adapter_route_portability")`. Before this target existed that state was green.
2. *Coverage.* Adding a third `#[path]` module to the spike's `main.rs` failed the census: `the shared set drifted. 2 consumer(s) among 729 Rust source(s): [...] This target compiles {"fixture", "image"}` with `left: {"determinism", "fixture", "image"}`.
3. *Fail-closed reading.* Wrapping the spike's `#[path]` attribute across two lines failed with `names the shared directory in code outside a literal #[path = "…"] attribute, so whether it is a consumer, and of which modules, cannot be established by reading`.

**What would make the check say no** is stated so it is not taken on faith: (a) any `crate::`-rooted path added to a shared module that the second root cannot resolve — a compile error in the ordinary package gate; (b) any consumer taking a module the target does not compile, or the target compiling one no consumer takes; (c) any reference to the owning directory in Rust code that the reader cannot resolve to one flat module name. All three were driven to failure above and the messages quoted. The `729 Rust source(s)` count printed in (2) is the same walk the passing run performs, so the population is visible rather than assumed.

**What this delivery does not do.** It does not make the spike build, and cannot: reason 1 is permanent by ADR 0113 and lives in `tiler-build`, a scope this ticket does not hold. The portability check is device-free and does not depend on the spike compiling, which is why it survives that.

**Out-of-scope defect found while reading, not repaired.** `crates/tiler-runtime/tests/adapter_route/image.rs` carries one doc block, opening `Runs one scalar entry over its launch grid on the calling thread`, that is attached to `contributor_columns` and describes `execute` — its `# Errors` and `# Panics` sections belong to `execute`, which has no documentation of its own. `missing_docs` cannot see it because the module is private inside a test binary. Adjacent to this ticket's edit and inside its scope, but a different defect with a different cause; it wants its own ticket rather than a silent fix under this one.
