---
id: cover-multi-position-stage-retention
title: Cover multi-position stage retention when a second artifact family is constructible
status: in-progress
priority: p3
dependencies: []
related: [make-stage-retention-reachable-from-a-test, retain-succeeding-metal-stage-tool-output, carry-one-payload-per-artifact-family-in-one-envelope]
scopes: [implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, diagnostics, testing]
claimed_from: todo
assignee: terra-multi-position-retention
lease_expires_at: 1786410519
---
## User-visible outcome

`stage_retention`'s delivery-position component and its all-or-nothing elision branch are exercised by a test, so a selection wider than one artifact family retains one correctly-labelled run per stage per position, or says why it retained none.

## Why this is open rather than deferred

**Correction — 2026-08-10.** Status moves `deferred` → `todo`. The Trigger's first clause ("A second `BoundMetalCompileDeclaration` becomes constructible") and its operational equivalent ("two-position selection is buildable from the existing `warning_toolchain` fixture with no new public item") both hold at this tree: `#[cfg(test)] pub(crate) fn second_artifact_family_fixture` constructs a second bound declaration, and `metal_plan` tests already pass two-element slices into `accept_or_publish_metal_plan` and assert `delivery_positions() == 2`. The 2026-08-05 and 2026-08-09 "not fired" log entries used a `pub fn`-only census that missed that cfg(test) constructor. Close condition is still unmet: no multi-position *retention* test asserts `tiler.metal.{0,1}.{metal,metallib}` labels, bytes, or stated totals under deliberate position transposition. Nine-or-more-position elision remains without a measured multi-family product producer; `Closes when` already permits leaving that half deferred with an explicit note.

**Fact (filing-time, partly false live) — both uncovered behaviours were framed as unreachable.** `stage_retention` in `crates/tiler-build/src/metal_cache.rs` loops over delivery positions and labels each run `tiler.metal.{delivery}.{tool}`; it falls back to `elided_retention` when `DebugRetention::retaining_with_stated_total` refuses, which for this producer means passing `MAX_RETAINED_RUNS` (16), i.e. nine or more delivery positions. The public non-test constructor remains only `first_macos_apple9` (`impl BoundMetalCompileDeclaration`; bare `pub fn` census still shows four methods and one constructor). Production and consumer paths (`tiler-macros` deliver, conformance publication, serial-sum-compile) still pass a one-element declaration slice. **Live correction:** under `cfg(test)`, `second_artifact_family_fixture` is a second constructor; position 1 is constructible on the real cache/plan path with no new public item. The nine-position run-limit configuration remains product-unreachable. Multi-position *retention* coverage — not multi-position constructibility — is the remaining gap.

**Fact — position 0 is covered.** `crates/tiler-build/src/metal_plan.rs`'s `a_succeeding_stages_output_returns_from_a_validated_cache_hit`, `a_silent_stage_is_retained_as_an_empty_run`, and `a_stage_that_outwrote_the_capture_bound_states_the_total_it_had` reach `stage_retention` through the real cache path with a fake toolchain, pin both governed labels under `tiler.metal.0.{metal,metallib}`, and fail under a deliberate stage swap. What they do not vary is the number of positions; no metal_plan retention test names `tiler.metal.1.*`.

**Inference (retired as a deferral reason — 2026-08-10).** The filing-time claim that multi-position coverage would cost a public `tiler_metal_aot::record::StageOutputs` constructor for unreachable configurations is obsolete: the two-position configuration exists for tests. `make-stage-retention-reachable-from-a-test` built and measured that surface for the *position-0 stated total* gap and withdrew it correctly; that withdrawal no longer justifies parking multi-position retention. Remaining work is an in-crate test composing `second_artifact_family_fixture` + `warning_toolchain` through `accept_or_publish_metal_plan` — no new public item.

## Trigger

A second `BoundMetalCompileDeclaration` becomes constructible — equivalently, `ArtifactFamilySelection` fans a request out to more than one compile target through `accept_or_publish_metal_plan`. **Correction — 2026-08-10.** The test-only half of this trigger has fired via `second_artifact_family_fixture`; product multi-family delivery remains behind `deliver-several-artifact-families-from-one-expansion` / `first-authoritative-ios-metal-compile-declaration` and is not required to write the two-position retention test this ticket authorizes. A two-position selection is buildable from the existing `warning_toolchain` fixture with no new public item. The nine-position elision case stays a question about a real multi-family producer rather than a synthetic one, and `Closes when` still allows leaving that half deferred with an explicit note if it is not reachable when the two-position retention test lands.

## Closes when

A test drives `accept_or_publish_metal_plan` (or the seam beneath it) with a selection of at least two artifact families, and asserts one run per stage per position under `tiler.metal.{delivery}.{tool}`, each carrying its own position's and its own stage's bytes and stated total — failing under a deliberate transposition of two positions. If a selection wide enough to pass `MAX_RETAINED_RUNS` is reachable by then, the same test covers `elided_retention`; if it is not, that half stays deferred and says so.

## Trigger check log

- 2026-08-05 — not fired. `BoundMetalCompileDeclaration` has one constructor, so every plan resolves at one delivery position. Reproduce: `grep -n "pub fn" crates/tiler-build/src/metal_declaration.rs` inside the `impl BoundMetalCompileDeclaration` block — one of the four is a constructor (`first_macos_apple9`).
- 2026-08-09 — **not fired.** The `impl BoundMetalCompileDeclaration` block still has exactly one constructor, `first_macos_apple9`; its other public methods inspect rows, recover the target profile, or validate a compiled artifact. No second delivery position is constructible, so the real cache path still cannot reach the multi-position subject. Recheck at `impl BoundMetalCompileDeclaration` and `pub fn first_macos_apple9` in `crates/tiler-build/src/metal_declaration.rs`.
- 2026-08-10 — **fired.** `#[cfg(test)] pub(crate) fn second_artifact_family_fixture` is a second constructor on `BoundMetalCompileDeclaration`; `one_envelope_carries_one_payload_per_artifact_family` and `a_payload_at_another_familys_delivery_position_is_refused` already drive `accept_or_publish_metal_plan` with two declarations and assert `delivery_positions() == 2` on the real plan/cache path. The 2026-08-09 entry used a `pub fn`-only census that missed the cfg(test) constructor. Multi-position *retention* assertions (`tiler.metal.1.*`) remain absent — remaining work, not a trigger miss. Nine-position elision half still not product-reachable. Reproduce: `rg -n 'second_artifact_family_fixture' crates/tiler-build/src/metal_declaration.rs crates/tiler-build/src/metal_plan.rs`; `rg -n 'delivery_positions\(\), 2' crates/tiler-build/src/metal_plan.rs`; `rg -n 'tiler\.metal\.1\.(metal|metallib)' crates/tiler-build/` (no metal_plan retention hit).
