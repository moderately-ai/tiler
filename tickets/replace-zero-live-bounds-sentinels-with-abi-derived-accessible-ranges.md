---
id: replace-zero-live-bounds-sentinels-with-abi-derived-accessible-ranges
title: Replace zero live-bounds sentinels with ABI-derived accessible ranges
status: in-progress
priority: p0
dependencies: [associate-live-extent-operands-with-symbolic-semantic-interface-axes]
related: [bind-repeated-invocations-over-caller-retained-tensors, prove-one-live-extent-artifact-payload-and-pipeline-at-two-n]
scopes: [implementation/ir, implementation/artifact, implementation/runtime, implementation/build, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, runtime, abi, correctness]
claimed_from: todo
assignee: worker-live-bounds
lease_expires_at: 1787159586
---
## User-visible outcome

A backend receives the routed accessible span derived from the live ABI fact. It neither receives a zero sentinel nor reconstructs reach from private row/column state, so a longer allocation cannot silently change the payload's address meaning.

## Exact gap and per-Fact audit at `f3e1efd3b3b4f896976b326e6a3d993147206cd3`

- **Verified.** `crates/tiler-ir/src/kernel/verify.rs` `access_elements` returns zero for live row-major and live contraction accesses; the schedule proof therefore carries a zero-length linear range rather than a dynamic witness.
- **Verified.** `crates/tiler-runtime/tests/adapter_route/fixture.rs` `live_extent_program` constructs zero-length byte windows and `let accessible = CheckedMultiply(zero, live_n)`. `one_live_extent_artifact_indexes_dense_f32_at_two_n_without_baking` explicitly asserts `accessible_bytes == Unsigned(0)` while claiming the range uses `N`.
- **Verified.** The scalar adapter's `plan_dispatch` replaces zero with `prepared.rows * parameter.value() * 4`; `ScalarImage::entry_for` replaces the routed read/write reach with locally computed values, so its supposed comparison is self-comparison. A conforming adapter that honors `RoutedBinding::accessible_bytes()` receives zero.
- **Verified.** Existing wrong-stride examples calculate helper values but do not perturb the routed range. They do not prove the route's authority is load-bearing.

Reproduce the wrong-positive with `cargo test -p tiler-artifact --lib one_live_extent_artifact_indexes_dense_f32_at_two_n_without_baking -- --nocapture`; the passing test asserts the zero range.

## Required work

- Replace the zero placeholder with a typed dynamic bounds/range witness derived from the artifact's existing `AbiRoot::InputExtent` authority. For the dense F32 `[rows,N]` worked instance, publish exactly `rows * N * 4`, with checked arithmetic.
- Make the runtime adapter consume the routed offset and accessible bytes unchanged when validating storage and preparing work. Remove `prepared.rows`/allocation-length reconstruction and live-path self-comparisons.
- Preserve the distinction between exact live reach and allocation capacity: longer storage is legal only when the published range fits; it does not redefine stride or reach.
- Refuse an unrepresentable range, an unbound fact, a zero sentinel on a live access, and any disagreement before routing commit. Do not default or repair malformed artifact data in the adapter.

## Required evidence

- The routed range moves at two neighbouring extents and the backend-observed range is byte-for-byte the routed value.
- Independently perturb the ABI fact, encoded range expression, routed range, and adapter's local dimensions. Each unchanged assertion must fail with quoted text.
- The retained longer-allocation wrong-stride oracle fails when capacity is substituted for live reach.
- Targeted IR/artifact/build/runtime tests, Clippy, rustdoc, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Binding the scalar parameter bytes to their backend transport is [`bind-frozen-live-extent-bytes-at-declared-backend-transports`](bind-frozen-live-extent-bytes-at-declared-backend-transports.md). Associating the operand with symbolic semantic meaning is the prerequisite [`associate-live-extent-operands-with-symbolic-semantic-interface-axes`](associate-live-extent-operands-with-symbolic-semantic-interface-axes.md).

## Closes when

The artifact route is the sole authority for accessible offset/range and no adapter can turn the former zero sentinel or its own allocation metadata into a second meaning of the live extent.
