---
id: correct-the-stale-fallbackonly-claims-in-tiler-macros-family-cfg
title: Correct the stale FallbackOnly claims in tiler-macros family_cfg
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## Two comments in `crates/tiler-macros/src/family_cfg.rs` assert an absence the crate refutes

Found while correcting `docs/roadmap.md`'s Milestone 0B text at base `44a85cfc`. Both sites are comments, so nothing fails; the claim is simply false.

**Fact — as found at base `209013bd`, corrected on this ticket's branch. The quoted text no longer occurs in the file, so it is quoted here rather than pinned as a citation.** The module doc stated: "A region can state a selected family since Tom accepted the `deliver` statement, but no expansion compiles one — the statement is refused before emission — so every expansion delivers `FallbackOnly`, which ADR 0053 defines as invoking no backend compiler."

**Fact — as found at base `209013bd`, corrected on this ticket's branch.** The `#[allow(dead_code, reason = ...)]` on `MAP_VERSION` restated it: "Nothing reads it during an expansion yet because every expansion delivers `FallbackOnly` — a stated selected family is refused before emission, since nothing compiles one".

**Fact — both are refuted by the same crate.** `crates/tiler-macros/src/aot.rs "pub(crate) fn deliver"` compiles a stated `macos` selection through `accept_or_publish_metal_plan`, and `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs "The complete inline AOT workflow, in an ordinary consumer crate"` is an out-of-tree consumer crate whose compilation runs it. `consumer_cfg` is reached from `crate::delivery`'s `items_source` for every selected family, so an expansion that delivers does embed a predicate from this table.

**Inference — the narrow claim underneath may still hold and should be checked rather than assumed.** `MAP_VERSION` itself appears to be read by nothing outside this module's tests, so the `dead_code` allowance may be correct while its stated reason is not. Verify before editing: the fix may be to correct the reason rather than to remove the allowance.

## Why this is a separate ticket

The roadmap correction that found it is scoped `contracts/navigation` and may not touch `crates/**`. AGENTS.md treats comments and examples as claims about current behaviour, which is what makes this a defect rather than untidiness.

## Closes when

Both sites state what the crate does at this base, the `dead_code` allowance is either justified by a true reason or removed on evidence, and the package's Clippy, rustdoc, and tests pass.

## Outcome

**Fact — the allowance survives; its stated reason does not.** The two were checked separately, as the ticket's Inference asked.

*The allowance is still needed.* Deleting the `#[allow(dead_code, …)]` and running `cargo check -p tiler-macros` produces `warning: constant MAP_VERSION is never used` against `crates/tiler-macros/src/family_cfg.rs`, and the same warning under `--all-targets`: the constant is read only by `family_cfg::tests`, so the non-test lib build has no reader. The workspace denies warnings in its Clippy gate, so the allowance is load-bearing rather than decorative. It is kept.

*Its reason was false in its middle clause and falsified in its last.* "Nothing reads it during an expansion yet" is true. "because every expansion delivers `FallbackOnly` … so no expansion embeds a predicate at all" is false: a delivering expansion embeds two predicates from this table. "The slice that first compiles a selected family is what makes it an identity input" was a prediction, and the slice has landed without making it one.

**Fact — what expansions actually deliver, by branch.** `crate::expand` branches on `ArtifactFamilySelection::invokes_backend_compiler`:

- A region stating no `deliver` statement, or `deliver fallback-only;`, resolves to `FallbackOnly`, takes `delivery::fallback_plan`, and reaches no backend compiler. `items_source` emits nothing for it, so it embeds no predicate. This is the only case the old text described, and it is a minority of the branches rather than all of them.
- A region stating `deliver macos;` reaches `crate::aot::deliver`, which runs the offline Metal driver through `accept_or_publish_metal_plan` and returns a `Payload` delivery. `DeliveryPlan::items_source` then emits `#[cfg(all(target_os = "macos", target_abi = ""))]` on the payload index plus a `not(any(…))` catch-all — both rendered by `consumer_cfg`. So an expansion does compile a selected family, and does embed this table.
- A family the one bound declaration does not measure is refused in `crate::aot::require_buildable`, not "before emission" in general. The check is equality on family, deployment minimum, and MSL version against `BoundMetalCompileDeclaration::first_macos_apple9`'s `aot_target`, so `deliver ios;` and `deliver macos-and-ios;` refuse while `deliver macos;` builds.
- A family-scoped toolchain failure retains a `compile_error!` under that family's predicate, which is a *second* way an expansion embeds this table.

**Fact — `MAP_VERSION` is not an identity input, but not for the reason given.** The compilation identity is a function of what `crate::aot::deliver` hands `accept_or_publish_metal_plan` — program, selected plan, bound declaration, optimization level, and toolchain fingerprint. This map is not among them because it renders the consumer `#[cfg]` that selects a payload once the bytes exist. The superseded ground, "the frontend computes no artifact identity", no longer holds either: `RouteFacts::artifact_identity` reads `artifact.canonical_identity()` and embeds it.

**Fact — one sibling carried the identical falsified premise, and one stale link was found in passing.** Both are in `crates/tiler-macros/**`, inside this ticket's declared scope:

- The `dead_code` reason on `BoundRegion::environment` said "no expansion builds an index region yet, because every region states `FallbackOnly` and invokes no backend compiler". The conclusion survives — nothing in the crate calls `IndexRegionBuilder::new_with_shape_environment`, checked by grep over `crates/tiler-macros/` — but the ground is the same false premise. The reason now cites the absent call rather than the policy.
- `crates/tiler-macros/src/delivery.rs` linked `crate::aot::delivered_plan`, which exists nowhere in `crates/`. The function is `crate::aot::deliver`.

**Files changed.** `crates/tiler-macros/src/family_cfg.rs` (module doc and the `MAP_VERSION` allowance reason), `crates/tiler-macros/src/binding.rs` (the `environment` allowance reason), `crates/tiler-macros/src/delivery.rs` (the stale doc link), and this ticket.

`shared_scopes` gained `project/tickets` because correcting the file invalidated this ticket's own pinned anchor and `make citations` failed on it; recording the outcome needs the same scope.

**Measurement — commands run in the ticket worktree with `CARGO_TARGET_DIR=./target`.** `cargo fmt --check`, `git diff --check`, `cargo check --workspace --all-targets`, `cargo clippy -p tiler-macros -p tiler --all-targets -- -D warnings`, and `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-macros -p tiler` all exit 0. `cargo nextest run -p tiler-macros -p tiler` reports 219 passed, 1 skipped; the skip is `delivery::tests::every_emitted_shape_compiles_as_the_five_target_matrix_says`, `#[ignore]`d at this base for cross-target `rust-std` components `deps.sh` does not install. `cargo test --workspace --doc` passes. `tkt lint` reports no problems. The delivering claim above is exercised rather than only read: `tiler::facade facade_reexport_contract` compiles `tests/facade/pass/deliver_compiles_embeds_and_routes.rs` as an out-of-tree crate, and it passed in that run.

**Boundary.** This corrects comments only; no behaviour changed, so no test was added or removed and the facade fixture counts floored in the `Makefile` are untouched at 10 pass and 9 fail. The claims are scoped to this base — they describe the one measured macOS declaration, and a second measured declaration would widen which families build.

## Outcome — done, 2026-08-08

Landed at merge **`e7dcc4af`** (worker commit `f87927e5`). `make full` exit 0, 1,091 release tests.

### The allowance and its reason were tested separately, and the answers differed

**The allowance is still needed** — established by deleting it and running `cargo check -p tiler-macros`, which reports `constant MAP_VERSION is never used`, same under `--all-targets`. Only the test module reads it, so the non-test lib build has no reader and Clippy denies warnings in the gate. Restored from backup.

**Its reason was false.** Split three ways on audit: "nothing reads it during an expansion yet" is **true**; "so no expansion embeds a predicate at all" is **false**; and "the slice that first compiles a selected family is what makes it an identity input" is a **falsified prediction** — the slice landed and did not make it one.

That is the distinction the filer flagged and it held: a dead-code allowance can be correct while its justification has rotted, and deleting on the strength of the rotted reason would have broken the build.

### Four branches, not one

`FallbackOnly` where there is no `deliver` statement, no backend compiler, or no predicate embedded; `deliver macos;` compiles and embeds `all(target_os = "macos", target_abi = "")` plus a `not(any(…))` catch-all; an unmeasured family is refused by equality against `first_macos_apple9`, so `deliver ios;` refuses while `deliver macos;` builds; and a family-scoped toolchain failure embeds the predicate a **second** way, on a retained `compile_error!`.

The adjacent ground "the frontend computes no artifact identity" was also corrected — `RouteFacts` embeds `artifact.canonical_identity()`. `MAP_VERSION` is still not an identity input, but **because the map renders the `#[cfg]` after the bytes exist**, not because no identity exists.

### Two siblings, same trap

`binding.rs`'s `dead_code` reason on `BoundRegion::environment` rested on the identical false premise — re-grounded on the absent call rather than deleted, since nothing calls the constructor. And `delivery.rs` linked `crate::aot::delivered_plan`, which **exists nowhere in `crates/`**; the function is `deliver`.

### `make citations` caught the ticket citing its own deleted text

The ticket's pinned anchor quoted the passage being removed, so the check failed. Re-anchored as a plain quote labelled "as found at base `209013bd`" — the convention for retired extents. A neat demonstration that the checker covers ticket citations of code the ticket itself is changing.

Fifth ticket this week whose `shared_scopes` was `[]` against a brief granting it.
