---
id: cover-multi-position-stage-retention
title: Cover multi-position stage retention when a second artifact family is constructible
status: deferred
priority: p3
dependencies: []
related: [make-stage-retention-reachable-from-a-test, retain-succeeding-metal-stage-tool-output]
scopes: [implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, diagnostics, testing]
---
## User-visible outcome

`stage_retention`'s delivery-position component and its all-or-nothing elision branch are exercised by a test, so a selection wider than one artifact family retains one correctly-labelled run per stage per position, or says why it retained none.

## Why this is deferred rather than todo

**Fact — both uncovered behaviours are unreachable in the product today.** `stage_retention` in `crates/tiler-build/src/metal_cache.rs` loops over delivery positions and labels each run `tiler.metal.{delivery}.{tool}`; it falls back to `elided_retention` when `DebugRetention::retaining_with_stated_total` refuses, which for this producer means passing `MAX_RETAINED_RUNS` (16), i.e. nine or more delivery positions. `crates/tiler-build/src/metal_declaration.rs` exposes exactly one `BoundMetalCompileDeclaration` constructor — `first_macos_apple9` (read the `impl BoundMetalCompileDeclaration` block; `pub fn` appears four times and only that one constructs) — so every call to `accept_or_publish_metal_plan` today passes a one-element declaration slice and produces exactly one delivery position. Position 1 and the run limit are configurations the product cannot currently reach.

**Fact — position 0 is covered.** `crates/tiler-build/src/metal_plan.rs`'s `a_succeeding_stages_output_returns_from_a_validated_cache_hit`, `a_silent_stage_is_retained_as_an_empty_run`, and `a_stage_that_outwrote_the_capture_bound_states_the_total_it_had` reach `stage_retention` through the real cache path with a fake toolchain, pin both governed labels, and fail under a deliberate stage swap. What they cannot vary is the number of positions.

**Inference — testing it now would cost a public boundary for unreachable coverage.** The only way to drive `stage_retention` at more than one position without a second declaration is to call it directly, which needs a `tiler_metal_aot::record::StageOutputs` constructor — a `tiler-metal-aot` public-surface addition, and Tom's to accept. `make-stage-retention-reachable-from-a-test` built and measured exactly that (see its outcome section) and withdrew it: the surface would have bought coverage of configurations the product cannot produce, while the one gap that was real — the stated total — closed with no new surface at all. Buy the surface when the configuration exists, not before.

## Trigger

A second `BoundMetalCompileDeclaration` becomes constructible — equivalently, `ArtifactFamilySelection` fans a request out to more than one compile target through `accept_or_publish_metal_plan`. At that point a two-position selection is buildable from the existing `warning_toolchain` fixture with no new public item, and the nine-position elision case becomes a question about a real producer rather than a synthetic one.

## Closes when

A test drives `accept_or_publish_metal_plan` (or the seam beneath it) with a selection of at least two artifact families, and asserts one run per stage per position under `tiler.metal.{delivery}.{tool}`, each carrying its own position's and its own stage's bytes and stated total — failing under a deliberate transposition of two positions. If a selection wide enough to pass `MAX_RETAINED_RUNS` is reachable by then, the same test covers `elided_retention`; if it is not, that half stays deferred and says so.

## Trigger check log

- 2026-08-05 — not fired. `BoundMetalCompileDeclaration` has one constructor, so every plan resolves at one delivery position. Reproduce: `grep -n "pub fn" crates/tiler-build/src/metal_declaration.rs` inside the `impl BoundMetalCompileDeclaration` block — one of the four is a constructor (`first_macos_apple9`).
- 2026-08-09 — **not fired.** The `impl BoundMetalCompileDeclaration` block still has exactly one constructor, `first_macos_apple9`; its other public methods inspect rows, recover the target profile, or validate a compiled artifact. No second delivery position is constructible, so the real cache path still cannot reach the multi-position subject. Recheck at `impl BoundMetalCompileDeclaration` and `pub fn first_macos_apple9` in `crates/tiler-build/src/metal_declaration.rs`.
