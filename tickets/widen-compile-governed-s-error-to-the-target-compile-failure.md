---
id: widen-compile-governed-s-error-to-the-target-compile-failure
title: Widen compile_governed's error to the target compile failure
status: done
priority: p2
dependencies: []
related: [accept-the-public-compiler-facade-boundary]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`session::compile_governed` stops silently discarding the typed refusal detail the general path retains, so a caller diagnosing a refusal through the convenience entry reads the same explanation as through `session::compile` — and the one facade item Tom excluded from the 2026-08-05 acceptance returns as a small delta.

## Why this exists

**Fact.** `compile_governed` ends in `.map_err(|failure| failure.failure)`, unwrapping the single-target wrapper (legitimate) but dropping `TargetCompileFailure::refusal` — the typed pre-trace detail naming which rule refused and why — with a doc comment that says nothing about the loss. **Fact.** The loss is reachable today: `NumericalContract::STRICT_BF16` against the governed profile produces exactly such a refusal, and `crates/tiler-compiler/tests/bf16_numerical_contract.rs` asserts the detail's content through the general path. **Fact — symptom, not design.** Both real producers migrated off this entry with fail-closed guards against returning; its remaining callers are three test call sites plus spike workspaces; deliberate narrowings in this tree state their reason and this one is silent. The facade acceptance packet (in `accept-the-public-compiler-facade-boundary`) carries the full derivation, including decline-entirely as the recorded second answer — this ticket takes the widening because spikes genuinely want a no-ceremony governed entry and the fix is one error-type change.

## What this ticket owes

- The error type widened so the refusal detail survives — the packet's named shape is returning `TargetCompileFailure` (or an equivalent that preserves `refusal`); run the small elimination at the code and state it.
- The doc comment rewritten to describe current behaviour, including the single-target unwrapping that *is* deliberate.
- The three in-crate test call sites updated; spike callers are other tickets' scopes — if the signature change breaks a spike, note it for the spike-restoration thread rather than editing spikes here.
- Watched-failing evidence: a STRICT_BF16 refusal through `compile_governed` carrying the same typed detail the general path reports, with the assertion observed failing against the pre-fix shape.
- Report the surface delta so the coordinator appends it to the facade acceptance as the excluded item's return.

## Non-goals

Any pipeline change (the two entries run identical compilation); removing the entry (recorded as the defensible second answer, not taken); accepting anything (the delta returns to Tom).

## Closes when

The refusal detail survives the convenience entry with the watched-failing evidence, the doc describes the behaviour, no caller sees a silent loss, and the delta is reported for acceptance.

## Outcome — 2026-08-05, at base `de377fb1`

### The shape elimination

**Fact — the numbers the elimination runs on, measured at this base.** `CompileFailure` is 80 bytes, `TargetCompileRefusal` 128, `TargetCompileFailure` 208 with the refusal inline and 88 with it boxed. Measured by a temporary `size_of` probe in `crates/tiler-compiler/tests/`, run and deleted.

Three shapes were tested; two are discarded on the record so the survivor can be refuted rather than only read.

- **Add `refusal` to `CompileFailure` itself.** Discarded on correctness. `CompileFailure` is what [`compile`] returns for a failure that loses *every* requested target, and `TargetCompileFailure` is what one slot's own refusal is reported in. Putting target-local detail on the coordination type collapses that distinction for the general path in order to fix the convenience one, which is a wider change in the wrong direction.
- **A bespoke two-variant error** (`Coordination(CompileFailure)` / `Target(TargetCompileFailure)`). Discarded on maintainability and on its own premise. It preserves a distinction that has nothing to distinguish here: the composed request names exactly one target, so a coordination failure and that target's refusal both mean "the governed compilation did not happen", and the actionable axis — which boundary refused — is `CompileFailureClass`, which both carry. Its cost is a new public type plus variants plus accessors on a facade Tom is mid-acceptance on, and it forces every caller to match before reading `class()`, making the convenience entry *less* convenient than the general one it exists to shortcut.
- **Return `TargetCompileFailure`** — the packet's named shape. **Survives.** The refusal reaches the caller whole; `class()` and `explain()` are unchanged; `Error::source` still yields the inner `CompileFailure`, so nothing is lost in the other direction. A pre-slot failure is reported in the same type with `refusal()` returning `None`, which is honest rather than lossy: the typed detail is minted from a per-target rejection and there is no rejected target to mint one from.

**One thing the elimination did not predict, found by running the check.** At 208 bytes the widened signature trips `clippy::result_large_err` (deny-by-default under the workspace `-D warnings`), at `compile_governed` and at the four `hot_path.rs` closures that return its `Result`. The fix is not an `#[allow]` — boxing `TargetCompileFailure::refusal` takes the type to 88 bytes and shrinks it for every batch slot as well, and `refusal()` keeps its exact signature including `const`, because a `match` on `&Option<Box<_>>` deref-coerces in a const context where `Option::as_deref` would not.

### Watched-failing evidence

Two forms, both observed, because they fail for different reasons.

1. **Behavioural, at the post-fix type.** `compile_governed`'s tail perturbed back to the pre-fix projection — `outcome.map_err(|failure| TargetCompileFailure::before_any_target(failure.failure))`, which is exactly what `.map_err(|failure| failure.failure)` did, re-wrapped so the signature still compiles. `the_governed_convenience_entry_carries_the_same_typed_refusal_as_the_general_path` then fails at `bf16_numerical_contract.rs:732` on `the convenience entry retains the pre-trace refusal detail`. Reverted; the test passes.
2. **Type-level, at the pre-fix signature.** With `crates/tiler-compiler/src/session.rs` restored to its `de377fb1` content and the new test in place, `cargo check -p tiler-compiler --tests` fails with three `error[E0599]: no method named 'refusal' found for struct 'CompileFailure'`. The pre-fix return type cannot express the assertion at all.

**Measurement — what the governed profile actually refuses.** A pure-BF16 program under `NumericalContract::STRICT_BF16` against `TargetProfile::governed()` is a target-local `NoFeasiblePlan` carrying `TargetCompileRefusal::DTypeDispatch { target_profile: "tiler.prototype-target-neutral-baseline.v1", resolved_type: tiler::bf16@1, disposition: Unknown }`. The packet named `DTypeNotDispatchable` as one of two candidates and this is the one that fires; the test asserts every field of it and asserts the convenience path's refusal equals the general path's.

### Surface delta, for the acceptance node

One item: `session::compile_governed`'s return type moves from `Result<Compilation, CompileFailure>` to `Result<Compilation, TargetCompileFailure>`. Nothing else on the public surface moves — no type is added, removed, or renamed, no signature but this one changes, and `TargetCompileFailure` was already in the transitively accepted set as `TargetCompilationResult::outcome`'s error. The internal boxing of its private `refusal` field is invisible at the boundary: `refusal()` still returns `Option<&TargetCompileRefusal>` and is still `const`, and `Clone`/`Debug`/`Eq`/`PartialEq` are unchanged in behaviour.

### Callers

**Fact — no call site needed an edit, in this workspace or in the spikes.** `cargo check --workspace --all-targets` passes with the signature change and zero call-site edits. The ticket anticipated three in-crate test call sites; the correct count is zero, and that is a fact about the call sites rather than about the change: not one of them names the error type. Reproduce with `grep -rn "compile_governed" crates prototypes spikes --include "*.rs"`, which returns **45** lines on this branch — 1 definition, 9 imports, 9 lines of prose, and **26** calls: 9 in `session.rs`'s own test module, 8 in `hot_path.rs`, 1 the new test in `bf16_numerical_contract.rs`, 3 in `tiler-build`, 5 in spikes. All 26 were read. Each `.expect()`s, `.unwrap()`s, discards with `let _`, reads `.class()` off the error, or hands the whole `Result` to a generic observer — every one source-compatible with a wider error type. (At `de377fb1` the population is 41: this branch adds one call, one import, and two prose lines, all in `bf16_numerical_contract.rs`.)

**Spike-caller note, for the spike threads.** The four spike call sites — `spikes/cache/hot-path-efficiency/harness/src/envelope.rs:88`, `spikes/cache/envelope-digest-coverage/harness/src/envelope.rs:87`, `spikes/cache/build-tool-exercise/envelope/src/lib.rs:63`, and `spikes/extensions/forkless-physical-provider/probe/tests/composition.rs:54,89` — are all `compile_governed(..).expect(..)` and were read rather than assumed; none names the error type, so none breaks. No spike was edited. `spikes/target-profiles/scalar-cpu-vertical/src/vertical.rs` does name `CompileFailure` (`VerticalError::Compile(Box<CompileFailure>)`, constructed at `:866`) but from `compile`, not `compile_governed`, and `compile`'s error type did not move.

### Not done here, deliberately

The disclosure sites that call this item an exclusion — `docs/correctness-and-testing.md`, `docs/status.md`, `docs/architecture.md` — are `contracts/*` scopes, not this ticket's, and they are correct until Tom accepts the delta. They move with the acceptance, not with the code.
