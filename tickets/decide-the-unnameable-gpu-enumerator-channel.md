---
id: decide-the-unnameable-gpu-enumerator-channel
title: Add a fallible GPU-enumerator channel when a second binding needs it
status: done
priority: p3
dependencies: []
related: [close-the-serial-sum-run-gpu-family-probe-table, close-the-metal-gpu-family-out-of-crate-total-map, widen-the-metal-gpu-family-vocabulary-to-apple10]
scopes: [implementation/metal, implementation/runtime, implementation/conformance, implementation/candle, contracts/decisions, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [api-conventions, metal, adr-0074, trigger-fired]
---

## Outcome

Accepted by Tom on 2026-08-12 in the live review and implemented from exact base `2c75f0a1dfd9ebb5666d66ff3c955c03b47a5926`.

`tiler_metal::applicability` now owns a generic fallible highest-family walk:

```rust
pub fn try_observe_highest_gpu_family<E>(
    supports_family: impl FnMut(AppleGpuFamilyConstant) -> Result<bool, E>,
) -> Result<MetalGpuFamilySupport, E>
```

The first error aborts the whole highest-first walk and no lower family is queried. A binding that cannot name an enumerator therefore cannot manufacture `Highest(lower)` by treating an unasked question as `false`. The old total `observe_highest_gpu_family` entry point was removed rather than retained as a compatibility path; this is pre-production software and leaving it reachable would preserve the exact misuse the decision closes.

The two `metal` 0.33.0 consumers return `Err(AppleGpuFamilyConstant)` directly from their partial raw-value crossing. Their duplicate `ProbedGpuFamily` enums, captured side channels, and discard-after-the-walk logic were deleted. The total `objc2-metal` consumer uses `Infallible`, so it states that its transparent newtype crossing cannot fail without inventing a fourth semantic state.

This is a Rust API change only. No artifact, request, target-profile, cache, canonical identity, schema, or domain bytes move.

## Source-first Fact audit

Read at exact base `2c75f0a1dfd9ebb5666d66ff3c955c03b47a5926` before editing.

- **Verified — the old channel was total.** `crates/tiler-metal/src/applicability.rs`, anchor `pub fn observe_highest_gpu_family`, accepted `FnMut(AppleGpuFamilyConstant) -> bool` and therefore could not represent a question the binding could not ask.
- **Verified — three workspace consumers use two foreign binding shapes.** `prototypes/candle-metal-adapter/src/adapter.rs`, anchor `pub fn observed_apple_family`, crosses the raw value into `objc2_metal::MTLGPUFamily`, a transparent newtype in resolved `objc2-metal` 0.3.2. `crates/tiler-conformance/src/dispatch.rs` and `prototypes/serial-sum-run/src/proof.rs`, anchor `binding_apple_enumerator`, use `metal` 0.33.0's non-exhaustive Rust enum and cannot construct an enumerator from an arbitrary raw value. The Candle manifest requests 0.3.1 compatibly, while `Cargo.lock` resolves 0.3.2; the earlier ticket text that stated only one version without this distinction was imprecise.
- **Verified — the trigger fired.** Both `metal` consumers independently carried `binding_apple_enumerator`, a private `ProbedGpuFamily`, a captured unnameable flag, and the same whole-walk discard rule. This is the second independent instance the 2026-08-01 deferral named.
- **Verified — whole-walk invalidation is required.** The governed walk is highest first and stops at the first supported family. Continuing after an unasked higher family can report a lower family as the highest observed one, an understatement shaped like a complete observation.
- **Imprecise — the generic channel does not make `tiler-metal` the owner of every error vocabulary.** The caller owns the crossing and its concrete error; `tiler-metal` owns the ordering and propagation rule. `Result<MetalGpuFamilySupport, E>` preserves that division without a backend-specific failure enum in the neutral vocabulary.
- **Verified — no identity consequence exists.** The observation and its error are live process evidence only and enter no canonical encoder or identity derivation.
- **Repaired — scopes were underdeclared.** Claim-time scopes now cover `implementation/metal`, `implementation/runtime`, `implementation/conformance`, and `implementation/candle`, plus the shared ticket record.

## Decision and eliminated alternatives

Use generic `Result`, not `Option<bool>` and not a new public three-state enum. The present state space is already mutually exclusive and exhaustive: `Ok(true)` means the device answered yes, `Ok(false)` means it answered no, and `Err(E)` means the caller could not obtain an answer. `E` preserves the crossing's own typed cause and leaves room for a later binding to carry more detail without widening `tiler-metal` again. A named enum becomes justified only if a genuinely distinct fourth semantic state appears; adding one now would duplicate `Result` and force every consumer to translate between isomorphic vocabularies.

- **Status quo local refusal:** correct but duplicates subtle ordering logic; eliminated after the trigger produced two independent implementations.
- **Treat an unnameable enumerator as `false`:** eliminated on correctness because it turns an unasked question into a device answer and may understate the highest supported family.
- **Return `Option<bool>` from the callback:** can encode the current states but erases the reason and cannot preserve a future binding's typed crossing error.
- **Add `Unnameable` to `MetalGpuFamilySupport`:** puts a caller/binding failure inside the device-answer vocabulary and cannot represent richer crossing failures without repeated enum growth.
- **Keep the old total wrapper beside the fallible function:** eliminated because it preserves the tempting wrong path and gives future consumers two authorities for the same walk.
- **Use `unsafe` to manufacture a foreign enum:** eliminated under ADR 0079 because safe typed refusal exists and an invalid Rust enum discriminant would be undefined behavior.

## Correctness evidence

The owning unit test records the exact queries. Apple9 answers `false`, Apple8 returns `Err`, and any Apple7-or-lower query panics. The accepted implementation returns Apple8's error after exactly `[1009, 1008]`.

The subject was perturbed by replacing `?` with `unwrap_or(false)`. `cargo test -p tiler-metal a_failed_query_aborts_before_any_lower_family_is_asked` failed with:

```text
the walk continued to lower family 1007 after an error
```

The correct propagation was then restored and the test passed. This demonstrates that the check reaches the ordering rule itself rather than merely exercising an assertion.

## Trigger check log

- 2026-08-01 — **not fired.** Tom accepted deferral until a second raw-value-less consumer needed the same rule or the Apple10 vocabulary widening forced the binding mismatch.
- 2026-08-10 — **fired.** `crates/tiler-conformance` independently reproduced the `metal`-binding probe, private result enum, captured side channel, and fail-closed mappings already present in `prototypes/serial-sum-run`. The Apple10 vocabulary widening remained deferred.
- 2026-08-12 — **resolved.** Tom accepted the generic fallible channel after a fresh exact-base audit and asked for the implementation and consumer migration.

## Verification

- `cargo check -p tiler-metal -p tiler-conformance -p tiler-prototype-run -p tiler-prototype-candle --all-targets`
- `cargo test -p tiler-metal applicability_tests`
- `cargo test -p tiler-conformance unnameable_enumerator`
- `cargo test -p tiler-prototype-run unnameable_enumerator`
- `cargo test -p tiler-prototype-run a_family_row_is_unrecognized_when_the_binding_could_not_ask`
- `cargo test -p tiler-metal` — 132 unit tests plus nine doctests passed.
- `cargo test -p tiler-prototype-run --all-targets` — 46 tests passed.
- `cargo test -p tiler-prototype-candle --all-targets` — 19 tests passed.
- `cargo clippy -p tiler-metal -p tiler-conformance --all-targets -- -D warnings`
- `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-metal -p tiler-conformance --no-deps`
- `make citations` — 1,187 pinned citations and 6,484 local Markdown links resolved.
- `tkt lint --format json`
- `git diff --check`

`make full` passed citations, formatting, workspace check, and the workspace Clippy gate, then stopped in nextest at test 1,420 of 3,304 on the pre-existing host-row mismatch: the current host reports macOS build `26A5406e` while `MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9` intentionally requires retained build `26A5388g`. `serial_sum::tests::this_host_is_refused_the_right_to_offer_the_declared_profile` therefore observed `OsBuild` rather than the test's expected later `NativeTranslationAuthority`. The same unrelated environment drift was already recorded before this ticket; the exact migrated packages and all affected device-free refusal paths pass above.
