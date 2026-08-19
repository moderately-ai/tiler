---
id: demote-the-m3-pro-subgroup-declaration-to-an-internal-evidence-fixture
title: Demote the M3 Pro subgroup declaration to an internal evidence fixture
status: in-progress
priority: p2
dependencies: [declare-metal-subgroup-realization-facts-in-the-target-profile]
related: []
scopes: [implementation/build, contracts/decisions, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: worker-profile-demotion
lease_expires_at: 1787104037
---
## User-visible outcome

`BoundMetalSubgroupDeclaration` and its `tiler.metal.macos-m3pro-apple9.msl4-0.subgroup-f32.v1` key are crate-private inside `tiler-build`: the pub re-export is removed, the type and factory are `pub(crate)`, and no public identity vocabulary carries a hardware model name. The declaration's validation, tests, and the retained on-device demonstration continue unchanged.

## Why this exists

Tom's 2026-08-18 revised acceptance on `declare-metal-subgroup-realization-facts-in-the-target-profile`: the stage surface is accepted; the host-named public profile is not — host model names belong in measurement provenance, not identity vocabulary. No further host-named profile key may be minted pending `decide-the-host-evidence-to-profile-composition-model`.

## Closes when

The re-export is gone, external unreachability has compile evidence, all existing tests and the spike still pass (the spike may need a crate-internal driver path or a recorded exception), and the module docs state the demotion with its provenance.

## Source-first Fact audit — 2026-08-18 at `50327207`

Per-Fact verdicts, each from a full read of the named source at this base, before any edit.

- **Verified — the demoted population is exactly what the outcome names, and the key string is where it says.** `crates/tiler-build/src/metal_subgroup_declaration.rs` holds `profile_key: "tiler.metal.macos-m3pro-apple9.msl4-0.subgroup-f32.v1"` inside `FIRST_M3PRO_APPLE9_SUBGROUP`, and the crate root re-exported the pair through `pub use metal_subgroup_declaration::` in `crates/tiler-build/src/lib.rs`. The outcome names the type and the factory; the same edit necessarily carries the two accessors (`profile`, `realized_subject`) and the error enum, since the factory's return type is the error and a `pub` accessor on a `pub(crate)` type is a private-interface leak.
- **Verified — the acceptance provenance is stated as the ticket relays it.** `declare-metal-subgroup-realization-facts-in-the-target-profile`, heading `Accepted decision — 2026-08-18, revised after Tom's naming objection`, item 2, anchors `demote from public surface to a crate-private evidence fixture` and `no public identity is minted from a hardware model name`; item 3 names `decide-the-host-evidence-to-profile-composition-model` and the anchor `no further host-named profile key may be minted`.
- **Verified — one public consumer existed outside the crate, and it is the spike.** `spikes/target-profiles/metal-subgroup-width-route-gate/src/main.rs`, anchor `use tiler_build::BoundMetalSubgroupDeclaration`. A repository-wide search found no other non-test consumer; the only remaining references are prose in `tickets/` and `.ticketsplease/decision-queue.md`.
- **Imprecise — "the retained on-device demonstration continue[s] unchanged" is true of the retained record and was already false of the harness at this base.** `spikes/…/metal-subgroup-width-route-gate` reuses `crates/tiler-runtime/tests/adapter_route/fixture.rs` through `#[path]`, and commit `2cb7c83c` added four `crate::adapter::ScalarEnvironmentSchema` references to that fixture which do not resolve in the spike's own crate. `git show 586c508a:crates/tiler-runtime/tests/adapter_route/fixture.rs | grep -c 'crate::adapter'` returns `0`; the same command at `2cb7c83c` and at this base returns `4`. So `cargo check` in the spike already failed at `50327207` with four `error[E0433]: cannot find adapter in crate`, before any visibility moved. The demotion adds a second, independent break; it did not cause the first. Recorded rather than repaired — the fixture is neither this ticket's file nor its scope.

## Delivery — 2026-08-18

Implemented at `9107812e` (the demotion, the compile evidence, the spike exception, and both ticket records) plus the closing commit that adds this hash, on `tkt/demote-the-m3-pro-subgroup-declaration-to-an-internal-evidence-fixture` from base `50327207`. Every check below was run at `9107812e`; the closing commit is a `tickets/`-only delta.

### Visibility census

Moved to `pub(crate)` in `crates/tiler-build/src/metal_subgroup_declaration.rs`: `BoundMetalSubgroupDeclaration`, `BoundMetalSubgroupDeclarationError`, `BoundMetalSubgroupDeclaration::first_m3_pro_apple9`, `BoundMetalSubgroupDeclaration::profile`, `BoundMetalSubgroupDeclaration::realized_subject`. Removed from `crates/tiler-build/src/lib.rs`: the `pub use metal_subgroup_declaration::{…}` re-export of the type and its error.

Unchanged: every row in `FIRST_M3PRO_APPLE9_SUBGROUP` including the profile key string, `MEASURED_PRODUCER`, the private `SubgroupLedgerRows` and `declare`, `measured_source`, all three Metal-owned refusals (`UnevidencedWidth`, `ShuffleUndefinedForArithmetic`, `ControlOnlyArithmetic`), the `From`/`Display`/`Error` impls, `#[non_exhaustive]` on the error, and all eight tests. The key stays verbatim because it is now an evidence label on a crate-private fixture rather than identity vocabulary a consumer can name, and rewriting it would falsify both the retained on-device log and the descriptor assertion in `the_descriptor_carries_the_subgroup_families_and_the_m3_pro_context`.

One addition the ticket did not anticipate: a module-level `#![cfg_attr(not(test), expect(dead_code, reason = …))]`. Removing the re-export left the module with no non-test caller anywhere in the crate, so the demotion produced seven `dead_code` warnings and would have failed `clippy -D warnings`. It is `expect` rather than `allow`, and negated rather than unconditional, following `crates/tiler-conformance/src/publication.rs`: the expectation fires as unfulfilled the moment the population stops being dead — observed firing during the perturbation below. `#[cfg(test)]` on the module was rejected as the alternative because rustdoc never compiles a test-only module, which would take the compile evidence with it.

### Unreachability evidence, and the perturbations that showed it can fail

Three E-code-pinned `compile_fail` doctests in the module documentation, collected and run by `cargo test -p tiler-build --doc` despite the module being private (6 doctests total in the crate, 3 of them these):

- `use tiler_build::BoundMetalSubgroupDeclaration;` → `compile_fail,E0432`;
- `use tiler_build::BoundMetalSubgroupDeclarationError;` → `compile_fail,E0432`;
- `use tiler_build::metal_subgroup_declaration::BoundMetalSubgroupDeclaration;` → `compile_fail,E0603`.

Subject perturbations, each reverted, with the failure text quoted:

- **Restoring only the `pub use` does not compile at all** — `error[E0365]: BoundMetalSubgroupDeclaration is only public within the crate, and cannot be re-exported outside`, twice. The demotion is enforced at the re-export site itself, not only at the doctest.
- **Undoing the demotion (`pub` items plus the restored re-export)** reddened both E0432 cases: `Test compiled successfully, but it's marked compile_fail` at both, and the `dead_code` expectation reported `warning: this lint expectation is unfulfilled`.
- **Publishing the owning module as well (`pub mod` plus `pub` items)** reddened the E0603 case too: `Test compiled successfully, but it's marked compile_fail` on `use tiler_build::metal_subgroup_declaration::BoundMetalSubgroupDeclaration`.
- **Negative control — `pub mod` with the items left `pub(crate)`** keeps all three green, which is what establishes that the E0603 case is pinned on the *item's* visibility rather than the module's. The module documentation states it that way rather than the other way round, which an earlier draft got wrong.

### Spike disposition — recorded exception, not a weakened demotion

`spikes/target-profiles/metal-subgroup-width-route-gate` is its own workspace and a separate crate, so no crate-internal driver path can reach a `pub(crate)` item: the only ways to keep it building are a feature-gated re-export (undoes the demotion) or a second copy of the rows (mints a second authority over one retained measurement). Neither is admissible, so the exception is recorded in the spike's README under the heading `Build exception — this harness does not compile at main`: rerun from `586c508a`, the commit the retained log names; both break causes are stated there, including the pre-existing `2cb7c83c` fixture drift above; and restoring a build path at the tip is gated on `decide-the-host-evidence-to-profile-composition-model`.

The spike's frontmatter `experiment_status: "reproducible"` was **left unchanged**, deliberately and with the reason stated here rather than silently: `spikes/README.md`'s catalogue row mirrors that value, and that file is `contracts/navigation`, outside this ticket's surface. Moving both together to `blocked` is a paired edit a coordinator should schedule if the tip-of-`main` state should be what the status describes.

### Commands

Run in the worktree at the delivered commit unless noted.

- `cargo check -p tiler-build --all-targets` — clean.
- `cargo nextest run -p tiler-build` — 104 passed.
- `cargo test -p tiler-build --doc` — 6 passed, including the three new compile-fail cases.
- `cargo clippy -p tiler-build --all-targets -- -D warnings` — clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-build` and the same with `--document-private-items` — clean.
- `cargo fmt --check`; `tkt lint`; `git diff --check`; `tkt guard tkt/demote-the-m3-pro-subgroup-declaration-to-an-internal-evidence-fixture --format json`.
- `cargo check --locked` in the spike directory — fails as recorded above; its `target/` was removed afterwards.

Scope note: `research/target-profiles` was added to this ticket's scopes for the spike README edit the ticket's own close condition authorizes.
